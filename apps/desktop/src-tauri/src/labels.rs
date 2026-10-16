use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use uuid::Uuid;

use crate::mutations::{self, MutationRequest};
use crate::storage::{current_workspace_id, document_data_root, open_database};

const DEFAULTS: &[(&str, &str)] = &[
    ("UI", "#6965bf"),
    ("Feature", "#aa73ca"),
    ("Core", "#d66370"),
    ("Bug", "#d66370"),
    ("Interaction", "#50a47d"),
    ("Feedback", "#50a47d"),
    ("Workflow", "#bd9651"),
    ("Improvement", "#5d9bd0"),
    ("Keyboard", "#999aa3"),
    ("Accessibility", "#5d9bd0"),
    ("Infrastructure", "#59a7b9"),
    ("Design", "#c99b90"),
    ("Security", "#999aa3"),
];

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelRecord {
    id: String,
    version: i64,
    name: String,
    color: String,
    archived_at: Option<String>,
    deleted_at: Option<String>,
    issue_count: i64,
    project_count: i64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    restored: usize,
    skipped: usize,
}

pub(crate) fn seed_default_labels(
    connection: &Connection,
    workspace_id: &str,
) -> Result<(), String> {
    for (name, color) in DEFAULTS {
        connection.execute(
            "INSERT OR IGNORE INTO label(id, workspace_id, name, color) VALUES (?1, ?2, ?3, ?4)",
            params![Uuid::new_v4().to_string(), workspace_id, name, color],
        ).map_err(|_| "Cannot create workspace labels.")?;
    }
    Ok(())
}

fn validate(name: &str, color: &str) -> Result<(), String> {
    if !(1..=80).contains(&name.trim().chars().count())
        || color.len() != 7
        || !color.starts_with('#')
        || !color[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("Enter a label name and a six-digit hex color.".into());
    }
    Ok(())
}

fn list_labels(connection: &Connection) -> Result<Vec<LabelRecord>, String> {
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT label.id, label.version, label.name, label.color, label.archived_at, label.deleted_at,
          (SELECT count(*) FROM issue_label WHERE label_id = label.id),
          (SELECT count(*) FROM project_label WHERE label_id = label.id)
         FROM label WHERE workspace_id = ?1 ORDER BY deleted_at IS NOT NULL,
           archived_at IS NOT NULL, lower(name), id",
        )
        .map_err(|_| "Cannot read labels.")?;
    let labels = statement
        .query_map([workspace_id], |row| {
            Ok(LabelRecord {
                id: row.get(0)?,
                version: row.get(1)?,
                name: row.get(2)?,
                color: row.get(3)?,
                archived_at: row.get(4)?,
                deleted_at: row.get(5)?,
                issue_count: row.get(6)?,
                project_count: row.get(7)?,
            })
        })
        .map_err(|_| "Cannot read labels.")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read labels.".to_owned())?;
    Ok(labels)
}

fn owner_labels(
    connection: &Connection,
    owners: &[String],
    project: bool,
) -> Result<HashMap<String, Vec<String>>, String> {
    if owners.len() > 200 {
        return Err("Request labels for at most 200 records.".into());
    }

    let (table, owner_column, owner_table) = if project {
        ("project_label", "project_id", "project")
    } else {
        ("issue_label", "issue_id", "issue")
    };
    let ids = serde_json::to_string(owners).map_err(|_| "Cannot read owner labels.")?;
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(&format!(
            "SELECT owner.id, label.name FROM {owner_table} owner
             JOIN {table} membership ON membership.{owner_column} = owner.id
             JOIN label ON label.id = membership.label_id
             WHERE owner.workspace_id = ?1 AND owner.id IN (SELECT value FROM json_each(?2))
               AND label.deleted_at IS NULL
             ORDER BY label.name COLLATE NOCASE, label.id"
        ))
        .map_err(|_| "Cannot read owner labels.")?;
    let mut labels: HashMap<String, Vec<String>> =
        owners.iter().cloned().map(|id| (id, Vec::new())).collect();
    let rows = statement
        .query_map(params![workspace_id, ids], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|_| "Cannot read owner labels.")?;
    for row in rows {
        let (id, name) = row.map_err(|_| "Cannot read owner labels.")?;
        if let Some(values) = labels.get_mut(&id) {
            values.push(name);
        }
    }
    Ok(labels)
}

#[api_contract_macros::command]
pub fn storage_list_owner_labels(
    app: AppHandle,
    owners: Vec<String>,
    project: bool,
) -> Result<HashMap<String, Vec<String>>, String> {
    owner_labels(
        &open_database(&document_data_root(&app)?)?,
        &owners,
        project,
    )
}

fn label_name(connection: &Connection, id: &str) -> Result<String, String> {
    connection
        .query_row(
            "SELECT name FROM label WHERE id = ?1 AND workspace_id = ?2",
            params![id, current_workspace_id(connection)?],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot find label.")?
        .ok_or("Label not found.".into())
}

fn create_label(
    connection: &Connection,
    name: &str,
    color: &str,
) -> Result<Vec<LabelRecord>, String> {
    validate(name, color)?;
    connection
        .execute(
            "INSERT INTO label(id, workspace_id, name, color) VALUES (?1, ?2, ?3, ?4)",
            params![
                Uuid::new_v4().to_string(),
                current_workspace_id(connection)?,
                name.trim(),
                color
            ],
        )
        .map_err(|_| "A label with this name already exists.")?;
    list_labels(connection)
}

fn sync_owner_labels(connection: &Connection, table: &str, owner_id: &str) -> Result<(), String> {
    let (membership, owner_column) = if table == "issue" {
        ("issue_label", "issue_id")
    } else {
        ("project_label", "project_id")
    };
    // JSON remains a projection for existing editors and search triggers. Joins own membership.
    connection
        .execute(
            &format!(
                "UPDATE {table} SET labels_json = COALESCE((SELECT json_group_array(name) FROM (
           SELECT label.name AS name FROM {membership} membership
           JOIN label ON label.id = membership.label_id
           WHERE membership.{owner_column} = ?1 AND label.deleted_at IS NULL
           ORDER BY lower(label.name), label.id)), '[]') WHERE id = ?1"
            ),
            [owner_id],
        )
        .map_err(|_| "Cannot update label associations.")?;
    Ok(())
}

fn assigned_owner_ids(
    connection: &Connection,
    table: &str,
    id: &str,
) -> Result<Vec<String>, String> {
    let (owner, _) = if table == "issue_label" {
        ("issue_id", "issue")
    } else {
        ("project_id", "project")
    };
    let mut statement = connection
        .prepare(&format!("SELECT {owner} FROM {table} WHERE label_id = ?1"))
        .map_err(|_| "Cannot read label associations.")?;
    let owners = statement
        .query_map([id], |row| row.get(0))
        .map_err(|_| "Cannot read label associations.")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read label associations.".to_owned())?;
    Ok(owners)
}

#[cfg(test)]
fn update_label(
    connection: &mut Connection,
    id: &str,
    name: &str,
    color: &str,
) -> Result<Vec<LabelRecord>, String> {
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin label update.".to_owned())?;
    update_label_in_tx(&tx, id, name, color)?;
    tx.commit().map_err(|_| "Cannot save label update.")?;
    list_labels(connection)
}

fn update_label_in_tx(
    connection: &Connection,
    id: &str,
    name: &str,
    color: &str,
) -> Result<(), String> {
    validate(name, color)?;
    let old = label_name(connection, id)?;
    let archived_at: Option<String> = connection
        .query_row("SELECT archived_at FROM label WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .map_err(|_| "Cannot read label archive state.")?;
    let changed = connection
        .execute(
            "UPDATE label SET name = ?1, color = ?2, archived_at = NULL
         WHERE id = ?3 AND workspace_id = ?4 AND deleted_at IS NULL",
            params![name.trim(), color, id, current_workspace_id(connection)?],
        )
        .map_err(|_| "A label with this name already exists.")?;
    if changed == 0 {
        return Err("Restore this label before editing it.".into());
    }
    if !old.eq_ignore_ascii_case(name.trim()) || old != name.trim() {
        for owner in assigned_owner_ids(connection, "issue_label", id)? {
            sync_owner_labels(connection, "issue", &owner)?;
        }
        for owner in assigned_owner_ids(connection, "project_label", id)? {
            sync_owner_labels(connection, "project", &owner)?;
        }
        rename_saved_filters(connection, &old, name.trim())?;
    }
    if let Some(archived_at) = archived_at {
        connection
            .execute(
                "UPDATE label SET archived_at = ?1 WHERE id = ?2",
                params![archived_at, id],
            )
            .map_err(|_| "Cannot preserve label archive state.")?;
    }
    Ok(())
}

fn rename_saved_filters(connection: &Connection, old: &str, new: &str) -> Result<(), String> {
    // Saved views still store label names in their definitions, so a rename must update them.
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare("SELECT id, definition_json FROM saved_view WHERE workspace_id = ?1")
        .map_err(|_| "Cannot read saved filters.")?;
    let views = statement
        .query_map([workspace_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|_| "Cannot read saved filters.")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read saved filters.")?;
    drop(statement);
    for (id, json) in views {
        let mut value: serde_json::Value =
            serde_json::from_str(&json).map_err(|_| "Cannot read saved filters.")?;
        for key in [
            "label",
            "labels",
            "excludedLabels",
            "projectLabels",
            "excludedProjectLabels",
        ] {
            match value.get_mut(key) {
                Some(serde_json::Value::String(item)) if item.eq_ignore_ascii_case(old) => {
                    *item = new.into()
                }
                Some(serde_json::Value::Array(items)) => {
                    for item in items {
                        if item
                            .as_str()
                            .is_some_and(|name| name.eq_ignore_ascii_case(old))
                        {
                            *item = new.into();
                        }
                    }
                }
                _ => {}
            }
        }
        connection
            .execute(
                "UPDATE saved_view SET definition_json = ?1 WHERE id = ?2",
                params![value.to_string(), id],
            )
            .map_err(|_| "Cannot rename saved label filters.")?;
    }
    Ok(())
}

fn set_archived(
    connection: &Connection,
    id: &str,
    archived: bool,
) -> Result<Vec<LabelRecord>, String> {
    let changed = connection.execute(
        "UPDATE label SET archived_at = CASE WHEN ?1 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') ELSE NULL END
         WHERE id = ?2 AND workspace_id = ?3 AND deleted_at IS NULL",
        params![archived, id, current_workspace_id(connection)?],
    ).map_err(|_| "Cannot change label archive state.")?;
    if changed == 0 {
        return Err("Active label not found.".into());
    }
    list_labels(connection)
}

#[cfg(test)]
fn trash_label(connection: &mut Connection, id: &str) -> Result<Vec<LabelRecord>, String> {
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin label trash.")?;
    trash_label_in_tx(&tx, id)?;
    tx.commit().map_err(|_| "Cannot save label trash.")?;
    list_labels(connection)
}

fn trash_label_in_tx(connection: &Connection, id: &str) -> Result<(), String> {
    // Preserve memberships before removing their JSON projections. Restore can then reattach
    // only owners that still exist and are eligible for the label.
    for (table, kind) in [("issue_label", "issue"), ("project_label", "project")] {
        for owner in assigned_owner_ids(connection, table, id)? {
            connection.execute("INSERT OR IGNORE INTO label_detached(label_id, owner_kind, owner_id) VALUES (?1, ?2, ?3)",
                params![id, kind, owner]).map_err(|_| "Cannot record label memberships.")?;
            connection
                .execute(
                    &format!("DELETE FROM {table} WHERE label_id = ?1 AND {kind}_id = ?2"),
                    params![id, owner],
                )
                .map_err(|_| "Cannot remove label membership.")?;
            sync_owner_labels(connection, kind, &owner)?;
        }
    }
    let changed = connection.execute("UPDATE label SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), archived_at = NULL
       WHERE id = ?1 AND workspace_id = ?2 AND deleted_at IS NULL", params![id, current_workspace_id(connection)?])
        .map_err(|_| "Cannot trash label.")?;
    if changed == 0 {
        return Err("Label already trashed.".into());
    }
    Ok(())
}

#[cfg(test)]
fn restore_label(connection: &mut Connection, id: &str) -> Result<RestoreReport, String> {
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin label restore.")?;
    let report = restore_label_in_tx(&tx, id)?;
    tx.commit().map_err(|_| "Cannot save label restore.")?;
    Ok(report)
}

fn restore_label_in_tx(connection: &Connection, id: &str) -> Result<RestoreReport, String> {
    let changed = connection.execute("UPDATE label SET deleted_at = NULL WHERE id = ?1 AND workspace_id = ?2 AND deleted_at IS NOT NULL",
        params![id, current_workspace_id(connection)?]).map_err(|_| "Another label already uses this name.")?;
    if changed == 0 {
        return Err("Trashed label not found.".into());
    }
    let mut statement = connection
        .prepare("SELECT owner_kind, owner_id FROM label_detached WHERE label_id = ?1")
        .map_err(|_| "Cannot read former label memberships.")?;
    let owners = statement
        .query_map([id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|_| "Cannot read former label memberships.")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read former label memberships.")?;
    drop(statement);
    let mut report = RestoreReport {
        restored: 0,
        skipped: 0,
    };
    for (kind, owner) in owners {
        let (table, membership, owner_column) = if kind == "issue" {
            ("issue", "issue_label", "issue_id")
        } else {
            ("project", "project_label", "project_id")
        };
        let changed = connection
            .execute(
                &format!(
                    "INSERT OR IGNORE INTO {membership}({owner_column}, label_id)
          SELECT id, ?1 FROM {table} WHERE id = ?2 AND workspace_id = ?3"
                ),
                params![id, owner, current_workspace_id(connection)?],
            )
            .map_err(|_| "Cannot restore label membership.")?;
        if changed == 1 {
            sync_owner_labels(connection, table, &owner)?;
            report.restored += 1;
        } else {
            report.skipped += 1;
        }
    }
    connection
        .execute("DELETE FROM label_detached WHERE label_id = ?1", [id])
        .map_err(|_| "Cannot clear label restore record.")?;
    Ok(report)
}

#[api_contract_macros::command]
pub fn storage_list_labels(app: AppHandle) -> Result<Vec<LabelRecord>, String> {
    list_labels(&open_database(&document_data_root(&app)?)?)
}

#[api_contract_macros::command]
pub fn storage_create_label(
    app: AppHandle,
    name: String,
    color: String,
    mutation: MutationRequest,
) -> Result<Vec<LabelRecord>, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "label.create",
        &(&name, &color),
        &mutation,
        |tx, _| create_label(tx, &name, &color),
    )
}

#[api_contract_macros::command]
pub fn storage_update_label(
    app: AppHandle,
    id: String,
    name: String,
    color: String,
    mutation: MutationRequest,
) -> Result<Vec<LabelRecord>, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "label.update",
        &(&id, &name, &color),
        &mutation,
        |tx, _| {
            mutations::check_version(tx, "label", &id, &mutation)?;
            update_label_in_tx(tx, &id, &name, &color)?;
            list_labels(tx)
        },
    )
}

#[api_contract_macros::command]
pub fn storage_archive_label(
    app: AppHandle,
    id: String,
    mutation: MutationRequest,
) -> Result<Vec<LabelRecord>, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "label.archive",
        &id,
        &mutation,
        |tx, _| {
            mutations::check_version(tx, "label", &id, &mutation)?;
            set_archived(tx, &id, true)
        },
    )
}

#[api_contract_macros::command]
pub fn storage_restore_archived_label(
    app: AppHandle,
    id: String,
    mutation: MutationRequest,
) -> Result<Vec<LabelRecord>, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "label.archive.restore",
        &id,
        &mutation,
        |tx, _| {
            mutations::check_version(tx, "label", &id, &mutation)?;
            set_archived(tx, &id, false)
        },
    )
}

#[api_contract_macros::command]
pub fn storage_trash_label(
    app: AppHandle,
    id: String,
    mutation: MutationRequest,
) -> Result<Vec<LabelRecord>, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "label.trash",
        &id,
        &mutation,
        |tx, _| {
            mutations::check_version(tx, "label", &id, &mutation)?;
            trash_label_in_tx(tx, &id)?;
            list_labels(tx)
        },
    )
}

#[api_contract_macros::command]
pub fn storage_restore_trashed_label(
    app: AppHandle,
    id: String,
    mutation: MutationRequest,
) -> Result<RestoreReport, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "label.trash.restore",
        &id,
        &mutation,
        |tx, _| {
            mutations::check_version(tx, "label", &id, &mutation)?;
            restore_label_in_tx(tx, &id)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{open_database, save_identity};

    #[test]
    fn label_mutations_replay_once_and_reject_stale_edits() {
        let root = std::env::temp_dir().join(format!("pm-app-label-receipt-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Tester", "Workspace").unwrap();
        let create = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: None,
            expected_versions: Default::default(),
        };
        let create_once = |connection: &mut Connection| {
            mutations::apply(
                connection,
                "label.create",
                &("Release", "#123456"),
                &create,
                |tx, _| create_label(tx, "Release", "#123456"),
            )
        };
        let first = create_once(&mut connection).unwrap();
        let replay = create_once(&mut connection).unwrap();
        let label = first.iter().find(|item| item.name == "Release").unwrap();
        assert_eq!(
            replay.iter().filter(|item| item.name == "Release").count(),
            1
        );
        assert_eq!(list_labels(&connection).unwrap().len(), first.len());

        let edit = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: Some(label.version),
            expected_versions: Default::default(),
        };
        let update = |connection: &mut Connection, request: &MutationRequest| {
            mutations::apply(
                connection,
                "label.update",
                &(&label.id, "Launch", "#abcdef"),
                request,
                |tx, _| {
                    mutations::check_version(tx, "label", &label.id, request)?;
                    update_label_in_tx(tx, &label.id, "Launch", "#abcdef")?;
                    list_labels(tx)
                },
            )
        };
        let updated = update(&mut connection, &edit).unwrap();
        assert_eq!(update(&mut connection, &edit).unwrap().len(), updated.len());
        assert!(
            updated
                .iter()
                .find(|item| item.id == label.id)
                .unwrap()
                .version
                > label.version
        );
        let stale = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: Some(label.version),
            expected_versions: Default::default(),
        };
        assert!(update(&mut connection, &stale)
            .unwrap_err()
            .contains("CONFLICT"));
        assert_eq!(label_name(&connection, &label.id).unwrap(), "Launch");
        drop(connection);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archive_trash_and_restore_preserve_label_memberships() {
        let root = std::env::temp_dir().join(format!("pm-app-labels-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Tester", "Workspace").unwrap();
        let workspace_id = current_workspace_id(&connection).unwrap();
        let project_id = Uuid::new_v4().to_string();
        let issue_id = Uuid::new_v4().to_string();
        connection
            .execute(
                "INSERT INTO project(id, workspace_id, name, identifier_prefix, labels_json)
            VALUES (?1, ?2, 'Example', 'EX', '[\"Custom\"]')",
                params![project_id, workspace_id],
            )
            .unwrap();
        connection.execute("INSERT INTO issue(id, workspace_id, identifier, project_id, title, status, priority, position, labels_json)
            VALUES (?1, ?2, 'EX-1', ?3, 'Task', 'Todo', 'No priority', 0, '[\"Custom\"]')",
            params![issue_id, workspace_id, project_id]).unwrap();
        let label = list_labels(&connection)
            .unwrap()
            .into_iter()
            .find(|item| item.name == "Custom")
            .unwrap();
        assert_eq!((label.issue_count, label.project_count), (1, 1));
        set_archived(&connection, &label.id, true).unwrap();
        let second = Uuid::new_v4().to_string();
        assert!(connection.execute("INSERT INTO issue(id, workspace_id, identifier, project_id, title, status, priority, position, labels_json)
            VALUES (?1, ?2, 'EX-2', ?3, 'Other', 'Todo', 'No priority', 1, '[\"Custom\"]')",
            params![second, workspace_id, project_id]).is_err());
        update_label(&mut connection, &label.id, "Renamed", "#123456").unwrap();
        assert!(list_labels(&connection)
            .unwrap()
            .into_iter()
            .find(|item| item.id == label.id)
            .unwrap()
            .archived_at
            .is_some());
        set_archived(&connection, &label.id, false).unwrap();
        let names: (String, String) = connection
            .query_row(
                "SELECT project.labels_json, issue.labels_json FROM project, issue
            WHERE project.id = ?1 AND issue.id = ?2",
                params![project_id, issue_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(names, ("[\"Renamed\"]".into(), "[\"Renamed\"]".into()));
        connection
            .execute(
                "UPDATE issue SET labels_json = '[]' WHERE id = ?1",
                [&issue_id],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO issue_label(issue_id, label_id) VALUES (?1, ?2)",
                params![issue_id, label.id],
            )
            .unwrap();
        update_label(&mut connection, &label.id, "Final", "#123456").unwrap();
        assert_eq!(
            owner_labels(&connection, &[issue_id.clone()], false).unwrap()[&issue_id],
            vec!["Final"]
        );
        let projection: String = connection
            .query_row(
                "SELECT labels_json FROM issue WHERE id = ?1",
                [&issue_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(projection, "[\"Final\"]");
        connection.execute("UPDATE issue SET archived_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1", [&issue_id]).unwrap();
        trash_label(&mut connection, &label.id).unwrap();
        let empty: String = connection
            .query_row(
                "SELECT labels_json FROM issue WHERE id = ?1",
                [&issue_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(empty, "[]");
        let report = restore_label(&mut connection, &label.id).unwrap();
        assert_eq!((report.restored, report.skipped), (2, 0));
        let restored = list_labels(&connection)
            .unwrap()
            .into_iter()
            .find(|item| item.id == label.id)
            .unwrap();
        assert_eq!((restored.issue_count, restored.project_count), (1, 1));
        drop(connection);
        std::fs::remove_dir_all(root).unwrap();
    }
}
