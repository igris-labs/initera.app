use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::mutations::{self, MutationRequest};
use crate::storage::{current_workspace_id, document_data_root, open_database};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceOptions {
    project_move_identifier: String,
}

#[api_contract_macros::command]
pub fn storage_get_workspace_options(app: AppHandle) -> Result<WorkspaceOptions, String> {
    let connection = open_database(&document_data_root(&app)?)?;
    let workspace_id = current_workspace_id(&connection)?;
    let preference: Option<String> = connection
        .query_row(
            "SELECT option_value FROM workspace_option
             WHERE workspace_id = ?1 AND option_key = 'project_move_identifier'",
            [workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot read workspace preference.".to_owned())?;
    Ok(WorkspaceOptions {
        project_move_identifier: preference.unwrap_or_else(|| "ask".into()),
    })
}

#[api_contract_macros::command]
pub fn storage_set_project_move_preference(
    app: AppHandle,
    preference: String,
    mutation: MutationRequest,
) -> Result<WorkspaceOptions, String> {
    if !matches!(preference.as_str(), "ask" | "keep" | "regenerate") {
        return Err("Choose an identifier preference.".into());
    }
    let mut connection = open_database(&document_data_root(&app)?)?;
    mutations::apply(
        &mut connection,
        "workspace.project_move_preference",
        &preference,
        &mutation,
        |tx, workspace_id| {
            tx.execute(
                "INSERT INTO workspace_option(workspace_id, option_key, option_value)
                 VALUES (?1, 'project_move_identifier', ?2)
                 ON CONFLICT(workspace_id, option_key) DO UPDATE SET option_value = excluded.option_value",
                params![workspace_id, preference],
            )
            .map_err(|_| "Cannot save workspace preference.".to_owned())?;
            Ok(WorkspaceOptions {
                project_move_identifier: preference.clone(),
            })
        },
    )
}
