use std::collections::HashSet;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use uuid::Uuid;

use crate::storage::{current_workspace_id, document_data_root, open_database};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentResult {
    activity_id: String,
    identifier: String,
    body: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningNodeResult {
    id: String,
    project_name: String,
    name: String,
    kind: String,
}

fn search_planning_nodes(
    connection: &Connection,
    text: &str,
) -> Result<Vec<PlanningNodeResult>, String> {
    let Some(expression) = search_expression(text) else {
        return Ok(Vec::new());
    };
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT node.id, project.name, node.name, node.kind
         FROM planning_node_search
         JOIN planning_node node ON node.rowid = planning_node_search.rowid
         JOIN project ON project.id = node.project_id
         WHERE planning_node_search MATCH ?1 AND project.workspace_id = ?2
           AND node.archived_at IS NULL AND node.deleted_at IS NULL
           AND project.archived_at IS NULL AND project.deleted_at IS NULL
         ORDER BY bm25(planning_node_search), node.name, node.id LIMIT 40",
        )
        .map_err(|_| "Cannot search project structure.".to_owned())?;
    let rows = statement
        .query_map(params![expression, workspace_id], |row| {
            Ok(PlanningNodeResult {
                id: row.get(0)?,
                project_name: row.get(1)?,
                name: row.get(2)?,
                kind: row.get(3)?,
            })
        })
        .map_err(|_| "Cannot search project structure.".to_owned())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Cannot search project structure.".to_owned())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueQuery {
    scope: String,
    triage: bool,
    statuses: Vec<String>,
    excluded_statuses: Vec<String>,
    priorities: Vec<String>,
    excluded_priorities: Vec<String>,
    projects: Vec<String>,
    excluded_projects: Vec<String>,
    #[serde(default)]
    project_statuses: Vec<String>,
    #[serde(default)]
    excluded_project_statuses: Vec<String>,
    #[serde(default)]
    project_priorities: Vec<String>,
    #[serde(default)]
    excluded_project_priorities: Vec<String>,
    #[serde(default)]
    project_labels: Vec<String>,
    #[serde(default)]
    excluded_project_labels: Vec<String>,
    #[serde(default)]
    empty_project_labels: bool,
    #[serde(default)]
    excluded_empty_project_labels: bool,
    #[serde(default)]
    project_leads: Vec<String>,
    #[serde(default)]
    excluded_project_leads: Vec<String>,
    #[serde(default)]
    empty_project_lead: bool,
    #[serde(default)]
    excluded_empty_project_lead: bool,
    #[serde(default)]
    project_milestones: Vec<String>,
    #[serde(default)]
    excluded_project_milestones: Vec<String>,
    #[serde(default)]
    empty_project_milestones: bool,
    #[serde(default)]
    excluded_empty_project_milestones: bool,
    #[serde(default)]
    milestones: Vec<String>,
    #[serde(default)]
    excluded_milestones: Vec<String>,
    #[serde(default)]
    empty_milestone: bool,
    #[serde(default)]
    excluded_empty_milestone: bool,
    labels: Vec<String>,
    excluded_labels: Vec<String>,
    empty_labels: bool,
    excluded_empty_labels: bool,
    due: String,
    text: String,
    #[serde(default)]
    advanced_groups: Vec<IssueFilterGroup>,
    #[serde(default)]
    filter_tree: Option<FilterTree>,
    #[serde(default)]
    sort_field: Option<String>,
    #[serde(default)]
    sort_direction: Option<String>,
    #[serde(default)]
    include_sub_issues: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum FilterTree {
    Group {
        op: String,
        children: Vec<FilterTree>,
    },
    Condition {
        field: String,
        operator: String,
        value: Option<serde_json::Value>,
    },
}

pub(crate) fn filter_tree_condition(tree: &FilterTree) -> Result<String, String> {
    fn compile(
        tree: &FilterTree,
        path: &str,
        depth: usize,
        count: &mut usize,
    ) -> Result<String, String> {
        if depth > 5 {
            return Err("Filter nesting is too deep.".into());
        }
        match tree {
            FilterTree::Group { op, children } => {
                if !matches!(op.as_str(), "and" | "or") {
                    return Err("Unknown filter group operator.".into());
                }
                if children.is_empty() {
                    return Ok("1 = 1".into());
                }
                let parts = children
                    .iter()
                    .enumerate()
                    .map(|(index, child)| {
                        compile(
                            child,
                            &format!("{path}.children[{index}]"),
                            depth + 1,
                            count,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!(
                    "({})",
                    parts.join(if op == "and" { " AND " } else { " OR " })
                ))
            }
            FilterTree::Condition {
                field,
                operator,
                value,
            } => {
                *count += 1;
                if *count > 50 {
                    return Err("Too many filter conditions.".into());
                }
                if value
                    .as_ref()
                    .is_some_and(|value| value.to_string().len() > 1024)
                {
                    return Err("Filter value is too long.".into());
                }
                if field == "labelIds" {
                    let ids = value
                        .as_ref()
                        .and_then(serde_json::Value::as_array)
                        .filter(|ids| {
                            !ids.is_empty()
                                && ids.len() <= 20
                                && ids.iter().all(|id| {
                                    id.as_str().is_some_and(|id| Uuid::parse_str(id).is_ok())
                                })
                        })
                        .ok_or("Choose label IDs for this filter.")?;
                    let parameter = format!("json_extract(?39, '{path}.value')");
                    let matches = format!("SELECT count(*) FROM issue_label WHERE issue_id = issue.id AND label_id IN (SELECT value FROM json_each({parameter}))");
                    return match operator.as_str() {
                        "hasAny" => Ok(format!("({matches}) > 0")),
                        "hasAll" => Ok(format!("({matches}) = {}", ids.len())),
                        "hasNone" => Ok(format!("({matches}) = 0")),
                        _ => Err("Invalid label filter operator.".into()),
                    };
                }
                let column = match field.as_str() {
                    "projectId" => "issue.project_id",
                    "nodeId" => {
                        "(SELECT node_id FROM planning_node_issue WHERE issue_id = issue.id)"
                    }
                    "milestoneId" => "issue.milestone_id",
                    "cycleId" => {
                        "(SELECT cycle_id FROM cycle_issue WHERE issue_id = issue.id LIMIT 1)"
                    }
                    "statusId" => "issue.status",
                    "statusCategory" => "status_def.category",
                    "priority" => "issue.priority",
                    "intakeState" => "issue.triage",
                    "title" => "issue.title",
                    "currentIdentifier" => "issue.identifier",
                    "estimatePoints" => "issue.estimate_value",
                    "dueDate" => "issue.due_date",
                    "createdAt" => "issue.created_at",
                    "updatedAt" => "issue.updated_at",
                    "parentIssueId" => "issue.parent_id",
                    _ => return Err("Unknown filter field.".into()),
                };
                let parameter = format!("json_extract(?39, '{path}.value')");
                let is_text = matches!(field.as_str(), "title" | "currentIdentifier");
                let nullable = matches!(
                    field.as_str(),
                    "projectId"
                        | "nodeId"
                        | "milestoneId"
                        | "cycleId"
                        | "estimatePoints"
                        | "dueDate"
                        | "parentIssueId"
                );
                let numeric = matches!(field.as_str(), "estimatePoints" | "intakeState");
                let date = matches!(field.as_str(), "dueDate" | "createdAt" | "updatedAt");
                let required = || {
                    value
                        .as_ref()
                        .ok_or_else(|| "Choose a filter value.".to_owned())
                };
                match operator.as_str() {
                    "isSet" | "isUnset" if nullable => Ok(format!("{column} IS {}NULL", if operator == "isSet" { "NOT " } else { "" })),
                    "eq" | "neq" if required()?.is_string() || (numeric && required()?.is_number()) => Ok(format!("{column} {} {parameter}", if operator == "eq" { "=" } else { "!=" })),
                    "in" | "notIn" if required()?.as_array().is_some_and(|values| !values.is_empty() && values.len() <= 20 && values.iter().all(|value| value.is_string() || (numeric && value.is_number()))) => Ok(format!("{column} {}IN (SELECT value FROM json_each({parameter}))", if operator == "in" { "" } else { "NOT " })),
                    "contains" | "equals" if is_text && required()?.is_string() => Ok(if operator == "contains" {
                        format!("instr(lower({column}), lower({parameter})) > 0")
                    } else { format!("{column} = {parameter} COLLATE NOCASE") }),
                    "gt" | "gte" | "lt" | "lte" if numeric && required()?.is_number() => Ok(format!("{column} {} {parameter}", match operator.as_str() { "gt" => ">", "gte" => ">=", "lt" => "<", _ => "<=" })),
                    "before" | "after" | "on" if date && required()?.is_string() => Ok(format!("substr({column}, 1, 10) {} {parameter}", match operator.as_str() { "before" => "<", "after" => ">", _ => "=" })),
                    "between" if date && required()?.as_array().is_some_and(|values| values.len() == 2 && values.iter().all(|value| value.is_string())) => Ok(format!("substr({column}, 1, 10) BETWEEN json_extract(?39, '{path}.value[0]') AND json_extract(?39, '{path}.value[1]')")),
                    _ => Err("Invalid filter operator or value.".into()),
                }
            }
        }
    }
    compile(tree, "$", 0, &mut 0)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssueFilterGroup {
    mode: String,
    rules: Vec<IssueFilterRule>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssueFilterRule {
    field: String,
    operator: String,
    value: Option<String>,
    end: Option<String>,
}

fn advanced_conditions(groups: &[IssueFilterGroup]) -> Result<String, String> {
    let mut conditions = Vec::with_capacity(groups.len());
    for group in groups {
        let mut rule_conditions = Vec::with_capacity(group.rules.len());
        for rule in &group.rules {
            let condition = match rule.field.as_str() {
                "due" => "issue.due_date",
                "start" => "issue.start_date",
                "created" => "substr(issue.created_at, 1, 10)",
                "updated" => "substr(issue.updated_at, 1, 10)",
                "triaged" => "substr(issue.triaged_at, 1, 10)",
                "parent" => "issue.parent_id",
                "subissues" => "(SELECT id FROM issue child WHERE child.parent_id = issue.id AND child.archived_at IS NULL LIMIT 1)",
                "blocks" => "(SELECT id FROM issue_relation WHERE source_id = issue.id AND kind = 'blocks' LIMIT 1)",
                "blockedBy" => "(SELECT id FROM issue_relation WHERE target_id = issue.id AND kind = 'blocks' LIMIT 1)",
                "related" => "(SELECT id FROM issue_relation WHERE (source_id = issue.id OR target_id = issue.id) AND kind = 'related' LIMIT 1)",
                "duplicates" => "(SELECT id FROM issue_relation WHERE (source_id = issue.id OR target_id = issue.id) AND kind = 'duplicates' LIMIT 1)",
                _ => return Err("Unknown advanced filter field.".into()),
            };
            let comparison = match rule.operator.as_str() {
                "has" => format!("{condition} IS NOT NULL"),
                "none" => format!("{condition} IS NULL"),
                "before" | "after" | "between"
                    if matches!(
                        rule.field.as_str(),
                        "due" | "start" | "created" | "updated" | "triaged"
                    ) =>
                {
                    let value = rule.value.as_deref().ok_or("Choose a filter date.")?;
                    if !crate::storage::valid_date(value) {
                        return Err("Choose a valid filter date.".into());
                    }
                    let escaped = value.replace('\'', "''");
                    match rule.operator.as_str() {
                        "before" => format!("{condition} < '{escaped}'"),
                        "after" => format!("{condition} > '{escaped}'"),
                        _ => {
                            let end = rule.end.as_deref().ok_or("Choose a second filter date.")?;
                            if !crate::storage::valid_date(end) || value > end {
                                return Err("Choose a valid filter range.".into());
                            }
                            format!(
                                "{condition} BETWEEN '{escaped}' AND '{}'",
                                end.replace('\'', "''")
                            )
                        }
                    }
                }
                _ => return Err("Unknown advanced filter operator.".into()),
            };
            rule_conditions.push(comparison);
        }
        conditions.push(format!(
            "({})",
            rule_conditions.join(if group.mode == "all" { " AND " } else { " OR " })
        ));
    }
    Ok(if conditions.is_empty() {
        "1 = 1".into()
    } else {
        conditions.join(" AND ")
    })
}

fn search_expression(text: &str) -> Option<String> {
    let terms: Vec<_> = text
        .chars()
        .take(200)
        .collect::<String>()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|term| !term.is_empty())
        .take(8)
        .map(|term| format!("\"{term}\"*"))
        .collect();

    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" AND "))
    }
}

pub(crate) fn search_issues(connection: &Connection, text: &str) -> Result<Vec<String>, String> {
    let Some(expression) = search_expression(text) else {
        return Ok(Vec::new());
    };
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT issue.identifier
             FROM issue_search JOIN issue ON issue.rowid = issue_search.rowid
             WHERE issue_search MATCH ?1 AND issue.workspace_id = ?2
               AND issue.archived_at IS NULL
             ORDER BY CASE WHEN issue.identifier = ?3 COLLATE NOCASE THEN 0 ELSE 1 END,
                      bm25(issue_search, 0, 0, 4, 2, 1, 1, 1), issue.position, issue.id
             LIMIT 40",
        )
        .map_err(|_| "Cannot search issues.".to_owned())?;
    let mut results = statement
        .query_map(params![expression, workspace_id, text.trim()], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|_| "Cannot search issues.".to_owned())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Cannot search issues.".to_owned())?;
    let alias_match: Option<String> = connection
        .query_row(
            "SELECT issue.identifier FROM issue_alias alias JOIN issue ON issue.id = alias.issue_id
             WHERE alias.workspace_id = ?1 AND alias.alias = ?2 COLLATE NOCASE
               AND issue.archived_at IS NULL",
            params![workspace_id, text.trim()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot search previous identifiers.".to_owned())?;
    if let Some(identifier) = alias_match {
        results.retain(|current| current != &identifier);
        results.insert(0, identifier);
    }
    let mut seen: HashSet<String> = results.iter().cloned().collect();
    let mut comment_query = connection
        .prepare(
            "SELECT issue.identifier FROM comment_search
             JOIN issue ON issue.id = comment_search.issue_id
             WHERE comment_search MATCH ?1 AND comment_search.workspace_id = ?2
               AND issue.archived_at IS NULL
             ORDER BY bm25(comment_search), issue.identifier LIMIT 40",
        )
        .map_err(|_| "Cannot search issue comments.".to_owned())?;
    let comment_matches = comment_query
        .query_map(params![expression, workspace_id], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|_| "Cannot search issue comments.".to_owned())?;
    for identifier in comment_matches {
        if results.len() >= 40 {
            break;
        }
        let identifier = identifier.map_err(|_| "Cannot search issue comments.".to_owned())?;
        if seen.insert(identifier.clone()) {
            results.push(identifier);
        }
    }
    Ok(results)
}

fn search_projects(connection: &Connection, text: &str) -> Result<Vec<String>, String> {
    let Some(expression) = search_expression(text) else {
        return Ok(Vec::new());
    };
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT project.id FROM project_search
             JOIN project ON project.rowid = project_search.rowid
             WHERE project_search MATCH ?1 AND project.workspace_id = ?2
             ORDER BY CASE WHEN project.name = ?3 COLLATE NOCASE THEN 0 ELSE 1 END,
                      bm25(project_search, 0, 0, 4, 2, 1, 1, 1), project.name, project.id
             LIMIT 40",
        )
        .map_err(|_| "Cannot search projects.".to_owned())?;
    let results = statement
        .query_map(params![expression, workspace_id, text.trim()], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|_| "Cannot search projects.".to_owned())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Cannot search projects.".to_owned())?;
    Ok(results)
}

fn search_comments(connection: &Connection, text: &str) -> Result<Vec<CommentResult>, String> {
    let Some(expression) = search_expression(text) else {
        return Ok(Vec::new());
    };
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT comment_search.activity_id, issue.identifier,
                    snippet(comment_search, 3, '', '', '…', 18)
             FROM comment_search JOIN issue ON issue.id = comment_search.issue_id
             WHERE comment_search MATCH ?1 AND comment_search.workspace_id = ?2
               AND issue.archived_at IS NULL
             ORDER BY bm25(comment_search), issue.identifier LIMIT 15",
        )
        .map_err(|_| "Cannot search comments.".to_owned())?;
    let matches = statement
        .query_map(params![expression, workspace_id], |row| {
            Ok(CommentResult {
                activity_id: row.get(0)?,
                identifier: row.get(1)?,
                body: row.get(2)?,
            })
        })
        .map_err(|_| "Cannot search comments.".to_owned())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Cannot search comments.".to_owned())?;
    Ok(matches)
}

fn find_comment(
    connection: &Connection,
    identifier: &str,
    activity_id: &str,
) -> Result<Option<crate::issues::IssueActivity>, String> {
    let workspace_id = current_workspace_id(connection)?;
    connection
        .query_row(
            "SELECT activity.id, activity.issue_id, activity.actor_name, activity.kind,
                    activity.body, activity.created_at, activity.body_document_json,
                    activity.edited_at, activity.deleted_at, activity.version
             FROM issue_activity activity JOIN issue ON issue.id = activity.issue_id
             WHERE activity.id = ?1 AND activity.kind = 'comment'
               AND issue.identifier = ?2 AND issue.workspace_id = ?3
               AND issue.archived_at IS NULL",
            params![activity_id, identifier, workspace_id],
            |row| {
                Ok(crate::issues::IssueActivity {
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
        .optional()
        .map_err(|_| "Cannot find comment.".to_owned())
}

pub(crate) fn valid_advanced_groups(groups: &[IssueFilterGroup]) -> bool {
    groups.len() <= 4
        && groups.iter().all(|group| {
            matches!(group.mode.as_str(), "all" | "any")
                && !group.rules.is_empty()
                && group.rules.len() <= 6
                && group.rules.iter().all(|rule| {
                    let date = matches!(
                        rule.field.as_str(),
                        "due" | "start" | "created" | "updated" | "triaged"
                    );
                    let relation = matches!(
                        rule.field.as_str(),
                        "parent" | "subissues" | "blocks" | "blockedBy" | "related" | "duplicates"
                    );
                    (date || relation)
                        && match rule.operator.as_str() {
                            "has" | "none" => true,
                            "before" | "after" | "between" if date => {
                                rule.value
                                    .as_deref()
                                    .is_some_and(crate::storage::valid_date)
                                    && (rule.operator != "between"
                                        || rule.end.as_deref().is_some_and(|end| {
                                            crate::storage::valid_date(end)
                                                && rule
                                                    .value
                                                    .as_deref()
                                                    .is_some_and(|start| start <= end)
                                        }))
                            }
                            _ => false,
                        }
                })
        })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueIdPage {
    ids: Vec<String>,
    total_count: i64,
    next_cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct IssueCursor {
    sort_key: String,
    id: String,
    query_hash: String,
    total_count: i64,
    workspace_revision: i64,
}

fn issue_query_hash(workspace_id: &str, query: &IssueQuery) -> Result<String, String> {
    let bytes =
        serde_json::to_vec(&(workspace_id, query)).map_err(|_| "Cannot encode issue query.")?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
fn query_issue_ids(connection: &Connection, query: &IssueQuery) -> Result<Vec<String>, String> {
    Ok(query_issue_ids_window(connection, query, -1)?.ids)
}

#[cfg(test)]
fn query_issue_ids_window(
    connection: &Connection,
    query: &IssueQuery,
    limit: i64,
) -> Result<IssueIdPage, String> {
    query_issue_ids_window_after(connection, query, limit, None)
}

fn query_issue_ids_window_after(
    connection: &Connection,
    query: &IssueQuery,
    limit: i64,
    cursor: Option<&str>,
) -> Result<IssueIdPage, String> {
    if !valid_advanced_groups(&query.advanced_groups) {
        return Err("Too many advanced filter conditions.".into());
    }
    if !matches!(query.scope.as_str(), "active" | "all")
        || !matches!(query.due.as_str(), "any" | "has" | "none")
        || query.sort_field.as_ref().is_some_and(|field| {
            !matches!(
                field.as_str(),
                "position" | "title" | "status" | "priority" | "dueDate" | "updatedAt"
            )
        })
        || query
            .sort_direction
            .as_ref()
            .is_some_and(|direction| !matches!(direction.as_str(), "asc" | "desc"))
        || [
            query.statuses.len(),
            query.excluded_statuses.len(),
            query.priorities.len(),
            query.excluded_priorities.len(),
            query.projects.len(),
            query.excluded_projects.len(),
            query.project_statuses.len(),
            query.excluded_project_statuses.len(),
            query.project_priorities.len(),
            query.excluded_project_priorities.len(),
            query.project_labels.len(),
            query.excluded_project_labels.len(),
            query.project_leads.len(),
            query.excluded_project_leads.len(),
            query.project_milestones.len(),
            query.excluded_project_milestones.len(),
            query.milestones.len(),
            query.excluded_milestones.len(),
            query.labels.len(),
            query.excluded_labels.len(),
        ]
        .iter()
        .any(|count| *count > 20)
        || query
            .statuses
            .iter()
            .chain(query.excluded_statuses.iter())
            .any(|value| value.trim().is_empty() || value.chars().count() > 32)
        || query
            .projects
            .iter()
            .chain(query.excluded_projects.iter())
            .any(|value| value.trim().is_empty() || value.chars().count() > 80)
        || query
            .labels
            .iter()
            .chain(query.excluded_labels.iter())
            .chain(query.project_labels.iter())
            .chain(query.excluded_project_labels.iter())
            .any(|value| value.trim().is_empty() || value.chars().count() > 80)
        || query
            .project_milestones
            .iter()
            .chain(query.excluded_project_milestones.iter())
            .any(|value| value.trim().is_empty() || value.chars().count() > 240)
        || query
            .project_leads
            .iter()
            .chain(query.excluded_project_leads.iter())
            .chain(query.milestones.iter())
            .chain(query.excluded_milestones.iter())
            .any(|value| Uuid::parse_str(value).is_err())
        || query
            .priorities
            .iter()
            .chain(query.excluded_priorities.iter())
            .chain(query.project_priorities.iter())
            .chain(query.excluded_project_priorities.iter())
            .any(|value| {
                !matches!(
                    value.as_str(),
                    "No priority" | "Low" | "Medium" | "High" | "Urgent"
                )
            })
        || query
            .project_statuses
            .iter()
            .chain(query.excluded_project_statuses.iter())
            .any(|value| {
                !matches!(
                    value.as_str(),
                    "Backlog" | "Planned" | "In Progress" | "Completed" | "Canceled"
                )
            })
    {
        return Err("Issue query contains an invalid filter.".into());
    }

    let expression = if query.text.trim().is_empty() {
        None
    } else {
        match search_expression(&query.text) {
            Some(expression) => Some(expression),
            None => {
                return Ok(IssueIdPage {
                    ids: Vec::new(),
                    total_count: 0,
                    next_cursor: None,
                })
            }
        }
    };
    let statuses = serde_json::to_string(&query.statuses).map_err(|_| "Cannot query issues.")?;
    let excluded_statuses =
        serde_json::to_string(&query.excluded_statuses).map_err(|_| "Cannot query issues.")?;
    let priorities =
        serde_json::to_string(&query.priorities).map_err(|_| "Cannot query issues.")?;
    let excluded_priorities =
        serde_json::to_string(&query.excluded_priorities).map_err(|_| "Cannot query issues.")?;
    let projects = serde_json::to_string(&query.projects).map_err(|_| "Cannot query issues.")?;
    let excluded_projects =
        serde_json::to_string(&query.excluded_projects).map_err(|_| "Cannot query issues.")?;
    let project_statuses =
        serde_json::to_string(&query.project_statuses).map_err(|_| "Cannot query issues.")?;
    let excluded_project_statuses = serde_json::to_string(&query.excluded_project_statuses)
        .map_err(|_| "Cannot query issues.")?;
    let project_priorities =
        serde_json::to_string(&query.project_priorities).map_err(|_| "Cannot query issues.")?;
    let excluded_project_priorities = serde_json::to_string(&query.excluded_project_priorities)
        .map_err(|_| "Cannot query issues.")?;
    let labels = serde_json::to_string(&query.labels).map_err(|_| "Cannot query issues.")?;
    let excluded_labels =
        serde_json::to_string(&query.excluded_labels).map_err(|_| "Cannot query issues.")?;
    let project_labels =
        serde_json::to_string(&query.project_labels).map_err(|_| "Cannot query issues.")?;
    let excluded_project_labels = serde_json::to_string(&query.excluded_project_labels)
        .map_err(|_| "Cannot query issues.")?;
    let project_leads =
        serde_json::to_string(&query.project_leads).map_err(|_| "Cannot query issues.")?;
    let excluded_project_leads =
        serde_json::to_string(&query.excluded_project_leads).map_err(|_| "Cannot query issues.")?;
    let project_milestones =
        serde_json::to_string(&query.project_milestones).map_err(|_| "Cannot query issues.")?;
    let excluded_project_milestones = serde_json::to_string(&query.excluded_project_milestones)
        .map_err(|_| "Cannot query issues.")?;
    let milestones =
        serde_json::to_string(&query.milestones).map_err(|_| "Cannot query issues.")?;
    let excluded_milestones =
        serde_json::to_string(&query.excluded_milestones).map_err(|_| "Cannot query issues.")?;
    let workspace_id = current_workspace_id(connection)?;
    let query_hash = issue_query_hash(&workspace_id, query)?;
    let workspace_revision: i64 = connection
        .query_row(
            "SELECT revision FROM workspace WHERE id = ?1",
            [&workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read workspace revision.".to_owned())?;
    // A keyset cursor is valid only for the exact filter and revision that produced it.
    // Reject stale cursors instead of silently skipping or repeating changed rows.
    let after = if let Some(token) = cursor {
        let bytes = URL_SAFE_NO_PAD
            .decode(token)
            .map_err(|_| "Issue page cursor is invalid.")?;
        if bytes.len() > 512 {
            return Err("Issue page cursor is too large.".into());
        }
        let value: IssueCursor =
            serde_json::from_slice(&bytes).map_err(|_| "Issue page cursor is invalid.")?;
        if value.query_hash != query_hash
            || value.workspace_revision != workspace_revision
            || Uuid::parse_str(&value.id).is_err()
            || value.total_count < 0
        {
            return Err("Issue page cursor does not match this view.".into());
        }
        Some(value)
    } else {
        None
    };
    let sort_expression = match query.sort_field.as_deref().unwrap_or("position") {
        "title" => "lower(issue.title)",
        "status" => "lower(issue.status)",
        "priority" => "printf('%02d', CASE issue.priority WHEN 'Urgent' THEN 0 WHEN 'High' THEN 1 WHEN 'Medium' THEN 2 WHEN 'Low' THEN 3 ELSE 4 END)",
        "dueDate" => "COALESCE(issue.due_date, '9999-12-31')",
        "updatedAt" => "issue.updated_at",
        _ => "printf('%020lld', issue.position)",
    };
    let direction = if query.sort_direction.as_deref() == Some("desc") {
        "DESC"
    } else {
        "ASC"
    };
    let comparison = if direction == "DESC" { "<" } else { ">" };
    // SQL fragments below come only from the validated allowlists above; user values remain bound.
    let advanced = advanced_conditions(&query.advanced_groups)?;
    let tree = query
        .filter_tree
        .as_ref()
        .map(filter_tree_condition)
        .transpose()?
        .unwrap_or_else(|| "1 = 1".into());
    let tree_json =
        serde_json::to_string(&query.filter_tree).map_err(|_| "Cannot query issues.")?;
    let mut statement = connection
        .prepare(&format!(
            "SELECT issue.identifier, count(*) OVER(), {sort_expression}, issue.id
             FROM issue
             LEFT JOIN project ON project.id = issue.project_id
             LEFT JOIN workflow_status status_def
               ON status_def.workspace_id = issue.workspace_id AND status_def.name = issue.status
             WHERE issue.workspace_id = ?1 AND issue.archived_at IS NULL AND issue.triage = ?2
               AND (?2 = 1 OR ?3 = 'all'
                    OR COALESCE(status_def.category, 'Started') NOT IN ('Completed', 'Canceled'))
               AND (?40 = 1 OR issue.parent_id IS NULL)
               AND (?4 = '[]' OR issue.status IN (SELECT value FROM json_each(?4)))
               AND (?5 = '[]' OR issue.status NOT IN (SELECT value FROM json_each(?5)))
               AND (?6 = '[]' OR issue.priority IN (SELECT value FROM json_each(?6)))
               AND (?7 = '[]' OR issue.priority NOT IN (SELECT value FROM json_each(?7)))
               AND (?8 = '[]' OR COALESCE(project.name, 'Unassigned') IN (SELECT value FROM json_each(?8)))
               AND (?9 = '[]' OR COALESCE(project.name, 'Unassigned') NOT IN (SELECT value FROM json_each(?9)))
               AND (?16 = '[]' OR project.status IN (SELECT value FROM json_each(?16)))
               AND (?17 = '[]' OR project.status IS NULL OR project.status NOT IN (SELECT value FROM json_each(?17)))
               AND (?18 = '[]' OR project.priority IN (SELECT value FROM json_each(?18)))
               AND (?19 = '[]' OR project.priority IS NULL OR project.priority NOT IN (SELECT value FROM json_each(?19)))
               AND ((?20 = '[]' AND ?22 = 0) OR EXISTS (
                 SELECT 1 FROM project_label JOIN label ON label.id = project_label.label_id
                 WHERE project_label.project_id = project.id AND label.name IN (SELECT value FROM json_each(?20))
               ) OR (?22 = 1 AND NOT EXISTS(SELECT 1 FROM project_label WHERE project_id = project.id)))
               AND (?21 = '[]' OR NOT EXISTS (
                 SELECT 1 FROM project_label JOIN label ON label.id = project_label.label_id
                 WHERE project_label.project_id = project.id AND label.name IN (SELECT value FROM json_each(?21))
               ))
               AND (?23 = 0 OR EXISTS(SELECT 1 FROM project_label WHERE project_id = project.id))
               AND ((?24 = '[]' AND ?26 = 0)
                    OR project.lead_actor_id IN (SELECT value FROM json_each(?24))
                    OR (?26 = 1 AND project.lead_actor_id IS NULL))
               AND (?25 = '[]' OR project.lead_actor_id IS NULL
                    OR project.lead_actor_id NOT IN (SELECT value FROM json_each(?25)))
               AND (?27 = 0 OR project.lead_actor_id IS NOT NULL)
               AND ((?28 = '[]' AND ?30 = 0) OR EXISTS (
                 SELECT 1 FROM milestone WHERE milestone.project_id = project.id
                   AND milestone.title IN (SELECT value FROM json_each(?28))
               ) OR (?30 = 1 AND NOT EXISTS (
                 SELECT 1 FROM milestone WHERE milestone.project_id = project.id
               )))
               AND (?29 = '[]' OR NOT EXISTS (
                 SELECT 1 FROM milestone WHERE milestone.project_id = project.id
                   AND milestone.title IN (SELECT value FROM json_each(?29))
               ))
               AND (?31 = 0 OR EXISTS (
                 SELECT 1 FROM milestone WHERE milestone.project_id = project.id
               ))
               AND ((?32 = '[]' AND ?34 = 0)
                    OR issue.milestone_id IN (SELECT value FROM json_each(?32))
                    OR (?34 = 1 AND issue.milestone_id IS NULL))
               AND (?33 = '[]' OR issue.milestone_id IS NULL
                    OR issue.milestone_id NOT IN (SELECT value FROM json_each(?33)))
               AND (?35 = 0 OR issue.milestone_id IS NOT NULL)
               AND ((?10 = '[]' AND ?14 = 0) OR EXISTS (
                 SELECT 1 FROM issue_label JOIN label ON label.id = issue_label.label_id
                 WHERE issue_label.issue_id = issue.id AND label.name IN (SELECT value FROM json_each(?10))
               ) OR (?14 = 1 AND NOT EXISTS(SELECT 1 FROM issue_label WHERE issue_id = issue.id)))
               AND (?11 = '[]' OR NOT EXISTS (
                 SELECT 1 FROM issue_label JOIN label ON label.id = issue_label.label_id
                 WHERE issue_label.issue_id = issue.id AND label.name IN (SELECT value FROM json_each(?11))
               ))
               AND (?15 = 0 OR EXISTS(SELECT 1 FROM issue_label WHERE issue_id = issue.id))
               AND (?12 = 'any' OR (?12 = 'has' AND issue.due_date IS NOT NULL)
                                   OR (?12 = 'none' AND issue.due_date IS NULL))
               AND (?13 IS NULL OR issue.rowid IN (
                 SELECT rowid FROM issue_search WHERE issue_search MATCH ?13
               ))
               AND ({advanced})
               AND (?39 IS NULL OR ({tree}))
               AND (?37 IS NULL OR {sort_expression} {comparison} ?37 OR ({sort_expression} = ?37 AND issue.id {comparison} ?38))
             ORDER BY {sort_expression} {direction}, issue.id {direction}
             LIMIT ?36",
        ))
        .map_err(|_| "Cannot query issues.".to_owned())?;
    let results = statement
        .query_map(
            params![
                workspace_id,
                query.triage,
                query.scope,
                statuses,
                excluded_statuses,
                priorities,
                excluded_priorities,
                projects,
                excluded_projects,
                labels,
                excluded_labels,
                query.due,
                expression,
                query.empty_labels,
                query.excluded_empty_labels,
                project_statuses,
                excluded_project_statuses,
                project_priorities,
                excluded_project_priorities,
                project_labels,
                excluded_project_labels,
                query.empty_project_labels,
                query.excluded_empty_project_labels,
                project_leads,
                excluded_project_leads,
                query.empty_project_lead,
                query.excluded_empty_project_lead,
                project_milestones,
                excluded_project_milestones,
                query.empty_project_milestones,
                query.excluded_empty_project_milestones,
                milestones,
                excluded_milestones,
                query.empty_milestone,
                query.excluded_empty_milestone,
                limit,
                after.as_ref().map(|value| value.sort_key.as_str()),
                after.as_ref().map(|value| value.id.as_str()),
                tree_json,
                query.include_sub_issues.unwrap_or(true),
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
        .map_err(|_| "Cannot query issues.".to_owned())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Cannot query issues.".to_owned())?;
    let remaining = results.first().map_or(0, |(_, count, _, _)| *count);
    let total_count = after.as_ref().map_or(remaining, |value| value.total_count);
    let next_cursor = if limit > 0 && results.len() == limit as usize && remaining > limit {
        let (_, _, sort_key, id) = results.last().expect("page is nonempty");
        Some(
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&IssueCursor {
                    sort_key: sort_key.clone(),
                    id: id.clone(),
                    query_hash,
                    total_count,
                    workspace_revision,
                })
                .map_err(|_| "Cannot encode issue page cursor.")?,
            ),
        )
    } else {
        None
    };
    Ok(IssueIdPage {
        total_count,
        ids: results.into_iter().map(|(id, _, _, _)| id).collect(),
        next_cursor,
    })
}

#[api_contract_macros::command]
pub fn storage_search_issues(app: AppHandle, text: String) -> Result<Vec<String>, String> {
    search_issues(&open_database(&document_data_root(&app)?)?, &text)
}

#[api_contract_macros::command]
pub fn storage_search_projects(app: AppHandle, text: String) -> Result<Vec<String>, String> {
    search_projects(&open_database(&document_data_root(&app)?)?, &text)
}

#[api_contract_macros::command]
pub fn storage_search_planning_nodes(
    app: AppHandle,
    text: String,
) -> Result<Vec<PlanningNodeResult>, String> {
    search_planning_nodes(&open_database(&document_data_root(&app)?)?, &text)
}

#[api_contract_macros::command]
pub fn storage_search_comments(app: AppHandle, text: String) -> Result<Vec<CommentResult>, String> {
    search_comments(&open_database(&document_data_root(&app)?)?, &text)
}

#[api_contract_macros::command]
pub fn storage_find_comment(
    app: AppHandle,
    identifier: String,
    activity_id: String,
) -> Result<Option<crate::issues::IssueActivity>, String> {
    find_comment(
        &open_database(&document_data_root(&app)?)?,
        &identifier,
        &activity_id,
    )
}

#[api_contract_macros::command]
pub fn storage_query_issue_id_page(
    app: AppHandle,
    query: IssueQuery,
    limit: u32,
    cursor: Option<String>,
) -> Result<IssueIdPage, String> {
    if limit == 0 || limit > 200 {
        return Err("Choose an issue page size between 1 and 200.".into());
    }
    query_issue_ids_window_after(
        &open_database(&document_data_root(&app)?)?,
        &query,
        i64::from(limit),
        cursor.as_deref(),
    )
}

#[cfg(test)]
mod tests;
