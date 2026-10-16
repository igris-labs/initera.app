use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use tauri::AppHandle;

use crate::mutations::{self, MutationRequest};
use crate::storage::{current_workspace_id, document_data_root, open_database};

const BASE_STATUSES: [&str; 6] = [
    "Backlog",
    "Todo",
    "In Progress",
    "On Hold",
    "Done",
    "Canceled",
];
const CATEGORIES: [&str; 5] = ["Backlog", "Unstarted", "Started", "Completed", "Canceled"];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStatus {
    name: String,
    category: String,
    description: String,
    icon: String,
    color: String,
    position: i64,
    archived: bool,
    is_default: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSnapshot {
    statuses: Vec<WorkflowStatus>,
    version: i64,
}

fn workflow_snapshot(connection: &Connection) -> Result<WorkflowSnapshot, String> {
    // Status order and appearance live in separate rows; hash the full snapshot as one
    // optimistic version for edits that replace the workflow together.
    let statuses = list_workflow(connection)?;
    let encoded =
        serde_json::to_vec(&statuses).map_err(|_| "Cannot read workflow version.".to_owned())?;
    let digest = Sha256::digest(encoded);
    let version = i64::from_be_bytes(digest[..8].try_into().unwrap()) & ((1_i64 << 53) - 1);
    Ok(WorkflowSnapshot { statuses, version })
}

pub(crate) fn status_available(
    connection: &Connection,
    workspace_id: &str,
    status: &str,
) -> Result<bool, String> {
    connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM workflow_status WHERE workspace_id = ?1 AND name = ?2 AND archived = 0)",
        params![workspace_id, status], |row| row.get(0),
    ).map_err(|_| "Cannot check the workflow status.".to_owned())
}

fn list_workflow(connection: &Connection) -> Result<Vec<WorkflowStatus>, String> {
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT name,category,description,icon,color,position,archived,is_default
         FROM workflow_status WHERE workspace_id = ?1 ORDER BY archived, position, name",
        )
        .map_err(|_| "Cannot read workflow statuses.".to_owned())?;
    let statuses = statement
        .query_map([workspace_id], |row| {
            Ok(WorkflowStatus {
                name: row.get(0)?,
                category: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                position: row.get(5)?,
                archived: row.get(6)?,
                is_default: row.get(7)?,
            })
        })
        .map_err(|_| "Cannot read workflow statuses.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read workflow statuses.".to_owned())?;
    Ok(statuses)
}

fn validate(statuses: &[WorkflowStatus]) -> Result<(), String> {
    let mut names = HashSet::new();
    let mut defaults = 0;
    for status in statuses {
        if status.name.trim() != status.name
            || status.name.is_empty()
            || status.name.chars().count() > 32
            || !names.insert(status.name.to_lowercase())
            || !CATEGORIES.contains(&status.category.as_str())
            || status.description.chars().count() > 500
            || !status.icon.starts_with("i-tabler-")
            || status.icon.len() > 80
            || !["gray", "blue", "purple", "green", "amber", "red"].contains(&status.color.as_str())
            || status.position < 0
            || status.position > 1000
        {
            return Err("Choose valid workflow names, categories, icons, and colors.".into());
        }
        if status.is_default {
            defaults += 1;
            if status.archived || !["Backlog", "Unstarted"].contains(&status.category.as_str()) {
                return Err("The default workflow status must be active and unstarted.".into());
            }
        }
    }
    if defaults != 1 {
        return Err("Choose one default workflow status.".into());
    }
    for category in CATEGORIES {
        if !statuses
            .iter()
            .any(|status| status.category == category && !status.archived)
        {
            return Err(format!("Keep an active status in {category}."));
        }
    }
    for base in BASE_STATUSES {
        let Some(status) = statuses.iter().find(|item| item.name == base) else {
            return Err(format!("The {base} status cannot be removed."));
        };
        if status.archived {
            return Err(format!("The {base} status cannot be archived."));
        }
        let category = match base {
            "Backlog" => "Backlog",
            "Todo" => "Unstarted",
            "In Progress" | "On Hold" => "Started",
            "Done" => "Completed",
            _ => "Canceled",
        };
        if status.category != category {
            return Err(format!("The {base} status must stay in {category}."));
        }
    }
    Ok(())
}

#[cfg(test)]
fn save_workflow(
    connection: &mut Connection,
    statuses: Vec<WorkflowStatus>,
    rename_from: Option<&str>,
    rename_to: Option<&str>,
) -> Result<Vec<WorkflowStatus>, String> {
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin workflow update.".to_owned())?;
    let saved = save_workflow_in_tx(&tx, statuses, rename_from, rename_to)?;
    tx.commit()
        .map_err(|_| "Cannot commit workflow statuses.".to_owned())?;
    Ok(saved)
}

fn save_workflow_in_tx(
    tx: &Connection,
    statuses: Vec<WorkflowStatus>,
    rename_from: Option<&str>,
    rename_to: Option<&str>,
) -> Result<Vec<WorkflowStatus>, String> {
    validate(&statuses)?;
    if rename_from.is_some() != rename_to.is_some() {
        return Err("Choose both workflow rename values.".into());
    }
    let workspace_id = current_workspace_id(tx)?;
    if let (Some(from), Some(to)) = (rename_from, rename_to) {
        // Rename issue memberships in the same transaction as the workflow definition.
        if BASE_STATUSES.contains(&from)
            || from == to
            || !statuses.iter().any(|status| status.name == to)
        {
            return Err("Choose a valid custom status rename.".into());
        }
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM workflow_status WHERE workspace_id = ?1 AND name = ?2)",
            params![workspace_id, from], |row| row.get(0),
        ).map_err(|_| "Cannot find the workflow status.".to_owned())?;
        if !exists {
            return Err("Workflow status no longer exists.".into());
        }
        let target_exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM workflow_status WHERE workspace_id = ?1 AND name = ?2)",
            params![workspace_id, to], |row| row.get(0),
        ).map_err(|_| "Cannot check the renamed workflow status.".to_owned())?;
        if target_exists {
            return Err("A workflow status with this name already exists.".into());
        }
        tx.execute(
            "INSERT INTO issue_activity(id,issue_id,actor_name,kind,body)
             SELECT lower(hex(randomblob(16))),issue.id,
                    (SELECT display_name FROM local_actor LIMIT 1),'changed',?1
             FROM issue WHERE workspace_id = ?2 AND status = ?3",
            params![
                format!("changed status from {from} to {to}"),
                workspace_id,
                from
            ],
        )
        .map_err(|_| "Cannot record workflow changes.".to_owned())?;
        tx.execute(
            "UPDATE issue SET status = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                    WHERE workspace_id = ?2 AND status = ?3",
            params![to, workspace_id, from],
        )
        .map_err(|_| "Cannot rename issue statuses.".to_owned())?;
    }
    let old: Vec<String> = {
        let mut query = tx
            .prepare("SELECT name FROM workflow_status WHERE workspace_id = ?1")
            .map_err(|_| "Cannot read workflow statuses.".to_owned())?;
        let names = query
            .query_map([&workspace_id], |row| row.get(0))
            .map_err(|_| "Cannot read workflow statuses.".to_owned())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "Cannot read workflow statuses.".to_owned())?;
        names
    };
    for name in &old {
        if Some(name.as_str()) == rename_from || statuses.iter().any(|status| status.name == *name)
        {
            continue;
        }
        let used: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM issue WHERE workspace_id = ?1 AND status = ?2)",
                params![workspace_id, name],
                |row| row.get(0),
            )
            .map_err(|_| "Cannot check issues in the workflow status.".to_owned())?;
        if used {
            return Err(format!("Move issues out of {name} before deleting it."));
        }
    }
    tx.execute(
        "DELETE FROM workflow_status WHERE workspace_id = ?1",
        [&workspace_id],
    )
    .map_err(|_| "Cannot replace workflow statuses.".to_owned())?;
    for status in &statuses {
        tx.execute(
            "INSERT INTO workflow_status(workspace_id,name,category,description,icon,color,position,archived,is_default)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![workspace_id,status.name,status.category,status.description,status.icon,
                    status.color,status.position,status.archived,status.is_default],
        ).map_err(|_| "Cannot save workflow statuses.".to_owned())?;
    }
    list_workflow(tx)
}

#[api_contract_macros::command]
pub fn storage_list_workflow(app: AppHandle) -> Result<WorkflowSnapshot, String> {
    workflow_snapshot(&open_database(&document_data_root(&app)?)?)
}

#[api_contract_macros::command]
pub fn storage_save_workflow(
    app: AppHandle,
    statuses: Vec<WorkflowStatus>,
    rename_from: Option<String>,
    rename_to: Option<String>,
    mutation: MutationRequest,
) -> Result<WorkflowSnapshot, String> {
    save_workflow_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        statuses,
        rename_from.as_deref(),
        rename_to.as_deref(),
        &mutation,
    )
}

fn save_workflow_mutation(
    connection: &mut Connection,
    statuses: Vec<WorkflowStatus>,
    rename_from: Option<&str>,
    rename_to: Option<&str>,
    mutation: &MutationRequest,
) -> Result<WorkflowSnapshot, String> {
    mutations::apply(
        connection,
        "workflow.save",
        &(&statuses, rename_from, rename_to),
        mutation,
        |tx, _| {
            let current = workflow_snapshot(tx)?;
            if mutation.expected_version != Some(current.version) {
                return Err("CONFLICT: Workflow changed. Reload statuses before saving.".into());
            }
            save_workflow_in_tx(tx, statuses.clone(), rename_from, rename_to)?;
            workflow_snapshot(tx)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{create_workspace, save_identity, switch_workspace};
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn workflow_receipt_replays_and_rejects_stale_snapshot() {
        let root = std::env::temp_dir().join(format!("pm-app-workflow-receipt-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
        let initial = workflow_snapshot(&connection).unwrap();
        let mut statuses = initial.statuses;
        statuses[0].description = "Ready for triage".into();
        let request = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: Some(initial.version),
            expected_versions: Default::default(),
        };

        let saved = save_workflow_mutation(&mut connection, statuses.clone(), None, None, &request)
            .unwrap();
        assert_ne!(saved.version, initial.version);
        let replay =
            save_workflow_mutation(&mut connection, statuses.clone(), None, None, &request)
                .unwrap();
        assert_eq!(replay.version, saved.version);
        let stale = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: Some(initial.version),
            expected_versions: Default::default(),
        };
        let error =
            save_workflow_mutation(&mut connection, statuses, None, None, &stale).unwrap_err();
        assert!(error.starts_with("CONFLICT:"));
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn workflow_edits_survive_restart_and_keep_issue_statuses_consistent() {
        let root =
            std::env::temp_dir().join(format!("pm-app-workflow-test-{}", uuid::Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
        let workspace = current_workspace_id(&connection).unwrap();
        let mut statuses = list_workflow(&connection).unwrap();
        assert_eq!(statuses.len(), 6);
        statuses.push(WorkflowStatus {
            name: "Review".into(),
            category: "Started".into(),
            description: "Needs approval".into(),
            icon: "i-tabler-shield-check".into(),
            color: "purple".into(),
            position: 6,
            archived: false,
            is_default: false,
        });
        save_workflow(&mut connection, statuses.clone(), None, None).unwrap();
        connection
            .execute(
                "INSERT INTO issue(id,workspace_id,identifier,title,status,priority,position)
             VALUES ('issue-one',?1,'LOCAL-1','Review this','Review','Medium',0)",
                [&workspace],
            )
            .unwrap();
        statuses.last_mut().unwrap().name = "In Review".into();
        save_workflow(
            &mut connection,
            statuses.clone(),
            Some("Review"),
            Some("In Review"),
        )
        .unwrap();
        let issue_status: String = connection
            .query_row(
                "SELECT status FROM issue WHERE id = 'issue-one'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(issue_status, "In Review");
        let activity: i64 = connection.query_row("SELECT COUNT(*) FROM issue_activity WHERE issue_id = 'issue-one' AND body LIKE '%Review to In Review%'", [], |row| row.get(0)).unwrap();
        assert_eq!(activity, 1);
        assert!(save_workflow(&mut connection, statuses[..6].to_vec(), None, None).is_err());
        assert!(save_workflow(&mut connection, statuses[..5].to_vec(), None, None).is_err());
        statuses.last_mut().unwrap().archived = true;
        save_workflow(&mut connection, statuses, None, None).unwrap();
        assert!(!status_available(&connection, &workspace, "In Review").unwrap());
        let mut statuses = list_workflow(&connection).unwrap();
        statuses
            .iter_mut()
            .find(|status| status.name == "Backlog")
            .unwrap()
            .is_default = false;
        statuses
            .iter_mut()
            .find(|status| status.name == "Todo")
            .unwrap()
            .is_default = true;
        save_workflow(&mut connection, statuses, None, None).unwrap();
        create_workspace(&mut connection, "Personal").unwrap();
        assert_eq!(list_workflow(&connection).unwrap().len(), 6);
        switch_workspace(&connection, &workspace).unwrap();
        drop(connection);
        let connection = open_database(&root).unwrap();
        assert_eq!(list_workflow(&connection).unwrap().len(), 7);
        assert!(
            list_workflow(&connection)
                .unwrap()
                .iter()
                .find(|status| status.name == "Todo")
                .unwrap()
                .is_default
        );
        assert_eq!(
            list_workflow(&connection)
                .unwrap()
                .last()
                .unwrap()
                .description,
            "Needs approval"
        );
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }
}
