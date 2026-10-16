use super::*;
use crate::storage::{create_workspace, open_database, save_identity, switch_workspace};
use rusqlite::params;
use std::fs;
use uuid::Uuid;

#[test]
fn search_tracks_issue_changes_and_workspace_scope() {
    let root = std::env::temp_dir().join(format!("pm-app-search-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).expect("database opens with FTS5");
    save_identity(&mut connection, "Rahul", "Igris Labs").expect("setup");
    let workspace_id = current_workspace_id(&connection).expect("current workspace");
    let actor_id: String = connection
        .query_row("SELECT id FROM local_actor LIMIT 1", [], |row| row.get(0))
        .expect("local actor");
    let project_id = Uuid::new_v4().to_string();
    let issue_id = Uuid::new_v4().to_string();

    connection
        .execute(
            "INSERT INTO project(id, workspace_id, name, identifier_prefix)
                 VALUES (?1, ?2, 'Desktop app', 'APP')",
            params![project_id, workspace_id],
        )
        .expect("create project");
    let node_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO planning_node(id, project_id, kind, name, description, position)
             VALUES (?1, ?2, 'module', 'Design tokens', 'Typography audit', 0)",
            params![node_id, project_id],
        )
        .expect("create planning node");
    assert_eq!(
        search_planning_nodes(&connection, "typography").unwrap()[0].id,
        node_id
    );
    connection
        .execute(
            "UPDATE planning_node SET description = 'Color audit' WHERE id = ?1",
            [&node_id],
        )
        .unwrap();
    assert!(search_planning_nodes(&connection, "typography")
        .unwrap()
        .is_empty());
    assert_eq!(
        search_planning_nodes(&connection, "color").unwrap()[0].id,
        node_id
    );
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, project_id, title,
                                   description, status, priority, labels_json, position)
                 VALUES (?1, ?2, 'APP-1', ?3, 'Compact picker', 'Review keyboard flow',
                         'Todo', 'High', '[\"Workflow\"]', 0)",
            params![issue_id, workspace_id, project_id],
        )
        .expect("create issue");

    connection
            .execute(
                "UPDATE project SET summary = 'Migration planning',
                 resources_json = '[{\"id\":\"resource-1\",\"title\":\"Architecture specification\",\"url\":\"https://example.com\"}]'
                 WHERE id = ?1",
                [&project_id],
            )
            .expect("update searchable project fields");
    let comment_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO issue_activity (id, issue_id, actor_name, kind, body)
                 VALUES (?1, ?2, 'Rahul', 'comment', 'Commentonly release note')",
            params![comment_id, issue_id],
        )
        .expect("create searchable comment");
    let comments = search_comments(&connection, "Commentonly").unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].activity_id, comment_id);
    assert_eq!(comments[0].identifier, "APP-1");
    assert!(comments[0].body.contains("Commentonly"));
    assert_eq!(
        find_comment(&connection, "APP-1", &comment_id)
            .unwrap()
            .unwrap()
            .body,
        "Commentonly release note"
    );
    assert!(find_comment(&connection, "APP-2", &comment_id)
        .unwrap()
        .is_none());
    for term in ["Desktop", "migration", "specification"] {
        assert_eq!(
            search_projects(&connection, term).unwrap(),
            vec![project_id.clone()]
        );
    }
    assert_eq!(
        search_issues(&connection, "Commentonly").unwrap(),
        vec!["APP-1"]
    );

    for text in ["APP-1", "compact", "keyboard", "Workflow", "Desktop"] {
        assert_eq!(search_issues(&connection, text).unwrap(), vec!["APP-1"]);
    }
    assert!(search_issues(&connection, "not-present")
        .unwrap()
        .is_empty());
    assert!(search_issues(&connection, "!!!").unwrap().is_empty());

    connection
        .execute(
            "UPDATE issue SET title = 'Polished picker' WHERE id = ?1",
            [&issue_id],
        )
        .expect("rename issue");
    assert!(search_issues(&connection, "compact").unwrap().is_empty());
    assert_eq!(
        search_issues(&connection, "polished").unwrap(),
        vec!["APP-1"]
    );

    connection
        .execute(
            "UPDATE project SET name = 'Desktop suite' WHERE id = ?1",
            [&project_id],
        )
        .expect("rename project");
    assert_eq!(search_issues(&connection, "suite").unwrap(), vec!["APP-1"]);
    assert_eq!(
        search_projects(&connection, "suite").unwrap(),
        vec![project_id.clone()]
    );
    assert!(search_projects(&connection, "app").unwrap().is_empty());

    let mut query = IssueQuery {
        scope: "active".into(),
        triage: false,
        statuses: vec!["Todo".into()],
        excluded_statuses: vec![],
        priorities: vec!["High".into()],
        excluded_priorities: vec![],
        projects: vec!["Desktop suite".into()],
        excluded_projects: vec![],
        project_statuses: vec![],
        excluded_project_statuses: vec![],
        project_priorities: vec![],
        excluded_project_priorities: vec![],
        project_labels: vec![],
        excluded_project_labels: vec![],
        empty_project_labels: false,
        excluded_empty_project_labels: false,
        project_leads: vec![],
        excluded_project_leads: vec![],
        empty_project_lead: false,
        excluded_empty_project_lead: false,
        project_milestones: vec![],
        excluded_project_milestones: vec![],
        empty_project_milestones: false,
        excluded_empty_project_milestones: false,
        milestones: vec![],
        excluded_milestones: vec![],
        empty_milestone: false,
        excluded_empty_milestone: false,
        advanced_groups: Vec::new(),
        filter_tree: None,
        sort_field: None,
        sort_direction: None,
        include_sub_issues: None,
        labels: vec!["Workflow".into()],
        excluded_labels: vec![],
        empty_labels: false,
        excluded_empty_labels: false,
        due: "none".into(),
        text: "polished".into(),
    };
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.filter_tree = Some(FilterTree::Group {
        op: "and".into(),
        children: vec![
            FilterTree::Condition {
                field: "priority".into(),
                operator: "eq".into(),
                value: Some(serde_json::json!("High")),
            },
            FilterTree::Group {
                op: "or".into(),
                children: vec![
                    FilterTree::Condition {
                        field: "title".into(),
                        operator: "contains".into(),
                        value: Some(serde_json::json!("Polished")),
                    },
                    FilterTree::Condition {
                        field: "title".into(),
                        operator: "contains".into(),
                        value: Some(serde_json::json!("Absent")),
                    },
                ],
            },
        ],
    });
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.filter_tree = Some(FilterTree::Condition {
        field: "priority".into(),
        operator: "eq".into(),
        value: Some(serde_json::json!("Low")),
    });
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.filter_tree = None;
    let page = query_issue_ids_window(&connection, &query, 1).unwrap();
    assert_eq!(page.ids, vec!["APP-1"]);
    assert_eq!(page.total_count, 1);
    query.advanced_groups = vec![IssueFilterGroup {
        mode: "any".into(),
        rules: vec![
            IssueFilterRule {
                field: "due".into(),
                operator: "has".into(),
                value: None,
                end: None,
            },
            IssueFilterRule {
                field: "created".into(),
                operator: "before".into(),
                value: Some("2100-01-01".into()),
                end: None,
            },
        ],
    }];
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    let page = query_issue_ids_window(&connection, &query, 1).unwrap();
    assert_eq!(page.ids, vec!["APP-1"]);
    assert_eq!(page.total_count, 1);
    query.advanced_groups.push(IssueFilterGroup {
        mode: "all".into(),
        rules: vec![IssueFilterRule {
            field: "blocks".into(),
            operator: "has".into(),
            value: None,
            end: None,
        }],
    });
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.advanced_groups[1].rules[0].operator = "none".into();
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.advanced_groups.clear();
    query.project_statuses = vec!["Planned".into()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.project_statuses = vec!["Backlog".into()];
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.excluded_project_statuses = vec!["Backlog".into()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.excluded_project_statuses.clear();
    query.project_priorities = vec!["High".into()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.project_priorities = vec!["No priority".into()];
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.excluded_project_priorities = vec!["No priority".into()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.excluded_project_priorities.clear();
    query.project_statuses.clear();
    query.project_priorities.clear();
    query.project_statuses = vec!["Invalid".into()];
    assert!(query_issue_ids(&connection, &query).is_err());
    query.project_statuses.clear();
    query.project_labels = vec!["Workflow".into()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.project_labels.clear();
    query.empty_project_labels = true;
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.empty_project_labels = false;

    connection
        .execute(
            "UPDATE project SET labels_json = '[\"Workflow\",\"UI\"]' WHERE id = ?1",
            [&project_id],
        )
        .expect("add project labels");
    query.project_labels = vec!["Workflow".into()];
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.project_labels = vec!["Missing".into(), "UI".into()];
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.excluded_project_labels = vec!["UI".into()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.excluded_project_labels.clear();
    query.empty_project_labels = true;
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.empty_project_labels = false;
    query.excluded_empty_project_labels = true;
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.excluded_empty_project_labels = false;
    query.project_labels = vec![" ".into()];
    assert!(query_issue_ids(&connection, &query).is_err());
    query.project_labels.clear();

    query.project_leads = vec![actor_id.clone()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.project_leads.clear();
    query.empty_project_lead = true;
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.empty_project_lead = false;
    query.empty_project_milestones = true;
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.empty_project_milestones = false;

    connection
        .execute(
            "UPDATE project SET lead_actor_id = ?1 WHERE id = ?2",
            params![actor_id, project_id],
        )
        .expect("assign project lead");
    let milestone_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO milestone(id, project_id, title, sort_order)
             VALUES (?1, ?2, 'Beta', 0)",
            params![milestone_id, project_id],
        )
        .expect("create project milestone");
    query.empty_milestone = true;
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.empty_milestone = false;
    query.excluded_empty_milestone = true;
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.excluded_empty_milestone = false;
    connection
        .execute(
            "UPDATE issue SET milestone_id = ?1 WHERE id = ?2",
            params![milestone_id, issue_id],
        )
        .expect("assign issue milestone");
    query.milestones = vec![milestone_id.clone()];
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.milestones.clear();
    query.excluded_milestones = vec![milestone_id.clone()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.excluded_milestones.clear();
    query.excluded_empty_milestone = true;
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.excluded_empty_milestone = false;
    query.empty_milestone = true;
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.empty_milestone = false;
    query.milestones = vec!["not-a-uuid".into()];
    assert!(query_issue_ids(&connection, &query).is_err());
    query.milestones.clear();
    query.project_leads = vec![actor_id.clone()];
    query.project_milestones = vec!["Beta".into()];
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.excluded_project_leads = vec![actor_id.clone()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.excluded_project_leads.clear();
    query.excluded_project_milestones = vec!["Beta".into()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.excluded_project_milestones.clear();
    query.project_leads = vec!["not-an-actor-id".into()];
    assert!(query_issue_ids(&connection, &query).is_err());
    query.project_leads.clear();
    query.project_milestones = vec![" ".into()];
    assert!(query_issue_ids(&connection, &query).is_err());
    query.project_milestones.clear();

    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, description,
                               status, priority, labels_json, position)
             VALUES (?1, ?2, 'LOCAL-10', 'Unassigned work', '', 'Todo', 'High', '[]', 99)",
            params![Uuid::new_v4().to_string(), workspace_id],
        )
        .expect("create unassigned issue");
    query.projects.clear();
    query.labels.clear();
    query.text.clear();
    query.empty_project_labels = true;
    assert_eq!(
        query_issue_ids(&connection, &query).unwrap(),
        vec!["LOCAL-10"]
    );
    query.empty_project_labels = false;
    query.excluded_empty_project_labels = true;
    assert!(!query_issue_ids(&connection, &query)
        .unwrap()
        .contains(&"LOCAL-10".to_owned()));
    query.excluded_empty_project_labels = false;
    query.empty_project_lead = true;
    query.empty_project_milestones = true;
    assert_eq!(
        query_issue_ids(&connection, &query).unwrap(),
        vec!["LOCAL-10"]
    );
    query.empty_project_lead = false;
    query.empty_project_milestones = false;
    query.projects = vec!["Desktop suite".into()];
    query.labels = vec!["Workflow".into()];
    query.text = "polished".into();
    query.priorities = vec!["Low".into()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.priorities = vec!["High".into()];
    query.triage = true;
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.triage = false;

    connection
        .execute(
            "UPDATE issue SET status = 'Done' WHERE id = ?1",
            [&issue_id],
        )
        .expect("complete issue");
    query.statuses = vec!["Done".into()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.scope = "all".into();
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.text.clear();
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);
    query.text = "polished".into();

    connection
        .execute("UPDATE issue SET triage = 1 WHERE id = ?1", [&issue_id])
        .expect("move issue to triage");
    query.triage = true;
    query.scope = "active".into();
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);

    let second_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, project_id, title,
                                   description, status, priority, labels_json, position)
                 VALUES (?1, ?2, 'APP-2', ?3, 'Second issue', '',
                         'Todo', 'Low', '[\"UI\"]', 1)",
            params![second_id, workspace_id, project_id],
        )
        .expect("create second issue");
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, project_id, title,
                                   description, status, priority, labels_json, position)
                 VALUES (?1, ?2, 'APP-3', ?3, 'Unlabeled issue', '',
                         'Todo', 'High', '[]', 2)",
            params![Uuid::new_v4().to_string(), workspace_id, project_id],
        )
        .expect("create unlabeled issue");
    connection
        .execute("UPDATE issue SET triage = 0 WHERE id = ?1", [&issue_id])
        .expect("accept first issue");
    query.triage = false;
    query.scope = "all".into();
    query.statuses = vec!["Done".into(), "Todo".into()];
    query.priorities = vec!["High".into(), "Low".into()];
    query.labels = vec!["Workflow".into(), "UI".into()];
    query.text.clear();
    assert_eq!(
        query_issue_ids(&connection, &query).unwrap(),
        vec!["APP-1", "APP-2"]
    );
    let first_page = query_issue_ids_window_after(&connection, &query, 1, None).unwrap();
    assert_eq!(first_page.ids, vec!["APP-1"]);
    assert_eq!(first_page.total_count, 2);
    let second_page =
        query_issue_ids_window_after(&connection, &query, 1, first_page.next_cursor.as_deref())
            .unwrap();
    assert_eq!(second_page.ids, vec!["APP-2"]);
    assert!(second_page.next_cursor.is_none());
    connection
        .execute(
            "UPDATE issue SET title = 'Updated second issue' WHERE id = ?1",
            [&second_id],
        )
        .unwrap();
    assert!(query_issue_ids_window_after(
        &connection,
        &query,
        1,
        first_page.next_cursor.as_deref(),
    )
    .is_err());
    query.text = "different".into();
    assert!(query_issue_ids_window_after(
        &connection,
        &query,
        1,
        first_page.next_cursor.as_deref(),
    )
    .is_err());
    query.text.clear();
    query.labels = vec!["UI".into()];
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-2"]);
    query.excluded_priorities = vec!["Low".into()];
    assert!(query_issue_ids(&connection, &query).unwrap().is_empty());
    query.excluded_priorities.clear();
    query.labels = vec!["Workflow".into(), "UI".into()];
    query.excluded_labels = vec!["Workflow".into()];
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-2"]);
    query.labels.clear();
    query.excluded_labels.clear();
    query.empty_labels = true;
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-3"]);
    query.empty_labels = false;
    query.excluded_empty_labels = true;
    assert_eq!(
        query_issue_ids(&connection, &query).unwrap(),
        vec!["APP-1", "APP-2"]
    );

    let document = serde_json::json!({
        "version": 1,
        "content": {"type": "doc", "content": [{
            "type": "taskList", "content": [{
                "type": "taskItem", "attrs": {"checked": true, "text": "hiddenattribute"},
                "content": [{"type": "paragraph", "content": [
                    {"type": "text", "text": "Visible launch handoff",
                     "marks": [{"type": "link", "attrs": {
                        "href": "https://example.com", "text": "hiddenmark"
                     }}]}
                ]}]
            }]
        }]}
    });
    connection
            .execute(
                "UPDATE issue SET description = 'stale markdown', description_document_json = ?1 WHERE id = ?2",
                params![document.to_string(), issue_id],
            )
            .expect("save structured description");
    assert_eq!(
        search_issues(&connection, "handoff").unwrap(),
        vec!["APP-1"]
    );
    assert!(search_issues(&connection, "stale").unwrap().is_empty());
    assert!(search_issues(&connection, "hiddenattribute")
        .unwrap()
        .is_empty());
    assert!(search_issues(&connection, "hiddenmark").unwrap().is_empty());

    query.text = "handoff".into();
    query.due = "any".into();
    assert_eq!(query_issue_ids(&connection, &query).unwrap(), vec!["APP-1"]);

    drop(connection);
    let mut connection = open_database(&root).expect("reopen search index");
    assert_eq!(
        search_issues(&connection, "handoff").unwrap(),
        vec!["APP-1"]
    );
    assert!(search_issues(&connection, "stale").unwrap().is_empty());

    create_workspace(&mut connection, "Other workspace").expect("second workspace");
    assert!(search_issues(&connection, "polished").unwrap().is_empty());
    assert!(search_projects(&connection, "suite").unwrap().is_empty());
    assert!(search_comments(&connection, "Commentonly")
        .unwrap()
        .is_empty());
    assert!(find_comment(&connection, "APP-1", &comment_id)
        .unwrap()
        .is_none());
    switch_workspace(&connection, &workspace_id).expect("switch back");
    assert_eq!(
        search_issues(&connection, "polished").unwrap(),
        vec!["APP-1"]
    );

    connection
        .execute(
            "UPDATE issue SET archived_at = '2026-10-05T00:00:00Z' WHERE id = ?1",
            [&issue_id],
        )
        .expect("archive issue");
    assert!(search_issues(&connection, "polished").unwrap().is_empty());
    assert!(search_issues(&connection, "Commentonly")
        .unwrap()
        .is_empty());
    assert!(search_comments(&connection, "Commentonly")
        .unwrap()
        .is_empty());
    assert!(find_comment(&connection, "APP-1", &comment_id)
        .unwrap()
        .is_none());
    drop(connection);
    fs::remove_dir_all(root).expect("remove test data");
}
