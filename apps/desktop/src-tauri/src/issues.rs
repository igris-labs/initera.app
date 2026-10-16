use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use tauri::AppHandle;
use uuid::Uuid;

use crate::content::{decode_document, encode_document, encode_document_for_storage};
use crate::mutations::{self, MutationRequest};
use crate::storage::{current_workspace_id, document_data_root, open_database, valid_date};
use crate::workflow::status_available;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueInput {
    title: String,
    #[serde(default)]
    title_markdown: String,
    description: String,
    #[serde(default)]
    description_document: Option<serde_json::Value>,
    status: String,
    priority: String,
    project: String,
    #[serde(default)]
    milestone_id: Option<String>,
    labels: Vec<String>,
    due: Option<String>,
    #[serde(default)]
    start_date: Option<String>,
    triage: bool,
    #[serde(default)]
    parent_id: Option<String>,
    #[serde(default)]
    estimate: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReorderMutation {
    mutation_id: String,
    versions: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRecord {
    id: String,
    entity_id: String,
    version: i64,
    title: String,
    title_markdown: String,
    description: String,
    description_document: Option<serde_json::Value>,
    status: String,
    priority: String,
    project: String,
    milestone_id: Option<String>,
    milestone_name: Option<String>,
    node_name: Option<String>,
    cycle_name: Option<String>,
    tracked_seconds: i64,
    labels: Vec<String>,
    due: Option<String>,
    start_date: Option<String>,
    triage: bool,
    triage_entered_at: Option<String>,
    triaged_at: Option<String>,
    parent_id: Option<String>,
    updated: String,
    created: String,
    archived_at: Option<String>,
    archive_root_id: Option<String>,
    estimate: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRecordPage {
    items: Vec<IssueRecord>,
    next_cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct IssueRecordCursor {
    workspace_id: String,
    revision: i64,
    archived: bool,
    position: i64,
    id: String,
    archived_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueActivity {
    pub(crate) id: String,
    pub(crate) issue_id: String,
    pub(crate) actor_name: String,
    pub(crate) kind: String,
    pub(crate) body: String,
    pub(crate) body_document_json: Option<String>,
    pub(crate) created_at: String,
    pub(crate) edited_at: Option<String>,
    pub(crate) deleted_at: Option<String>,
    pub(crate) version: i64,
    pub(crate) changes: Option<serde_json::Value>,
    pub(crate) mutation_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusVisit {
    status_name: String,
    status_category: String,
    entered_at: String,
    left_at: Option<String>,
    mutation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriageTarget {
    identifier: String,
    version: i64,
}

fn bulk_triage(
    connection: &mut Connection,
    targets: &[TriageTarget],
    accept: bool,
    status: &str,
    reason: Option<&str>,
    mutation: &MutationRequest,
) -> Result<Vec<IssueRecord>, String> {
    if targets.is_empty() || targets.len() > 100 {
        return Err("Select between 1 and 100 triage issues.".into());
    }
    if targets
        .iter()
        .map(|item| &item.identifier)
        .collect::<HashSet<_>>()
        .len()
        != targets.len()
    {
        return Err("Triage selection contains duplicates.".into());
    }
    let trimmed_reason = reason.map(str::trim).filter(|value| !value.is_empty());
    if trimmed_reason.is_some_and(|value| value.chars().count() > 1000) {
        return Err("Keep the decline reason under 1,000 characters.".into());
    }
    mutations::apply(
        connection,
        "issue.triage.bulk",
        &(targets, accept, status, trimmed_reason),
        mutation,
        |tx, workspace_id| {
            if !status_available(tx, workspace_id, status)? {
                return Err("Choose an active workflow status.".into());
            }
            let mut ids = Vec::with_capacity(targets.len());
            for target in targets {
                let (id, current_version, pending): (String, i64, bool) = tx
                    .query_row(
                        "SELECT id, version, triage FROM issue
                         WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
                        params![workspace_id, target.identifier],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .map_err(|_| "Triage issue no longer exists.".to_owned())?;
                if current_version != target.version || !pending {
                    return Err("CONFLICT: Triage selection changed. Reload and try again.".into());
                }
                ids.push(id);
            }
            for id in ids {
                tx.execute(
                    "UPDATE issue SET triage = 0, status = ?1,
                       triaged_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                       updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                     WHERE id = ?2",
                    params![status, id],
                )
                .map_err(|_| "Cannot decide triage issue.".to_owned())?;
                let body = if accept {
                    "accepted issue from triage".to_owned()
                } else if let Some(reason) = trimmed_reason {
                    format!("declined issue from triage: {reason}")
                } else {
                    "declined issue from triage".to_owned()
                };
                record_activity(tx, &id, "changed", &body)?;
            }
            issue_records_by_identifiers(
                tx,
                &targets
                    .iter()
                    .map(|target| target.identifier.clone())
                    .collect::<Vec<_>>(),
            )
        },
    )
}

#[api_contract_macros::command]
pub fn storage_bulk_triage(
    app: AppHandle,
    targets: Vec<TriageTarget>,
    accept: bool,
    status: String,
    reason: Option<String>,
    mutation: MutationRequest,
) -> Result<Vec<IssueRecord>, String> {
    bulk_triage(
        &mut open_database(&document_data_root(&app)?)?,
        &targets,
        accept,
        &status,
        reason.as_deref(),
        &mutation,
    )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueMovePreview {
    identifier: String,
    destination: String,
    proposed_identifier: String,
    revision: String,
    affected_count: usize,
    clears_milestones: usize,
    clears_nodes: usize,
    clears_cycles: usize,
}

fn move_target(
    connection: &Connection,
    workspace_id: &str,
    name: &str,
) -> Result<(Option<String>, String), String> {
    if name == "Unassigned" {
        return Ok((None, "LOCAL".into()));
    }
    connection
        .query_row(
            "SELECT id, identifier_prefix FROM project WHERE workspace_id = ?1 AND name = ?2
             AND archived_at IS NULL AND deleted_at IS NULL",
            params![workspace_id, name],
            |row| Ok((Some(row.get(0)?), row.get(1)?)),
        )
        .optional()
        .map_err(|_| "Cannot find destination project.".to_owned())?
        .ok_or("Destination project not found.".into())
}

fn move_subtree(
    connection: &Connection,
    workspace_id: &str,
    identifier: &str,
) -> Result<Vec<(String, String, Option<String>, String)>, String> {
    let mut statement = connection.prepare(
        "WITH RECURSIVE descendants(id, depth) AS (
           SELECT id, 0 FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL
           UNION ALL
           SELECT child.id, descendants.depth + 1 FROM issue child JOIN descendants ON child.parent_id = descendants.id
           WHERE child.workspace_id = ?1 AND child.archived_at IS NULL
         ) SELECT issue.id, issue.identifier, issue.milestone_id, issue.updated_at
         FROM descendants JOIN issue ON issue.id = descendants.id ORDER BY descendants.depth, issue.id"
    ).map_err(|_| "Cannot inspect issue hierarchy.".to_owned())?;
    let rows = statement
        .query_map(params![workspace_id, identifier], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|_| "Cannot inspect issue hierarchy.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot inspect issue hierarchy.".to_owned())?;
    if rows.is_empty() {
        return Err("Issue no longer exists.".into());
    }
    Ok(rows)
}

fn incompatible_cycle_count(
    connection: &Connection,
    issue_id: &str,
    destination: Option<&str>,
) -> Result<usize, String> {
    let mut statement = connection
        .prepare(
            "SELECT cycle.project_scope_json FROM cycle_issue
         JOIN cycle ON cycle.id = cycle_issue.cycle_id
         WHERE cycle_issue.issue_id = ?1 AND cycle.status != 'completed'",
        )
        .map_err(|_| "Cannot inspect issue cycles.".to_owned())?;
    let scopes = statement
        .query_map([issue_id], |row| row.get::<_, String>(0))
        .map_err(|_| "Cannot inspect issue cycles.".to_owned())?;
    let mut incompatible = 0;
    for scope in scopes {
        let projects: Vec<String> =
            serde_json::from_str(&scope.map_err(|_| "Cannot read cycle scope.")?)
                .map_err(|_| "Cannot read cycle scope.")?;
        if !projects.is_empty()
            && !destination.is_some_and(|id| projects.iter().any(|project| project == id))
        {
            incompatible += 1;
        }
    }
    Ok(incompatible)
}

fn preview_issue_move(
    connection: &Connection,
    identifier: &str,
    destination: &str,
) -> Result<IssueMovePreview, String> {
    let workspace_id = current_workspace_id(connection)?;
    let (target_id, prefix) = move_target(connection, &workspace_id, destination)?;
    let issues = move_subtree(connection, &workspace_id, identifier)?;
    let current_project: Option<String> = connection
        .query_row(
            "SELECT project_id FROM issue WHERE workspace_id = ?1 AND identifier = ?2",
            params![workspace_id, identifier],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot inspect source project.".to_owned())?;
    if current_project == target_id {
        return Err("Issue is already in this project.".into());
    }
    let next: i64 = connection
        .query_row(
            "SELECT next_number FROM issue_sequence WHERE workspace_id = ?1 AND prefix = ?2",
            params![workspace_id, prefix],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot preview an issue identifier.".to_owned())?
        .unwrap_or(1);
    let mut clears_nodes = 0;
    let mut clears_cycles = 0;
    for (id, _, _, _) in &issues {
        clears_nodes += connection
            .query_row(
                "SELECT count(*) FROM planning_node_issue WHERE issue_id = ?1",
                [id],
                |row| row.get::<_, usize>(0),
            )
            .map_err(|_| "Cannot inspect issue structure.".to_owned())?;
        clears_cycles += incompatible_cycle_count(connection, id, target_id.as_deref())?;
    }
    Ok(IssueMovePreview {
        identifier: identifier.into(),
        destination: destination.into(),
        proposed_identifier: format!("{prefix}-{next}"),
        // The workspace revision covers descendants, memberships, and identifier allocation.
        // A move preview expires even when another record in the subtree changes.
        revision: connection
            .query_row(
                "SELECT revision FROM workspace WHERE id = ?1",
                [&workspace_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|_| "Cannot read issue move revision.".to_owned())?
            .to_string(),
        affected_count: issues.len(),
        clears_milestones: issues
            .iter()
            .filter(|(_, _, milestone, _)| milestone.is_some())
            .count(),
        clears_nodes,
        clears_cycles,
    })
}

#[cfg(test)]
fn move_issue_project(
    connection: &mut Connection,
    identifier: &str,
    destination: &str,
    policy: &str,
    revision: &str,
) -> Result<Vec<IssueRecord>, String> {
    move_issue_project_with_mutation(connection, identifier, destination, policy, revision, None)
}

fn move_issue_project_with_mutation(
    connection: &mut Connection,
    identifier: &str,
    destination: &str,
    policy: &str,
    revision: &str,
    mutation: Option<&MutationRequest>,
) -> Result<Vec<IssueRecord>, String> {
    if !matches!(policy, "keep" | "regenerate") {
        return Err("Choose an identifier option.".into());
    }
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin project move.".to_owned())?;
    let workspace_id = current_workspace_id(&tx)?;
    let request_hash = mutations::request_hash(
        "issue.move_project",
        &(
            identifier,
            destination,
            policy,
            revision,
            mutation.and_then(|value| value.expected_version),
        ),
    )?;
    if let Some(request) = mutation {
        if let Some(result) = mutations::replay(
            &tx,
            &workspace_id,
            "issue.move_project",
            &request_hash,
            request,
        )? {
            return Ok(result);
        }
        let issue_id: String = tx.query_row(
            "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
            params![workspace_id, identifier],
            |row| row.get(0),
        ).map_err(|_| "Issue no longer exists.".to_owned())?;
        mutations::check_version(&tx, "issue", &issue_id, request)?;
    }
    let preview = preview_issue_move(&tx, identifier, destination)?;
    if preview.revision != revision {
        return Err("Issue changed. Review the project move again.".into());
    }
    let (target_id, prefix) = move_target(&tx, &workspace_id, destination)?;
    let subtree = move_subtree(&tx, &workspace_id, identifier)?;
    let mut public_ids = Vec::new();
    for (internal_id, old_identifier, _, _) in subtree {
        let new_identifier = if policy == "regenerate" {
            tx.execute(
                "INSERT INTO issue_sequence(workspace_id, prefix, next_number) VALUES (?1, ?2, 2)
                 ON CONFLICT(workspace_id, prefix) DO UPDATE SET next_number = next_number + 1",
                params![workspace_id, prefix],
            )
            .map_err(|_| "Cannot allocate destination identifier.".to_owned())?;
            let number: i64 = tx.query_row(
                "SELECT next_number - 1 FROM issue_sequence WHERE workspace_id = ?1 AND prefix = ?2",
                params![workspace_id, prefix], |row| row.get(0),
            ).map_err(|_| "Cannot allocate destination identifier.".to_owned())?;
            let next = format!("{prefix}-{number}");
            tx.execute(
                "INSERT INTO issue_alias(workspace_id, alias, issue_id) VALUES (?1, ?2, ?3)",
                params![workspace_id, old_identifier, internal_id],
            )
            .map_err(|_| "Cannot keep the previous identifier.".to_owned())?;
            next
        } else {
            old_identifier.clone()
        };
        tx.execute(
            "UPDATE issue SET project_id = ?1, identifier = ?2, milestone_id = NULL,
             triage = ?3, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?4",
            params![
                target_id,
                new_identifier,
                destination == "Unassigned",
                internal_id
            ],
        )
        .map_err(|_| "Cannot move issue to project.".to_owned())?;
        if incompatible_cycle_count(&tx, &internal_id, target_id.as_deref())? > 0 {
            tx.execute(
                "DELETE FROM cycle_issue WHERE issue_id = ?1 AND cycle_id IN
                 (SELECT cycle.id FROM cycle WHERE cycle.status != 'completed'
                  AND json_array_length(cycle.project_scope_json) > 0
                  AND NOT EXISTS (SELECT 1 FROM json_each(cycle.project_scope_json) scope WHERE scope.value = ?2))",
                params![internal_id, target_id],
            ).map_err(|_| "Cannot clear incompatible cycle assignment.".to_owned())?;
        }
        record_activity(
            &tx,
            &internal_id,
            "changed",
            &format!("moved issue to {destination}"),
        )?;
        public_ids.push(new_identifier);
    }
    let records = issue_records_by_identifiers(&tx, &public_ids)?;
    if let Some(request) = mutation {
        mutations::save(
            &tx,
            &workspace_id,
            "issue.move_project",
            &request_hash,
            request,
            &records,
        )?;
    }
    tx.commit()
        .map_err(|_| "Cannot finish project move.".to_owned())?;
    Ok(records)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelation {
    id: String,
    source_id: String,
    target_id: String,
    kind: String,
}

fn list_issue_relations(
    connection: &Connection,
    identifier: &str,
) -> Result<Vec<IssueRelation>, String> {
    let workspace_id = current_workspace_id(connection)?;
    let issue_id: String = connection
        .query_row(
            "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2",
            params![workspace_id, identifier],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot find the issue.".to_owned())?
        .ok_or_else(|| "Issue not found.".to_owned())?;
    let mut statement = connection
        .prepare(
            "SELECT relation.id, source.identifier, target.identifier, relation.kind
         FROM issue_relation relation
         JOIN issue source ON source.id = relation.source_id
         JOIN issue target ON target.id = relation.target_id
         WHERE relation.workspace_id = ?1 AND (relation.source_id = ?2 OR relation.target_id = ?2)
         ORDER BY relation.created_at, relation.id",
        )
        .map_err(|_| "Cannot read issue relations.".to_owned())?;
    let relations = statement
        .query_map(params![workspace_id, issue_id], |row| {
            Ok(IssueRelation {
                id: row.get(0)?,
                source_id: row.get(1)?,
                target_id: row.get(2)?,
                kind: row.get(3)?,
            })
        })
        .map_err(|_| "Cannot read issue relations.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read issue relations.".to_owned())?;
    Ok(relations)
}

#[cfg(test)]
fn create_issue_relation(
    connection: &mut Connection,
    identifier: &str,
    target_identifier: &str,
    kind: &str,
) -> Result<IssueRelation, String> {
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin issue relation.".to_owned())?;
    let relation = create_issue_relation_in_tx(&tx, identifier, target_identifier, kind)?;
    tx.commit()
        .map_err(|_| "Cannot save the issue relation.".to_owned())?;
    Ok(relation)
}

fn create_issue_relation_in_tx(
    tx: &Transaction<'_>,
    identifier: &str,
    target_identifier: &str,
    kind: &str,
) -> Result<IssueRelation, String> {
    if !["blocks", "blocked_by", "related", "duplicates"].contains(&kind) {
        return Err("Choose a valid issue relation.".into());
    }
    let workspace_id = current_workspace_id(tx)?;
    let resolve = |identifier: &str| -> Result<String, String> {
        tx.query_row(
            "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
            params![workspace_id, identifier], |row| row.get(0),
        ).optional().map_err(|_| "Cannot find the issue.".to_owned())?
            .ok_or_else(|| "Issue is missing or archived.".to_owned())
    };
    let current_id = resolve(identifier)?;
    let other_id = resolve(target_identifier)?;
    if current_id == other_id {
        return Err("An issue cannot link to itself.".into());
    }
    let (mut source_id, mut target_id) = if kind == "blocked_by" {
        (other_id.clone(), current_id.clone())
    } else {
        (current_id.clone(), other_id.clone())
    };
    let stored_kind = if kind == "blocked_by" { "blocks" } else { kind };
    if stored_kind != "blocks" && source_id > target_id {
        std::mem::swap(&mut source_id, &mut target_id);
    }
    if stored_kind == "blocks" {
        let reaches_source: bool = tx
            .query_row(
                "WITH RECURSIVE reachable(id) AS (
               SELECT ?1
               UNION
               SELECT relation.target_id FROM issue_relation relation
               JOIN reachable ON relation.source_id = reachable.id
               WHERE relation.workspace_id = ?2 AND relation.kind = 'blocks'
             ) SELECT EXISTS(SELECT 1 FROM reachable WHERE id = ?3)",
                params![target_id, workspace_id, source_id],
                |row| row.get(0),
            )
            .map_err(|_| "Cannot check blocker dependencies.".to_owned())?;
        if reaches_source {
            return Err("Blocking this issue would create a dependency cycle.".into());
        }
    }
    let id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO issue_relation(id, workspace_id, source_id, target_id, kind)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, workspace_id, source_id, target_id, stored_kind],
    )
    .map_err(|_| "These issues are already linked.".to_owned())?;
    record_activity(
        &tx,
        &current_id,
        "changed",
        &format!("linked issue {target_identifier}"),
    )?;
    record_activity(
        &tx,
        &other_id,
        "changed",
        &format!("linked issue {identifier}"),
    )?;
    touch_relation_issues(tx, &current_id, &other_id)?;
    Ok(list_issue_relations(tx, identifier)?
        .into_iter()
        .find(|relation| relation.id == id)
        .ok_or_else(|| "Cannot confirm the saved issue relation.".to_owned())?)
}

#[cfg(test)]
fn delete_issue_relation(
    connection: &mut Connection,
    identifier: &str,
    relation_id: &str,
) -> Result<(), String> {
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin unlinking issues.".to_owned())?;
    delete_issue_relation_in_tx(&tx, identifier, relation_id)?;
    tx.commit()
        .map_err(|_| "Cannot save the removed relation.".to_owned())
}

fn delete_issue_relation_in_tx(
    tx: &Transaction<'_>,
    identifier: &str,
    relation_id: &str,
) -> Result<(), String> {
    let workspace_id = current_workspace_id(tx)?;
    let (current_id, other_id): (String, String) = tx.query_row(
        "SELECT issue.id, CASE WHEN relation.source_id = issue.id THEN relation.target_id ELSE relation.source_id END
         FROM issue JOIN issue_relation relation ON relation.workspace_id = issue.workspace_id
         WHERE issue.workspace_id = ?1 AND issue.identifier = ?2 AND relation.id = ?3
           AND (relation.source_id = issue.id OR relation.target_id = issue.id)",
        params![workspace_id, identifier, relation_id], |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional().map_err(|_| "Cannot find the relation.".to_owned())?
        .ok_or_else(|| "Relation no longer exists.".to_owned())?;
    tx.execute(
        "DELETE FROM issue_relation WHERE id = ?1 AND workspace_id = ?2",
        params![relation_id, workspace_id],
    )
    .map_err(|_| "Cannot unlink issues.".to_owned())?;
    record_activity(&tx, &current_id, "changed", "removed an issue relation")?;
    record_activity(&tx, &other_id, "changed", "removed an issue relation")?;
    touch_relation_issues(tx, &current_id, &other_id)?;
    Ok(())
}

fn touch_relation_issues(tx: &Transaction<'_>, first: &str, second: &str) -> Result<(), String> {
    tx.execute(
        "UPDATE issue SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id IN (?1, ?2)",
        params![first, second],
    )
    .map_err(|_| "Cannot update linked issues.".to_owned())?;
    Ok(())
}

fn record_activity(
    tx: &Transaction<'_>,
    issue_id: &str,
    kind: &str,
    body: &str,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO issue_activity (id, issue_id, actor_name, kind, body)
         VALUES (?1, ?2, (SELECT display_name FROM local_actor WHERE singleton = 1), ?3, ?4)",
        params![id, issue_id, kind, body],
    )
    .map_err(|_| "Cannot record issue activity.".to_owned())?;
    Ok(id)
}

fn validate(input: &IssueInput) -> Result<(), String> {
    encode_document(input.description_document.as_ref())?;
    if input.title.trim().is_empty()
        || input.title.chars().count() > 255
        || input.title_markdown.len() > 4096
    {
        return Err("Issue titles must contain 1–255 characters.".into());
    }
    if input.status.trim().is_empty() || input.status.chars().count() > 32 {
        return Err("Choose a valid status.".into());
    }
    if !["No priority", "Low", "Medium", "High", "Urgent"].contains(&input.priority.as_str()) {
        return Err("Choose a valid priority.".into());
    }
    if input.labels.len() > 32
        || input
            .labels
            .iter()
            .any(|label| label.trim().is_empty() || label.chars().count() > 80)
    {
        return Err("Issue labels must contain 1–80 characters; select no more than 32.".into());
    }
    if [input.start_date.as_deref(), input.due.as_deref()]
        .into_iter()
        .flatten()
        .any(|date| !valid_date(date))
    {
        return Err("Issue dates must be valid calendar dates.".into());
    }
    if input.start_date.is_some() && input.due.is_some() && input.start_date > input.due {
        return Err("Due date must be on or after the start date.".into());
    }
    if input
        .estimate
        .is_some_and(|value| !(1..=1_000_000).contains(&value))
    {
        return Err("Choose a valid issue estimate.".into());
    }
    Ok(())
}

fn project_id(
    tx: &Transaction<'_>,
    workspace_id: &str,
    name: &str,
    allow_inactive: bool,
) -> Result<(Option<String>, String), String> {
    if name == "Unassigned" {
        return Ok((None, "LOCAL".into()));
    }
    tx.query_row(
        "SELECT id, identifier_prefix FROM project WHERE workspace_id = ?1 AND name = ?2
         AND (?3 OR (archived_at IS NULL AND deleted_at IS NULL))",
        params![workspace_id, name, allow_inactive],
        |row| Ok((Some(row.get(0)?), row.get(1)?)),
    )
    .optional()
    .map_err(|_| "Cannot read the project.".to_owned())?
    .ok_or_else(|| "The selected project no longer exists.".to_owned())
}

fn resolve_milestone(
    tx: &Transaction<'_>,
    project_id: Option<&str>,
    milestone_id: Option<&str>,
    existing: Option<&str>,
) -> Result<Option<String>, String> {
    let Some(milestone_id) = milestone_id else {
        return Ok(None);
    };
    let Some(project_id) = project_id else {
        return Err("Choose a project before assigning a milestone.".into());
    };
    let belongs_to_project: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM milestone WHERE id = ?1 AND project_id = ?2
         AND deleted_at IS NULL AND (archived_at IS NULL OR id = ?3))",
            params![milestone_id, project_id, existing],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot check the project milestone.".to_owned())?;
    if !belongs_to_project {
        return Err("Choose a milestone from this project.".into());
    }
    Ok(Some(milestone_id.to_owned()))
}

fn resolve_parent(
    tx: &Transaction<'_>,
    workspace_id: &str,
    identifier: Option<&str>,
    issue_id: Option<&str>,
) -> Result<Option<String>, String> {
    let Some(identifier) = identifier else {
        return Ok(None);
    };
    let mut current: Option<String> = tx
        .query_row(
            "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
            params![workspace_id, identifier],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot read the parent issue.".to_owned())?;
    let parent = current
        .clone()
        .ok_or_else(|| "Parent issue no longer exists.".to_owned())?;
    let mut seen = HashSet::new();
    while let Some(id) = current {
        if Some(id.as_str()) == issue_id || !seen.insert(id.clone()) {
            return Err("An issue cannot be its own parent or descendant.".into());
        }
        current = tx
            .query_row("SELECT parent_id FROM issue WHERE id = ?1", [&id], |row| {
                row.get(0)
            })
            .map_err(|_| "Cannot read the issue hierarchy.".to_owned())?;
    }
    Ok(Some(parent))
}

#[cfg(test)]
fn query_issues_window(
    connection: &Connection,
    archived: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<IssueRecord>, String> {
    let records = query_issues_window_for_ids(connection, archived, -1, None)?;
    Ok(records
        .into_iter()
        .skip(offset.max(0) as usize)
        .take(if limit < 0 {
            usize::MAX
        } else {
            limit as usize
        })
        .collect())
}

fn query_issues_window_for_ids(
    connection: &Connection,
    archived: bool,
    limit: i64,
    identifiers: Option<&str>,
) -> Result<Vec<IssueRecord>, String> {
    let workspace_id = current_workspace_id(connection)?;
    let mut query = connection
        .prepare(
            "SELECT issue.identifier, issue.title, issue.description, issue.status, issue.priority,
                COALESCE(project.name, 'Unassigned'),
                COALESCE((SELECT json_group_array(name) FROM (
                    SELECT label.name FROM issue_label JOIN label ON label.id = issue_label.label_id
                    WHERE issue_label.issue_id = issue.id AND label.deleted_at IS NULL
                    ORDER BY label.name COLLATE NOCASE, label.id
                )), '[]'), issue.due_date,
                issue.triage, issue.updated_at, issue.title_markdown, parent.identifier,
                issue.start_date, issue.archived_at, archive_root.identifier, COALESCE(issue.estimate_value, issue.estimate),
                issue.description_document_json, issue.triage_entered_at, issue.triaged_at,
                issue.milestone_id, issue.created_at, issue.version,
                (SELECT milestone.title FROM milestone WHERE milestone.id = issue.milestone_id),
                (SELECT planning_node.name FROM planning_node_issue
                 JOIN planning_node ON planning_node.id = planning_node_issue.node_id
                 WHERE planning_node_issue.issue_id = issue.id),
                (SELECT cycle.title FROM cycle_issue JOIN cycle ON cycle.id = cycle_issue.cycle_id
                 WHERE cycle_issue.issue_id = issue.id
                 ORDER BY cycle.end_date DESC, cycle.id DESC LIMIT 1),
                COALESCE((SELECT SUM(duration_seconds) FROM time_entry
                          WHERE time_entry.issue_id = issue.id AND time_entry.deleted_at IS NULL), 0), issue.id
         FROM issue LEFT JOIN project ON project.id = issue.project_id
         LEFT JOIN issue AS parent ON parent.id = issue.parent_id
         LEFT JOIN issue AS archive_root ON archive_root.id = issue.archive_root_id
         WHERE issue.workspace_id = ?1 AND (issue.archived_at IS NOT NULL) = ?2
           AND (?4 IS NULL OR issue.identifier IN (SELECT value FROM json_each(?4)))
         ORDER BY CASE WHEN ?2 THEN issue.archived_at END DESC, issue.position, issue.id
         LIMIT ?3",
        )
        .map_err(|_| "Cannot read issues.".to_owned())?;
    let records = query
        .query_map(params![workspace_id, archived, limit, identifiers], |row| {
            let labels_json: String = row.get(6)?;
            Ok(IssueRecord {
                id: row.get(0)?,
                entity_id: row.get(26)?,
                version: row.get(21)?,
                title: row.get(1)?,
                title_markdown: row.get(10)?,
                description: row.get(2)?,
                description_document: decode_document(row.get(16)?)?,
                status: row.get(3)?,
                priority: row.get(4)?,
                project: row.get(5)?,
                labels: serde_json::from_str(&labels_json).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?,
                due: row.get(7)?,
                start_date: row.get(12)?,
                triage: row.get(8)?,
                triage_entered_at: row.get(17)?,
                triaged_at: row.get(18)?,
                parent_id: row.get(11)?,
                updated: row.get(9)?,
                created: row.get(20)?,
                archived_at: row.get(13)?,
                archive_root_id: row.get(14)?,
                estimate: row.get(15)?,
                milestone_id: row.get(19)?,
                milestone_name: row.get(22)?,
                node_name: row.get(23)?,
                cycle_name: row.get(24)?,
                tracked_seconds: row.get(25)?,
            })
        })
        .map_err(|_| "Cannot read issues.".to_owned())?;
    records
        .map(|record| record.map_err(|_| "Cannot read issues.".to_owned()))
        .collect()
}

fn issue_records_by_identifiers(
    connection: &Connection,
    identifiers: &[String],
) -> Result<Vec<IssueRecord>, String> {
    let mut records = Vec::with_capacity(identifiers.len());
    for batch in identifiers.chunks(200) {
        let ids = serde_json::to_string(batch).map_err(|_| "Cannot read changed issues.")?;
        records.extend(query_issues_window_for_ids(
            connection,
            false,
            batch.len() as i64,
            Some(&ids),
        )?);
    }
    Ok(records)
}

#[cfg(test)]
fn query_issues(connection: &Connection, archived: bool) -> Result<Vec<IssueRecord>, String> {
    query_issues_window(connection, archived, -1, 0)
}

#[cfg(test)]
fn list_issues(connection: &Connection) -> Result<Vec<IssueRecord>, String> {
    query_issues(connection, false)
}

#[cfg(test)]
fn list_archived_issues(connection: &Connection) -> Result<Vec<IssueRecord>, String> {
    query_issues(connection, true)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCounts {
    active: i64,
    triage: i64,
    archived: i64,
    statuses: std::collections::HashMap<String, i64>,
    projects: std::collections::HashMap<String, ProjectIssueCounts>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIssueCounts {
    total: i64,
    completed: i64,
    milestones: std::collections::HashMap<String, MilestoneIssueCounts>,
}

#[derive(Serialize)]
pub struct MilestoneIssueCounts {
    total: i64,
    done: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueChildPage {
    items: Vec<IssueRecord>,
    total_count: i64,
    next_cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct IssueChildCursor {
    parent: String,
    revision: i64,
    position: i64,
    id: String,
}

fn issue_children(
    connection: &Connection,
    identifier: &str,
    cursor: Option<&str>,
    limit: u32,
) -> Result<IssueChildPage, String> {
    let workspace_id = current_workspace_id(connection)?;
    let parent: Option<String> = connection
        .query_row(
            "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2",
            params![workspace_id, identifier],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot find parent issue.")?;
    let parent = parent.ok_or_else(|| "Parent issue not found.".to_owned())?;
    let revision: i64 = connection
        .query_row(
            "SELECT revision FROM workspace WHERE id = ?1",
            [&workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read sub-issue page revision.".to_owned())?;
    let after = cursor
        .map(|token| -> Result<IssueChildCursor, String> {
            let bytes = URL_SAFE_NO_PAD
                .decode(token)
                .map_err(|_| "Sub-issue cursor is invalid.")?;
            if bytes.len() > 512 {
                return Err("Sub-issue cursor is too large.".into());
            }
            let value: IssueChildCursor =
                serde_json::from_slice(&bytes).map_err(|_| "Sub-issue cursor is invalid.")?;
            if value.parent != parent
                || value.revision != revision
                || Uuid::parse_str(&value.id).is_err()
            {
                return Err("Sub-issues changed. Reload and try again.".into());
            }
            Ok(value)
        })
        .transpose()?;
    let total_count = connection
        .query_row(
            "SELECT count(*) FROM issue WHERE parent_id = ?1 AND archived_at IS NULL",
            [&parent],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot count sub-issues.".to_owned())?;
    let mut statement = connection
        .prepare(
            "SELECT identifier, position, id FROM issue WHERE parent_id = ?1 AND archived_at IS NULL
             AND (?2 IS NULL OR position > ?3 OR (position = ?3 AND id > ?2))
             ORDER BY position, id LIMIT ?4",
        )
        .map_err(|_| "Cannot read sub-issues.".to_owned())?;
    let identifiers = statement
        .query_map(
            params![
                parent,
                after.as_ref().map(|value| &value.id),
                after.as_ref().map(|value| value.position),
                i64::from(limit) + 1
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .map_err(|_| "Cannot read sub-issues.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read sub-issues.".to_owned())?;
    let has_more = identifiers.len() > limit as usize;
    let identifiers = &identifiers[..identifiers.len().min(limit as usize)];
    let names: Vec<_> = identifiers.iter().map(|item| item.0.as_str()).collect();
    let ids = serde_json::to_string(&names).map_err(|_| "Cannot read sub-issues.")?;
    let mut items = query_issues_window_for_ids(connection, false, i64::from(limit), Some(&ids))?;
    let order: std::collections::HashMap<_, _> = identifiers
        .iter()
        .enumerate()
        .map(|(index, item)| (item.0.as_str(), index))
        .collect();
    items.sort_by_key(|issue| order.get(issue.id.as_str()).copied().unwrap_or(usize::MAX));
    let next_cursor = if has_more {
        let last = identifiers.last().ok_or("Cannot read sub-issue cursor.")?;
        Some(
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&IssueChildCursor {
                    parent,
                    revision,
                    position: last.1,
                    id: last.2.clone(),
                })
                .map_err(|_| "Cannot encode sub-issue cursor.")?,
            ),
        )
    } else {
        None
    };
    Ok(IssueChildPage {
        items,
        total_count,
        next_cursor,
    })
}

fn issue_counts(connection: &Connection) -> Result<IssueCounts, String> {
    let workspace_id = current_workspace_id(connection)?;
    let (active, triage, archived) = connection
        .query_row(
            "SELECT
               COALESCE(sum(CASE WHEN issue.archived_at IS NULL AND issue.triage = 0
                 AND COALESCE(status.category, 'Started') NOT IN ('Completed', 'Canceled') THEN 1 ELSE 0 END), 0),
               COALESCE(sum(CASE WHEN issue.archived_at IS NULL AND issue.triage = 1 THEN 1 ELSE 0 END), 0),
               COALESCE(sum(CASE WHEN issue.archived_at IS NOT NULL THEN 1 ELSE 0 END), 0)
             FROM issue LEFT JOIN workflow_status status
               ON status.workspace_id = issue.workspace_id AND status.name = issue.status
             WHERE issue.workspace_id = ?1",
            [&workspace_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "Cannot count issues.".to_owned())?;
    let mut statement = connection
        .prepare("SELECT status, count(*) FROM issue WHERE workspace_id = ?1 GROUP BY status")
        .map_err(|_| "Cannot count status usage.".to_owned())?;
    let statuses = statement
        .query_map([&workspace_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|_| "Cannot count status usage.".to_owned())?
        .collect::<rusqlite::Result<std::collections::HashMap<_, _>>>()
        .map_err(|_| "Cannot count status usage.".to_owned())?;
    let mut statement = connection
        .prepare(
            "SELECT project.name, count(issue.id),
               COALESCE(sum(CASE WHEN status.category = 'Completed' THEN 1 ELSE 0 END), 0)
             FROM project LEFT JOIN issue ON issue.project_id = project.id
               AND issue.archived_at IS NULL AND issue.triage = 0
             LEFT JOIN workflow_status status
               ON status.workspace_id = issue.workspace_id AND status.name = issue.status
             WHERE project.workspace_id = ?1 GROUP BY project.id",
        )
        .map_err(|_| "Cannot count project issues.".to_owned())?;
    let mut projects = statement
        .query_map([&workspace_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                ProjectIssueCounts {
                    total: row.get(1)?,
                    completed: row.get(2)?,
                    milestones: std::collections::HashMap::new(),
                },
            ))
        })
        .map_err(|_| "Cannot count project issues.".to_owned())?
        .collect::<rusqlite::Result<std::collections::HashMap<_, _>>>()
        .map_err(|_| "Cannot count project issues.".to_owned())?;
    let mut statement = connection
        .prepare(
            "SELECT project.name, issue.milestone_id, count(*),
               COALESCE(sum(CASE WHEN status.category = 'Completed' THEN 1 ELSE 0 END), 0)
             FROM issue JOIN project ON project.id = issue.project_id
             LEFT JOIN workflow_status status
               ON status.workspace_id = issue.workspace_id AND status.name = issue.status
             WHERE issue.workspace_id = ?1 AND issue.archived_at IS NULL
               AND issue.triage = 0 AND issue.milestone_id IS NOT NULL
             GROUP BY project.id, issue.milestone_id",
        )
        .map_err(|_| "Cannot count milestone issues.".to_owned())?;
    let rows = statement
        .query_map([&workspace_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                MilestoneIssueCounts {
                    total: row.get(2)?,
                    done: row.get(3)?,
                },
            ))
        })
        .map_err(|_| "Cannot count milestone issues.".to_owned())?;
    for row in rows {
        let (project, milestone, counts) = row.map_err(|_| "Cannot count milestone issues.")?;
        if let Some(project) = projects.get_mut(&project) {
            project.milestones.insert(milestone, counts);
        }
    }
    Ok(IssueCounts {
        active,
        triage,
        archived,
        statuses,
        projects,
    })
}

fn issue_descendant_count(connection: &Connection, identifier: &str) -> Result<i64, String> {
    let workspace_id = current_workspace_id(connection)?;
    connection
        .query_row(
            "WITH RECURSIVE descendants(id) AS (
               SELECT id FROM issue WHERE identifier = ?1 AND workspace_id = ?2
               UNION ALL
               SELECT child.id FROM issue child JOIN descendants ON child.parent_id = descendants.id
             ) SELECT CASE WHEN count(*) > 0 THEN count(*) - 1 ELSE 0 END FROM descendants",
            params![identifier, workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot count sub-issues.".to_owned())
}

#[cfg(test)]
fn create_issue(connection: &mut Connection, input: IssueInput) -> Result<IssueRecord, String> {
    create_issue_with_mutation(connection, input, None)
}

fn create_issue_with_mutation(
    connection: &mut Connection,
    input: IssueInput,
    mutation: Option<MutationRequest>,
) -> Result<IssueRecord, String> {
    validate(&input)?;
    let workspace_id = current_workspace_id(connection)?;
    let request_hash = mutations::request_hash("issue.create", &input)?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin issue creation.".to_owned())?;
    if let Some(request) = &mutation {
        if let Some(previous) =
            mutations::replay(&tx, &workspace_id, "issue.create", &request_hash, request)?
        {
            return Ok(previous);
        }
        if request.expected_version.is_some() {
            return Err("A new issue cannot have an expected version.".into());
        }
    }
    if !status_available(&tx, &workspace_id, &input.status)? {
        return Err("Choose an active workflow status.".into());
    }
    let (project_id, prefix) = project_id(&tx, &workspace_id, &input.project, false)?;
    let milestone_id = resolve_milestone(
        &tx,
        project_id.as_deref(),
        input.milestone_id.as_deref(),
        None,
    )?;
    let parent_id = resolve_parent(&tx, &workspace_id, input.parent_id.as_deref(), None)?;
    tx.execute(
        "INSERT INTO issue_sequence(workspace_id, prefix, next_number) VALUES (?1, ?2, 2)
         ON CONFLICT(workspace_id, prefix) DO UPDATE SET next_number = next_number + 1",
        params![workspace_id, prefix],
    )
    .map_err(|_| "Cannot allocate an issue identifier.".to_owned())?;
    let number: i64 = tx
        .query_row(
            "SELECT next_number - 1 FROM issue_sequence WHERE workspace_id = ?1 AND prefix = ?2",
            params![workspace_id, prefix],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot allocate an issue identifier.".to_owned())?;
    let identifier = format!("{prefix}-{number}");
    let labels =
        serde_json::to_string(&input.labels).map_err(|_| "Cannot save issue labels.".to_owned())?;
    let issue_id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO issue(id, workspace_id, identifier, project_id, title, description,
                           status, priority, labels_json, due_date, triage, position, title_markdown, parent_id, start_date, estimate_value,
                           description_document_json, triage_entered_at, milestone_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                 COALESCE((SELECT MIN(position) - 1 FROM issue WHERE workspace_id = ?2), 0), ?12, ?13, ?14, ?15, ?16,
                 CASE WHEN ?11 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') ELSE NULL END, ?17)",
        params![
            issue_id,
            workspace_id,
            identifier,
            project_id,
            input.title.trim(),
            input.description.trim(),
            input.status,
            input.priority,
            labels,
            input.due,
            input.triage,
            input.title_markdown,
            parent_id,
            input.start_date,
            input.estimate,
            encode_document_for_storage(&tx, input.description_document.as_ref())?,
            milestone_id
        ],
    )
    .map_err(|_| "Cannot create the issue.".to_owned())?;
    record_activity(&tx, &issue_id, "created", "created this issue")?;
    let record = issue_records_by_identifiers(&tx, &[identifier])?
        .into_iter()
        .next()
        .ok_or_else(|| "Cannot confirm the new issue.".to_owned())?;
    if let Some(request) = &mutation {
        mutations::save(
            &tx,
            &workspace_id,
            "issue.create",
            &request_hash,
            request,
            &record,
        )?;
    }
    tx.commit()
        .map_err(|_| "Cannot save the issue.".to_owned())?;
    Ok(record)
}

#[cfg(test)]
fn update_issue(
    connection: &mut Connection,
    identifier: &str,
    input: IssueInput,
    order: Option<Vec<String>>,
) -> Result<IssueRecord, String> {
    update_issue_with_mutation(connection, identifier, input, order, None)
}

fn update_issue_with_mutation(
    connection: &mut Connection,
    identifier: &str,
    input: IssueInput,
    order: Option<Vec<String>>,
    mutation: Option<MutationRequest>,
) -> Result<IssueRecord, String> {
    validate(&input)?;
    let workspace_id = current_workspace_id(connection)?;
    let request_hash = mutations::request_hash(
        "issue.update",
        &(
            identifier,
            &input,
            &order,
            mutation
                .as_ref()
                .and_then(|request| request.expected_version),
        ),
    )?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin the issue update.".to_owned())?;
    if let Some(request) = &mutation {
        if let Some(previous) =
            mutations::replay(&tx, &workspace_id, "issue.update", &request_hash, request)?
        {
            return Ok(previous);
        }
    }
    let (project_id, _) = project_id(&tx, &workspace_id, &input.project, true)?;
    let issue_id: String = tx
        .query_row(
            "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
            params![workspace_id, identifier],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot read the issue.".to_owned())?
        .ok_or_else(|| "Issue no longer exists.".to_owned())?;
    if let Some(request) = &mutation {
        mutations::check_version(&tx, "issue", &issue_id, request)?;
    }
    let parent_id = resolve_parent(
        &tx,
        &workspace_id,
        input.parent_id.as_deref(),
        Some(&issue_id),
    )?;
    let labels =
        serde_json::to_string(&input.labels).map_err(|_| "Cannot save issue labels.".to_owned())?;
    let previous: (
        String,
        String,
        Option<String>,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        bool,
        Option<String>,
        Option<i64>,
        Option<String>,
    ) = tx
        .query_row(
            "SELECT status, priority, project_id, title, description, labels_json,
                    due_date, start_date, triage, parent_id, COALESCE(estimate_value, estimate), milestone_id FROM issue WHERE id = ?1",
            [&issue_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                ))
            },
        )
        .map_err(|_| "Cannot read the previous issue state.".to_owned())?;
    let milestone_id = resolve_milestone(
        &tx,
        project_id.as_deref(),
        input.milestone_id.as_deref(),
        previous.11.as_deref(),
    )?;
    let mut changes = Vec::new();
    if previous.0 != input.status && !status_available(&tx, &workspace_id, &input.status)? {
        return Err("Choose an active workflow status.".into());
    }
    if previous.0 != input.status {
        changes.push(format!(
            "changed status from {} to {}",
            previous.0, input.status
        ));
    }
    if previous.1 != input.priority {
        changes.push(format!(
            "changed priority from {} to {}",
            previous.1, input.priority
        ));
    }
    if previous.2 != project_id {
        return Err("Use the reviewed project move to change an issue's project.".into());
    }
    if previous.3 != input.title.trim() {
        changes.push("renamed the issue".to_owned());
    }
    if previous.4 != input.description.trim() {
        changes.push("edited the description".to_owned());
    }
    if previous.5 != labels {
        changes.push("changed labels".to_owned());
    }
    if previous.6 != input.due {
        changes.push("changed the due date".to_owned());
    }
    if previous.7 != input.start_date {
        changes.push("changed the start date".to_owned());
    }
    if previous.8 != input.triage {
        changes.push(
            if input.triage {
                "sent issue to triage"
            } else if input.status == "Canceled" {
                "declined issue from triage"
            } else {
                "accepted issue from triage"
            }
            .to_owned(),
        );
    }
    if previous.9 != parent_id {
        changes.push("changed the parent issue".to_owned());
    }
    if previous.10 != input.estimate {
        changes.push("changed the estimate".to_owned());
    }
    if previous.11 != milestone_id {
        changes.push("changed the project milestone".to_owned());
    }
    let changed = tx
        .execute(
            "UPDATE issue SET project_id = ?1, title = ?2, description = ?3, status = ?4,
                priority = ?5, labels_json = ?6, due_date = ?7, triage = ?8,
                title_markdown = ?11, parent_id = ?12, start_date = ?13, estimate = NULL, estimate_value = ?14,
                description_document_json = ?15, milestone_id = ?16,
                triage_entered_at = CASE WHEN triage = 0 AND ?8 = 1
                  THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') ELSE triage_entered_at END,
                triaged_at = CASE WHEN triage = 1 AND ?8 = 0
                  THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                  WHEN triage = 0 AND ?8 = 1 THEN NULL ELSE triaged_at END,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE workspace_id = ?9 AND identifier = ?10 AND archived_at IS NULL",
            params![
                project_id,
                input.title.trim(),
                input.description.trim(),
                input.status,
                input.priority,
                labels,
                input.due,
                input.triage,
                workspace_id,
                identifier,
                input.title_markdown,
                parent_id,
                input.start_date,
                input.estimate,
                encode_document_for_storage(&tx, input.description_document.as_ref())?,
                milestone_id
            ],
        )
        .map_err(|_| "Cannot update the issue.".to_owned())?;
    if changed == 0 {
        return Err("Issue no longer exists.".into());
    }
    if !changes.is_empty() {
        let mut deltas = Vec::new();
        if previous.0 != input.status {
            deltas.push(
                serde_json::json!({"field":"status","before":previous.0,"after":input.status}),
            );
        }
        if previous.1 != input.priority {
            deltas.push(
                serde_json::json!({"field":"priority","before":previous.1,"after":input.priority}),
            );
        }
        if previous.3 != input.title.trim() {
            deltas.push(
                serde_json::json!({"field":"title","before":previous.3,"after":input.title.trim()}),
            );
        }
        if previous.4 != input.description.trim() {
            deltas.push(serde_json::json!({"field":"description","before":previous.4,"after":input.description.trim()}));
        }
        if previous.5 != labels {
            deltas.push(serde_json::json!({"field":"labels","before":previous.5,"after":labels}));
        }
        if previous.6 != input.due {
            deltas
                .push(serde_json::json!({"field":"dueDate","before":previous.6,"after":input.due}));
        }
        if previous.7 != input.start_date {
            deltas.push(serde_json::json!({"field":"startDate","before":previous.7,"after":input.start_date}));
        }
        if previous.8 != input.triage {
            deltas.push(
                serde_json::json!({"field":"triage","before":previous.8,"after":input.triage}),
            );
        }
        if previous.9 != parent_id {
            deltas.push(
                serde_json::json!({"field":"parentId","before":previous.9,"after":parent_id}),
            );
        }
        if previous.10 != input.estimate {
            deltas.push(
                serde_json::json!({"field":"estimate","before":previous.10,"after":input.estimate}),
            );
        }
        if previous.11 != milestone_id {
            deltas.push(serde_json::json!({"field":"milestoneId","before":previous.11,"after":milestone_id}));
        }
        let activity_id = record_activity(&tx, &issue_id, "changed", &changes.join("; "))?;
        tx.execute(
            "UPDATE issue_activity SET changes_json = ?1, mutation_id = ?2 WHERE id = ?3",
            params![
                serde_json::to_string(&deltas).map_err(|_| "Cannot encode issue changes.")?,
                mutation.as_ref().map(|request| &request.mutation_id),
                activity_id
            ],
        )
        .map_err(|_| "Cannot save structured issue changes.")?;
        if previous.0 != input.status {
            tx.execute(
                "UPDATE issue_status_visit SET mutation_id = ?1
                 WHERE id = (SELECT id FROM issue_status_visit WHERE issue_id = ?2 ORDER BY id DESC LIMIT 1)",
                params![mutation.as_ref().map(|request| &request.mutation_id), issue_id],
            ).map_err(|_| "Cannot link status transition to mutation.")?;
        }
    }
    if let Some(order) = order {
        apply_order(&tx, &workspace_id, &order)?;
    }
    let record = issue_records_by_identifiers(&tx, &[identifier.to_owned()])?
        .into_iter()
        .next()
        .ok_or_else(|| "Cannot confirm the issue update.".to_owned())?;
    if let Some(request) = &mutation {
        mutations::save(
            &tx,
            &workspace_id,
            "issue.update",
            &request_hash,
            request,
            &record,
        )?;
    }
    tx.commit()
        .map_err(|_| "Cannot save the issue update.".to_owned())?;
    Ok(record)
}

fn apply_order(
    tx: &Transaction<'_>,
    workspace_id: &str,
    identifiers: &[String],
) -> Result<(), String> {
    if identifiers.iter().collect::<HashSet<_>>().len() != identifiers.len() {
        return Err("Issue order contains duplicates.".into());
    }
    let mut statement = tx
        .prepare("SELECT identifier, position FROM issue WHERE workspace_id = ?1 AND archived_at IS NULL ORDER BY position, id")
        .map_err(|_| "Cannot read issue order.".to_owned())?;
    let all = statement
        .query_map([workspace_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|_| "Cannot read issue order.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read issue order.".to_owned())?;
    let selected: HashSet<_> = identifiers.iter().collect();
    if all.iter().filter(|(id, _)| selected.contains(id)).count() != identifiers.len() {
        return Err("Issue order changed. Reload and try again.".into());
    }
    let unique_positions = all
        .iter()
        .map(|(_, position)| position)
        .collect::<HashSet<_>>()
        .len()
        == all.len();
    let positions = all
        .iter()
        .map(|(id, position)| (id, position))
        .collect::<HashMap<_, _>>();
    // Stable rank slots avoid rewriting thousands of unseen issues on each visible-page drag.
    // ponytail: a preexisting duplicate rank forces a one-time full rebalance.
    let mut reordered = identifiers.iter();
    for (index, (current, position)) in all.iter().enumerate() {
        if !selected.contains(current) {
            continue;
        }
        let identifier = reordered
            .next()
            .ok_or("Issue order changed. Reload and try again.")?;
        let target_position = if unique_positions {
            *position
        } else {
            index as i64
        };
        if unique_positions
            && positions
                .get(identifier)
                .is_some_and(|original| **original == target_position)
        {
            continue;
        }
        tx.execute(
            "UPDATE issue SET position = ?1 WHERE workspace_id = ?2 AND identifier = ?3 AND archived_at IS NULL",
            params![target_position, workspace_id, identifier],
        )
        .map_err(|_| "Cannot reorder issues.".to_owned())?;
    }
    if unique_positions {
        return Ok(());
    }
    for (position, (identifier, _)) in all.iter().enumerate() {
        if selected.contains(identifier) {
            continue;
        }
        tx.execute(
            "UPDATE issue SET position = ?1 WHERE workspace_id = ?2 AND identifier = ?3 AND archived_at IS NULL",
            params![position as i64, workspace_id, identifier],
        )
        .map_err(|_| "Cannot reorder issues.".to_owned())?;
    }
    Ok(())
}

#[cfg(test)]
fn reorder_issues(connection: &mut Connection, identifiers: Vec<String>) -> Result<(), String> {
    reorder_issues_with_mutation(connection, identifiers, None)
}

fn reorder_issues_with_mutation(
    connection: &mut Connection,
    identifiers: Vec<String>,
    mutation: Option<ReorderMutation>,
) -> Result<(), String> {
    let workspace_id = current_workspace_id(connection)?;
    let request_hash = mutations::request_hash(
        "issue.reorder",
        &(&identifiers, &mutation.as_ref().map(|item| &item.versions)),
    )?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin issue reorder.".to_owned())?;
    if let Some(mutation) = &mutation {
        let request = MutationRequest {
            mutation_id: mutation.mutation_id.clone(),
            expected_version: None,
            expected_versions: Default::default(),
        };
        if mutations::replay::<()>(&tx, &workspace_id, "issue.reorder", &request_hash, &request)?
            .is_some()
        {
            return Ok(());
        }
        if mutation.versions.len() != identifiers.len()
            || identifiers
                .iter()
                .any(|id| !mutation.versions.contains_key(id))
        {
            return Err("CONFLICT: Every reordered issue needs its current version.".into());
        }
        for identifier in &identifiers {
            let current: Option<i64> = tx
                .query_row(
                    "SELECT version FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
                    params![workspace_id, identifier],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|_| "Cannot read issue version.".to_owned())?;
            if current != mutation.versions.get(identifier).copied() {
                return Err("CONFLICT: Issue order changed. Reload and try again.".into());
            }
        }
        apply_order(&tx, &workspace_id, &identifiers)?;
        mutations::save(
            &tx,
            &workspace_id,
            "issue.reorder",
            &request_hash,
            &request,
            &(),
        )?;
    } else {
        apply_order(&tx, &workspace_id, &identifiers)?;
    }
    tx.commit()
        .map_err(|_| "Cannot save issue order.".to_owned())
}

fn rehome_sub_issues(
    tx: &Transaction<'_>,
    workspace_id: &str,
    issue_id: &str,
    strategy: &str,
    target_identifier: Option<&str>,
) -> Result<(), String> {
    if strategy == "all" {
        return Ok(());
    }
    if strategy != "detach" && strategy != "move" {
        return Err("Choose how to handle sub-issues.".into());
    }

    let target_id: Option<String> = if strategy == "move" {
        let identifier = target_identifier.ok_or("Choose a destination issue.")?;
        let target: String = tx
            .query_row(
                "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
                params![workspace_id, identifier],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "Cannot read the destination issue.".to_owned())?
            .ok_or("Destination issue is unavailable.")?;
        let inside_tree: bool = tx
            .query_row(
                "WITH RECURSIVE descendants(id) AS (
                   SELECT id FROM issue WHERE id = ?1
                   UNION ALL SELECT child.id FROM issue child JOIN descendants parent ON child.parent_id = parent.id
                 ) SELECT EXISTS(SELECT 1 FROM descendants WHERE id = ?2)",
                params![issue_id, target],
                |row| row.get(0),
            )
            .map_err(|_| "Cannot validate the destination issue.".to_owned())?;
        if inside_tree {
            return Err("Move sub-issues outside their current issue tree.".into());
        }
        Some(target)
    } else {
        None
    };

    // Restore descendants before detaching them so an archived parent can be deleted safely.
    tx.execute(
        "WITH RECURSIVE descendants(id) AS (
           SELECT id FROM issue WHERE parent_id = ?1 AND workspace_id = ?2
           UNION ALL SELECT child.id FROM issue child JOIN descendants parent ON child.parent_id = parent.id
         ) UPDATE issue SET archived_at = NULL, archive_root_id = NULL
         WHERE workspace_id = ?2 AND id IN (SELECT id FROM descendants)",
        params![issue_id, workspace_id],
    )
    .map_err(|_| "Cannot restore sub-issues.".to_owned())?;
    tx.execute(
        "UPDATE issue SET parent_id = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE workspace_id = ?2 AND parent_id = ?3",
        params![target_id, workspace_id, issue_id],
    )
    .map_err(|_| "Cannot move sub-issues.".to_owned())?;
    Ok(())
}

#[cfg(test)]
fn archive_issue_with_strategy(
    connection: &mut Connection,
    identifier: &str,
    strategy: &str,
    target_identifier: Option<&str>,
) -> Result<(), String> {
    archive_issue_with_mutation(connection, identifier, strategy, target_identifier, None)
}

fn archive_issue_with_mutation(
    connection: &mut Connection,
    identifier: &str,
    strategy: &str,
    target_identifier: Option<&str>,
    mutation: Option<&MutationRequest>,
) -> Result<(), String> {
    crate::time::heartbeat(connection)?;
    let workspace_id = current_workspace_id(connection)?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin issue archive.".to_owned())?;
    let request_hash = mutations::request_hash(
        "issue.archive",
        &(
            identifier,
            strategy,
            target_identifier,
            mutation.and_then(|item| item.expected_version),
        ),
    )?;
    if let Some(request) = mutation {
        if mutations::replay::<()>(&tx, &workspace_id, "issue.archive", &request_hash, request)?
            .is_some()
        {
            return Ok(());
        }
    }
    let issue_id: String = tx
        .query_row(
            "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
            params![workspace_id, identifier],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot read the issue.".to_owned())?
        .ok_or_else(|| "Issue is missing or already archived.".to_owned())?;
    if let Some(request) = mutation {
        mutations::check_version(&tx, "issue", &issue_id, request)?;
    }
    rehome_sub_issues(&tx, &workspace_id, &issue_id, strategy, target_identifier)?;
    tx.execute(
        "WITH RECURSIVE descendants(id) AS (
           SELECT id FROM issue WHERE id = ?1 AND workspace_id = ?2
           UNION ALL SELECT child.id FROM issue child JOIN descendants parent ON child.parent_id = parent.id
           WHERE child.workspace_id = ?2
         ) UPDATE time_entry SET stopped_at = unixepoch('now')
         WHERE workspace_id = ?2 AND stopped_at IS NULL AND issue_id IN (SELECT id FROM descendants)",
        params![issue_id, workspace_id],
    )
    .map_err(|_| "Cannot pause time tracking before archiving.".to_owned())?;
    tx.execute(
        "WITH RECURSIVE descendants(id) AS (
           SELECT id FROM issue WHERE id = ?1 AND workspace_id = ?2
           UNION ALL
           SELECT child.id FROM issue child JOIN descendants parent ON child.parent_id = parent.id
           WHERE child.workspace_id = ?2
         )
         UPDATE issue SET archived_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                          archive_root_id = ?1,
                          updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE workspace_id = ?2 AND id IN (SELECT id FROM descendants) AND archived_at IS NULL",
        params![issue_id, workspace_id],
    )
    .map_err(|_| "Cannot archive the issue and its sub-issues.".to_owned())?;
    record_activity(
        &tx,
        &issue_id,
        "archived",
        if strategy == "all" {
            "archived this issue and its active sub-issues"
        } else {
            "archived this issue"
        },
    )?;
    if let Some(request) = mutation {
        mutations::save(
            &tx,
            &workspace_id,
            "issue.archive",
            &request_hash,
            request,
            &(),
        )?;
    }
    tx.commit()
        .map_err(|_| "Cannot save the issue archive.".to_owned())
}

#[cfg(test)]
fn archive_issue(connection: &mut Connection, identifier: &str) -> Result<(), String> {
    archive_issue_with_strategy(connection, identifier, "all", None)
}

#[cfg(test)]
fn delete_archived_issue(
    connection: &mut Connection,
    identifier: &str,
    strategy: &str,
    target_identifier: Option<&str>,
) -> Result<(), String> {
    delete_archived_issue_with_mutation(connection, identifier, strategy, target_identifier, None)
}

fn delete_archived_issue_with_mutation(
    connection: &mut Connection,
    identifier: &str,
    strategy: &str,
    target_identifier: Option<&str>,
    mutation: Option<&MutationRequest>,
) -> Result<(), String> {
    let workspace_id = current_workspace_id(connection)?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin issue deletion.".to_owned())?;
    let request_hash = mutations::request_hash(
        "issue.delete_archived",
        &(
            identifier,
            strategy,
            target_identifier,
            mutation.and_then(|item| item.expected_version),
        ),
    )?;
    if let Some(request) = mutation {
        if mutations::replay::<()>(
            &tx,
            &workspace_id,
            "issue.delete_archived",
            &request_hash,
            request,
        )?
        .is_some()
        {
            return Ok(());
        }
    }
    let issue_id: String = tx.query_row(
        "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NOT NULL",
        params![workspace_id, identifier],
        |row| row.get(0),
    ).optional().map_err(|_| "Cannot read the archived issue.".to_owned())?
        .ok_or("Archived issue not found.")?;
    if let Some(request) = mutation {
        mutations::check_version(&tx, "issue", &issue_id, request)?;
    }

    rehome_sub_issues(&tx, &workspace_id, &issue_id, strategy, target_identifier)?;
    if strategy == "all" {
        tx.execute(
            "WITH RECURSIVE descendants(id) AS (
               SELECT id FROM issue WHERE id = ?1 AND workspace_id = ?2
               UNION ALL SELECT child.id FROM issue child JOIN descendants parent ON child.parent_id = parent.id
             ) DELETE FROM issue WHERE workspace_id = ?2 AND id IN (SELECT id FROM descendants)",
            params![issue_id, workspace_id],
        ).map_err(|_| "Cannot delete the issue and its sub-issues.".to_owned())?;
    } else {
        tx.execute(
            "DELETE FROM issue WHERE workspace_id = ?1 AND id = ?2",
            params![workspace_id, issue_id],
        )
        .map_err(|_| "Cannot delete the archived issue.".to_owned())?;
    }
    if let Some(request) = mutation {
        mutations::save(
            &tx,
            &workspace_id,
            "issue.delete_archived",
            &request_hash,
            request,
            &(),
        )?;
    }
    tx.commit()
        .map_err(|_| "Cannot save issue deletion.".to_owned())
}

#[cfg(test)]
fn restore_issue(connection: &mut Connection, identifier: &str) -> Result<(), String> {
    restore_issue_with_mutation(connection, identifier, None)
}

fn restore_issue_with_mutation(
    connection: &mut Connection,
    identifier: &str,
    mutation: Option<&MutationRequest>,
) -> Result<(), String> {
    let workspace_id = current_workspace_id(connection)?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin issue restoration.".to_owned())?;
    let request_hash = mutations::request_hash(
        "issue.restore",
        &(identifier, mutation.and_then(|item| item.expected_version)),
    )?;
    if let Some(request) = mutation {
        if mutations::replay::<()>(&tx, &workspace_id, "issue.restore", &request_hash, request)?
            .is_some()
        {
            return Ok(());
        }
    }
    let (issue_id, archive_root_id, parent_archived): (String, Option<String>, bool) = tx
        .query_row(
            "SELECT issue.id, issue.archive_root_id, parent.archived_at IS NOT NULL
             FROM issue LEFT JOIN issue AS parent ON parent.id = issue.parent_id
             WHERE issue.workspace_id = ?1 AND issue.identifier = ?2 AND issue.archived_at IS NOT NULL",
            params![workspace_id, identifier],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| "Cannot read the archived issue.".to_owned())?
        .ok_or_else(|| "Issue is missing or already active.".to_owned())?;
    if let Some(request) = mutation {
        mutations::check_version(&tx, "issue", &issue_id, request)?;
    }
    if archive_root_id.as_deref() != Some(issue_id.as_str()) || parent_archived {
        return Err("Restore the archived parent issue first.".into());
    }
    tx.execute(
        "UPDATE issue SET archived_at = NULL, archive_root_id = NULL,
                          updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE workspace_id = ?1 AND archive_root_id = ?2 AND archived_at IS NOT NULL",
        params![workspace_id, issue_id],
    )
    .map_err(|_| "Cannot restore the issue and its sub-issues.".to_owned())?;
    record_activity(
        &tx,
        &issue_id,
        "restored",
        "restored this issue and its sub-issues",
    )?;
    if let Some(request) = mutation {
        mutations::save(
            &tx,
            &workspace_id,
            "issue.restore",
            &request_hash,
            request,
            &(),
        )?;
    }
    tx.commit()
        .map_err(|_| "Cannot save the issue restoration.".to_owned())
}

#[cfg(test)]
fn restore_sub_issue(connection: &mut Connection, identifier: &str) -> Result<(), String> {
    restore_sub_issue_with_mutation(connection, identifier, None)
}

fn restore_sub_issue_with_mutation(
    connection: &mut Connection,
    identifier: &str,
    mutation: Option<&MutationRequest>,
) -> Result<(), String> {
    let workspace_id = current_workspace_id(connection)?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin sub-issue restoration.".to_owned())?;
    let request_hash = mutations::request_hash(
        "issue.restore_sub",
        &(identifier, mutation.and_then(|item| item.expected_version)),
    )?;
    if let Some(request) = mutation {
        if mutations::replay::<()>(
            &tx,
            &workspace_id,
            "issue.restore_sub",
            &request_hash,
            request,
        )?
        .is_some()
        {
            return Ok(());
        }
    }
    let issue_id: String = tx.query_row(
        "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NOT NULL",
        params![workspace_id, identifier],
        |row| row.get(0),
    ).optional().map_err(|_| "Cannot read the archived sub-issue.".to_owned())?
        .ok_or("Archived issue not found.")?;
    if let Some(request) = mutation {
        mutations::check_version(&tx, "issue", &issue_id, request)?;
    }
    tx.execute(
        "WITH RECURSIVE descendants(id) AS (
           SELECT id FROM issue WHERE id = ?1 AND workspace_id = ?2
           UNION ALL SELECT child.id FROM issue child JOIN descendants parent ON child.parent_id = parent.id
         ) UPDATE issue SET archived_at = NULL, archive_root_id = NULL,
                            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE workspace_id = ?2 AND id IN (SELECT id FROM descendants)",
        params![issue_id, workspace_id],
    ).map_err(|_| "Cannot restore the sub-issue tree.".to_owned())?;
    tx.execute(
        "UPDATE issue SET parent_id = NULL WHERE id = ?1 AND workspace_id = ?2",
        params![issue_id, workspace_id],
    )
    .map_err(|_| "Cannot detach the restored sub-issue.".to_owned())?;
    record_activity(
        &tx,
        &issue_id,
        "restored",
        "restored this issue as a top-level issue",
    )?;
    if let Some(request) = mutation {
        mutations::save(
            &tx,
            &workspace_id,
            "issue.restore_sub",
            &request_hash,
            request,
            &(),
        )?;
    }
    tx.commit()
        .map_err(|_| "Cannot save sub-issue restoration.".to_owned())
}

#[cfg(test)]
fn list_issue_activity(
    connection: &Connection,
    identifier: &str,
    offset: i64,
) -> Result<Vec<IssueActivity>, String> {
    if offset < 0 {
        return Err("Invalid activity page.".into());
    }
    let mut entries = Vec::new();
    let mut cursor = None;
    loop {
        let page = issue_activity_page(connection, identifier, cursor.as_deref())?;
        entries.extend(page.items);
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }
    Ok(entries.into_iter().skip(offset as usize).take(30).collect())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueActivityPage {
    items: Vec<IssueActivity>,
    next_cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ActivityCursor {
    owner: String,
    created_at: String,
    row_id: i64,
}

fn issue_activity_page(
    connection: &Connection,
    identifier: &str,
    cursor: Option<&str>,
) -> Result<IssueActivityPage, String> {
    let workspace_id = current_workspace_id(connection)?;
    let issue_id: String = connection
        .query_row(
            "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2",
            params![workspace_id, identifier],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot find the issue.".to_owned())?
        .ok_or_else(|| "Issue not found.".to_owned())?;
    let after = cursor
        .map(|token| -> Result<ActivityCursor, String> {
            let bytes = URL_SAFE_NO_PAD
                .decode(token)
                .map_err(|_| "Activity cursor is invalid.")?;
            if bytes.len() > 512 {
                return Err("Activity cursor is too large.".into());
            }
            let value: ActivityCursor =
                serde_json::from_slice(&bytes).map_err(|_| "Activity cursor is invalid.")?;
            if value.owner != issue_id || value.row_id < 1 {
                return Err("Activity cursor does not match this issue.".into());
            }
            Ok(value)
        })
        .transpose()?;
    let mut statement = connection
        .prepare(
            "SELECT id, issue_id, actor_name, kind, body, created_at, body_document_json,
                    edited_at, deleted_at, version, rowid, changes_json, mutation_id
         FROM issue_activity WHERE issue_id = ?1
           AND (?2 IS NULL OR created_at < ?2 OR (created_at = ?2 AND rowid < ?3))
         ORDER BY created_at DESC, rowid DESC LIMIT 31",
        )
        .map_err(|_| "Cannot read issue activity.".to_owned())?;
    let mut rows = statement
        .query_map(
            params![
                issue_id,
                after.as_ref().map(|value| &value.created_at),
                after.as_ref().map(|value| value.row_id)
            ],
            |row| {
                Ok((
                    IssueActivity {
                        id: row.get(0)?,
                        issue_id: row.get(1)?,
                        actor_name: row.get(2)?,
                        kind: row.get(3)?,
                        body: row.get(4)?,
                        created_at: row.get(5)?,
                        body_document_json: row.get(6)?,
                        edited_at: row.get(7)?,
                        deleted_at: row.get(8)?,
                        version: row.get(9)?,
                        changes: row
                            .get::<_, Option<String>>(11)?
                            .map(|value| {
                                serde_json::from_str(&value)
                                    .map_err(|_| rusqlite::Error::InvalidQuery)
                            })
                            .transpose()?,
                        mutation_id: row.get(12)?,
                    },
                    row.get::<_, i64>(10)?,
                ))
            },
        )
        .map_err(|_| "Cannot read issue activity.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read issue activity.".to_owned())?;
    let has_more = rows.len() > 30;
    rows.truncate(30);
    let next_cursor = if has_more {
        let last = rows.last().ok_or("Cannot read activity cursor.")?;
        Some(
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&ActivityCursor {
                    owner: issue_id,
                    created_at: last.0.created_at.clone(),
                    row_id: last.1,
                })
                .map_err(|_| "Cannot encode activity cursor.")?,
            ),
        )
    } else {
        None
    };
    Ok(IssueActivityPage {
        items: rows.into_iter().map(|row| row.0).collect(),
        next_cursor,
    })
}

#[api_contract_macros::command]
pub fn storage_list_issue_status_visits(
    app: AppHandle,
    identifier: String,
) -> Result<Vec<StatusVisit>, String> {
    let connection = open_database(&document_data_root(&app)?)?;
    let workspace_id = current_workspace_id(&connection)?;
    let mut statement = connection.prepare(
        "SELECT visit.status_name, visit.status_category, visit.entered_at, visit.left_at, visit.mutation_id
         FROM issue_status_visit visit JOIN issue ON issue.id = visit.issue_id
         WHERE issue.workspace_id = ?1 AND issue.identifier = ?2
         ORDER BY visit.entered_at DESC, visit.id DESC"
    ).map_err(|_| "Cannot read status history.")?;
    let visits = statement
        .query_map(params![workspace_id, identifier], |row| {
            Ok(StatusVisit {
                status_name: row.get(0)?,
                status_category: row.get(1)?,
                entered_at: row.get(2)?,
                left_at: row.get(3)?,
                mutation_id: row.get(4)?,
            })
        })
        .map_err(|_| "Cannot read status history.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read status history.".to_owned())?;
    Ok(visits)
}

#[cfg(test)]
fn create_issue_comment(
    connection: &mut Connection,
    identifier: &str,
    body: &str,
    body_document: Option<&serde_json::Value>,
) -> Result<IssueActivity, String> {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 100_000 {
        return Err("Write a comment of up to 100,000 characters.".into());
    }
    let encoded_document = encode_document_for_storage(connection, body_document)?;
    let workspace_id = current_workspace_id(connection)?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin issue comment.".to_owned())?;
    let comment = insert_issue_comment(
        &tx,
        &workspace_id,
        identifier,
        trimmed,
        encoded_document.as_deref(),
    )?;
    tx.commit()
        .map_err(|_| "Cannot save the comment.".to_owned())?;
    Ok(comment)
}

fn insert_issue_comment(
    tx: &Transaction<'_>,
    workspace_id: &str,
    identifier: &str,
    body: &str,
    encoded_document: Option<&str>,
) -> Result<IssueActivity, String> {
    let issue_id: String = tx.query_row(
        "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
        params![workspace_id, identifier], |row| row.get(0),
    ).optional().map_err(|_| "Cannot find the issue.".to_owned())?
        .ok_or_else(|| "Issue is missing or archived.".to_owned())?;
    let id = record_activity(tx, &issue_id, "comment", body)?;
    if let Some(document) = encoded_document {
        tx.execute(
            "UPDATE issue_activity SET body_document_json = ?1 WHERE id = ?2",
            params![document, id],
        )
        .map_err(|_| "Cannot save the comment document.".to_owned())?;
    }
    tx.query_row(
        "SELECT id, issue_id, actor_name, kind, body, created_at, body_document_json,
                edited_at, deleted_at, version FROM issue_activity WHERE id = ?1",
        [&id],
        |row| {
            Ok(IssueActivity {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                actor_name: row.get(2)?,
                kind: row.get(3)?,
                body: row.get(4)?,
                created_at: row.get(5)?,
                body_document_json: row.get(6)?,
                edited_at: row.get(7)?,
                deleted_at: row.get(8)?,
                version: row.get(9)?,
                changes: None,
                mutation_id: None,
            })
        },
    )
    .map_err(|_| "Cannot read the saved comment.".to_owned())
}

#[api_contract_macros::command]
pub fn storage_issue_counts(app: AppHandle) -> Result<IssueCounts, String> {
    issue_counts(&open_database(&document_data_root(&app)?)?)
}

#[api_contract_macros::command]
pub fn storage_issue_descendant_count(app: AppHandle, identifier: String) -> Result<i64, String> {
    issue_descendant_count(&open_database(&document_data_root(&app)?)?, &identifier)
}

#[api_contract_macros::command]
pub fn storage_list_issue_children(
    app: AppHandle,
    identifier: String,
    cursor: Option<String>,
    limit: u32,
) -> Result<IssueChildPage, String> {
    if limit == 0 || limit > 200 {
        return Err("Choose an issue page size between 1 and 200.".into());
    }
    issue_children(
        &open_database(&document_data_root(&app)?)?,
        &identifier,
        cursor.as_deref(),
        limit,
    )
}

#[api_contract_macros::command]
pub fn storage_issue_statuses(
    app: AppHandle,
    identifiers: Vec<String>,
) -> Result<std::collections::HashMap<String, String>, String> {
    if identifiers.len() > 200 {
        return Err("Request statuses for at most 200 issues.".into());
    }
    let connection = open_database(&document_data_root(&app)?)?;
    let ids = serde_json::to_string(&identifiers).map_err(|_| "Cannot read issue statuses.")?;
    let workspace_id = current_workspace_id(&connection)?;
    let mut statement = connection
        .prepare(
            "SELECT identifier, status FROM issue
             WHERE workspace_id = ?1 AND archived_at IS NULL AND triage = 0
               AND identifier IN (SELECT value FROM json_each(?2))",
        )
        .map_err(|_| "Cannot read issue statuses.".to_owned())?;
    let statuses = statement
        .query_map(params![workspace_id, ids], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|_| "Cannot read issue statuses.".to_owned())?
        .collect::<rusqlite::Result<std::collections::HashMap<_, _>>>()
        .map_err(|_| "Cannot read issue statuses.".to_owned())?;
    Ok(statuses)
}

#[api_contract_macros::command]
pub fn storage_list_issue_page(
    app: AppHandle,
    limit: u32,
    archived: bool,
    cursor: Option<String>,
) -> Result<IssueRecordPage, String> {
    if limit == 0 || limit > 200 {
        return Err("Choose an issue page size between 1 and 200.".into());
    }
    issue_record_page(
        &open_database(&document_data_root(&app)?)?,
        archived,
        limit,
        cursor.as_deref(),
    )
}

pub(crate) fn issue_record_page(
    connection: &Connection,
    archived: bool,
    limit: u32,
    cursor: Option<&str>,
) -> Result<IssueRecordPage, String> {
    let workspace_id = current_workspace_id(connection)?;
    let revision: i64 = connection
        .query_row(
            "SELECT revision FROM workspace WHERE id = ?1",
            [&workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read issue page revision.".to_owned())?;
    let after = cursor
        .map(|token| -> Result<IssueRecordCursor, String> {
            let bytes = URL_SAFE_NO_PAD
                .decode(token)
                .map_err(|_| "Issue page cursor is invalid.")?;
            if bytes.len() > 512 {
                return Err("Issue page cursor is too large.".into());
            }
            let value: IssueRecordCursor =
                serde_json::from_slice(&bytes).map_err(|_| "Issue page cursor is invalid.")?;
            if value.workspace_id != workspace_id
                || value.revision != revision
                || value.archived != archived
                || Uuid::parse_str(&value.id).is_err()
            {
                return Err("Issue page changed. Reload and try again.".into());
            }
            Ok(value)
        })
        .transpose()?;
    let mut statement = connection
        .prepare(
            "SELECT identifier, position, id, coalesce(archived_at, '') FROM issue
         WHERE workspace_id = ?1 AND (archived_at IS NOT NULL) = ?2
           AND (?3 IS NULL OR
             (?2 = 0 AND (position > ?4 OR (position = ?4 AND id > ?5))) OR
             (?2 = 1 AND (archived_at < ?6 OR
               (archived_at = ?6 AND (position > ?4 OR (position = ?4 AND id > ?5))))))
         ORDER BY CASE WHEN ?2 THEN archived_at END DESC, position, id LIMIT ?7",
        )
        .map_err(|_| "Cannot read issue page.".to_owned())?;
    let rows = statement
        .query_map(
            params![
                workspace_id,
                archived,
                after.as_ref().map(|item| &item.id),
                after.as_ref().map(|item| item.position),
                after.as_ref().map(|item| &item.id),
                after.as_ref().map(|item| &item.archived_at),
                i64::from(limit) + 1
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .map_err(|_| "Cannot read issue page.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read issue page.".to_owned())?;
    let has_more = rows.len() > limit as usize;
    let rows = &rows[..rows.len().min(limit as usize)];
    let identifiers: Vec<String> = rows.iter().map(|row| row.0.clone()).collect();
    let ids =
        serde_json::to_string(&identifiers).map_err(|_| "Cannot read issue page.".to_owned())?;
    let mut items =
        query_issues_window_for_ids(connection, archived, i64::from(limit), Some(&ids))?;
    let order: HashMap<_, _> = identifiers
        .iter()
        .enumerate()
        .map(|(index, id)| (id.as_str(), index))
        .collect();
    items.sort_by_key(|item| order.get(item.id.as_str()).copied().unwrap_or(usize::MAX));
    let next_cursor = if has_more {
        let last = rows.last().ok_or("Cannot read issue page cursor.")?;
        Some(
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&IssueRecordCursor {
                    workspace_id,
                    revision,
                    archived,
                    position: last.1,
                    id: last.2.clone(),
                    archived_at: last.3.clone(),
                })
                .map_err(|_| "Cannot encode issue page cursor.")?,
            ),
        )
    } else {
        None
    };
    Ok(IssueRecordPage { items, next_cursor })
}

#[api_contract_macros::command]
pub fn storage_list_issue_records(
    app: AppHandle,
    identifiers: Vec<String>,
    archived: bool,
) -> Result<Vec<IssueRecord>, String> {
    if identifiers.len() > 200 {
        return Err("Request at most 200 issues.".into());
    }
    let ids = serde_json::to_string(&identifiers).map_err(|_| "Cannot read issues.")?;
    query_issues_window_for_ids(
        &open_database(&document_data_root(&app)?)?,
        archived,
        200,
        Some(&ids),
    )
}

#[api_contract_macros::command]
pub fn storage_list_issue_relations(
    app: AppHandle,
    identifier: String,
) -> Result<Vec<IssueRelation>, String> {
    list_issue_relations(&open_database(&document_data_root(&app)?)?, &identifier)
}

#[api_contract_macros::command]
pub fn storage_create_issue_relation(
    app: AppHandle,
    identifier: String,
    target_identifier: String,
    kind: String,
    target_version: i64,
    mutation: MutationRequest,
) -> Result<IssueRelation, String> {
    create_issue_relation_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        &target_identifier,
        &kind,
        target_version,
        &mutation,
    )
}

fn create_issue_relation_mutation(
    connection: &mut Connection,
    identifier: &str,
    target_identifier: &str,
    kind: &str,
    target_version: i64,
    mutation: &MutationRequest,
) -> Result<IssueRelation, String> {
    mutations::apply(
        connection,
        "issue_relation.create",
        &(identifier, target_identifier, kind, target_version),
        mutation,
        |tx, workspace_id| {
            let resolve = |public_id: &str| -> Result<String, String> {
                tx.query_row(
                    "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
                    params![workspace_id, public_id],
                    |row| row.get(0),
                )
                .map_err(|_| "Issue is missing or archived.".to_owned())
            };
            mutations::check_version(tx, "issue", &resolve(identifier)?, mutation)?;
            mutations::check_version(
                tx,
                "issue",
                &resolve(target_identifier)?,
                &MutationRequest {
                    mutation_id: mutation.mutation_id.clone(),
                    expected_version: Some(target_version),
                    expected_versions: Default::default(),
                },
            )?;
            create_issue_relation_in_tx(tx, identifier, target_identifier, kind)
        },
    )
}

#[api_contract_macros::command]
pub fn storage_delete_issue_relation(
    app: AppHandle,
    identifier: String,
    relation_id: String,
    target_identifier: String,
    target_version: i64,
    mutation: MutationRequest,
) -> Result<(), String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "issue_relation.delete",
        &(
            &identifier,
            &relation_id,
            &target_identifier,
            target_version,
        ),
        &mutation,
        |tx, workspace_id| {
            let (source, target): (String, String) = tx
                .query_row(
                    "SELECT issue.id, CASE WHEN relation.source_id = issue.id THEN relation.target_id ELSE relation.source_id END
                     FROM issue JOIN issue_relation relation ON relation.workspace_id = issue.workspace_id
                     WHERE issue.workspace_id = ?1 AND issue.identifier = ?2 AND relation.id = ?3
                       AND (relation.source_id = issue.id OR relation.target_id = issue.id)",
                    params![workspace_id, identifier, relation_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|_| "Relation no longer exists.".to_owned())?;
            let expected_target: String = tx
                .query_row(
                    "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2",
                    params![workspace_id, target_identifier],
                    |row| row.get(0),
                )
                .map_err(|_| "Target issue no longer exists.".to_owned())?;
            if target != expected_target {
                return Err("CONFLICT: Relation target changed. Reload and try again.".into());
            }
            mutations::check_version(tx, "issue", &source, &mutation)?;
            mutations::check_version(
                tx,
                "issue",
                &target,
                &MutationRequest {
                    mutation_id: mutation.mutation_id.clone(),
                    expected_version: Some(target_version),
                    expected_versions: Default::default(),
                },
            )?;
            delete_issue_relation_in_tx(tx, &identifier, &relation_id)
        },
    )
}

#[api_contract_macros::command]
pub fn storage_list_issue_activity(
    app: AppHandle,
    identifier: String,
    cursor: Option<String>,
) -> Result<IssueActivityPage, String> {
    issue_activity_page(
        &open_database(&document_data_root(&app)?)?,
        &identifier,
        cursor.as_deref(),
    )
}

#[api_contract_macros::command]
pub fn storage_create_issue_comment(
    app: AppHandle,
    identifier: String,
    body: String,
    body_document: Option<serde_json::Value>,
    mutation: MutationRequest,
) -> Result<IssueActivity, String> {
    create_issue_comment_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        &body,
        body_document.as_ref(),
        &mutation,
    )
}

fn change_issue_comment(
    connection: &mut Connection,
    identifier: &str,
    comment_id: &str,
    body: Option<&str>,
    document: Option<&serde_json::Value>,
    mutation: &MutationRequest,
) -> Result<IssueActivity, String> {
    let trimmed = body.map(str::trim);
    if trimmed.is_some_and(|value| value.is_empty() || value.chars().count() > 100_000) {
        return Err("Write a comment of up to 100,000 characters.".into());
    }
    let encoded = if body.is_some() {
        encode_document_for_storage(connection, document)?
    } else {
        None
    };
    let expected = mutation
        .expected_version
        .filter(|value| *value > 0)
        .ok_or("CONFLICT: Reload the comment before changing it.")?;

    mutations::apply(
        connection,
        if body.is_some() {
            "issue.comment.update"
        } else {
            "issue.comment.remove"
        },
        &(identifier, comment_id, trimmed, &encoded),
        mutation,
        |tx, workspace_id| {
            let changed = tx.execute(
                "UPDATE issue_activity
                 SET body = ?1, body_document_json = ?2,
                     edited_at = CASE WHEN ?3 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') ELSE edited_at END,
                     deleted_at = CASE WHEN ?3 THEN NULL ELSE strftime('%Y-%m-%dT%H:%M:%fZ', 'now') END,
                     version = version + 1
                 WHERE id = ?4 AND version = ?5 AND kind = 'comment' AND deleted_at IS NULL
                   AND issue_id IN (
                     SELECT issue.id FROM issue
                     WHERE issue.identifier = ?6 AND issue.workspace_id = ?7
                       AND issue.archived_at IS NULL
                   )",
                params![
                    trimmed.unwrap_or("Comment deleted"),
                    encoded,
                    body.is_some(),
                    comment_id,
                    expected,
                    identifier,
                    workspace_id
                ],
            ).map_err(|_| "Cannot change comment.".to_owned())?;
            if changed == 0 {
                return Err(
                    "CONFLICT: Comment changed or is unavailable. Reload and try again.".into(),
                );
            }
            tx.execute(
                "DELETE FROM comment_search WHERE rowid =
                   (SELECT rowid FROM issue_activity WHERE id = ?1)",
                [comment_id],
            )
            .map_err(|_| "Cannot update comment search.".to_owned())?;
            if body.is_some() {
                tx.execute(
                    "INSERT INTO comment_search(rowid, activity_id, issue_id, workspace_id, body)
                     SELECT activity.rowid, activity.id, activity.issue_id, ?2, activity.body
                     FROM issue_activity activity WHERE activity.id = ?1",
                    params![comment_id, workspace_id],
                )
                .map_err(|_| "Cannot update comment search.".to_owned())?;
            }
            tx.query_row(
                "SELECT id, issue_id, actor_name, kind, body, created_at, body_document_json,
                        edited_at, deleted_at, version FROM issue_activity WHERE id = ?1",
                [comment_id],
                |row| {
                    Ok(IssueActivity {
                        id: row.get(0)?,
                        issue_id: row.get(1)?,
                        actor_name: row.get(2)?,
                        kind: row.get(3)?,
                        body: row.get(4)?,
                        created_at: row.get(5)?,
                        body_document_json: row.get(6)?,
                        edited_at: row.get(7)?,
                        deleted_at: row.get(8)?,
                        version: row.get(9)?,
                        changes: None,
                        mutation_id: None,
                    })
                },
            )
            .map_err(|_| "Cannot read changed comment.".to_owned())
        },
    )
}

#[api_contract_macros::command]
pub fn storage_update_issue_comment(
    app: AppHandle,
    identifier: String,
    comment_id: String,
    body: String,
    body_document: Option<serde_json::Value>,
    mutation: MutationRequest,
) -> Result<IssueActivity, String> {
    change_issue_comment(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        &comment_id,
        Some(&body),
        body_document.as_ref(),
        &mutation,
    )
}

#[api_contract_macros::command]
pub fn storage_remove_issue_comment(
    app: AppHandle,
    identifier: String,
    comment_id: String,
    mutation: MutationRequest,
) -> Result<IssueActivity, String> {
    change_issue_comment(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        &comment_id,
        None,
        None,
        &mutation,
    )
}

fn create_issue_comment_with_mutation(
    connection: &mut Connection,
    identifier: &str,
    body: &str,
    body_document: Option<&serde_json::Value>,
    mutation: &MutationRequest,
) -> Result<IssueActivity, String> {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 100_000 {
        return Err("Write a comment of up to 100,000 characters.".into());
    }
    let encoded_document = encode_document_for_storage(connection, body_document)?;
    mutations::apply(
        connection,
        "issue.comment.create",
        &(identifier, trimmed, &encoded_document),
        mutation,
        |tx, workspace_id| {
            insert_issue_comment(
                tx,
                workspace_id,
                identifier,
                trimmed,
                encoded_document.as_deref(),
            )
        },
    )
}

#[api_contract_macros::command]
pub fn storage_preview_issue_move(
    app: AppHandle,
    identifier: String,
    destination: String,
) -> Result<IssueMovePreview, String> {
    preview_issue_move(
        &open_database(&document_data_root(&app)?)?,
        &identifier,
        &destination,
    )
}

#[api_contract_macros::command]
pub fn storage_move_issue_project(
    app: AppHandle,
    identifier: String,
    destination: String,
    policy: String,
    revision: String,
    mutation: MutationRequest,
) -> Result<Vec<IssueRecord>, String> {
    move_issue_project_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        &destination,
        &policy,
        &revision,
        Some(&mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_archive_issue(
    app: AppHandle,
    identifier: String,
    mutation: MutationRequest,
) -> Result<(), String> {
    archive_issue_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        "all",
        None,
        Some(&mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_archive_issue_with_strategy(
    app: AppHandle,
    identifier: String,
    strategy: String,
    target_identifier: Option<String>,
    mutation: MutationRequest,
) -> Result<(), String> {
    archive_issue_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        &strategy,
        target_identifier.as_deref(),
        Some(&mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_delete_archived_issue(
    app: AppHandle,
    identifier: String,
    strategy: String,
    target_identifier: Option<String>,
    mutation: MutationRequest,
) -> Result<(), String> {
    delete_archived_issue_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        &strategy,
        target_identifier.as_deref(),
        Some(&mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_restore_issue(
    app: AppHandle,
    identifier: String,
    mutation: MutationRequest,
) -> Result<(), String> {
    restore_issue_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        Some(&mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_restore_sub_issue(
    app: AppHandle,
    identifier: String,
    mutation: MutationRequest,
) -> Result<(), String> {
    restore_sub_issue_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        Some(&mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_create_issue(
    app: AppHandle,
    input: IssueInput,
    mutation: MutationRequest,
) -> Result<IssueRecord, String> {
    create_issue_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        input,
        Some(mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_update_issue(
    app: AppHandle,
    identifier: String,
    input: IssueInput,
    order: Option<Vec<String>>,
    mutation: MutationRequest,
) -> Result<IssueRecord, String> {
    update_issue_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &identifier,
        input,
        order,
        Some(mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_reorder_issues(
    app: AppHandle,
    identifiers: Vec<String>,
    mutation: ReorderMutation,
) -> Result<(), String> {
    reorder_issues_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        identifiers,
        Some(mutation),
    )
}

#[cfg(test)]
mod tests;
