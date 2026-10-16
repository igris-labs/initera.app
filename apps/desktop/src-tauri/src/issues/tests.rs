use super::*;
use crate::storage::{create_project, create_workspace, save_identity, switch_workspace};
use std::fs;
use std::time::Instant;

fn draft(project: &str) -> IssueInput {
    IssueInput {
        title: "First issue".into(),
        title_markdown: "**First** issue".into(),
        description: "## Details".into(),
        description_document: None,
        status: "Backlog".into(),
        priority: "Medium".into(),
        project: project.into(),
        milestone_id: None,
        labels: vec!["Core".into()],
        due: Some("2026-10-02".into()),
        start_date: Some("2026-09-30".into()),
        triage: true,
        parent_id: None,
        estimate: None,
    }
}

#[test]
fn repeated_status_visits_and_structured_changes_replay_once() {
    let root = std::env::temp_dir().join(format!("pm-app-status-history-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let mut input = draft("Unassigned");
    input.status = "Todo".into();
    let created = create_issue(&mut connection, input.clone()).unwrap();

    input.status = "In Progress".into();
    input.priority = "High".into();
    let first_mutation = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(created.version),
        expected_versions: Default::default(),
    };
    let first = update_issue_with_mutation(
        &mut connection,
        &created.id,
        input.clone(),
        None,
        Some(first_mutation.clone()),
    )
    .unwrap();
    let replay = update_issue_with_mutation(
        &mut connection,
        &created.id,
        input.clone(),
        None,
        Some(first_mutation),
    )
    .unwrap();
    assert_eq!(first.version, replay.version);

    input.status = "Todo".into();
    update_issue(&mut connection, &created.id, input, None).unwrap();
    let visits: Vec<(String, Option<String>)> = connection
        .prepare(
            "SELECT status_name, left_at FROM issue_status_visit WHERE issue_id = ?1 ORDER BY id",
        )
        .unwrap()
        .query_map([&created.entity_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        visits
            .iter()
            .map(|(status, _)| status.as_str())
            .collect::<Vec<_>>(),
        ["Todo", "In Progress", "Todo"]
    );
    assert!(visits[0].1.is_some() && visits[1].1.is_some() && visits[2].1.is_none());

    let events = list_issue_activity(&connection, &created.id, 0).unwrap();
    let change = events
        .iter()
        .find(|event| {
            event
                .changes
                .as_ref()
                .is_some_and(|value| value.as_array().is_some_and(|items| items.len() == 2))
        })
        .unwrap();
    assert_eq!(change.changes.as_ref().unwrap()[0]["field"], "status");
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bulk_triage_is_atomic_and_replays_the_same_decision() {
    let root = std::env::temp_dir().join(format!("pm-app-bulk-triage-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let first = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let second = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let mutation = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let mut targets = vec![
        TriageTarget {
            identifier: first.id.clone(),
            version: first.version,
        },
        TriageTarget {
            identifier: second.id.clone(),
            version: second.version + 1,
        },
    ];
    assert!(bulk_triage(
        &mut connection,
        &targets,
        false,
        "Canceled",
        Some("Duplicate"),
        &mutation
    )
    .is_err());
    let pending: i64 = connection
        .query_row("SELECT count(*) FROM issue WHERE triage = 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(pending, 2);

    targets[1].version = second.version;
    let decided = bulk_triage(
        &mut connection,
        &targets,
        false,
        "Canceled",
        Some("Duplicate"),
        &mutation,
    )
    .unwrap();
    assert_eq!(decided.len(), 2);
    assert!(decided
        .iter()
        .all(|issue| !issue.triage && issue.status == "Canceled"));
    assert_eq!(
        bulk_triage(
            &mut connection,
            &targets,
            false,
            "Canceled",
            Some("Duplicate"),
            &mutation
        )
        .unwrap()
        .len(),
        2
    );
    let reasons: i64 = connection.query_row(
            "SELECT count(*) FROM issue_activity WHERE body = 'declined issue from triage: Duplicate'",
            [], |row| row.get(0),
        ).unwrap();
    assert_eq!(reasons, 2);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn issue_lifecycle_receipts_replay_and_reject_stale_versions() {
    let root = std::env::temp_dir().join(format!("pm-app-lifecycle-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let issue = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let archive = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(issue.version),
        expected_versions: Default::default(),
    };
    archive_issue_with_mutation(&mut connection, &issue.id, "all", None, Some(&archive)).unwrap();
    archive_issue_with_mutation(&mut connection, &issue.id, "all", None, Some(&archive)).unwrap();
    assert!(archive_issue_with_mutation(
        &mut connection,
        &issue.id,
        "detach",
        None,
        Some(&archive),
    )
    .unwrap_err()
    .contains("IDEMPOTENCY_CONFLICT"));

    let archived = query_issues_window(&connection, true, 10, 0).unwrap();
    let restore = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(archived[0].version),
        expected_versions: Default::default(),
    };
    restore_issue_with_mutation(&mut connection, &issue.id, Some(&restore)).unwrap();
    restore_issue_with_mutation(&mut connection, &issue.id, Some(&restore)).unwrap();
    let stale = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(issue.version),
        expected_versions: Default::default(),
    };
    assert!(
        archive_issue_with_mutation(&mut connection, &issue.id, "all", None, Some(&stale),)
            .unwrap_err()
            .contains("CONFLICT:")
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_move_receipt_replays_without_allocating_another_identifier() {
    let root = std::env::temp_dir().join(format!("pm-app-move-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    create_project(&mut connection, "Design", "DSN").unwrap();
    let issue = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let preview = preview_issue_move(&connection, &issue.id, "Design").unwrap();
    let mutation = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(issue.version),
        expected_versions: Default::default(),
    };
    let first = move_issue_project_with_mutation(
        &mut connection,
        &issue.id,
        "Design",
        "regenerate",
        &preview.revision,
        Some(&mutation),
    )
    .unwrap();
    let replay = move_issue_project_with_mutation(
        &mut connection,
        &issue.id,
        "Design",
        "regenerate",
        &preview.revision,
        Some(&mutation),
    )
    .unwrap();
    assert_eq!(first[0].id, replay[0].id);
    let count: i64 = connection
        .query_row("SELECT count(*) FROM issue", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn issue_pages_preserve_order_and_archive_scope() {
    let root = std::env::temp_dir().join(format!("pm-app-issue-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();

    let first = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let second = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let third = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let counts = issue_counts(&connection).unwrap();
    assert_eq!(counts.triage, 3);
    assert_eq!(counts.statuses.get("Backlog"), Some(&3));

    let core_id: String = connection
        .query_row(
            "SELECT id FROM label WHERE name = 'Core' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute(
            "UPDATE issue SET labels_json = '[]' WHERE identifier = ?1",
            [&first.id],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO issue_label(issue_id, label_id)
                 SELECT id, ?1 FROM issue WHERE identifier = ?2",
            params![core_id, first.id],
        )
        .unwrap();
    assert_eq!(
        query_issues_window(&connection, false, 3, 0).unwrap()[2].labels,
        vec!["Core"]
    );

    assert_eq!(
        query_issues_window(&connection, false, 2, 0)
            .unwrap()
            .iter()
            .map(|issue| issue.id.as_str())
            .collect::<Vec<_>>(),
        vec![third.id.as_str(), second.id.as_str()]
    );
    assert_eq!(
        query_issues_window(&connection, false, 2, 2).unwrap()[0].id,
        first.id
    );
    let selected = serde_json::to_string(&vec![first.id.clone(), third.id.clone()]).unwrap();
    assert_eq!(
        query_issues_window_for_ids(&connection, false, 200, Some(&selected))
            .unwrap()
            .iter()
            .map(|issue| issue.id.as_str())
            .collect::<Vec<_>>(),
        vec![third.id.as_str(), first.id.as_str()]
    );

    archive_issue(&mut connection, &second.id).unwrap();
    assert_eq!(
        issue_counts(&connection).unwrap().statuses.get("Backlog"),
        Some(&3)
    );
    assert_eq!(
        query_issues_window(&connection, true, 2, 0).unwrap()[0].id,
        second.id
    );
    assert_eq!(
        query_issues_window(&connection, false, 2, 0).unwrap().len(),
        2
    );

    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn versioned_issue_writes_replay_and_reject_stale_updates() {
    let root = std::env::temp_dir().join(format!("pm-app-receipt-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let input = draft("Unassigned");
    let create = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let first =
        create_issue_with_mutation(&mut connection, input.clone(), Some(create.clone())).unwrap();
    let repeated =
        create_issue_with_mutation(&mut connection, input.clone(), Some(create.clone())).unwrap();
    assert_eq!(first.id, repeated.id);
    assert_eq!(first.version, 1);
    assert!(create_issue_with_mutation(
        &mut connection,
        {
            let mut other = input.clone();
            other.title = "Different".into();
            other
        },
        Some(create)
    )
    .unwrap_err()
    .contains("IDEMPOTENCY_CONFLICT"));

    let mut changed = input.clone();
    changed.title = "Updated".into();
    let update = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(first.version),
        expected_versions: Default::default(),
    };
    let saved = update_issue_with_mutation(
        &mut connection,
        &first.id,
        changed.clone(),
        None,
        Some(update.clone()),
    )
    .unwrap();
    assert_eq!(saved.version, 2);
    let repeated = update_issue_with_mutation(
        &mut connection,
        &first.id,
        changed.clone(),
        None,
        Some(update),
    )
    .unwrap();
    assert_eq!(repeated.version, 2);
    let stale = update_issue_with_mutation(
        &mut connection,
        &first.id,
        input,
        None,
        Some(MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: Some(1),
            expected_versions: Default::default(),
        }),
    );
    assert!(stale.unwrap_err().contains("CONFLICT"));
    let receipt_count: i64 = connection
        .query_row("SELECT count(*) FROM mutation_receipt", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(receipt_count, 2);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn inactive_project_cannot_receive_new_issues_or_milestones() {
    let root = std::env::temp_dir().join(format!("pm-app-inactive-project-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    create_project(&mut connection, "Desktop app", "APP").unwrap();
    let project_id: String = connection
        .query_row(
            "SELECT id FROM project WHERE name = 'Desktop app'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let milestone_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO milestone(id, project_id, title, sort_order) VALUES (?1, ?2, 'Launch', 0)",
            params![milestone_id, project_id],
        )
        .unwrap();
    let first = create_issue(&mut connection, draft("Unassigned")).unwrap();
    connection
        .execute(
            "UPDATE milestone SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            [&milestone_id],
        )
        .unwrap();
    let mut with_removed_milestone = draft("Desktop app");
    with_removed_milestone.milestone_id = Some(milestone_id);
    assert!(create_issue(&mut connection, with_removed_milestone).is_err());
    connection
        .execute(
            "UPDATE project SET archived_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            [&project_id],
        )
        .unwrap();
    assert!(create_issue(&mut connection, draft("Desktop app")).is_err());
    assert!(preview_issue_move(&connection, &first.id, "Desktop app").is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn milestone_assignment_stays_with_its_project() {
    let root = std::env::temp_dir().join(format!("pm-app-milestone-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    create_project(&mut connection, "Release", "REL").unwrap();
    create_project(&mut connection, "Other", "OTH").unwrap();
    let project_id: String = connection
        .query_row("SELECT id FROM project WHERE name = 'Release'", [], |row| {
            row.get(0)
        })
        .unwrap();
    let milestone_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO milestone(id, project_id, title, sort_order) VALUES (?1, ?2, 'Beta', 0)",
            params![milestone_id, project_id],
        )
        .unwrap();

    let mut input = draft("Release");
    input.milestone_id = Some(milestone_id.clone());
    let issue = create_issue(&mut connection, input.clone()).unwrap();
    assert_eq!(issue.milestone_id.as_deref(), Some(milestone_id.as_str()));
    drop(connection);

    let mut connection = open_database(&root).unwrap();
    assert_eq!(
        list_issues(&connection).unwrap()[0].milestone_id,
        Some(milestone_id.clone())
    );
    input.project = "Other".into();
    assert!(update_issue(&mut connection, &issue.id, input.clone(), None).is_err());
    assert_eq!(list_issues(&connection).unwrap()[0].project, "Release");
    input.milestone_id = None;
    let preview = preview_issue_move(&connection, &issue.id, "Other").unwrap();
    let moved = move_issue_project(
        &mut connection,
        &issue.id,
        "Other",
        "keep",
        &preview.revision,
    )
    .unwrap();
    assert_eq!(moved[0].milestone_id, None);
    let preview = preview_issue_move(&connection, &issue.id, "Release").unwrap();
    move_issue_project(
        &mut connection,
        &issue.id,
        "Release",
        "keep",
        &preview.revision,
    )
    .unwrap();
    input.project = "Release".into();
    input.milestone_id = Some(milestone_id.clone());
    update_issue(&mut connection, &issue.id, input, None).unwrap();
    connection
        .execute("DELETE FROM milestone WHERE id = ?1", [&milestone_id])
        .unwrap();
    assert_eq!(list_issues(&connection).unwrap()[0].milestone_id, None);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reorder_partial_page_keeps_unloaded_issue_in_place() {
    let root = std::env::temp_dir().join(format!("pm-app-reorder-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace_id = current_workspace_id(&connection).unwrap();
    let tx = connection.transaction().unwrap();
    for index in 0..201 {
        tx.execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, description, status,
                 priority, labels_json, position) VALUES (?1, ?2, ?3, 'Issue', '', 'Todo',
                 'Medium', '[]', ?4)",
            params![
                Uuid::new_v4().to_string(),
                workspace_id,
                format!("LOCAL-{index}"),
                index
            ],
        )
        .unwrap();
    }
    tx.commit().unwrap();

    let page = issue_record_page(&connection, false, 200, None).unwrap();
    assert_eq!(page.items.len(), 200);
    let next = issue_record_page(&connection, false, 200, page.next_cursor.as_deref()).unwrap();
    assert_eq!(next.items.len(), 1);
    assert!(next.next_cursor.is_none());

    let first_page = query_issues_window(&connection, false, 200, 0).unwrap();
    let versions = first_page
        .iter()
        .map(|issue| (issue.id.clone(), issue.version))
        .collect();
    let mut visible: Vec<_> = first_page.into_iter().map(|issue| issue.id).collect();
    visible.swap(0, 1);
    let mutation = ReorderMutation {
        mutation_id: Uuid::new_v4().to_string(),
        versions,
    };
    reorder_issues_with_mutation(&mut connection, visible.clone(), Some(mutation.clone())).unwrap();
    reorder_issues_with_mutation(&mut connection, visible.clone(), Some(mutation.clone())).unwrap();
    let mut changed_request = mutation.clone();
    changed_request.versions.insert("LOCAL-0".into(), 999);
    assert!(
        reorder_issues_with_mutation(&mut connection, visible.clone(), Some(changed_request),)
            .unwrap_err()
            .contains("IDEMPOTENCY_CONFLICT")
    );
    assert!(reorder_issues_with_mutation(
        &mut connection,
        visible,
        Some(ReorderMutation {
            mutation_id: Uuid::new_v4().to_string(),
            versions: mutation.versions,
        }),
    )
    .unwrap_err()
    .contains("CONFLICT"));

    let order: Vec<String> = connection
        .prepare("SELECT identifier FROM issue ORDER BY position, id")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(&order[..3], &["LOCAL-1", "LOCAL-0", "LOCAL-2"]);
    assert_eq!(order[200], "LOCAL-200");
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn activity_cursor_keeps_equal_timestamp_entries_in_order() {
    let root = std::env::temp_dir().join(format!("pm-app-activity-page-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let issue = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let issue_id: String = connection
        .query_row(
            "SELECT id FROM issue WHERE identifier = ?1",
            [&issue.id],
            |row| row.get(0),
        )
        .unwrap();
    for index in 0..31 {
        connection
            .execute(
                "INSERT INTO issue_activity(id, issue_id, actor_name, kind, body, created_at)
                 VALUES (?1, ?2, 'Rahul', 'comment', ?3, '2026-01-01T00:00:00Z')",
                params![
                    Uuid::new_v4().to_string(),
                    issue_id,
                    format!("Comment {index}")
                ],
            )
            .unwrap();
    }
    let first = issue_activity_page(&connection, &issue.id, None).unwrap();
    assert_eq!(first.items.len(), 30);
    let second = issue_activity_page(&connection, &issue.id, first.next_cursor.as_deref()).unwrap();
    assert_eq!(second.items.len(), 2);
    assert!(second.next_cursor.is_none());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn ten_thousand_issue_records_load_in_position_order() {
    let root = std::env::temp_dir().join(format!("pm-app-large-issue-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace_id = current_workspace_id(&connection).unwrap();
    let description =
        "A representative issue description with context and acceptance criteria. ".repeat(3);

    let transaction = connection.transaction().unwrap();
    {
        let mut insert = transaction
            .prepare(
                "INSERT INTO issue(id, workspace_id, identifier, title, description,
                                       status, priority, labels_json, position)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'Todo', 'Medium', '[\"Core\",\"UI\"]', ?6)",
            )
            .unwrap();

        for index in 0..10_000 {
            let identifier = format!("LOCAL-{}", index + 1);
            insert
                .execute(params![
                    Uuid::new_v4().to_string(),
                    workspace_id,
                    identifier,
                    format!("Performance issue {}", index + 1),
                    description,
                    index,
                ])
                .unwrap();
        }
    }
    transaction.commit().unwrap();

    let started = Instant::now();
    let records = list_issues(&connection).unwrap();
    let read_time = started.elapsed();
    let serialized = serde_json::to_vec(&records).unwrap();
    eprintln!(
        "10,000 native issue records read in {:?}; serialized {} bytes in {:?}",
        read_time,
        serialized.len(),
        started.elapsed() - read_time
    );
    assert_eq!(records.len(), 10_000);
    assert_eq!(records.first().unwrap().id, "LOCAL-1");
    assert_eq!(records.last().unwrap().id, "LOCAL-10000");

    let mut page_times = Vec::with_capacity(30);
    for _ in 0..30 {
        let started = Instant::now();
        let page = issue_record_page(&connection, false, 200, None).unwrap();
        page_times.push(started.elapsed().as_micros());
        assert_eq!(page.items.len(), 200);
        assert!(page.next_cursor.is_some());
    }
    page_times.sort_unstable();
    eprintln!(
        "10,000-issue first-page warm trials (µs): {:?}; p95: {} µs",
        page_times, page_times[28]
    );

    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn archive_and_restore_cascade_within_the_current_workspace() {
    let root = std::env::temp_dir().join(format!("pm-app-issue-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let first_workspace = current_workspace_id(&connection).unwrap();
    let parent = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let mut child_input = draft("Unassigned");
    child_input.parent_id = Some(parent.id.clone());
    let child = create_issue(&mut connection, child_input).unwrap();
    connection
        .execute(
            "INSERT INTO time_entry (id, workspace_id, issue_id, started_at)
             SELECT ?1, workspace_id, id, unixepoch('now') - 120 FROM issue WHERE identifier = ?2",
            params![Uuid::new_v4().to_string(), child.id],
        )
        .unwrap();
    archive_issue(&mut connection, &parent.id).unwrap();
    let paused: bool = connection.query_row(
            "SELECT stopped_at IS NOT NULL FROM time_entry WHERE issue_id = (SELECT id FROM issue WHERE identifier = ?1)",
            [&child.id], |row| row.get(0),
        ).unwrap();
    assert!(paused);
    assert!(list_issues(&connection).unwrap().is_empty());
    let archived = list_archived_issues(&connection).unwrap();
    assert_eq!(archived.len(), 2);
    assert!(archived
        .iter()
        .all(|issue| issue.archive_root_id.as_deref() == Some(parent.id.as_str())));
    assert!(restore_issue(&mut connection, &child.id).is_err());
    reorder_issues(&mut connection, vec![]).unwrap();

    create_workspace(&mut connection, "Personal").unwrap();
    let second_workspace = current_workspace_id(&connection).unwrap();
    assert!(list_archived_issues(&connection).unwrap().is_empty());
    let other = create_issue(&mut connection, draft("Unassigned")).unwrap();
    assert_eq!(other.id, parent.id);
    assert!(restore_issue(&mut connection, &parent.id).is_err());
    assert_eq!(list_issues(&connection).unwrap().len(), 1);

    switch_workspace(&connection, &first_workspace).unwrap();
    restore_issue(&mut connection, &parent.id).unwrap();
    assert!(list_archived_issues(&connection).unwrap().is_empty());
    assert_eq!(list_issues(&connection).unwrap().len(), 2);
    reorder_issues(&mut connection, vec![child.id.clone(), parent.id.clone()]).unwrap();
    drop(connection);

    let connection = open_database(&root).unwrap();
    assert_eq!(list_issues(&connection).unwrap().len(), 2);
    switch_workspace(&connection, &second_workspace).unwrap();
    assert_eq!(list_issues(&connection).unwrap().len(), 1);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn archive_receipt_lists_parent_and_child_as_removed() {
    let root = std::env::temp_dir().join(format!("pm-app-issue-effects-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let parent = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let mut child_input = draft("Unassigned");
    child_input.parent_id = Some(parent.id.clone());
    let child = create_issue(&mut connection, child_input).unwrap();
    let parent_entity_id: String = connection
        .query_row(
            "SELECT id FROM issue WHERE identifier = ?1",
            [&parent.id],
            |row| row.get(0),
        )
        .unwrap();
    let child_entity_id: String = connection
        .query_row(
            "SELECT id FROM issue WHERE identifier = ?1",
            [&child.id],
            |row| row.get(0),
        )
        .unwrap();
    let version: i64 = connection
        .query_row(
            "SELECT version FROM issue WHERE id = ?1",
            [&parent_entity_id],
            |row| row.get(0),
        )
        .unwrap();
    let request = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(version),
        expected_versions: std::collections::BTreeMap::from([(parent_entity_id.clone(), version)]),
    };
    let _capture = crate::api_contract::begin_change_capture();
    archive_issue_with_mutation(&mut connection, &parent.id, "all", None, Some(&request)).unwrap();
    let json: String = connection
        .query_row(
            "SELECT effects_json FROM mutation_receipt WHERE mutation_id = ?1",
            [&request.mutation_id],
            |row| row.get(0),
        )
        .unwrap();
    let effects: crate::api_contract::MutationEffects = serde_json::from_str(&json).unwrap();
    assert!(effects
        .removed_from_active
        .iter()
        .any(|item| item.id == parent_entity_id));
    assert!(effects
        .removed_from_active
        .iter()
        .any(|item| item.id == child_entity_id));
    drop(_capture);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn archive_and_delete_can_detach_or_move_sub_issues() {
    let root = std::env::temp_dir().join(format!("pm-app-issue-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let parent = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let destination = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let mut child_input = draft("Unassigned");
    child_input.parent_id = Some(parent.id.clone());
    let child = create_issue(&mut connection, child_input).unwrap();

    assert!(
        archive_issue_with_strategy(&mut connection, &parent.id, "move", Some(&child.id)).is_err()
    );
    archive_issue_with_strategy(&mut connection, &parent.id, "move", Some(&destination.id))
        .unwrap();
    let active = list_issues(&connection).unwrap();
    assert_eq!(
        active
            .iter()
            .find(|issue| issue.id == child.id)
            .unwrap()
            .parent_id
            .as_deref(),
        Some(destination.id.as_str())
    );
    assert_eq!(list_archived_issues(&connection).unwrap().len(), 1);
    delete_archived_issue(&mut connection, &parent.id, "all", None).unwrap();
    assert_eq!(list_issues(&connection).unwrap().len(), 2);

    archive_issue(&mut connection, &destination.id).unwrap();
    assert_eq!(list_archived_issues(&connection).unwrap().len(), 2);
    delete_archived_issue(&mut connection, &destination.id, "detach", None).unwrap();
    let remaining = list_issues(&connection).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, child.id);
    assert_eq!(remaining[0].parent_id, None);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn archived_sub_issue_can_be_restored_without_restoring_its_parent() {
    let root = std::env::temp_dir().join(format!("pm-app-issue-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let parent = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let mut child_input = draft("Unassigned");
    child_input.parent_id = Some(parent.id.clone());
    let child = create_issue(&mut connection, child_input).unwrap();
    archive_issue(&mut connection, &parent.id).unwrap();
    restore_sub_issue(&mut connection, &child.id).unwrap();
    assert_eq!(list_archived_issues(&connection).unwrap().len(), 1);
    let active = list_issues(&connection).unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, child.id);
    assert_eq!(active[0].parent_id, None);
    restore_issue(&mut connection, &parent.id).unwrap();
    assert_eq!(list_issues(&connection).unwrap().len(), 2);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn issue_activity_and_comments_survive_restart_and_follow_workspace_scope() {
    let root = std::env::temp_dir().join(format!("pm-app-issue-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let first_workspace = current_workspace_id(&connection).unwrap();
    let issue = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let mut changed = draft("Unassigned");
    changed.status = "In Progress".into();
    update_issue(&mut connection, &issue.id, changed, None).unwrap();
    assert!(create_issue_comment(&mut connection, &issue.id, " ", None).is_err());
    let comment =
        create_issue_comment(&mut connection, &issue.id, "**Working** on it", None).unwrap();
    assert_eq!(comment.actor_name, "Rahul");
    let entries = list_issue_activity(&connection, &issue.id, 0).unwrap();
    assert_eq!(entries.len(), 3);
    assert!(entries
        .iter()
        .any(|entry| entry.body.contains("changed status")));
    assert!(entries.iter().any(|entry| entry.kind == "created"));
    create_workspace(&mut connection, "Other").unwrap();
    assert!(list_issue_activity(&connection, &issue.id, 0).is_err());
    drop(connection);

    let connection = open_database(&root).unwrap();
    assert!(list_issue_activity(&connection, &issue.id, 0).is_err());
    switch_workspace(&connection, &first_workspace).unwrap();
    assert_eq!(
        list_issue_activity(&connection, &issue.id, 0)
            .unwrap()
            .len(),
        3
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn comment_image_document_survives_restart_without_using_the_text_limit() {
    let root = std::env::temp_dir().join(format!("pm-app-comment-image-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let issue = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let document = serde_json::json!({
        "version": 1,
        "content": {"type": "doc", "content": [{"type": "image", "attrs": {
            "src": "data:image/png;base64,iVBORw0KGgo=", "alt": "Diagram"
        }}]}
    });

    create_issue_comment(&mut connection, &issue.id, "Image", Some(&document)).unwrap();
    drop(connection);

    let reopened = open_database(&root).unwrap();
    let activity = list_issue_activity(&reopened, &issue.id, 0).unwrap();
    let saved = activity
        .iter()
        .find(|entry| entry.kind == "comment")
        .unwrap();
    assert_eq!(saved.body, "Image");
    assert_eq!(
        saved.body_document_json.as_deref(),
        Some(document.to_string().as_str())
    );
    drop(reopened);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn duplicate_comment_request_returns_one_activity_entry() {
    let root = std::env::temp_dir().join(format!("pm-app-comment-replay-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let issue = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let mutation = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let first = create_issue_comment_with_mutation(
        &mut connection,
        &issue.id,
        "Release is ready",
        None,
        &mutation,
    )
    .unwrap();
    let repeated = create_issue_comment_with_mutation(
        &mut connection,
        &issue.id,
        "Release is ready",
        None,
        &mutation,
    )
    .unwrap();
    assert_eq!(first.id, repeated.id);
    assert_eq!(
        issue_activity_page(&connection, &issue.id, None)
            .unwrap()
            .items
            .len(),
        2
    );
    assert!(create_issue_comment_with_mutation(
        &mut connection,
        &issue.id,
        "Another comment",
        None,
        &mutation,
    )
    .unwrap_err()
    .contains("IDEMPOTENCY_CONFLICT"));
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn comment_edits_update_search_and_removal_keeps_a_tombstone() {
    let root = std::env::temp_dir().join(format!("pm-app-comment-lifecycle-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let issue = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let comment = create_issue_comment(&mut connection, &issue.id, "Originalword", None).unwrap();
    let edit = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(comment.version),
        expected_versions: Default::default(),
    };
    let updated = change_issue_comment(
        &mut connection,
        &issue.id,
        &comment.id,
        Some("Revisedword"),
        None,
        &edit,
    )
    .unwrap();
    assert!(updated.edited_at.is_some());
    assert_eq!(
        change_issue_comment(
            &mut connection,
            &issue.id,
            &comment.id,
            Some("Revisedword"),
            None,
            &edit
        )
        .unwrap()
        .version,
        updated.version
    );
    assert!(change_issue_comment(
        &mut connection,
        &issue.id,
        &comment.id,
        Some("Stale"),
        None,
        &edit
    )
    .is_err());
    let old_hits: i64 = connection
        .query_row(
            "SELECT count(*) FROM comment_search WHERE comment_search MATCH 'Originalword'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let new_hits: i64 = connection
        .query_row(
            "SELECT count(*) FROM comment_search WHERE comment_search MATCH 'Revisedword'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!((old_hits, new_hits), (0, 1));

    let remove = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(updated.version),
        expected_versions: Default::default(),
    };
    let tombstone =
        change_issue_comment(&mut connection, &issue.id, &comment.id, None, None, &remove).unwrap();
    assert_eq!(tombstone.body, "Comment deleted");
    assert!(tombstone.deleted_at.is_some());
    let remaining_hits: i64 = connection
        .query_row(
            "SELECT count(*) FROM comment_search WHERE comment_search MATCH 'Revisedword'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(remaining_hits, 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn issues_survive_restart_and_updates_keep_identifiers() {
    let root = std::env::temp_dir().join(format!("pm-app-issue-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    assert!(create_issue(&mut connection, draft("Unassigned")).is_err());
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let mut first_draft = draft("Unassigned");
    first_draft.description_document = Some(serde_json::json!({
        "version": 1,
        "content": {"type": "doc", "content": [{
            "type": "taskList", "content": [{
                "type": "taskItem", "attrs": {"checked": true},
                "content": [{"type": "paragraph", "content": [
                    {"type": "text", "text": "Details"}
                ]}]
            }]
        }]}
    }));
    let first = create_issue(&mut connection, first_draft).unwrap();
    assert!(first.triage_entered_at.is_some());
    assert!(first.triaged_at.is_none());
    assert_eq!(first.estimate, None);
    assert_eq!(first.id, "LOCAL-1");
    assert_eq!(first.title_markdown, "**First** issue");
    assert_eq!(first.description, "## Details");
    assert_eq!(
        first.description_document.as_ref().unwrap()["content"]["content"][0]["content"][0]
            ["attrs"]["checked"],
        true
    );
    let second = create_issue(&mut connection, draft("Unassigned")).unwrap();
    assert_eq!(second.id, "LOCAL-2");
    let mut edited = draft("Unassigned");
    edited.description_document = first.description_document.clone();
    edited.title = "Updated title".into();
    edited.labels = vec!["Core".into(), "Design".into()];
    edited.estimate = Some(5);
    let updated = update_issue(&mut connection, &first.id, edited.clone(), None).unwrap();
    assert_eq!(updated.id, first.id);
    assert_eq!(updated.estimate, Some(5));
    assert_eq!(updated.triage_entered_at, first.triage_entered_at);
    let mut accepted = edited.clone();
    accepted.triage = false;
    let accepted = update_issue(&mut connection, &first.id, accepted, None).unwrap();
    assert!(accepted.triaged_at.is_some());
    let reopened = update_issue(&mut connection, &first.id, edited, None).unwrap();
    assert!(reopened.triage_entered_at.is_some());
    assert!(reopened.triaged_at.is_none());
    drop(connection);
    let mut connection = open_database(&root).unwrap();
    assert_eq!(
        list_issues(&connection).unwrap()[1].description_document,
        first.description_document
    );
    assert_eq!(list_issues(&connection).unwrap()[0].id, second.id);
    assert_eq!(list_issues(&connection).unwrap()[1].labels.len(), 2);
    assert_eq!(list_issues(&connection).unwrap()[1].estimate, Some(5));
    assert_eq!(
        list_issues(&connection).unwrap()[1].start_date.as_deref(),
        Some("2026-09-30")
    );
    assert_eq!(
        list_issues(&connection).unwrap()[1].due.as_deref(),
        Some("2026-10-02")
    );
    reorder_issues(&mut connection, vec![first.id.clone(), second.id.clone()]).unwrap();
    assert_eq!(list_issues(&connection).unwrap()[0].id, first.id);
    reorder_issues(&mut connection, vec![first.id]).unwrap();
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_issue_write_rolls_back_identifier_allocation() {
    let root = std::env::temp_dir().join(format!("pm-app-issue-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let mut bad = draft("Unassigned");
    bad.title = " ".into();
    assert!(create_issue(&mut connection, bad).is_err());
    assert!(create_issue(&mut connection, draft("Missing project")).is_err());
    let mut invalid_estimate = draft("Unassigned");
    invalid_estimate.estimate = Some(0);
    assert!(create_issue(&mut connection, invalid_estimate).is_err());
    let mut unknown_status = draft("Unassigned");
    unknown_status.status = "Missing".into();
    assert!(create_issue(&mut connection, unknown_status).is_err());
    assert_eq!(
        create_issue(&mut connection, draft("Unassigned"))
            .unwrap()
            .id,
        "LOCAL-1"
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_assignment_uses_project_prefix_and_atomic_move() {
    let root = std::env::temp_dir().join(format!("pm-app-issue-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    create_project(&mut connection, "Desktop app", "APP").unwrap();
    let first = create_issue(&mut connection, draft("Desktop app")).unwrap();
    assert_eq!(first.id, "APP-1");
    let second = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let mut move_input = draft("Unassigned");
    move_input.triage = false;
    assert!(update_issue(
        &mut connection,
        &first.id,
        move_input.clone(),
        Some(vec![first.id.clone()])
    )
    .is_err());
    assert_eq!(
        list_issues(&connection)
            .unwrap()
            .iter()
            .find(|issue| issue.id == first.id)
            .unwrap()
            .project,
        "Desktop app"
    );
    let moved = update_issue(
        &mut connection,
        &first.id,
        move_input,
        Some(vec![first.id.clone(), second.id]),
    );
    assert!(moved.is_err());
    assert_eq!(
        list_issues(&connection)
            .unwrap()
            .iter()
            .find(|issue| issue.id == first.id)
            .unwrap()
            .project,
        "Desktop app"
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reviewed_project_move_keeps_hierarchy_and_old_identifiers() {
    let root = std::env::temp_dir().join(format!("pm-app-reviewed-move-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    create_project(&mut connection, "Desktop app", "APP").unwrap();
    create_project(&mut connection, "Design", "DSN").unwrap();
    let parent = create_issue(&mut connection, draft("Desktop app")).unwrap();
    let mut child_draft = draft("Desktop app");
    child_draft.parent_id = Some(parent.id.clone());
    let child = create_issue(&mut connection, child_draft).unwrap();
    let preview = preview_issue_move(&connection, &parent.id, "Design").unwrap();
    assert_eq!(preview.affected_count, 2);
    assert_eq!(preview.proposed_identifier, "DSN-1");
    assert!(
        move_issue_project(&mut connection, &parent.id, "Design", "regenerate", "stale").is_err()
    );
    let moved = move_issue_project(
        &mut connection,
        &parent.id,
        "Design",
        "regenerate",
        &preview.revision,
    )
    .unwrap();
    assert_eq!(moved.len(), 2);
    let new_parent = moved.iter().find(|issue| issue.id == "DSN-1").unwrap();
    let new_child = moved
        .iter()
        .find(|issue| issue.parent_id.as_deref() == Some("DSN-1"))
        .unwrap();
    assert_eq!(new_child.project, "Design");
    assert_ne!(new_child.id, child.id);
    drop(connection);

    let reopened = open_database(&root).unwrap();
    let alias: String = reopened.query_row(
            "SELECT issue.identifier FROM issue_alias alias JOIN issue ON issue.id = alias.issue_id WHERE alias.alias = ?1",
            [&parent.id], |row| row.get(0),
        ).unwrap();
    assert_eq!(alias, new_parent.id);
    drop(reopened);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sub_issues_survive_restart_and_reject_cycles() {
    let root = std::env::temp_dir().join(format!("pm-app-issue-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let parent = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let mut child_input = draft("Unassigned");
    child_input.title = "Child task".into();
    child_input.parent_id = Some(parent.id.clone());
    let child = create_issue(&mut connection, child_input).unwrap();
    assert_eq!(child.parent_id.as_deref(), Some(parent.id.as_str()));
    let mut invalid = draft("Unassigned");
    invalid.parent_id = Some(child.id.clone());
    assert!(update_issue(&mut connection, &parent.id, invalid, None).is_err());
    let mut invalid = draft("Unassigned");
    invalid.parent_id = Some(child.id.clone());
    assert!(update_issue(&mut connection, &child.id, invalid, None).is_err());
    drop(connection);
    let connection = open_database(&root).unwrap();
    let saved = list_issues(&connection).unwrap();
    assert_eq!(
        saved
            .iter()
            .find(|item| item.id == child.id)
            .unwrap()
            .parent_id,
        Some(parent.id)
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn relation_receipt_replays_and_checks_both_issue_versions() {
    let root = std::env::temp_dir().join(format!("pm-app-relation-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let first = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let second = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let mutation = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(first.version),
        expected_versions: Default::default(),
    };

    let relation = create_issue_relation_mutation(
        &mut connection,
        &first.id,
        &second.id,
        "blocks",
        second.version,
        &mutation,
    )
    .unwrap();
    let replay = create_issue_relation_mutation(
        &mut connection,
        &first.id,
        &second.id,
        "blocks",
        second.version,
        &mutation,
    )
    .unwrap();
    assert_eq!(relation.id, replay.id);
    assert_eq!(
        list_issue_relations(&connection, &first.id).unwrap().len(),
        1
    );

    let stale = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(first.version),
        expected_versions: Default::default(),
    };
    let error = create_issue_relation_mutation(
        &mut connection,
        &first.id,
        &second.id,
        "related",
        second.version,
        &stale,
    )
    .unwrap_err();
    assert!(error.starts_with("CONFLICT:"));
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn issue_relations_reject_cycles_and_cross_workspace_links() {
    let root = std::env::temp_dir().join(format!("pm-app-relation-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let first_workspace = current_workspace_id(&connection).unwrap();
    let a = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let b = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let c = create_issue(&mut connection, draft("Unassigned")).unwrap();
    let first = create_issue_relation(&mut connection, &a.id, &b.id, "blocks").unwrap();
    create_issue_relation(&mut connection, &b.id, &c.id, "blocks").unwrap();
    assert!(create_issue_relation(&mut connection, &c.id, &a.id, "blocks").is_err());
    assert!(create_issue_relation(&mut connection, &b.id, &a.id, "blocked_by").is_err());
    assert!(create_issue_relation(&mut connection, &a.id, &a.id, "related").is_err());
    let related = create_issue_relation(&mut connection, &a.id, &c.id, "related").unwrap();
    assert!(create_issue_relation(&mut connection, &c.id, &a.id, "related").is_err());
    create_workspace(&mut connection, "Personal").unwrap();
    assert!(list_issue_relations(&connection, &a.id).is_err());
    let local = create_issue(&mut connection, draft("Unassigned")).unwrap();
    assert!(create_issue_relation(&mut connection, &local.id, &b.id, "blocks").is_err());
    switch_workspace(&connection, &first_workspace).unwrap();
    delete_issue_relation(&mut connection, &a.id, &related.id).unwrap();
    assert!(list_issue_relations(&connection, &a.id)
        .unwrap()
        .iter()
        .any(|item| item.id == first.id));
    drop(connection);
    let connection = open_database(&root).unwrap();
    assert_eq!(list_issue_relations(&connection, &a.id).unwrap().len(), 1);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}
