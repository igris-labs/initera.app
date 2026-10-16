use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use uuid::Uuid;

use crate::cycle_time::{boundary, local_date_now, utc_now, valid_time, valid_zone};
use crate::mutations::{self, MutationRequest};
use crate::storage::{current_workspace_id, document_data_root, open_database, valid_date};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleRecord {
    id: String,
    version: i64,
    title: String,
    description: String,
    start_date: String,
    end_date: String,
    status: String,
    scheduled: bool,
    series_id: Option<String>,
    successor_id: Option<String>,
    issue_ids: Vec<String>,
    project_ids: Vec<String>,
    completed_count: Option<i64>,
    total_count: Option<i64>,
    rollover_policy: String,
    transition_result: Option<String>,
    time_zone: String,
    start_local_time: String,
    end_local_time: String,
    start_at: String,
    end_at: String,
    precise_end: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CycleSettings {
    id: String,
    name: String,
    duration_days: i64,
    interval_days: i64,
    enabled: bool,
    start_date: String,
    rollover_policy: String,
    project_ids: Vec<String>,
    time_zone: String,
    start_local_time: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleSettingsView {
    #[serde(flatten)]
    settings: CycleSettings,
    version: i64,
}

fn series_version(settings: &CycleSettings) -> Result<i64, String> {
    let encoded = serde_json::to_vec(settings)
        .map_err(|_| "Cannot encode cycle series version.".to_owned())?;
    let digest = Sha256::digest(encoded);
    Ok((i64::from_be_bytes(digest[..8].try_into().unwrap()) & ((1_i64 << 53) - 1)).max(1))
}

fn current_series_version(connection: &Connection, id: &str) -> Result<i64, String> {
    let settings = list_cycle_series(connection)?
        .into_iter()
        .find(|series| series.id == id)
        .ok_or("Cycle series not found.")?;
    series_version(&settings)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleReconcileProgress {
    series_id: String,
    total: i64,
    processed: i64,
}

fn validate_project_scope(connection: &Connection, project_ids: &[String]) -> Result<(), String> {
    if project_ids.len() > 50 || project_ids.iter().any(|id| Uuid::parse_str(id).is_err()) {
        return Err("Choose valid projects for the cycle.".into());
    }
    let workspace_id = current_workspace_id(connection)?;
    for id in project_ids {
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM project WHERE id = ?1 AND workspace_id = ?2)",
                params![id, workspace_id],
                |row| row.get(0),
            )
            .map_err(|_| "Cannot check cycle projects.".to_owned())?;
        if !exists {
            return Err("Cycle project is not in this workspace.".into());
        }
    }
    Ok(())
}

fn list_cycle_series(connection: &Connection) -> Result<Vec<CycleSettings>, String> {
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT id, name, duration_days, interval_days, enabled, start_date,
                        rollover_policy, project_scope_json, time_zone, start_local_time
                  FROM cycle_series WHERE workspace_id = ?1 ORDER BY name",
        )
        .map_err(|_| "Cannot read cycle series.".to_owned())?;
    let series = statement
        .query_map([workspace_id], |row| {
            let scope: String = row.get(7)?;
            Ok(CycleSettings {
                id: row.get(0)?,
                name: row.get(1)?,
                duration_days: row.get(2)?,
                interval_days: row.get(3)?,
                enabled: row.get(4)?,
                start_date: row.get(5)?,
                rollover_policy: row.get(6)?,
                project_ids: serde_json::from_str(&scope)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                time_zone: row.get(8)?,
                start_local_time: row.get(9)?,
            })
        })
        .map_err(|_| "Cannot read cycle series.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read cycle series.".to_owned())?;
    Ok(series)
}

fn offset_date(connection: &Connection, date: &str, days: i64) -> Result<String, String> {
    connection
        .query_row(
            "SELECT date(?1, printf('%+d days', ?2))",
            params![date, days],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot calculate cycle dates.".to_owned())
}

// Reconcile on read so a desktop app catches up after being closed across cycle boundaries.
fn reconcile_cycles(connection: &mut Connection) -> Result<(), String> {
    reconcile_manual_cycles(connection)?;
    for settings in list_cycle_series(connection)? {
        reconcile_series(connection, &settings)?;
    }
    Ok(())
}

fn reconcile_series(connection: &mut Connection, settings: &CycleSettings) -> Result<(), String> {
    let workspace_id = current_workspace_id(connection)?;
    let now = utc_now();
    let today = NaiveDate::parse_from_str(&local_date_now(&settings.time_zone)?, "%Y-%m-%d")
        .map_err(|_| "Cannot calculate cycle dates.".to_owned())?;
    let anchor = NaiveDate::parse_from_str(&settings.start_date, "%Y-%m-%d")
        .map_err(|_| "Cannot calculate cycle dates.".to_owned())?;
    let span = settings.interval_days;
    let scope = serde_json::to_string(&settings.project_ids)
        .map_err(|_| "Cannot encode cycle projects.")?;
    let mut index = ((today - anchor).num_days() / span).max(0);
    let candidate_start = offset_date(connection, &settings.start_date, index * span)?;
    let candidate_end = offset_date(connection, &candidate_start, settings.duration_days)?;
    if boundary(
        &candidate_end,
        &settings.start_local_time,
        &settings.time_zone,
    )? <= now
    {
        index += 1;
    }
    let mut future_count = 0;

    // Calculate position directly. Years of downtime create only current/future cycles.
    while settings.enabled && future_count < 12 {
        let start = offset_date(connection, &settings.start_date, index * span)?;
        let end = offset_date(connection, &start, settings.duration_days - 1)?;
        let start_at = boundary(&start, &settings.start_local_time, &settings.time_zone)?;
        let end_at = boundary(
            &offset_date(connection, &end, 1)?,
            &settings.start_local_time,
            &settings.time_zone,
        )?;
        connection.execute(
            "INSERT OR IGNORE INTO cycle (id, workspace_id, title, start_date, end_date, schedule_index, series_id, project_scope_json, rollover_policy, time_zone, start_local_time, end_local_time, start_at, end_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11, ?12, ?13)",
            params![Uuid::new_v4().to_string(), workspace_id, format!("{} {}", settings.name, index + 1), start, end, index, settings.id, scope, settings.rollover_policy, settings.time_zone, settings.start_local_time, start_at, end_at],
        ).map_err(|_| "Cannot create upcoming cycles.".to_owned())?;
        if start_at > now {
            future_count += 1;
        }
        index += 1;
    }

    // Transition receipts are the durable cursor: a restart skips captured
    // boundaries, while unresolved automatic rollovers get a fair retry order.
    let remaining: i64 = connection
        .query_row(
            "SELECT count(*) FROM cycle WHERE series_id = ?1 AND status != 'completed'
         AND end_at <= ?2 AND NOT EXISTS
           (SELECT 1 FROM cycle_transition WHERE source_cycle_id = cycle.id)",
            params![settings.id, now],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot count elapsed cycles.".to_owned())?;
    if remaining > 100 {
        connection
            .execute(
                "INSERT INTO cycle_reconcile_progress(series_id, total, processed)
             VALUES (?1, ?2, 0)
             ON CONFLICT(series_id) DO UPDATE SET total = max(total, processed + excluded.total)",
                params![settings.id, remaining],
            )
            .map_err(|_| "Cannot record cycle catch-up progress.".to_owned())?;
    }

    let mut statement = connection
        .prepare(
            "SELECT id, schedule_index, rollover_policy FROM cycle
                  WHERE series_id = ?1 AND status != 'completed' AND end_at <= ?2
                    AND NOT EXISTS
                      (SELECT 1 FROM cycle_transition WHERE source_cycle_id = cycle.id)
                  ORDER BY schedule_index LIMIT 100",
        )
        .map_err(|_| "Cannot inspect elapsed cycles.".to_owned())?;
    let mut elapsed = statement
        .query_map(params![settings.id, now], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|_| "Cannot inspect elapsed cycles.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot inspect elapsed cycles.".to_owned())?;
    drop(statement);
    let mut retry = connection
        .prepare(
            "SELECT cycle.id, cycle.schedule_index, cycle.rollover_policy
         FROM cycle JOIN cycle_transition transition ON transition.source_cycle_id = cycle.id
         WHERE cycle.series_id = ?1 AND cycle.status != 'completed' AND cycle.end_at <= ?2
           AND cycle.rollover_policy = 'automatic' AND transition.result = 'pending'
         ORDER BY coalesce(transition.last_retried_at, transition.captured_at),
                  cycle.schedule_index LIMIT 100",
        )
        .map_err(|_| "Cannot inspect pending cycle rollovers.".to_owned())?;
    elapsed.extend(
        retry
            .query_map(params![settings.id, now], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|_| "Cannot inspect pending cycle rollovers.".to_owned())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "Cannot inspect pending cycle rollovers.".to_owned())?,
    );
    drop(retry);
    elapsed.sort_by_key(|(_, index, _)| *index);
    for (id, index, policy) in elapsed {
        let destination: Option<String> = connection
            .query_row(
                "SELECT id FROM cycle WHERE series_id = ?1 AND schedule_index > ?2
                 AND status != 'completed' AND end_at > ?3
                 ORDER BY schedule_index LIMIT 1",
                params![settings.id, index, now],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "Cannot read the next cycle.".to_owned())?;
        capture_cycle_once(connection, &id)?;
        if policy == "automatic" {
            connection
                .execute(
                    "UPDATE cycle_transition
                 SET last_retried_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE source_cycle_id = ?1",
                    [&id],
                )
                .map_err(|_| "Cannot record cycle rollover attempt.".to_owned())?;
        }
        if policy == "ask" || (policy == "automatic" && destination.is_none()) {
            continue;
        }
        complete_cycle(
            connection,
            &id,
            if policy == "automatic" {
                destination.as_deref()
            } else {
                None
            },
        )
        .or_else(|error| {
            if error == "Cycle is already completed."
                || error == "Next cycle does not include all unfinished issue projects."
            {
                Ok(())
            } else {
                Err(error)
            }
        })?;
    }
    let pending: i64 = connection
        .query_row(
            "SELECT count(*) FROM cycle WHERE series_id = ?1 AND status != 'completed'
         AND end_at <= ?2 AND NOT EXISTS
           (SELECT 1 FROM cycle_transition WHERE source_cycle_id = cycle.id)",
            params![settings.id, now],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot count remaining cycles.".to_owned())?;
    connection
        .execute(
            "UPDATE cycle_reconcile_progress SET processed = total - ?2,
         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE series_id = ?1",
            params![settings.id, pending],
        )
        .map_err(|_| "Cannot update cycle catch-up progress.".to_owned())?;
    if pending == 0 {
        connection
            .execute(
                "DELETE FROM cycle_reconcile_progress WHERE series_id = ?1",
                [&settings.id],
            )
            .map_err(|_| "Cannot finish cycle catch-up.".to_owned())?;
    }
    Ok(())
}

fn reconcile_manual_cycles(connection: &mut Connection) -> Result<(), String> {
    // Manual cycles have no series index. Only a designated successor may receive unfinished
    // work automatically; an unresolved successor remains pending for user review.
    let workspace_id = current_workspace_id(connection)?;
    let now = utc_now();
    let mut statement = connection
        .prepare(
            "SELECT id, rollover_policy, successor_id FROM cycle
         WHERE workspace_id = ?1 AND schedule_index IS NULL AND status != 'completed'
           AND end_at <= ?2 ORDER BY end_at, id",
        )
        .map_err(|_| "Cannot inspect elapsed manual cycles.".to_owned())?;
    let elapsed = statement
        .query_map(params![workspace_id, now], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(|_| "Cannot inspect elapsed manual cycles.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot inspect elapsed manual cycles.".to_owned())?;
    drop(statement);
    for (id, policy, successor) in elapsed {
        capture_cycle_once(connection, &id)?;
        if policy == "leave" {
            complete_cycle(connection, &id, None)?;
        } else if policy == "automatic" {
            if let Some(successor) = successor {
                complete_cycle(connection, &id, Some(&successor)).or_else(|error| {
                    if error == "Next cycle does not include all unfinished issue projects." {
                        Ok(())
                    } else {
                        Err(error)
                    }
                })?;
            }
        }
    }
    Ok(())
}

fn set_cycle_successor(
    connection: &Connection,
    cycle_id: &str,
    successor_id: Option<&str>,
) -> Result<(), String> {
    let workspace_id = current_workspace_id(connection)?;
    let source_end: String = connection
        .query_row(
            "SELECT end_at FROM cycle WHERE id = ?1 AND workspace_id = ?2
             AND schedule_index IS NULL AND status != 'completed'",
            params![cycle_id, workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot inspect manual cycle.".to_owned())?
        .ok_or("Open manual cycle not found.")?;
    if let Some(successor_id) = successor_id {
        let successor_start: String = connection
            .query_row(
                "SELECT start_at FROM cycle WHERE id = ?1 AND workspace_id = ?2
                 AND schedule_index IS NULL AND status != 'completed'",
                params![successor_id, workspace_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "Cannot inspect manual successor.".to_owned())?
            .ok_or("Open manual successor not found.")?;
        if source_end > successor_start {
            return Err("Successor must start after this cycle ends.".into());
        }
    }
    connection
        .execute(
            "UPDATE cycle SET successor_id = ?1 WHERE id = ?2 AND workspace_id = ?3",
            params![successor_id, cycle_id, workspace_id],
        )
        .map_err(|_| "Cannot save manual successor.".to_owned())?;
    Ok(())
}

pub(crate) fn reconcile_current_workspace(connection: &mut Connection) -> Result<(), String> {
    if current_workspace_id(connection).is_err() {
        return Ok(());
    }
    reconcile_cycles(connection)
}

fn valid_settings(settings: &CycleSettings) -> bool {
    let dates_fit = || {
        let anchor = NaiveDate::parse_from_str(&settings.start_date, "%Y-%m-%d").ok()?;
        let today =
            NaiveDate::parse_from_str(&local_date_now(&settings.time_zone).ok()?, "%Y-%m-%d")
                .ok()?;
        let last = NaiveDate::from_ymd_opt(9999, 12, 31)?;
        let available = (last - anchor.max(today)).num_days();
        Some(settings.interval_days <= available / 13)
    };

    !settings.name.trim().is_empty()
        && settings.name.chars().count() <= 80
        && (1..=56).contains(&settings.duration_days)
        && settings.interval_days >= settings.duration_days
        && valid_date(&settings.start_date)
        && valid_zone(&settings.time_zone)
        && dates_fit() == Some(true)
        && valid_time(&settings.start_local_time)
        && matches!(
            settings.rollover_policy.as_str(),
            "automatic" | "leave" | "ask"
        )
}

pub(crate) fn configure_cycles(
    connection: &mut Connection,
    settings: CycleSettings,
) -> Result<(), String> {
    if !valid_settings(&settings) {
        return Err("Choose a valid cycle schedule.".into());
    }
    validate_project_scope(connection, &settings.project_ids)?;
    let workspace_id = current_workspace_id(connection)?;
    let scope = serde_json::to_string(&settings.project_ids)
        .map_err(|_| "Cannot encode cycle projects.")?;
    connection
        .execute(
            "INSERT INTO cycle_series (id, workspace_id, name, duration_days, interval_days,
          cooldown_days, upcoming_count, start_date, rollover_policy, project_scope_json,
          time_zone, start_local_time, enabled)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, 12, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                Uuid::new_v4().to_string(),
                workspace_id,
                settings.name.trim(),
                settings.duration_days,
                settings.interval_days,
                settings.start_date,
                settings.rollover_policy,
                scope,
                settings.time_zone,
                settings.start_local_time,
                settings.enabled
            ],
        )
        .map_err(|_| "Cannot create cycle series. Choose a unique name.".to_owned())?;
    reconcile_cycles(connection)
}

fn set_cycle_series_enabled(
    connection: &mut Connection,
    series_id: &str,
    enabled: bool,
) -> Result<(), String> {
    let workspace_id = current_workspace_id(connection)?;
    let changed = connection
        .execute(
            "UPDATE cycle_series SET enabled = ?1 WHERE id = ?2 AND workspace_id = ?3",
            params![enabled, series_id, workspace_id],
        )
        .map_err(|_| "Cannot change cycle series state.".to_owned())?;
    if changed == 0 {
        return Err("Cycle series not found.".into());
    }
    if enabled {
        reconcile_cycles(connection)?;
    }
    Ok(())
}

fn list_reconcile_progress(connection: &Connection) -> Result<Vec<CycleReconcileProgress>, String> {
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT progress.series_id, progress.total, progress.processed
         FROM cycle_reconcile_progress progress
         JOIN cycle_series series ON series.id = progress.series_id
         WHERE series.workspace_id = ?1 ORDER BY series.name",
        )
        .map_err(|_| "Cannot read cycle catch-up progress.".to_owned())?;
    let progress = statement
        .query_map([workspace_id], |row| {
            Ok(CycleReconcileProgress {
                series_id: row.get(0)?,
                total: row.get(1)?,
                processed: row.get(2)?,
            })
        })
        .map_err(|_| "Cannot read cycle catch-up progress.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read cycle catch-up progress.".to_owned())?;
    Ok(progress)
}

fn update_cycle_schedule(
    connection: &mut Connection,
    settings: CycleSettings,
) -> Result<(), String> {
    if !valid_settings(&settings) || Uuid::parse_str(&settings.id).is_err() {
        return Err("Choose a valid cycle schedule.".into());
    }
    validate_project_scope(connection, &settings.project_ids)?;
    let workspace_id = current_workspace_id(connection)?;
    let existing = list_cycle_series(connection)?
        .into_iter()
        .find(|series| series.id == settings.id)
        .ok_or("Cycle series not found.")?;
    reconcile_series(connection, &existing)?;
    let today = local_date_now(&settings.time_zone)?;
    let locked: Option<(i64, String)> = connection
        .query_row(
            "SELECT schedule_index, end_date FROM cycle
         WHERE series_id = ?1 AND (start_date <= ?2 OR status = 'completed')
         ORDER BY schedule_index DESC LIMIT 1",
            params![settings.id, today],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| "Cannot inspect active cycles.".to_owned())?;
    let first_index = locked.as_ref().map_or(0, |(index, _)| index + 1);
    if let Some((_, end)) = &locked {
        if settings.start_date <= *end {
            return Err("Next cycle must start after the current cycle ends.".into());
        }
    }
    if settings.start_date <= today {
        return Err("Next cycle must start in the future.".into());
    }
    let span = settings.interval_days;
    let scope = serde_json::to_string(&settings.project_ids)
        .map_err(|_| "Cannot encode cycle projects.")?;
    let anchor = offset_date(connection, &settings.start_date, -first_index * span)?;
    let mut future = connection
        .prepare(
            "SELECT id, schedule_index FROM cycle
         WHERE series_id = ?1 AND schedule_index >= ?2 AND status != 'completed'
         ORDER BY schedule_index",
        )
        .map_err(|_| "Cannot inspect upcoming cycles.".to_owned())?;
    let rows = future
        .query_map(params![settings.id, first_index], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|_| "Cannot inspect upcoming cycles.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot inspect upcoming cycles.".to_owned())?;
    drop(future);

    if !settings.project_ids.is_empty() {
        for (id, _) in &rows {
            let incompatible: i64 = connection
                .query_row(
                    "SELECT count(*) FROM cycle_issue JOIN issue ON issue.id = cycle_issue.issue_id
                 WHERE cycle_issue.cycle_id = ?1 AND COALESCE(issue.project_id, '')
                   NOT IN (SELECT value FROM json_each(?2))",
                    params![id, scope],
                    |row| row.get(0),
                )
                .map_err(|_| "Cannot validate future cycle projects.".to_owned())?;
            if incompatible > 0 {
                return Err("Move affected issues before changing future cycle projects.".into());
            }
        }
    }

    let dates = rows
        .into_iter()
        .map(|(id, index)| {
            let start = offset_date(
                connection,
                &settings.start_date,
                (index - first_index) * span,
            )?;
            let end = offset_date(connection, &start, settings.duration_days - 1)?;
            let start_at = boundary(&start, &settings.start_local_time, &settings.time_zone)?;
            let end_at = boundary(
                &offset_date(connection, &end, 1)?,
                &settings.start_local_time,
                &settings.time_zone,
            )?;
            Ok((id, start, end, start_at, end_at))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let tx = connection
        .savepoint()
        .map_err(|_| "Cannot begin schedule edit.".to_owned())?;
    tx.execute(
        "UPDATE cycle_series SET name = ?1, duration_days = ?2, interval_days = ?3,
         cooldown_days = 0, upcoming_count = 12, start_date = ?4, rollover_policy = ?5,
         project_scope_json = ?6, time_zone = ?7, start_local_time = ?8, enabled = ?9
         WHERE id = ?10 AND workspace_id = ?11",
        params![
            settings.name.trim(),
            settings.duration_days,
            settings.interval_days,
            anchor,
            settings.rollover_policy,
            scope,
            settings.time_zone,
            settings.start_local_time,
            settings.enabled,
            settings.id,
            workspace_id
        ],
    )
    .map_err(|_| "Cannot update cycle series. Choose a unique name.".to_owned())?;
    for (id, start, end, start_at, end_at) in dates {
        tx.execute(
            "UPDATE cycle SET start_date = ?1, end_date = ?2, project_scope_json = ?3,
             time_zone = ?5, start_local_time = ?6, end_local_time = ?6, start_at = ?7, end_at = ?8
             WHERE id = ?4",
            params![
                start,
                end,
                scope,
                id,
                settings.time_zone,
                settings.start_local_time,
                start_at,
                end_at
            ],
        )
        .map_err(|_| "Cannot reschedule upcoming cycle.".to_owned())?;
    }
    tx.commit()
        .map_err(|_| "Cannot save cycle schedule.".to_owned())?;
    reconcile_cycles(connection)
}

fn list_cycles(connection: &Connection) -> Result<Vec<CycleRecord>, String> {
    let workspace_id = current_workspace_id(connection)?;
    let mut statement = connection
        .prepare(
            "SELECT id, title, description, start_date, end_date,
                CASE WHEN status = 'completed' THEN 'completed'
                     WHEN start_at <= strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                      AND end_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now') THEN 'active'
                     ELSE 'planned' END,
                schedule_index IS NOT NULL, completed_count, total_count, project_scope_json,
                rollover_policy, (SELECT result FROM cycle_transition WHERE source_cycle_id = cycle.id),
                time_zone, start_local_time, end_local_time, start_at, end_at,
                (schedule_index IS NULL AND end_at != (date(end_date, '+1 day') || 'T00:00:00Z')),
                series_id, successor_id
         FROM cycle WHERE workspace_id = ?1 ORDER BY start_date DESC, created_at DESC",
        )
        .map_err(|_| "Cannot read cycles.".to_owned())?;
    let rows = statement
        .query_map([&workspace_id], |row| {
            let scope: String = row.get(9)?;
            Ok(CycleRecord {
                id: row.get(0)?,
                version: 0,
                title: row.get(1)?,
                description: row.get(2)?,
                start_date: row.get(3)?,
                end_date: row.get(4)?,
                status: row.get(5)?,
                scheduled: row.get(6)?,
                series_id: row.get(18)?,
                successor_id: row.get(19)?,
                issue_ids: Vec::new(),
                completed_count: row.get(7)?,
                total_count: row.get(8)?,
                project_ids: serde_json::from_str(&scope)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                rollover_policy: row.get(10)?,
                transition_result: row.get(11)?,
                time_zone: row.get(12)?,
                start_local_time: row.get(13)?,
                end_local_time: row.get(14)?,
                start_at: row.get(15)?,
                end_at: row.get(16)?,
                precise_end: row.get(17)?,
            })
        })
        .map_err(|_| "Cannot read cycles.".to_owned())?;
    let mut cycles = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot read cycles.".to_owned())?;
    for cycle in &mut cycles {
        let mut members = connection
            .prepare(
                "SELECT identifier FROM cycle_snapshot_issue WHERE cycle_id = ?1 AND ?2 = 'completed'
                 UNION ALL
                 SELECT issue.identifier FROM cycle_issue JOIN issue ON issue.id = cycle_issue.issue_id
                 WHERE cycle_issue.cycle_id = ?1 AND ?2 != 'completed'
                 ORDER BY identifier",
            )
            .map_err(|_| "Cannot read cycle issues.".to_owned())?;
        cycle.issue_ids = members
            .query_map(params![cycle.id, cycle.status], |row| row.get(0))
            .map_err(|_| "Cannot read cycle issues.".to_owned())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "Cannot read cycle issues.".to_owned())?;
        let encoded =
            serde_json::to_vec(cycle).map_err(|_| "Cannot encode cycle version.".to_owned())?;
        let digest = Sha256::digest(encoded);
        cycle.version =
            (i64::from_be_bytes(digest[..8].try_into().unwrap()) & ((1_i64 << 53) - 1)).max(1);
    }
    Ok(cycles)
}

fn cycle_version(connection: &Connection, id: &str) -> Result<i64, String> {
    list_cycles(connection)?
        .into_iter()
        .find(|cycle| cycle.id == id)
        .map(|cycle| cycle.version)
        .ok_or("Cycle not found.".into())
}

#[cfg(test)]
fn create_cycle(
    connection: &mut Connection,
    title: &str,
    start_date: &str,
    end_date: &str,
) -> Result<CycleRecord, String> {
    create_cycle_scoped(connection, title, start_date, end_date, &[])
}

#[cfg(test)]
fn create_cycle_scoped(
    connection: &mut Connection,
    title: &str,
    start_date: &str,
    end_date: &str,
    project_ids: &[String],
) -> Result<CycleRecord, String> {
    create_cycle_timed(
        connection,
        title,
        start_date,
        end_date,
        project_ids,
        "UTC",
        "00:00",
        "00:00",
        "ask",
        false,
    )
}

fn create_cycle_timed(
    connection: &Connection,
    title: &str,
    start_date: &str,
    end_date: &str,
    project_ids: &[String],
    zone: &str,
    start_time: &str,
    end_time: &str,
    policy: &str,
    precise_end: bool,
) -> Result<CycleRecord, String> {
    let title = title.trim();
    if title.is_empty()
        || title.chars().count() > 80
        || !valid_date(start_date)
        || !valid_date(end_date)
        || start_date > end_date
        || !valid_zone(zone)
        || !valid_time(start_time)
        || !valid_time(end_time)
        || !matches!(policy, "ask" | "automatic" | "leave")
    {
        return Err("Enter a title and a valid cycle date range.".into());
    }
    let workspace_id = current_workspace_id(connection)?;
    validate_project_scope(connection, project_ids)?;
    let scope = serde_json::to_string(project_ids).map_err(|_| "Cannot encode cycle projects.")?;
    let id = Uuid::new_v4().to_string();
    let start_at = boundary(start_date, start_time, zone)?;
    let end_boundary_date = if precise_end {
        end_date.to_owned()
    } else {
        offset_date(connection, end_date, 1)?
    };
    let end_at = boundary(&end_boundary_date, end_time, zone)?;
    if end_at <= start_at {
        return Err("Cycle end must be after its start.".into());
    }
    connection.execute(
        "INSERT INTO cycle (id, workspace_id, title, start_date, end_date, project_scope_json, start_at, end_at, time_zone, start_local_time, end_local_time, rollover_policy) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![id, workspace_id, title, start_date, end_date, scope, start_at, end_at, zone, start_time, end_time, policy],
    ).map_err(|_| "Cannot create the cycle.".to_owned())?;
    list_cycles(connection)?
        .into_iter()
        .find(|cycle| cycle.id == id)
        .ok_or("Cannot confirm the new cycle.".into())
}

#[cfg(test)]
fn update_cycle(
    connection: &mut Connection,
    cycle_id: &str,
    title: &str,
    description: &str,
    start_date: &str,
    end_date: &str,
) -> Result<CycleRecord, String> {
    let scope: String = connection
        .query_row(
            "SELECT project_scope_json FROM cycle WHERE id = ?1",
            [cycle_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cycle not found.".to_owned())?;
    let project_ids: Vec<String> =
        serde_json::from_str(&scope).map_err(|_| "Cannot read cycle projects.")?;
    update_cycle_scoped(
        connection,
        cycle_id,
        title,
        description,
        start_date,
        end_date,
        &project_ids,
    )
}

#[cfg(test)]
fn update_cycle_scoped(
    connection: &mut Connection,
    cycle_id: &str,
    title: &str,
    description: &str,
    start_date: &str,
    end_date: &str,
    project_ids: &[String],
) -> Result<CycleRecord, String> {
    update_cycle_timed(
        connection,
        cycle_id,
        title,
        description,
        start_date,
        end_date,
        project_ids,
        "UTC",
        "00:00",
        "00:00",
        "ask",
        false,
        None,
    )
}

fn update_cycle_timed(
    connection: &Connection,
    cycle_id: &str,
    title: &str,
    description: &str,
    start_date: &str,
    end_date: &str,
    project_ids: &[String],
    zone: &str,
    start_time: &str,
    end_time: &str,
    policy: &str,
    precise_end: bool,
    successor_id: Option<&str>,
) -> Result<CycleRecord, String> {
    let title = title.trim();
    if title.is_empty()
        || title.chars().count() > 80
        || description.chars().count() > 4_000
        || !valid_date(start_date)
        || !valid_date(end_date)
        || start_date > end_date
        || !valid_zone(zone)
        || !valid_time(start_time)
        || !valid_time(end_time)
        || !matches!(policy, "ask" | "automatic" | "leave")
    {
        return Err("Enter a title, description, and valid date range.".into());
    }
    let workspace_id = current_workspace_id(connection)?;
    validate_project_scope(connection, project_ids)?;
    let scope = serde_json::to_string(project_ids).map_err(|_| "Cannot encode cycle projects.")?;
    let start_at = boundary(start_date, start_time, zone)?;
    let end_boundary_date = if precise_end {
        end_date.to_owned()
    } else {
        offset_date(connection, end_date, 1)?
    };
    let end_at = boundary(&end_boundary_date, end_time, zone)?;
    if end_at <= start_at {
        return Err("Cycle end must be after its start.".into());
    }
    let scheduled: bool = connection
        .query_row(
            "SELECT schedule_index IS NOT NULL FROM cycle WHERE id = ?1 AND workspace_id = ?2",
            params![cycle_id, workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot inspect cycle schedule.".to_owned())?
        .ok_or("Cycle not found.")?;
    if scheduled && successor_id.is_some() {
        return Err("Only manual cycles can have a designated successor.".into());
    }
    if let Some(successor_id) = successor_id {
        let successor_start: String = connection
            .query_row(
                "SELECT start_at FROM cycle WHERE id = ?1 AND workspace_id = ?2
                 AND schedule_index IS NULL AND status != 'completed'",
                params![successor_id, workspace_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "Cannot inspect manual successor.".to_owned())?
            .ok_or("Open manual successor not found.")?;
        if end_at > successor_start {
            return Err("Successor must start after this cycle ends.".into());
        }
    }
    let predecessor_end: Option<String> = connection
        .query_row(
            "SELECT max(end_at) FROM cycle WHERE successor_id = ?1 AND status != 'completed'",
            [cycle_id],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot inspect manual predecessors.".to_owned())?;
    if predecessor_end.is_some_and(|end| end > start_at) {
        return Err("This cycle must start after its predecessor ends.".into());
    }
    if !project_ids.is_empty() {
        let incompatible: i64 = connection.query_row(
            "SELECT count(*) FROM cycle_issue JOIN issue ON issue.id = cycle_issue.issue_id
             WHERE cycle_issue.cycle_id = ?1 AND issue.project_id NOT IN (SELECT value FROM json_each(?2))",
            params![cycle_id, scope], |row| row.get(0),
        ).map_err(|_| "Cannot validate cycle projects.".to_owned())?;
        if incompatible > 0 {
            return Err("Remove out-of-scope issues before changing cycle projects.".into());
        }
    }
    let changed = connection
        .execute(
            "UPDATE cycle SET title = ?1, description = ?2,
         start_date = CASE WHEN schedule_index IS NULL THEN ?3 ELSE start_date END,
         end_date = CASE WHEN schedule_index IS NULL THEN ?4 ELSE end_date END,
         start_at = CASE WHEN schedule_index IS NULL THEN ?8 ELSE start_at END,
         end_at = CASE WHEN schedule_index IS NULL THEN ?9 ELSE end_at END,
         time_zone = CASE WHEN schedule_index IS NULL THEN ?10 ELSE time_zone END,
         start_local_time = CASE WHEN schedule_index IS NULL THEN ?11 ELSE start_local_time END,
         end_local_time = CASE WHEN schedule_index IS NULL THEN ?12 ELSE end_local_time END,
         rollover_policy = ?13,
         successor_id = CASE WHEN schedule_index IS NULL THEN ?14 ELSE successor_id END,
         project_scope_json = ?5
         WHERE id = ?6 AND workspace_id = ?7 AND status != 'completed'
           AND NOT EXISTS (SELECT 1 FROM cycle_transition WHERE source_cycle_id = ?6)",
            params![
                title,
                description.trim(),
                start_date,
                end_date,
                scope,
                cycle_id,
                workspace_id,
                start_at,
                end_at,
                zone,
                start_time,
                end_time,
                policy,
                successor_id
            ],
        )
        .map_err(|_| "Cannot update the cycle.".to_owned())?;
    if changed == 0 {
        return Err("Open cycle not found.".into());
    }
    list_cycles(connection)?
        .into_iter()
        .find(|cycle| cycle.id == cycle_id)
        .ok_or("Cannot confirm the updated cycle.".into())
}

#[cfg(test)]
fn set_cycle_issue(
    connection: &mut Connection,
    cycle_id: &str,
    identifier: &str,
    assigned: bool,
) -> Result<(), String> {
    let tx = connection
        .savepoint()
        .map_err(|_| "Cannot begin cycle assignment.".to_owned())?;
    set_cycle_issue_in_tx(&tx, cycle_id, identifier, assigned)?;
    tx.commit()
        .map_err(|_| "Cannot save cycle assignment.".to_owned())
}

fn set_cycle_issue_in_tx(
    connection: &Connection,
    cycle_id: &str,
    identifier: &str,
    assigned: bool,
) -> Result<(), String> {
    let workspace_id = current_workspace_id(connection)?;
    let (cycle_status, project_scope, transition_pending): (String, String, bool) = connection
        .query_row(
            "SELECT status, project_scope_json,
              EXISTS(SELECT 1 FROM cycle_transition WHERE source_cycle_id = cycle.id)
             FROM cycle WHERE workspace_id = ?1 AND id = ?2",
            params![workspace_id, cycle_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| "Cannot read the cycle.".to_owned())?
        .ok_or("Cycle not found.")?;
    if cycle_status == "completed" {
        return Err("Completed cycles cannot be changed.".into());
    }
    if assigned && transition_pending {
        return Err("This cycle has ended. Choose another cycle for new work.".into());
    }
    let (issue_id, issue_project): (String, Option<String>) = connection.query_row(
        "SELECT id, project_id FROM issue WHERE workspace_id = ?1 AND identifier = ?2 AND archived_at IS NULL",
        params![workspace_id, identifier], |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional().map_err(|_| "Cannot read the issue.".to_owned())?.ok_or("Active issue not found.")?;
    let projects: Vec<String> =
        serde_json::from_str(&project_scope).map_err(|_| "Cannot read cycle projects.")?;
    if assigned
        && !projects.is_empty()
        && !issue_project
            .as_ref()
            .is_some_and(|id| projects.contains(id))
    {
        return Err("This issue is outside the cycle's projects.".into());
    }
    if assigned {
        connection
            .execute(
                "DELETE FROM cycle_issue WHERE issue_id = ?1 AND cycle_id IN
             (SELECT id FROM cycle WHERE workspace_id = ?2 AND status != 'completed' AND id != ?3)",
                params![issue_id, workspace_id, cycle_id],
            )
            .map_err(|_| "Cannot move the issue from its previous cycle.".to_owned())?;
        connection.execute(
            "INSERT OR IGNORE INTO cycle_issue (cycle_id, issue_id) VALUES (?1, ?2)",
            params![cycle_id, issue_id],
        )
    } else {
        connection.execute(
            "DELETE FROM cycle_issue WHERE cycle_id = ?1 AND issue_id = ?2",
            params![cycle_id, issue_id],
        )
    }
    .map_err(|_| "Cannot update cycle issues.".to_owned())?;
    Ok(())
}

// A boundary is captured once, even when the app resumes days later or the user
// waits to decide where unfinished work belongs.
fn capture_cycle_transition(connection: &Connection, cycle_id: &str) -> Result<(), String> {
    let (end_at, policy): (String, String) = connection
        .query_row(
            "SELECT end_at, rollover_policy FROM cycle WHERE id = ?1",
            [cycle_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Cycle not found.".to_owned())?;
    let captured_at = utc_now();
    let boundary_at = if end_at > captured_at {
        &captured_at
    } else {
        &end_at
    };
    connection.execute(
        "INSERT OR IGNORE INTO cycle_transition(source_cycle_id, policy, boundary_at, captured_at, result)
         VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), 'pending')",
        params![cycle_id, policy, boundary_at],
    ).map_err(|_| "Cannot capture cycle boundary.".to_owned())?;
    connection.execute(
        "INSERT OR IGNORE INTO cycle_snapshot_issue(cycle_id, issue_id, identifier, status, estimate_value, completed)
         SELECT ?1, issue.id, issue.identifier, issue.status, issue.estimate_value,
                CASE WHEN workflow.category = 'Completed' THEN 1 ELSE 0 END
         FROM cycle_issue
         JOIN issue ON issue.id = cycle_issue.issue_id
         LEFT JOIN workflow_status workflow ON workflow.workspace_id = issue.workspace_id AND workflow.name = issue.status
         WHERE cycle_issue.cycle_id = ?1",
        [cycle_id],
    ).map_err(|_| "Cannot snapshot cycle work.".to_owned())?;
    Ok(())
}

fn capture_cycle_once(connection: &mut Connection, cycle_id: &str) -> Result<(), String> {
    let tx = connection
        .savepoint()
        .map_err(|_| "Cannot begin cycle capture.".to_owned())?;
    capture_cycle_transition(&tx, cycle_id)?;
    tx.commit()
        .map_err(|_| "Cannot commit cycle capture.".to_owned())
}

fn complete_cycle(
    connection: &mut Connection,
    cycle_id: &str,
    next_cycle_id: Option<&str>,
) -> Result<(), String> {
    complete_cycle_selected(connection, cycle_id, next_cycle_id, None)
}

fn complete_cycle_selected(
    connection: &mut Connection,
    cycle_id: &str,
    next_cycle_id: Option<&str>,
    selected_identifiers: Option<&[String]>,
) -> Result<(), String> {
    let tx = connection
        .savepoint()
        .map_err(|_| "Cannot begin cycle completion.".to_owned())?;
    complete_cycle_selected_in_tx(&tx, cycle_id, next_cycle_id, selected_identifiers)?;
    tx.commit()
        .map_err(|_| "Cannot save cycle completion.".to_owned())
}

fn complete_cycle_selected_in_tx(
    tx: &Connection,
    cycle_id: &str,
    next_cycle_id: Option<&str>,
    selected_identifiers: Option<&[String]>,
) -> Result<(), String> {
    let workspace_id = current_workspace_id(tx)?;
    let status: String = tx
        .query_row(
            "SELECT status FROM cycle WHERE workspace_id = ?1 AND id = ?2",
            params![workspace_id, cycle_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Cannot read the cycle.".to_owned())?
        .ok_or("Cycle not found.")?;
    if status == "completed" {
        return Err("Cycle is already completed.".into());
    }
    capture_cycle_transition(&tx, cycle_id)?;
    if let Some(selected) = selected_identifiers {
        if selected.len() > 10_000 {
            return Err("Too many rollover issues were selected.".into());
        }
        for identifier in selected {
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM cycle_snapshot_issue WHERE cycle_id = ?1 AND identifier = ?2)",
                params![cycle_id, identifier], |row| row.get(0),
            ).map_err(|_| "Cannot validate rollover selection.".to_owned())?;
            if !exists {
                return Err("A selected issue is not in this cycle.".into());
            }
        }
    }
    let selection_json = selected_identifiers
        .map(|ids| serde_json::to_string(ids))
        .transpose()
        .map_err(|_| "Cannot encode rollover selection.")?;
    let (total_count, completed_count): (i64, i64) = tx
        .query_row(
            "SELECT count(*), COALESCE(sum(completed), 0)
         FROM cycle_snapshot_issue WHERE cycle_id = ?1",
            [cycle_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Cannot snapshot cycle progress.".to_owned())?;
    if let Some(next_id) = next_cycle_id {
        if next_id == cycle_id {
            return Err("Choose a different cycle for rollover.".into());
        }
        let (next_status, next_scope): (String, String) = tx
            .query_row(
                "SELECT status, project_scope_json FROM cycle WHERE workspace_id = ?1 AND id = ?2",
                params![workspace_id, next_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|_| "Cannot read the next cycle.".to_owned())?
            .ok_or("Next cycle not found.")?;
        if next_status == "completed" {
            return Err("Choose an open cycle for rollover.".into());
        }
        let incompatible: i64 = tx.query_row(
            "SELECT count(*) FROM cycle_issue
             JOIN issue ON issue.id = cycle_issue.issue_id
             LEFT JOIN workflow_status workflow ON workflow.workspace_id = issue.workspace_id AND workflow.name = issue.status
             WHERE cycle_issue.cycle_id = ?1 AND issue.archived_at IS NULL AND issue.triage = 0
               AND COALESCE(workflow.category, 'Unstarted') NOT IN ('Completed', 'Canceled')
               AND (?3 IS NULL OR issue.identifier IN (SELECT value FROM json_each(?3)))
               AND ?2 != '[]' AND COALESCE(issue.project_id, '') NOT IN (SELECT value FROM json_each(?2))",
            params![cycle_id, next_scope, selection_json], |row| row.get(0),
        ).map_err(|_| "Cannot validate rollover scope.".to_owned())?;
        if incompatible > 0 {
            return Err("Next cycle does not include all unfinished issue projects.".into());
        }
        tx.execute(
            "INSERT OR IGNORE INTO cycle_issue (cycle_id, issue_id)
             SELECT ?1, issue.id FROM cycle_issue
             JOIN issue ON issue.id = cycle_issue.issue_id
             LEFT JOIN workflow_status workflow ON workflow.workspace_id = issue.workspace_id AND workflow.name = issue.status
             WHERE cycle_issue.cycle_id = ?2 AND issue.archived_at IS NULL
               AND issue.triage = 0
               AND (?3 = '[]' OR issue.project_id IN (SELECT value FROM json_each(?3)))
               AND COALESCE(workflow.category, 'Unstarted') NOT IN ('Completed', 'Canceled')
               AND (?4 IS NULL OR issue.identifier IN (SELECT value FROM json_each(?4)))",
            params![next_id, cycle_id, next_scope, selection_json],
        ).map_err(|_| "Cannot roll over unfinished issues.".to_owned())?;
    }
    tx.execute(
        "UPDATE cycle_snapshot_issue AS snapshot SET rollover_decision =
           CASE
             WHEN NOT EXISTS (SELECT 1 FROM cycle_issue member WHERE member.cycle_id = ?1 AND member.issue_id = snapshot.issue_id)
               THEN 'skipped'
             WHEN ?2 IS NULL OR (?3 IS NOT NULL AND snapshot.identifier NOT IN (SELECT value FROM json_each(?3)))
               THEN 'left'
             WHEN EXISTS (SELECT 1 FROM cycle_issue member WHERE member.cycle_id = ?2 AND member.issue_id = snapshot.issue_id)
               THEN 'moved'
             ELSE 'skipped'
           END
         WHERE snapshot.cycle_id = ?1 AND snapshot.completed = 0",
        params![cycle_id, next_cycle_id, selection_json],
    ).map_err(|_| "Cannot record rollover decisions.".to_owned())?;
    tx.execute(
        "UPDATE cycle SET status = 'completed', total_count = ?3, completed_count = ?4,
         completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE workspace_id = ?1 AND id = ?2",
        params![workspace_id, cycle_id, total_count, completed_count],
    )
    .map_err(|_| "Cannot complete the cycle.".to_owned())?;
    tx.execute(
        "UPDATE cycle_transition SET destination_cycle_id = ?2, result = 'completed' WHERE source_cycle_id = ?1",
        params![cycle_id, next_cycle_id],
    ).map_err(|_| "Cannot record cycle transition.".to_owned())?;
    Ok(())
}

#[api_contract_macros::command]
pub fn storage_list_cycles(app: AppHandle) -> Result<Vec<CycleRecord>, String> {
    let root = document_data_root(&app)?;
    let mut connection = open_database(&root)?;
    if !crate::portability::restore_catchup_pending(&root) {
        reconcile_cycles(&mut connection)?;
    }
    list_cycles(&connection)
}

#[api_contract_macros::command]
pub fn storage_list_cycle_series(app: AppHandle) -> Result<Vec<CycleSettingsView>, String> {
    list_cycle_series(&open_database(&document_data_root(&app)?)?)?
        .into_iter()
        .map(|settings| {
            Ok(CycleSettingsView {
                version: series_version(&settings)?,
                settings,
            })
        })
        .collect()
}

#[api_contract_macros::command]
pub fn storage_set_cycle_series_enabled(
    app: AppHandle,
    series_id: String,
    enabled: bool,
    mutation: MutationRequest,
) -> Result<(), String> {
    mutations::apply_with_connection(
        &mut open_database(&document_data_root(&app)?)?,
        "cycle.series.enable",
        &(&series_id, enabled),
        &mutation,
        |connection, _| {
            if mutation.expected_version != Some(current_series_version(connection, &series_id)?) {
                return Err("CONFLICT: Cycle series changed. Reload and try again.".into());
            }
            set_cycle_series_enabled(connection, &series_id, enabled)
        },
    )
}

#[api_contract_macros::command]
pub fn storage_list_cycle_reconcile_progress(
    app: AppHandle,
) -> Result<Vec<CycleReconcileProgress>, String> {
    list_reconcile_progress(&open_database(&document_data_root(&app)?)?)
}

#[api_contract_macros::command]
pub fn storage_configure_cycles(
    app: AppHandle,
    settings: CycleSettings,
    mutation: MutationRequest,
) -> Result<(), String> {
    mutations::apply_with_connection(
        &mut open_database(&document_data_root(&app)?)?,
        "cycle.series.create",
        &settings,
        &mutation,
        |connection, _| {
            if mutation.expected_version.is_some() {
                return Err("New cycle series cannot have an expected version.".into());
            }
            configure_cycles(connection, settings.clone())
        },
    )
}

#[api_contract_macros::command]
pub fn storage_update_cycle_schedule(
    app: AppHandle,
    settings: CycleSettings,
    mutation: MutationRequest,
) -> Result<(), String> {
    mutations::apply_with_connection(
        &mut open_database(&document_data_root(&app)?)?,
        "cycle.series.update",
        &settings,
        &mutation,
        |connection, _| {
            if mutation.expected_version != Some(current_series_version(connection, &settings.id)?)
            {
                return Err("CONFLICT: Cycle series changed. Reload and try again.".into());
            }
            update_cycle_schedule(connection, settings.clone())
        },
    )
}

#[api_contract_macros::command]
pub fn storage_create_cycle(
    app: AppHandle,
    title: String,
    start_date: String,
    end_date: String,
    project_ids: Vec<String>,
    time_zone: Option<String>,
    start_local_time: Option<String>,
    end_local_time: Option<String>,
    rollover_policy: Option<String>,
    mutation: MutationRequest,
) -> Result<CycleRecord, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "cycle.create",
        &(
            &title,
            &start_date,
            &end_date,
            &project_ids,
            &time_zone,
            &start_local_time,
            &end_local_time,
            &rollover_policy,
        ),
        &mutation,
        |tx, _| {
            if mutation.expected_version.is_some() {
                return Err("New cycles cannot have an expected version.".into());
            }
            create_cycle_timed(
                tx,
                &title,
                &start_date,
                &end_date,
                &project_ids,
                time_zone.as_deref().unwrap_or("UTC"),
                start_local_time.as_deref().unwrap_or("00:00"),
                end_local_time.as_deref().unwrap_or("00:00"),
                rollover_policy.as_deref().unwrap_or("ask"),
                end_local_time.is_some(),
            )
        },
    )
}

#[api_contract_macros::command]
pub fn storage_update_cycle(
    app: AppHandle,
    cycle_id: String,
    title: String,
    description: String,
    start_date: String,
    end_date: String,
    project_ids: Vec<String>,
    time_zone: Option<String>,
    start_local_time: Option<String>,
    end_local_time: Option<String>,
    rollover_policy: Option<String>,
    precise_end: Option<bool>,
    successor_id: Option<String>,
    mutation: MutationRequest,
) -> Result<CycleRecord, String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "cycle.update",
        &(
            &cycle_id,
            &title,
            &description,
            &start_date,
            &end_date,
            &project_ids,
            &time_zone,
            &start_local_time,
            &end_local_time,
            &rollover_policy,
            &precise_end,
            &successor_id,
        ),
        &mutation,
        |tx, _| {
            if mutation.expected_version != Some(cycle_version(tx, &cycle_id)?) {
                return Err("CONFLICT: Cycle changed. Reload and try again.".into());
            }
            update_cycle_timed(
                tx,
                &cycle_id,
                &title,
                &description,
                &start_date,
                &end_date,
                &project_ids,
                time_zone.as_deref().unwrap_or("UTC"),
                start_local_time.as_deref().unwrap_or("00:00"),
                end_local_time.as_deref().unwrap_or("00:00"),
                rollover_policy.as_deref().unwrap_or("ask"),
                precise_end.unwrap_or(end_local_time.is_some()),
                successor_id.as_deref(),
            )
        },
    )
}

#[api_contract_macros::command]
pub fn storage_set_cycle_issue(
    app: AppHandle,
    cycle_id: String,
    identifier: String,
    assigned: bool,
    mutation: MutationRequest,
) -> Result<(), String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "cycle.issue.set",
        &(&cycle_id, &identifier, assigned),
        &mutation,
        |tx, _| {
            if mutation.expected_version != Some(cycle_version(tx, &cycle_id)?) {
                return Err("CONFLICT: Cycle changed. Reload and try again.".into());
            }
            set_cycle_issue_in_tx(tx, &cycle_id, &identifier, assigned)
        },
    )
}

#[api_contract_macros::command]
pub fn storage_set_cycle_successor(
    app: AppHandle,
    cycle_id: String,
    successor_id: Option<String>,
    mutation: MutationRequest,
) -> Result<(), String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "cycle.successor.set",
        &(&cycle_id, &successor_id),
        &mutation,
        |tx, _| {
            if mutation.expected_version != Some(cycle_version(tx, &cycle_id)?) {
                return Err("CONFLICT: Cycle changed. Reload and try again.".into());
            }
            set_cycle_successor(tx, &cycle_id, successor_id.as_deref())
        },
    )
}

#[api_contract_macros::command]
pub fn storage_complete_cycle(
    app: AppHandle,
    cycle_id: String,
    next_cycle_id: Option<String>,
    selected_identifiers: Option<Vec<String>>,
    next_cycle_version: Option<i64>,
    mutation: MutationRequest,
) -> Result<(), String> {
    mutations::apply(
        &mut open_database(&document_data_root(&app)?)?,
        "cycle.complete",
        &(
            &cycle_id,
            &next_cycle_id,
            &selected_identifiers,
            next_cycle_version,
        ),
        &mutation,
        |tx, _| {
            if mutation.expected_version != Some(cycle_version(tx, &cycle_id)?) {
                return Err("CONFLICT: Cycle changed. Reload and try again.".into());
            }
            if let Some(next_id) = &next_cycle_id {
                if next_cycle_version != Some(cycle_version(tx, next_id)?) {
                    return Err("CONFLICT: Destination cycle changed. Reload and try again.".into());
                }
            }
            complete_cycle_selected_in_tx(
                tx,
                &cycle_id,
                next_cycle_id.as_deref(),
                selected_identifiers.as_deref(),
            )
        },
    )
}

#[cfg(test)]
mod tests;
