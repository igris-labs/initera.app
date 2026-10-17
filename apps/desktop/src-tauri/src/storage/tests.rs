use super::*;
use std::time::Instant;

#[test]
fn project_resources_have_individual_receipted_operations() {
    let root = std::env::temp_dir().join(format!("pm-app-resources-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Tester", "Workspace").unwrap();
    let project = create_project(&mut connection, "Desktop", "APP").unwrap();
    let request = |version| MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(version),
        expected_versions: Default::default(),
    };
    let first = ProjectResource {
        id: Uuid::new_v4().to_string(),
        title: "Spec".into(),
        url: "https://example.com/spec".into(),
    };
    let second = ProjectResource {
        id: Uuid::new_v4().to_string(),
        title: "Design".into(),
        url: "https://example.com/design".into(),
    };
    let created = change_project_resource(
        &mut connection,
        project.id.clone(),
        "create".into(),
        Some(first.clone()),
        None,
        None,
        request(project.version),
    )
    .unwrap();
    let second_request = request(created.version);
    let with_two = change_project_resource(
        &mut connection,
        project.id.clone(),
        "create".into(),
        Some(second.clone()),
        None,
        None,
        second_request.clone(),
    )
    .unwrap();
    let replay = change_project_resource(
        &mut connection,
        project.id.clone(),
        "create".into(),
        Some(second.clone()),
        None,
        None,
        second_request,
    )
    .unwrap();
    assert_eq!(replay.resources, with_two.resources);
    let reordered = change_project_resource(
        &mut connection,
        project.id.clone(),
        "reorder".into(),
        None,
        None,
        Some(vec![second.id.clone(), first.id.clone()]),
        request(with_two.version),
    )
    .unwrap();
    assert_eq!(reordered.resources[0].id, second.id);
    assert!(change_project_resource(
        &mut connection,
        project.id.clone(),
        "remove".into(),
        None,
        Some(first.id.clone()),
        None,
        request(with_two.version)
    )
    .is_err());
    let removed = change_project_resource(
        &mut connection,
        project.id.clone(),
        "remove".into(),
        None,
        Some(first.id),
        None,
        request(reordered.version),
    )
    .unwrap();
    assert_eq!(removed.resources.len(), 1);
    drop(connection);
    let connection = open_database(&root).unwrap();
    assert_eq!(
        read_project(&connection, &project.id).unwrap().resources,
        vec![second]
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn acceptance_root_requires_an_existing_marked_absolute_directory() {
    let root = std::env::temp_dir().join(format!("pm-app-acceptance-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    assert!(acceptance_data_root(&root).is_err());
    fs::write(root.join(".pm-app-acceptance-root"), b"isolated test data").unwrap();
    assert_eq!(
        acceptance_data_root(&root).unwrap(),
        root.canonicalize().unwrap()
    );
    assert!(acceptance_data_root(Path::new("relative/root")).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "Run in release mode to measure the full native fixture"]
fn full_native_performance_fixture() {
    let root = std::env::temp_dir().join(format!("pm-app-perf-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Fixture", "Performance workspace").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    let mut project_ids = Vec::with_capacity(20);
    let mut issue_ids = Vec::with_capacity(10_000);

    let tx = connection.transaction().unwrap();
    for index in 0..20 {
        let id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO project(id, workspace_id, name, identifier_prefix)
                 VALUES (?1, ?2, ?3, ?4)",
            params![
                id,
                workspace,
                format!("Performance project {index}"),
                format!("P{index:02}")
            ],
        )
        .unwrap();
        project_ids.push(id);
    }
    for index in 0..100 {
        tx.execute(
            "INSERT INTO label(id, workspace_id, name, color) VALUES (?1, ?2, ?3, '#aaaaaa')",
            params![
                Uuid::new_v4().to_string(),
                workspace,
                format!("Label {index}")
            ],
        )
        .unwrap();
    }
    for index in 0..500 {
        tx.execute(
            "INSERT INTO planning_node(id, project_id, kind, name, description, position)
                 VALUES (?1, ?2, 'feature', ?3, 'Fixture planning node', ?4)",
            params![
                Uuid::new_v4().to_string(),
                project_ids[index % 20],
                format!("Node {index}"),
                index
            ],
        )
        .unwrap();
    }
    {
        let mut insert = tx
            .prepare(
                "INSERT INTO issue(id, workspace_id, identifier, project_id, title,
                                   description, status, priority, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'Fixture description', ?6, 'High', ?7)",
            )
            .unwrap();
        for index in 0..10_000 {
            let id = Uuid::new_v4().to_string();
            insert
                .execute(params![
                    id,
                    workspace,
                    format!("P{:02}-{}", index % 20, index + 1),
                    project_ids[index % 20],
                    format!("Performance issue {}", index + 1),
                    if index % 4 == 0 { "Done" } else { "Todo" },
                    index,
                ])
                .unwrap();
            issue_ids.push(id);
        }
    }
    {
        let mut insert = tx
            .prepare(
                "INSERT INTO issue_activity(id, issue_id, actor_name, kind, body)
                 VALUES (?1, ?2, 'Fixture', ?3, ?4)",
            )
            .unwrap();
        for index in 0..80_000 {
            insert
                .execute(params![
                    Uuid::new_v4().to_string(),
                    issue_ids[index % 10_000],
                    if index < 30_000 { "comment" } else { "changed" },
                    if index < 30_000 {
                        "Fixture comment"
                    } else {
                        "Fixture change"
                    },
                ])
                .unwrap();
        }
    }
    {
        let mut insert = tx
            .prepare(
                "INSERT INTO time_entry(id, workspace_id, issue_id, started_at, stopped_at,
                                        duration_seconds)
                 VALUES (?1, ?2, ?3, 100, 160, 60)",
            )
            .unwrap();
        for id in &issue_ids {
            insert
                .execute(params![Uuid::new_v4().to_string(), workspace, id])
                .unwrap();
        }
    }
    tx.commit().unwrap();

    let mut encoded_pixel = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(1, 1)
        .write_to(&mut encoded_pixel, image::ImageFormat::Png)
        .unwrap();
    let pixel = encoded_pixel.into_inner();
    for _ in 0..100 {
        crate::media::stage_image(&root, &connection, &pixel).unwrap();
    }

    fn measure(label: &str, mut run: impl FnMut()) {
        let mut samples = Vec::with_capacity(30);
        for _ in 0..30 {
            let started = Instant::now();
            run();
            samples.push(started.elapsed().as_micros());
        }
        samples.sort_unstable();
        eprintln!(
            "{label}: p95={} µs; sorted samples={samples:?}",
            samples[28]
        );
    }

    measure("issue first page", || {
        crate::issues::issue_record_page(&connection, false, 200, None).unwrap();
    });
    measure("project index", || {
        list_projects_page_query(&connection, None, 50, "updated", None, None, None).unwrap();
    });
    measure("global issue search", || {
        crate::search::search_issues(&connection, "Performance issue 9999").unwrap();
    });
    measure("filtered issue count", || {
        connection
            .query_row(
                "SELECT count(*) FROM issue WHERE workspace_id = ?1 AND status = 'Todo'
                   AND priority = 'High' AND archived_at IS NULL",
                [&workspace],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
    });

    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preferences_receipt_replays_and_checks_snapshot_version() {
    let root = std::env::temp_dir().join(format!("pm-app-preferences-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let version = preferences_snapshot(&connection).unwrap().version;
    assert!(version <= (1_i64 << 53) - 1);
    let mut preferences = PreferencesRecord::default();
    preferences.theme = "light".into();
    let request = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(version),
        expected_versions: Default::default(),
    };
    let save = |connection: &mut Connection, request: &MutationRequest| {
        mutations::apply(
            connection,
            "preferences.save",
            &preferences,
            request,
            |tx, _| {
                if request.expected_version != Some(preferences_snapshot(tx)?.version) {
                    return Err("CONFLICT: Workspace preferences changed.".into());
                }
                save_preferences_in_tx(tx, &preferences)?;
                preferences_snapshot(tx)
            },
        )
    };
    let first = save(&mut connection, &request).unwrap();
    let replay = save(&mut connection, &request).unwrap();
    assert_eq!(first.version, replay.version);
    assert_eq!(first.preferences.theme, "light");
    let stale = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        ..request
    };
    assert!(save(&mut connection, &stale)
        .err()
        .unwrap()
        .contains("CONFLICT:"));
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn workspace_creation_receipt_does_not_duplicate_after_retry() {
    let root = std::env::temp_dir().join(format!("pm-app-workspace-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let request = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let first =
        create_workspace_with_mutation(&mut connection, "Personal", Some(&request)).unwrap();
    let replay =
        create_workspace_with_mutation(&mut connection, "Personal", Some(&request)).unwrap();
    assert_eq!(first.workspace_id, replay.workspace_id);
    assert_eq!(list_workspaces(&connection).unwrap().len(), 2);
    assert!(
        create_workspace_with_mutation(&mut connection, "Other", Some(&request))
            .err()
            .unwrap()
            .contains("IDEMPOTENCY_CONFLICT")
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn workspace_switch_receipt_replays_the_selected_workspace() {
    let root = std::env::temp_dir().join(format!("pm-app-switch-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    let original = save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let target = create_workspace(&mut connection, "Personal").unwrap();
    switch_workspace(&connection, &original.workspace_id).unwrap();
    let request = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let first =
        switch_workspace_with_mutation(&mut connection, &target.workspace_id, &request).unwrap();
    switch_workspace(&connection, &original.workspace_id).unwrap();
    let replay =
        switch_workspace_with_mutation(&mut connection, &target.workspace_id, &request).unwrap();
    assert_eq!(first.workspace_id, replay.workspace_id);
    assert_eq!(
        current_workspace_id(&connection).unwrap(),
        target.workspace_id
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn identity_receipt_replays_and_rejects_stale_settings() {
    let root = std::env::temp_dir().join(format!("pm-app-identity-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    let initial = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let created =
        save_identity_with_mutation(&mut connection, "Rahul", "Igris Labs", Some(&initial))
            .unwrap();
    assert_eq!(
        save_identity_with_mutation(&mut connection, "Rahul", "Igris Labs", Some(&initial))
            .unwrap()
            .workspace_id,
        created.workspace_id
    );
    let old_version = identity_version(&connection).unwrap();
    let update = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: old_version,
        expected_versions: Default::default(),
    };
    save_identity_with_mutation(&mut connection, "Rahul", "Design", Some(&update)).unwrap();
    assert_eq!(
        save_identity_with_mutation(&mut connection, "Rahul", "Design", Some(&update))
            .unwrap()
            .workspace_name,
        "Design"
    );
    let stale = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: old_version,
        expected_versions: Default::default(),
    };
    assert!(
        save_identity_with_mutation(&mut connection, "Rahul", "Old", Some(&stale))
            .err()
            .unwrap()
            .contains("CONFLICT:")
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn documents_database_is_created_with_media_folders() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let connection = open_database(&root).expect("database opens");
    assert!(root.join("workspace.sqlite").exists());
    assert!(root.join("media").is_dir());
    assert!(root.join("staging").is_dir());
    assert!(read_identity(&connection)
        .expect("identity query")
        .is_none());
    drop(connection);
    fs::remove_dir_all(root).expect("remove test data");
}

#[test]
fn names_have_bounded_nonempty_content() {
    assert!(validate_name("  Rahul  ").is_ok());
    assert!(validate_name("  ").is_err());
    assert!(validate_name(&"a".repeat(81)).is_err());
}

#[test]
fn identity_survives_reopen_and_rename_keeps_ids() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).expect("database opens");
    let initial = save_identity(&mut connection, "Rahul", "Igris Labs").expect("setup");
    drop(connection);

    let mut connection = open_database(&root).expect("database reopens");
    let persisted = read_identity(&connection)
        .expect("query")
        .expect("identity");
    assert_eq!(persisted.workspace_id, initial.workspace_id);
    assert_eq!(persisted.actor_id, initial.actor_id);
    assert_eq!(persisted.workspace_name, "Igris Labs");
    let renamed = save_identity(&mut connection, "Rahul V", "Product Studio").expect("rename");
    assert_eq!(renamed.workspace_id, initial.workspace_id);
    assert_eq!(renamed.actor_id, initial.actor_id);
    assert_eq!(renamed.workspace_name, "Product Studio");
    drop(connection);
    fs::remove_dir_all(root).expect("remove test data");
}

#[test]
fn project_pages_use_stable_cursor_and_reject_changed_index() {
    let root = std::env::temp_dir().join(format!("pm-app-project-page-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    create_project(&mut connection, "Bravo", "BRV").unwrap();
    create_project(&mut connection, "Alpha", "ALP").unwrap();
    create_project(&mut connection, "Charlie", "CHR").unwrap();

    let first = list_projects_page_query(&connection, None, 2, "name", None, None, None).unwrap();
    assert_eq!(first.total_count, 3);
    assert_eq!(
        first
            .items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Alpha", "Bravo"]
    );
    let second = list_projects_page_query(
        &connection,
        first.next_cursor.as_deref(),
        2,
        "name",
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(second.items[0].name, "Charlie");
    assert!(second.next_cursor.is_none());

    create_project(&mut connection, "Delta", "DLT").unwrap();
    assert!(list_projects_page_query(
        &connection,
        first.next_cursor.as_deref(),
        2,
        "name",
        None,
        None,
        None
    )
    .unwrap_err()
    .starts_with("CONFLICT:"));
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn newer_schema_is_rejected_without_downgrade() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let connection = open_database(&root).expect("database opens");
    connection
        .pragma_update(None, "user_version", 2)
        .expect("set future version");
    drop(connection);
    assert!(open_database(&root).is_err());
    let connection = Connection::open(root.join("workspace.sqlite")).expect("inspect database");
    let version: i32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("read version");
    assert_eq!(version, 2);
    drop(connection);
    fs::remove_dir_all(root).expect("remove test data");
}

#[test]
fn project_trash_cascades_and_restores_its_own_descendants() {
    let root = std::env::temp_dir().join(format!("pm-app-project-lifecycle-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    let identity = save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let project = create_project(&mut connection, "Desktop app", "APP").unwrap();
    let milestone_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO milestone(id, project_id, title, sort_order) VALUES (?1, ?2, 'Launch', 0)",
            params![milestone_id, project.id],
        )
        .unwrap();
    let issue_id = Uuid::new_v4().to_string();
    connection.execute(
            "INSERT INTO issue(id, workspace_id, identifier, project_id, milestone_id, title, status, priority, position)
             VALUES (?1, ?2, 'APP-1', ?3, ?4, 'Ship', 'Todo', 'High', 0)",
            params![issue_id, identity.workspace_id, project.id, milestone_id],
        ).unwrap();

    let preview = preview_project_lifecycle(&connection, &project.id).unwrap();
    assert_eq!(preview.active_issues, 1);
    connection.execute(
            "INSERT INTO time_entry(id, workspace_id, issue_id, started_at) VALUES (?1, ?2, ?3, 100)",
            params![Uuid::new_v4().to_string(), identity.workspace_id, issue_id],
        ).unwrap();
    assert!(
        set_project_lifecycle(&mut connection, &project.id, "archive")
            .unwrap_err()
            .contains("Pause")
    );
    connection
        .execute(
            "UPDATE time_entry SET stopped_at = 101 WHERE issue_id = ?1",
            [&issue_id],
        )
        .unwrap();
    let cycle_id = Uuid::new_v4().to_string();
    connection.execute(
            "INSERT INTO cycle(id, workspace_id, title, start_date, end_date, status, project_scope_json)
             VALUES (?1, ?2, 'Launch week', '2026-10-01', '2026-10-08', 'planned', json_array(?3))",
            params![cycle_id, identity.workspace_id, project.id],
        ).unwrap();
    assert!(
        set_project_lifecycle(&mut connection, &project.id, "archive")
            .unwrap_err()
            .contains("cycle")
    );
    connection
        .execute(
            "UPDATE cycle SET status = 'completed' WHERE id = ?1",
            [&cycle_id],
        )
        .unwrap();

    set_project_lifecycle(&mut connection, &project.id, "archive").unwrap();
    assert!(list_projects(&connection).unwrap().is_empty());
    assert_eq!(
        list_inactive_projects(&connection).unwrap()[0].state,
        "archived"
    );
    set_project_lifecycle(&mut connection, &project.id, "trash").unwrap();
    assert_eq!(
        list_inactive_projects(&connection).unwrap()[0].state,
        "trash"
    );
    let trashed: i64 = connection
        .query_row(
            "SELECT count(*) FROM issue WHERE id = ?1 AND deleted_at IS NOT NULL",
            [&issue_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(trashed, 1);
    set_project_lifecycle(&mut connection, &project.id, "restore").unwrap();
    assert_eq!(
        list_projects(&connection).unwrap()[0].milestones[0].id,
        milestone_id
    );
    let linked: i64 = connection
        .query_row(
            "SELECT count(*) FROM issue WHERE project_id = ?1 AND milestone_id = ?2",
            params![project.id, milestone_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(linked, 1);
    let restored: i64 = connection
        .query_row(
            "SELECT count(*) FROM issue WHERE id = ?1 AND deleted_at IS NULL",
            [&issue_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(restored, 1);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_mutation_receipt_replays_and_checks_version() {
    let root = std::env::temp_dir().join(format!("pm-app-project-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let draft = ProjectInput {
        name: "Launch".into(),
        name_markdown: String::new(),
        identifier_prefix: "LCH".into(),
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
    };
    let create = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let project =
        create_project_full_with_mutation(&mut connection, draft.clone(), Some(create.clone()))
            .unwrap();
    let replayed =
        create_project_full_with_mutation(&mut connection, draft.clone(), Some(create)).unwrap();
    assert_eq!(project.id, replayed.id);
    assert_eq!(project.version, 1);
    let mut changed = draft;
    changed.summary = "A release".into();
    let update = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(1),
        expected_versions: Default::default(),
    };
    let saved = update_project_full_with_mutation(
        &mut connection,
        &project.id,
        changed.clone(),
        Some(update.clone()),
    )
    .unwrap();
    assert_eq!(saved.version, 2);
    let replayed = update_project_full_with_mutation(
        &mut connection,
        &project.id,
        changed.clone(),
        Some(update),
    )
    .unwrap();
    assert_eq!(saved.version, replayed.version);
    let stale = update_project_full_with_mutation(
        &mut connection,
        &project.id,
        changed,
        Some(MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: Some(1),
            expected_versions: Default::default(),
        }),
    );
    assert!(stale.unwrap_err().contains("CONFLICT"));
    let archive = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(saved.version),
        expected_versions: Default::default(),
    };
    set_project_lifecycle_with_mutation(
        &mut connection,
        &project.id,
        "archive",
        Some(archive.clone()),
    )
    .unwrap();
    set_project_lifecycle_with_mutation(&mut connection, &project.id, "archive", Some(archive))
        .unwrap();
    assert_eq!(list_inactive_projects(&connection).unwrap().len(), 1);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn removed_milestone_clears_links_and_restores_only_eligible_issues() {
    let root = std::env::temp_dir().join(format!("pm-app-milestone-restore-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    let identity = save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let project = create_project(&mut connection, "Desktop app", "APP").unwrap();
    let other = create_project(&mut connection, "Other", "OTH").unwrap();
    let old_milestone = Uuid::new_v4().to_string();
    let replacement = Uuid::new_v4().to_string();
    for id in [&old_milestone, &replacement] {
        connection.execute("INSERT INTO milestone(id, project_id, title, sort_order) VALUES (?1, ?2, 'Launch', 0)", params![id, project.id]).unwrap();
    }
    let issue_ids = (0..3).map(|index| {
            let id = Uuid::new_v4().to_string();
            connection.execute(
                "INSERT INTO issue(id, workspace_id, identifier, project_id, milestone_id, title, status, priority, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'Ship', 'Todo', 'High', ?6)",
                params![id, identity.workspace_id, format!("APP-{}", index + 1), project.id, old_milestone, index],
            ).unwrap();
            id
        }).collect::<Vec<_>>();
    let input = ProjectInput {
        name: project.name.clone(),
        name_markdown: String::new(),
        identifier_prefix: "APP".into(),
        icon: project.icon.clone(),
        color: project.color.clone(),
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
        milestones: vec![MilestoneInput {
            id: Some(replacement.clone()),
            title: "Launch".into(),
            description: String::new(),
            target_date: None,
        }],
    };
    update_project_full(&mut connection, &project.id, input).unwrap();
    let linked: i64 = connection
        .query_row(
            "SELECT count(*) FROM issue WHERE milestone_id = ?1",
            [&old_milestone],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(linked, 0);
    connection
        .execute(
            "UPDATE issue SET milestone_id = ?1 WHERE id = ?2",
            params![replacement, issue_ids[1]],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE issue SET project_id = ?1 WHERE id = ?2",
            params![other.id, issue_ids[2]],
        )
        .unwrap();
    let request = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(read_project(&connection, &project.id).unwrap().version),
        expected_versions: Default::default(),
    };
    let restore = |connection: &mut Connection, request: &MutationRequest| {
        mutations::apply(
            connection,
            "milestone.restore",
            &(&project.id, &old_milestone, true),
            request,
            |tx, workspace_id| {
                mutations::check_version(tx, "project", &project.id, request)?;
                let result =
                    restore_milestone_in_tx(tx, workspace_id, &project.id, &old_milestone, true)?;
                tx.execute(
                    "UPDATE project SET version = version + 1 WHERE id = ?1",
                    [&project.id],
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
    };
    let report = restore(&mut connection, &request).unwrap();
    assert_eq!(report.reattached, 1);
    assert_eq!(report.skipped, 2);
    assert_eq!(restore(&mut connection, &request).unwrap().reattached, 1);
    let stale = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        ..request
    };
    assert!(restore(&mut connection, &stale)
        .err()
        .unwrap()
        .contains("CONFLICT:"));
    let restored: String = connection
        .query_row(
            "SELECT milestone_id FROM issue WHERE id = ?1",
            [&issue_ids[0]],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(restored, old_milestone);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn projects_survive_reopen_and_reject_duplicate_names_and_prefixes() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).expect("database opens");
    assert!(create_project(&mut connection, "Too early", "EARLY").is_err());
    let identity = save_identity(&mut connection, "Rahul", "Igris Labs").expect("setup");
    let project = create_project(&mut connection, "Desktop app", "app").expect("create");
    assert_eq!(project.workspace_id, identity.workspace_id);
    assert_eq!(project.identifier_prefix, "APP");
    assert!(create_project(&mut connection, "desktop APP", "OTHER").is_err());
    assert!(create_project(&mut connection, "Another", "App").is_err());
    assert!(create_project(&mut connection, "Wrong", "A-1").is_err());
    drop(connection);
    let connection = open_database(&root).expect("database reopens");
    let projects = list_projects(&connection).expect("projects load");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].id, project.id);
    drop(connection);
    fs::remove_dir_all(root).expect("remove test data");
}

#[test]
fn workspaces_keep_projects_separate_and_selection_survives_reopen() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    let first = save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    create_project(&mut connection, "Desktop app", "APP").unwrap();
    connection
        .execute(
            "INSERT INTO issue (id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'APP-1', 'First task', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), first.workspace_id],
        )
        .unwrap();
    let second = create_workspace(&mut connection, "Personal").unwrap();
    assert_ne!(first.workspace_id, second.workspace_id);
    assert_eq!(first.actor_id, second.actor_id);
    assert!(list_projects(&connection).unwrap().is_empty());
    create_project(&mut connection, "Desktop app", "APP").unwrap();
    connection
        .execute(
            "INSERT INTO issue (id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'APP-1', 'Second task', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), second.workspace_id],
        )
        .unwrap();
    assert_eq!(list_projects(&connection).unwrap().len(), 1);
    assert!(switch_workspace(&connection, "unknown").is_err());
    let restored = switch_workspace(&connection, &first.workspace_id).unwrap();
    assert_eq!(restored.workspace_name, "Igris Labs");
    assert_eq!(list_projects(&connection).unwrap().len(), 1);
    assert_eq!(list_workspaces(&connection).unwrap().len(), 2);
    drop(connection);
    let reopened = open_database(&root).unwrap();
    assert_eq!(current_workspace_id(&reopened).unwrap(), first.workspace_id);
    drop(reopened);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_updates_survive_reopen_and_reject_invalid_writes() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let project = create_project(&mut connection, "Desktop app", "APP").unwrap();
    assert!(create_project_update(&connection, &project.id, "  ").is_err());
    assert!(create_project_update(&connection, "missing", "Update").is_err());
    let mutation = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let saved = create_project_update_with_mutation(
        &mut connection,
        &project.id,
        "**Release** is ready",
        &mutation,
    )
    .unwrap();
    let repeated = create_project_update_with_mutation(
        &mut connection,
        &project.id,
        "**Release** is ready",
        &mutation,
    )
    .unwrap();
    assert_eq!(repeated.id, saved.id);
    assert!(create_project_update_with_mutation(
        &mut connection,
        &project.id,
        "Different update",
        &mutation,
    )
    .unwrap_err()
    .contains("IDEMPOTENCY_CONFLICT"));
    assert_eq!(saved.actor_name, "Rahul");
    assert_eq!(saved.body, "**Release** is ready");
    assert_eq!(saved.kind, "update");
    drop(connection);
    let connection = open_database(&root).unwrap();
    let updates = list_project_updates(&connection, &project.id, 0).unwrap();
    assert_eq!(updates[0].kind, "update");
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].id, saved.id);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_activity_pages_keep_order_and_workspace_scope() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    let first_workspace = save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let project = create_project(&mut connection, "Desktop app", "APP").unwrap();

    for index in 0..35 {
        create_project_update(&connection, &project.id, &format!("Update {index}")).unwrap();
    }

    let first_page = list_project_updates(&connection, &project.id, 0).unwrap();
    let second_page = list_project_updates(&connection, &project.id, 30).unwrap();
    assert_eq!(first_page.len(), 30);
    assert_eq!(second_page.len(), 5);
    assert_eq!(first_page[0].body, "Update 34");
    assert_eq!(second_page[0].body, "Update 4");
    assert_eq!(second_page[4].body, "Update 0");
    let first_cursor_page = project_update_page(&connection, &project.id, None).unwrap();
    let second_cursor_page = project_update_page(
        &connection,
        &project.id,
        first_cursor_page.next_cursor.as_deref(),
    )
    .unwrap();
    assert_eq!(first_cursor_page.items.len(), 30);
    assert_eq!(second_cursor_page.items.len(), 5);
    assert!(second_cursor_page.next_cursor.is_none());
    assert!(list_project_updates(&connection, &project.id, -1).is_err());

    create_workspace(&mut connection, "Personal").unwrap();
    assert!(list_project_updates(&connection, &project.id, 0).is_err());
    switch_workspace(&connection, &first_workspace.workspace_id).unwrap();
    assert_eq!(
        list_project_updates(&connection, &project.id, 30)
            .unwrap()
            .len(),
        5
    );

    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_and_initial_milestones_commit_together_and_survive_restart() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let draft = ProjectInput {
        name: "Launch".into(),
        name_markdown: "**Launch**".into(),
        identifier_prefix: "LCH".into(),
        icon: "i-tabler-rocket".into(),
        color: "blue".into(),
        summary: "Ship the release".into(),
        description: "## Release brief".into(),
        description_document: Some(serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "content": [
                {"type": "heading", "attrs": {"level": 2}, "content": [
                    {"type": "text", "text": "Release brief"}
                ]}
            ]}
        })),
        status: "Planned".into(),
        priority: "High".into(),
        lead_actor_id: None,
        start_date: Some("2026-10-01".into()),
        target_date: Some("2026-10-31".into()),
        labels: vec!["Workflow".into()],
        resources: vec![ProjectResource {
            id: "brief".into(),
            title: "Release brief".into(),
            url: "https://example.com/brief".into(),
        }],
        milestones: vec![
            MilestoneInput {
                id: None,
                title: "Beta".into(),
                description: "Private release".into(),
                target_date: Some("2026-10-15".into()),
            },
            MilestoneInput {
                id: None,
                title: "Public".into(),
                description: String::new(),
                target_date: None,
            },
        ],
    };
    let mut invalid_lead = draft.clone();
    invalid_lead.lead_actor_id = Some(Uuid::new_v4().to_string());
    assert!(create_project_full(&mut connection, invalid_lead).is_err());
    let project = create_project_full(&mut connection, draft).unwrap();
    assert_eq!(project.milestones.len(), 2);
    assert_eq!(
        project.milestones[0].target_date.as_deref(),
        Some("2026-10-15")
    );
    drop(connection);
    let connection = open_database(&root).unwrap();
    let saved = list_projects(&connection).unwrap();
    assert_eq!(saved[0].id, project.id);
    assert_eq!(saved[0].summary, "Ship the release");
    assert_eq!(saved[0].name_markdown, "**Launch**");
    assert_eq!(saved[0].icon, "i-tabler-rocket");
    assert_eq!(saved[0].color, "blue");
    assert_eq!(saved[0].description, "## Release brief");
    assert_eq!(
        saved[0].description_document.as_ref().unwrap()["version"],
        1
    );
    assert_eq!(saved[0].labels, vec!["Workflow"]);
    assert_eq!(saved[0].resources[0].title, "Release brief");
    assert_eq!(saved[0].milestones[0].description, "Private release");
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_edits_keep_milestone_ids_and_roll_back_invalid_changes() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    let identity = save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let draft = ProjectInput {
        name: "Launch".into(),
        name_markdown: "Launch".into(),
        identifier_prefix: "LCH".into(),
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
        milestones: vec![MilestoneInput {
            id: None,
            title: "Beta".into(),
            description: String::new(),
            target_date: None,
        }],
    };
    let created = create_project_full(&mut connection, draft.clone()).unwrap();
    let milestone_id = created.milestones[0].id.clone();
    let mut edited = draft;
    edited.name = "Public launch".into();
    edited.lead_actor_id = Some(identity.actor_id.clone());
    edited.summary = "Release plan".into();
    edited.labels = vec!["Core".into(), "Workflow".into()];
    edited.resources = vec![ProjectResource {
        id: "roadmap".into(),
        title: "Roadmap".into(),
        url: "https://example.com/roadmap".into(),
    }];
    edited.milestones[0].id = Some(milestone_id.clone());
    edited.milestones[0].title = "Private beta".into();
    edited.milestones.push(MilestoneInput {
        id: None,
        title: "General availability".into(),
        description: String::new(),
        target_date: None,
    });
    let updated = update_project_full(&mut connection, &created.id, edited.clone()).unwrap();
    assert_eq!(updated.milestones[0].id, milestone_id);
    assert_eq!(updated.milestones.len(), 2);
    assert_eq!(
        updated.lead_actor_id.as_deref(),
        Some(identity.actor_id.as_str())
    );
    let changes = list_project_updates(&connection, &created.id, 0).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].kind, "change");
    assert!(changes[0]
        .body
        .contains("renamed project from Launch to Public launch"));
    assert!(changes[0].body.contains("changed labels"));
    assert!(changes[0].body.contains("assigned the project lead"));
    assert!(changes[0].body.contains("changed milestones"));
    let mut invalid_lead = edited.clone();
    invalid_lead.lead_actor_id = Some(Uuid::new_v4().to_string());
    assert!(update_project_full(&mut connection, &created.id, invalid_lead).is_err());
    edited.milestones[0].id = Some(Uuid::new_v4().to_string());
    assert!(update_project_full(&mut connection, &created.id, edited).is_err());
    assert_eq!(
        list_project_updates(&connection, &created.id, 0)
            .unwrap()
            .len(),
        1
    );
    drop(connection);
    let connection = open_database(&root).unwrap();
    let saved = read_project(&connection, &created.id).unwrap();
    assert_eq!(saved.name, "Public launch");
    assert_eq!(saved.summary, "Release plan");
    assert_eq!(saved.labels, vec!["Core", "Workflow"]);
    assert_eq!(saved.resources[0].title, "Roadmap");
    assert_eq!(saved.milestones[0].id, milestone_id);
    assert_eq!(
        saved.lead_actor_id.as_deref(),
        Some(identity.actor_id.as_str())
    );
    assert_eq!(
        list_project_updates(&connection, &created.id, 0)
            .unwrap()
            .len(),
        1
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_milestone_or_date_does_not_create_project() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let mut draft = ProjectInput {
        name: "Launch".into(),
        name_markdown: String::new(),
        identifier_prefix: "LCH".into(),
        icon: default_project_icon(),
        color: default_project_color(),
        summary: String::new(),
        description: String::new(),
        description_document: None,
        status: "Backlog".into(),
        priority: "No priority".into(),
        lead_actor_id: None,
        start_date: Some("2026-02-30".into()),
        target_date: None,
        labels: Vec::new(),
        resources: Vec::new(),
        milestones: Vec::new(),
    };
    assert!(create_project_full(&mut connection, draft).is_err());
    draft = ProjectInput {
        name: "Launch".into(),
        name_markdown: String::new(),
        identifier_prefix: "LCH".into(),
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
        milestones: vec![MilestoneInput {
            id: None,
            title: " ".into(),
            description: String::new(),
            target_date: None,
        }],
    };
    assert!(create_project_full(&mut connection, draft).is_err());
    assert!(list_projects(&connection).unwrap().is_empty());
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preferences_survive_reopen_and_reject_invalid_values() {
    let root = std::env::temp_dir().join(format!("pm-app-storage-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).expect("database opens");
    assert!(load_preferences(&connection).is_err());
    save_identity(&mut connection, "Rahul", "Igris Labs").expect("setup");
    assert_eq!(
        load_preferences(&connection).unwrap(),
        PreferencesRecord::default()
    );

    let preferences = PreferencesRecord {
        theme: "light".into(),
        issue_scope: "all".into(),
        issue_layout: "board".into(),
        board_group_by: "priority".into(),
        group_issues_by_status: false,
        show_issue_priority: false,
        show_issue_project: true,
        estimate_scale: "tshirt".into(),
        timer_on_exit: "continue".into(),
        concurrent_timers: false,
        timer_on_sleep: "pause".into(),
    };
    save_preferences(&mut connection, &preferences).expect("save");
    let mut invalid = preferences.clone();
    invalid.theme = "unknown".into();
    assert!(save_preferences(&mut connection, &invalid).is_err());
    drop(connection);

    let connection = open_database(&root).expect("database reopens");
    assert_eq!(load_preferences(&connection).unwrap(), preferences);
    drop(connection);
    fs::remove_dir_all(root).expect("remove test data");
}

#[test]
fn disabling_concurrent_timers_requires_resolution() {
    let root = std::env::temp_dir().join(format!("pm-app-timer-pref-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    for identifier in ["APP-1", "APP-2"] {
        let issue = Uuid::new_v4().to_string();
        connection
            .execute(
                "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
                 VALUES (?1, ?2, ?3, 'Timer', 'Todo', 'No priority', 0)",
                params![issue, workspace, identifier],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO time_entry(id, workspace_id, issue_id, started_at, last_seen_at)
                 VALUES (?1, ?2, ?3, unixepoch('now'), unixepoch('now'))",
                params![Uuid::new_v4().to_string(), workspace, issue],
            )
            .unwrap();
    }
    let mut preferences = PreferencesRecord::default();
    preferences.concurrent_timers = true;
    save_preferences(&mut connection, &preferences).unwrap();
    preferences.concurrent_timers = false;
    assert!(save_preferences(&mut connection, &preferences)
        .unwrap_err()
        .contains("CONFIRMATION_REQUIRED"));
    connection
        .execute(
            "UPDATE time_entry SET stopped_at = unixepoch('now') WHERE issue_id =
             (SELECT id FROM issue WHERE identifier = 'APP-2' AND workspace_id = ?1)",
            [&workspace],
        )
        .unwrap();
    save_preferences(&mut connection, &preferences).unwrap();
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}
