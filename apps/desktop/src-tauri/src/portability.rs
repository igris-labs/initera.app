use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

use crate::{
    mutations::{self, MutationRequest},
    storage::{current_workspace_id, document_data_root, open_database},
};

#[derive(Deserialize, Serialize)]
struct PendingRestoreReceipt {
    request: MutationRequest,
    operation: String,
    request_hash: String,
    ready: bool,
}

fn restore_receipt_path(root: &Path) -> PathBuf {
    root.join("recovery").join("restore-receipt.json")
}

fn sync_file(path: &Path) -> std::io::Result<()> {
    // FlushFileBuffers needs a writable handle on Windows.
    fs::OpenOptions::new().write(true).open(path)?.sync_all()
}

fn write_restore_receipt(root: &Path, receipt: &PendingRestoreReceipt) -> Result<(), String> {
    let path = restore_receipt_path(root);
    let temporary = path.with_extension("json.tmp");
    let bytes =
        serde_json::to_vec(receipt).map_err(|_| "Cannot encode restore receipt.".to_owned())?;
    fs::write(&temporary, bytes).map_err(|_| "Cannot stage restore receipt.".to_owned())?;
    sync_file(&temporary).map_err(|_| "Cannot secure restore receipt.".to_owned())?;
    fs::rename(&temporary, &path).map_err(|_| "Cannot save restore receipt.".to_owned())
}

pub(crate) fn stage_restore_receipt(
    root: &Path,
    request: &MutationRequest,
    operation: &str,
    request_hash: &str,
) -> Result<(), String> {
    if restore_receipt_path(root).exists() || root.join("recovery/restore.requested").exists() {
        return Err("CONFLICT: Finish the pending restore before starting another.".into());
    }
    write_restore_receipt(
        root,
        &PendingRestoreReceipt {
            request: request.clone(),
            operation: operation.to_owned(),
            request_hash: request_hash.to_owned(),
            ready: false,
        },
    )
}

pub(crate) fn mark_restore_receipt_ready(root: &Path) -> Result<(), String> {
    let path = restore_receipt_path(root);
    let bytes = fs::read(&path).map_err(|_| "Cannot read restore receipt.".to_owned())?;
    let mut receipt: PendingRestoreReceipt =
        serde_json::from_slice(&bytes).map_err(|_| "Cannot decode restore receipt.".to_owned())?;
    receipt.ready = true;
    write_restore_receipt(root, &receipt)
}

pub(crate) fn discard_restore_receipt(root: &Path) {
    let _ = fs::remove_file(restore_receipt_path(root));
    let _ = fs::remove_file(root.join("recovery/restore.requested"));
}

pub(crate) fn install_restore_receipt(
    root: &Path,
    restore_was_requested: bool,
) -> Result<(), String> {
    let path = restore_receipt_path(root);
    if !path.exists() {
        return Ok(());
    }
    let bytes = fs::read(&path).map_err(|_| "Cannot read restore receipt.".to_owned())?;
    let receipt: PendingRestoreReceipt =
        serde_json::from_slice(&bytes).map_err(|_| "Cannot decode restore receipt.".to_owned())?;
    if !receipt.ready && !restore_was_requested {
        fs::remove_file(path).map_err(|_| "Cannot clear incomplete restore receipt.".to_owned())?;
        return Ok(());
    }
    let completed = root.join("recovery/restore.completed");
    if !completed.exists() {
        if !receipt.ready || restore_was_requested {
            return Err("Restore receipt has no completed restore.".into());
        }
        // The database swap finished, but the process stopped before its completion marker.
        fs::write(&completed, "recovered")
            .map_err(|_| "Cannot mark completed restore.".to_owned())?;
    }
    let connection = open_database(root)?;
    let workspace_id = current_workspace_id(&connection)?;
    if mutations::replay::<()>(
        &connection,
        &workspace_id,
        &receipt.operation,
        &receipt.request_hash,
        &receipt.request,
    )?
    .is_none()
    {
        let _capture = crate::api_contract::begin_change_capture();
        crate::api_contract::record_change(&workspace_id, "workspace", false);
        mutations::save(
            &connection,
            &workspace_id,
            &receipt.operation,
            &receipt.request_hash,
            &receipt.request,
            &(),
        )?;
    }
    fs::remove_file(path).map_err(|_| "Cannot clear restore receipt.".to_owned())
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntry {
    id: String,
    created_at: u64,
    bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    checksum: String,
    workspace_name: String,
    projects: i64,
    issues: i64,
    cycles: i64,
    images: usize,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCatchup {
    processed: i64,
    total: i64,
    done: bool,
}

pub(crate) fn restore_catchup_pending(root: &Path) -> bool {
    root.join("recovery").join("restore.completed").exists()
}

fn backup_path(root: &Path, id: &str) -> Result<std::path::PathBuf, String> {
    Uuid::parse_str(id).map_err(|_| "Invalid backup identifier.".to_owned())?;
    let archive = root.join("backups").join(format!("{id}.zip"));
    if archive.exists() {
        Ok(archive)
    } else {
        Ok(root.join("backups").join(format!("{id}.sqlite")))
    }
}

fn validate_snapshot(path: &Path) -> Result<(), String> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| "Cannot read the selected backup.".to_owned())?;
    let integrity: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|_| "Cannot inspect the selected backup.".to_owned())?;
    let version: i32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| "Cannot inspect backup version.".to_owned())?;
    let application_id: i32 = connection
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(|_| "Cannot inspect backup application ID.".to_owned())?;
    let foreign_key_errors: i64 = connection
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .map_err(|_| "Cannot inspect backup relations.".to_owned())?;
    let required_tables: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name IN
             ('workspace', 'local_actor', 'project', 'issue', 'cycle', 'time_entry')",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot inspect backup structure.".to_owned())?;
    let search_tables: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table'
             AND name IN ('issue_search', 'project_search', 'comment_search')",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot inspect backup search indexes.".to_owned())?;
    let series_tables: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'cycle_series'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot inspect backup cycle series.".to_owned())?;
    let lifecycle_tables: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table'
         AND name IN ('project_resource', 'timer_recovery_notice')",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Cannot inspect backup resource and timer records.".to_owned())?;
    if integrity != "ok"
        || foreign_key_errors != 0
        || version != 1
        || application_id != 0x504d4150
        || required_tables != 6
        || search_tables != 3
        || series_tables != 1
        || lifecycle_tables != 2
    {
        return Err("Backup is damaged or uses an incompatible database version.".into());
    }
    Ok(())
}

fn create_backup(root: &Path) -> Result<BackupEntry, String> {
    let connection = open_database(root)?;
    let id = Uuid::new_v4().to_string();
    let snapshot = root.join("staging").join(format!("{id}.sqlite"));
    let temporary = root.join("staging").join(format!("{id}.zip.tmp"));
    let final_path = root.join("backups").join(format!("{id}.zip"));
    let result = (|| {
        connection
            .backup(rusqlite::MAIN_DB, &snapshot, None)
            .map_err(|error| format!("Cannot create backup: {error}"))?;
        validate_snapshot(&snapshot)?;
        let media = snapshot_media(root, &snapshot)?;
        let bytes = fs::read(&snapshot).map_err(|_| "Cannot read backup snapshot.".to_owned())?;
        let archive = crate::portable_archive::encode(&bytes, &media)?;
        fs::write(&temporary, archive).map_err(|_| "Cannot write backup archive.".to_owned())?;
        sync_file(&temporary).map_err(|_| "Cannot finish writing backup.".to_owned())?;
        fs::rename(&temporary, &final_path).map_err(|_| "Cannot finish backup.".to_owned())
    })();
    let _ = fs::remove_file(snapshot);
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result?;
    backup_entry(&final_path)
}

fn backup_entry(path: &Path) -> Result<BackupEntry, String> {
    let metadata = fs::metadata(path).map_err(|_| "Cannot inspect backup.".to_owned())?;
    let id = path
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or("Invalid backup name.")?
        .to_owned();
    let created_at = metadata
        .modified()
        .map_err(|_| "Cannot read backup date.".to_owned())?
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Cannot read backup date.".to_owned())?
        .as_secs();
    Ok(BackupEntry {
        id,
        created_at,
        bytes: metadata.len(),
    })
}

fn list_backups(root: &Path) -> Result<Vec<BackupEntry>, String> {
    let mut entries = Vec::new();
    for item in fs::read_dir(root.join("backups")).map_err(|_| "Cannot list backups.".to_owned())? {
        let path = item.map_err(|_| "Cannot list backups.".to_owned())?.path();
        if !matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("sqlite" | "zip")
        ) {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        if Uuid::parse_str(id).is_ok() {
            entries.push(backup_entry(&path)?);
        }
    }
    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(entries)
}

fn load_backup(root: &Path, id: &str) -> Result<crate::portable_archive::PortableData, String> {
    let path = backup_path(root, id)?;
    if path.extension().and_then(|value| value.to_str()) == Some("zip") {
        let bytes = fs::read(path).map_err(|_| "Cannot read the selected backup.".to_owned())?;
        crate::portable_archive::decode(&bytes)
    } else {
        validate_snapshot(&path)?;
        Ok(crate::portable_archive::PortableData {
            snapshot: fs::read(&path).map_err(|_| "Cannot read the selected backup.".to_owned())?,
            media: snapshot_media(root, &path)?,
        })
    }
}

fn stage_restore(
    root: &Path,
    portable: &crate::portable_archive::PortableData,
) -> Result<PathBuf, String> {
    // Validate the snapshot and every referenced image in staging before touching live files.
    let stage = root
        .join("staging")
        .join(format!("restore-{}", Uuid::new_v4()));
    fs::create_dir_all(stage.join("media"))
        .map_err(|_| "Cannot stage backup images.".to_owned())?;

    let result = (|| {
        let snapshot = stage.join("workspace.sqlite");
        fs::write(&snapshot, &portable.snapshot)
            .map_err(|_| "Cannot stage backup for restore.".to_owned())?;
        validate_snapshot(&snapshot)?;
        let connection = Connection::open_with_flags(&snapshot, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| "Cannot inspect backup images.".to_owned())?;
        let count: i64 = connection
            .query_row("SELECT count(*) FROM media", [], |row| row.get(0))
            .map_err(|_| "Cannot inspect backup images.".to_owned())?;
        if count as usize != portable.media.len() {
            return Err("Backup is missing managed images.".into());
        }
        for image in &portable.media {
            let registered: Option<(String, usize, String)> = connection
                .query_row(
                    "SELECT mime, bytes, sha256 FROM media WHERE id = ?1",
                    [&image.id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
                .map_err(|_| "Cannot inspect backup images.".to_owned())?;
            let hash = format!("{:x}", Sha256::digest(&image.bytes));
            if registered != Some((image.mime.clone(), image.bytes.len(), hash)) {
                return Err("Backup image does not match its database record.".into());
            }
            let destination = crate::media::media_path(&stage, &image.id, &image.mime)?;
            fs::write(&destination, &image.bytes)
                .map_err(|_| "Cannot stage backup image.".to_owned())?;
            sync_file(&destination).map_err(|_| "Cannot secure backup image.".to_owned())?;
        }
        sync_file(&snapshot).map_err(|_| "Cannot secure backup snapshot.".to_owned())?;
        Ok(())
    })();

    if let Err(error) = result {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    Ok(stage)
}

fn prepare_restore(root: &Path, id: &str) -> Result<(), String> {
    // Preflight the archive now; setup stages it again because the source may change before restart.
    let stage = stage_restore(root, &load_backup(root, id)?)?;
    fs::remove_dir_all(stage).map_err(|_| "Cannot clear restore staging.".to_owned())?;

    let request = root.join("recovery").join("restore.requested");
    fs::write(&request, id).map_err(|_| "Cannot request backup restore.".to_owned())?;
    sync_file(&request).map_err(|_| "Cannot secure backup restore request.".to_owned())
}

pub(crate) fn apply_pending_restore(root: &Path) -> Result<(), String> {
    recover_interrupted_restore(root)?;
    let request = root.join("recovery").join("restore.requested");
    if !request.exists() {
        return Ok(());
    }
    let id = fs::read_to_string(&request).map_err(|_| "Cannot read restore request.".to_owned())?;
    let stage = stage_restore(root, &load_backup(root, id.trim())?)?;

    let live = root.join("workspace.sqlite");
    let recovery = root.join("recovery").join("restore-previous.sqlite");
    let previous_media = root.join("recovery").join("restore-previous-media");
    let marker = root.join("recovery").join("restore.pending");

    // This runs during Tauri setup, before the renderer can issue database commands.
    let connection = Connection::open_with_flags(&live, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|_| "Cannot open database for restore.".to_owned())?;
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| "Cannot prepare database for restore.".to_owned())?;
    connection
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|_| "Cannot prepare database for restore.".to_owned())?;
    connection
        .backup(rusqlite::MAIN_DB, &recovery, None)
        .map_err(|_| "Cannot save recovery copy before restore.".to_owned())?;
    drop(connection);
    sync_file(&recovery).map_err(|_| "Cannot secure recovery copy before restore.".to_owned())?;
    if previous_media.exists() {
        fs::remove_dir_all(&previous_media)
            .map_err(|_| "Cannot prepare media recovery folder.".to_owned())?;
    }

    // Checkpoint completed above. Remove stale sidecars before swapping database files.
    for suffix in ["-wal", "-shm"] {
        let sidecar = root.join(format!("workspace.sqlite{suffix}"));
        if sidecar.exists() {
            fs::remove_file(sidecar)
                .map_err(|_| "Cannot clear database sidecar before restore.".to_owned())?;
        }
    }

    // Recovery inspects this marker after a crash and rolls forward or restores both DB and media.
    fs::write(&marker, &id).map_err(|_| "Cannot mark restore in progress.".to_owned())?;
    sync_file(&marker).map_err(|_| "Cannot secure restore marker.".to_owned())?;
    let old = root.join("recovery").join("restore-old.sqlite");
    if old.exists() {
        fs::remove_file(&old).map_err(|_| "Cannot prepare recovery folder.".to_owned())?;
    }
    let swap = (|| {
        fs::rename(&live, &old)
            .map_err(|_| "Cannot move existing database to recovery.".to_owned())?;
        fs::rename(root.join("media"), &previous_media)
            .map_err(|_| "Cannot move existing images to recovery.".to_owned())?;
        fs::rename(stage.join("workspace.sqlite"), &live)
            .map_err(|_| "Cannot install restored database.".to_owned())?;
        fs::rename(stage.join("media"), root.join("media"))
            .map_err(|_| "Cannot install restored images.".to_owned())
    })();
    if let Err(error) = swap {
        recover_interrupted_restore(root)?;
        return Err(error);
    }
    let finish = (|| {
        fs::remove_file(&marker)
            .map_err(|_| "Restore finished but recovery marker remains.".to_owned())?;
        fs::remove_file(&request)
            .map_err(|_| "Restore finished but restore request remains.".to_owned())
    })();
    if let Err(error) = finish {
        if marker.exists() {
            recover_interrupted_restore(root)?;
        }
        return Err(error);
    }
    let _ = fs::remove_dir_all(stage);
    let failed = root.join("recovery").join("restore.failed");
    if failed.exists() {
        fs::remove_file(failed)
            .map_err(|_| "Restore finished but previous error remains.".to_owned())?;
    }
    let _ = fs::write(root.join("recovery").join("restore.completed"), id);
    Ok(())
}

fn recover_interrupted_restore(root: &Path) -> Result<(), String> {
    let marker = root.join("recovery").join("restore.pending");
    if !marker.exists() {
        return Ok(());
    }

    let previous = root.join("recovery").join("restore-previous.sqlite");
    validate_snapshot(&previous).map_err(|_| "Cannot recover interrupted restore.".to_owned())?;
    for suffix in ["-wal", "-shm"] {
        let sidecar = root.join(format!("workspace.sqlite{suffix}"));
        if sidecar.exists() {
            fs::remove_file(sidecar)
                .map_err(|_| "Cannot clear database sidecar during recovery.".to_owned())?;
        }
    }
    fs::copy(&previous, root.join("workspace.sqlite"))
        .map_err(|_| "Cannot recover interrupted restore.".to_owned())?;
    let previous_media = root.join("recovery").join("restore-previous-media");
    if previous_media.exists() {
        let live_media = root.join("media");
        if live_media.exists() {
            fs::remove_dir_all(&live_media)
                .map_err(|_| "Cannot clear interrupted restore images.".to_owned())?;
        }
        fs::rename(previous_media, live_media)
            .map_err(|_| "Cannot recover interrupted restore images.".to_owned())?;
    }
    fs::remove_file(marker).map_err(|_| "Cannot finish interrupted restore recovery.".to_owned())
}

fn import_backup(root: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > 100 * 1024 * 1024 {
        return Err("Choose a SQLite backup smaller than 100 MB.".into());
    }
    let _ = open_database(root)?;
    let id = Uuid::new_v4().to_string();
    let temporary = root.join("staging").join(format!("import-{id}.sqlite"));
    fs::write(&temporary, bytes).map_err(|_| "Cannot stage imported backup.".to_owned())?;
    if let Err(error) = validate_snapshot(&temporary) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    let result = (|| {
        let media = snapshot_media(root, &temporary)?;
        let archive = crate::portable_archive::encode(bytes, &media)?;
        import_portable(root, &archive)
    })();
    let _ = fs::remove_file(temporary);
    result
}

fn inspect_import(root: &Path, bytes: &[u8], kind: &str) -> Result<ImportSummary, String> {
    let data = match kind {
        "portable" => crate::portable_json::decode(root, bytes)
            .or_else(|_| crate::portable_archive::decode(bytes))?,
        "backup" => {
            if bytes.is_empty() || bytes.len() > 100 * 1024 * 1024 {
                return Err("Choose a SQLite backup smaller than 100 MB.".into());
            }
            let path = root
                .join("staging")
                .join(format!("inspect-{}.sqlite", Uuid::new_v4()));
            fs::write(&path, bytes).map_err(|_| "Cannot stage imported backup.".to_owned())?;
            let result: Result<crate::portable_archive::PortableData, String> = (|| {
                validate_snapshot(&path)?;
                Ok(crate::portable_archive::PortableData {
                    snapshot: bytes.to_vec(),
                    media: snapshot_media(root, &path)?,
                })
            })();
            let _ = fs::remove_file(path);
            result?
        }
        _ => return Err("Choose a supported import format.".into()),
    };

    let stage = stage_restore(root, &data)?;
    let result = (|| {
        let connection = Connection::open_with_flags(
            stage.join("workspace.sqlite"),
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|_| "Cannot inspect imported workspace.".to_owned())?;
        let count = |table: &str| {
            connection
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .map_err(|_| "Cannot inspect imported workspace.".to_owned())
        };
        let workspace_name = connection
            .query_row(
                "SELECT name FROM workspace WHERE id = (SELECT workspace_id FROM local_actor WHERE singleton = 1)",
                [],
                |row| row.get(0),
            )
            .map_err(|_| "Cannot read imported workspace name.".to_owned())?;
        Ok(ImportSummary {
            checksum: format!("{:x}", Sha256::digest(bytes)),
            workspace_name,
            projects: count("project")?,
            issues: count("issue")?,
            cycles: count("cycle")?,
            images: data.media.len(),
        })
    })();
    let _ = fs::remove_dir_all(stage);
    result
}

fn snapshot_media(
    root: &Path,
    snapshot_path: &Path,
) -> Result<Vec<crate::portable_archive::MediaFile>, String> {
    let connection = Connection::open_with_flags(snapshot_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| "Cannot inspect portable images.".to_owned())?;
    let mut statement = connection
        .prepare("SELECT id, mime, bytes, sha256 FROM media ORDER BY id")
        .map_err(|_| "Cannot list portable images.".to_owned())?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, usize>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|_| "Cannot list portable images.".to_owned())?;
    let mut media = Vec::new();
    for row in rows {
        let (id, mime, expected_bytes, expected_hash) =
            row.map_err(|_| "Cannot list portable images.".to_owned())?;
        if expected_bytes > crate::media::MAX_IMAGE_BYTES {
            return Err("A managed image exceeds the size limit.".into());
        }
        let bytes = fs::read(crate::media::media_path(root, &id, &mime)?)
            .map_err(|_| "A managed image is missing. Restore it before exporting.".to_owned())?;
        if bytes.len() != expected_bytes || format!("{:x}", Sha256::digest(&bytes)) != expected_hash
        {
            return Err("A managed image failed its integrity check.".into());
        }
        media.push(crate::portable_archive::MediaFile { id, mime, bytes });
    }
    Ok(media)
}

fn import_portable(root: &Path, bytes: &[u8]) -> Result<(), String> {
    let _ = open_database(root)?;
    let portable = match crate::portable_json::decode(root, bytes) {
        Ok(data) => data,
        Err(_) => {
            let mut legacy = crate::portable_archive::decode(bytes)?;
            clear_import_receipts(root, &mut legacy.snapshot)?;
            legacy
        }
    };
    let canonical = crate::portable_archive::encode(&portable.snapshot, &portable.media)?;
    let id = Uuid::new_v4().to_string();
    let temporary = root.join("staging").join(format!("import-{id}.zip"));
    let final_path = root.join("backups").join(format!("{id}.zip"));
    fs::write(&temporary, canonical).map_err(|_| "Cannot stage imported backup.".to_owned())?;
    let result = (|| {
        sync_file(&temporary).map_err(|_| "Cannot secure imported backup.".to_owned())?;
        fs::rename(&temporary, &final_path)
            .map_err(|_| "Cannot keep imported backup.".to_owned())?;
        prepare_restore(root, &id)
    })();
    let _ = fs::remove_file(temporary);
    if result.is_err() {
        let _ = fs::remove_file(final_path);
    }
    result
}

fn clear_import_receipts(root: &Path, snapshot: &mut Vec<u8>) -> Result<(), String> {
    let path = root
        .join("staging")
        .join(format!("import-receipts-{}.sqlite", Uuid::new_v4()));
    let result = (|| {
        fs::write(&path, &*snapshot).map_err(|_| "Cannot stage portable data.".to_owned())?;
        let connection =
            Connection::open(&path).map_err(|_| "Cannot inspect imported data.".to_owned())?;
        let has_receipts: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'mutation_receipt')",
                [],
                |row| row.get(0),
            )
            .map_err(|_| "Cannot inspect imported data.".to_owned())?;
        if has_receipts {
            connection
                .execute("DELETE FROM mutation_receipt", [])
                .map_err(|_| "Cannot reset imported mutation receipts.".to_owned())?;
        }
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
            .map_err(|_| "Cannot finish imported data.".to_owned())?;
        *snapshot = fs::read(&path).map_err(|_| "Cannot read imported data.".to_owned())?;
        Ok(())
    })();
    let _ = fs::remove_file(&path);
    for suffix in ["-wal", "-shm"] {
        let _ = fs::remove_file(format!("{}{suffix}", path.display()));
    }
    result
}

#[api_contract_macros::command(mutation)]
pub fn storage_create_backup(app: AppHandle) -> Result<BackupEntry, String> {
    create_backup(&document_data_root(&app)?)
}

#[api_contract_macros::command]
pub fn storage_list_backups(app: AppHandle) -> Result<Vec<BackupEntry>, String> {
    let root = document_data_root(&app)?;
    let _ = open_database(&root)?;
    list_backups(&root)
}

#[api_contract_macros::command(mutation)]
pub fn storage_restore_backup(app: AppHandle, id: String) -> Result<(), String> {
    prepare_restore(&document_data_root(&app)?, &id)?;
    Ok(())
}

#[api_contract_macros::command(mutation)]
pub fn storage_export_portable_file(app: AppHandle) -> Result<Option<String>, String> {
    let Some(path) = choose_export_path(&app, "workspace.zip")? else {
        return Ok(None);
    };
    let root = document_data_root(&app)?;
    let snapshot = create_backup(&root)?;
    let data = load_backup(&root, &snapshot.id)?;
    write_export(&path, |temporary| {
        crate::portable_json::encode_to_file(&root, &data.snapshot, &data.media, temporary)
    })
    .map(Some)
}

#[api_contract_macros::command(mutation)]
pub fn storage_export_backup_file(app: AppHandle, id: String) -> Result<Option<String>, String> {
    let Some(path) = choose_export_path(&app, &format!("backup-{id}.zip"))? else {
        return Ok(None);
    };
    let root = document_data_root(&app)?;
    let source = backup_path(&root, &id)?;
    if source.extension().and_then(|value| value.to_str()) == Some("zip") {
        return write_export(&path, |temporary| {
            fs::copy(source, temporary)
                .map(|_| ())
                .map_err(|_| "Cannot copy backup export.".to_owned())
        })
        .map(Some);
    }
    let backup = load_backup(&root, &id)?;
    let stage = stage_restore(&root, &backup)?;
    let _ = fs::remove_dir_all(stage);
    write_export(&path, |temporary| {
        crate::portable_archive::encode_to_file(&backup.snapshot, &backup.media, temporary)
    })
    .map(Some)
}

fn choose_export_path(app: &AppHandle, filename: &str) -> Result<Option<PathBuf>, String> {
    let selected = app
        .dialog()
        .file()
        .set_file_name(filename)
        .add_filter("ZIP archive", &["zip"])
        .blocking_save_file();
    selected
        .map(|file| {
            file.into_path()
                .map_err(|_| "Choose a local export path.".to_owned())
        })
        .transpose()
}

fn write_export(
    path: &Path,
    write: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<String, String> {
    let parent = path
        .parent()
        .filter(|parent| parent.is_dir())
        .ok_or_else(|| "Choose an existing export folder.".to_owned())?;
    let temporary = parent.join(format!(".pm-app-export-{}.part", Uuid::new_v4()));
    let result = (|| {
        write(&temporary)?;
        sync_file(&temporary).map_err(|_| "Cannot secure export.".to_owned())?;
        let file = fs::File::open(&temporary).map_err(|_| "Cannot inspect export.".to_owned())?;
        if file
            .metadata()
            .map_err(|_| "Cannot inspect export.".to_owned())?
            .len()
            == 0
        {
            return Err("Export is empty.".into());
        }
        let mut archive =
            zip::ZipArchive::new(file).map_err(|_| "Cannot verify export archive.".to_owned())?;
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|_| "Cannot verify export entry.".to_owned())?;
            if entry.is_file() {
                std::io::copy(&mut entry, &mut std::io::sink())
                    .map_err(|_| "Export entry checksum failed.".to_owned())?;
            }
        }
        drop(archive);
        fs::rename(&temporary, path)
            .map_err(|_| "Cannot save export at chosen location.".to_owned())?;
        #[cfg(not(windows))]
        fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| "Cannot secure export folder.".to_owned())?;
        Ok(path.to_string_lossy().into_owned())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn import_upload_path(root: &Path, id: &str) -> Result<PathBuf, String> {
    Uuid::parse_str(id).map_err(|_| "Invalid import identifier.".to_owned())?;
    Ok(root
        .join("staging")
        .join(format!("import-upload-{id}.part")))
}

#[api_contract_macros::command(mutation)]
pub fn storage_begin_import_upload(app: AppHandle) -> Result<String, String> {
    let root = document_data_root(&app)?;
    let _ = open_database(&root)?;
    let id = Uuid::new_v4().to_string();
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(import_upload_path(&root, &id)?)
        .map_err(|_| "Cannot begin import upload.".to_owned())?;
    Ok(id)
}

#[api_contract_macros::command(mutation)]
pub fn storage_append_import_chunk(
    app: AppHandle,
    id: String,
    offset: u64,
    chunk: String,
) -> Result<(), String> {
    let bytes = STANDARD
        .decode(chunk)
        .map_err(|_| "Import chunk is invalid.".to_owned())?;
    if bytes.is_empty() || bytes.len() > 256 * 1024 {
        return Err("Import chunk exceeds 256 KB.".into());
    }
    let path = import_upload_path(&document_data_root(&app)?, &id)?;
    append_import_bytes(&path, offset, &bytes)
}

fn append_import_bytes(path: &Path, offset: u64, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|_| "Import upload is unavailable.".to_owned())?;
    let size = file
        .metadata()
        .map_err(|_| "Cannot inspect import upload.".to_owned())?
        .len();
    if size >= offset.saturating_add(bytes.len() as u64) {
        file.seek(SeekFrom::Start(offset))
            .map_err(|_| "Cannot inspect import chunk.".to_owned())?;
        let mut saved = vec![0; bytes.len()];
        file.read_exact(&mut saved)
            .map_err(|_| "Cannot inspect import chunk.".to_owned())?;
        if saved == bytes {
            return Ok(());
        }
    }
    if size != offset || size.saturating_add(bytes.len() as u64) > 550 * 1024 * 1024 {
        return Err("Import upload is out of order or too large.".into());
    }
    file.seek(SeekFrom::End(0))
        .map_err(|_| "Cannot append import chunk.".to_owned())?;
    file.write_all(bytes)
        .map_err(|_| "Cannot write import chunk.".to_owned())
}

#[api_contract_macros::command(mutation)]
pub fn storage_discard_import_upload(app: AppHandle, id: String) -> Result<(), String> {
    let path = import_upload_path(&document_data_root(&app)?, &id)?;
    if path.exists() {
        fs::remove_file(path).map_err(|_| "Cannot discard import upload.".to_owned())?;
    }
    Ok(())
}

#[api_contract_macros::command(mutation)]
pub fn storage_finish_import_upload(
    app: AppHandle,
    id: String,
    kind: String,
    checksum: String,
) -> Result<(), String> {
    let root = document_data_root(&app)?;
    let path = import_upload_path(&root, &id)?;
    let bytes = fs::read(&path).map_err(|_| "Cannot read import upload.".to_owned())?;
    if format!("{:x}", Sha256::digest(&bytes)) != checksum {
        return Err("Imported file changed after inspection. Select it again.".into());
    }
    let result = match kind.as_str() {
        "portable" => import_portable(&root, &bytes),
        "backup" => import_backup(&root, &bytes),
        _ => Err("Choose a supported import format.".into()),
    };
    let _ = fs::remove_file(path);
    result?;
    Ok(())
}

#[api_contract_macros::command]
pub fn storage_inspect_import_upload(
    app: AppHandle,
    id: String,
    kind: String,
) -> Result<ImportSummary, String> {
    let root = document_data_root(&app)?;
    let bytes = fs::read(import_upload_path(&root, &id)?)
        .map_err(|_| "Cannot read import upload.".to_owned())?;
    inspect_import(&root, &bytes, &kind)
}

#[api_contract_macros::command(mutation)]
pub fn storage_run_restore_catchup(app: AppHandle) -> Result<RestoreCatchup, String> {
    let root = document_data_root(&app)?;
    run_restore_catchup(&root)
}

fn run_restore_catchup(root: &Path) -> Result<RestoreCatchup, String> {
    if !restore_catchup_pending(&root) {
        return Ok(RestoreCatchup {
            processed: 0,
            total: 0,
            done: true,
        });
    }
    let mut connection = open_database(root)?;
    let selected_workspace = crate::storage::current_workspace_id(&connection)?;
    let workspace_ids = {
        let mut statement = connection
            .prepare("SELECT id FROM workspace ORDER BY id")
            .map_err(|_| "Cannot list restored workspaces.".to_owned())?;
        let ids = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|_| "Cannot list restored workspaces.".to_owned())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "Cannot list restored workspaces.".to_owned())?;
        ids
    };

    // Reconciliation uses the selected workspace. Restore that selection even if a
    // later workspace fails so retry can resume without changing the user's view.
    let result = workspace_ids.iter().try_for_each(|workspace_id| {
        crate::storage::switch_workspace(&connection, workspace_id)?;
        crate::cycles::reconcile_current_workspace(&mut connection)
    });
    crate::storage::switch_workspace(&connection, &selected_workspace)?;
    result?;
    let (processed, total): (i64, i64) = connection
        .query_row(
            "SELECT coalesce(sum(processed), 0), coalesce(sum(total), 0) FROM cycle_reconcile_progress",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Cannot read cycle catch-up progress.".to_owned())?;
    let done = total == 0;
    if done {
        fs::remove_file(root.join("recovery").join("restore.completed"))
            .map_err(|_| "Cannot finish restore review.".to_owned())?;
    }
    Ok(RestoreCatchup {
        processed,
        total,
        done,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cycles::CycleSettings;
    use crate::storage::{create_workspace, current_workspace_id, save_identity};
    use image::{DynamicImage, ImageFormat};
    use std::io::Cursor;

    #[test]
    fn failed_export_keeps_destination_and_removes_temporary_output() {
        let root = std::env::temp_dir().join(format!("pm-app-export-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let destination = root.join("previous.zip");
        fs::write(&destination, b"previous export").unwrap();
        assert!(write_export(&destination, |temporary| {
            fs::write(temporary, b"incomplete").unwrap();
            Err("Write interrupted.".into())
        })
        .is_err());
        assert_eq!(fs::read(&destination).unwrap(), b"previous export");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn import_chunk_retry_keeps_original_bytes() {
        let path = std::env::temp_dir().join(format!("pm-app-import-chunk-{}", Uuid::new_v4()));
        fs::write(&path, []).unwrap();
        append_import_bytes(&path, 0, b"first").unwrap();
        append_import_bytes(&path, 0, b"first").unwrap();
        append_import_bytes(&path, 5, b"second").unwrap();
        assert!(append_import_bytes(&path, 0, b"other").is_err());
        assert_eq!(fs::read(&path).unwrap(), b"firstsecond");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn restore_catchup_reconciles_every_workspace_and_preserves_selection() {
        let root = std::env::temp_dir().join(format!("pm-app-catchup-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "First").unwrap();
        let selected = current_workspace_id(&connection).unwrap();
        create_workspace(&mut connection, "Second").unwrap();
        let second = current_workspace_id(&connection).unwrap();
        let settings: CycleSettings = serde_json::from_value(serde_json::json!({
            "id": "",
            "name": "Second series",
            "durationDays": 7,
            "intervalDays": 7,
            "enabled": true,
            "startDate": chrono::Utc::now().date_naive().to_string(),
            "rolloverPolicy": "automatic",
            "projectIds": [],
            "timeZone": "UTC",
            "startLocalTime": "00:00"
        }))
        .unwrap();
        crate::cycles::configure_cycles(&mut connection, settings).unwrap();
        connection
            .execute("DELETE FROM cycle WHERE workspace_id = ?1", [&second])
            .unwrap();
        crate::storage::switch_workspace(&connection, &selected).unwrap();
        fs::write(root.join("recovery").join("restore.completed"), b"").unwrap();
        drop(connection);

        assert!(run_restore_catchup(&root).unwrap().done);
        let connection = open_database(&root).unwrap();
        assert_eq!(current_workspace_id(&connection).unwrap(), selected);
        let count: i64 = connection
            .query_row(
                "SELECT count(*) FROM cycle WHERE workspace_id = ?1",
                [&second],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count >= 12);
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn portable_export_restores_managed_images_with_their_database_records() {
        let root = std::env::temp_dir().join(format!("pm-app-portable-media-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Media workspace").unwrap();
        let mut bytes = Vec::new();
        DynamicImage::new_rgba8(2, 2)
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        let staged = crate::media::stage_image(&root, &connection, &bytes).unwrap();
        let workspace_id = current_workspace_id(&connection).unwrap();
        let document = serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "content": [
                {"type": "image", "attrs": {"mediaId": staged.id, "alt": "Diagram"}}
            ]}
        });
        connection.execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position, description_document_json)
             VALUES (?1, ?2, 'LOCAL-1', 'Image issue', 'Todo', 'No priority', 0, ?3)",
            rusqlite::params![Uuid::new_v4().to_string(), workspace_id, document.to_string()],
        ).unwrap();
        connection.execute(
            "INSERT INTO mutation_receipt(mutation_id, workspace_id, operation, request_hash, result_json, workspace_revision)
             VALUES (?1, ?2, 'issue.create', 'hash', '{}', 1)",
            rusqlite::params![Uuid::new_v4().to_string(), workspace_id],
        ).unwrap();
        drop(connection);

        let backup = create_backup(&root).unwrap();
        let path = backup_path(&root, &backup.id).unwrap();
        let archive = fs::read(&path).unwrap();
        let image_path = crate::media::media_path(&root, &staged.id, "image/png").unwrap();
        let saved_image = fs::read(&image_path).unwrap();
        assert!(saved_image.len() <= bytes.len());
        fs::remove_file(&image_path).unwrap();

        prepare_restore(&root, &backup.id).unwrap();
        assert!(!image_path.exists());
        apply_pending_restore(&root).unwrap();
        assert!(restore_catchup_pending(&root));
        assert!(run_restore_catchup(&root).unwrap().done);
        assert!(!restore_catchup_pending(&root));
        assert_eq!(fs::read(&image_path).unwrap(), saved_image);
        let receipt_count = |root: &Path| -> i64 {
            open_database(root)
                .unwrap()
                .query_row("SELECT count(*) FROM mutation_receipt", [], |row| {
                    row.get(0)
                })
                .unwrap()
        };
        assert_eq!(receipt_count(&root), 1);

        fs::remove_file(&image_path).unwrap();
        import_portable(&root, &archive).unwrap();
        assert!(!image_path.exists());
        apply_pending_restore(&root).unwrap();
        assert_eq!(fs::read(&image_path).unwrap(), saved_image);
        assert_eq!(receipt_count(&root), 0);
        let restored: String = open_database(&root)
            .unwrap()
            .query_row(
                "SELECT description_document_json FROM issue WHERE identifier = 'LOCAL-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(restored.contains(&staged.id));
        let data = load_backup(&root, &backup.id).unwrap();
        let json_archive =
            crate::portable_json::encode(&root, &data.snapshot, &data.media).unwrap();
        fs::remove_file(&image_path).unwrap();
        let summary = inspect_import(&root, &json_archive, "portable").unwrap();
        assert_eq!(summary.workspace_name, "Media workspace");
        assert_eq!((summary.issues, summary.images), (1, 1));
        assert!(inspect_import(&root, b"invalid", "portable").is_err());
        import_portable(&root, &json_archive).unwrap();
        apply_pending_restore(&root).unwrap();
        assert_eq!(fs::read(&image_path).unwrap(), saved_image);
        assert_eq!(receipt_count(&root), 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn backup_roundtrip_and_invalid_backup_keep_live_data() {
        let root = std::env::temp_dir().join(format!("pm-app-backup-test-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "First").unwrap();
        let workspace_id = current_workspace_id(&connection).unwrap();
        let image_document = serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "content": [{"type": "image", "attrs": {"src": "data:image/png;base64,iVBORw0KGgo=", "alt": "Diagram"}}]}
        }).to_string();
        connection.execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position, description_document_json)
             VALUES (?1, ?2, 'LOCAL-1', 'Illustrated issue', 'Todo', 'No priority', 0, ?3)",
            rusqlite::params![Uuid::new_v4().to_string(), workspace_id, image_document],
        ).unwrap();
        drop(connection);
        let backup = create_backup(&root).unwrap();
        assert_eq!(
            backup_path(&root, &backup.id).unwrap().extension().unwrap(),
            "zip"
        );
        assert!(list_backups(&root)
            .unwrap()
            .iter()
            .any(|entry| entry.id == backup.id));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Changed").unwrap();
        drop(connection);
        let restore_request = MutationRequest {
            mutation_id: Uuid::new_v4().to_string(),
            expected_version: None,
            expected_versions: Default::default(),
        };
        stage_restore_receipt(
            &root,
            &restore_request,
            "storage_restore_backup",
            "same-request",
        )
        .unwrap();
        prepare_restore(&root, &backup.id).unwrap();
        mark_restore_receipt_ready(&root).unwrap();
        let before_restart: String = open_database(&root)
            .unwrap()
            .query_row("SELECT name FROM workspace LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(before_restart, "Changed");
        apply_pending_restore(&root).unwrap();
        install_restore_receipt(&root, true).unwrap();
        let connection = open_database(&root).unwrap();
        assert!(mutations::replay::<()>(
            &connection,
            &current_workspace_id(&connection).unwrap(),
            "storage_restore_backup",
            "same-request",
            &restore_request,
        )
        .unwrap()
        .is_some());
        let name: String = connection
            .query_row("SELECT name FROM workspace LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "First");
        let restored_image: String = connection
            .query_row(
                "SELECT description_document_json FROM issue WHERE identifier = 'LOCAL-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(restored_image.contains("data:image/png;base64,"));
        drop(connection);
        assert!(prepare_restore(&root, "../bad").is_err());
        assert!(import_backup(&root, b"not a database").is_err());
        let fake = root.join("staging").join("fake.sqlite");
        let fake_connection = Connection::open(&fake).unwrap();
        fake_connection
            .pragma_update(None, "user_version", 29)
            .unwrap();
        drop(fake_connection);
        assert!(import_backup(&root, &fs::read(&fake).unwrap()).is_err());
        let unchanged: String = open_database(&root)
            .unwrap()
            .query_row("SELECT name FROM workspace LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(unchanged, "First");
        let bytes = fs::read(backup_path(&root, &backup.id).unwrap()).unwrap();
        let decoded = crate::portable_archive::decode(&bytes).unwrap();
        import_backup(&root, &decoded.snapshot).unwrap();
        apply_pending_restore(&root).unwrap();
        let imported_image: String = open_database(&root)
            .unwrap()
            .query_row(
                "SELECT description_document_json FROM issue WHERE identifier = 'LOCAL-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(imported_image, image_document);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unavailable_requested_backup_leaves_live_database_intact() {
        let root = std::env::temp_dir().join(format!("pm-app-backup-test-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Live workspace").unwrap();
        drop(connection);

        let backup = create_backup(&root).unwrap();
        prepare_restore(&root, &backup.id).unwrap();
        fs::remove_file(backup_path(&root, &backup.id).unwrap()).unwrap();
        assert!(apply_pending_restore(&root).is_err());

        let live_name: String = open_database(&root)
            .unwrap()
            .query_row("SELECT name FROM workspace LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(live_name, "Live workspace");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn interrupted_restore_recovers_previous_database_before_checking_request() {
        let root = std::env::temp_dir().join(format!("pm-app-backup-test-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Live workspace").unwrap();
        drop(connection);

        let backup = create_backup(&root).unwrap();
        prepare_restore(&root, &backup.id).unwrap();
        let live = root.join("workspace.sqlite");
        fs::copy(&live, root.join("recovery").join("restore-previous.sqlite")).unwrap();
        fs::write(root.join("recovery").join("restore.pending"), &backup.id).unwrap();
        fs::remove_file(&live).unwrap();
        fs::remove_file(backup_path(&root, &backup.id).unwrap()).unwrap();

        assert!(apply_pending_restore(&root).is_err());
        assert!(!root.join("recovery").join("restore.pending").exists());
        let name: String = open_database(&root)
            .unwrap()
            .query_row("SELECT name FROM workspace LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "Live workspace");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn interrupted_restore_recovers_database_and_images_together() {
        let root = std::env::temp_dir().join(format!("pm-app-backup-test-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Before restore").unwrap();
        let mut bytes = Vec::new();
        DynamicImage::new_rgba8(2, 2)
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        let image = crate::media::stage_image(&root, &connection, &bytes).unwrap();
        let saved_image =
            fs::read(crate::media::media_path(&root, &image.id, "image/png").unwrap()).unwrap();
        let backup = create_backup(&root).unwrap();
        prepare_restore(&root, &backup.id).unwrap();
        connection
            .backup(
                rusqlite::MAIN_DB,
                root.join("recovery").join("restore-previous.sqlite"),
                None,
            )
            .unwrap();
        drop(connection);

        fs::rename(
            root.join("media"),
            root.join("recovery").join("restore-previous-media"),
        )
        .unwrap();
        fs::create_dir(root.join("media")).unwrap();
        fs::write(root.join("media").join("new-image"), b"incomplete restore").unwrap();
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Interrupted restore").unwrap();
        drop(connection);
        fs::write(root.join("recovery").join("restore.pending"), &backup.id).unwrap();
        fs::remove_file(backup_path(&root, &backup.id).unwrap()).unwrap();

        assert!(apply_pending_restore(&root).is_err());
        let connection = open_database(&root).unwrap();
        let name: String = connection
            .query_row("SELECT name FROM workspace LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "Before restore");
        assert_eq!(
            fs::read(crate::media::media_path(&root, &image.id, "image/png").unwrap()).unwrap(),
            saved_image
        );
        assert!(!root.join("media").join("new-image").exists());
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }
}
