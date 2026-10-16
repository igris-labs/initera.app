use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use uuid::Uuid;

use crate::mutations::{self, MutationRequest};
use crate::storage::{current_workspace_id, document_data_root, open_database};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningNode {
    id: String,
    project_id: String,
    parent_id: Option<String>,
    kind: String,
    name: String,
    description: String,
    position: i64,
    issue_ids: Vec<String>,
    archived_at: Option<String>,
    deleted_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningSnapshot {
    items: Vec<PlanningNode>,
    version: i64,
}

fn planning_snapshot(
    connection: &Connection,
    project_id: &str,
) -> Result<PlanningSnapshot, String> {
    let items = list_nodes(connection, project_id)?;
    let encoded = serde_json::to_vec(&items)
        .map_err(|_| "Cannot read project structure version.".to_owned())?;
    let digest = Sha256::digest(encoded);
    let version = i64::from_be_bytes(digest[..8].try_into().unwrap()) & ((1_i64 << 53) - 1);
    Ok(PlanningSnapshot { items, version })
}

fn mutate_planning<P, F>(
    connection: &mut Connection,
    operation: &str,
    project_id: &str,
    payload: &P,
    mutation: &MutationRequest,
    action: F,
) -> Result<PlanningSnapshot, String>
where
    P: Serialize,
    F: FnOnce(&Connection) -> Result<(), String>,
{
    mutations::apply(
        connection,
        operation,
        &(project_id, payload),
        mutation,
        |tx, _| {
            if mutation.expected_version != Some(planning_snapshot(tx, project_id)?.version) {
                return Err("CONFLICT: Project structure changed. Reload before saving.".into());
            }
            action(tx)?;
            planning_snapshot(tx, project_id)
        },
    )
}

#[derive(Deserialize, Serialize)]
struct DissolveSnapshot {
    child_ids: Vec<String>,
    issue_ids: Vec<String>,
}

fn project_exists(connection: &Connection, project_id: &str) -> Result<(), String> {
    let workspace_id = current_workspace_id(connection)?;
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1 AND workspace_id = ?2)",
            params![project_id, workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot check the project.".to_owned())?;
    if exists {
        Ok(())
    } else {
        Err("Project not found in this workspace.".into())
    }
}

fn node_project_id(connection: &Connection, id: &str) -> Result<String, String> {
    connection
        .query_row(
            "SELECT project_id FROM planning_node WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| "Project node not found.".to_owned())
}

fn check_node_project(connection: &Connection, id: &str, project_id: &str) -> Result<(), String> {
    if node_project_id(connection, id)? != project_id {
        return Err("CROSS_PROJECT: Node belongs to another project.".into());
    }
    Ok(())
}

fn validate_node(kind: &str, name: &str, description: &str) -> Result<(), String> {
    if !matches!(kind, "module" | "feature")
        || !(1..=120).contains(&name.trim().chars().count())
        || description.chars().count() > 10_000
    {
        return Err(
            "Enter a module or feature name and a description under 10,000 characters.".into(),
        );
    }
    Ok(())
}

fn validate_parent(
    connection: &Connection,
    project_id: &str,
    parent_id: Option<&str>,
) -> Result<(), String> {
    let Some(parent_id) = parent_id else {
        return Ok(());
    };
    let parent_project: Option<String> = connection
        .query_row(
            "SELECT project_id FROM planning_node WHERE id = ?1 AND archived_at IS NULL AND deleted_at IS NULL",
            [parent_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot check the parent node.".to_owned())?;
    if parent_project.as_deref() == Some(project_id) {
        Ok(())
    } else {
        Err("Parent must belong to the same project.".into())
    }
}

fn list_nodes(connection: &Connection, project_id: &str) -> Result<Vec<PlanningNode>, String> {
    project_exists(connection, project_id)?;
    let mut statement = connection
        .prepare(
            "SELECT id, project_id, parent_id, kind, name, description, position, archived_at, deleted_at
                  FROM planning_node WHERE project_id = ?1 ORDER BY position, name, id",
        )
        .map_err(|_| "Cannot read project structure.".to_owned())?;
    let mut nodes = statement
        .query_map([project_id], |row| {
            Ok(PlanningNode {
                id: row.get(0)?,
                project_id: row.get(1)?,
                parent_id: row.get(2)?,
                kind: row.get(3)?,
                name: row.get(4)?,
                description: row.get(5)?,
                position: row.get(6)?,
                issue_ids: Vec::new(),
                archived_at: row.get(7)?,
                deleted_at: row.get(8)?,
            })
        })
        .map_err(|_| "Cannot read project structure.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read project structure.".to_owned())?;
    let mut assignments = connection
        .prepare(
            "SELECT planning_node_issue.node_id, issue.identifier
                  FROM planning_node_issue JOIN issue ON issue.id = planning_node_issue.issue_id
                  JOIN planning_node ON planning_node.id = planning_node_issue.node_id
                  WHERE planning_node.project_id = ?1 AND issue.archived_at IS NULL
                  ORDER BY planning_node_issue.node_id, issue.identifier",
        )
        .map_err(|_| "Cannot read structure assignments.".to_owned())?;
    let rows = assignments
        .query_map([project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|_| "Cannot read structure assignments.".to_owned())?;
    for row in rows {
        let (node_id, identifier) = row.map_err(|_| "Cannot read structure assignments.")?;
        if let Some(node) = nodes.iter_mut().find(|node| node.id == node_id) {
            node.issue_ids.push(identifier);
        }
    }
    Ok(nodes)
}

fn create_node(
    connection: &Connection,
    project_id: &str,
    parent_id: Option<&str>,
    kind: &str,
    name: &str,
    description: &str,
) -> Result<Vec<PlanningNode>, String> {
    project_exists(connection, project_id)?;
    validate_parent(connection, project_id, parent_id)?;
    validate_node(kind, name, description)?;
    connection.execute(
        "INSERT INTO planning_node(id, project_id, parent_id, kind, name, description, position)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                 (SELECT COALESCE(MAX(position) + 1, 0) FROM planning_node WHERE project_id = ?2 AND parent_id IS ?3))",
        params![Uuid::new_v4().to_string(), project_id, parent_id, kind, name.trim(), description],
    ).map_err(|_| "Cannot create project node.".to_owned())?;
    list_nodes(connection, project_id)
}

fn update_node(
    connection: &Connection,
    id: &str,
    parent_id: Option<&str>,
    kind: &str,
    name: &str,
    description: &str,
) -> Result<Vec<PlanningNode>, String> {
    validate_node(kind, name, description)?;
    let project_id: String = connection
        .query_row(
            "SELECT project_id FROM planning_node WHERE id = ?1 AND archived_at IS NULL AND deleted_at IS NULL",
            [id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot find project node.".to_owned())?
        .ok_or("Project node not found.")?;
    project_exists(connection, &project_id)?;
    validate_parent(connection, &project_id, parent_id)?;
    if let Some(parent_id) = parent_id {
        let cyclic: bool = connection.query_row(
            "WITH RECURSIVE descendants(id) AS (
               SELECT id FROM planning_node WHERE id = ?1
               UNION ALL SELECT child.id FROM planning_node child JOIN descendants ON child.parent_id = descendants.id
             ) SELECT EXISTS(SELECT 1 FROM descendants WHERE id = ?2)",
            params![id, parent_id], |row| row.get(0),
        ).map_err(|_| "Cannot check project node hierarchy.".to_owned())?;
        if cyclic {
            return Err("A node cannot be nested under itself or its descendants.".into());
        }
    }
    connection
        .execute(
            "UPDATE planning_node SET parent_id = ?2, kind = ?3, name = ?4, description = ?5,
         position = CASE WHEN parent_id IS ?2 THEN position ELSE
           (SELECT COALESCE(MAX(position) + 1, 0) FROM planning_node AS siblings
            WHERE siblings.project_id = ?6 AND siblings.parent_id IS ?2) END
         WHERE id = ?1",
            params![id, parent_id, kind, name.trim(), description, project_id],
        )
        .map_err(|_| "Cannot update project node.".to_owned())?;
    list_nodes(connection, &project_id)
}

fn active_node(connection: &Connection, id: &str) -> Result<(String, Option<String>), String> {
    connection
        .query_row(
            "SELECT project_id, parent_id FROM planning_node
             WHERE id = ?1 AND archived_at IS NULL AND deleted_at IS NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| "Cannot find project node.".to_owned())?
        .ok_or("Active project node not found.".into())
}

#[cfg(test)]
fn reorder_node(
    connection: &mut Connection,
    id: &str,
    target: usize,
) -> Result<Vec<PlanningNode>, String> {
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin node reorder.")?;
    let result = reorder_node_in_tx(&tx, id, target)?;
    tx.commit().map_err(|_| "Cannot save node order.")?;
    Ok(result)
}

fn reorder_node_in_tx(
    connection: &Connection,
    id: &str,
    target: usize,
) -> Result<Vec<PlanningNode>, String> {
    let (project_id, parent_id) = active_node(connection, id)?;
    project_exists(connection, &project_id)?;
    let mut statement = connection
        .prepare(
            "SELECT id FROM planning_node WHERE project_id = ?1 AND parent_id IS ?2
         AND archived_at IS NULL AND deleted_at IS NULL ORDER BY position, id",
        )
        .map_err(|_| "Cannot read node order.")?;
    let mut siblings = statement
        .query_map(params![project_id, parent_id], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|_| "Cannot read node order.")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read node order.")?;
    drop(statement);
    if target >= siblings.len() {
        return Err("Choose a valid position in this level.".into());
    }
    let old = siblings
        .iter()
        .position(|sibling| sibling == id)
        .ok_or("Node not found in this level.")?;
    let moved = siblings.remove(old);
    siblings.insert(target, moved);
    for (position, sibling) in siblings.iter().enumerate() {
        connection
            .execute(
                "UPDATE planning_node SET position = ?1 WHERE id = ?2",
                params![position, sibling],
            )
            .map_err(|_| "Cannot reorder project nodes.")?;
    }
    list_nodes(connection, &project_id)
}

fn archive_node(connection: &Connection, id: &str) -> Result<Vec<PlanningNode>, String> {
    let (project_id, _) = active_node(connection, id)?;
    project_exists(connection, &project_id)?;
    let running: bool = connection.query_row(
        "WITH RECURSIVE descendants(id) AS (
           SELECT id FROM planning_node WHERE id = ?1
           UNION ALL SELECT child.id FROM planning_node child JOIN descendants ON child.parent_id = descendants.id
         ) SELECT EXISTS(SELECT 1 FROM time_entry timer
           JOIN planning_node_issue membership ON membership.issue_id = timer.issue_id
           WHERE membership.node_id IN (SELECT id FROM descendants) AND timer.stopped_at IS NULL)",
        [id], |row| row.get(0),
    ).map_err(|_| "Cannot inspect running timers.")?;
    if running {
        return Err("Stop running timers in this structure before archiving.".into());
    }
    // Group descendants by this action so restoring one archived subtree does not revive
    // nodes archived in an earlier, unrelated action.
    let group_id = Uuid::new_v4().to_string();
    connection.execute(
        "WITH RECURSIVE descendants(id) AS (
           SELECT id FROM planning_node WHERE id = ?1
           UNION ALL SELECT child.id FROM planning_node child JOIN descendants ON child.parent_id = descendants.id
         ) UPDATE planning_node SET archived_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
           archive_group_id = ?2 WHERE id IN (SELECT id FROM descendants)
           AND archived_at IS NULL AND deleted_at IS NULL",
        params![id, group_id],
    ).map_err(|_| "Cannot archive project structure.")?;
    list_nodes(connection, &project_id)
}

fn restore_archived_node(connection: &Connection, id: &str) -> Result<Vec<PlanningNode>, String> {
    let (project_id, parent_id, group_id): (String, Option<String>, String) = connection
        .query_row(
            "SELECT project_id, parent_id, archive_group_id FROM planning_node
         WHERE id = ?1 AND archived_at IS NOT NULL AND deleted_at IS NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| "Cannot find archived node.")?
        .ok_or("Archived node not found.")?;
    project_exists(connection, &project_id)?;
    if let Some(parent) = parent_id.as_deref() {
        validate_parent(connection, &project_id, Some(parent))?;
    }
    connection
        .execute(
            "UPDATE planning_node SET archived_at = NULL, archive_group_id = NULL
         WHERE project_id = ?1 AND archive_group_id = ?2 AND deleted_at IS NULL",
            params![project_id, group_id],
        )
        .map_err(|_| "Cannot restore project structure.")?;
    list_nodes(connection, &project_id)
}

#[cfg(test)]
fn dissolve_node(connection: &mut Connection, id: &str) -> Result<Vec<PlanningNode>, String> {
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin node dissolve.")?;
    let result = dissolve_node_in_tx(&tx, id)?;
    tx.commit().map_err(|_| "Cannot save node dissolve.")?;
    Ok(result)
}

fn dissolve_node_in_tx(connection: &Connection, id: &str) -> Result<Vec<PlanningNode>, String> {
    // Dissolving promotes children and assignments; snapshot their original membership so
    // an optional restore can put them back without moving later additions.
    let (project_id, parent_id) = active_node(connection, id)?;
    project_exists(connection, &project_id)?;
    let mut children = connection.prepare(
        "SELECT id FROM planning_node WHERE parent_id = ?1 AND deleted_at IS NULL ORDER BY position, id",
    ).map_err(|_| "Cannot read child nodes.")?;
    let child_ids = children
        .query_map([id], |row| row.get::<_, String>(0))
        .map_err(|_| "Cannot read child nodes.")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read child nodes.")?;
    drop(children);
    let mut assignments = connection
        .prepare("SELECT issue_id FROM planning_node_issue WHERE node_id = ?1")
        .map_err(|_| "Cannot read node issues.")?;
    let issue_ids = assignments
        .query_map([id], |row| row.get::<_, String>(0))
        .map_err(|_| "Cannot read node issues.")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read node issues.")?;
    drop(assignments);
    let snapshot = serde_json::to_string(&DissolveSnapshot {
        child_ids: child_ids.clone(),
        issue_ids: issue_ids.clone(),
    })
    .map_err(|_| "Cannot record node memberships.")?;
    let next_position: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM planning_node
         WHERE project_id = ?1 AND parent_id IS ?2 AND deleted_at IS NULL",
            params![project_id, parent_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read sibling order.")?;
    for (offset, child) in child_ids.iter().enumerate() {
        connection
            .execute(
                "UPDATE planning_node SET parent_id = ?1, position = ?2 WHERE id = ?3",
                params![parent_id, next_position + offset as i64, child],
            )
            .map_err(|_| "Cannot reparent child nodes.")?;
    }
    for issue in &issue_ids {
        if let Some(parent) = parent_id.as_deref() {
            connection
                .execute(
                    "UPDATE planning_node_issue SET node_id = ?1 WHERE issue_id = ?2",
                    params![parent, issue],
                )
                .map_err(|_| "Cannot reparent node issues.")?;
        } else {
            connection
                .execute(
                    "DELETE FROM planning_node_issue WHERE issue_id = ?1",
                    [issue],
                )
                .map_err(|_| "Cannot move node issues to project root.")?;
        }
    }
    connection
        .execute(
            "UPDATE planning_node SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
         dissolve_snapshot_json = ?2 WHERE id = ?1",
            params![id, snapshot],
        )
        .map_err(|_| "Cannot dissolve project node.")?;
    list_nodes(connection, &project_id)
}

#[cfg(test)]
fn restore_dissolved_node(
    connection: &mut Connection,
    id: &str,
    memberships: bool,
) -> Result<Vec<PlanningNode>, String> {
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin node restore.")?;
    let result = restore_dissolved_node_in_tx(&tx, id, memberships)?;
    tx.commit().map_err(|_| "Cannot save node restore.")?;
    Ok(result)
}

fn restore_dissolved_node_in_tx(
    connection: &Connection,
    id: &str,
    memberships: bool,
) -> Result<Vec<PlanningNode>, String> {
    let (project_id, parent_id, snapshot): (String, Option<String>, String) = connection
        .query_row(
            "SELECT project_id, parent_id, dissolve_snapshot_json FROM planning_node
         WHERE id = ?1 AND deleted_at IS NOT NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| "Cannot find dissolved node.")?
        .ok_or("Dissolved node not found.")?;
    project_exists(connection, &project_id)?;
    let snapshot: DissolveSnapshot =
        serde_json::from_str(&snapshot).map_err(|_| "Cannot read dissolved memberships.")?;
    let parent_active = parent_id
        .as_deref()
        .map(|parent| validate_parent(connection, &project_id, Some(parent)).is_ok())
        .unwrap_or(true);
    let restored_parent = if parent_active { parent_id } else { None };
    connection
        .execute(
            "UPDATE planning_node SET deleted_at = NULL, parent_id = ?2 WHERE id = ?1",
            params![id, restored_parent],
        )
        .map_err(|_| "Cannot restore project node.")?;
    if memberships {
        for child in snapshot.child_ids {
            connection
                .execute(
                    "UPDATE planning_node SET parent_id = ?1 WHERE id = ?2 AND parent_id IS ?3
                AND deleted_at IS NULL AND archived_at IS NULL",
                    params![id, child, restored_parent],
                )
                .map_err(|_| "Cannot restore child placement.")?;
        }
        for issue in snapshot.issue_ids {
            if let Some(parent) = restored_parent.as_deref() {
                connection.execute("UPDATE planning_node_issue SET node_id = ?1 WHERE issue_id = ?2 AND node_id = ?3",
                    params![id, issue, parent])
                    .map_err(|_| "Cannot restore issue placement.")?;
            } else {
                connection
                    .execute(
                        "INSERT OR IGNORE INTO planning_node_issue(issue_id, node_id)
                    SELECT issue.id, ?1 FROM issue WHERE issue.id = ?2 AND issue.project_id = ?3
                    AND issue.archived_at IS NULL AND NOT EXISTS
                    (SELECT 1 FROM planning_node_issue WHERE issue_id = issue.id)",
                        params![id, issue, project_id],
                    )
                    .map_err(|_| "Cannot restore issue placement.")?;
            }
        }
    }
    list_nodes(connection, &project_id)
}

fn set_node_issue(
    connection: &Connection,
    project_id: &str,
    identifier: &str,
    node_id: Option<&str>,
) -> Result<Vec<PlanningNode>, String> {
    project_exists(connection, project_id)?;
    validate_parent(connection, project_id, node_id)?;
    let issue_id: String = connection.query_row(
        "SELECT issue.id FROM issue WHERE issue.identifier = ?1 AND issue.project_id = ?2 AND issue.archived_at IS NULL",
        params![identifier, project_id], |row| row.get(0),
    ).optional().map_err(|_| "Cannot find issue in project.".to_owned())?
        .ok_or("Issue not found in this project.")?;
    if let Some(node_id) = node_id {
        connection
            .execute(
                "INSERT INTO planning_node_issue(issue_id, node_id) VALUES (?1, ?2)
             ON CONFLICT(issue_id) DO UPDATE SET node_id = excluded.node_id",
                params![issue_id, node_id],
            )
            .map_err(|_| "Cannot assign issue to project node.".to_owned())?;
    } else {
        connection
            .execute(
                "DELETE FROM planning_node_issue WHERE issue_id = ?1",
                [issue_id],
            )
            .map_err(|_| "Cannot remove issue from project node.".to_owned())?;
    }
    list_nodes(connection, project_id)
}

#[api_contract_macros::command]
pub fn storage_list_planning_nodes(
    app: AppHandle,
    project_id: String,
) -> Result<PlanningSnapshot, String> {
    planning_snapshot(&open_database(&document_data_root(&app)?)?, &project_id)
}

#[api_contract_macros::command]
pub fn storage_create_planning_node(
    app: AppHandle,
    project_id: String,
    parent_id: Option<String>,
    kind: String,
    name: String,
    description: String,
    mutation: MutationRequest,
) -> Result<PlanningSnapshot, String> {
    mutate_planning(
        &mut open_database(&document_data_root(&app)?)?,
        "planning.create",
        &project_id,
        &(&parent_id, &kind, &name, &description),
        &mutation,
        |tx| {
            create_node(
                tx,
                &project_id,
                parent_id.as_deref(),
                &kind,
                &name,
                &description,
            )
            .map(|_| ())
        },
    )
}

#[api_contract_macros::command]
pub fn storage_update_planning_node(
    app: AppHandle,
    id: String,
    project_id: String,
    parent_id: Option<String>,
    kind: String,
    name: String,
    description: String,
    mutation: MutationRequest,
) -> Result<PlanningSnapshot, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    check_node_project(&connection, &id, &project_id)?;
    mutate_planning(
        &mut connection,
        "planning.update",
        &project_id,
        &(&id, &parent_id, &kind, &name, &description),
        &mutation,
        |tx| update_node(tx, &id, parent_id.as_deref(), &kind, &name, &description).map(|_| ()),
    )
}

#[api_contract_macros::command]
pub fn storage_reorder_planning_node(
    app: AppHandle,
    id: String,
    project_id: String,
    index: usize,
    mutation: MutationRequest,
) -> Result<PlanningSnapshot, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    check_node_project(&connection, &id, &project_id)?;
    mutate_planning(
        &mut connection,
        "planning.reorder",
        &project_id,
        &(&id, index),
        &mutation,
        |tx| reorder_node_in_tx(tx, &id, index).map(|_| ()),
    )
}

#[api_contract_macros::command]
pub fn storage_archive_planning_node(
    app: AppHandle,
    id: String,
    project_id: String,
    mutation: MutationRequest,
) -> Result<PlanningSnapshot, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    check_node_project(&connection, &id, &project_id)?;
    mutate_planning(
        &mut connection,
        "planning.archive",
        &project_id,
        &id,
        &mutation,
        |tx| archive_node(tx, &id).map(|_| ()),
    )
}

#[api_contract_macros::command]
pub fn storage_restore_archived_planning_node(
    app: AppHandle,
    id: String,
    project_id: String,
    mutation: MutationRequest,
) -> Result<PlanningSnapshot, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    check_node_project(&connection, &id, &project_id)?;
    mutate_planning(
        &mut connection,
        "planning.restore_archive",
        &project_id,
        &id,
        &mutation,
        |tx| restore_archived_node(tx, &id).map(|_| ()),
    )
}

#[api_contract_macros::command]
pub fn storage_dissolve_planning_node(
    app: AppHandle,
    id: String,
    project_id: String,
    mutation: MutationRequest,
) -> Result<PlanningSnapshot, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    check_node_project(&connection, &id, &project_id)?;
    mutate_planning(
        &mut connection,
        "planning.dissolve",
        &project_id,
        &id,
        &mutation,
        |tx| dissolve_node_in_tx(tx, &id).map(|_| ()),
    )
}

#[api_contract_macros::command]
pub fn storage_restore_dissolved_planning_node(
    app: AppHandle,
    id: String,
    project_id: String,
    memberships: bool,
    mutation: MutationRequest,
) -> Result<PlanningSnapshot, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    check_node_project(&connection, &id, &project_id)?;
    mutate_planning(
        &mut connection,
        "planning.restore_dissolve",
        &project_id,
        &(&id, memberships),
        &mutation,
        |tx| restore_dissolved_node_in_tx(tx, &id, memberships).map(|_| ()),
    )
}

#[api_contract_macros::command]
pub fn storage_set_planning_node_issue(
    app: AppHandle,
    project_id: String,
    identifier: String,
    node_id: Option<String>,
    mutation: MutationRequest,
) -> Result<PlanningSnapshot, String> {
    mutate_planning(
        &mut open_database(&document_data_root(&app)?)?,
        "planning.assign_issue",
        &project_id,
        &(&identifier, &node_id),
        &mutation,
        |tx| set_node_issue(tx, &project_id, &identifier, node_id.as_deref()).map(|_| ()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{open_database, save_identity};
    use std::fs;

    #[test]
    fn planning_receipt_replays_and_rejects_stale_structure() {
        let root = std::env::temp_dir().join(format!("pm-app-planning-receipt-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
        let project_id = Uuid::new_v4().to_string();
        connection.execute(
            "INSERT INTO project(id, workspace_id, name, identifier_prefix) VALUES (?1, ?2, 'Release', 'REL')",
            params![project_id, current_workspace_id(&connection).unwrap()],
        ).unwrap();
        let version = planning_snapshot(&connection, &project_id).unwrap().version;
        let request = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: Some(version),
            expected_versions: Default::default(),
        };
        let create = |connection: &mut Connection, request: &MutationRequest| {
            mutate_planning(
                connection,
                "planning.create",
                &project_id,
                &("module", "App"),
                request,
                |tx| create_node(tx, &project_id, None, "module", "App", "").map(|_| ()),
            )
        };

        let created = create(&mut connection, &request).unwrap();
        assert_eq!(created.items.len(), 1);
        let replay = create(&mut connection, &request).unwrap();
        assert_eq!(replay.items[0].id, created.items[0].id);
        let stale = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: Some(version),
            expected_versions: Default::default(),
        };
        assert!(create(&mut connection, &stale)
            .unwrap_err()
            .starts_with("CONFLICT:"));
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn nested_nodes_keep_issue_membership_and_reject_cycles() {
        let root = std::env::temp_dir().join(format!("pm-app-planning-test-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
        let workspace_id = current_workspace_id(&connection).unwrap();
        let project_id = Uuid::new_v4().to_string();
        let other_project_id = Uuid::new_v4().to_string();
        connection.execute(
            "INSERT INTO project(id, workspace_id, name, identifier_prefix) VALUES (?1, ?2, 'Release', 'REL')",
            params![project_id, workspace_id],
        ).unwrap();
        connection.execute(
            "INSERT INTO project(id, workspace_id, name, identifier_prefix) VALUES (?1, ?2, 'Other', 'OTH')",
            params![other_project_id, workspace_id],
        ).unwrap();
        let issue_id = Uuid::new_v4().to_string();
        connection.execute(
            "INSERT INTO issue(id, workspace_id, identifier, project_id, title, status, priority, position)
             VALUES (?1, ?2, 'REL-1', ?3, 'Ship beta', 'Todo', 'No priority', 0)",
            params![issue_id, workspace_id, project_id],
        ).unwrap();

        let root_node = create_node(&connection, &project_id, None, "module", "App", "").unwrap()
            [0]
        .id
        .clone();
        let child = create_node(
            &connection,
            &project_id,
            Some(&root_node),
            "feature",
            "Beta",
            "",
        )
        .unwrap()
        .into_iter()
        .find(|node| node.parent_id.as_deref() == Some(&root_node))
        .unwrap();
        assert!(update_node(
            &connection,
            &root_node,
            Some(&child.id),
            "module",
            "App",
            ""
        )
        .is_err());
        assert!(create_node(
            &connection,
            &other_project_id,
            Some(&root_node),
            "feature",
            "Wrong",
            ""
        )
        .is_err());
        let assigned = set_node_issue(&connection, &project_id, "REL-1", Some(&child.id)).unwrap();
        assert_eq!(
            assigned
                .iter()
                .find(|node| node.id == child.id)
                .unwrap()
                .issue_ids,
            vec!["REL-1"]
        );
        drop(connection);

        let connection = open_database(&root).unwrap();
        assert_eq!(list_nodes(&connection, &project_id).unwrap().len(), 2);
        connection
            .execute(
                "UPDATE issue SET project_id = ?1 WHERE id = ?2",
                params![other_project_id, issue_id],
            )
            .unwrap();
        assert!(list_nodes(&connection, &project_id)
            .unwrap()
            .iter()
            .all(|node| node.issue_ids.is_empty()));
        drop(connection);
        let mut connection = open_database(&root).unwrap();
        dissolve_node(&mut connection, &root_node).unwrap();
        assert_eq!(list_nodes(&connection, &project_id).unwrap().len(), 2);
        assert!(list_nodes(&connection, &project_id)
            .unwrap()
            .iter()
            .find(|node| node.id == root_node)
            .unwrap()
            .deleted_at
            .is_some());
        restore_dissolved_node(&mut connection, &root_node, true).unwrap();
        assert!(list_nodes(&connection, &project_id)
            .unwrap()
            .iter()
            .find(|node| node.id == root_node)
            .unwrap()
            .deleted_at
            .is_none());
        let sibling = create_node(&connection, &project_id, None, "module", "Web", "")
            .unwrap()
            .into_iter()
            .find(|node| node.name == "Web")
            .unwrap();
        let reordered = reorder_node(&mut connection, &sibling.id, 0).unwrap();
        assert!(
            reordered
                .iter()
                .find(|node| node.id == sibling.id)
                .unwrap()
                .position
                < reordered
                    .iter()
                    .find(|node| node.id == root_node)
                    .unwrap()
                    .position
        );
        let archived = archive_node(&mut connection, &root_node).unwrap();
        assert!(archived
            .iter()
            .find(|node| node.id == child.id)
            .unwrap()
            .archived_at
            .is_some());
        connection
            .execute(
                "UPDATE issue SET project_id = ?1 WHERE id = ?2",
                params![project_id, issue_id],
            )
            .unwrap();
        assert!(set_node_issue(&connection, &project_id, "REL-1", Some(&child.id)).is_err());
        let restored = restore_archived_node(&connection, &root_node).unwrap();
        assert!(restored
            .iter()
            .find(|node| node.id == child.id)
            .unwrap()
            .archived_at
            .is_none());
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }
}
