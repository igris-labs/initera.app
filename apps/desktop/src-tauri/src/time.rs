use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Tz;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use uuid::Uuid;

use crate::mutations::{self, MutationRequest};
use crate::storage::{current_workspace_id, document_data_root, open_database};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeEntry {
    id: String,
    version: i64,
    issue_id: String,
    issue_identifier: String,
    started_at: i64,
    stopped_at: Option<i64>,
    duration_seconds: i64,
    source: String,
    started_at_ms: i64,
    stopped_at_ms: Option<i64>,
    elapsed_ms: i64,
    start_at: i64,
    end_at: Option<i64>,
    running: bool,
    note: String,
    time_zone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSummary {
    entries: Vec<TimeEntry>,
    running: Option<TimeEntry>,
    total_seconds: i64,
    total_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerRecoveryNotice {
    id: String,
    issue_identifier: String,
    kind: String,
    gap_seconds: i64,
    gap_ms: i64,
    created_at: i64,
}

fn list_timer_recovery_notices(
    connection: &Connection,
) -> Result<Vec<TimerRecoveryNotice>, String> {
    let workspace = current_workspace_id(connection)?;
    let mut statement = connection.prepare(
        "SELECT notice.id, issue.identifier, notice.kind, notice.gap_seconds, notice.created_at, notice.gap_ms
         FROM timer_recovery_notice notice
         JOIN time_entry entry ON entry.id = notice.entry_id
         JOIN issue ON issue.id = entry.issue_id
         WHERE entry.workspace_id = ?1 AND notice.resolved_at IS NULL
         ORDER BY notice.created_at, notice.id"
    ).map_err(|_| "Cannot load timer recovery notices.".to_owned())?;
    let notices = statement
        .query_map([workspace], |row| {
            Ok(TimerRecoveryNotice {
                id: row.get(0)?,
                issue_identifier: row.get(1)?,
                kind: row.get(2)?,
                gap_seconds: row.get(3)?,
                created_at: row.get(4)?,
                gap_ms: row.get(5)?,
            })
        })
        .map_err(|_| "Cannot load timer recovery notices.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot load timer recovery notices.".to_owned())?;
    Ok(notices)
}

#[api_contract_macros::command]
pub fn storage_list_timer_recovery_notices(
    app: AppHandle,
) -> Result<Vec<TimerRecoveryNotice>, String> {
    list_timer_recovery_notices(&open_database(&document_data_root(&app)?)?)
}

#[api_contract_macros::command]
pub fn storage_resolve_timer_recovery_notice(
    app: AppHandle,
    id: String,
    include_gap: bool,
    mutation: MutationRequest,
) -> Result<(), String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    resolve_timer_recovery_notice(&mut connection, &id, include_gap, Some(&mutation))
}

fn resolve_timer_recovery_notice(
    connection: &mut Connection,
    id: &str,
    include_gap: bool,
    mutation: Option<&MutationRequest>,
) -> Result<(), String> {
    let workspace = current_workspace_id(&connection)?;
    let hash = mutations::request_hash("timer.recovery.resolve", &(id, include_gap))?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin timer review.".to_owned())?;
    if let Some(request) = mutation {
        if mutations::replay::<()>(&tx, &workspace, "timer.recovery.resolve", &hash, request)?
            .is_some()
        {
            return Ok(());
        }
    }
    let notice: Option<(String, String, i64)> = tx
        .query_row(
            "SELECT notice.entry_id, notice.kind, notice.gap_ms
         FROM timer_recovery_notice notice JOIN time_entry entry ON entry.id = notice.entry_id
         WHERE notice.id = ?1 AND entry.workspace_id = ?2 AND notice.resolved_at IS NULL",
            params![id, workspace],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| "Cannot read timer review.".to_owned())?;
    let Some((entry_id, kind, gap)) = notice else {
        return Err("Timer review not found.".into());
    };
    if include_gap {
        if kind != "long_gap" || gap <= 86_400_000 {
            return Err("Only a long positive gap can be included after review.".into());
        }
        tx.execute(
            "UPDATE time_entry SET duration_ms = duration_ms + ?1,
               duration_seconds = (duration_ms + ?1) / 1000 WHERE id = ?2",
            params![gap, entry_id],
        )
        .map_err(|_| "Cannot include timer gap.".to_owned())?;
    }
    tx.execute(
        "UPDATE timer_recovery_notice SET resolved_at = unixepoch('now') WHERE id = ?1",
        [id],
    )
    .map_err(|_| "Cannot finish timer review.".to_owned())?;
    if let Some(request) = mutation {
        mutations::save(
            &tx,
            &workspace,
            "timer.recovery.resolve",
            &hash,
            request,
            &(),
        )?;
    }
    tx.commit()
        .map_err(|_| "Cannot finish timer review.".to_owned())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceTimeIssue {
    identifier: String,
    title: String,
    project: String,
    completed_seconds: i64,
    entry_count: i64,
    running_started_at: Option<i64>,
    archived: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceTimeEntry {
    id: String,
    issue_id: String,
    issue_identifier: String,
    issue_title: String,
    project: String,
    started_at: i64,
    stopped_at: Option<i64>,
    duration_seconds: i64,
    source: String,
    started_at_ms: i64,
    stopped_at_ms: Option<i64>,
    elapsed_ms: i64,
    start_at: i64,
    end_at: Option<i64>,
    running: bool,
    note: String,
    time_zone: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceTimePage {
    items: Vec<WorkspaceTimeEntry>,
    next_cursor: Option<String>,
    total_seconds: i64,
    total_ms: i64,
    total_count: i64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkspaceTimeCursor {
    workspace_id: String,
    query: String,
    started_at: i64,
    id: String,
}

fn workspace_time_page(
    connection: &Connection,
    issue: Option<&str>,
    project: Option<&str>,
    start: Option<i64>,
    end: Option<i64>,
    cursor: Option<&str>,
    limit: usize,
) -> Result<WorkspaceTimePage, String> {
    if !(1..=200).contains(&limit) || start.zip(end).is_some_and(|(a, b)| a >= b) {
        return Err("Choose a valid time range and page size.".into());
    }
    let workspace_id = current_workspace_id(connection)?;
    let query = format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&(issue, project, start, end))
                .map_err(|_| "Cannot encode time query.".to_owned())?
        )
    );
    let after = cursor
        .map(|token| -> Result<WorkspaceTimeCursor, String> {
            if token.len() > 1024 {
                return Err("Time cursor is too large.".into());
            }
            let bytes = URL_SAFE_NO_PAD
                .decode(token)
                .map_err(|_| "Time cursor is invalid.")?;
            let value: WorkspaceTimeCursor =
                serde_json::from_slice(&bytes).map_err(|_| "Time cursor is invalid.")?;
            if value.workspace_id != workspace_id || value.query != query {
                return Err("CONFLICT: Time filters changed. Reload the first page.".into());
            }
            Ok(value)
        })
        .transpose()?;
    let filters = "FROM time_entry entry JOIN issue ON issue.id = entry.issue_id
        LEFT JOIN project ON project.id = issue.project_id
        WHERE entry.workspace_id = ?1 AND entry.deleted_at IS NULL
          AND (?2 IS NULL OR issue.identifier = ?2)
          AND (?3 IS NULL OR COALESCE(project.name, 'Unassigned') = ?3)
          AND (?4 IS NULL OR COALESCE(entry.stopped_at, unixepoch('now')) > ?4
               OR (entry.started_at = entry.stopped_at AND entry.started_at >= ?4))
          AND (?5 IS NULL OR entry.started_at < ?5)";
    let aggregate = format!(
        "SELECT count(*), COALESCE(sum(CASE
           WHEN entry.stopped_at IS NULL AND ?4 IS NULL AND ?5 IS NULL THEN entry.duration_seconds
           WHEN entry.stopped_at IS NULL THEN CAST(entry.duration_seconds
                * max(0, min(unixepoch('now'), COALESCE(?5, unixepoch('now')))
                - max(entry.started_at, COALESCE(?4, entry.started_at)))
                / max(1, unixepoch('now') - entry.started_at) AS INTEGER)
           WHEN ?4 IS NULL AND ?5 IS NULL THEN entry.duration_seconds
           WHEN entry.started_at = entry.stopped_at THEN entry.duration_seconds
           ELSE CAST(entry.duration_seconds * max(0, min(entry.stopped_at, COALESCE(?5, entry.stopped_at))
                - max(entry.started_at, COALESCE(?4, entry.started_at)))
                / max(1, entry.stopped_at - entry.started_at) AS INTEGER) END), 0) {filters}"
    );
    let (total_count, total_seconds) = connection
        .query_row(
            &aggregate,
            params![workspace_id, issue, project, start, end],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Cannot total time entries.".to_owned())?;
    let total_ms: i64 = connection
        .query_row(
            &aggregate.replace("entry.duration_seconds", "entry.duration_ms"),
            params![workspace_id, issue, project, start, end],
            |row| row.get(1),
        )
        .map_err(|_| "Cannot total time milliseconds.".to_owned())?;
    let query_sql = format!(
        "SELECT entry.id, issue.identifier, issue.title, COALESCE(project.name, 'Unassigned'),
                entry.started_at, entry.stopped_at, entry.duration_seconds, entry.note, entry.time_zone,
                entry.source, entry.started_at_ms, entry.stopped_at_ms, entry.duration_ms, issue.id
         {filters} AND (?6 IS NULL OR entry.started_at < ?6
             OR (entry.started_at = ?6 AND entry.id < ?7))
         ORDER BY entry.started_at DESC, entry.id DESC LIMIT ?8"
    );
    let mut statement = connection
        .prepare(&query_sql)
        .map_err(|_| "Cannot read time entries.".to_owned())?;
    let mut items = statement
        .query_map(
            params![
                workspace_id,
                issue,
                project,
                start,
                end,
                after.as_ref().map(|value| value.started_at),
                after.as_ref().map(|value| &value.id),
                limit as i64 + 1
            ],
            |row| {
                Ok(WorkspaceTimeEntry {
                    id: row.get(0)?,
                    issue_id: row.get(13)?,
                    issue_identifier: row.get(1)?,
                    issue_title: row.get(2)?,
                    project: row.get(3)?,
                    started_at: row.get(4)?,
                    stopped_at: row.get(5)?,
                    duration_seconds: row.get(6)?,
                    note: row.get(7)?,
                    time_zone: row.get(8)?,
                    source: row.get(9)?,
                    started_at_ms: row.get(10)?,
                    stopped_at_ms: row.get(11)?,
                    elapsed_ms: row.get(12)?,
                    start_at: row.get(10)?,
                    end_at: row.get(11)?,
                    running: row.get::<_, Option<i64>>(5)?.is_none(),
                })
            },
        )
        .map_err(|_| "Cannot read time entries.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read time entries.".to_owned())?;
    let has_more = items.len() > limit;
    items.truncate(limit);
    let next_cursor = if has_more {
        let last = items.last().ok_or("Cannot encode time cursor.")?;
        Some(
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&WorkspaceTimeCursor {
                    workspace_id,
                    query,
                    started_at: last.started_at,
                    id: last.id.clone(),
                })
                .map_err(|_| "Cannot encode time cursor.".to_owned())?,
            ),
        )
    } else {
        None
    };
    Ok(WorkspaceTimePage {
        items,
        next_cursor,
        total_seconds,
        total_ms,
        total_count,
    })
}

fn list_workspace_time(connection: &Connection) -> Result<Vec<WorkspaceTimeIssue>, String> {
    let workspace = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT issue.identifier, issue.title, COALESCE(project.name, 'Unassigned'),
                    COALESCE(SUM(CASE WHEN time_entry.stopped_at IS NOT NULL
                                      THEN time_entry.duration_seconds ELSE 0 END), 0) AS completed_seconds,
                    COUNT(*),
                    MAX(CASE WHEN time_entry.stopped_at IS NULL
                             THEN time_entry.started_at END),
                    issue.archived_at IS NOT NULL
             FROM time_entry JOIN issue ON issue.id = time_entry.issue_id
             LEFT JOIN project ON project.id = issue.project_id
             WHERE time_entry.workspace_id = ?1 AND time_entry.deleted_at IS NULL
             GROUP BY issue.id
             ORDER BY completed_seconds DESC, issue.identifier",
        )
        .map_err(|_| "Cannot read workspace time.".to_owned())?;
    let rows = statement
        .query_map([workspace], |row| {
            Ok(WorkspaceTimeIssue {
                identifier: row.get(0)?,
                title: row.get(1)?,
                project: row.get(2)?,
                completed_seconds: row.get(3)?,
                entry_count: row.get(4)?,
                running_started_at: row.get(5)?,
                archived: row.get(6)?,
            })
        })
        .map_err(|_| "Cannot read workspace time.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read workspace time.".to_owned())?;
    Ok(rows)
}

fn now(connection: &Connection) -> Result<i64, String> {
    connection
        .query_row("SELECT unixepoch('now')", [], |row| row.get(0))
        .map_err(|_| "Cannot read timer clock.".to_owned())
}

fn now_ms(connection: &Connection) -> Result<i64, String> {
    connection
        .query_row(
            "SELECT CAST(unixepoch('subsec') * 1000 AS INTEGER)",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read precise timer clock.".to_owned())
}

fn checkpoint_clock() -> &'static Mutex<Option<Instant>> {
    static CLOCK: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    CLOCK.get_or_init(|| Mutex::new(None))
}

static SUSPENDED: AtomicBool = AtomicBool::new(false);

pub(crate) fn pause_on_sleep(connection: &mut Connection) -> Result<i64, String> {
    heartbeat(connection)?;
    let timestamp = now(connection)?;
    let timestamp_ms = now_ms(connection)?;
    let paused = connection
        .execute(
            "UPDATE time_entry SET stopped_at = ?1, stopped_at_ms = ?2
         WHERE stopped_at IS NULL AND sleep_policy = 'pause'",
            params![timestamp, timestamp_ms],
        )
        .map_err(|_| "Cannot pause timers for sleep.".to_owned())? as i64;
    SUSPENDED.store(true, Ordering::SeqCst);
    Ok(paused)
}

pub(crate) fn resume_after_sleep(connection: &mut Connection) -> Result<(), String> {
    if !SUSPENDED.load(Ordering::SeqCst) {
        return Ok(());
    }
    let timestamp = now(connection)?;
    let timestamp_ms = now_ms(connection)?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin wake reconciliation.".to_owned())?;
    tx.execute(
        "INSERT OR IGNORE INTO timer_recovery_notice
           (id, entry_id, kind, gap_seconds, gap_ms, created_at)
         SELECT lower(hex(randomblob(16))), id, 'backward_clock',
                (?1 - COALESCE(last_seen_at_ms, started_at_ms)) / 1000,
                ?1 - COALESCE(last_seen_at_ms, started_at_ms), ?2
           FROM time_entry WHERE stopped_at IS NULL AND sleep_policy = 'include'
             AND ?1 < COALESCE(last_seen_at_ms, started_at_ms)",
        params![timestamp_ms, timestamp],
    )
    .map_err(|_| "Cannot record a backward clock review.".to_owned())?;
    tx.execute(
        "UPDATE time_entry SET stopped_at = COALESCE(last_seen_at, started_at),
           stopped_at_ms = COALESCE(last_seen_at_ms, started_at_ms)
         WHERE stopped_at IS NULL AND sleep_policy = 'include'
           AND ?1 < COALESCE(last_seen_at_ms, started_at_ms)",
        [timestamp_ms],
    )
    .map_err(|_| "Cannot pause timer after backward clock change.".to_owned())?;
    tx.execute(
        "INSERT OR IGNORE INTO timer_recovery_notice
           (id, entry_id, kind, gap_seconds, gap_ms, created_at)
         SELECT lower(hex(randomblob(16))), id, 'long_gap',
                (?1 - COALESCE(last_seen_at_ms, started_at_ms)) / 1000,
                ?1 - COALESCE(last_seen_at_ms, started_at_ms), ?2
           FROM time_entry WHERE stopped_at IS NULL AND sleep_policy = 'include'
             AND ?1 - COALESCE(last_seen_at_ms, started_at_ms) > 86400000",
        params![timestamp_ms, timestamp],
    )
    .map_err(|_| "Cannot record a long sleep review.".to_owned())?;
    tx.execute(
        "UPDATE time_entry SET stopped_at = COALESCE(last_seen_at, started_at),
           stopped_at_ms = COALESCE(last_seen_at_ms, started_at_ms)
         WHERE stopped_at IS NULL AND sleep_policy = 'include'
           AND ?1 - COALESCE(last_seen_at_ms, started_at_ms) > 86400000",
        [timestamp_ms],
    )
    .map_err(|_| "Cannot pause timer after a long sleep.".to_owned())?;
    tx.execute(
        "UPDATE time_entry SET
           duration_ms = duration_ms + max(0, ?1 - COALESCE(last_seen_at_ms, started_at_ms)),
           duration_seconds = (duration_ms + max(0, ?1 - COALESCE(last_seen_at_ms, started_at_ms))) / 1000,
           last_seen_at = ?2, last_seen_at_ms = ?1
         WHERE stopped_at IS NULL AND sleep_policy = 'include'",
        params![timestamp_ms, timestamp],
    ).map_err(|_| "Cannot reconcile time during sleep.".to_owned())?;
    tx.commit()
        .map_err(|_| "Cannot finish wake reconciliation.".to_owned())?;
    *checkpoint_clock()
        .lock()
        .map_err(|_| "Cannot reset timer clock.".to_owned())? = Some(Instant::now());
    SUSPENDED.store(false, Ordering::SeqCst);
    Ok(())
}

pub(crate) fn recover_on_start(connection: &mut Connection) -> Result<i64, String> {
    let timestamp = now(connection)?;
    let timestamp_ms = now_ms(connection)?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin timer recovery.".to_owned())?;
    let clean_exit: bool = tx
        .query_row(
            "SELECT clean_exit FROM timer_session WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read previous timer session.".to_owned())?;
    let recovered = if clean_exit {
        0
    } else {
        tx.execute(
            "UPDATE time_entry
             SET stopped_at = max(started_at, min(COALESCE(last_seen_at, started_at), ?1)),
                 stopped_at_ms = max(started_at_ms, min(COALESCE(last_seen_at_ms, started_at_ms), ?2))
             WHERE stopped_at IS NULL AND exit_policy = 'pause'",
            params![timestamp, timestamp_ms],
        )
        .map_err(|_| "Cannot recover interrupted timers.".to_owned())? as i64
    };
    let continuing = {
        let mut statement = tx
            .prepare(
                "SELECT id, COALESCE(last_seen_at_ms, started_at_ms) FROM time_entry
             WHERE stopped_at IS NULL AND exit_policy = 'continue'",
            )
            .map_err(|_| "Cannot read continuing timers.".to_owned())?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|_| "Cannot read continuing timers.".to_owned())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "Cannot read continuing timers.".to_owned())?;
        rows
    };
    // Never charge a backward clock jump or an unreviewed gap over 24 hours to a timer.
    for (entry_id, last_seen_ms) in continuing {
        let gap_ms = timestamp_ms - last_seen_ms;
        let gap = gap_ms / 1000;
        let kind = if gap_ms < 0 {
            "backward_clock"
        } else if gap_ms > 86_400_000 {
            "long_gap"
        } else {
            "closed"
        };
        if kind == "closed" {
            tx.execute(
                "UPDATE time_entry SET duration_seconds = (duration_ms + ?1) / 1000,
                   duration_ms = duration_ms + ?1, last_seen_at = ?2, last_seen_at_ms = ?3
                 WHERE id = ?4",
                params![gap_ms, timestamp, timestamp_ms, entry_id],
            )
            .map_err(|_| "Cannot reconcile continuing timer.".to_owned())?;
        } else {
            tx.execute(
                "UPDATE time_entry SET stopped_at = max(started_at, ?1),
                   stopped_at_ms = max(started_at_ms, COALESCE(last_seen_at_ms, ?2))
                 WHERE id = ?3",
                params![last_seen_ms / 1000, last_seen_ms, entry_id],
            )
            .map_err(|_| "Cannot pause timer for recovery review.".to_owned())?;
        }
        if gap_ms != 0 {
            tx.execute(
                "INSERT OR IGNORE INTO timer_recovery_notice(id, entry_id, kind, gap_seconds, gap_ms, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![Uuid::new_v4().to_string(), entry_id, kind, gap, gap_ms, timestamp],
            ).map_err(|_| "Cannot record timer recovery review.".to_owned())?;
        }
    }
    tx.execute(
        "UPDATE timer_session SET clean_exit = 0, recovered_timers = recovered_timers + ?1
         WHERE singleton = 1",
        [recovered],
    )
    .map_err(|_| "Cannot mark timer session active.".to_owned())?;
    tx.commit()
        .map_err(|_| "Cannot finish timer recovery.".to_owned())?;
    *checkpoint_clock()
        .lock()
        .map_err(|_| "Cannot initialize timer clock.".to_owned())? = Some(Instant::now());
    Ok(recovered)
}

pub(crate) fn heartbeat(connection: &mut Connection) -> Result<i64, String> {
    if SUSPENDED.load(Ordering::SeqCst) {
        return Ok(0);
    }
    let timestamp = now(connection)?;
    let timestamp_ms = now_ms(connection)?;
    let current_tick = Instant::now();
    let mut checkpoint = checkpoint_clock()
        .lock()
        .map_err(|_| "Cannot read timer clock.".to_owned())?;
    // Duration comes from the monotonic clock; wall time only locates the saved checkpoint.
    let elapsed_ms = checkpoint
        .as_ref()
        .map(|previous| current_tick.duration_since(*previous).as_millis())
        .unwrap_or(0);
    let elapsed_ms = i64::try_from(elapsed_ms).unwrap_or(i64::MAX);
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin timer heartbeat.".to_owned())?;
    // Fallback for an unavailable native suspend signal: do not claim the gap as active work.
    tx.execute(
        "INSERT INTO timer_recovery_notice(id, entry_id, kind, gap_seconds, gap_ms, created_at)
         SELECT lower(hex(randomblob(16))), id, 'sleep_uncertain',
                ?1 - COALESCE(last_seen_at, started_at),
                ?2 - COALESCE(last_seen_at_ms, started_at_ms), ?1
         FROM time_entry WHERE stopped_at IS NULL
           AND ?1 - COALESCE(last_seen_at, started_at) > 180 AND sleep_policy = 'pause'",
        params![timestamp, timestamp_ms],
    )
    .map_err(|_| "Cannot flag uncertain timer gap.".to_owned())?;
    let paused = tx
        .execute(
            "UPDATE time_entry
             SET stopped_at = max(started_at, COALESCE(last_seen_at, started_at)),
                 stopped_at_ms = max(started_at_ms, COALESCE(last_seen_at_ms, started_at_ms))
             WHERE stopped_at IS NULL AND ?1 - COALESCE(last_seen_at, started_at) > 180
               AND sleep_policy = 'pause'",
            [timestamp],
        )
        .map_err(|_| "Cannot pause timers after sleep.".to_owned())? as i64;
    tx.execute(
        "UPDATE time_entry SET duration_ms = duration_ms + ?2,
           duration_seconds = (duration_ms + ?2) / 1000,
           last_seen_at = max(COALESCE(last_seen_at, started_at), ?1),
           last_seen_at_ms = max(COALESCE(last_seen_at_ms, started_at_ms), ?3)
         WHERE stopped_at IS NULL",
        params![timestamp, elapsed_ms, timestamp_ms],
    )
    .map_err(|_| "Cannot update timer heartbeat.".to_owned())?;
    tx.commit()
        .map_err(|_| "Cannot finish timer heartbeat.".to_owned())?;
    *checkpoint = Some(
        checkpoint
            .as_ref()
            .map(|previous| *previous + Duration::from_millis(elapsed_ms as u64))
            .unwrap_or(current_tick),
    );
    Ok(paused)
}

fn issue_id(connection: &Connection, identifier: &str) -> Result<String, String> {
    connection.query_row(
        "SELECT id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
        params![current_workspace_id(connection)?, identifier],
        |row| row.get(0),
    ).optional().map_err(|_| "Cannot read issue for timer.".to_owned())?
        .ok_or("Active issue not found.".into())
}

fn read_time_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimeEntry> {
    let stopped_at_ms = row.get(10)?;
    Ok(TimeEntry {
        id: row.get(0)?,
        version: row.get(1)?,
        issue_id: row.get(12)?,
        issue_identifier: row.get(2)?,
        started_at: row.get(3)?,
        stopped_at: row.get(4)?,
        duration_seconds: row.get(5)?,
        note: row.get(6)?,
        time_zone: row.get(7)?,
        source: row.get(8)?,
        started_at_ms: row.get(9)?,
        stopped_at_ms,
        elapsed_ms: row.get(11)?,
        start_at: row.get(9)?,
        end_at: stopped_at_ms,
        running: stopped_at_ms.is_none(),
    })
}

fn list_time(connection: &Connection, identifier: &str) -> Result<TimeSummary, String> {
    let issue = issue_id(connection, identifier)?;
    let workspace = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT time_entry.id, time_entry.version, issue.identifier, started_at, stopped_at,
                    duration_seconds, note, time_zone, source, started_at_ms, stopped_at_ms, duration_ms, issue.id
         FROM time_entry JOIN issue ON issue.id = time_entry.issue_id
         WHERE time_entry.issue_id = ?1 AND time_entry.deleted_at IS NULL ORDER BY started_at DESC, time_entry.id DESC",
        )
        .map_err(|_| "Cannot read time entries.".to_owned())?;
    let entries = statement
        .query_map([&issue], read_time_entry)
        .map_err(|_| "Cannot read time entries.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read time entries.".to_owned())?;
    let running = connection
        .query_row(
            "SELECT time_entry.id, time_entry.version, issue.identifier, started_at, stopped_at,
                    duration_seconds, note, time_zone, source, started_at_ms, stopped_at_ms, duration_ms, issue.id
         FROM time_entry JOIN issue ON issue.id = time_entry.issue_id
         WHERE time_entry.workspace_id = ?1 AND stopped_at IS NULL AND time_entry.deleted_at IS NULL
         ORDER BY time_entry.started_at LIMIT 1",
            [&workspace],
            read_time_entry,
        )
        .optional()
        .map_err(|_| "Cannot read running timer.".to_owned())?;
    let total_seconds = entries.iter().map(|entry| entry.duration_seconds).sum();
    let total_ms = entries.iter().map(|entry| entry.elapsed_ms).sum();
    Ok(TimeSummary {
        entries,
        running,
        total_seconds,
        total_ms,
    })
}

fn running_timer(connection: &Connection) -> Result<Option<TimeEntry>, String> {
    let workspace = current_workspace_id(connection)?;
    connection
        .query_row(
            "SELECT time_entry.id, time_entry.version, issue.identifier, started_at, stopped_at,
                    duration_seconds, note, time_zone, source, started_at_ms, stopped_at_ms, duration_ms, issue.id
             FROM time_entry JOIN issue ON issue.id = time_entry.issue_id
             WHERE time_entry.workspace_id = ?1 AND stopped_at IS NULL AND time_entry.deleted_at IS NULL
             ORDER BY time_entry.started_at LIMIT 1",
            [&workspace],
            read_time_entry,
        )
        .optional()
        .map_err(|_| "Cannot read running timer.".to_owned())
}

fn running_timers(connection: &Connection) -> Result<Vec<TimeEntry>, String> {
    let workspace = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT time_entry.id, time_entry.version, issue.identifier, started_at, stopped_at,
                duration_seconds, note, time_zone, source, started_at_ms, stopped_at_ms,
                duration_ms, issue.id FROM time_entry
         JOIN issue ON issue.id = time_entry.issue_id
         WHERE time_entry.workspace_id = ?1 AND stopped_at IS NULL AND time_entry.deleted_at IS NULL
         ORDER BY started_at, time_entry.id",
        )
        .map_err(|_| "Cannot read running timers.".to_owned())?;
    let entries = statement
        .query_map([workspace], read_time_entry)
        .map_err(|_| "Cannot read running timers.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read running timers.".to_owned())?;
    Ok(entries)
}

fn timer_state_version(connection: &Connection) -> Result<i64, String> {
    let workspace = current_workspace_id(connection)?;
    let running: String = connection
        .query_row(
            "SELECT COALESCE(group_concat(id, ','), '') FROM
             (SELECT id FROM time_entry WHERE workspace_id = ?1 AND stopped_at IS NULL AND deleted_at IS NULL ORDER BY id)",
            [&workspace],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot read timer state.".to_owned())?;
    if running.is_empty() {
        return Ok(1);
    }
    let digest = Sha256::digest(running.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    Ok((i64::from_be_bytes(bytes) & ((1_i64 << 53) - 1)).max(2))
}

fn check_timer_state(connection: &Connection, mutation: &MutationRequest) -> Result<(), String> {
    if mutation.expected_version != Some(timer_state_version(connection)?) {
        return Err("CONFLICT: Timer changed. Reload and try again.".into());
    }
    Ok(())
}

#[cfg(test)]
fn start_timer(connection: &mut Connection, identifier: &str) -> Result<TimeSummary, String> {
    heartbeat(connection)?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot start timer transaction.".to_owned())?;
    let result = start_timer_in_tx(&tx, identifier, true)?;
    tx.commit().map_err(|_| "Cannot save timer.".to_owned())?;
    Ok(result)
}

fn start_timer_in_tx(
    connection: &Connection,
    identifier: &str,
    switch_current: bool,
) -> Result<TimeSummary, String> {
    let issue = issue_id(connection, identifier)?;
    let workspace = current_workspace_id(connection)?;
    let timestamp = now(connection)?;
    let timestamp_ms = now_ms(connection)?;
    let already_running: bool = connection.query_row(
        "SELECT count(*) > 0 FROM time_entry WHERE workspace_id = ?1 AND issue_id = ?2 AND stopped_at IS NULL",
        params![workspace, issue], |row| row.get(0),
    ).map_err(|_| "Cannot inspect running timer.".to_owned())?;
    if !already_running {
        let policies: (bool, String, String) = connection.query_row(
            "SELECT concurrent_timers, timer_on_exit, timer_on_sleep FROM workspace_preferences WHERE workspace_id = ?1",
            [&workspace], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        ).optional().map_err(|_| "Cannot read timer settings.".to_owned())?
          .unwrap_or((false, "pause".into(), "pause".into()));
        let another_running: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM time_entry WHERE workspace_id = ?1
             AND issue_id <> ?2 AND stopped_at IS NULL AND deleted_at IS NULL)",
                params![workspace, issue],
                |row| row.get(0),
            )
            .map_err(|_| "Cannot inspect running timers.".to_owned())?;
        if another_running && !policies.0 && !switch_current {
            return Err(
                "CONFIRMATION_REQUIRED: Pause the current timer and start this issue?".into(),
            );
        }
        if another_running && !policies.0 {
            connection
                .execute(
                    "UPDATE time_entry SET stopped_at = ?1, stopped_at_ms = ?2
             WHERE workspace_id = ?3 AND stopped_at IS NULL",
                    params![timestamp, timestamp_ms, workspace],
                )
                .map_err(|_| "Cannot pause previous timer.".to_owned())?;
        }
        connection
            .execute(
                "INSERT INTO time_entry (id, workspace_id, issue_id, started_at, last_seen_at,
                   started_at_ms, last_seen_at_ms, exit_policy, sleep_policy)
             VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?5, ?6, ?7)",
                params![
                    Uuid::new_v4().to_string(),
                    workspace,
                    issue,
                    timestamp,
                    timestamp_ms,
                    policies.1,
                    policies.2
                ],
            )
            .map_err(|_| "Cannot start timer.".to_owned())?;
    }
    list_time(connection, identifier)
}

#[cfg(test)]
fn stop_timer(connection: &mut Connection, identifier: &str) -> Result<TimeSummary, String> {
    heartbeat(connection)?;
    stop_timer_in_tx(connection, identifier)
}

fn stop_timer_in_tx(connection: &Connection, identifier: &str) -> Result<TimeSummary, String> {
    let issue = issue_id(connection, identifier)?;
    let timestamp = now(connection)?;
    let timestamp_ms = now_ms(connection)?;
    connection
        .execute(
            "UPDATE time_entry SET stopped_at = ?1, stopped_at_ms = ?2
         WHERE issue_id = ?3 AND stopped_at IS NULL",
            params![timestamp, timestamp_ms, issue],
        )
        .map_err(|_| "Cannot pause timer.".to_owned())?;
    list_time(connection, identifier)
}

#[cfg(test)]
fn add_time(
    connection: &Connection,
    identifier: &str,
    seconds: i64,
    note: &str,
) -> Result<TimeSummary, String> {
    if !(1..=31_536_000).contains(&seconds) || note.chars().count() > 500 {
        return Err("Enter a duration of up to one year and a short note.".into());
    }
    let issue = issue_id(connection, identifier)?;
    let workspace = current_workspace_id(connection)?;
    let timestamp = now(connection)?;
    connection
        .execute(
            "INSERT INTO time_entry (id, workspace_id, issue_id, started_at, stopped_at,
           started_at_ms, stopped_at_ms, duration_seconds, duration_ms, source, note)
         VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?5, ?6, ?7, 'manual', ?8)",
            params![
                Uuid::new_v4().to_string(),
                workspace,
                issue,
                timestamp,
                now_ms(connection)?,
                seconds,
                seconds * 1000,
                note.trim()
            ],
        )
        .map_err(|_| "Cannot save time entry.".to_owned())?;
    list_time(connection, identifier)
}

fn resolve_manual_start(
    date: &str,
    start_time: &str,
    time_zone: &str,
    ambiguous: Option<&str>,
) -> Result<i64, String> {
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| "Choose a valid entry date.".to_owned())?;
    let time = NaiveTime::parse_from_str(start_time, "%H:%M")
        .map_err(|_| "Choose a valid start time.".to_owned())?;
    let zone: Tz = time_zone
        .parse()
        .map_err(|_| "Choose a valid IANA timezone.".to_owned())?;
    let local = date.and_time(time);
    let instant = match zone.from_local_datetime(&local) {
        chrono::LocalResult::Single(value) => value,
        chrono::LocalResult::Ambiguous(earlier, later) => match ambiguous {
            Some("earlier") => earlier,
            Some("later") => later,
            _ => return Err("Choose earlier or later for this ambiguous local time.".into()),
        },
        chrono::LocalResult::None => {
            return Err("This local time does not exist after the daylight saving change.".into())
        }
    };
    Ok(instant.timestamp())
}

fn check_manual_overlap(
    connection: &Connection,
    workspace: &str,
    start: i64,
    end: i64,
    except: Option<&str>,
    allow_overlap: bool,
) -> Result<(), String> {
    if allow_overlap {
        return Ok(());
    }
    let overlaps: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM time_entry WHERE workspace_id = ?1 AND deleted_at IS NULL
          AND (?4 IS NULL OR id <> ?4) AND started_at < ?3
          AND COALESCE(stopped_at, unixepoch('now')) > ?2)",
            params![workspace, start, end, except],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot inspect time overlap.".to_owned())?;
    if overlaps {
        return Err("CONFIRMATION_REQUIRED: Time entry overlaps existing tracked time.".into());
    }
    Ok(())
}

fn record_time_activity(
    connection: &Connection,
    issue: &str,
    body: &str,
    before: serde_json::Value,
    after: serde_json::Value,
    mutation_id: Option<&str>,
) -> Result<(), String> {
    let changes = serde_json::json!([{"field":"timeEntry","before":before,"after":after}]);
    connection.execute(
        "INSERT INTO issue_activity(id, issue_id, actor_name, kind, body, changes_json, mutation_id)
         VALUES (?1, ?2, (SELECT display_name FROM local_actor WHERE singleton = 1), 'changed', ?3, ?4, ?5)",
        params![Uuid::new_v4().to_string(), issue, body, changes.to_string(), mutation_id],
    ).map_err(|_| "Cannot record time change.".to_owned())?;
    Ok(())
}

fn add_time_at(
    connection: &Connection,
    identifier: &str,
    seconds: i64,
    note: &str,
    date: &str,
    start_time: &str,
    time_zone: &str,
    ambiguous: Option<&str>,
    allow_overlap: bool,
    mutation_id: Option<&str>,
) -> Result<TimeSummary, String> {
    if !(1..=31_536_000).contains(&seconds) || note.chars().count() > 500 {
        return Err("Enter a duration of up to one year and a short note.".into());
    }
    let start = resolve_manual_start(date, start_time, time_zone, ambiguous)?;
    let end = start
        .checked_add(seconds)
        .ok_or("Entry end is out of range.")?;
    if end > now(connection)? {
        return Err("Entry end cannot be in the future.".into());
    }
    let issue = issue_id(connection, identifier)?;
    let workspace = current_workspace_id(connection)?;
    check_manual_overlap(connection, &workspace, start, end, None, allow_overlap)?;
    let entry_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO time_entry(id, workspace_id, issue_id, started_at, stopped_at,
           started_at_ms, stopped_at_ms, duration_seconds, duration_ms, source, note, time_zone)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'manual', ?10, ?11)",
            params![
                entry_id,
                workspace,
                issue,
                start,
                end,
                start * 1000,
                end * 1000,
                seconds,
                seconds * 1000,
                note.trim(),
                time_zone
            ],
        )
        .map_err(|_| "Cannot save time entry.".to_owned())?;
    record_time_activity(
        connection,
        &issue,
        "added time entry",
        serde_json::Value::Null,
        serde_json::json!({"id":entry_id,"startedAt":start,"stoppedAt":end,"seconds":seconds,"note":note.trim(),"timeZone":time_zone}),
        mutation_id,
    )?;
    list_time(connection, identifier)
}

#[cfg(test)]
fn update_time(
    connection: &Connection,
    identifier: &str,
    entry_id: &str,
    seconds: i64,
    note: &str,
) -> Result<TimeSummary, String> {
    if !(1..=31_536_000).contains(&seconds) || note.chars().count() > 500 {
        return Err("Enter a duration of up to one year and a short note.".into());
    }
    let issue = issue_id(connection, identifier)?;
    let changed = connection
        .execute(
            "UPDATE time_entry SET duration_seconds = ?1, duration_ms = ?2, note = ?3
             WHERE id = ?4 AND issue_id = ?5 AND stopped_at IS NOT NULL AND deleted_at IS NULL",
            params![seconds, seconds * 1000, note.trim(), entry_id, issue],
        )
        .map_err(|_| "Cannot update time entry.".to_owned())?;
    if changed == 0 {
        return Err("Completed time entry not found.".into());
    }
    list_time(connection, identifier)
}

fn update_time_at(
    connection: &Connection,
    identifier: &str,
    entry_id: &str,
    seconds: i64,
    note: &str,
    date: &str,
    start_time: &str,
    time_zone: &str,
    ambiguous: Option<&str>,
    allow_overlap: bool,
    mutation_id: Option<&str>,
) -> Result<TimeSummary, String> {
    if !(1..=31_536_000).contains(&seconds) || note.chars().count() > 500 {
        return Err("Enter a duration of up to one year and a short note.".into());
    }
    let start = resolve_manual_start(date, start_time, time_zone, ambiguous)?;
    let end = start
        .checked_add(seconds)
        .ok_or("Entry end is out of range.")?;
    if end > now(connection)? {
        return Err("Entry end cannot be in the future.".into());
    }
    let issue = issue_id(connection, identifier)?;
    let before: (i64, i64, i64, String, Option<String>) = connection
        .query_row(
            "SELECT started_at, stopped_at, duration_seconds, note, time_zone FROM time_entry
         WHERE id = ?1 AND issue_id = ?2 AND stopped_at IS NOT NULL AND deleted_at IS NULL",
            params![entry_id, issue],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|_| "Completed time entry not found.".to_owned())?;
    let workspace = current_workspace_id(connection)?;
    check_manual_overlap(
        connection,
        &workspace,
        start,
        end,
        Some(entry_id),
        allow_overlap,
    )?;
    let changed = connection
        .execute(
            "UPDATE time_entry SET started_at = ?1, stopped_at = ?2, started_at_ms = ?3,
         stopped_at_ms = ?4, duration_seconds = ?5, duration_ms = ?6,
         note = ?7, time_zone = ?8 WHERE id = ?9 AND issue_id = ?10
         AND stopped_at IS NOT NULL AND deleted_at IS NULL",
            params![
                start,
                end,
                start * 1000,
                end * 1000,
                seconds,
                seconds * 1000,
                note.trim(),
                time_zone,
                entry_id,
                issue
            ],
        )
        .map_err(|_| "Cannot update time entry.".to_owned())?;
    if changed == 0 {
        return Err("Completed time entry not found.".into());
    }
    record_time_activity(
        connection,
        &issue,
        "edited time entry",
        serde_json::json!({"id":entry_id,"startedAt":before.0,"stoppedAt":before.1,"seconds":before.2,"note":before.3,"timeZone":before.4}),
        serde_json::json!({"id":entry_id,"startedAt":start,"stoppedAt":end,"seconds":seconds,"note":note.trim(),"timeZone":time_zone}),
        mutation_id,
    )?;
    list_time(connection, identifier)
}

fn delete_time(
    connection: &Connection,
    identifier: &str,
    entry_id: &str,
    mutation_id: Option<&str>,
) -> Result<TimeSummary, String> {
    let issue = issue_id(connection, identifier)?;
    let before: (i64, i64, i64, String, Option<String>) = connection
        .query_row(
            "SELECT started_at, stopped_at, duration_seconds, note, time_zone FROM time_entry
         WHERE id = ?1 AND issue_id = ?2 AND stopped_at IS NOT NULL AND deleted_at IS NULL",
            params![entry_id, issue],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|_| "Completed time entry not found.".to_owned())?;
    let changed = connection
        .execute(
            "UPDATE time_entry SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1 AND issue_id = ?2 AND stopped_at IS NOT NULL AND deleted_at IS NULL",
            params![entry_id, issue],
        )
        .map_err(|_| "Cannot delete time entry.".to_owned())?;
    if changed == 0 {
        return Err("Completed time entry not found.".into());
    }
    record_time_activity(
        connection,
        &issue,
        "removed time entry",
        serde_json::json!({"id":entry_id,"startedAt":before.0,"stoppedAt":before.1,"seconds":before.2,"note":before.3,"timeZone":before.4}),
        serde_json::Value::Null,
        mutation_id,
    )?;
    list_time(connection, identifier)
}

pub(crate) fn pause_running_on_exit(connection: &mut Connection) -> Result<(), String> {
    heartbeat(connection)?;
    let timestamp = now(connection)?;
    let timestamp_ms = now_ms(connection)?;
    let tx = connection
        .transaction()
        .map_err(|_| "Cannot begin timer shutdown.".to_owned())?;
    tx.execute(
        "UPDATE time_entry SET stopped_at = ?1, stopped_at_ms = ?2
             WHERE stopped_at IS NULL AND exit_policy = 'pause'",
        params![timestamp, timestamp_ms],
    )
    .map_err(|_| "Cannot pause running timer before exit.".to_owned())?;
    tx.execute(
        "UPDATE timer_session SET clean_exit = 1 WHERE singleton = 1",
        [],
    )
    .map_err(|_| "Cannot mark timer session closed.".to_owned())?;
    tx.commit()
        .map_err(|_| "Cannot finish timer shutdown.".to_owned())?;
    Ok(())
}

#[api_contract_macros::command]
pub fn storage_list_time(app: AppHandle, identifier: String) -> Result<TimeSummary, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    heartbeat(&mut connection)?;
    list_time(&connection, &identifier)
}

#[api_contract_macros::command]
pub fn storage_list_workspace_time(app: AppHandle) -> Result<Vec<WorkspaceTimeIssue>, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    heartbeat(&mut connection)?;
    list_workspace_time(&connection)
}

#[api_contract_macros::command]
pub fn storage_list_workspace_time_page(
    app: AppHandle,
    issue: Option<String>,
    project: Option<String>,
    start: Option<i64>,
    end: Option<i64>,
    cursor: Option<String>,
    limit: Option<usize>,
) -> Result<WorkspaceTimePage, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    heartbeat(&mut connection)?;
    workspace_time_page(
        &connection,
        issue.as_deref(),
        project.as_deref(),
        start,
        end,
        cursor.as_deref(),
        limit.unwrap_or(50),
    )
}

#[api_contract_macros::command]
pub fn storage_running_timer(app: AppHandle) -> Result<Option<TimeEntry>, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    heartbeat(&mut connection)?;
    running_timer(&connection)
}

#[api_contract_macros::command]
pub fn storage_running_timers(app: AppHandle) -> Result<Vec<TimeEntry>, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    heartbeat(&mut connection)?;
    running_timers(&connection)
}

#[api_contract_macros::command]
pub fn storage_timer_state_version(app: AppHandle) -> Result<i64, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    heartbeat(&mut connection)?;
    timer_state_version(&connection)
}

#[api_contract_macros::command]
pub fn storage_start_timer(
    app: AppHandle,
    identifier: String,
    switch_current: Option<bool>,
    mutation: MutationRequest,
) -> Result<TimeSummary, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    heartbeat(&mut connection)?;
    mutations::apply(
        &mut connection,
        "timer.start",
        &(&identifier, switch_current),
        &mutation,
        |tx, _| {
            check_timer_state(tx, &mutation)?;
            start_timer_in_tx(tx, &identifier, switch_current.unwrap_or(false))
        },
    )
}

#[api_contract_macros::command]
pub fn storage_stop_timer(
    app: AppHandle,
    identifier: String,
    mutation: MutationRequest,
) -> Result<TimeSummary, String> {
    let mut connection = open_database(&document_data_root(&app)?)?;
    heartbeat(&mut connection)?;
    mutations::apply(
        &mut connection,
        "timer.stop",
        &identifier,
        &mutation,
        |tx, _| {
            check_timer_state(tx, &mutation)?;
            stop_timer_in_tx(tx, &identifier)
        },
    )
}

#[api_contract_macros::command]
pub fn storage_add_time(
    app: AppHandle,
    identifier: String,
    seconds: i64,
    note: String,
    date: String,
    start_time: String,
    time_zone: String,
    ambiguous: Option<String>,
    allow_overlap: Option<bool>,
    mutation: MutationRequest,
) -> Result<TimeSummary, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "time.add",
        &(
            &identifier,
            seconds,
            &note,
            &date,
            &start_time,
            &time_zone,
            &ambiguous,
            allow_overlap,
        ),
        &mutation,
        |tx, _| {
            add_time_at(
                tx,
                &identifier,
                seconds,
                &note,
                &date,
                &start_time,
                &time_zone,
                ambiguous.as_deref(),
                allow_overlap.unwrap_or(false),
                Some(&mutation.mutation_id),
            )
        },
    )
}

#[api_contract_macros::command]
pub fn storage_update_time(
    app: AppHandle,
    identifier: String,
    entry_id: String,
    seconds: i64,
    note: String,
    date: String,
    start_time: String,
    time_zone: String,
    ambiguous: Option<String>,
    allow_overlap: Option<bool>,
    mutation: MutationRequest,
) -> Result<TimeSummary, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "time.update",
        &(
            &identifier,
            &entry_id,
            seconds,
            &note,
            &date,
            &start_time,
            &time_zone,
            &ambiguous,
            allow_overlap,
        ),
        &mutation,
        |tx, _| {
            mutations::check_version(tx, "time_entry", &entry_id, &mutation)?;
            update_time_at(
                tx,
                &identifier,
                &entry_id,
                seconds,
                &note,
                &date,
                &start_time,
                &time_zone,
                ambiguous.as_deref(),
                allow_overlap.unwrap_or(false),
                Some(&mutation.mutation_id),
            )
        },
    )
}

#[api_contract_macros::command]
pub fn storage_delete_time(
    app: AppHandle,
    identifier: String,
    entry_id: String,
    mutation: MutationRequest,
) -> Result<TimeSummary, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "time.delete",
        &(&identifier, &entry_id),
        &mutation,
        |tx, _| {
            mutations::check_version(tx, "time_entry", &entry_id, &mutation)?;
            delete_time(tx, &identifier, &entry_id, Some(&mutation.mutation_id))
        },
    )
}

#[cfg(test)]
mod tests;
