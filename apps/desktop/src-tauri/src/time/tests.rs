use super::*;
use crate::storage::{create_workspace, save_identity, switch_workspace};
use std::fs;

#[test]
fn manual_local_times_handle_dst_and_midnight() {
    assert!(
        resolve_manual_start("2026-03-08", "02:30", "America/New_York", None)
            .unwrap_err()
            .contains("does not exist")
    );
    assert!(
        resolve_manual_start("2026-11-01", "01:30", "America/New_York", None)
            .unwrap_err()
            .contains("ambiguous")
    );
    let earlier =
        resolve_manual_start("2026-11-01", "01:30", "America/New_York", Some("earlier")).unwrap();
    let later =
        resolve_manual_start("2026-11-01", "01:30", "America/New_York", Some("later")).unwrap();
    assert_eq!(later - earlier, 3600);
    let before_midnight =
        resolve_manual_start("2026-10-10", "23:30", "Asia/Kolkata", None).unwrap();
    let after_midnight = resolve_manual_start("2026-10-11", "00:30", "Asia/Kolkata", None).unwrap();
    assert_eq!(after_midnight - before_midnight, 3600);
}

#[test]
fn timer_checkpoint_ignores_backward_wall_clock_and_reconciles_closed_time_once() {
    let root = std::env::temp_dir().join(format!("pm-app-timer-clock-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Tester", "Workspace").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    let issue = Uuid::new_v4().to_string();
    let timestamp = now(&connection).unwrap();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'LOCAL-1', 'Timed', 'Todo', 'No priority', 0)",
            params![issue, workspace],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO time_entry(id, workspace_id, issue_id, started_at, last_seen_at,
              duration_seconds, exit_policy, sleep_policy)
             VALUES (?1, ?2, ?3, ?4, ?5, 37, 'continue', 'pause')",
            params![
                Uuid::new_v4().to_string(),
                workspace,
                issue,
                timestamp - 100,
                timestamp + 60
            ],
        )
        .unwrap();
    recover_on_start(&mut connection).unwrap();
    heartbeat(&mut connection).unwrap();
    let first: i64 = connection
        .query_row(
            "SELECT duration_seconds FROM time_entry WHERE issue_id = ?1",
            [&issue],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(first, 37);
    pause_running_on_exit(&mut connection).unwrap();
    recover_on_start(&mut connection).unwrap();
    let second: i64 = connection
        .query_row(
            "SELECT duration_seconds FROM time_entry WHERE issue_id = ?1",
            [&issue],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(second, first);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn long_and_backward_restart_gaps_wait_for_review_without_double_counting() {
    let root = std::env::temp_dir().join(format!("pm-app-timer-review-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Tester", "Workspace").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    let issue = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'LOCAL-1', 'Timed', 'Todo', 'No priority', 0)",
            params![issue, workspace],
        )
        .unwrap();
    let current_ms = now_ms(&connection).unwrap();
    let entry = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO time_entry(id, workspace_id, issue_id, started_at, started_at_ms,
               last_seen_at, last_seen_at_ms, duration_seconds, duration_ms, exit_policy)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 5, 5000, 'continue')",
            params![
                entry,
                workspace,
                issue,
                (current_ms - 90_000_500) / 1000,
                current_ms - 90_000_500,
                (current_ms - 90_000_500) / 1000,
                current_ms - 90_000_500
            ],
        )
        .unwrap();
    recover_on_start(&mut connection).unwrap();
    let notice = list_timer_recovery_notices(&connection).unwrap().remove(0);
    assert_eq!(notice.kind, "long_gap");
    assert_eq!(list_time(&connection, "LOCAL-1").unwrap().total_ms, 5000);
    let request = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    resolve_timer_recovery_notice(&mut connection, &notice.id, true, Some(&request)).unwrap();
    resolve_timer_recovery_notice(&mut connection, &notice.id, true, Some(&request)).unwrap();
    assert_eq!(
        list_time(&connection, "LOCAL-1").unwrap().total_ms,
        5000 + notice.gap_ms
    );
    assert!(resolve_timer_recovery_notice(&mut connection, &notice.id, true, None).is_err());

    let backward_entry = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO time_entry(id, workspace_id, issue_id, started_at, started_at_ms,
               last_seen_at, last_seen_at_ms, duration_seconds, duration_ms, exit_policy)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 3, 3000, 'continue')",
            params![
                backward_entry,
                workspace,
                issue,
                current_ms / 1000,
                current_ms,
                (current_ms + 60_000) / 1000,
                current_ms + 60_000
            ],
        )
        .unwrap();
    recover_on_start(&mut connection).unwrap();
    let backward = list_timer_recovery_notices(&connection).unwrap().remove(0);
    assert_eq!(backward.kind, "backward_clock");
    assert!(resolve_timer_recovery_notice(&mut connection, &backward.id, true, None).is_err());
    assert_eq!(
        list_time(&connection, "LOCAL-1").unwrap().total_ms,
        8000 + notice.gap_ms
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn workspace_time_pages_filter_and_clip_intervals() {
    let root = std::env::temp_dir().join(format!("pm-app-time-page-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    let issue = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'LOCAL-1', 'Timed', 'Todo', 'No priority', 0)",
            params![issue, workspace],
        )
        .unwrap();
    connection.execute(
            "INSERT INTO time_entry(id, workspace_id, issue_id, started_at, stopped_at, duration_seconds)
             VALUES (?1, ?2, ?3, 100, 200, 100)",
            params![Uuid::new_v4().to_string(), workspace, issue],
        ).unwrap();
    let page = workspace_time_page(
        &connection,
        Some("LOCAL-1"),
        None,
        Some(150),
        Some(250),
        None,
        1,
    )
    .unwrap();
    assert_eq!(page.total_count, 1);
    assert_eq!(page.total_seconds, 50);
    assert_eq!(page.items.len(), 1);
    connection
        .execute("UPDATE time_entry SET deleted_at = '2026-01-01'", [])
        .unwrap();
    assert_eq!(
        workspace_time_page(&connection, None, None, None, None, None, 50)
            .unwrap()
            .total_count,
        0
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn timer_start_receipt_replays_and_rejects_stale_state() {
    let root = std::env::temp_dir().join(format!("pm-app-timer-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
             VALUES (?1, ?2, 'APP-1', 'Timer', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), workspace],
        )
        .unwrap();

    let request = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(timer_state_version(&connection).unwrap()),
        expected_versions: Default::default(),
    };
    let start = |connection: &mut Connection, request: &MutationRequest| {
        mutations::apply(connection, "timer.start", &"APP-1", request, |tx, _| {
            check_timer_state(tx, request)?;
            start_timer_in_tx(tx, "APP-1", false)
        })
    };
    let first = start(&mut connection, &request).unwrap();
    let replay = start(&mut connection, &request).unwrap();
    assert_eq!(
        first.running.as_ref().unwrap().id,
        replay.running.as_ref().unwrap().id
    );
    let stale = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        ..request
    };
    assert!(start(&mut connection, &stale)
        .unwrap_err()
        .contains("CONFLICT:"));
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn manual_time_mutations_replay_once_and_check_entry_version() {
    let root = std::env::temp_dir().join(format!("pm-app-time-receipt-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Tester", "Workspace").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
                 VALUES (?1, ?2, 'LOCAL-1', 'Timed', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), workspace],
        )
        .unwrap();

    let create = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: None,
        expected_versions: Default::default(),
    };
    let added = mutations::apply(
        &mut connection,
        "time.add",
        &("LOCAL-1", 600_i64, "Work"),
        &create,
        |tx, _| add_time(tx, "LOCAL-1", 600, "Work"),
    )
    .unwrap();
    let replay: TimeSummary = mutations::apply(
        &mut connection,
        "time.add",
        &("LOCAL-1", 600_i64, "Work"),
        &create,
        |tx, _| add_time(tx, "LOCAL-1", 600, "Work"),
    )
    .unwrap();
    assert_eq!(replay.entries.len(), 1);
    let entry = &added.entries[0];
    let update = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(entry.version),
        expected_versions: Default::default(),
    };
    let updated = mutations::apply(
        &mut connection,
        "time.update",
        &(&entry.id, 900_i64, "More"),
        &update,
        |tx, _| {
            mutations::check_version(tx, "time_entry", &entry.id, &update)?;
            update_time(tx, "LOCAL-1", &entry.id, 900, "More")
        },
    )
    .unwrap();
    assert!(updated.entries[0].version > entry.version);
    let stale = MutationRequest {
        mutation_id: Uuid::new_v4().to_string(),
        expected_version: Some(entry.version),
        expected_versions: Default::default(),
    };
    let result: Result<TimeSummary, String> = mutations::apply(
        &mut connection,
        "time.update",
        &(&entry.id, 1200_i64, "Stale"),
        &stale,
        |tx, _| {
            mutations::check_version(tx, "time_entry", &entry.id, &stale)?;
            update_time(tx, "LOCAL-1", &entry.id, 1200, "Stale")
        },
    );
    assert!(result.unwrap_err().contains("CONFLICT"));
    assert_eq!(list_time(&connection, "LOCAL-1").unwrap().entries.len(), 1);
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn timers_switch_and_manual_entries_survive_restart() {
    let root = std::env::temp_dir().join(format!("pm-app-time-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    for (index, identifier) in ["APP-1", "APP-2"].iter().enumerate() {
        connection.execute(
                "INSERT INTO issue (id, workspace_id, identifier, title, status, priority, position)
                 VALUES (?1, ?2, ?3, 'Timer', 'Todo', 'No priority', ?4)",
                params![Uuid::new_v4().to_string(), workspace, identifier, index],
            ).unwrap();
    }
    start_timer(&mut connection, "APP-1").unwrap();
    start_timer(&mut connection, "APP-2").unwrap();
    let first = list_time(&connection, "APP-1").unwrap();
    assert!(first.entries[0].stopped_at.is_some());
    assert_eq!(first.running.unwrap().issue_identifier, "APP-2");
    stop_timer(&mut connection, "APP-2").unwrap();
    let summary = add_time(&mut connection, "APP-1", 3600, "Review").unwrap();
    assert_eq!(summary.total_seconds, 3600 + first.total_seconds);
    let manual_id = summary
        .entries
        .iter()
        .find(|entry| entry.note == "Review")
        .unwrap()
        .id
        .clone();
    let updated = update_time(&mut connection, "APP-1", &manual_id, 5400, "Pair review").unwrap();
    let edited = updated
        .entries
        .iter()
        .find(|entry| entry.id == manual_id)
        .unwrap();
    assert_eq!(edited.duration_seconds, 5400);
    assert_eq!(edited.elapsed_ms, 5_400_000);
    assert_eq!(edited.source, "manual");
    assert!(!edited.issue_id.is_empty());
    assert_eq!(edited.start_at, edited.started_at_ms);
    assert_eq!(edited.note, "Pair review");
    assert!(update_time(&mut connection, "APP-2", &manual_id, 60, "Wrong issue").is_err());
    let overview = list_workspace_time(&connection).unwrap();
    assert_eq!(overview.len(), 2);
    assert_eq!(overview[0].identifier, "APP-1");
    assert_eq!(overview[0].completed_seconds, 5400 + first.total_seconds);
    assert_eq!(overview[0].entry_count, 2);
    create_workspace(&mut connection, "Second workspace").unwrap();
    assert!(list_workspace_time(&connection).unwrap().is_empty());
    switch_workspace(&connection, &workspace).unwrap();
    drop(connection);
    let connection = open_database(&root).unwrap();
    assert_eq!(list_time(&connection, "APP-1").unwrap().entries.len(), 2);
    assert_eq!(
        list_time(&connection, "APP-1")
            .unwrap()
            .entries
            .iter()
            .find(|entry| entry.id == manual_id)
            .unwrap()
            .duration_seconds,
        5400
    );
    drop(connection);
    let mut connection = open_database(&root).unwrap();
    start_timer(&mut connection, "APP-1").unwrap();
    assert_eq!(
        running_timer(&connection)
            .unwrap()
            .unwrap()
            .issue_identifier,
        "APP-1"
    );
    pause_running_on_exit(&mut connection).unwrap();
    assert!(list_time(&connection, "APP-1").unwrap().running.is_none());
    assert!(running_timer(&connection).unwrap().is_none());
    connection
        .execute(
            "INSERT INTO workspace_preferences
                   (workspace_id, theme, issue_scope, issue_layout, board_group_by,
                    group_issues_by_status, show_issue_priority, show_issue_project, timer_on_exit)
                 VALUES (?1, 'dark', 'active', 'list', 'status', 1, 1, 1, 'continue')",
            [&workspace],
        )
        .unwrap();
    start_timer(&mut connection, "APP-1").unwrap();
    pause_running_on_exit(&mut connection).unwrap();
    assert!(list_time(&connection, "APP-1").unwrap().running.is_some());
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn interrupted_and_sleeping_timers_follow_workspace_exit_policy() {
    let root = std::env::temp_dir().join(format!("pm-app-time-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    connection
        .execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
                 VALUES (?1, ?2, 'APP-1', 'Timer', 'Todo', 'No priority', 0)",
            params![Uuid::new_v4().to_string(), workspace],
        )
        .unwrap();

    assert_eq!(recover_on_start(&mut connection).unwrap(), 0);
    start_timer(&mut connection, "APP-1").unwrap();
    connection
        .execute(
            "UPDATE time_entry SET started_at = unixepoch('now') - 600,
                 last_seen_at = unixepoch('now') - 300, duration_seconds = 300
                 WHERE stopped_at IS NULL",
            [],
        )
        .unwrap();
    assert_eq!(heartbeat(&mut connection).unwrap(), 1);
    let slept = list_time(&connection, "APP-1").unwrap();
    assert!(slept.running.is_none());
    assert_eq!(slept.total_seconds, 300);

    start_timer(&mut connection, "APP-1").unwrap();
    connection
        .execute(
            "UPDATE time_entry SET started_at = unixepoch('now') - 600,
                 last_seen_at = unixepoch('now') - 300, duration_seconds = 300
                 WHERE stopped_at IS NULL",
            [],
        )
        .unwrap();
    assert_eq!(recover_on_start(&mut connection).unwrap(), 1);
    assert!(running_timer(&connection).unwrap().is_none());
    let recovered: i64 = connection
        .query_row("SELECT recovered_timers FROM timer_session", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(recovered, 1);

    connection
            .execute(
                "INSERT INTO workspace_preferences
                 (workspace_id, theme, issue_scope, issue_layout, board_group_by,
                  group_issues_by_status, show_issue_priority, show_issue_project, timer_on_exit, timer_on_sleep)
                 VALUES (?1, 'dark', 'active', 'list', 'status', 1, 1, 1, 'continue', 'include')",
                [&workspace],
            )
            .unwrap();
    start_timer(&mut connection, "APP-1").unwrap();
    connection
        .execute(
            "UPDATE time_entry SET last_seen_at = unixepoch('now') - 300
                 WHERE stopped_at IS NULL",
            [],
        )
        .unwrap();
    assert_eq!(heartbeat(&mut connection).unwrap(), 0);
    assert_eq!(recover_on_start(&mut connection).unwrap(), 0);
    assert!(running_timer(&connection).unwrap().is_some());
    pause_running_on_exit(&mut connection).unwrap();
    assert_eq!(recover_on_start(&mut connection).unwrap(), 0);
    assert!(running_timer(&connection).unwrap().is_some());

    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn switching_requires_confirmation_and_concurrent_mode_keeps_both() {
    let root = std::env::temp_dir().join(format!("pm-app-switch-test-{}", Uuid::new_v4()));
    let mut connection = open_database(&root).unwrap();
    save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
    let workspace = current_workspace_id(&connection).unwrap();
    for identifier in ["APP-1", "APP-2"] {
        connection
            .execute(
                "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position)
                 VALUES (?1, ?2, ?3, 'Timer', 'Todo', 'No priority', 0)",
                params![Uuid::new_v4().to_string(), workspace, identifier],
            )
            .unwrap();
    }

    start_timer(&mut connection, "APP-1").unwrap();
    assert!(start_timer_in_tx(&connection, "APP-2", false)
        .unwrap_err()
        .contains("CONFIRMATION_REQUIRED"));
    assert!(list_time(&connection, "APP-1").unwrap().entries[0]
        .stopped_at
        .is_none());
    start_timer_in_tx(&connection, "APP-2", true).unwrap();
    assert!(list_time(&connection, "APP-1").unwrap().entries[0]
        .stopped_at
        .is_some());

    connection.execute(
            "INSERT INTO workspace_preferences(workspace_id, theme, issue_scope, issue_layout, board_group_by,
             group_issues_by_status, show_issue_priority, show_issue_project, concurrent_timers)
             VALUES (?1, 'dark', 'active', 'list', 'status', 1, 1, 1, 1)",
            [&workspace],
        ).unwrap();
    start_timer_in_tx(&connection, "APP-1", false).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM time_entry WHERE workspace_id = ?1 AND stopped_at IS NULL",
                [&workspace],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        2
    );
    start_timer_in_tx(&connection, "APP-1", false).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM time_entry WHERE workspace_id = ?1 AND stopped_at IS NULL",
                [&workspace],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        2
    );
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}
