use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
};

use rusqlite::{hooks::PreUpdateCase, types::ValueRef, OptionalExtension};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri::AppHandle;
use uuid::Uuid;

use crate::mutations::{self, MutationRequest};
use crate::storage::{current_workspace_id, document_data_root, open_database};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationEffects {
    pub changed: Vec<EntityRef>,
    pub removed_from_active: Vec<EntityRef>,
}

#[derive(Clone)]
struct CapturedChange {
    reference: EntityRef,
    removed: bool,
}

thread_local! {
    // SQLite hooks run on the mutating thread; keep effects scoped to one command response.
    static CAPTURED_CHANGES: RefCell<Option<Vec<CapturedChange>>> = const { RefCell::new(None) };
}

pub struct ChangeCaptureGuard;

pub fn begin_change_capture() -> ChangeCaptureGuard {
    CAPTURED_CHANGES.with(|changes| *changes.borrow_mut() = Some(Vec::new()));
    ChangeCaptureGuard
}

impl Drop for ChangeCaptureGuard {
    fn drop(&mut self) {
        CAPTURED_CHANGES.with(|changes| *changes.borrow_mut() = None);
    }
}

pub fn captured_effects() -> Option<MutationEffects> {
    CAPTURED_CHANGES.with(|changes| {
        changes.borrow().as_ref().map(|changes| {
            let mut effects = MutationEffects {
                changed: changes
                    .iter()
                    .map(|change| change.reference.clone())
                    .collect(),
                removed_from_active: changes
                    .iter()
                    .filter(|change| change.removed)
                    .map(|change| change.reference.clone())
                    .collect(),
            };
            for refs in [&mut effects.changed, &mut effects.removed_from_active] {
                refs.sort_by(|a, b| (&a.kind, &a.id).cmp(&(&b.kind, &b.id)));
                refs.dedup_by(|a, b| a.kind == b.kind && a.id == b.id);
            }
            effects
        })
    })
}

pub fn record_change(id: &str, kind: &str, removed: bool) {
    CAPTURED_CHANGES.with(|changes| {
        if let Some(changes) = changes.borrow_mut().as_mut() {
            changes.push(CapturedChange {
                reference: EntityRef {
                    id: id.to_owned(),
                    kind: kind.to_owned(),
                },
                removed,
            });
        }
    });
}

pub fn install_change_capture(connection: &rusqlite::Connection) -> Result<(), String> {
    // Discover column positions from the live schema so a column reorder cannot silently
    // mislabel a mutation effect. Only registered entities appear in API receipts.
    let mut metadata = HashMap::new();
    for (table, kind, id_column) in [
        ("issue", "issue", "id"),
        ("project", "project", "id"),
        ("saved_view", "savedView", "id"),
        ("label", "label", "id"),
        ("time_entry", "timeEntry", "id"),
        ("cycle", "cycle", "id"),
        ("cycle_series", "cycleSeries", "id"),
        ("planning_node", "planningNode", "id"),
        ("milestone", "milestone", "id"),
        ("issue_activity", "activity", "id"),
        ("issue_relation", "issueRelation", "id"),
        ("project_update", "projectUpdate", "id"),
        ("media", "media", "id"),
        ("workspace", "workspace", "id"),
        ("workspace_preferences", "workspace", "workspace_id"),
        ("local_actor", "workspace", "workspace_id"),
    ] {
        let mut statement = connection
            .prepare(&format!("PRAGMA table_info({table})"))
            .map_err(|_| "Cannot inspect mutation effect tables.".to_owned())?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|_| "Cannot inspect mutation effect tables.".to_owned())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Cannot inspect mutation effect tables.".to_owned())?;
        if let Some(id_index) = columns.iter().position(|column| column == id_column) {
            let lifecycle = columns
                .iter()
                .enumerate()
                .filter(|(_, column)| {
                    matches!(
                        column.as_str(),
                        "archived_at" | "deleted_at" | "dissolved_at" | "removed_at"
                    )
                })
                .map(|(index, _)| index as i32)
                .collect::<Vec<_>>();
            metadata.insert(table, (kind, id_index as i32, lifecycle));
        }
    }

    connection.preupdate_hook(Some(
        move |_, database: &str, table: &str, change: &PreUpdateCase| {
            if database != "main" {
                return;
            }
            let Some((kind, id_index, lifecycle)) = metadata.get(table) else {
                return;
            };
            let (id, removed) = match change {
                PreUpdateCase::Insert(new) => (new.get_new_column_value(*id_index), false),
                PreUpdateCase::Delete(old) => (
                    old.get_old_column_value(*id_index),
                    lifecycle.iter().all(|index| {
                        old.get_old_column_value(*index)
                            .is_ok_and(|value| value == ValueRef::Null)
                    }),
                ),
                PreUpdateCase::Update {
                    old_value_accessor: old,
                    new_value_accessor: new,
                } => {
                    let removed = lifecycle.iter().any(|index| {
                        old.get_old_column_value(*index)
                            .is_ok_and(|value| value == ValueRef::Null)
                            && new
                                .get_new_column_value(*index)
                                .is_ok_and(|value| value != ValueRef::Null)
                    });
                    (new.get_new_column_value(*id_index), removed)
                }
                PreUpdateCase::Unknown => return,
            };
            let Ok(ValueRef::Text(id)) = id else {
                return;
            };
            let Ok(id) = std::str::from_utf8(id) else {
                return;
            };
            if Uuid::parse_str(id).is_err() {
                return;
            }
            CAPTURED_CHANGES.with(|changes| {
                if let Some(changes) = changes.borrow_mut().as_mut() {
                    changes.push(CapturedChange {
                        reference: EntityRef {
                            id: id.to_owned(),
                            kind: (*kind).to_owned(),
                        },
                        removed,
                    });
                }
            });
        },
    ));
    Ok(())
}

pub fn apply_side_effect<T, F>(
    app: &AppHandle,
    operation: &str,
    mutation_id: &str,
    input: &serde_json::Value,
    action: F,
) -> Result<T, ApiError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Result<T, String>,
{
    let root = document_data_root(app).map_err(ApiError::from)?;
    let connection = open_database(&root).map_err(ApiError::from)?;
    let workspace_id = current_workspace_id(&connection).map_err(ApiError::from)?;
    let request = MutationRequest {
        mutation_id: mutation_id.to_owned(),
        expected_version: None,
        expected_versions: BTreeMap::new(),
    };
    let hash = mutations::request_hash(operation, input).map_err(ApiError::from)?;
    if let Some(result) = mutations::replay(&connection, &workspace_id, operation, &hash, &request)
        .map_err(ApiError::from)?
    {
        return Ok(result);
    }
    drop(connection);

    let is_restore = matches!(
        operation,
        "storage_restore_backup" | "storage_finish_import_upload"
    );
    if is_restore {
        crate::portability::stage_restore_receipt(&root, &request, operation, &hash)
            .map_err(ApiError::from)?;
    }
    let result = match action() {
        Ok(result) => result,
        Err(error) => {
            if is_restore {
                crate::portability::discard_restore_receipt(&root);
            }
            return Err(ApiError::from(error));
        }
    };
    if is_restore {
        if let Err(error) = crate::portability::mark_restore_receipt_ready(&root) {
            crate::portability::discard_restore_receipt(&root);
            return Err(ApiError::from(error));
        }
        record_change(&workspace_id, "workspace", false);
    }
    let connection = match open_database(&root) {
        Ok(connection) => connection,
        Err(error) => {
            if is_restore {
                crate::portability::discard_restore_receipt(&root);
            }
            return Err(ApiError::from(error));
        }
    };
    if let Err(error) = mutations::save(
        &connection,
        &workspace_id,
        operation,
        &hash,
        &request,
        &result,
    ) {
        if is_restore {
            crate::portability::discard_restore_receipt(&root);
        }
        return Err(ApiError::from(error));
    }
    Ok(result)
}

pub fn restart_after_response(app: AppHandle) {
    let Ok(root) = document_data_root(&app) else {
        return;
    };
    if !root.join("recovery/restore.requested").exists() {
        return;
    }
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(250));
        app.restart();
    });
}

#[api_contract_macros::command]
pub fn storage_resolve_issue_entity_ids(
    app: AppHandle,
    identifiers: Vec<String>,
) -> Result<BTreeMap<String, String>, String> {
    if identifiers.len() > 1_000 {
        return Err("VALIDATION: Resolve at most 1,000 issues at once.".into());
    }
    let connection = open_database(&document_data_root(&app)?)?;
    let workspace_id = current_workspace_id(&connection)?;
    let mut ids = BTreeMap::new();
    for identifier in identifiers {
        let id: String = connection
            .query_row(
                "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2",
                [&workspace_id, &identifier],
                |row| row.get(0),
            )
            .map_err(|_| format!("NOT_FOUND: Issue {identifier} was not found."))?;
        ids.insert(identifier, id);
    }
    Ok(ids)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApiRequest<T> {
    contract_version: u8,
    request_id: Option<String>,
    mutation_id: Option<String>,
    expected_versions: Option<BTreeMap<String, i64>>,
    pub input: T,
}

impl<T> ApiRequest<T> {
    pub fn mutation_id(&self) -> Option<&str> {
        self.mutation_id.as_deref()
    }

    pub fn expected_versions(&self) -> Option<&BTreeMap<String, i64>> {
        self.expected_versions.as_ref()
    }

    pub fn validate(&self, mutation_required: bool) -> Result<(), ApiError> {
        if self.contract_version != 1 {
            return Err(ApiError::new(
                "UNSUPPORTED_VERSION",
                "Unsupported API version.",
                false,
            ));
        }
        if mutation_required && self.request_id.is_some() {
            return Err(ApiError::validation(
                "Mutation requests cannot contain a query ID.",
            ));
        }
        if !mutation_required && self.mutation_id.is_some() {
            return Err(ApiError::validation(
                "Queries cannot contain a mutation ID.",
            ));
        }
        if !mutation_required && self.expected_versions.is_some() {
            return Err(ApiError::validation(
                "Queries cannot contain expected versions.",
            ));
        }
        let id = if mutation_required {
            self.mutation_id.as_deref()
        } else {
            self.request_id.as_deref()
        };
        if !id.is_some_and(|id| Uuid::parse_str(id).is_ok()) {
            return Err(ApiError::new(
                "VALIDATION",
                "A valid request ID is required.",
                false,
            ));
        }
        if mutation_required && self.expected_versions.is_none() {
            return Err(ApiError::new(
                "VALIDATION",
                "Expected versions are required.",
                false,
            ));
        }
        if mutation_required
            && self.expected_versions.as_ref().is_some_and(|versions| {
                versions
                    .iter()
                    .any(|(id, version)| Uuid::parse_str(id).is_err() || *version < 0)
            })
        {
            return Err(ApiError::validation(
                "Expected versions must use entity UUIDs and nonnegative versions.",
            ));
        }
        Ok(())
    }

    pub fn validate_mutation_payload(
        &self,
        app: &AppHandle,
        command: &str,
        payload: &serde_json::Value,
    ) -> Result<(), ApiError>
    where
        T: Serialize,
    {
        let id = payload
            .get("mutationId")
            .and_then(serde_json::Value::as_str);
        if id != self.mutation_id.as_deref() {
            return Err(ApiError::new(
                "VALIDATION",
                "Mutation IDs do not match.",
                false,
            ));
        }
        let input = serde_json::to_value(&self.input)
            .map_err(|_| ApiError::validation("Cannot encode request input."))?;
        let mut expected = BTreeMap::new();
        let root = document_data_root(app).map_err(ApiError::from)?;
        let connection = open_database(&root).map_err(ApiError::from)?;
        let workspace_id = current_workspace_id(&connection).ok();

        if let Some(version) = payload
            .get("expectedVersion")
            .and_then(serde_json::Value::as_i64)
        {
            let target = if command.contains("planning_node")
                || matches!(
                    command,
                    "storage_start_timer" | "storage_stop_timer" | "storage_restore_milestone"
                ) {
                if command == "storage_restore_milestone" || command.contains("planning_node") {
                    input.get("projectId").and_then(serde_json::Value::as_str)
                } else {
                    workspace_id.as_deref()
                }
            } else {
                [
                    "id",
                    "entryId",
                    "commentId",
                    "cycleId",
                    "seriesId",
                    "milestoneId",
                    "projectId",
                    "identifier",
                ]
                .iter()
                .find_map(|name| input.get(*name).and_then(serde_json::Value::as_str))
                .or_else(|| {
                    input
                        .get("view")
                        .and_then(|view| view.get("id"))
                        .and_then(serde_json::Value::as_str)
                })
                .or_else(|| {
                    input
                        .get("settings")
                        .and_then(|settings| settings.get("id"))
                        .and_then(serde_json::Value::as_str)
                })
                .or(workspace_id.as_deref())
            }
            .ok_or_else(|| ApiError::validation("Cannot identify versioned entity."))?;
            expected.insert(
                resolve_entity_id(&connection, workspace_id.as_deref(), target)?,
                version,
            );
        }

        if let Some(versions) = payload
            .get("versions")
            .and_then(serde_json::Value::as_object)
        {
            for (identifier, version) in versions {
                let version = version
                    .as_i64()
                    .ok_or_else(|| ApiError::validation("Invalid version."))?;
                expected.insert(
                    resolve_entity_id(&connection, workspace_id.as_deref(), identifier)?,
                    version,
                );
            }
        }
        if let Some(targets) = input.get("targets").and_then(serde_json::Value::as_array) {
            for target in targets {
                let identifier = target
                    .get("identifier")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| ApiError::validation("Invalid target identifier."))?;
                let version = target
                    .get("version")
                    .and_then(serde_json::Value::as_i64)
                    .ok_or_else(|| ApiError::validation("Invalid target version."))?;
                expected.insert(
                    resolve_entity_id(&connection, workspace_id.as_deref(), identifier)?,
                    version,
                );
            }
        }
        for (id_field, version_field) in [
            ("targetIdentifier", "targetVersion"),
            ("nextCycleId", "nextCycleVersion"),
        ] {
            if let (Some(identifier), Some(version)) = (
                input.get(id_field).and_then(serde_json::Value::as_str),
                input.get(version_field).and_then(serde_json::Value::as_i64),
            ) {
                expected.insert(
                    resolve_entity_id(&connection, workspace_id.as_deref(), identifier)?,
                    version,
                );
            }
        }
        if self.expected_versions.as_ref() != Some(&expected) {
            return Err(ApiError::validation(
                "Expected versions do not match the affected entities.",
            ));
        }
        Ok(())
    }
}

fn resolve_entity_id(
    connection: &rusqlite::Connection,
    workspace_id: Option<&str>,
    key: &str,
) -> Result<String, ApiError> {
    if Uuid::parse_str(key).is_ok() {
        return Ok(key.to_owned());
    }
    let workspace_id =
        workspace_id.ok_or_else(|| ApiError::validation("Workspace is required."))?;
    connection
        .query_row(
            "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2",
            [workspace_id, key],
            |row| row.get(0),
        )
        .map_err(|_| ApiError::new("NOT_FOUND", "Versioned issue was not found.", false))
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRef {
    pub id: String,
    pub kind: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub data: T,
    pub workspace_revision: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed: Option<Vec<EntityRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed_from_active: Option<Vec<EntityRef>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_errors: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    fn new(code: &'static str, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code,
            message: message.into(),
            retryable,
            field_errors: None,
            details: None,
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION", message, false)
    }
}

impl From<String> for ApiError {
    fn from(error: String) -> Self {
        const CODES: &[&str] = &[
            "VALIDATION",
            "NOT_FOUND",
            "ARCHIVED",
            "CONFLICT",
            "HIERARCHY_CYCLE",
            "CROSS_PROJECT",
            "CYCLE_SCOPE",
            "IDENTIFIER_CONFLICT",
            "TIMER_RUNNING",
            "CONFIRMATION_REQUIRED",
            "INVALID_CONTENT",
            "MEDIA_FAILURE",
            "STORAGE_FULL",
            "DATABASE_BUSY",
            "MIGRATION_FAILED",
            "UNSUPPORTED_VERSION",
            "IMPORT_INVALID",
            "IDEMPOTENCY_CONFLICT",
        ];
        let (explicit_code, message) = error
            .split_once(": ")
            .filter(|(prefix, _)| CODES.contains(prefix))
            .map_or((None, error.as_str()), |(code, message)| {
                (Some(code), message)
            });
        let code = explicit_code.unwrap_or_else(|| {
            let lower = message.to_ascii_lowercase();
            if lower.contains("not found") || lower.contains("no longer exists") {
                "NOT_FOUND"
            } else if lower.contains("database is locked") || lower.contains("database is busy") {
                "DATABASE_BUSY"
            } else if lower.contains("database or disk is full") {
                "STORAGE_FULL"
            } else if [
                "Choose",
                "Enter",
                "Keep",
                "Select",
                "Invalid",
                "Unknown",
                "Too many",
                "Issue query contains",
                "Write",
            ]
            .iter()
            .any(|prefix| message.starts_with(prefix))
            {
                "VALIDATION"
            } else {
                "INTERNAL"
            }
        });
        let message = if message.len() > 240 || message.contains(['/', '\\', '\n', '\r']) {
            "The operation could not be completed."
        } else {
            message
        };
        let code = CODES
            .iter()
            .copied()
            .find(|item| *item == code)
            .unwrap_or("INTERNAL");
        Self::new(
            code,
            message,
            matches!(code, "CONFLICT" | "DATABASE_BUSY" | "INTERNAL"),
        )
    }
}

pub fn command_error(
    app: &AppHandle,
    error: String,
    expected_versions: Option<&BTreeMap<String, i64>>,
) -> ApiError {
    enrich_command_error(app, ApiError::from(error), expected_versions)
}

pub fn enrich_command_error(
    app: &AppHandle,
    mut error: ApiError,
    expected_versions: Option<&BTreeMap<String, i64>>,
) -> ApiError {
    if error.code != "CONFLICT" {
        return error;
    }

    let records = document_data_root(app)
        .ok()
        .and_then(|root| open_database(&root).ok())
        .map(|connection| {
            expected_versions
                .into_iter()
                .flat_map(|versions| versions.keys())
                .filter_map(|id| current_record(&connection, id))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    error.details = Some(serde_json::json!({ "currentRecords": records }));
    error
}

fn current_record(connection: &rusqlite::Connection, id: &str) -> Option<serde_json::Value> {
    for (table, kind) in [
        ("issue", "issue"),
        ("project", "project"),
        ("saved_view", "savedView"),
        ("label", "label"),
        ("time_entry", "timeEntry"),
        ("cycle", "cycle"),
        ("cycle_series", "cycleSeries"),
        ("planning_node", "planningNode"),
        ("milestone", "milestone"),
        ("issue_activity", "comment"),
        ("issue_relation", "issueRelation"),
        ("media", "media"),
        ("workspace", "workspace"),
    ] {
        let Ok(mut statement) = connection.prepare(&format!("SELECT * FROM {table} WHERE id = ?1"))
        else {
            continue;
        };
        let names = statement
            .column_names()
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let record = statement
            .query_row([id], |row| {
                let mut fields = serde_json::Map::new();
                for (index, name) in names.iter().enumerate() {
                    let value = match row.get_ref(index)? {
                        rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                        rusqlite::types::ValueRef::Integer(value) => serde_json::json!(value),
                        rusqlite::types::ValueRef::Real(value) => serde_json::json!(value),
                        rusqlite::types::ValueRef::Text(value) => {
                            serde_json::Value::String(String::from_utf8_lossy(value).into_owned())
                        }
                        rusqlite::types::ValueRef::Blob(_) => serde_json::Value::Null,
                    };
                    fields.insert(name.clone(), value);
                }
                Ok(serde_json::Value::Object(fields))
            })
            .optional()
            .ok()
            .flatten();
        if let Some(record) = record {
            return Some(serde_json::json!({ "kind": kind, "record": record }));
        }
    }
    None
}

pub fn respond<T: Serialize>(
    app: &AppHandle,
    data: T,
    mutation_id: Option<&str>,
    expected_versions: Option<&BTreeMap<String, i64>>,
    command: &str,
    input: Option<&serde_json::Value>,
) -> Result<ApiResponse<T>, ApiError> {
    let root = document_data_root(app).map_err(ApiError::from)?;
    let connection = open_database(&root).map_err(ApiError::from)?;
    let workspace_id = current_workspace_id(&connection).ok();
    let current_revision = match workspace_id.as_deref() {
        Some(workspace_id) => connection
            .query_row(
                "SELECT revision FROM workspace WHERE id = ?1",
                [workspace_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| ApiError::new("INTERNAL", "Cannot read workspace revision.", true))?
            .unwrap_or(0),
        None => 0,
    };
    let workspace_revision = mutation_id
        .map(|id| {
            connection
                .query_row(
                    "SELECT workspace_revision FROM mutation_receipt WHERE mutation_id = ?1",
                    [id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|_| ApiError::new("INTERNAL", "Cannot read mutation receipt.", true))
        })
        .transpose()?
        .flatten()
        .unwrap_or(current_revision);
    // Replayed mutations need the effects captured at commit time, not effects from this request.
    let stored_effects = mutation_id
        .map(|id| {
            connection
                .query_row(
                    "SELECT effects_json FROM mutation_receipt WHERE mutation_id = ?1",
                    [id],
                    |row| row.get::<_, Option<String>>(0),
                )
                .optional()
                .map_err(|_| ApiError::new("INTERNAL", "Cannot read mutation effects.", true))
        })
        .transpose()?
        .flatten()
        .flatten()
        .map(|json| serde_json::from_str::<MutationEffects>(&json))
        .transpose()
        .map_err(|_| ApiError::new("INTERNAL", "Cannot decode mutation effects.", true))?;
    let changed = stored_effects
        .as_ref()
        .map(|effects| effects.changed.clone())
        .or_else(|| {
            mutation_id.map(|_| {
                let kind = entity_kind(command);
                let value = serde_json::to_value(&data).unwrap_or(serde_json::Value::Null);
                let candidates = match &value {
                    serde_json::Value::Array(items) => items.iter().collect::<Vec<_>>(),
                    serde_json::Value::Object(_) => vec![&value],
                    _ => Vec::new(),
                };
                let mut refs = candidates
                    .iter()
                    .filter_map(|item| {
                        item.get("entityId")
                            .or_else(|| item.get("id"))
                            .or_else(|| item.get("workspaceId"))
                    })
                    .filter_map(serde_json::Value::as_str)
                    .filter_map(|id| {
                        resolve_entity_id(&connection, workspace_id.as_deref(), id).ok()
                    })
                    .map(|id| EntityRef {
                        id,
                        kind: kind.to_owned(),
                    })
                    .collect::<Vec<_>>();
                let has_output_ref = !refs.is_empty();
                let version_kind = if command.contains("issue_relation") {
                    "issue"
                } else if command.contains("planning_node")
                    || command == "storage_restore_milestone"
                {
                    "project"
                } else if command.contains("timer")
                    || command.contains("workflow")
                    || command.contains("preferences")
                {
                    "workspace"
                } else {
                    kind
                };
                refs.extend(expected_versions.into_iter().flat_map(|versions| {
                    versions.keys().map(|id| EntityRef {
                        id: id.clone(),
                        kind: version_kind.to_owned(),
                    })
                }));
                if let Some(input) = input.filter(|_| !has_output_ref) {
                    for field in [
                        "relationId",
                        "commentId",
                        "milestoneId",
                        "nodeId",
                        "id",
                        "entryId",
                        "cycleId",
                        "seriesId",
                        "identifier",
                        "projectId",
                    ] {
                        if command.contains("timer") && field == "identifier" {
                            continue;
                        }
                        if let Some(id) = input.get(field).and_then(serde_json::Value::as_str) {
                            if let Ok(id) =
                                resolve_entity_id(&connection, workspace_id.as_deref(), id)
                            {
                                refs.push(EntityRef {
                                    id,
                                    kind: kind.to_owned(),
                                });
                                break;
                            }
                        }
                    }
                }
                refs.sort_by(|a, b| (&a.kind, &a.id).cmp(&(&b.kind, &b.id)));
                refs.dedup_by(|a, b| a.kind == b.kind && a.id == b.id);
                refs
            })
        });
    let removes_active = !command.contains("restore")
        && command != "storage_delete_archived_issue"
        && (command.contains("archive")
            || command.contains("trash")
            || command.contains("delete")
            || command.contains("dissolve")
            || (command == "storage_set_project_lifecycle"
                && input
                    .and_then(|value| value.get("action"))
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|action| action != "restore")));
    let removed_from_active = stored_effects
        .map(|effects| effects.removed_from_active)
        .or_else(|| {
            mutation_id.map(|_| {
                if removes_active {
                    changed
                        .as_ref()
                        .into_iter()
                        .flatten()
                        .filter(|item| item.kind == entity_kind(command))
                        .cloned()
                        .collect()
                } else {
                    Vec::new()
                }
            })
        });
    Ok(ApiResponse {
        data,
        workspace_revision,
        changed,
        removed_from_active,
    })
}

fn entity_kind(command: &str) -> &'static str {
    if command.contains("issue_relation") {
        "issueRelation"
    } else if command.contains("issue_comment") {
        "comment"
    } else if command.contains("issue") || command.contains("triage") {
        "issue"
    } else if command.contains("milestone") {
        "milestone"
    } else if command.contains("project_update") {
        "projectUpdate"
    } else if command.contains("project") {
        "project"
    } else if command.contains("planning_node") {
        "planningNode"
    } else if command.contains("label") {
        "label"
    } else if command.contains("view") {
        "savedView"
    } else if command.contains("cycle_series")
        || command.contains("cycle_schedule")
        || command == "storage_configure_cycles"
    {
        "cycleSeries"
    } else if command.contains("cycle") {
        "cycle"
    } else if command.contains("time") || command.contains("timer") {
        "timeEntry"
    } else if command.contains("workspace") || command.contains("identity") {
        "workspace"
    } else {
        "workspace"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_and_mutation_envelopes_validate() {
        let query: ApiRequest<serde_json::Value> = serde_json::from_value(serde_json::json!({
            "contractVersion": 1,
            "requestId": Uuid::new_v4().to_string(),
            "input": { "text": "alpha" }
        }))
        .unwrap();
        assert!(query.validate(false).is_ok());
        assert!(query.validate(true).is_err());

        let query_with_mutation_id: ApiRequest<serde_json::Value> =
            serde_json::from_value(serde_json::json!({
                "contractVersion": 1,
                "mutationId": Uuid::new_v4().to_string(),
                "input": {}
            }))
            .unwrap();
        assert_eq!(
            query_with_mutation_id.validate(false).unwrap_err().code,
            "VALIDATION"
        );

        let id = Uuid::new_v4().to_string();
        let mutation: ApiRequest<serde_json::Value> = serde_json::from_value(serde_json::json!({
            "contractVersion": 1,
            "mutationId": id,
            "expectedVersions": { Uuid::new_v4().to_string(): 3 },
            "input": { "mutation": { "mutationId": id, "expectedVersion": 3 } }
        }))
        .unwrap();
        assert!(mutation.validate(true).is_ok());

        let invalid: ApiRequest<serde_json::Value> = serde_json::from_value(serde_json::json!({
            "contractVersion": 1,
            "mutationId": Uuid::new_v4().to_string(),
            "expectedVersions": { "APP-123": 3 },
            "input": {}
        }))
        .unwrap();
        assert_eq!(invalid.validate(true).unwrap_err().code, "VALIDATION");
    }

    #[test]
    fn native_errors_have_stable_fields_and_hide_paths() {
        let conflict = ApiError::from("CONFLICT: Entity changed.".to_owned());
        assert_eq!(conflict.code, "CONFLICT");
        assert!(conflict.retryable);
        let serialized = serde_json::to_value(&conflict).unwrap();
        assert_eq!(serialized["code"], "CONFLICT");
        assert_eq!(serialized["retryable"], true);
        let hidden = ApiError::from("Cannot open /private/workspace.sqlite".to_owned());
        assert_eq!(hidden.code, "INTERNAL");
        assert!(!hidden.message.contains("/private"));
        assert_eq!(
            ApiError::from("Issue not found.".to_owned()).code,
            "NOT_FOUND"
        );
    }

    #[test]
    fn response_envelopes_use_the_documented_shapes() {
        let query = ApiResponse {
            data: serde_json::json!({ "id": "one" }),
            workspace_revision: 4,
            changed: None,
            removed_from_active: None,
        };
        let query = serde_json::to_value(query).unwrap();
        assert_eq!(query["workspaceRevision"], 4);
        assert!(query.get("changed").is_none());

        let mutation = ApiResponse {
            data: serde_json::json!({ "id": "one" }),
            workspace_revision: 5,
            changed: Some(vec![EntityRef {
                id: Uuid::new_v4().to_string(),
                kind: "issue".to_owned(),
            }]),
            removed_from_active: Some(Vec::new()),
        };
        let mutation = serde_json::to_value(mutation).unwrap();
        assert_eq!(mutation["changed"][0]["kind"], "issue");
        assert_eq!(mutation["removedFromActive"], serde_json::json!([]));
    }

    #[test]
    fn mutation_receipt_keeps_all_cascaded_row_effects() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE workspace (id TEXT PRIMARY KEY, revision INTEGER NOT NULL);
                 CREATE TABLE issue (id TEXT PRIMARY KEY, archived_at TEXT);
                 CREATE TABLE mutation_receipt (
                   mutation_id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL,
                   operation TEXT NOT NULL, request_hash TEXT NOT NULL,
                   result_json TEXT NOT NULL, workspace_revision INTEGER NOT NULL,
                   effects_json TEXT, created_at TEXT DEFAULT CURRENT_TIMESTAMP
                 );",
            )
            .unwrap();
        install_change_capture(&connection).unwrap();
        let workspace_id = Uuid::new_v4().to_string();
        let first = Uuid::new_v4().to_string();
        let child = Uuid::new_v4().to_string();
        connection
            .execute(
                "INSERT INTO workspace (id, revision) VALUES (?1, 2)",
                [&workspace_id],
            )
            .unwrap();
        connection
            .execute("INSERT INTO issue (id) VALUES (?1)", [&first])
            .unwrap();
        connection
            .execute("INSERT INTO issue (id) VALUES (?1)", [&child])
            .unwrap();

        let _capture = begin_change_capture();
        connection
            .execute("UPDATE issue SET archived_at = 'today'", [])
            .unwrap();
        let request = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: None,
            expected_versions: BTreeMap::new(),
        };
        mutations::save(
            &connection,
            &workspace_id,
            "issue.archive",
            "hash",
            &request,
            &(),
        )
        .unwrap();
        let json: String = connection
            .query_row(
                "SELECT effects_json FROM mutation_receipt WHERE mutation_id = ?1",
                [&request.mutation_id],
                |row| row.get(0),
            )
            .unwrap();
        let effects: MutationEffects = serde_json::from_str(&json).unwrap();
        assert_eq!(effects.changed.len(), 2);
        assert_eq!(effects.removed_from_active.len(), 2);
        assert!(effects.changed.iter().any(|item| item.id == first));
        assert!(effects.changed.iter().any(|item| item.id == child));
    }

    #[test]
    fn issue_identifiers_resolve_to_versioned_entity_uuids() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection
            .execute_batch("CREATE TABLE issue (id TEXT, workspace_id TEXT, identifier TEXT);")
            .unwrap();
        let workspace_id = Uuid::new_v4().to_string();
        let issue_id = Uuid::new_v4().to_string();
        connection
            .execute(
                "INSERT INTO issue (id, workspace_id, identifier) VALUES (?1, ?2, ?3)",
                rusqlite::params![issue_id, workspace_id, "APP-123"],
            )
            .unwrap();

        assert_eq!(
            resolve_entity_id(&connection, Some(&workspace_id), "APP-123").unwrap(),
            issue_id
        );
        let current = current_record(&connection, &issue_id).unwrap();
        assert_eq!(current["kind"], "issue");
        assert_eq!(current["record"]["identifier"], "APP-123");
        assert_eq!(
            resolve_entity_id(&connection, Some(&workspace_id), "APP-404")
                .unwrap_err()
                .code,
            "NOT_FOUND"
        );
    }
}
