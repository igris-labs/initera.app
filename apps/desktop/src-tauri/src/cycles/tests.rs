use super::*;
use crate::storage::{create_workspace, open_database, save_identity, switch_workspace};
use std::fs;

#[test]
fn cycle_create_and_update_receipts_replay_and_check_version() {
    let root = std::env::temp_dir().join(format!("pm-app-cycle-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let create = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let make = |connection: &mut Connection| {
        mutations::apply(
            connection,
            "cycle.create",
            &("Sprint", "2030-01-01", "2030-01-07"),
            &create,
            |tx, _| {
                create_cycle_timed(
                    tx,
                    "Sprint",
                    "2030-01-01",
                    "2030-01-07",
                    &[],
                    "UTC",
                    "00:00",
                    "00:00",
                    "ask",
                    false,
                )
            },
        )
    };
    let first = make(&mut connection).unwrap();
    assert_eq!(first.id, make(&mut connection).unwrap().id);
    assert!(first.version <= (1_i64 << 53) - 1);

    let update = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(first.version),
        expected_versions: Default::default(),
    };
    let edit = |connection: &mut Connection, request: &MutationRequest| {
        mutations::apply(
            connection,
            "cycle.update",
            &(&first.id, "Renamed"),
            request,
            |tx, _| {
                if request.expected_version != Some(cycle_version(tx, &first.id)?) {
                    return Err("CONFLICT: Cycle changed.".into());
                }
                update_cycle_timed(
                    tx,
                    &first.id,
                    "Renamed",
                    "",
                    "2030-01-01",
                    "2030-01-07",
                    &[],
                    "UTC",
                    "00:00",
                    "00:00",
                    "ask",
                    false,
                    None,
                )
            },
        )
    };
    let renamed = edit(&mut connection, &update).unwrap();
    assert_eq!(renamed.id, edit(&mut connection, &update).unwrap().id);
    let stale = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        ..update
    };
    assert!(edit(&mut connection, &stale)
        .err()
        .unwrap()
        .contains("CONFLICT:"));
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cycle_assignment_receipt_replays_and_checks_membership_version() {
    let root = std::env::temp_dir().join(format!(
        "pm-app-cycle-assignment-receipt-{}",
        Uuid::new_v4()
    ));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    let cycle = create_cycle(&mut connection, "Sprint", "2030-01-01", "2030-01-07").unwrap();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'LOCAL-1', 'Task', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), workspace],
        )
        .unwrap();
    let request = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(cycle.version),
        expected_versions: Default::default(),
    };
    let assign = |connection: &mut Connection, request: &MutationRequest| {
        mutations::apply(
            connection,
            "cycle.issue.set",
            &(&cycle.id, "LOCAL-1", true),
            request,
            |tx, _| {
                if request.expected_version != Some(cycle_version(tx, &cycle.id)?) {
                    return Err("CONFLICT: Cycle changed.".into());
                }
                set_cycle_issue_in_tx(tx, &cycle.id, "LOCAL-1", true)
            },
        )
    };
    assign(&mut connection, &request).unwrap();
    assign(&mut connection, &request).unwrap();
    let stale = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        ..request
    };
    assert!(assign(&mut connection, &stale)
        .err()
        .unwrap()
        .contains("CONFLICT:"));
    assert_eq!(
        list_cycles(&connection)
            .unwrap()
            .into_iter()
            .find(|item| item.id == cycle.id)
            .unwrap()
            .issue_ids,
        vec!["LOCAL-1"]
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cycle_series_receipts_cover_reconcile_and_stale_toggle() {
    let root = std::env::temp_dir().join(format!("pm-app-series-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let settings = CycleSettings {
        id: String::new(),
        name: "Future".into(),
        duration_days: 7,
        interval_days: 7,
        enabled: true,
        start_date: "2030-01-01".into(),
        rollover_policy: "ask".into(),
        project_ids: vec![],
        time_zone: "UTC".into(),
        start_local_time: "09:00".into(),
    };
    let create = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let make = |connection: &mut Connection| {
        mutations::apply_with_connection(
            connection,
            "cycle.series.create",
            &settings,
            &create,
            |connection, _| configure_cycles(connection, settings.clone()),
        )
    };
    make(&mut connection).unwrap();
    make(&mut connection).unwrap();
    let series = list_cycle_series(&connection).unwrap();
    assert_eq!(series.len(), 1);
    assert_eq!(list_cycles(&connection).unwrap().len(), 12);
    let version = series_version(&series[0]).unwrap();
    let toggle = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(version),
        expected_versions: Default::default(),
    };
    let disable = |connection: &mut Connection, request: &MutationRequest| {
        mutations::apply_with_connection(
            connection,
            "cycle.series.enable",
            &(&series[0].id, false),
            request,
            |connection, _| {
                if request.expected_version
                    != Some(current_series_version(connection, &series[0].id)?)
                {
                    return Err("CONFLICT: Cycle series changed.".into());
                }
                set_cycle_series_enabled(connection, &series[0].id, false)
            },
        )
    };
    disable(&mut connection, &toggle).unwrap();
    disable(&mut connection, &toggle).unwrap();
    let stale = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        ..toggle
    };
    assert!(disable(&mut connection, &stale)
        .err()
        .unwrap()
        .contains("CONFLICT:"));
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn selective_rollover_keeps_frozen_history_after_issue_deletion() {
    let root = std::env::temp_dir().join(format!("pm-app-cycle-snapshot-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    let source = create_cycle(&mut connection, "Source", "2030-01-01", "2030-01-07").unwrap();
    let destination = create_cycle(&mut connection, "Next", "2030-01-08", "2030-01-14").unwrap();
    for identifier in ["APP-1", "APP-2"] {
        connection
            .execute(
                "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
                 VALUES (?1, ?2, ?3, 'Cycle work', 'Todo', 'No priority', 0)",
                params![Uuid::new_v4().to_string(), workspace, identifier],
            )
            .unwrap();
        set_cycle_issue(&mut connection, &source.id, identifier, true).unwrap();
    }
    complete_cycle_selected(
        &mut connection,
        &source.id,
        Some(&destination.id),
        Some(&["APP-1".into()]),
    )
    .unwrap();
    let decisions: Vec<(String, String)> = connection.prepare(
            "SELECT identifier, rollover_decision FROM cycle_snapshot_issue WHERE cycle_id = ?1 ORDER BY identifier"
        ).unwrap().query_map([&source.id], |row| Ok((row.get(0)?, row.get(1)?))).unwrap()
          .map(Result::unwrap).collect();
    assert_eq!(
        decisions,
        vec![
            ("APP-1".into(), "moved".into()),
            ("APP-2".into(), "left".into())
        ]
    );
    assert!(complete_cycle(&mut connection, &source.id, Some(&destination.id)).is_err());
    connection
        .execute("DELETE FROM issue WHERE identifier = 'APP-1'", [])
        .unwrap();
    drop(connection);

    let reopened = open_database(&root).unwrap();
    let historical = list_cycles(&reopened)
        .unwrap()
        .into_iter()
        .find(|cycle| cycle.id == source.id)
        .unwrap();
    assert_eq!(historical.issue_ids, vec!["APP-1", "APP-2"]);
    assert_eq!(historical.total_count, Some(2));
    drop(reopened);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn elapsed_manual_cycle_captures_once_and_waits_for_a_decision() {
    let root = std::env::temp_dir().join(format!("pm-app-cycle-downtime-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    let cycle = create_cycle(&mut connection, "Past", "2020-01-01", "2020-01-07").unwrap();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'APP-1', 'Past work', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), workspace],
        )
        .unwrap();
    set_cycle_issue(&mut connection, &cycle.id, "APP-1", true).unwrap();
    reconcile_current_workspace(&mut connection).unwrap();
    assert_eq!(
        list_cycles(&connection)
            .unwrap()
            .into_iter()
            .find(|item| item.id == cycle.id)
            .unwrap()
            .transition_result
            .as_deref(),
        Some("pending")
    );
    connection
        .execute(
            "UPDATE issue SET status = 'Done' WHERE identifier = 'APP-1'",
            [],
        )
        .unwrap();
    reconcile_current_workspace(&mut connection).unwrap();
    let snapshot: (String, i64) = connection
        .query_row(
            "SELECT status, completed FROM cycle_snapshot_issue WHERE cycle_id = ?1",
            [&cycle.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(snapshot, ("Todo".into(), 0));
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_scope_rejects_other_project_issues_and_persists_edits() {
    let root = std::env::temp_dir().join(format!("pm-app-cycle-scope-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace_id = current_workspace_id(&connection).unwrap();
    let desktop = Uuid::new_v4().to_string();
    let design = Uuid::new_v4().to_string();
    for (id, name, prefix) in [(&desktop, "Desktop", "APP"), (&design, "Design", "DSN")] {
        connection.execute(
                "INSERT INTO project(id, workspace_id, name, identifier_prefix) VALUES (?1, ?2, ?3, ?4)",
                params![id, workspace_id, name, prefix],
            ).unwrap();
    }
    for (id, identifier) in [(&desktop, "APP-1"), (&design, "DSN-1")] {
        connection.execute(
                "INSERT INTO issue(id, workspace_id, project_id, identifier, title, status, priority, position) VALUES (?1, ?2, ?3, ?4, 'Scoped', 'Todo', 'No priority', 0)",
                params![Uuid::new_v4().to_string(), workspace_id, id, identifier],
            ).unwrap();
    }
    let cycle = create_cycle_scoped(
        &mut connection,
        "Desktop",
        "2026-10-01",
        "2026-10-07",
        &[desktop.clone()],
    )
    .unwrap();
    set_cycle_issue(&mut connection, &cycle.id, "APP-1", true).unwrap();
    assert!(set_cycle_issue(&mut connection, &cycle.id, "DSN-1", true).is_err());
    assert!(update_cycle_scoped(
        &mut connection,
        &cycle.id,
        "Desktop",
        "",
        "2026-10-01",
        "2026-10-07",
        &[Uuid::new_v4().to_string()]
    )
    .is_err());
    let edited = update_cycle_scoped(
        &mut connection,
        &cycle.id,
        "Both",
        "",
        "2026-10-01",
        "2026-10-07",
        &[desktop, design],
    )
    .unwrap();
    assert_eq!(edited.project_ids.len(), 2);
    set_cycle_issue(&mut connection, &cycle.id, "DSN-1", true).unwrap();
    drop(connection);
    let reopened = open_database(&root).unwrap();
    let saved = list_cycles(&reopened)
        .unwrap()
        .into_iter()
        .find(|item| item.id == cycle.id)
        .unwrap();
    assert_eq!(saved.project_ids.len(), 2);
    assert_eq!(saved.issue_ids.len(), 2);
    drop(reopened);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn scheduled_rollover_obeys_automatic_leave_and_ask() {
    for policy in ["automatic", "leave", "ask"] {
        let root =
            std::env::temp_dir().join(format!("pm-app-rollover-{policy}-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
        let workspace_id = current_workspace_id(&connection).unwrap();
        let today = local_date_now("UTC").unwrap();
        let start = offset_date(&connection, &today, -2).unwrap();
        configure_cycles(
            &mut connection,
            CycleSettings {
                id: String::new(),
                name: "Main".into(),
                duration_days: 7,
                interval_days: 7,
                enabled: true,
                start_date: start.clone(),
                rollover_policy: policy.into(),
                project_ids: vec![],
                time_zone: "UTC".into(),
                start_local_time: "00:00".into(),
            },
        )
        .unwrap();
        let current = list_cycles(&connection)
            .unwrap()
            .into_iter()
            .find(|cycle| cycle.start_date == start)
            .unwrap();
        connection
            .execute(
                "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
                 VALUES (?1, ?2, 'APP-1', 'Carry me', 'Todo', 'No priority', 0)",
                params![Uuid::new_v4().to_string(), workspace_id],
            )
            .unwrap();
        set_cycle_issue(&mut connection, &current.id, "APP-1", true).unwrap();
        let yesterday = offset_date(&connection, &today, -1).unwrap();
        connection
                .execute(
                    "UPDATE cycle SET end_date = ?1, end_at = date(?1, '+1 day') || 'T00:00:00Z' WHERE id = ?2",
                    params![yesterday, current.id],
                )
                .unwrap();
        reconcile_cycles(&mut connection).unwrap();
        let cycles = list_cycles(&connection).unwrap();
        let old = cycles.iter().find(|cycle| cycle.id == current.id).unwrap();
        let next = cycles
            .iter()
            .filter(|cycle| cycle.start_date > today)
            .min_by_key(|cycle| &cycle.start_date)
            .unwrap();
        assert_eq!(old.status == "completed", policy != "ask");
        assert_eq!(
            next.issue_ids.contains(&"APP-1".to_owned()),
            policy == "automatic"
        );
        if policy == "ask" {
            complete_cycle(&mut connection, &old.id, Some(&next.id)).unwrap();
            assert!(list_cycles(&connection)
                .unwrap()
                .iter()
                .find(|cycle| cycle.id == next.id)
                .unwrap()
                .issue_ids
                .contains(&"APP-1".to_owned()));
        }
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn cycles_persist_membership_and_roll_over_only_unfinished_work() {
    let root = std::env::temp_dir().join(format!("pm-app-cycle-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace_id = current_workspace_id(&connection).unwrap();
    let current = create_cycle(&mut connection, "Week one", "2026-10-01", "2026-10-07").unwrap();
    let next = create_cycle(&mut connection, "Week two", "2026-10-08", "2026-10-14").unwrap();
    for (identifier, status, position) in [("APP-1", "Todo", 0), ("APP-2", "Done", 1)] {
        connection.execute(
                "INSERT INTO issue (id, workspace_id, identifier, title, status, priority, position)
                 VALUES (?1, ?2, ?3, 'Cycle issue', ?4, 'No priority', ?5)",
                params![Uuid::new_v4().to_string(), workspace_id, identifier, status, position],
            ).unwrap();
    }
    set_cycle_issue(&mut connection, &current.id, "APP-1", true).unwrap();
    set_cycle_issue(&mut connection, &current.id, "APP-2", true).unwrap();
    let edited = update_cycle(
        &mut connection,
        &current.id,
        "Week one",
        "First sprint",
        "2026-10-01",
        "2026-10-07",
    )
    .unwrap();
    assert_eq!(edited.description, "First sprint");
    complete_cycle(&mut connection, &current.id, Some(&next.id)).unwrap();
    connection
        .execute(
            "UPDATE issue SET status = 'Done' WHERE identifier = 'APP-1'",
            [],
        )
        .unwrap();
    let snapshot = list_cycles(&connection).unwrap();
    let completed = snapshot
        .iter()
        .find(|cycle| cycle.id == current.id)
        .unwrap();
    assert_eq!(
        (completed.completed_count, completed.total_count),
        (Some(1), Some(2))
    );
    assert!(update_cycle(
        &mut connection,
        &current.id,
        "Changed",
        "",
        "2026-10-01",
        "2026-10-07"
    )
    .is_err());
    assert_eq!(
        list_cycles(&connection)
            .unwrap()
            .iter()
            .find(|cycle| cycle.id == next.id)
            .unwrap()
            .issue_ids,
        vec!["APP-1"]
    );
    assert!(set_cycle_issue(&mut connection, &current.id, "APP-1", false).is_err());
    create_workspace(&mut connection, "Personal").unwrap();
    assert!(list_cycles(&connection).unwrap().is_empty());
    switch_workspace(&connection, &workspace_id).unwrap();
    assert_eq!(list_cycles(&connection).unwrap().len(), 2);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn schedule_creates_upcoming_cycles_once_and_rolls_active_issues_forward() {
    let root = std::env::temp_dir().join(format!("pm-app-schedule-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace_id = current_workspace_id(&connection).unwrap();
    let today = local_date_now("UTC").unwrap();
    let start = offset_date(&connection, &today, -15).unwrap();
    configure_cycles(
        &mut connection,
        CycleSettings {
            id: String::new(),
            name: "Main".into(),
            duration_days: 7,
            interval_days: 7,
            enabled: true,
            start_date: start.clone(),
            rollover_policy: "automatic".into(),
            project_ids: vec![],
            time_zone: "UTC".into(),
            start_local_time: "00:00".into(),
        },
    )
    .unwrap();
    let series_id = list_cycle_series(&connection).unwrap()[0].id.clone();
    let previous_id = Uuid::new_v4().to_string();
    let previous_start = offset_date(&connection, &start, 7).unwrap();
    let previous_end = offset_date(&connection, &previous_start, 6).unwrap();
    let previous_end_at = format!(
        "{}T00:00:00Z",
        offset_date(&connection, &previous_end, 1).unwrap()
    );
    connection.execute(
            "INSERT INTO cycle (id, workspace_id, title, start_date, end_date, schedule_index,
             series_id, project_scope_json, rollover_policy, time_zone, start_local_time,
             end_local_time, start_at, end_at)
             VALUES (?1, ?2, 'Main 2', ?3, ?4, 1, ?5, '[]', 'automatic', 'UTC', '00:00', '00:00', ?6, ?7)",
            params![previous_id, workspace_id, previous_start, previous_end, series_id,
                    format!("{previous_start}T00:00:00Z"), previous_end_at],
        ).unwrap();
    let cycles = list_cycles(&connection).unwrap();
    assert_eq!(
        cycles
            .iter()
            .filter(|cycle| cycle.start_date > today)
            .count(),
        12
    );
    assert_eq!(
        cycles
            .iter()
            .filter(|cycle| cycle.status == "completed")
            .count(),
        0
    );
    let previous = cycles.iter().find(|cycle| cycle.title == "Main 2").unwrap();
    let current = cycles.iter().find(|cycle| cycle.title == "Main 3").unwrap();
    connection
        .execute(
            "INSERT INTO issue (id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'APP-1', 'Unfinished', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), workspace_id],
        )
        .unwrap();
    connection.execute(
            "INSERT INTO cycle_issue (cycle_id, issue_id) SELECT ?1, id FROM issue WHERE identifier = 'APP-1'",
            [&previous.id],
        ).unwrap();
    reconcile_cycles(&mut connection).unwrap();
    reconcile_cycles(&mut connection).unwrap();
    let refreshed = list_cycles(&connection).unwrap();
    assert_eq!(refreshed.len(), cycles.len());
    assert_eq!(
        refreshed
            .iter()
            .find(|cycle| cycle.id == current.id)
            .unwrap()
            .issue_ids,
        vec!["APP-1"]
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn editing_schedule_keeps_current_cycle_and_future_membership() {
    let root = std::env::temp_dir().join(format!("pm-app-schedule-edit-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    let today = local_date_now("UTC").unwrap();
    let start = offset_date(&connection, &today, -2).unwrap();
    configure_cycles(
        &mut connection,
        CycleSettings {
            id: String::new(),
            name: "Main".into(),
            duration_days: 7,
            interval_days: 7,
            enabled: true,
            start_date: start,
            rollover_policy: "automatic".into(),
            project_ids: vec![],
            time_zone: "UTC".into(),
            start_local_time: "00:00".into(),
        },
    )
    .unwrap();
    let before = list_cycles(&connection).unwrap();
    let active = before
        .iter()
        .find(|cycle| cycle.status == "active")
        .unwrap();
    let future = before
        .iter()
        .filter(|cycle| cycle.status == "planned")
        .min_by_key(|cycle| &cycle.start_date)
        .unwrap();
    connection
        .execute(
            "INSERT INTO issue (id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'APP-1', 'Scheduled', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), workspace],
        )
        .unwrap();
    set_cycle_issue(&mut connection, &future.id, "APP-1", true).unwrap();
    let next_start = offset_date(&connection, &active.end_date, 3).unwrap();
    let series_id = list_cycle_series(&connection).unwrap()[0].id.clone();
    update_cycle_schedule(
        &mut connection,
        CycleSettings {
            id: series_id,
            name: "Main".into(),
            duration_days: 14,
            interval_days: 16,
            enabled: true,
            start_date: next_start.clone(),
            rollover_policy: "automatic".into(),
            project_ids: vec![],
            time_zone: "UTC".into(),
            start_local_time: "00:00".into(),
        },
    )
    .unwrap();
    let after = list_cycles(&connection).unwrap();
    let same_active = after.iter().find(|cycle| cycle.id == active.id).unwrap();
    assert_eq!(same_active.end_date, active.end_date);
    let same_future = after.iter().find(|cycle| cycle.id == future.id).unwrap();
    assert_eq!(same_future.start_date, next_start);
    assert_eq!(same_future.issue_ids, vec!["APP-1"]);
    assert!(
        after
            .iter()
            .filter(|cycle| cycle.status == "planned")
            .count()
            >= 3
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn day_series_stay_independent_and_skip_empty_downtime() {
    let root = std::env::temp_dir().join(format!("pm-app-cycle-series-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    for (name, duration_days) in [("Daily", 1), ("Sprint", 14)] {
        configure_cycles(
            &mut connection,
            CycleSettings {
                id: String::new(),
                name: name.into(),
                duration_days,
                interval_days: duration_days + 1,
                enabled: true,
                start_date: "2000-01-01".into(),
                rollover_policy: "automatic".into(),
                project_ids: vec![],
                time_zone: "UTC".into(),
                start_local_time: "09:00".into(),
            },
        )
        .unwrap();
    }
    let series = list_cycle_series(&connection).unwrap();
    assert_eq!(series.len(), 2);
    let now = utc_now();
    let before = list_cycles(&connection).unwrap();
    for item in &series {
        let group: Vec<_> = before
            .iter()
            .filter(|cycle| cycle.series_id.as_deref() == Some(&item.id))
            .collect();
        assert_eq!(
            group.iter().filter(|cycle| cycle.start_at > now).count(),
            12
        );
        assert!(group.len() <= 13);
        assert!(group
            .iter()
            .all(|cycle| cycle.start_date.as_str() > "2020-01-01"));
    }
    reconcile_cycles(&mut connection).unwrap();
    assert_eq!(list_cycles(&connection).unwrap().len(), before.len());
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn paused_series_and_large_catch_up_resume_without_duplicate_receipts() {
    let root = std::env::temp_dir().join(format!("pm-app-cycle-batch-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace_id = current_workspace_id(&connection).unwrap();
    configure_cycles(
        &mut connection,
        CycleSettings {
            id: String::new(),
            name: "Batch".into(),
            duration_days: 1,
            interval_days: 2,
            enabled: true,
            start_date: "2000-01-01".into(),
            rollover_policy: "leave".into(),
            project_ids: vec![],
            time_zone: "UTC".into(),
            start_local_time: "09:00".into(),
        },
    )
    .unwrap();
    let series_id = list_cycle_series(&connection).unwrap()[0].id.clone();

    for index in 0..201 {
        let start = offset_date(&connection, "2000-01-01", index * 2).unwrap();
        connection
            .execute(
                "INSERT INTO cycle (id, workspace_id, title, start_date, end_date,
                 schedule_index, series_id, project_scope_json, rollover_policy,
                 time_zone, start_local_time, end_local_time, start_at, end_at)
                 VALUES (?1, ?2, 'Past', ?3, ?3, ?4, ?5, '[]', 'leave',
                         'UTC', '09:00', '09:00', ?6, ?7)",
                params![
                    Uuid::new_v4().to_string(),
                    workspace_id,
                    start,
                    index,
                    series_id,
                    format!("{start}T09:00:00Z"),
                    format!("{}T09:00:00Z", offset_date(&connection, &start, 1).unwrap())
                ],
            )
            .unwrap();
    }

    reconcile_cycles(&mut connection).unwrap();
    let progress = list_reconcile_progress(&connection).unwrap();
    assert_eq!((progress[0].processed, progress[0].total), (100, 201));
    drop(connection);
    let mut connection = open_database(&root).unwrap();
    reconcile_cycles(&mut connection).unwrap();
    assert_eq!(
        list_reconcile_progress(&connection).unwrap()[0].processed,
        200
    );
    reconcile_cycles(&mut connection).unwrap();
    assert!(list_reconcile_progress(&connection).unwrap().is_empty());
    let receipts: i64 = connection
        .query_row(
            "SELECT count(*) FROM cycle_transition WHERE source_cycle_id IN
             (SELECT id FROM cycle WHERE series_id = ?1 AND title = 'Past')",
            [&series_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(receipts, 201);

    set_cycle_series_enabled(&mut connection, &series_id, false).unwrap();
    assert!(!list_cycle_series(&connection).unwrap()[0].enabled);
    let future_id: String = connection
        .query_row(
            "SELECT id FROM cycle WHERE series_id = ?1 AND start_at > ?2
             ORDER BY schedule_index DESC LIMIT 1",
            params![series_id, utc_now()],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute("DELETE FROM cycle WHERE id = ?1", [&future_id])
        .unwrap();
    reconcile_cycles(&mut connection).unwrap();
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM cycle WHERE id = ?1)",
            [&future_id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!exists);
    set_cycle_series_enabled(&mut connection, &series_id, true).unwrap();
    let future_count: i64 = connection
        .query_row(
            "SELECT count(*) FROM cycle WHERE series_id = ?1 AND start_at > ?2",
            params![series_id, utc_now()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(future_count, 12);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn interval_uses_supported_date_range_instead_of_a_cooldown_limit() {
    let mut settings = CycleSettings {
        id: String::new(),
        name: "Long range".into(),
        duration_days: 14,
        interval_days: 4_000,
        enabled: true,
        start_date: local_date_now("UTC").unwrap(),
        rollover_policy: "ask".into(),
        project_ids: vec![],
        time_zone: "UTC".into(),
        start_local_time: "09:00".into(),
    };
    assert!(valid_settings(&settings));
    settings.interval_days = 300_000;
    assert!(!valid_settings(&settings));
}

#[test]
fn elapsed_manual_cycle_uses_its_designated_successor_once() {
    let root = std::env::temp_dir().join(format!("pm-app-manual-successor-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace_id = current_workspace_id(&connection).unwrap();
    let source = create_cycle(&mut connection, "First", "2020-01-01", "2020-01-07").unwrap();
    let successor = create_cycle(&mut connection, "Second", "2020-01-08", "2020-01-14").unwrap();
    assert!(set_cycle_successor(&connection, &successor.id, Some(&source.id)).is_err());
    set_cycle_successor(&connection, &source.id, Some(&successor.id)).unwrap();
    assert!(update_cycle_timed(
        &mut connection,
        &source.id,
        "Changed",
        "",
        "2020-01-01",
        "2020-01-09",
        &[],
        "UTC",
        "00:00",
        "00:00",
        "automatic",
        false,
        Some(&successor.id),
    )
    .is_err());
    assert_eq!(
        list_cycles(&connection)
            .unwrap()
            .iter()
            .find(|cycle| cycle.id == source.id)
            .unwrap()
            .title,
        "First"
    );
    connection
        .execute(
            "UPDATE cycle SET rollover_policy = 'automatic' WHERE id = ?1",
            [&source.id],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'APP-1', 'Carry forward', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), workspace_id],
        )
        .unwrap();
    set_cycle_issue(&mut connection, &source.id, "APP-1", true).unwrap();
    reconcile_cycles(&mut connection).unwrap();
    reconcile_cycles(&mut connection).unwrap();
    let cycles = list_cycles(&connection).unwrap();
    assert_eq!(
        cycles
            .iter()
            .find(|cycle| cycle.id == source.id)
            .unwrap()
            .status,
        "completed"
    );
    assert_eq!(
        cycles
            .iter()
            .find(|cycle| cycle.id == successor.id)
            .unwrap()
            .issue_ids,
        vec!["APP-1"]
    );
    let transitions: i64 = connection
        .query_row(
            "SELECT count(*) FROM cycle_transition WHERE source_cycle_id = ?1",
            [&source.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(transitions, 1);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn long_downtime_moves_work_past_expired_cycles() {
    let root = std::env::temp_dir().join(format!("pm-app-cycle-catch-up-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    configure_cycles(
        &mut connection,
        CycleSettings {
            id: String::new(),
            name: "Main".into(),
            duration_days: 7,
            interval_days: 7,
            enabled: true,
            start_date: "2020-01-01".into(),
            rollover_policy: "automatic".into(),
            project_ids: vec![],
            time_zone: "UTC".into(),
            start_local_time: "00:00".into(),
        },
    )
    .unwrap();
    let series = list_cycle_series(&connection).unwrap().remove(0);
    let old_ids: Vec<_> = (0..2)
        .map(|index| {
            let id = Uuid::new_v4().to_string();
            let start = offset_date(&connection, "2020-01-01", index * 7).unwrap();
            let end = offset_date(&connection, &start, 6).unwrap();
            let next = offset_date(&connection, &end, 1).unwrap();
            connection
                .execute(
                    "INSERT INTO cycle (id, workspace_id, title, start_date, end_date,
                 schedule_index, series_id, start_at, end_at, rollover_policy)
                 VALUES (?1, ?2, 'Old', ?3, ?4, ?5, ?6, ?7, ?8, 'automatic')",
                    params![
                        id,
                        workspace,
                        start,
                        end,
                        index,
                        series.id,
                        format!("{start}T00:00:00Z"),
                        format!("{next}T00:00:00Z")
                    ],
                )
                .unwrap();
            id
        })
        .collect();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'APP-1', 'Carry forward', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), workspace],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO cycle_issue(cycle_id, issue_id)
             SELECT ?1, id FROM issue WHERE identifier = 'APP-1'",
            [&old_ids[0]],
        )
        .unwrap();

    reconcile_cycles(&mut connection).unwrap();
    let cycles = list_cycles(&connection).unwrap();
    assert!(old_ids.iter().all(|id| cycles
        .iter()
        .any(|cycle| &cycle.id == id && cycle.status == "completed")));
    assert!(cycles
        .iter()
        .find(|cycle| cycle.id == old_ids[1])
        .unwrap()
        .issue_ids
        .is_empty());
    let current = cycles.iter().find(|cycle| {
        cycle.start_date.as_str() > "2020-01-14"
            && cycle.start_date <= local_date_now("UTC").unwrap()
            && cycle.status == "active"
    });
    assert_eq!(current.unwrap().issue_ids, vec!["APP-1"]);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}
