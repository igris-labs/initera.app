use std::{
    collections::HashSet,
    fs,
    io::{Cursor, Read, Seek, Write},
    path::Path,
};

use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{
    params_from_iter,
    types::{Value, ValueRef},
    Connection, OpenFlags,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::portable_archive::{MediaFile, PortableData};
use crate::storage::{open_database, searchable_issue_description};

const TABLES: &[&str] = &[
    "workspace",
    "local_actor",
    "workspace_preferences",
    "project",
    "project_resource",
    "milestone",
    "project_update",
    "issue",
    "issue_sequence",
    "issue_activity",
    "issue_relation",
    "workflow_status",
    "saved_view",
    "cycle",
    "cycle_issue",
    "cycle_settings",
    "time_entry",
    "timer_recovery_notice",
    "planning_node",
    "planning_node_issue",
    "issue_alias",
    "cycle_transition",
    "cycle_snapshot_issue",
    "media",
    "cycle_series",
    "label",
    "issue_label",
    "project_label",
    "label_detached",
    "mutation_receipt",
    "milestone_detached_issue",
];
const MAX_ENTITIES_BYTES: usize = 200 * 1024 * 1024;
const MAX_ARCHIVE_BYTES: usize = 550 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    version: u8,
    schema_version: u32,
    entities_bytes: usize,
    entities_sha256: String,
    media: Vec<MediaEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MediaEntry {
    id: String,
    mime: String,
    bytes: usize,
    sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EntityCollection {
    table: String,
    columns: Vec<String>,
    rows: Vec<Vec<Cell>>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
enum Cell {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(String),
}

fn checksum(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn media_name(id: &str, mime: &str) -> Result<String, String> {
    Uuid::parse_str(id).map_err(|_| "Portable image ID is invalid.".to_owned())?;
    let extension = match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => return Err("Portable image type is invalid.".into()),
    };
    Ok(format!("media/{id}.{extension}"))
}

fn columns(connection: &Connection, table: &str) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info(\"{table}\")"))
        .map_err(|_| "Cannot inspect entity columns.".to_owned())?;
    let names = statement
        .query_map([], |row| row.get(1))
        .map_err(|_| "Cannot inspect entity columns.".to_owned())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Cannot inspect entity columns.".to_owned())?;
    Ok(names)
}

fn export_entities(connection: &Connection) -> Result<Vec<u8>, String> {
    // Portable archives contain explicit table collections. Keep TABLES aligned with the V1
    // schema; column names travel with rows so import can validate their shape.
    let mut collections = Vec::with_capacity(TABLES.len());
    for &table in TABLES {
        let names = columns(connection, table)?;
        if names.is_empty() {
            return Err("Portable source schema is incomplete.".into());
        }
        let mut statement = connection
            .prepare(&format!("SELECT * FROM \"{table}\""))
            .map_err(|_| "Cannot read portable entities.".to_owned())?;
        let mut query = statement
            .query([])
            .map_err(|_| "Cannot read portable entities.".to_owned())?;
        let mut rows = Vec::new();
        while let Some(row) = query
            .next()
            .map_err(|_| "Cannot read portable entities.".to_owned())?
        {
            let mut cells = Vec::with_capacity(names.len());
            for index in 0..names.len() {
                cells.push(
                    match row
                        .get_ref(index)
                        .map_err(|_| "Cannot read portable entity.".to_owned())?
                    {
                        ValueRef::Null => Cell::Null,
                        ValueRef::Integer(value) => Cell::Integer(value),
                        ValueRef::Real(value) if value.is_finite() => Cell::Real(value),
                        ValueRef::Real(_) => {
                            return Err("Portable entity has a nonfinite number.".into())
                        }
                        ValueRef::Text(value) => Cell::Text(
                            String::from_utf8(value.to_vec())
                                .map_err(|_| "Portable entity text is invalid.".to_owned())?,
                        ),
                        ValueRef::Blob(value) => Cell::Blob(STANDARD.encode(value)),
                    },
                );
            }
            rows.push(cells);
        }
        collections.push(EntityCollection {
            table: table.into(),
            columns: names,
            rows,
        });
    }
    let bytes = serde_json::to_vec(&collections)
        .map_err(|_| "Cannot encode portable entities.".to_owned())?;
    if bytes.len() > MAX_ENTITIES_BYTES {
        return Err("Portable entities exceed 200 MB.".into());
    }
    Ok(bytes)
}

fn write_archive<W: Write + Seek>(
    root: &Path,
    snapshot: &[u8],
    media: &[MediaFile],
    output: W,
) -> Result<W, String> {
    let stage = root
        .join("staging")
        .join(format!("json-export-{}.sqlite", Uuid::new_v4()));
    fs::write(&stage, snapshot).map_err(|_| "Cannot stage portable export.".to_owned())?;
    let entities = (|| {
        let connection = Connection::open_with_flags(&stage, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| "Cannot inspect portable snapshot.".to_owned())?;
        export_entities(&connection)
    })();
    let _ = fs::remove_file(stage);
    let entities = entities?;
    let mut total_media = 0_usize;
    let mut names = HashSet::new();
    let mut media_entries = Vec::with_capacity(media.len());
    for file in media {
        let name = media_name(&file.id, &file.mime)?;
        if !names.insert(name) || file.bytes.len() > crate::media::MAX_IMAGE_BYTES {
            return Err("Portable export contains a repeated or oversized image.".into());
        }
        total_media = total_media.saturating_add(file.bytes.len());
        if total_media > 400 * 1024 * 1024 {
            return Err("Portable images exceed 400 MB.".into());
        }
        media_entries.push(MediaEntry {
            id: file.id.clone(),
            mime: file.mime.clone(),
            bytes: file.bytes.len(),
            sha256: checksum(&file.bytes),
        });
    }
    let manifest = Manifest {
        format: "pm-app-json".into(),
        version: 1,
        schema_version: 1,
        entities_bytes: entities.len(),
        entities_sha256: checksum(&entities),
        media: media_entries,
    };
    let mut writer = ZipWriter::new(output);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    writer
        .start_file("manifest.json", options)
        .map_err(|_| "Cannot write portable manifest.".to_owned())?;
    writer
        .write_all(&serde_json::to_vec(&manifest).map_err(|_| "Cannot encode portable manifest.")?)
        .map_err(|_| "Cannot write portable manifest.".to_owned())?;
    writer
        .start_file("entities.json", options)
        .map_err(|_| "Cannot write portable entities.".to_owned())?;
    writer
        .write_all(&entities)
        .map_err(|_| "Cannot write portable entities.".to_owned())?;
    for file in media {
        writer
            .start_file(media_name(&file.id, &file.mime)?, options)
            .map_err(|_| "Cannot write portable image.".to_owned())?;
        writer
            .write_all(&file.bytes)
            .map_err(|_| "Cannot write portable image.".to_owned())?;
    }
    writer
        .finish()
        .map_err(|_| "Cannot finish portable export.".to_owned())
}

#[cfg(test)]
pub(crate) fn encode(root: &Path, snapshot: &[u8], media: &[MediaFile]) -> Result<Vec<u8>, String> {
    let bytes = write_archive(root, snapshot, media, Cursor::new(Vec::new()))?.into_inner();
    if bytes.len() > MAX_ARCHIVE_BYTES {
        return Err("Portable export exceeds 550 MB.".into());
    }
    Ok(bytes)
}

pub(crate) fn encode_to_file(
    root: &Path,
    snapshot: &[u8],
    media: &[MediaFile],
    path: &Path,
) -> Result<(), String> {
    let result = (|| {
        let file =
            fs::File::create(path).map_err(|_| "Cannot create portable export file.".to_owned())?;
        let file = write_archive(root, snapshot, media, file)?;
        if file
            .metadata()
            .map_err(|_| "Cannot inspect portable export file.")?
            .len()
            > MAX_ARCHIVE_BYTES as u64
        {
            return Err("Portable export exceeds 550 MB.".into());
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}

fn read_entry(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
    limit: usize,
) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    archive
        .by_name(name)
        .map_err(|_| "Portable export is missing an entry.".to_owned())?
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read portable entry.".to_owned())?;
    if bytes.len() > limit {
        return Err("Portable entry exceeds its size limit.".into());
    }
    Ok(bytes)
}

fn restore_entities(root: &Path, collections: Vec<EntityCollection>) -> Result<Vec<u8>, String> {
    let stage = root
        .join("staging")
        .join(format!("json-import-{}", Uuid::new_v4()));
    let result = (|| {
        let mut connection = open_database(&stage)?;
        let expected_tables = TABLES;
        if collections.len() != expected_tables.len() {
            return Err("Portable entity collection is incomplete.".into());
        }
        let triggers = {
            let mut statement = connection.prepare("SELECT name, sql FROM sqlite_master WHERE type = 'trigger' AND sql IS NOT NULL")
                .map_err(|_| "Cannot inspect import schema.".to_owned())?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|_| "Cannot inspect import schema.".to_owned())?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|_| "Cannot inspect import schema.".to_owned())?
        };
        connection
            .execute_batch("PRAGMA foreign_keys = OFF;")
            .map_err(|_| "Cannot prepare portable import.".to_owned())?;
        let transaction = connection
            .transaction()
            .map_err(|_| "Cannot begin portable import.".to_owned())?;
        for (name, _) in &triggers {
            transaction
                .execute_batch(&format!("DROP TRIGGER \"{}\";", name.replace('"', "\"\"")))
                .map_err(|_| "Cannot prepare portable import triggers.".to_owned())?;
        }
        for (index, collection) in collections.into_iter().enumerate() {
            let table = expected_tables[index];
            let current_columns = columns(&transaction, table)?;
            if collection.table != table
                || collection
                    .columns
                    .iter()
                    .any(|name| !current_columns.contains(name))
            {
                return Err(format!(
                    "Portable collection {table} does not match this schema."
                ));
            }
            let quoted_columns = collection
                .columns
                .iter()
                .map(|name| format!("\"{}\"", name.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(", ");
            let placeholders = vec!["?"; collection.columns.len()].join(", ");
            let sql = format!("INSERT INTO \"{table}\" ({quoted_columns}) VALUES ({placeholders})");
            let mut statement = transaction
                .prepare(&sql)
                .map_err(|_| "Cannot prepare portable collection.".to_owned())?;
            for row in collection.rows {
                if row.len() != collection.columns.len() {
                    return Err("Portable row has wrong field count.".into());
                }
                let values = row
                    .into_iter()
                    .map(|cell| match cell {
                        Cell::Null => Ok(Value::Null),
                        Cell::Integer(value) => Ok(Value::Integer(value)),
                        Cell::Real(value) if value.is_finite() => Ok(Value::Real(value)),
                        Cell::Real(_) => Err("Portable row has nonfinite number.".to_owned()),
                        Cell::Text(value) => Ok(Value::Text(value)),
                        Cell::Blob(value) => STANDARD
                            .decode(value)
                            .map(Value::Blob)
                            .map_err(|_| "Portable blob is invalid.".to_owned()),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                statement
                    .execute(params_from_iter(values))
                    .map_err(|_| format!("Portable collection {table} has invalid data."))?;
            }
        }
        transaction
            .execute(
                "UPDATE time_entry SET stopped_at = COALESCE(last_seen_at, started_at),
                    stopped_at_ms = COALESCE(last_seen_at_ms, started_at_ms)
             WHERE stopped_at IS NULL",
                [],
            )
            .map_err(|_| "Cannot pause imported timers.".to_owned())?;
        transaction
            .execute("DELETE FROM timer_session", [])
            .map_err(|_| "Cannot reset imported timer session.".to_owned())?;
        // Receipts describe commands against the source installation. Replaying them
        // after a replace import could return stale results without applying a write.
        transaction
            .execute("DELETE FROM mutation_receipt", [])
            .map_err(|_| "Cannot reset imported mutation receipts.".to_owned())?;
        transaction
            .commit()
            .map_err(|_| "Cannot commit portable collections.".to_owned())?;
        for (_, sql) in triggers {
            connection
                .execute_batch(&sql)
                .map_err(|_| "Cannot rebuild import triggers.".to_owned())?;
        }
        let issue_description = searchable_issue_description("issue");
        let project_description = searchable_issue_description("project");
        connection.execute_batch(&format!(
            "DELETE FROM issue_search;
             INSERT INTO issue_search(rowid, issue_id, workspace_id, identifier, title, description, labels, project)
               SELECT issue.rowid, issue.id, issue.workspace_id, issue.identifier, issue.title,
                 {issue_description}, issue.labels_json, COALESCE(project.name, 'Unassigned')
               FROM issue LEFT JOIN project ON project.id = issue.project_id WHERE issue.archived_at IS NULL;
             DELETE FROM project_search;
             INSERT INTO project_search(rowid, project_id, workspace_id, name, summary, description, labels, resources)
               SELECT project.rowid, project.id, project.workspace_id, project.name, project.summary,
                 {project_description}, project.labels_json, project.resources_json FROM project;
             DELETE FROM comment_search;
             INSERT INTO comment_search(rowid, activity_id, issue_id, workspace_id, body)
               SELECT activity.rowid, activity.id, activity.issue_id, issue.workspace_id, activity.body
               FROM issue_activity activity JOIN issue ON issue.id = activity.issue_id
               WHERE activity.kind = 'comment';"
        )).map_err(|_| "Cannot rebuild imported search indexes.".to_owned())?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|_| "Cannot validate portable relations.".to_owned())?;
        let invalid: i64 = connection
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .map_err(|_| "Cannot validate portable relations.".to_owned())?;
        let integrity: String = connection
            .query_row("PRAGMA quick_check", [], |row| row.get(0))
            .map_err(|_| "Cannot validate portable database.".to_owned())?;
        if invalid != 0 || integrity != "ok" {
            return Err("Portable collections contain invalid relations or data.".into());
        }
        let snapshot = stage.join("snapshot.sqlite");
        connection
            .backup(rusqlite::MAIN_DB, &snapshot, None)
            .map_err(|_| "Cannot snapshot imported collections.".to_owned())?;
        fs::read(snapshot).map_err(|_| "Cannot read imported snapshot.".to_owned())
    })();
    let _ = fs::remove_dir_all(stage);
    result
}

pub(crate) fn decode(root: &Path, bytes: &[u8]) -> Result<PortableData, String> {
    if bytes.is_empty() || bytes.len() > MAX_ARCHIVE_BYTES {
        return Err("Choose a portable export smaller than 550 MB.".into());
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| "Cannot open portable export.".to_owned())?;
    let manifest_bytes = read_entry(&mut archive, "manifest.json", 65_536)?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| "Portable manifest is invalid.".to_owned())?;
    if manifest.format != "pm-app-json"
        || manifest.version != 1
        || manifest.schema_version != 1
        || manifest.entities_bytes == 0
        || manifest.entities_bytes > MAX_ENTITIES_BYTES
    {
        return Err("Portable export version is not supported.".into());
    }
    let mut expected = HashSet::from(["manifest.json".to_owned(), "entities.json".to_owned()]);
    let mut media_bytes = 0_usize;
    for item in &manifest.media {
        if item.bytes > crate::media::MAX_IMAGE_BYTES
            || !expected.insert(media_name(&item.id, &item.mime)?)
        {
            return Err("Portable export has repeated or oversized images.".into());
        }
        media_bytes = media_bytes.saturating_add(item.bytes);
    }
    if media_bytes > 400 * 1024 * 1024 || archive.len() != expected.len() {
        return Err("Portable export has unexpected entries.".into());
    }
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|_| "Cannot inspect portable export.".to_owned())?;
        if file.is_dir() || !expected.remove(file.name()) {
            return Err("Portable export has an unsafe or repeated path.".into());
        }
    }
    let entities = read_entry(&mut archive, "entities.json", MAX_ENTITIES_BYTES)?;
    if entities.len() != manifest.entities_bytes || checksum(&entities) != manifest.entities_sha256
    {
        return Err("Portable entities failed integrity check.".into());
    }
    let collections = serde_json::from_slice(&entities)
        .map_err(|_| "Portable entities are invalid.".to_owned())?;
    let mut media = Vec::with_capacity(manifest.media.len());
    for item in manifest.media {
        let image = read_entry(
            &mut archive,
            &media_name(&item.id, &item.mime)?,
            crate::media::MAX_IMAGE_BYTES,
        )?;
        if image.len() != item.bytes || checksum(&image) != item.sha256 {
            return Err("Portable image failed integrity check.".into());
        }
        media.push(MediaFile {
            id: item.id,
            mime: item.mime,
            bytes: image,
        });
    }
    Ok(PortableData {
        snapshot: restore_entities(root, collections)?,
        media,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{current_workspace_id, save_identity};
    use rusqlite::params;

    #[test]
    fn json_collections_roundtrip_relations_and_search() {
        let root = std::env::temp_dir().join(format!("pm-app-json-portable-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Igris Labs").unwrap();
        let workspace_id = current_workspace_id(&connection).unwrap();
        let project_id = Uuid::new_v4().to_string();
        connection.execute(
            "INSERT INTO project(id, workspace_id, name, identifier_prefix) VALUES (?1, ?2, 'Desktop app', 'APP')",
            params![project_id, workspace_id],
        ).unwrap();
        connection.execute(
            "INSERT INTO issue(id, workspace_id, identifier, project_id, title, status, priority, position)
             VALUES (?1, ?2, 'APP-1', ?3, 'Ship portable data', 'Todo', 'High', 0)",
            params![Uuid::new_v4().to_string(), workspace_id, project_id],
        ).unwrap();
        let mutation_id = Uuid::new_v4().to_string();
        connection.execute(
            "INSERT INTO mutation_receipt(mutation_id, workspace_id, operation, request_hash, result_json, workspace_revision)
             VALUES (?1, ?2, 'issue.create', 'hash', '{}', 2)",
            params![mutation_id, workspace_id],
        ).unwrap();
        let source = root.join("snapshot.sqlite");
        connection.backup(rusqlite::MAIN_DB, &source, None).unwrap();
        drop(connection);

        let archive = encode(&root, &fs::read(&source).unwrap(), &[]).unwrap();
        let mut zip = ZipArchive::new(Cursor::new(archive.as_slice())).unwrap();
        assert!(zip.by_name("entities.json").is_ok());
        assert!(zip.by_name("workspace.sqlite").is_err());
        drop(zip);

        let restored = decode(&root, &archive).unwrap();
        let path = root.join("restored.sqlite");
        fs::write(&path, restored.snapshot).unwrap();
        let reopened = Connection::open(&path).unwrap();
        let linked: String = reopened
            .query_row(
                "SELECT project.name FROM issue JOIN project ON project.id = issue.project_id
             WHERE issue.identifier = 'APP-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(linked, "Desktop app");
        let hits: i64 = reopened
            .query_row(
                "SELECT count(*) FROM issue_search WHERE issue_search MATCH 'portable'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(hits, 1);
        let receipts: i64 = reopened
            .query_row(
                "SELECT count(*) FROM mutation_receipt WHERE mutation_id = ?1",
                [&mutation_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(receipts, 0);
        drop(reopened);
        fs::remove_dir_all(root).unwrap();
    }
}
