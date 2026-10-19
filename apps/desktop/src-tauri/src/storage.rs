use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::content::{decode_document, encode_document, encode_document_for_storage};
use crate::mutations::{self, MutationRequest};

const DATA_FOLDER: &str = "Initera";
const LEGACY_DATA_FOLDER: &str = "pm-app-v1";

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    workspace_id: String,
    actor_id: String,
    user_name: String,
    workspace_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    data_root: String,
    contract_version: u8,
    workspace_revision: i64,
    identity_version: Option<i64>,
    identity: Option<Identity>,
    workspaces: Vec<WorkspaceEntry>,
    restore_error: Option<String>,
    restored_backup: bool,
    recovered_timers: i64,
}

fn identity_version(connection: &Connection) -> Result<Option<i64>, String> {
    let Some(identity) = read_identity(connection)? else {
        return Ok(None);
    };
    let encoded = serde_json::to_vec(&identity)
        .map_err(|_| "Cannot encode workspace identity.".to_owned())?;
    // Identity has no mutable row version; expose a stable, JS-safe token for optimistic checks.
    let digest = Sha256::digest(encoded);
    Ok(Some(
        (i64::from_be_bytes(digest[..8].try_into().unwrap()) & ((1_i64 << 53) - 1)).max(1),
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEntry {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRecord {
    id: String,
    version: i64,
    workspace_id: String,
    name: String,
    name_markdown: String,
    identifier_prefix: String,
    icon: String,
    color: String,
    created_at: String,
    updated_at: String,
    summary: String,
    description: String,
    description_document: Option<serde_json::Value>,
    status: String,
    priority: String,
    lead_actor_id: Option<String>,
    start_date: Option<String>,
    target_date: Option<String>,
    labels: Vec<String>,
    resources: Vec<ProjectResource>,
    milestones: Vec<MilestoneRecord>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPage {
    items: Vec<ProjectRecord>,
    next_cursor: Option<String>,
    total_count: i64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectCursor {
    workspace_id: String,
    revision: i64,
    query_hash: String,
    sort_key: String,
    id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLifecycleEntry {
    id: String,
    name: String,
    state: String,
    version: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLifecyclePreview {
    active_issues: i64,
    nodes: i64,
    milestones: i64,
    running_timers: i64,
    cycle_dependencies: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResource {
    id: String,
    title: String,
    url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdate {
    id: String,
    project_id: String,
    actor_name: String,
    body: String,
    kind: String,
    created_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdatePage {
    items: Vec<ProjectUpdate>,
    next_cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectUpdateCursor {
    project_id: String,
    created_at: String,
    row_id: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneInput {
    #[serde(default)]
    id: Option<String>,
    title: String,
    description: String,
    target_date: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneRecord {
    id: String,
    title: String,
    description: String,
    target_date: Option<String>,
    archived_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovedMilestone {
    id: String,
    title: String,
    description: String,
    target_date: Option<String>,
    detached_issue_count: i64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneRestoreReport {
    reattached: usize,
    skipped: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    name: String,
    #[serde(default)]
    name_markdown: String,
    identifier_prefix: String,
    #[serde(default = "default_project_icon")]
    icon: String,
    #[serde(default = "default_project_color")]
    color: String,
    summary: String,
    description: String,
    #[serde(default)]
    description_document: Option<serde_json::Value>,
    status: String,
    priority: String,
    #[serde(default)]
    lead_actor_id: Option<String>,
    start_date: Option<String>,
    target_date: Option<String>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    resources: Vec<ProjectResource>,
    milestones: Vec<MilestoneInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreferencesRecord {
    theme: String,
    issue_scope: String,
    issue_layout: String,
    board_group_by: String,
    group_issues_by_status: bool,
    show_issue_priority: bool,
    show_issue_project: bool,
    #[serde(default = "default_estimate_scale")]
    estimate_scale: String,
    #[serde(default = "default_timer_on_exit")]
    timer_on_exit: String,
    #[serde(default)]
    concurrent_timers: bool,
    #[serde(default = "default_timer_on_exit")]
    timer_on_sleep: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferencesSnapshot {
    preferences: PreferencesRecord,
    version: i64,
}

fn default_estimate_scale() -> String {
    "fibonacci".into()
}

fn default_timer_on_exit() -> String {
    "pause".into()
}

impl Default for PreferencesRecord {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            issue_scope: "active".into(),
            issue_layout: "list".into(),
            board_group_by: "status".into(),
            group_issues_by_status: true,
            show_issue_priority: true,
            show_issue_project: true,
            estimate_scale: default_estimate_scale(),
            timer_on_exit: default_timer_on_exit(),
            concurrent_timers: false,
            timer_on_sleep: default_timer_on_exit(),
        }
    }
}

pub(crate) fn document_data_root(app: &AppHandle) -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("PM_APP_ACCEPTANCE_ROOT") {
        return acceptance_data_root(Path::new(&path));
    }

    let documents = app.path().document_dir().map_err(|_| {
        "The Documents folder is unavailable. Check its location and permissions.".to_owned()
    })?;
    resolve_documents_data_root(&documents)
}

fn resolve_documents_data_root(documents: &Path) -> Result<PathBuf, String> {
    let root = documents.join(DATA_FOLDER);
    let legacy = documents.join(LEGACY_DATA_FOLDER);

    if legacy.join("workspace.sqlite").is_file() {
        if root.exists() {
            return Err(
                "An Initera folder already exists beside the old workspace. Neither folder was changed."
                    .into(),
            );
        }

        // Renaming the directory keeps the database, media, and backups together.
        fs::rename(&legacy, &root)
            .map_err(|error| format!("Cannot move the existing workspace to Initera: {error}"))?;
    }

    Ok(root)
}

fn acceptance_data_root(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("Acceptance data root must be an absolute path.".into());
    }
    let root = path
        .canonicalize()
        .map_err(|_| "Acceptance data root does not exist.".to_owned())?;
    if !root.join(".pm-app-acceptance-root").is_file() {
        return Err("Acceptance data root needs its marker file.".into());
    }
    Ok(root)
}

pub(crate) fn open_database(root: &Path) -> Result<Connection, String> {
    fs::create_dir_all(root)
        .map_err(|_| "Cannot create the app data folder in Documents.".to_owned())?;
    for folder in ["media", "staging", "backups", "recovery"] {
        fs::create_dir_all(root.join(folder))
            .map_err(|_| "Cannot create the data folders in Documents.".to_owned())?;
    }

    let connection = Connection::open(root.join("workspace.sqlite"))
        .map_err(|_| "Cannot open workspace.sqlite in Documents.".to_owned())?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|_| "Cannot configure the workspace database.".to_owned())?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
        .map_err(|_| "Cannot configure the workspace database.".to_owned())?;

    let version: i32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| "Cannot read the workspace schema version.".to_owned())?;
    let application_id: i32 = connection
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(|_| "Cannot read the workspace application ID.".to_owned())?;

    // V1 is the clean-install baseline. Reject old development schemas rather than replaying
    // migrations against data whose shape is no longer guaranteed.
    if version == 0 && application_id == 0 {
        let has_tables: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%')",
                [],
                |row| row.get(0),
            )
            .map_err(|_| "Cannot inspect the workspace database.".to_owned())?;
        if has_tables {
            return Err("This unversioned database is not a fresh V1 workspace.".into());
        }

        connection
            .execute_batch(&format!(
                "BEGIN IMMEDIATE;
{}
COMMIT;",
                include_str!("schema.sql")
            ))
            .map_err(|error| format!("Cannot initialize the V1 workspace: {error}"))?;
    } else if version != 1 || application_id != 0x504d4150 {
        return Err("This workspace uses an unsupported development schema. Start a new V1 profile; the existing database was not changed.".into());
    }

    connection
        .execute(
            "INSERT OR IGNORE INTO timer_session(singleton, clean_exit, recovered_timers) VALUES (1, 1, 0)",
            [],
        )
        .map_err(|_| "Cannot initialize the timer session.".to_owned())?;
    crate::api_contract::install_change_capture(&connection)?;
    Ok(connection)
}

pub(crate) fn searchable_issue_description(row: &str) -> String {
    format!(
        "CASE WHEN {row}.description_document_json IS NULL THEN {row}.description
              ELSE COALESCE((SELECT group_concat(value, ' ') FROM (
                SELECT value FROM json_tree({row}.description_document_json, '$.content')
                WHERE key = 'text' AND type = 'text'
                  AND fullkey NOT LIKE '%.attrs%'
                  AND fullkey NOT LIKE '%.marks%'
                ORDER BY id
              )), '') END"
    )
}

fn load_preferences(connection: &Connection) -> Result<PreferencesRecord, String> {
    let identity = read_identity(connection)?
        .ok_or_else(|| "Set up your workspace before loading preferences.".to_owned())?;
    connection
        .query_row(
            "SELECT theme, issue_scope, issue_layout, board_group_by,
                    group_issues_by_status, show_issue_priority, show_issue_project, estimate_scale,
                    timer_on_exit, concurrent_timers, timer_on_sleep
             FROM workspace_preferences WHERE workspace_id = ?1",
            [&identity.workspace_id],
            |row| {
                Ok(PreferencesRecord {
                    theme: row.get(0)?,
                    issue_scope: row.get(1)?,
                    issue_layout: row.get(2)?,
                    board_group_by: row.get(3)?,
                    group_issues_by_status: row.get(4)?,
                    show_issue_priority: row.get(5)?,
                    show_issue_project: row.get(6)?,
                    estimate_scale: row.get(7)?,
                    timer_on_exit: row.get(8)?,
                    concurrent_timers: row.get(9)?,
                    timer_on_sleep: row.get(10)?,
                })
            },
        )
        .optional()
        .map(|value| value.unwrap_or_default())
        .map_err(|_| "Cannot read workspace preferences.".to_owned())
}

fn preferences_snapshot(connection: &Connection) -> Result<PreferencesSnapshot, String> {
    let preferences = load_preferences(connection)?;
    let encoded = serde_json::to_vec(&preferences)
        .map_err(|_| "Cannot encode workspace preferences.".to_owned())?;
    let digest = Sha256::digest(encoded);
    let version =
        (i64::from_be_bytes(digest[..8].try_into().unwrap()) & ((1_i64 << 53) - 1)).max(1);
    Ok(PreferencesSnapshot {
        preferences,
        version,
    })
}

#[cfg(test)]
fn save_preferences(
    connection: &mut Connection,
    preferences: &PreferencesRecord,
) -> Result<PreferencesRecord, String> {
    let transaction = connection
        .transaction()
        .map_err(|_| "Cannot begin the preferences update.".to_owned())?;
    let result = save_preferences_in_tx(&transaction, preferences)?;
    transaction
        .commit()
        .map_err(|_| "Cannot save workspace preferences.".to_owned())?;
    Ok(result)
}

fn save_preferences_in_tx(
    connection: &Connection,
    preferences: &PreferencesRecord,
) -> Result<PreferencesRecord, String> {
    let identity = read_identity(connection)?
        .ok_or_else(|| "Set up your workspace before saving preferences.".to_owned())?;
    if !["system", "light", "dark"].contains(&preferences.theme.as_str())
        || !["active", "all"].contains(&preferences.issue_scope.as_str())
        || !["list", "board"].contains(&preferences.issue_layout.as_str())
        || !["status", "priority"].contains(&preferences.board_group_by.as_str())
        || !["fibonacci", "linear", "exponential", "tshirt", "time"]
            .contains(&preferences.estimate_scale.as_str())
        || !["pause", "continue"].contains(&preferences.timer_on_exit.as_str())
        || !["pause", "include"].contains(&preferences.timer_on_sleep.as_str())
    {
        return Err("Workspace preferences contain an invalid option.".into());
    }
    if !preferences.concurrent_timers {
        let running: i64 = connection.query_row(
            "SELECT count(*) FROM time_entry WHERE workspace_id = ?1 AND stopped_at IS NULL AND deleted_at IS NULL",
            [&identity.workspace_id], |row| row.get(0),
        ).map_err(|_| "Cannot inspect active timers.".to_owned())?;
        if running > 1 {
            return Err("CONFIRMATION_REQUIRED: Choose a timer to keep or pause all before disabling concurrent timers.".into());
        }
    }
    connection
        .execute(
            "INSERT INTO workspace_preferences
               (workspace_id, theme, issue_scope, issue_layout, board_group_by,
                group_issues_by_status, show_issue_priority, show_issue_project, estimate_scale,
                timer_on_exit, concurrent_timers, timer_on_sleep)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT (workspace_id) DO UPDATE SET
               theme = excluded.theme, issue_scope = excluded.issue_scope,
               issue_layout = excluded.issue_layout, board_group_by = excluded.board_group_by,
               group_issues_by_status = excluded.group_issues_by_status,
               show_issue_priority = excluded.show_issue_priority,
               show_issue_project = excluded.show_issue_project,
               estimate_scale = excluded.estimate_scale,
               timer_on_exit = excluded.timer_on_exit,
               concurrent_timers = excluded.concurrent_timers,
               timer_on_sleep = excluded.timer_on_sleep",
            params![
                identity.workspace_id,
                preferences.theme,
                preferences.issue_scope,
                preferences.issue_layout,
                preferences.board_group_by,
                preferences.group_issues_by_status,
                preferences.show_issue_priority,
                preferences.show_issue_project,
                preferences.estimate_scale,
                preferences.timer_on_exit,
                preferences.concurrent_timers,
                preferences.timer_on_sleep
            ],
        )
        .map_err(|_| "Cannot save workspace preferences.".to_owned())?;
    Ok(preferences.clone())
}

#[api_contract_macros::command]
pub fn storage_load_preferences(app: AppHandle) -> Result<PreferencesSnapshot, String> {
    let root = document_data_root(&app)?;
    preferences_snapshot(&open_database(&root)?)
}

#[api_contract_macros::command]
pub fn storage_save_preferences(
    app: AppHandle,
    preferences: PreferencesRecord,
    keep_timer_issue: Option<String>,
    mutation: MutationRequest,
) -> Result<PreferencesSnapshot, String> {
    let root = document_data_root(&app)?;
    mutations::apply(
        &mut open_database(&root)?,
        "preferences.save",
        &(&preferences, &keep_timer_issue),
        &mutation,
        |tx, _| {
            if mutation.expected_version != Some(preferences_snapshot(tx)?.version) {
                return Err(
                    "CONFLICT: Workspace preferences changed. Reload and try again.".into(),
                );
            }
            if !preferences.concurrent_timers {
                let workspace = current_workspace_id(tx)?;
                let running: i64 = tx.query_row(
                    "SELECT count(*) FROM time_entry WHERE workspace_id = ?1 AND stopped_at IS NULL AND deleted_at IS NULL",
                    [&workspace], |row| row.get(0),
                ).map_err(|_| "Cannot inspect active timers.".to_owned())?;
                if running > 1 {
                    let choice = keep_timer_issue
                        .as_deref()
                        .ok_or("CONFIRMATION_REQUIRED: Choose a timer to keep or pause all.")?;
                    if !choice.is_empty() {
                        let found: bool = tx.query_row(
                            "SELECT EXISTS(SELECT 1 FROM time_entry JOIN issue ON issue.id = time_entry.issue_id
                             WHERE time_entry.workspace_id = ?1 AND issue.identifier = ?2 AND time_entry.stopped_at IS NULL)",
                            params![workspace, choice], |row| row.get(0),
                        ).map_err(|_| "Cannot inspect selected timer.".to_owned())?;
                        if !found {
                            return Err("Selected timer is no longer running.".into());
                        }
                    }
                    tx.execute(
                        "UPDATE time_entry SET stopped_at = unixepoch('now')
                         WHERE workspace_id = ?1 AND stopped_at IS NULL
                           AND (?2 = '' OR issue_id NOT IN
                             (SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2))",
                        params![workspace, choice],
                    )
                    .map_err(|_| "Cannot pause concurrent timers.".to_owned())?;
                }
            }
            save_preferences_in_tx(tx, &preferences)?;
            preferences_snapshot(tx)
        },
    )
}

#[cfg(test)]
fn list_projects(connection: &Connection) -> Result<Vec<ProjectRecord>, String> {
    let mut statement = connection
        .prepare(
            "SELECT project.id
             FROM project JOIN workspace ON workspace.id = project.workspace_id
             WHERE workspace.id = (SELECT workspace_id FROM local_actor WHERE singleton = 1)
               AND project.archived_at IS NULL AND project.deleted_at IS NULL
             ORDER BY project.name COLLATE NOCASE, project.id",
        )
        .map_err(|_| "Cannot read projects.".to_owned())?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|_| "Cannot read projects.".to_owned())?;
    let ids = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read projects.".to_owned())?;
    ids.iter().map(|id| read_project(connection, id)).collect()
}

fn list_projects_page_query(
    connection: &Connection,
    cursor: Option<&str>,
    limit: usize,
    sort: &str,
    status: Option<&str>,
    priority: Option<&str>,
    search: Option<&str>,
) -> Result<ProjectPage, String> {
    if !(1..=200).contains(&limit) {
        return Err("Request between 1 and 200 projects per page.".into());
    }
    let (sort_expression, direction) = match sort {
        "updated" => ("COALESCE(updated_at, created_at)", "DESC"),
        "name" => ("name COLLATE NOCASE", "ASC"),
        "status" => ("status COLLATE NOCASE", "ASC"),
        "target" => ("COALESCE(target_date, '9999-12-31')", "ASC"),
        "priority" => ("CASE priority WHEN 'Urgent' THEN '0' WHEN 'High' THEN '1' WHEN 'Medium' THEN '2' WHEN 'Low' THEN '3' ELSE '4' END", "ASC"),
        _ => return Err("Choose a valid project sort.".into()),
    };
    let search = search.map(str::trim).filter(|value| !value.is_empty());
    if search.is_some_and(|value| value.len() > 256) {
        return Err("Project search is too long.".into());
    }
    let query_hash = format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&(sort, status, priority, search))
                .map_err(|_| "Cannot encode project query.".to_owned())?
        )
    );
    let workspace_id = current_workspace_id(connection)?;
    let revision: i64 = connection
        .query_row(
            "SELECT revision FROM workspace WHERE id = ?1",
            [&workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read project index revision.".to_owned())?;
    let cursor = cursor
        .map(|value| {
            let bytes = URL_SAFE_NO_PAD
                .decode(value)
                .map_err(|_| "Invalid project cursor.".to_owned())?;
            serde_json::from_slice::<ProjectCursor>(&bytes)
                .map_err(|_| "Invalid project cursor.".to_owned())
        })
        .transpose()?;
    if cursor.as_ref().is_some_and(|value| {
        value.workspace_id != workspace_id
            || value.revision != revision
            || value.query_hash != query_hash
    }) {
        return Err("CONFLICT: Project index changed. Reload the first page.".into());
    }
    let predicate = "workspace_id = ?1 AND archived_at IS NULL AND deleted_at IS NULL
                     AND (?2 IS NULL OR status = ?2)
                     AND (?3 IS NULL OR priority = ?3)
                     AND (?4 IS NULL OR instr(lower(name), lower(?4)) > 0)";
    let total_count: i64 = connection
        .query_row(
            &format!("SELECT count(*) FROM project WHERE {predicate}"),
            params![workspace_id, status, priority, search],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot count projects.".to_owned())?;
    let mut statement = connection
        .prepare(&format!(
            "SELECT id, {sort_expression} AS sort_key FROM project
             WHERE {predicate}
               AND (?5 IS NULL OR sort_key {comparison} ?5
                    OR (sort_key = ?5 AND id {comparison} ?6))
             ORDER BY sort_key {direction}, id {direction} LIMIT ?7",
            comparison = if direction == "DESC" { "<" } else { ">" },
        ))
        .map_err(|_| "Cannot read projects.".to_owned())?;
    let mut entries = statement
        .query_map(
            params![
                workspace_id,
                status,
                priority,
                search,
                cursor.as_ref().map(|value| &value.sort_key),
                cursor.as_ref().map(|value| &value.id),
                (limit + 1) as i64
            ],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|_| "Cannot read projects.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read projects.".to_owned())?;
    let has_more = entries.len() > limit;
    entries.truncate(limit);
    let next_cursor = if has_more {
        entries.last().map(|(id, sort_key)| {
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&ProjectCursor {
                    workspace_id: workspace_id.clone(),
                    revision,
                    query_hash: query_hash.clone(),
                    sort_key: sort_key.clone(),
                    id: id.clone(),
                })
                .expect("project cursor serializes"),
            )
        })
    } else {
        None
    };
    let items = entries
        .iter()
        .map(|(id, _)| read_project(connection, id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ProjectPage {
        items,
        next_cursor,
        total_count,
    })
}

fn list_inactive_projects(connection: &Connection) -> Result<Vec<ProjectLifecycleEntry>, String> {
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT id, name, CASE WHEN deleted_at IS NOT NULL THEN 'trash' ELSE 'archived' END, version
             FROM project WHERE workspace_id = ?1
               AND (archived_at IS NOT NULL OR deleted_at IS NOT NULL)
             ORDER BY name COLLATE NOCASE, id",
        )
        .map_err(|_| "Cannot list inactive projects.".to_owned())?;
    let entries = statement
        .query_map([workspace_id], |row| {
            Ok(ProjectLifecycleEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                state: row.get(2)?,
                version: row.get(3)?,
            })
        })
        .map_err(|_| "Cannot list inactive projects.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot list inactive projects.".to_owned())?;
    Ok(entries)
}

fn preview_project_lifecycle(
    connection: &Connection,
    id: &str,
) -> Result<ProjectLifecyclePreview, String> {
    let workspace_id = current_workspace_id(connection)?;
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1 AND workspace_id = ?2)",
            params![id, workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot inspect project lifecycle.".to_owned())?;
    if !exists {
        return Err("Project not found.".into());
    }
    let active_issues = connection
        .query_row(
            "SELECT count(*) FROM issue WHERE project_id = ?1 AND archived_at IS NULL AND deleted_at IS NULL",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot count project issues.".to_owned())?;
    let running_timers = connection
        .query_row(
            "SELECT count(*) FROM time_entry entry JOIN issue ON issue.id = entry.issue_id
         WHERE issue.project_id = ?1 AND entry.stopped_at IS NULL",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot count running timers.".to_owned())?;
    let nodes = connection
        .query_row(
            "SELECT count(*) FROM planning_node WHERE project_id = ?1 AND deleted_at IS NULL",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot count project nodes.".to_owned())?;
    let milestones = connection
        .query_row(
            "SELECT count(*) FROM milestone WHERE project_id = ?1 AND deleted_at IS NULL",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot count project milestones.".to_owned())?;
    let cycle_dependencies = connection.query_row(
        "SELECT
           (SELECT count(*) FROM cycle_series series WHERE series.workspace_id = ?2 AND series.enabled = 1
             AND (json_array_length(series.project_scope_json) = 0 OR EXISTS (
               SELECT 1 FROM json_each(series.project_scope_json) WHERE value = ?1)))
           + (SELECT count(*) FROM cycle WHERE workspace_id = ?2 AND status != 'completed'
             AND (EXISTS (SELECT 1 FROM cycle_issue membership JOIN issue ON issue.id = membership.issue_id
                          WHERE membership.cycle_id = cycle.id AND issue.project_id = ?1)
               OR EXISTS (SELECT 1 FROM json_each(cycle.project_scope_json) WHERE value = ?1)))",
        params![id, workspace_id], |row| row.get(0),
    ).map_err(|_| "Cannot inspect project cycles.".to_owned())?;
    Ok(ProjectLifecyclePreview {
        active_issues,
        nodes,
        milestones,
        running_timers,
        cycle_dependencies,
    })
}

#[cfg(test)]
fn set_project_lifecycle(
    connection: &mut Connection,
    id: &str,
    action: &str,
) -> Result<(), String> {
    set_project_lifecycle_with_mutation(connection, id, action, None)
}

fn set_project_lifecycle_with_mutation(
    connection: &mut Connection,
    id: &str,
    action: &str,
    mutation: Option<MutationRequest>,
) -> Result<(), String> {
    if !matches!(action, "archive" | "trash" | "restore") {
        return Err("Choose a valid project action.".into());
    }
    let workspace_id = current_workspace_id(connection)?;
    let request_hash = mutations::request_hash(
        "project.lifecycle",
        &(
            id,
            action,
            mutation
                .as_ref()
                .and_then(|request| request.expected_version),
        ),
    )?;
    let transaction = connection
        .transaction()
        .map_err(|_| "Cannot begin project lifecycle change.".to_owned())?;
    if let Some(request) = &mutation {
        if mutations::replay::<()>(
            &transaction,
            &workspace_id,
            "project.lifecycle",
            &request_hash,
            request,
        )?
        .is_some()
        {
            return Ok(());
        }
        mutations::check_version(&transaction, "project", id, request)?;
    }
    if action != "restore" {
        let preview = preview_project_lifecycle(&transaction, id)?;
        if action == "archive" && preview.running_timers > 0 {
            return Err("Pause project timers before archiving or trashing it.".into());
        }
        if preview.cycle_dependencies > 0 {
            return Err(
                "Resolve project cycle scopes and memberships before archiving or trashing it."
                    .into(),
            );
        }
    }
    let changed = match action {
        "archive" => transaction.execute(
            "UPDATE project SET archived_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1 AND workspace_id = ?2 AND archived_at IS NULL AND deleted_at IS NULL",
            params![id, workspace_id],
        ),
        "trash" => {
            // Record only descendants affected by this trash operation. Restore must not
            // resurrect records that were already removed before the project was trashed.
            let cascade_id = Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO project_cascade_member(cascade_id, project_id, kind, member_id, was_archived)
                 SELECT ?1, ?2, 'issue', id, archived_at IS NOT NULL
                 FROM issue WHERE project_id = ?2 AND deleted_at IS NULL",
                params![cascade_id, id],
            ).map_err(|_| "Cannot record project issue cascade.".to_owned())?;
            transaction.execute(
                "UPDATE issue SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                 archived_at = COALESCE(archived_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                 WHERE project_id = ?1 AND deleted_at IS NULL",
                [id],
            ).map_err(|_| "Cannot trash project issues.".to_owned())?;
            for (kind, table) in [("node", "planning_node"), ("milestone", "milestone")] {
                transaction.execute(
                    &format!("INSERT INTO project_cascade_member(cascade_id, project_id, kind, member_id)
                              SELECT ?1, ?2, ?3, id FROM {table} WHERE project_id = ?2 AND deleted_at IS NULL"),
                    params![cascade_id, id, kind],
                ).map_err(|_| "Cannot record project cascade.".to_owned())?;
                transaction.execute(
                    &format!("UPDATE {table} SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                              WHERE project_id = ?1 AND deleted_at IS NULL"),
                    [id],
                ).map_err(|_| "Cannot trash project descendants.".to_owned())?;
            }
            transaction.execute(
                "UPDATE time_entry SET stopped_at = unixepoch('now')
                 WHERE stopped_at IS NULL AND issue_id IN
                   (SELECT member_id FROM project_cascade_member WHERE cascade_id = ?1 AND kind = 'issue')",
                [&cascade_id],
            ).map_err(|_| "Cannot pause project timers.".to_owned())?;
            transaction.execute(
                "UPDATE project SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), cascade_id = ?3
                 WHERE id = ?1 AND workspace_id = ?2 AND deleted_at IS NULL",
                params![id, workspace_id, cascade_id],
            )
        }
        "restore" => {
            let cascade_id: Option<String> = transaction.query_row(
                "SELECT cascade_id FROM project WHERE id = ?1 AND workspace_id = ?2",
                params![id, workspace_id], |row| row.get(0),
            ).optional().map_err(|_| "Cannot read project cascade.".to_owned())?.flatten();
            if let Some(cascade_id) = cascade_id {
                transaction.execute(
                    "UPDATE issue SET deleted_at = NULL,
                     archived_at = CASE WHEN
                       (SELECT was_archived FROM project_cascade_member member
                        WHERE member.cascade_id = ?1 AND member.kind = 'issue' AND member.member_id = issue.id) = 0
                       THEN NULL ELSE archived_at END
                     WHERE id IN (SELECT member_id FROM project_cascade_member
                                  WHERE cascade_id = ?1 AND kind = 'issue') AND deleted_at IS NOT NULL",
                    [&cascade_id],
                ).map_err(|_| "Cannot restore project issues.".to_owned())?;
                for (kind, table) in [("node", "planning_node"), ("milestone", "milestone")] {
                    transaction.execute(
                        &format!("UPDATE {table} SET deleted_at = NULL WHERE id IN
                                  (SELECT member_id FROM project_cascade_member
                                   WHERE cascade_id = ?1 AND kind = ?2) AND deleted_at IS NOT NULL"),
                        params![cascade_id, kind],
                    ).map_err(|_| "Cannot restore project descendants.".to_owned())?;
                }
            }
            transaction.execute(
                "UPDATE project SET archived_at = NULL, deleted_at = NULL, cascade_id = NULL
                 WHERE id = ?1 AND workspace_id = ?2 AND (archived_at IS NOT NULL OR deleted_at IS NOT NULL)",
                params![id, workspace_id],
            )
        }
        _ => unreachable!(),
    }.map_err(|_| "Cannot change project lifecycle.".to_owned())?;
    if changed == 0 {
        return Err("Project is unavailable or already in that state.".into());
    }
    if let Some(request) = &mutation {
        mutations::save(
            &transaction,
            &workspace_id,
            "project.lifecycle",
            &request_hash,
            request,
            &(),
        )?;
    }
    transaction
        .commit()
        .map_err(|_| "Cannot save project lifecycle change.".to_owned())?;
    Ok(())
}

#[api_contract_macros::command]
pub fn storage_preview_project_lifecycle(
    app: AppHandle,
    id: String,
) -> Result<ProjectLifecyclePreview, String> {
    preview_project_lifecycle(&open_database(&document_data_root(&app)?)?, &id)
}

#[api_contract_macros::command]
pub fn storage_list_inactive_projects(
    app: AppHandle,
) -> Result<Vec<ProjectLifecycleEntry>, String> {
    list_inactive_projects(&open_database(&document_data_root(&app)?)?)
}

#[api_contract_macros::command]
pub fn storage_set_project_lifecycle(
    app: AppHandle,
    id: String,
    action: String,
    mutation: MutationRequest,
) -> Result<(), String> {
    set_project_lifecycle_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &id,
        &action,
        Some(mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_list_removed_milestones(
    app: AppHandle,
    project_id: String,
) -> Result<Vec<RemovedMilestone>, String> {
    let connection = open_database(&document_data_root(&app)?)?;
    let workspace_id = current_workspace_id(&connection)?;
    let mut statement = connection
        .prepare("SELECT milestone.id, title, description, target_date,
                    (SELECT count(*) FROM milestone_detached_issue detached
                     WHERE detached.milestone_id = milestone.id) FROM milestone
                  JOIN project ON project.id = milestone.project_id
                  WHERE project_id = ?1 AND project.workspace_id = ?2 AND milestone.deleted_at IS NOT NULL
                  ORDER BY milestone.sort_order, milestone.id")
        .map_err(|_| "Cannot list removed milestones.".to_owned())?;
    let milestones = statement
        .query_map(params![project_id, workspace_id], |row| {
            Ok(RemovedMilestone {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                target_date: row.get(3)?,
                detached_issue_count: row.get(4)?,
            })
        })
        .map_err(|_| "Cannot list removed milestones.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot list removed milestones.".to_owned())?;
    Ok(milestones)
}

#[api_contract_macros::command]
pub fn storage_preview_milestone_removal(
    app: AppHandle,
    project_id: String,
    milestone_id: String,
) -> Result<i64, String> {
    let connection = open_database(&document_data_root(&app)?)?;
    let workspace_id = current_workspace_id(&connection)?;
    connection
        .query_row(
            "SELECT count(issue.id) FROM milestone
         JOIN project ON project.id = milestone.project_id
         LEFT JOIN issue ON issue.milestone_id = milestone.id AND issue.archived_at IS NULL
         WHERE milestone.id = ?1 AND project.id = ?2 AND project.workspace_id = ?3
           AND milestone.deleted_at IS NULL",
            params![milestone_id, project_id, workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot preview milestone removal.".to_owned())
}

#[api_contract_macros::command]
pub fn storage_set_milestone_archived(
    app: AppHandle,
    project_id: String,
    milestone_id: String,
    archived: bool,
    mutation: MutationRequest,
) -> Result<ProjectRecord, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "milestone.archive",
        &(&project_id, &milestone_id, archived),
        &mutation,
        |tx, workspace_id| {
            mutations::check_version(tx, "project", &project_id, &mutation)?;
            let changed = tx.execute(
                "UPDATE milestone SET archived_at = CASE WHEN ?1 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') ELSE NULL END
                 WHERE id = ?2 AND project_id = ?3 AND deleted_at IS NULL
                   AND EXISTS (SELECT 1 FROM project WHERE id = ?3 AND workspace_id = ?4)",
                params![archived, milestone_id, project_id, workspace_id],
            ).map_err(|_| "Cannot change milestone archive state.".to_owned())?;
            if changed == 0 {
                return Err("Milestone no longer exists.".into());
            }
            tx.execute("UPDATE project SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1", [&project_id])
                .map_err(|_| "Cannot update project milestone state.".to_owned())?;
            read_project(tx, &project_id)
        },
    )
}

#[api_contract_macros::command]
pub fn storage_restore_milestone(
    app: AppHandle,
    project_id: String,
    milestone_id: String,
    reassign_issues: Option<bool>,
    mutation: MutationRequest,
) -> Result<MilestoneRestoreReport, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "milestone.restore",
        &(&project_id, &milestone_id, reassign_issues.unwrap_or(false)),
        &mutation,
        |tx, workspace_id| {
            mutations::check_version(tx, "project", &project_id, &mutation)?;
            let result = restore_milestone_in_tx(
                tx,
                workspace_id,
                &project_id,
                &milestone_id,
                reassign_issues.unwrap_or(false),
            )?;
            tx.execute(
                "UPDATE project SET version = version + 1 WHERE id = ?1",
                [&project_id],
            )
            .map_err(|_| "Cannot update project version.".to_owned())?;
            tx.execute(
                "UPDATE workspace SET revision = revision + 1 WHERE id = ?1",
                [workspace_id],
            )
            .map_err(|_| "Cannot update workspace revision.".to_owned())?;
            Ok(result)
        },
    )
}

fn restore_milestone_in_tx(
    transaction: &Connection,
    workspace_id: &str,
    project_id: &str,
    milestone_id: &str,
    reassign_issues: bool,
) -> Result<MilestoneRestoreReport, String> {
    let changed = transaction
        .execute(
            "UPDATE milestone SET deleted_at = NULL WHERE id = ?1 AND project_id = ?2
         AND deleted_at IS NOT NULL AND EXISTS (
           SELECT 1 FROM project WHERE id = ?2 AND workspace_id = ?3 AND deleted_at IS NULL
         )",
            params![milestone_id, project_id, workspace_id],
        )
        .map_err(|_| "Cannot restore milestone.".to_owned())?;
    if changed == 0 {
        return Err("Removed milestone was not found.".into());
    }
    let total: usize = transaction
        .query_row(
            "SELECT count(*) FROM milestone_detached_issue WHERE milestone_id = ?1",
            [&milestone_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read former milestone assignments.".to_owned())?;
    let reattached = if reassign_issues {
        transaction
            .execute(
                "UPDATE issue SET milestone_id = ?1
             WHERE id IN (SELECT issue_id FROM milestone_detached_issue WHERE milestone_id = ?1)
               AND project_id = ?2 AND milestone_id IS NULL",
                params![milestone_id, project_id],
            )
            .map_err(|_| "Cannot restore milestone assignments.".to_owned())?
    } else {
        0
    };
    transaction
        .execute(
            "DELETE FROM milestone_detached_issue WHERE milestone_id = ?1",
            [&milestone_id],
        )
        .map_err(|_| "Cannot finish milestone restore.".to_owned())?;
    Ok(MilestoneRestoreReport {
        reattached,
        skipped: total - reattached,
    })
}

fn read_project(connection: &Connection, id: &str) -> Result<ProjectRecord, String> {
    let mut project = connection
        .query_row(
            "SELECT id, workspace_id, name, identifier_prefix, created_at,
                summary, description, status, priority, start_date, target_date, name_markdown,
                COALESCE((SELECT json_group_array(name) FROM (
                    SELECT label.name FROM project_label JOIN label ON label.id = project_label.label_id
                    WHERE project_label.project_id = project.id AND label.deleted_at IS NULL
                    ORDER BY label.name COLLATE NOCASE, label.id
                )), '[]'), resources_json, icon, color, description_document_json, lead_actor_id, project.version,
                COALESCE(project.updated_at, project.created_at)
         FROM project WHERE id = ?1",
            [id],
            |row| {
                Ok(ProjectRecord {
                    id: row.get(0)?,
                    version: row.get(18)?,
                    workspace_id: row.get(1)?,
                    name: row.get(2)?,
                    name_markdown: row.get(11)?,
                    identifier_prefix: row.get(3)?,
                    icon: row.get(14)?,
                    color: row.get(15)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(19)?,
                    summary: row.get(5)?,
                    description: row.get(6)?,
                    description_document: decode_document(row.get(16)?)?,
                    status: row.get(7)?,
                    priority: row.get(8)?,
                    lead_actor_id: row.get(17)?,
                    start_date: row.get(9)?,
                    target_date: row.get(10)?,
                    labels: serde_json::from_str::<Vec<String>>(&row.get::<_, String>(12)?)
                        .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    resources: Vec::new(),
                    milestones: Vec::new(),
                })
            },
        )
        .map_err(|_| "Cannot read project details.".to_owned())?;
    let mut resource_statement = connection.prepare(
        "SELECT id, title, url FROM project_resource WHERE project_id = ?1 ORDER BY sort_order, id"
    ).map_err(|_| "Cannot read project resources.".to_owned())?;
    project.resources = resource_statement
        .query_map([id], |row| {
            Ok(ProjectResource {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
            })
        })
        .map_err(|_| "Cannot read project resources.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read project resources.".to_owned())?;
    let mut statement = connection
        .prepare(
            "SELECT id, title, description, target_date, archived_at FROM milestone
         WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY sort_order, id",
        )
        .map_err(|_| "Cannot read project milestones.".to_owned())?;
    project.milestones = statement
        .query_map([id], |row| {
            Ok(MilestoneRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                target_date: row.get(3)?,
                archived_at: row.get(4)?,
            })
        })
        .map_err(|_| "Cannot read project milestones.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read project milestones.".to_owned())?;
    Ok(project)
}

fn sync_project_resources(
    connection: &Connection,
    project_id: &str,
    resources: &[ProjectResource],
) -> Result<(), String> {
    // A whole-project edit supplies the ordered resource set. Update stable IDs first, then
    // remove omitted rows; callers run this inside the project mutation transaction.
    for (index, resource) in resources.iter().enumerate() {
        let changed = connection.execute(
            "UPDATE project_resource SET
                version = version + CASE WHEN title <> ?1 OR url <> ?2 OR sort_order <> ?3 THEN 1 ELSE 0 END,
                title = ?1, url = ?2, sort_order = ?3
             WHERE id = ?4 AND project_id = ?5",
            params![resource.title.trim(), resource.url.trim(), index as i64, resource.id, project_id],
        ).map_err(|_| "Cannot update project resources.".to_owned())?;
        if changed == 0 {
            connection
                .execute(
                    "INSERT INTO project_resource(id, project_id, title, url, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        resource.id,
                        project_id,
                        resource.title.trim(),
                        resource.url.trim(),
                        index as i64
                    ],
                )
                .map_err(|_| "Cannot add project resource.".to_owned())?;
        }
    }
    let ids = serde_json::to_string(&resources.iter().map(|item| &item.id).collect::<Vec<_>>())
        .map_err(|_| "Cannot encode resource order.".to_owned())?;
    connection
        .execute(
            "DELETE FROM project_resource WHERE project_id = ?1 AND id NOT IN
         (SELECT value FROM json_each(?2))",
            params![project_id, ids],
        )
        .map_err(|_| "Cannot remove project resource.".to_owned())?;
    Ok(())
}

pub(crate) fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return false;
    }
    let year: u32 = value[0..4].parse().unwrap_or(0);
    let month: u32 = value[5..7].parse().unwrap_or(0);
    let day: u32 = value[8..10].parse().unwrap_or(0);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    year > 0 && day > 0 && day <= days
}

fn default_project_icon() -> String {
    "i-tabler-cube".into()
}
fn default_project_color() -> String {
    "amber".into()
}

fn validate_project_input(input: &ProjectInput) -> Result<(&str, String), String> {
    encode_document(input.description_document.as_ref())?;
    let name = validate_name(&input.name)?;
    let prefix = input.identifier_prefix.trim().to_ascii_uppercase();
    if name.eq_ignore_ascii_case("Unassigned") || prefix == "LOCAL" {
        return Err("Unassigned and LOCAL are reserved for issues without a project.".into());
    }
    if !(2..=8).contains(&prefix.len()) || !prefix.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Err("Project identifiers need 2–8 letters.".into());
    }
    if input.summary.chars().count() > 500
        || input.description.len() > 2_097_152
        || input.name_markdown.len() > 4096
    {
        return Err("Project summary or description is too long.".into());
    }
    if !["Backlog", "Planned", "In Progress", "Completed", "Canceled"]
        .contains(&input.status.as_str())
        || !["No priority", "Low", "Medium", "High", "Urgent"].contains(&input.priority.as_str())
    {
        return Err("Choose a valid project status and priority.".into());
    }
    if ![
        "i-tabler-cube",
        "i-tabler-stack-2",
        "i-tabler-device-desktop",
        "i-tabler-palette",
        "i-tabler-bookmark",
        "i-tabler-inbox",
        "i-tabler-tag",
        "i-tabler-target",
        "i-tabler-flag",
        "i-tabler-calendar",
        "i-tabler-clock",
        "i-tabler-rocket",
        "i-tabler-bulb",
        "i-tabler-star",
        "i-tabler-code",
        "i-tabler-bug",
        "i-tabler-git-branch",
        "i-tabler-git-merge",
        "i-tabler-shield-check",
        "i-tabler-sparkles",
        "i-tabler-bolt",
        "i-tabler-flame",
        "i-tabler-message-circle",
        "i-tabler-list",
    ]
    .contains(&input.icon.as_str())
        || ![
            "gray", "blue", "purple", "green", "amber", "red", "cyan", "peach",
        ]
        .contains(&input.color.as_str())
    {
        return Err("Choose a valid project icon and color.".into());
    }
    for date in [input.start_date.as_deref(), input.target_date.as_deref()]
        .into_iter()
        .flatten()
    {
        if !valid_date(date) {
            return Err("Project dates must be valid calendar dates.".into());
        }
    }
    if input.start_date > input.target_date && input.target_date.is_some() {
        return Err("Target date must be on or after the start date.".into());
    }
    if input.milestones.len() > 100 {
        return Err("A project can start with up to 100 milestones.".into());
    }
    if input.labels.len() > 32
        || input
            .labels
            .iter()
            .any(|label| label.trim().is_empty() || label.chars().count() > 80)
        || input.labels.iter().collect::<HashSet<_>>().len() != input.labels.len()
    {
        return Err("Choose up to 32 unique project labels.".into());
    }
    if input.resources.len() > 100
        || input.resources.iter().any(|resource| {
            resource.id.trim().is_empty()
                || resource.id.len() > 80
                || resource.title.trim().is_empty()
                || resource.title.chars().count() > 120
                || resource.url.len() > 2048
                || !(resource.url.starts_with("https://") || resource.url.starts_with("http://"))
        })
        || input
            .resources
            .iter()
            .map(|resource| &resource.id)
            .collect::<HashSet<_>>()
            .len()
            != input.resources.len()
    {
        return Err("Check each project resource title and HTTP(S) link.".into());
    }
    for milestone in &input.milestones {
        if milestone.title.trim().is_empty()
            || milestone.title.chars().count() > 240
            || milestone.description.len() > 2_097_152
            || milestone
                .target_date
                .as_deref()
                .is_some_and(|date| !valid_date(date))
        {
            return Err("Check each milestone title, description, and target date.".into());
        }
    }
    Ok((name, prefix))
}

#[cfg(test)]
pub(crate) fn create_project(
    connection: &mut Connection,
    name: &str,
    identifier_prefix: &str,
) -> Result<ProjectRecord, String> {
    create_project_full(
        connection,
        ProjectInput {
            name: name.into(),
            name_markdown: String::new(),
            identifier_prefix: identifier_prefix.into(),
            icon: default_project_icon(),
            color: default_project_color(),
            summary: String::new(),
            description: String::new(),
            description_document: None,
            status: "Backlog".into(),
            priority: "No priority".into(),
            lead_actor_id: None,
            start_date: None,
            target_date: None,
            labels: Vec::new(),
            resources: Vec::new(),
            milestones: Vec::new(),
        },
    )
}

#[cfg(test)]
fn create_project_full(
    connection: &mut Connection,
    input: ProjectInput,
) -> Result<ProjectRecord, String> {
    create_project_full_with_mutation(connection, input, None)
}

fn create_project_full_with_mutation(
    connection: &mut Connection,
    input: ProjectInput,
    mutation: Option<MutationRequest>,
) -> Result<ProjectRecord, String> {
    let (name, prefix) = validate_project_input(&input)?;
    let identity = read_identity(connection)?
        .ok_or_else(|| "Set up your workspace before creating a project.".to_owned())?;
    let request_hash = mutations::request_hash("project.create", &input)?;
    if input
        .lead_actor_id
        .as_deref()
        .is_some_and(|id| id != identity.actor_id)
    {
        return Err("Choose the local user as project lead.".into());
    }
    let id = Uuid::new_v4().to_string();
    let transaction = connection
        .transaction()
        .map_err(|_| "Cannot begin the project update.".to_owned())?;
    if let Some(request) = &mutation {
        if let Some(previous) = mutations::replay(
            &transaction,
            &identity.workspace_id,
            "project.create",
            &request_hash,
            request,
        )? {
            return Ok(previous);
        }
        if request.expected_version.is_some() {
            return Err("A new project cannot have an expected version.".into());
        }
    }
    transaction
        .execute(
            "INSERT INTO project (id, workspace_id, name, identifier_prefix, summary,
                                  description, status, priority, start_date, target_date, name_markdown, labels_json, resources_json, icon, color, description_document_json, lead_actor_id, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![
                id,
                identity.workspace_id,
                name,
                prefix,
                input.summary.trim(),
                input.description.trim(),
                input.status,
                input.priority,
                input.start_date,
                input.target_date,
                input.name_markdown,
                serde_json::to_string(&input.labels).map_err(|_| "Cannot encode project labels.".to_owned())?,
                serde_json::to_string(&input.resources).map_err(|_| "Cannot encode project resources.".to_owned())?,
                input.icon,
                input.color,
                encode_document_for_storage(&transaction, input.description_document.as_ref())?,
                input.lead_actor_id,
            ],
        )
        .map_err(|error| match error {
            rusqlite::Error::SqliteFailure(_, Some(message))
                if message.contains("project.workspace_id, project.name") =>
            {
                "A project with this name already exists.".to_owned()
            }
            rusqlite::Error::SqliteFailure(_, Some(message))
                if message.contains("project.workspace_id, project.identifier_prefix") =>
            {
                "This project identifier is already in use.".to_owned()
            }
            _ => "Cannot create the project.".to_owned(),
        })?;
    for (index, milestone) in input.milestones.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO milestone(id, project_id, title, description, target_date, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    Uuid::new_v4().to_string(),
                    id,
                    milestone.title.trim(),
                    milestone.description.trim(),
                    milestone.target_date,
                    index as i64
                ],
            )
            .map_err(|_| "Cannot create the project milestones.".to_owned())?;
    }
    sync_project_resources(&transaction, &id, &input.resources)?;
    let record = read_project(&transaction, &id)?;
    if let Some(request) = &mutation {
        mutations::save(
            &transaction,
            &identity.workspace_id,
            "project.create",
            &request_hash,
            request,
            &record,
        )?;
    }
    transaction
        .commit()
        .map_err(|_| "Cannot save the project.".to_owned())?;
    Ok(record)
}

#[cfg(test)]
fn update_project_full(
    connection: &mut Connection,
    id: &str,
    input: ProjectInput,
) -> Result<ProjectRecord, String> {
    update_project_full_with_mutation(connection, id, input, None)
}

fn update_project_full_with_mutation(
    connection: &mut Connection,
    id: &str,
    input: ProjectInput,
    mutation: Option<MutationRequest>,
) -> Result<ProjectRecord, String> {
    let (name, prefix) = validate_project_input(&input)?;
    let workspace_id = current_workspace_id(connection)?;
    let request_hash = mutations::request_hash(
        "project.update",
        &(
            id,
            &input,
            mutation
                .as_ref()
                .and_then(|request| request.expected_version),
        ),
    )?;
    let transaction = connection
        .transaction()
        .map_err(|_| "Cannot begin the project update.".to_owned())?;
    if let Some(request) = &mutation {
        if let Some(previous) = mutations::replay(
            &transaction,
            &workspace_id,
            "project.update",
            &request_hash,
            request,
        )? {
            return Ok(previous);
        }
        mutations::check_version(&transaction, "project", id, request)?;
    }
    let previous = read_project(&transaction, id)?;
    let actor =
        read_identity(&transaction)?.ok_or_else(|| "Set up your workspace first.".to_owned())?;
    if input
        .lead_actor_id
        .as_deref()
        .is_some_and(|lead| lead != actor.actor_id)
    {
        return Err("Choose the local user as project lead.".into());
    }
    let changes = project_changes(&previous, &input);
    let existing = {
        let mut statement = transaction
            .prepare("SELECT id FROM milestone WHERE project_id = ?1 AND deleted_at IS NULL")
            .map_err(|_| "Cannot read project milestones.".to_owned())?;
        let milestones = statement
            .query_map([id], |row| row.get::<_, String>(0))
            .map_err(|_| "Cannot read project milestones.".to_owned())?
            .collect::<rusqlite::Result<HashSet<_>>>()
            .map_err(|_| "Cannot read project milestones.".to_owned())?;
        milestones
    };
    let mut retained = HashSet::new();
    for milestone in &input.milestones {
        if let Some(milestone_id) = &milestone.id {
            if !existing.contains(milestone_id) || !retained.insert(milestone_id.clone()) {
                return Err("A milestone no longer belongs to this project.".into());
            }
        }
    }
    let changed = transaction
        .execute(
            "UPDATE project SET name = ?1, name_markdown = ?2, identifier_prefix = ?3,
             summary = ?4, description = ?5, status = ?6, priority = ?7,
             start_date = ?8, target_date = ?9, labels_json = ?10, resources_json = ?11,
             icon = ?12, color = ?13, description_document_json = ?16,
             lead_actor_id = ?17
             WHERE id = ?14 AND workspace_id = ?15",
            params![
                name,
                input.name_markdown,
                prefix,
                input.summary.trim(),
                input.description.trim(),
                input.status,
                input.priority,
                input.start_date,
                input.target_date,
                serde_json::to_string(&input.labels)
                    .map_err(|_| "Cannot encode project labels.".to_owned())?,
                serde_json::to_string(&input.resources)
                    .map_err(|_| "Cannot encode project resources.".to_owned())?,
                input.icon,
                input.color,
                id,
                workspace_id,
                encode_document_for_storage(&transaction, input.description_document.as_ref())?,
                input.lead_actor_id,
            ],
        )
        .map_err(|error| match error {
            rusqlite::Error::SqliteFailure(_, Some(message)) if message.contains("UNIQUE") => {
                "Project name and identifier must be unique.".to_owned()
            }
            _ => "Cannot update the project.".to_owned(),
        })?;
    if changed == 0 {
        return Err("Project no longer exists.".into());
    }
    for old_id in existing.difference(&retained) {
        transaction
            .execute(
                "INSERT OR IGNORE INTO milestone_detached_issue(milestone_id, issue_id)
             SELECT ?1, id FROM issue WHERE milestone_id = ?1",
                [old_id],
            )
            .map_err(|_| "Cannot preserve milestone assignments.".to_owned())?;
        transaction
            .execute(
                "INSERT INTO issue_activity(id, issue_id, actor_name, kind, body)
             SELECT lower(hex(randomblob(16))), issue.id, ?2, 'changed', 'removed project milestone'
             FROM issue WHERE issue.milestone_id = ?1",
                params![old_id, actor.user_name],
            )
            .map_err(|_| "Cannot record milestone removal.".to_owned())?;
        transaction
            .execute(
                "UPDATE issue SET milestone_id = NULL WHERE milestone_id = ?1",
                [old_id],
            )
            .map_err(|_| "Cannot clear removed milestone assignments.".to_owned())?;
        transaction
            .execute("UPDATE milestone SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1", [old_id])
            .map_err(|_| "Cannot remove a project milestone.".to_owned())?;
    }
    for (index, milestone) in input.milestones.iter().enumerate() {
        if let Some(milestone_id) = &milestone.id {
            transaction
                .execute(
                    "UPDATE milestone SET title = ?1, description = ?2, target_date = ?3,
                     sort_order = ?4 WHERE id = ?5 AND project_id = ?6",
                    params![
                        milestone.title.trim(),
                        milestone.description.trim(),
                        milestone.target_date,
                        index as i64,
                        milestone_id,
                        id
                    ],
                )
                .map_err(|_| "Cannot update a project milestone.".to_owned())?;
        } else {
            transaction
                .execute(
                    "INSERT INTO milestone(id, project_id, title, description, target_date, sort_order)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![Uuid::new_v4().to_string(), id, milestone.title.trim(), milestone.description.trim(), milestone.target_date, index as i64],
                )
                .map_err(|_| "Cannot add a project milestone.".to_owned())?;
        }
    }
    if previous.resources != input.resources {
        sync_project_resources(&transaction, id, &input.resources)?;
    }
    if !changes.is_empty() {
        transaction
            .execute(
                "INSERT INTO project_update(id, project_id, actor_name, body, kind)
                 VALUES (?1, ?2, ?3, ?4, 'change')",
                params![
                    Uuid::new_v4().to_string(),
                    id,
                    actor.user_name,
                    changes.join("; ")
                ],
            )
            .map_err(|_| "Cannot record project changes.".to_owned())?;
    }
    let record = read_project(&transaction, id)?;
    if let Some(request) = &mutation {
        mutations::save(
            &transaction,
            &workspace_id,
            "project.update",
            &request_hash,
            request,
            &record,
        )?;
    }
    transaction
        .commit()
        .map_err(|_| "Cannot save the project update.".to_owned())?;
    Ok(record)
}

fn project_changes(previous: &ProjectRecord, input: &ProjectInput) -> Vec<String> {
    let mut changes = Vec::new();
    if previous.name != input.name.trim() {
        changes.push(format!(
            "renamed project from {} to {}",
            previous.name,
            input.name.trim()
        ));
    }
    if previous.name_markdown != input.name_markdown {
        changes.push("edited the project title formatting".into());
    }
    if previous.identifier_prefix != input.identifier_prefix.trim().to_uppercase() {
        changes.push("changed the project identifier".into());
    }
    if previous.status != input.status {
        changes.push(format!(
            "changed status from {} to {}",
            previous.status, input.status
        ));
    }
    if previous.priority != input.priority {
        changes.push(format!(
            "changed priority from {} to {}",
            previous.priority, input.priority
        ));
    }
    if previous.lead_actor_id != input.lead_actor_id {
        changes.push(if input.lead_actor_id.is_some() {
            "assigned the project lead".into()
        } else {
            "removed the project lead".into()
        });
    }
    if previous.start_date != input.start_date {
        changes.push("changed the start date".into());
    }
    if previous.target_date != input.target_date {
        changes.push("changed the target date".into());
    }
    if previous.labels != input.labels {
        changes.push("changed labels".into());
    }
    if previous.icon != input.icon || previous.color != input.color {
        changes.push("changed the project icon or color".into());
    }
    if previous.summary != input.summary.trim() {
        changes.push("edited the summary".into());
    }
    if previous.description != input.description.trim()
        || previous.description_document != input.description_document
    {
        changes.push("edited the description".into());
    }
    if previous.resources != input.resources {
        changes.push("changed resources".into());
    }
    if previous.milestones.len() != input.milestones.len()
        || previous
            .milestones
            .iter()
            .zip(&input.milestones)
            .any(|(old, new)| {
                new.id.as_deref() != Some(old.id.as_str())
                    || old.title != new.title.trim()
                    || old.description != new.description.trim()
                    || old.target_date != new.target_date
            })
    {
        changes.push("changed milestones".into());
    }
    changes
}

#[api_contract_macros::command]
pub fn storage_update_project_full(
    app: AppHandle,
    id: String,
    input: ProjectInput,
    mutation: MutationRequest,
) -> Result<ProjectRecord, String> {
    update_project_full_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &id,
        input,
        Some(mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_change_project_resource(
    app: AppHandle,
    project_id: String,
    action: String,
    resource: Option<ProjectResource>,
    resource_id: Option<String>,
    order: Option<Vec<String>>,
    mutation: MutationRequest,
) -> Result<ProjectRecord, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    change_project_resource(
        &mut connection,
        project_id,
        action,
        resource,
        resource_id,
        order,
        mutation,
    )
}

fn change_project_resource(
    connection: &mut Connection,
    project_id: String,
    action: String,
    resource: Option<ProjectResource>,
    resource_id: Option<String>,
    order: Option<Vec<String>>,
    mutation: MutationRequest,
) -> Result<ProjectRecord, String> {
    let workspace = current_workspace_id(&connection)?;
    let hash = mutations::request_hash(
        "project.resource.change",
        &(
            &project_id,
            &action,
            &resource,
            &resource_id,
            &order,
            mutation.expected_version,
        ),
    )?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin resource update.".to_owned())?;
    if let Some(previous) =
        mutations::replay(&tx, &workspace, "project.resource.change", &hash, &mutation)?
    {
        return Ok(previous);
    }
    mutations::check_version(&tx, "project", &project_id, &mutation)?;
    let active: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1 AND workspace_id = ?2
         AND archived_at IS NULL AND deleted_at IS NULL)",
            params![project_id, workspace],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read project resource scope.".to_owned())?;
    if !active {
        return Err("Active project not found.".into());
    }

    let mut resources = read_project(&tx, &project_id)?.resources;
    match action.as_str() {
        "create" | "update" => {
            let item = resource.ok_or("Choose a resource.".to_owned())?;
            if item.id.len() > 80
                || item.id.is_empty()
                || item.title.trim().is_empty()
                || item.title.chars().count() > 120
                || item.url.len() > 2048
                || !(item.url.starts_with("https://") || item.url.starts_with("http://"))
            {
                return Err("Enter a title and an HTTP(S) link.".into());
            }
            let existing = resources
                .iter()
                .position(|candidate| candidate.id == item.id);
            if action == "create" {
                if existing.is_some() || resources.len() >= 100 {
                    return Err("Resource already exists or project is full.".into());
                }
                resources.push(item);
            } else {
                let index = existing.ok_or("Resource not found.".to_owned())?;
                resources[index] = item;
            }
        }
        "remove" => {
            let id = resource_id.ok_or("Choose a resource.".to_owned())?;
            let before = resources.len();
            resources.retain(|item| item.id != id);
            if resources.len() == before {
                return Err("Resource not found.".into());
            }
        }
        "reorder" => {
            let ids = order.ok_or("Choose resource order.".to_owned())?;
            if ids.len() != resources.len() || ids.iter().collect::<HashSet<_>>().len() != ids.len()
            {
                return Err("Resource order is incomplete.".into());
            }
            let mut ordered = Vec::with_capacity(ids.len());
            for id in ids {
                let index = resources
                    .iter()
                    .position(|item| item.id == id)
                    .ok_or("Resource order has unknown items.".to_owned())?;
                ordered.push(resources.remove(index));
            }
            resources = ordered;
        }
        _ => return Err("Unknown resource action.".into()),
    }

    sync_project_resources(&tx, &project_id, &resources)?;
    tx.execute(
        "UPDATE project SET resources_json = ?1 WHERE id = ?2",
        params![
            serde_json::to_string(&resources).map_err(|_| "Cannot encode resources.".to_owned())?,
            project_id
        ],
    )
    .map_err(|_| "Cannot save project resource.".to_owned())?;
    let actor = read_identity(&tx)?.ok_or("Set up your workspace first.".to_owned())?;
    let activity = match action.as_str() {
        "create" => "added a resource",
        "update" => "edited a resource",
        "remove" => "removed a resource",
        _ => "reordered resources",
    };
    tx.execute(
        "INSERT INTO project_update(id, project_id, actor_name, body, kind)
         VALUES (?1, ?2, ?3, ?4, 'change')",
        params![
            Uuid::new_v4().to_string(),
            project_id,
            actor.user_name,
            activity
        ],
    )
    .map_err(|_| "Cannot record resource change.".to_owned())?;
    let record = read_project(&tx, &project_id)?;
    mutations::save(
        &tx,
        &workspace,
        "project.resource.change",
        &hash,
        &mutation,
        &record,
    )?;
    tx.commit()
        .map_err(|_| "Cannot save project resource.".to_owned())?;
    Ok(record)
}

#[api_contract_macros::command]
pub fn storage_list_projects_page(
    app: AppHandle,
    cursor: Option<String>,
    limit: Option<usize>,
    sort: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    search: Option<String>,
) -> Result<ProjectPage, String> {
    list_projects_page_query(
        &open_database(&document_data_root(&app)?)?,
        cursor.as_deref(),
        limit.unwrap_or(50),
        sort.as_deref().unwrap_or("updated"),
        status.as_deref(),
        priority.as_deref(),
        search.as_deref(),
    )
}

#[api_contract_macros::command]
pub fn storage_get_project(app: AppHandle, id: String) -> Result<ProjectRecord, String> {
    let connection = open_database(&document_data_root(&app)?)?;
    let workspace_id = current_workspace_id(&connection)?;
    let valid: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1 AND workspace_id = ?2 AND archived_at IS NULL AND deleted_at IS NULL)",
            params![id, workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot find project.".to_owned())?;
    if !valid {
        return Err("Project not found.".into());
    }
    read_project(&connection, &id)
}

#[api_contract_macros::command]
pub fn storage_get_project_by_name(app: AppHandle, name: String) -> Result<ProjectRecord, String> {
    let connection = open_database(&document_data_root(&app)?)?;
    let workspace_id = current_workspace_id(&connection)?;
    let id: String = connection
        .query_row(
            "SELECT id FROM project WHERE workspace_id = ?1 AND name = ?2 AND archived_at IS NULL AND deleted_at IS NULL",
            params![workspace_id, name],
            |row| row.get(0),
        )
        .map_err(|_| "Project not found.".to_owned())?;
    read_project(&connection, &id)
}

#[api_contract_macros::command]
pub fn storage_create_project_full(
    app: AppHandle,
    input: ProjectInput,
    mutation: MutationRequest,
) -> Result<ProjectRecord, String> {
    let root = document_data_root(&app)?;
    create_project_full_with_mutation(&mut open_database(&root)?, input, Some(mutation))
}

#[cfg(test)]
fn list_project_updates(
    connection: &Connection,
    project_id: &str,
    offset: i64,
) -> Result<Vec<ProjectUpdate>, String> {
    if offset < 0 {
        return Err("Invalid activity page.".into());
    }
    let mut updates = Vec::new();
    let mut cursor = None;
    loop {
        let page = project_update_page(connection, project_id, cursor.as_deref())?;
        updates.extend(page.items);
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }
    Ok(updates.into_iter().skip(offset as usize).take(30).collect())
}

fn project_update_page(
    connection: &Connection,
    project_id: &str,
    cursor: Option<&str>,
) -> Result<ProjectUpdatePage, String> {
    let workspace_id = current_workspace_id(connection)?;
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1 AND workspace_id = ?2)",
            params![project_id, workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot find the project.".to_owned())?;
    if !exists {
        return Err("Project not found.".into());
    }
    let after = cursor
        .map(|token| -> Result<ProjectUpdateCursor, String> {
            let bytes = URL_SAFE_NO_PAD
                .decode(token)
                .map_err(|_| "Project activity cursor is invalid.")?;
            if bytes.len() > 512 {
                return Err("Project activity cursor is too large.".into());
            }
            let value: ProjectUpdateCursor = serde_json::from_slice(&bytes)
                .map_err(|_| "Project activity cursor is invalid.")?;
            if value.project_id != project_id || value.row_id < 1 {
                return Err("Project activity cursor does not match this project.".into());
            }
            Ok(value)
        })
        .transpose()?;
    let mut statement = connection
        .prepare(
            "SELECT id, project_id, actor_name, body, created_at, kind, rowid FROM project_update
         WHERE project_id = ?1 AND (?2 IS NULL OR created_at < ?2 OR
           (created_at = ?2 AND rowid < ?3))
         ORDER BY created_at DESC, rowid DESC LIMIT 31",
        )
        .map_err(|_| "Cannot read project updates.".to_owned())?;
    let mut rows = statement
        .query_map(
            params![
                project_id,
                after.as_ref().map(|value| &value.created_at),
                after.as_ref().map(|value| value.row_id)
            ],
            |row| {
                Ok((
                    ProjectUpdate {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        actor_name: row.get(2)?,
                        body: row.get(3)?,
                        created_at: row.get(4)?,
                        kind: row.get(5)?,
                    },
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .map_err(|_| "Cannot read project updates.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read project updates.".to_owned())?;
    let has_more = rows.len() > 30;
    rows.truncate(30);
    let next_cursor = if has_more {
        let last = rows.last().ok_or("Cannot read project activity cursor.")?;
        Some(
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&ProjectUpdateCursor {
                    project_id: project_id.to_owned(),
                    created_at: last.0.created_at.clone(),
                    row_id: last.1,
                })
                .map_err(|_| "Cannot encode project activity cursor.")?,
            ),
        )
    } else {
        None
    };
    Ok(ProjectUpdatePage {
        items: rows.into_iter().map(|row| row.0).collect(),
        next_cursor,
    })
}

fn create_project_update(
    connection: &Connection,
    project_id: &str,
    body: &str,
) -> Result<ProjectUpdate, String> {
    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 100_000 {
        return Err("Write an update of up to 100,000 characters.".into());
    }
    let identity =
        read_identity(connection)?.ok_or_else(|| "Set up your workspace first.".to_owned())?;
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1 AND workspace_id = ?2)",
            params![project_id, identity.workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot find the project.".to_owned())?;
    if !exists {
        return Err("Project not found.".into());
    }
    let id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO project_update (id, project_id, actor_name, body) VALUES (?1, ?2, ?3, ?4)",
            params![id, project_id, identity.user_name, trimmed],
        )
        .map_err(|_| "Cannot save the project update.".to_owned())?;
    connection
        .query_row(
            "SELECT id, project_id, actor_name, body, created_at, kind FROM project_update WHERE id = ?1",
            [&id],
            |row| {
                Ok(ProjectUpdate {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    actor_name: row.get(2)?,
                    body: row.get(3)?,
                    created_at: row.get(4)?,
                    kind: row.get(5)?,
                })
            },
        )
        .map_err(|_| "Cannot read the saved project update.".to_owned())
}

#[api_contract_macros::command]
pub fn storage_list_project_updates(
    app: AppHandle,
    project_id: String,
    cursor: Option<String>,
) -> Result<ProjectUpdatePage, String> {
    project_update_page(
        &open_database(&document_data_root(&app)?)?,
        &project_id,
        cursor.as_deref(),
    )
}

#[api_contract_macros::command]
pub fn storage_create_project_update(
    app: AppHandle,
    project_id: String,
    body: String,
    mutation: MutationRequest,
) -> Result<ProjectUpdate, String> {
    create_project_update_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &project_id,
        &body,
        &mutation,
    )
}

fn create_project_update_with_mutation(
    connection: &mut Connection,
    project_id: &str,
    body: &str,
    mutation: &MutationRequest,
) -> Result<ProjectUpdate, String> {
    mutations::apply(
        connection,
        "project.update.post",
        &(project_id, body),
        mutation,
        |tx, workspace_id| {
            let update = create_project_update(tx, &project_id, &body)?;
            tx.execute(
                "UPDATE workspace SET revision = revision + 1 WHERE id = ?1",
                [workspace_id],
            )
            .map_err(|_| "Cannot update workspace revision.".to_owned())?;
            Ok(update)
        },
    )
}

fn read_identity(connection: &Connection) -> Result<Option<Identity>, String> {
    connection
        .query_row(
            "SELECT workspace.id, local_actor.id, local_actor.display_name, workspace.name
             FROM workspace JOIN local_actor ON local_actor.workspace_id = workspace.id
             WHERE local_actor.singleton = 1",
            [],
            |row| {
                Ok(Identity {
                    workspace_id: row.get(0)?,
                    actor_id: row.get(1)?,
                    user_name: row.get(2)?,
                    workspace_name: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|_| "Cannot read the workspace database.".to_owned())
}

pub(crate) fn current_workspace_id(connection: &Connection) -> Result<String, String> {
    read_identity(connection)?
        .map(|identity| identity.workspace_id)
        .ok_or_else(|| "Set up your workspace first.".to_owned())
}

fn validate_name(name: &str) -> Result<&str, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 80 {
        return Err("Names must contain between 1 and 80 characters.".into());
    }
    Ok(trimmed)
}

#[api_contract_macros::command]
pub fn storage_bootstrap(app: AppHandle) -> Result<Bootstrap, String> {
    let root = document_data_root(&app)?;
    let connection = open_database(&root)?;
    let restored = root.join("recovery").join("restore.completed");
    let restored_backup = restored.exists();
    let identity = read_identity(&connection)?;
    let workspace_revision = identity
        .as_ref()
        .map_or(Ok(0), |identity| {
            connection.query_row(
                "SELECT revision FROM workspace WHERE id = ?1",
                [&identity.workspace_id],
                |row| row.get(0),
            )
        })
        .map_err(|_| "Cannot read workspace revision.".to_owned())?;
    let workspaces = list_workspaces(&connection)?;
    let recovered_timers: i64 = connection
        .query_row(
            "SELECT recovered_timers FROM timer_session WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read timer recovery.".to_owned())?;
    connection
        .execute(
            "UPDATE timer_session SET recovered_timers = 0 WHERE singleton = 1",
            [],
        )
        .map_err(|_| "Cannot clear timer recovery notice.".to_owned())?;
    Ok(Bootstrap {
        data_root: root.to_string_lossy().into_owned(),
        contract_version: 1,
        workspace_revision,
        identity_version: identity_version(&connection)?,
        identity,
        workspaces,
        restore_error: fs::read_to_string(root.join("recovery").join("restore.failed")).ok(),
        restored_backup,
        recovered_timers,
    })
}

fn list_workspaces(connection: &Connection) -> Result<Vec<WorkspaceEntry>, String> {
    let mut query = connection
        .prepare("SELECT id, name FROM workspace ORDER BY name COLLATE NOCASE, id")
        .map_err(|_| "Cannot list workspaces.".to_owned())?;
    let workspaces = query
        .query_map([], |row| {
            Ok(WorkspaceEntry {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|_| "Cannot list workspaces.".to_owned())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Cannot list workspaces.".to_owned())?;
    Ok(workspaces)
}

#[api_contract_macros::command]
pub fn storage_create_workspace(
    app: AppHandle,
    name: String,
    mutation: MutationRequest,
) -> Result<Identity, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    create_workspace_with_mutation(&mut connection, &name, Some(&mutation))
}

#[cfg(test)]
pub(crate) fn create_workspace(
    connection: &mut Connection,
    name: &str,
) -> Result<Identity, String> {
    create_workspace_with_mutation(connection, name, None)
}

fn create_workspace_with_mutation(
    connection: &mut Connection,
    name: &str,
    mutation: Option<&MutationRequest>,
) -> Result<Identity, String> {
    let name = validate_name(name)?;
    let transaction = connection
        .transaction()
        .map_err(|_| "Cannot create workspace.".to_owned())?;
    let request_hash = mutations::request_hash("workspace.create", &name)?;
    if let Some(request) = mutation {
        if request.expected_version.is_some() {
            return Err("New workspaces cannot have an expected version.".into());
        }
        let receipt_workspace: Option<String> = transaction
            .query_row(
                "SELECT workspace_id FROM mutation_receipt WHERE mutation_id = ?1",
                [&request.mutation_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "Cannot read workspace creation receipt.".to_owned())?;
        if let Some(workspace_id) = receipt_workspace {
            if let Some(identity) = mutations::replay::<Identity>(
                &transaction,
                &workspace_id,
                "workspace.create",
                &request_hash,
                request,
            )? {
                transaction
                    .execute(
                        "UPDATE local_actor SET workspace_id = ?1 WHERE singleton = 1",
                        [&identity.workspace_id],
                    )
                    .map_err(|_| "Cannot select new workspace.".to_owned())?;
                transaction
                    .commit()
                    .map_err(|_| "Cannot select new workspace.".to_owned())?;
                return Ok(identity);
            }
        }
    }
    let id = Uuid::new_v4().to_string();
    transaction
        .execute(
            "INSERT INTO workspace (id, name) VALUES (?1, ?2)",
            params![id, name],
        )
        .map_err(|_| "Cannot create workspace.".to_owned())?;
    crate::labels::seed_default_labels(&transaction, &id)?;
    transaction
        .execute(
            "UPDATE local_actor SET workspace_id = ?1 WHERE singleton = 1",
            [&id],
        )
        .map_err(|_| "Cannot select new workspace.".to_owned())?;
    let identity =
        read_identity(&transaction)?.ok_or_else(|| "Cannot select new workspace.".to_owned())?;
    if let Some(request) = mutation {
        mutations::save(
            &transaction,
            &id,
            "workspace.create",
            &request_hash,
            request,
            &identity,
        )?;
    }
    transaction
        .commit()
        .map_err(|_| "Cannot save new workspace.".to_owned())?;
    Ok(identity)
}

#[api_contract_macros::command]
pub fn storage_switch_workspace(
    app: AppHandle,
    id: String,
    mutation: MutationRequest,
) -> Result<Identity, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    switch_workspace_with_mutation(&mut connection, &id, &mutation)
}

fn switch_workspace_with_mutation(
    connection: &mut Connection,
    id: &str,
    mutation: &MutationRequest,
) -> Result<Identity, String> {
    if mutation.expected_version.is_some() {
        return Err("Switching workspaces cannot have an expected version.".into());
    }
    let request_hash = mutations::request_hash("workspace.switch", &id)?;
    let transaction = connection
        .transaction()
        .map_err(|_| "Cannot switch workspace.".to_owned())?;
    if let Some(identity) = mutations::replay::<Identity>(
        &transaction,
        id,
        "workspace.switch",
        &request_hash,
        mutation,
    )? {
        transaction
            .execute(
                "UPDATE local_actor SET workspace_id = ?1 WHERE singleton = 1",
                [id],
            )
            .map_err(|_| "Cannot switch workspace.".to_owned())?;
        transaction
            .commit()
            .map_err(|_| "Cannot switch workspace.".to_owned())?;
        return Ok(identity);
    }
    let identity = switch_workspace(&transaction, id)?;
    mutations::save(
        &transaction,
        id,
        "workspace.switch",
        &request_hash,
        mutation,
        &identity,
    )?;
    transaction
        .commit()
        .map_err(|_| "Cannot switch workspace.".to_owned())?;
    Ok(identity)
}

pub(crate) fn switch_workspace(connection: &Connection, id: &str) -> Result<Identity, String> {
    let changed = connection
        .execute(
            "UPDATE local_actor SET workspace_id = ?1 WHERE singleton = 1
         AND EXISTS (SELECT 1 FROM workspace WHERE id = ?1)",
            [&id],
        )
        .map_err(|_| "Cannot switch workspace.".to_owned())?;
    if changed == 0 {
        return Err("Workspace not found.".into());
    }
    read_identity(&connection)?.ok_or_else(|| "Cannot switch workspace.".to_owned())
}

#[api_contract_macros::command]
pub fn storage_save_identity(
    app: AppHandle,
    user_name: String,
    workspace_name: String,
    mutation: MutationRequest,
) -> Result<Identity, String> {
    let root = document_data_root(&app)?;
    let mut connection = open_database(&root)?;
    save_identity_with_mutation(
        &mut connection,
        &user_name,
        &workspace_name,
        Some(&mutation),
    )
}

#[api_contract_macros::command]
pub fn storage_identity_version(app: AppHandle) -> Result<Option<i64>, String> {
    identity_version(&open_database(&document_data_root(&app)?)?)
}

#[cfg(test)]
pub(crate) fn save_identity(
    connection: &mut Connection,
    user_name: &str,
    workspace_name: &str,
) -> Result<Identity, String> {
    save_identity_with_mutation(connection, user_name, workspace_name, None)
}

fn save_identity_with_mutation(
    connection: &mut Connection,
    user_name: &str,
    workspace_name: &str,
    mutation: Option<&MutationRequest>,
) -> Result<Identity, String> {
    let user_name = validate_name(user_name)?;
    let workspace_name = validate_name(workspace_name)?;
    let transaction = connection
        .transaction()
        .map_err(|_| "Cannot begin the workspace update.".to_owned())?;
    let request_hash = mutations::request_hash(
        "workspace.identity.save",
        &(
            user_name,
            workspace_name,
            mutation.and_then(|item| item.expected_version),
        ),
    )?;
    if let Some(request) = mutation {
        let receipt_workspace: Option<String> = transaction
            .query_row(
                "SELECT workspace_id FROM mutation_receipt WHERE mutation_id = ?1",
                [&request.mutation_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "Cannot read workspace identity receipt.".to_owned())?;
        if let Some(workspace_id) = receipt_workspace {
            if let Some(identity) = mutations::replay::<Identity>(
                &transaction,
                &workspace_id,
                "workspace.identity.save",
                &request_hash,
                request,
            )? {
                return Ok(identity);
            }
        }
        if request.expected_version != identity_version(&transaction)? {
            return Err("CONFLICT: Workspace identity changed. Reload and try again.".into());
        }
    }
    if read_identity(&transaction)?.is_some() {
        transaction
            .execute(
                "UPDATE workspace SET name = ?1 WHERE id = (SELECT workspace_id FROM local_actor WHERE singleton = 1)",
                [workspace_name],
            )
            .map_err(|_| "Cannot update the workspace name.".to_owned())?;
        transaction
            .execute(
                "UPDATE local_actor SET display_name = ?1 WHERE singleton = 1",
                [user_name],
            )
            .map_err(|_| "Cannot update your name.".to_owned())?;
    } else {
        let workspace_id = Uuid::new_v4().to_string();
        let actor_id = Uuid::new_v4().to_string();
        transaction
            .execute(
                "INSERT INTO workspace (id, name) VALUES (?1, ?2)",
                params![workspace_id, workspace_name],
            )
            .map_err(|_| "Cannot create the workspace.".to_owned())?;
        crate::labels::seed_default_labels(&transaction, &workspace_id)?;
        transaction
            .execute(
                "INSERT INTO local_actor (singleton, id, workspace_id, display_name)
                 VALUES (1, ?1, ?2, ?3)",
                params![actor_id, workspace_id, user_name],
            )
            .map_err(|_| "Cannot create your local profile.".to_owned())?;
    }
    let identity = read_identity(&transaction)?
        .ok_or_else(|| "Cannot confirm the workspace update.".to_owned())?;
    if let Some(request) = mutation {
        mutations::save(
            &transaction,
            &identity.workspace_id,
            "workspace.identity.save",
            &request_hash,
            request,
            &identity,
        )?;
    }
    transaction
        .commit()
        .map_err(|_| "Cannot save the workspace update.".to_owned())?;
    Ok(identity)
}

#[cfg(test)]
mod tests;
