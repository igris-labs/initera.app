use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use uuid::Uuid;

use crate::mutations::{self, MutationRequest};
use crate::storage::{current_workspace_id, document_data_root, open_database};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SavedView {
    id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    version: Option<i64>,
    name: String,
    description: String,
    layout: String,
    group_by: String,
    scope: String,
    status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    statuses: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_statuses: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    projects: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_projects: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project_statuses: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_project_statuses: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project_priorities: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_project_priorities: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project_labels: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_project_labels: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    empty_project_labels: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_empty_project_labels: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project_leads: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_project_leads: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    empty_project_lead: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_empty_project_lead: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project_milestones: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_project_milestones: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    empty_project_milestones: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_empty_project_milestones: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    milestones: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_milestones: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    empty_milestone: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_empty_milestone: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    priorities: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_priorities: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_labels: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    empty_labels: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excluded_empty_labels: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    due: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    advanced_groups: Option<Vec<crate::search::IssueFilterGroup>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    filter_tree: Option<crate::search::FilterTree>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    columns: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sort_field: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sort_direction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    density: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    show_empty_groups: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    include_sub_issues: Option<bool>,
    favorite: bool,
}

fn valid_text_values(values: Option<&Vec<String>>, max_length: usize) -> bool {
    values.is_none_or(|items| {
        items.len() <= 20
            && items
                .iter()
                .all(|item| !item.trim().is_empty() && item.chars().count() <= max_length)
    })
}

fn valid_priorities(values: Option<&Vec<String>>) -> bool {
    values.is_none_or(|items| {
        items.len() <= 20
            && items.iter().all(|item| {
                matches!(
                    item.as_str(),
                    "No priority" | "Low" | "Medium" | "High" | "Urgent"
                )
            })
    })
}

fn valid_project_statuses(values: Option<&Vec<String>>) -> bool {
    values.is_none_or(|items| {
        items.len() <= 20
            && items.iter().all(|item| {
                matches!(
                    item.as_str(),
                    "Backlog" | "Planned" | "In Progress" | "Completed" | "Canceled"
                )
            })
    })
}

fn valid_uuid_ids(values: Option<&Vec<String>>) -> bool {
    values.is_none_or(|items| {
        items.len() <= 20 && items.iter().all(|item| Uuid::parse_str(item).is_ok())
    })
}

fn validate_view(view: &SavedView) -> Result<(), String> {
    // These limits mirror the issue query validator. Saved definitions are persisted input,
    // so validate before the query builder ever sees them.
    if Uuid::parse_str(&view.id).is_err() {
        return Err("Saved view ID is invalid.".into());
    }
    if view.name.trim().is_empty() || view.name.chars().count() > 80 {
        return Err("View names must contain between 1 and 80 characters.".into());
    }
    if view.description.chars().count() > 500 {
        return Err("View descriptions must be 500 characters or fewer.".into());
    }
    if !matches!(view.layout.as_str(), "list" | "board")
        || !matches!(
            view.group_by.as_str(),
            "none" | "status" | "priority" | "project"
        )
        || !matches!(view.scope.as_str(), "active" | "all")
    {
        return Err("Saved view layout is invalid.".into());
    }
    if view.status.trim().is_empty() || view.status.chars().count() > 32 {
        return Err("Saved view status is invalid.".into());
    }
    if view
        .project
        .as_ref()
        .is_some_and(|value| value.chars().count() > 80)
        || !valid_text_values(view.statuses.as_ref(), 32)
        || !valid_text_values(view.excluded_statuses.as_ref(), 32)
        || !valid_text_values(view.projects.as_ref(), 80)
        || !valid_text_values(view.excluded_projects.as_ref(), 80)
        || !valid_project_statuses(view.project_statuses.as_ref())
        || !valid_project_statuses(view.excluded_project_statuses.as_ref())
        || !valid_priorities(view.project_priorities.as_ref())
        || !valid_priorities(view.excluded_project_priorities.as_ref())
        || !valid_text_values(view.project_labels.as_ref(), 80)
        || !valid_text_values(view.excluded_project_labels.as_ref(), 80)
        || !valid_uuid_ids(view.project_leads.as_ref())
        || !valid_uuid_ids(view.excluded_project_leads.as_ref())
        || !valid_text_values(view.project_milestones.as_ref(), 240)
        || !valid_text_values(view.excluded_project_milestones.as_ref(), 240)
        || !valid_uuid_ids(view.milestones.as_ref())
        || !valid_uuid_ids(view.excluded_milestones.as_ref())
        || !valid_text_values(view.labels.as_ref(), 80)
        || !valid_text_values(view.excluded_labels.as_ref(), 80)
        || !valid_priorities(view.priorities.as_ref())
        || !valid_priorities(view.excluded_priorities.as_ref())
        || view
            .label
            .as_ref()
            .is_some_and(|value| value.chars().count() > 80)
        || view.priority.as_ref().is_some_and(|value| {
            !matches!(
                value.as_str(),
                "No priority" | "Low" | "Medium" | "High" | "Urgent"
            )
        })
        || view
            .due
            .as_ref()
            .is_some_and(|value| !matches!(value.as_str(), "any" | "has" | "none"))
        || view
            .advanced_groups
            .as_ref()
            .is_some_and(|groups| !crate::search::valid_advanced_groups(groups))
        || view
            .filter_tree
            .as_ref()
            .is_some_and(|tree| crate::search::filter_tree_condition(tree).is_err())
        || view.columns.as_ref().is_some_and(|columns| {
            columns.len() > 13
                || columns.iter().any(|column| {
                    !matches!(
                        column.as_str(),
                        "identifier"
                            | "title"
                            | "status"
                            | "priority"
                            | "project"
                            | "node"
                            | "milestone"
                            | "cycle"
                            | "labels"
                            | "estimate"
                            | "dueDate"
                            | "updatedAt"
                            | "trackedTime"
                    )
                })
        })
        || view.sort_field.as_ref().is_some_and(|field| {
            !matches!(
                field.as_str(),
                "position" | "title" | "status" | "priority" | "dueDate" | "updatedAt"
            )
        })
        || view
            .sort_direction
            .as_ref()
            .is_some_and(|direction| !matches!(direction.as_str(), "asc" | "desc"))
        || view
            .density
            .as_ref()
            .is_some_and(|density| !matches!(density.as_str(), "compact" | "default"))
    {
        return Err("Saved view filter is invalid.".into());
    }
    Ok(())
}

fn list_views(connection: &Connection) -> Result<Vec<SavedView>, String> {
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT definition_json FROM saved_view
             WHERE workspace_id = ?1 ORDER BY position, id",
        )
        .map_err(|_| "Cannot list saved views.".to_owned())?;
    let definitions = statement
        .query_map([workspace_id], |row| row.get::<_, String>(0))
        .map_err(|_| "Cannot list saved views.".to_owned())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Cannot list saved views.".to_owned())?;
    definitions
        .iter()
        .map(|definition| {
            serde_json::from_str(definition).map_err(|_| {
                "A saved view could not be read. Restore a database backup.".to_owned()
            })
        })
        .collect()
}

fn list_views_versioned(connection: &Connection) -> Result<Vec<SavedView>, String> {
    let mut views = list_views(connection)?;
    for view in &mut views {
        view.version = Some(view_version(connection, &view.id)?);
    }
    Ok(views)
}

fn view_version(connection: &Connection, id: &str) -> Result<i64, String> {
    connection
        .query_row(
            "SELECT version FROM saved_view WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read saved view version.".to_owned())
}

fn bump_revision(connection: &Connection, workspace_id: &str) -> Result<i64, String> {
    connection
        .query_row(
            "UPDATE workspace SET revision = revision + 1 WHERE id = ?1 RETURNING revision",
            [workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot update saved view revision.".to_owned())
}

fn create_view(connection: &Connection, view: SavedView) -> Result<SavedView, String> {
    validate_view(&view)?;
    let workspace_id = current_workspace_id(connection)?;
    let definition =
        serde_json::to_string(&view).map_err(|_| "Cannot encode the saved view.".to_owned())?;
    connection
        .execute(
            "INSERT INTO saved_view (id, workspace_id, definition_json, position)
             SELECT ?1, ?2, ?3, COALESCE(MAX(position), -1) + 1
             FROM saved_view WHERE workspace_id = ?2",
            params![view.id, workspace_id, definition],
        )
        .map_err(|_| "Cannot create the saved view.".to_owned())?;
    Ok(view)
}

fn update_view(connection: &Connection, view: SavedView) -> Result<SavedView, String> {
    validate_view(&view)?;
    let workspace_id = current_workspace_id(connection)?;
    let definition =
        serde_json::to_string(&view).map_err(|_| "Cannot encode the saved view.".to_owned())?;
    let changed = connection
        .execute(
            "UPDATE saved_view SET definition_json = ?1 WHERE id = ?2 AND workspace_id = ?3",
            params![definition, view.id, workspace_id],
        )
        .map_err(|_| "Cannot update the saved view.".to_owned())?;
    if changed == 0 {
        return Err("Saved view no longer exists in this workspace.".into());
    }
    Ok(view)
}

fn delete_view(connection: &Connection, id: &str) -> Result<(), String> {
    let workspace_id = current_workspace_id(connection)?;
    let changed = connection
        .execute(
            "DELETE FROM saved_view WHERE id = ?1 AND workspace_id = ?2",
            params![id, workspace_id],
        )
        .map_err(|_| "Cannot delete the saved view.".to_owned())?;
    if changed == 0 {
        return Err("Saved view no longer exists in this workspace.".into());
    }
    Ok(())
}

#[api_contract_macros::command]
pub fn storage_list_views(app: AppHandle) -> Result<Vec<SavedView>, String> {
    list_views_versioned(&open_database(&document_data_root(&app)?)?)
}

#[api_contract_macros::command]
pub fn storage_create_view(
    app: AppHandle,
    view: SavedView,
    mutation: MutationRequest,
) -> Result<SavedView, String> {
    create_view_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        view,
        &mutation,
    )
}

#[api_contract_macros::command]
pub fn storage_update_view(
    app: AppHandle,
    view: SavedView,
    mutation: MutationRequest,
) -> Result<SavedView, String> {
    update_view_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        view,
        &mutation,
    )
}

#[api_contract_macros::command]
pub fn storage_delete_view(
    app: AppHandle,
    id: String,
    mutation: MutationRequest,
) -> Result<(), String> {
    delete_view_with_mutation(
        &mut open_database(&document_data_root(&app)?)?,
        &id,
        &mutation,
    )
}

fn create_view_with_mutation(
    connection: &mut Connection,
    mut view: SavedView,
    mutation: &MutationRequest,
) -> Result<SavedView, String> {
    view.version = None;
    let payload = view.clone();
    mutations::apply(
        connection,
        "view.create",
        &payload,
        mutation,
        |tx, workspace_id| {
            let mut saved = create_view(tx, view)?;
            saved.version = Some(view_version(tx, &saved.id)?);
            bump_revision(tx, workspace_id)?;
            Ok(saved)
        },
    )
}

fn update_view_with_mutation(
    connection: &mut Connection,
    mut view: SavedView,
    mutation: &MutationRequest,
) -> Result<SavedView, String> {
    view.version = None;
    let payload = view.clone();
    mutations::apply(
        connection,
        "view.update",
        &payload,
        mutation,
        |tx, workspace_id| {
            mutations::check_version(tx, "saved_view", &view.id, mutation)?;
            let mut saved = update_view(tx, view)?;
            saved.version = Some(view_version(tx, &saved.id)?);
            bump_revision(tx, workspace_id)?;
            Ok(saved)
        },
    )
}

fn delete_view_with_mutation(
    connection: &mut Connection,
    id: &str,
    mutation: &MutationRequest,
) -> Result<(), String> {
    mutations::apply(
        connection,
        "view.delete",
        &id,
        mutation,
        |tx, workspace_id| {
            mutations::check_version(tx, "saved_view", id, mutation)?;
            delete_view(tx, id)?;
            bump_revision(tx, workspace_id)?;
            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{create_workspace, save_identity, switch_workspace};
    use std::fs;

    fn sample_view(name: &str) -> SavedView {
        SavedView {
            id: Uuid::new_v4().to_string(),
            version: None,
            name: name.into(),
            description: String::new(),
            layout: "list".into(),
            group_by: "status".into(),
            scope: "active".into(),
            status: "All statuses".into(),
            statuses: None,
            excluded_statuses: None,
            project: None,
            projects: None,
            excluded_projects: None,
            project_statuses: None,
            excluded_project_statuses: None,
            project_priorities: None,
            excluded_project_priorities: None,
            project_labels: None,
            excluded_project_labels: None,
            empty_project_labels: None,
            excluded_empty_project_labels: None,
            project_leads: None,
            excluded_project_leads: None,
            empty_project_lead: None,
            excluded_empty_project_lead: None,
            project_milestones: None,
            excluded_project_milestones: None,
            empty_project_milestones: None,
            excluded_empty_project_milestones: None,
            milestones: None,
            excluded_milestones: None,
            empty_milestone: None,
            excluded_empty_milestone: None,
            priority: None,
            priorities: None,
            excluded_priorities: None,
            label: None,
            labels: None,
            excluded_labels: None,
            empty_labels: None,
            excluded_empty_labels: None,
            due: None,
            advanced_groups: None,
            filter_tree: None,
            columns: None,
            sort_field: None,
            sort_direction: None,
            density: None,
            show_empty_groups: None,
            include_sub_issues: None,
            favorite: false,
        }
    }

    #[test]
    fn views_survive_restart_and_remain_in_their_workspace() {
        let root = std::env::temp_dir().join(format!("pm-app-views-test-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).expect("database opens");
        save_identity(&mut connection, "Rahul", "Igris Labs").expect("setup");
        let original_id = current_workspace_id(&connection).expect("current workspace");
        let view = create_view(&connection, sample_view("Review queue")).expect("create view");

        let mut changed = view.clone();
        changed.layout = "board".into();
        changed.favorite = true;
        changed.statuses = Some(vec!["Todo".into(), "In Progress".into()]);
        changed.priorities = Some(vec!["High".into(), "Urgent".into()]);
        changed.labels = Some(vec!["UI".into(), "Workflow".into()]);
        changed.excluded_labels = Some(vec!["Blocked".into()]);
        changed.project_statuses = Some(vec!["Planned".into()]);
        changed.excluded_project_priorities = Some(vec!["Low".into()]);
        changed.project_labels = Some(vec!["Workflow".into()]);
        changed.empty_project_labels = Some(true);
        changed.project_leads = Some(vec![Uuid::new_v4().to_string()]);
        changed.empty_project_lead = Some(true);
        changed.project_milestones = Some(vec!["Beta".into()]);
        changed.excluded_empty_project_milestones = Some(true);
        changed.milestones = Some(vec![Uuid::new_v4().to_string()]);
        changed.empty_milestone = Some(true);
        changed.empty_labels = Some(true);
        changed.advanced_groups = Some(serde_json::from_value(serde_json::json!([{
            "mode": "any",
            "rules": [
                {"field": "due", "operator": "between", "value": "2026-10-01", "end": "2026-10-31"},
                {"field": "blocks", "operator": "has"}
            ]
        }])).unwrap());
        update_view(&connection, changed.clone()).expect("update view");
        drop(connection);

        let mut connection = open_database(&root).expect("database reopens");
        assert_eq!(
            list_views(&connection).expect("list views"),
            vec![changed.clone()]
        );
        create_workspace(&mut connection, "Other workspace").expect("second workspace");
        assert!(list_views(&connection)
            .expect("list other workspace")
            .is_empty());
        assert!(update_view(&connection, changed.clone()).is_err());
        assert!(delete_view(&connection, &view.id).is_err());

        switch_workspace(&connection, &original_id).expect("switch back");
        delete_view(&connection, &view.id).expect("delete view");
        assert!(list_views(&connection)
            .expect("list deleted views")
            .is_empty());
        drop(connection);
        fs::remove_dir_all(root).expect("remove test data");
    }

    #[test]
    fn invalid_view_is_rejected_before_writing() {
        let root = std::env::temp_dir().join(format!("pm-app-views-test-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).expect("database opens");
        save_identity(&mut connection, "Rahul", "Igris Labs").expect("setup");
        let mut view = sample_view(" ");
        assert!(create_view(&connection, view.clone()).is_err());
        view.name = "Valid".into();
        view.layout = "invalid".into();
        assert!(create_view(&connection, view).is_err());
        let mut invalid_status = sample_view("Invalid project status");
        invalid_status.project_statuses = Some(vec!["Unknown".into()]);
        assert!(create_view(&connection, invalid_status).is_err());
        let mut invalid_label = sample_view("Invalid project label");
        invalid_label.project_labels = Some(vec![" ".into()]);
        assert!(create_view(&connection, invalid_label).is_err());
        let mut invalid_lead = sample_view("Invalid project lead");
        invalid_lead.project_leads = Some(vec!["not-an-actor-id".into()]);
        assert!(create_view(&connection, invalid_lead).is_err());
        let mut invalid_milestone = sample_view("Invalid project milestone");
        invalid_milestone.project_milestones = Some(vec![" ".into()]);
        assert!(create_view(&connection, invalid_milestone).is_err());
        let mut invalid_assigned_milestone = sample_view("Invalid assigned milestone");
        invalid_assigned_milestone.milestones = Some(vec!["not-a-milestone-id".into()]);
        assert!(create_view(&connection, invalid_assigned_milestone).is_err());
        assert!(list_views(&connection).expect("list views").is_empty());
        drop(connection);
        fs::remove_dir_all(root).expect("remove test data");
    }

    #[test]
    fn saved_view_mutations_replay_and_reject_stale_versions() {
        let root = std::env::temp_dir().join(format!("pm-app-view-receipt-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();

        let draft = sample_view("Review queue");
        let create = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: None,
            expected_versions: Default::default(),
        };
        let saved = create_view_with_mutation(&mut connection, draft.clone(), &create).unwrap();
        assert_eq!(saved.version, Some(1));
        assert_eq!(
            create_view_with_mutation(&mut connection, draft, &create).unwrap(),
            saved
        );

        let mut changed = saved.clone();
        changed.name = "Current queue".into();
        let update = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: saved.version,
            expected_versions: Default::default(),
        };
        let updated = update_view_with_mutation(&mut connection, changed.clone(), &update).unwrap();
        assert_eq!(updated.version, Some(2));
        assert_eq!(
            update_view_with_mutation(&mut connection, changed, &update).unwrap(),
            updated
        );
        assert!(update_view_with_mutation(
            &mut connection,
            saved.clone(),
            &MutationRequest {
                mutation_id: Uuid::new_v4().to_string(),
                expected_version: saved.version,
                expected_versions: Default::default(),
            }
        )
        .unwrap_err()
        .contains("CONFLICT"));

        let remove = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: updated.version,
            expected_versions: Default::default(),
        };
        delete_view_with_mutation(&mut connection, &saved.id, &remove).unwrap();
        delete_view_with_mutation(&mut connection, &saved.id, &remove).unwrap();
        assert!(list_views(&connection).unwrap().is_empty());

        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }
}
