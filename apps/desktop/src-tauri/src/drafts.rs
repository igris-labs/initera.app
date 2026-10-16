use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::mutations::{self, MutationRequest};
use crate::storage::{current_workspace_id, document_data_root, open_database};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftRecord {
    key: String,
    payload: serde_json::Value,
    version: i64,
    updated_at: String,
}

fn load(connection: &Connection, key: &str) -> Result<Option<DraftRecord>, String> {
    let workspace_id = current_workspace_id(connection)?;
    connection
        .query_row(
            "SELECT draft_key, payload_json, version, updated_at FROM draft
             WHERE workspace_id = ?1 AND draft_key = ?2",
            params![workspace_id, key],
            |row| {
                let payload: String = row.get(1)?;
                Ok((row.get(0)?, payload, row.get(2)?, row.get(3)?))
            },
        )
        .optional()
        .map_err(|_| "Cannot load recovery draft.".to_owned())?
        .map(|(key, payload, version, updated_at)| {
            Ok(DraftRecord {
                key,
                payload: serde_json::from_str(&payload)
                    .map_err(|_| "Cannot read recovery draft.".to_owned())?,
                version,
                updated_at,
            })
        })
        .transpose()
}

#[api_contract_macros::command]
pub fn storage_load_draft(app: AppHandle, key: String) -> Result<Option<DraftRecord>, String> {
    if key.len() > 200 || key.is_empty() {
        return Err("Choose a valid draft key.".into());
    }
    load(&open_database(&document_data_root(&app)?)?, &key)
}

#[api_contract_macros::command]
pub fn storage_save_draft(
    app: AppHandle,
    key: String,
    payload: serde_json::Value,
    expected_version: Option<i64>,
    mutation: MutationRequest,
) -> Result<DraftRecord, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    save_draft(&mut connection, key, payload, expected_version, mutation)
}

fn save_draft(
    connection: &mut Connection,
    key: String,
    payload: serde_json::Value,
    expected_version: Option<i64>,
    mutation: MutationRequest,
) -> Result<DraftRecord, String> {
    if key.is_empty() || key.len() > 200 {
        return Err("Choose a valid draft key.".into());
    }
    let encoded =
        serde_json::to_string(&payload).map_err(|_| "Cannot encode recovery draft.".to_owned())?;
    if encoded.len() > 2_000_000 {
        return Err("Recovery draft is too large.".into());
    }
    mutations::apply(
        connection,
        "draft.save",
        &(&key, &payload, expected_version),
        &mutation,
        |tx, workspace_id| {
            let current = load(tx, &key)?;
            if current.as_ref().map(|item| item.version) != expected_version {
                return Err("CONFLICT: Recovery draft changed in another window.".into());
            }
            tx.execute(
                "INSERT INTO draft(workspace_id, draft_key, payload_json)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(workspace_id, draft_key) DO UPDATE SET
                   payload_json = excluded.payload_json,
                   version = draft.version + 1,
                   updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
                params![workspace_id, key, encoded],
            )
            .map_err(|_| "Cannot save recovery draft.".to_owned())?;
            load(tx, &key)?.ok_or_else(|| "Cannot read saved draft.".to_owned())
        },
    )
}

#[api_contract_macros::command]
pub fn storage_discard_draft(
    app: AppHandle,
    key: String,
    expected_version: Option<i64>,
    mutation: MutationRequest,
) -> Result<bool, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    discard_draft(&mut connection, key, expected_version, mutation)
}

fn discard_draft(
    connection: &mut Connection,
    key: String,
    expected_version: Option<i64>,
    mutation: MutationRequest,
) -> Result<bool, String> {
    if key.is_empty() || key.len() > 200 {
        return Err("Choose a valid draft key.".into());
    }
    mutations::apply(
        connection,
        "draft.discard",
        &(&key, expected_version),
        &mutation,
        |tx, workspace_id| {
            let current = load(tx, &key)?;
            if current.as_ref().map(|item| item.version) != expected_version {
                return Err("CONFLICT: Recovery draft changed in another window.".into());
            }
            tx.execute(
                "DELETE FROM draft WHERE workspace_id = ?1 AND draft_key = ?2",
                params![workspace_id, key],
            )
            .map_err(|_| "Cannot discard recovery draft.".to_owned())?;
            Ok(current.is_some())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{open_database, save_identity};

    fn request() -> MutationRequest {
        MutationRequest {
            mutation_id: uuid::Uuid::new_v4().to_string(),
            expected_version: None,
            expected_versions: Default::default(),
        }
    }

    #[test]
    fn draft_survives_failed_save_reopen_and_explicit_discard() {
        let root = std::env::temp_dir().join(format!("pm-app-draft-test-{}", uuid::Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
        let first_request = request();
        let first = save_draft(
            &mut connection,
            "issue:new".into(),
            serde_json::json!({"title":"Recovered"}),
            None,
            first_request.clone(),
        )
        .unwrap();
        let replay = save_draft(
            &mut connection,
            "issue:new".into(),
            serde_json::json!({"title":"Recovered"}),
            None,
            first_request,
        )
        .unwrap();
        assert_eq!(replay.version, first.version);
        assert!(save_draft(
            &mut connection,
            "issue:new".into(),
            serde_json::json!({"title":"Stale"}),
            None,
            request(),
        )
        .unwrap_err()
        .contains("CONFLICT"));
        assert!(save_draft(
            &mut connection,
            "issue:new".into(),
            serde_json::json!({"text":"x".repeat(2_000_001)}),
            Some(first.version),
            request(),
        )
        .unwrap_err()
        .contains("too large"));
        drop(connection);
        let mut reopened = open_database(&root).unwrap();
        assert_eq!(
            load(&reopened, "issue:new").unwrap().unwrap().payload["title"],
            "Recovered"
        );
        assert!(discard_draft(
            &mut reopened,
            "issue:new".into(),
            Some(first.version),
            request(),
        )
        .unwrap());
        assert!(load(&reopened, "issue:new").unwrap().is_none());
        drop(reopened);
        std::fs::remove_dir_all(root).unwrap();
    }
}
