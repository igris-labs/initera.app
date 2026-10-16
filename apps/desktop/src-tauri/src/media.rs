use std::{
    collections::HashSet,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{
    codecs::png::{CompressionType, FilterType, PngEncoder},
    io::Reader as ImageReader,
    GenericImageView, ImageEncoder, ImageFormat,
};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use uuid::Uuid;

use crate::storage::{document_data_root, open_database};

pub(crate) const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;
const MAX_PIXELS: u64 = 40_000_000;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaRecord {
    pub(crate) id: String,
    mime: String,
    width: u32,
    height: u32,
}

pub(crate) fn media_path(root: &Path, id: &str, mime: &str) -> Result<PathBuf, String> {
    Uuid::parse_str(id).map_err(|_| "Invalid image identifier.".to_owned())?;
    let extension = match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => return Err("Unsupported image type.".into()),
    };
    Ok(root.join("media").join(format!("{id}.{extension}")))
}

fn inspect_image(bytes: &[u8]) -> Result<(&'static str, image::DynamicImage), String> {
    if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
        return Err("Choose an image under 20 MB.".into());
    }
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| "Cannot identify this image.".to_owned())?;
    let format = reader.format().ok_or("Cannot identify this image.")?;
    let mime = match format {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::WebP => "image/webp",
        _ => return Err("Choose a PNG, JPEG, or WebP image.".into()),
    };
    let dimensions = reader
        .into_dimensions()
        .map_err(|_| "Cannot read image dimensions.".to_owned())?;
    if u64::from(dimensions.0) * u64::from(dimensions.1) > MAX_PIXELS {
        return Err("Choose an image below 40 megapixels.".into());
    }
    let decoded = image::load_from_memory_with_format(bytes, format)
        .map_err(|_| "Image data is damaged.".to_owned())?;
    if decoded.dimensions() != dimensions {
        return Err("Image dimensions changed while decoding.".into());
    }
    Ok((mime, decoded))
}

pub(crate) fn stage_image(
    root: &Path,
    connection: &Connection,
    bytes: &[u8],
) -> Result<MediaRecord, String> {
    // Store media beside SQLite and keep only its UUID/metadata in the database. PNG can be
    // recompressed losslessly; preserve JPEG/WebP bytes to avoid a lossy transcode.
    let (mime, decoded) = inspect_image(bytes)?;
    let (width, height) = decoded.dimensions();
    let mut optimized = Vec::new();
    if mime == "image/png" {
        PngEncoder::new_with_quality(&mut optimized, CompressionType::Best, FilterType::Adaptive)
            .write_image(decoded.as_bytes(), width, height, decoded.color())
            .map_err(|_| "Cannot compress image.".to_owned())?;
    }
    // JPEG and WebP uploads are already compressed. Keep their original bytes and metadata.
    let saved_bytes = if !optimized.is_empty() && optimized.len() < bytes.len() {
        optimized.as_slice()
    } else {
        bytes
    };
    let id = Uuid::new_v4().to_string();
    let temporary = root.join("staging").join(format!("{id}.image"));
    let final_path = media_path(root, &id, mime)?;
    let saved = (|| {
        fs::write(&temporary, saved_bytes).map_err(|_| "Cannot stage image.".to_owned())?;
        fs::File::open(&temporary)
            .and_then(|file| file.sync_all())
            .map_err(|_| "Cannot secure staged image.".to_owned())?;
        fs::rename(&temporary, &final_path).map_err(|_| "Cannot save image.".to_owned())
    })();
    if let Err(error) = saved {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    let hash = format!("{:x}", Sha256::digest(saved_bytes));
    if let Err(error) = connection.execute(
        "INSERT INTO media(id, mime, bytes, width, height, sha256) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, mime, saved_bytes.len(), width, height, hash],
    ) {
        let _ = fs::remove_file(final_path);
        return Err(format!("Cannot register image: {error}"));
    }
    Ok(MediaRecord {
        id,
        mime: mime.into(),
        width,
        height,
    })
}

pub(crate) fn cleanup_media(root: &Path, connection: &Connection) -> Result<(), String> {
    // Grace period protects images in unsaved editor drafts; the transaction prevents a save
    // from attaching an old image between the reference scan and its record deletion.
    let transaction = Transaction::new_unchecked(connection, TransactionBehavior::Immediate)
        .map_err(|_| "Cannot begin media cleanup.".to_owned())?;
    let removed = (|| {
        let mut references = HashSet::new();
        for table in [
            "issue.description_document_json",
            "project.description_document_json",
            "issue_activity.body_document_json",
            "draft.payload_json",
        ] {
            let (name, column) = table.split_once('.').unwrap();
            let query = format!(
                "SELECT json_tree.value FROM {name}, json_tree({name}.{column}) WHERE json_tree.key = 'mediaId'"
            );
            let mut statement = transaction
                .prepare(&query)
                .map_err(|_| "Cannot inspect image references.".to_owned())?;
            let rows = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|_| "Cannot inspect image references.".to_owned())?;
            for row in rows {
                references.insert(row.map_err(|_| "Cannot inspect image references.".to_owned())?);
            }
        }

        let mut statement = transaction
            .prepare(
                "SELECT id, mime FROM media WHERE datetime(created_at) < datetime('now', '-1 day')",
            )
            .map_err(|_| "Cannot inspect old images.".to_owned())?;
        let stale = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|_| "Cannot inspect old images.".to_owned())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "Cannot inspect old images.".to_owned())?;
        let mut removed = Vec::new();
        for (id, mime) in stale {
            if references.contains(&id) {
                continue;
            }
            transaction
                .execute("DELETE FROM media WHERE id = ?1", [&id])
                .map_err(|_| "Cannot clear unused image record.".to_owned())?;
            removed.push((id, mime));
        }
        Ok::<_, String>(removed)
    })()?;
    transaction
        .commit()
        .map_err(|_| "Cannot finish media cleanup.".to_owned())?;
    for (id, mime) in removed {
        let path = media_path(root, &id, &mime)?;
        if path.exists() {
            fs::remove_file(path).map_err(|_| "Cannot clear unused image.".to_owned())?;
        }
    }

    let cutoff = SystemTime::now() - Duration::from_secs(24 * 60 * 60);
    for folder in ["media", "staging"] {
        for item in fs::read_dir(root.join(folder))
            .map_err(|_| "Cannot inspect media storage.".to_owned())?
        {
            let path = item
                .map_err(|_| "Cannot inspect media storage.".to_owned())?
                .path();
            let metadata =
                fs::metadata(&path).map_err(|_| "Cannot inspect media storage.".to_owned())?;
            if metadata.modified().is_ok_and(|modified| modified >= cutoff) {
                continue;
            }
            if folder == "media" {
                let Some((id, mime)) = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| name.rsplit_once('.'))
                else {
                    continue;
                };
                let mime = match mime {
                    "png" => "image/png",
                    "jpg" => "image/jpeg",
                    "webp" => "image/webp",
                    _ => continue,
                };
                if Uuid::parse_str(id).is_err() || media_path(root, id, mime)? != path {
                    continue;
                }
                let exists: bool = connection
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM media WHERE id = ?1)",
                        [id],
                        |row| row.get(0),
                    )
                    .map_err(|_| "Cannot inspect image record.".to_owned())?;
                if exists {
                    continue;
                }
                fs::remove_file(path).map_err(|_| "Cannot clear unused image.".to_owned())?;
            } else if metadata.is_dir() {
                fs::remove_dir_all(path).map_err(|_| "Cannot clear stale staging.".to_owned())?;
            } else {
                fs::remove_file(path).map_err(|_| "Cannot clear stale staging.".to_owned())?;
            }
        }
    }
    Ok(())
}

fn read_image(root: &Path, connection: &Connection, id: &str) -> Result<String, String> {
    Uuid::parse_str(id).map_err(|_| "Invalid image identifier.".to_owned())?;
    let (mime, expected_bytes, hash): (String, usize, String) = connection
        .query_row(
            "SELECT mime, bytes, sha256 FROM media WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| "Cannot find image.".to_owned())?
        .ok_or("Image is unavailable.")?;
    if expected_bytes > MAX_IMAGE_BYTES {
        return Err("Image exceeds the size limit.".into());
    }
    let bytes = fs::read(media_path(root, id, &mime)?)
        .map_err(|_| "Image file is unavailable.".to_owned())?;
    if bytes.len() != expected_bytes || format!("{:x}", Sha256::digest(&bytes)) != hash {
        return Err("Image file failed its integrity check.".into());
    }
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}

pub(crate) fn validate_references(
    connection: &Connection,
    document: Option<&Value>,
) -> Result<(), String> {
    fn visit(connection: &Connection, node: &Value) -> Result<(), String> {
        if let Some(id) = node
            .get("attrs")
            .and_then(|attrs| attrs.get("mediaId"))
            .and_then(Value::as_str)
        {
            Uuid::parse_str(id).map_err(|_| "Invalid image identifier.".to_owned())?;
            let exists: bool = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM media WHERE id = ?1)",
                    [id],
                    |row| row.get(0),
                )
                .map_err(|_| "Cannot check image reference.".to_owned())?;
            if !exists {
                return Err("Image is unavailable. Paste it again.".into());
            }
        }
        if let Some(children) = node.get("content").and_then(Value::as_array) {
            for child in children {
                visit(connection, child)?;
            }
        }
        Ok(())
    }
    if let Some(content) = document.and_then(|value| value.get("content")) {
        visit(connection, content)?;
    }
    Ok(())
}

#[api_contract_macros::command(mutation)]
pub fn storage_stage_image(app: AppHandle, source: String) -> Result<MediaRecord, String> {
    let encoded = source
        .split_once(";base64,")
        .filter(|(prefix, _)| {
            matches!(
                *prefix,
                "data:image/png" | "data:image/jpeg" | "data:image/webp"
            )
        })
        .map(|(_, data)| data)
        .ok_or("Choose a PNG, JPEG, or WebP image.")?;
    if encoded.len() > (MAX_IMAGE_BYTES + 2) / 3 * 4 {
        return Err("Choose an image under 20 MB.".into());
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| "Cannot read image data.".to_owned())?;
    let root = document_data_root(&app)?;
    let connection = open_database(&root)?;
    stage_image(&root, &connection, &bytes)
}

#[api_contract_macros::command]
pub fn storage_read_image(app: AppHandle, id: String) -> Result<String, String> {
    let root = document_data_root(&app)?;
    let connection = open_database(&root)?;
    read_image(&root, &connection, &id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{current_workspace_id, save_identity};

    #[test]
    fn staged_image_survives_reopen_and_rejects_damaged_data() {
        let root = std::env::temp_dir().join(format!("pm-app-media-test-{}", Uuid::new_v4()));
        let connection = open_database(&root).unwrap();
        let image = image::DynamicImage::new_rgba8(2, 2);
        let mut bytes = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        let record = stage_image(&root, &connection, &bytes).unwrap();
        let stored = fs::read(media_path(&root, &record.id, "image/png").unwrap()).unwrap();
        assert!(stored.len() <= bytes.len());
        assert_eq!(
            image::load_from_memory(&stored).unwrap().to_rgba8(),
            image::load_from_memory(&bytes).unwrap().to_rgba8()
        );
        let document = serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "content": [
                {"type": "image", "attrs": {"mediaId": record.id, "src": "data:image/png;base64,iVBORw0KGgo=", "alt": "Diagram"}}
            ]}
        });
        let encoded = crate::content::encode_document_for_storage(&connection, Some(&document))
            .unwrap()
            .unwrap();
        assert!(encoded.contains(&record.id));
        assert!(!encoded.contains("data:image"));
        let missing = serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "content": [
                {"type": "image", "attrs": {"mediaId": Uuid::new_v4().to_string()}}
            ]}
        });
        assert!(crate::content::encode_document_for_storage(&connection, Some(&missing)).is_err());
        drop(connection);
        assert!(
            read_image(&root, &open_database(&root).unwrap(), &record.id)
                .unwrap()
                .starts_with("data:image/png;base64,")
        );
        assert!(stage_image(&root, &open_database(&root).unwrap(), b"not an image").is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cleanup_keeps_referenced_images_and_removes_old_unused_files() {
        let root = std::env::temp_dir().join(format!("pm-app-media-cleanup-{}", Uuid::new_v4()));
        let mut connection = open_database(&root).unwrap();
        save_identity(&mut connection, "Rahul", "Media workspace").unwrap();
        let mut bytes = Vec::new();
        image::DynamicImage::new_rgba8(2, 2)
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        let kept = stage_image(&root, &connection, &bytes).unwrap();
        let removed = stage_image(&root, &connection, &bytes).unwrap();
        connection.execute(
            "INSERT INTO issue(id, workspace_id, identifier, title, status, priority, position, description_document_json)
             VALUES (?1, ?2, 'MEDIA-1', 'Referenced image', 'Todo', 'No priority', 0, ?3)",
            params![
                Uuid::new_v4().to_string(),
                current_workspace_id(&connection).unwrap(),
                serde_json::json!({"version": 1, "content": {"type": "doc", "content": [
                    {"type": "image", "attrs": {"mediaId": kept.id}}
                ]}}).to_string()
            ],
        ).unwrap();
        connection
            .execute(
                "UPDATE media SET created_at = datetime('now', '-2 days')",
                [],
            )
            .unwrap();

        let orphan = media_path(&root, &Uuid::new_v4().to_string(), "image/png").unwrap();
        fs::write(&orphan, &bytes).unwrap();
        let abandoned = root.join("staging").join("abandoned.image");
        fs::write(&abandoned, &bytes).unwrap();
        let old = fs::FileTimes::new()
            .set_modified(SystemTime::now() - Duration::from_secs(2 * 24 * 60 * 60));
        fs::File::open(&orphan).unwrap().set_times(old).unwrap();
        fs::File::open(&abandoned).unwrap().set_times(old).unwrap();

        cleanup_media(&root, &connection).unwrap();
        assert!(media_path(&root, &kept.id, "image/png").unwrap().exists());
        assert!(!media_path(&root, &removed.id, "image/png")
            .unwrap()
            .exists());
        assert!(!orphan.exists());
        assert!(!abandoned.exists());
        assert!(read_image(&root, &connection, &kept.id).is_ok());
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }
}
