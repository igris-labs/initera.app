use std::collections::BTreeMap;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MutationRequest {
    pub mutation_id: String,
    pub expected_version: Option<i64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub expected_versions: BTreeMap<String, i64>,
}

pub fn request_hash<T: Serialize>(operation: &str, payload: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(&(operation, payload))
        .map_err(|_| "Cannot encode mutation request.".to_owned())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub fn replay<T: DeserializeOwned>(
    tx: &Connection,
    workspace_id: &str,
    operation: &str,
    request_hash: &str,
    request: &MutationRequest,
) -> Result<Option<T>, String> {
    // The same ID may replay only the same operation, payload, and workspace. This makes a
    // retry safe when SQLite committed but the renderer never received its response.
    Uuid::parse_str(&request.mutation_id).map_err(|_| "Mutation ID must be a UUID.".to_owned())?;
    let previous: Option<(String, String, String)> = tx
        .query_row(
            "SELECT workspace_id, operation || ':' || request_hash, result_json
             FROM mutation_receipt WHERE mutation_id = ?1",
            [&request.mutation_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| "Cannot read mutation receipt.".to_owned())?;
    let Some((previous_workspace, previous_request, result_json)) = previous else {
        return Ok(None);
    };
    if previous_workspace != workspace_id
        || previous_request != format!("{operation}:{request_hash}")
    {
        return Err("IDEMPOTENCY_CONFLICT: Mutation ID was used for another request.".into());
    }
    serde_json::from_str(&result_json)
        .map(Some)
        .map_err(|_| "Cannot read mutation result.".to_owned())
}

pub fn check_version(
    tx: &Connection,
    table: &str,
    entity_id: &str,
    request: &MutationRequest,
) -> Result<(), String> {
    let expected = (if request.expected_versions.is_empty() {
        request.expected_version
    } else {
        request.expected_versions.get(entity_id).copied()
    })
    .filter(|value| *value > 0)
    .ok_or_else(|| "CONFLICT: An expected entity version is required.".to_owned())?;
    if !matches!(
        table,
        "issue" | "project" | "saved_view" | "label" | "time_entry"
    ) {
        return Err("Unsupported versioned entity.".into());
    }
    let current: i64 = tx
        .query_row(
            &format!("SELECT version FROM {table} WHERE id = ?1"),
            [entity_id],
            |row| row.get(0),
        )
        .map_err(|_| "Entity no longer exists.".to_owned())?;
    if current != expected {
        return Err(format!(
            "CONFLICT: Entity changed. Current version is {current}."
        ));
    }
    Ok(())
}

pub fn apply<T, P, F>(
    connection: &mut Connection,
    operation: &str,
    payload: &P,
    request: &MutationRequest,
    action: F,
) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
    P: Serialize,
    F: FnOnce(&Transaction<'_>, &str) -> Result<T, String>,
{
    // Persist the result with the domain write so a crash cannot leave one without the other.
    let workspace_id = crate::storage::current_workspace_id(connection)?;
    let hash = request_hash(operation, &(payload, request.expected_version))?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin mutation.".to_owned())?;
    if let Some(result) = replay(&tx, &workspace_id, operation, &hash, request)? {
        return Ok(result);
    }
    let result = action(&tx, &workspace_id)?;
    save(&tx, &workspace_id, operation, &hash, request, &result)?;
    tx.commit()
        .map_err(|_| "Cannot commit mutation.".to_owned())?;
    Ok(result)
}

pub fn save<T: Serialize>(
    tx: &Connection,
    workspace_id: &str,
    operation: &str,
    request_hash: &str,
    request: &MutationRequest,
    result: &T,
) -> Result<(), String> {
    let result_json =
        serde_json::to_string(result).map_err(|_| "Cannot encode mutation receipt.".to_owned())?;
    let effects_json = crate::api_contract::captured_effects()
        .map(|effects| serde_json::to_string(&effects))
        .transpose()
        .map_err(|_| "Cannot encode mutation effects.".to_owned())?;
    tx.execute(
        "INSERT INTO mutation_receipt(mutation_id, workspace_id, operation, request_hash,
                                      result_json, workspace_revision, effects_json)
         VALUES (?1, ?2, ?3, ?4, ?5, (SELECT revision FROM workspace WHERE id = ?2), ?6)",
        params![
            request.mutation_id,
            workspace_id,
            operation,
            request_hash,
            result_json,
            effects_json,
        ],
    )
    .map_err(|_| "Cannot save mutation receipt.".to_owned())?;
    Ok(())
}

// Some native operations use nested savepoints while reconciling cycles. Keep the
// receipt in the same outer transaction without borrowing the connection for the
// lifetime of a rusqlite Transaction.
pub fn apply_with_connection<T, P, F>(
    connection: &mut Connection,
    operation: &str,
    payload: &P,
    request: &MutationRequest,
    action: F,
) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
    P: Serialize,
    F: FnOnce(&mut Connection, &str) -> Result<T, String>,
{
    let workspace_id = crate::storage::current_workspace_id(connection)?;
    let hash = request_hash(operation, &(payload, request.expected_version))?;
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|_| "Cannot begin mutation.".to_owned())?;
    let result = (|| {
        if let Some(result) = replay(connection, &workspace_id, operation, &hash, request)? {
            return Ok(result);
        }
        let result = action(connection, &workspace_id)?;
        save(
            connection,
            &workspace_id,
            operation,
            &hash,
            request,
            &result,
        )?;
        Ok(result)
    })();
    match result {
        Ok(value) => {
            if connection.execute_batch("COMMIT").is_err() {
                let _ = connection.execute_batch("ROLLBACK");
                return Err("Cannot commit mutation.".into());
            }
            Ok(value)
        }
        Err(error) => {
            let _ = connection.execute_batch("ROLLBACK");
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{open_database, save_identity};

    #[test]
    fn connection_mutation_rolls_back_data_and_receipt_on_failure() {
        let root =
            std::env::temp_dir().join(format!("pm-app-connection-mutation-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
        let request = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: None,
            expected_versions: Default::default(),
        };
        let result: Result<(), String> = apply_with_connection(
            &mut connection,
            "test.rollback",
            &"project",
            &request,
            |connection, workspace_id| {
                connection.execute(
                    "INSERT INTO project(id, workspace_id, name, identifier_prefix) VALUES (?1, ?2, 'Transient', 'TMP')",
                    params![Uuid::new_v4().to_string(), workspace_id],
                ).map_err(|_| "Cannot insert project.".to_owned())?;
                Err("Injected failure".into())
            },
        );
        assert_eq!(result.unwrap_err(), "Injected failure");
        let projects: i64 = connection
            .query_row("SELECT count(*) FROM project", [], |row| row.get(0))
            .unwrap();
        let receipts: i64 = connection
            .query_row("SELECT count(*) FROM mutation_receipt", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!((projects, receipts), (0, 0));
        drop(connection);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn version_map_is_authoritative_for_native_updates() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("CREATE TABLE project (id TEXT PRIMARY KEY, version INTEGER);")
            .unwrap();
        let id = Uuid::new_v4().to_string();
        connection
            .execute("INSERT INTO project (id, version) VALUES (?1, 2)", [&id])
            .unwrap();
        let mut request = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: Some(1),
            expected_versions: BTreeMap::from([(id.clone(), 2)]),
        };
        assert!(check_version(&connection, "project", &id, &request).is_ok());
        request.expected_versions.clear();
        assert!(check_version(&connection, "project", &id, &request).is_err());
    }
}
