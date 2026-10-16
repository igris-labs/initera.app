use std::{
    collections::HashSet,
    fs::{self, File},
    io::{Cursor, Read, Seek, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const MAX_SNAPSHOT_BYTES: usize = 100 * 1024 * 1024;
const MAX_MEDIA_BYTES: usize = 400 * 1024 * 1024;
const MAX_ARCHIVE_BYTES: usize = 550 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    version: u8,
    snapshot_bytes: usize,
    snapshot_sha256: String,
    #[serde(default)]
    media: Vec<MediaManifestEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MediaManifestEntry {
    id: String,
    mime: String,
    bytes: usize,
    sha256: String,
}

pub(crate) struct MediaFile {
    pub id: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

pub(crate) struct PortableData {
    pub snapshot: Vec<u8>,
    pub media: Vec<MediaFile>,
}

fn checksum(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn media_name(id: &str, mime: &str) -> Result<String, String> {
    Uuid::parse_str(id).map_err(|_| "Portable image has an invalid ID.".to_owned())?;
    let extension = match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => return Err("Portable image has an unsupported type.".into()),
    };
    Ok(format!("media/{id}.{extension}"))
}

fn write_archive<W: Write + Seek>(
    snapshot: &[u8],
    media: &[MediaFile],
    output: W,
) -> Result<W, String> {
    if snapshot.is_empty() || snapshot.len() > MAX_SNAPSHOT_BYTES {
        return Err("Portable export exceeds the 100 MB database limit.".into());
    }
    let mut total_media = 0_usize;
    let mut names = HashSet::new();
    let mut media_manifest = Vec::with_capacity(media.len());
    for file in media {
        let name = media_name(&file.id, &file.mime)?;
        if !names.insert(name) || file.bytes.len() > crate::media::MAX_IMAGE_BYTES {
            return Err("Portable export contains a repeated or oversized image.".into());
        }
        total_media = total_media.saturating_add(file.bytes.len());
        if total_media > MAX_MEDIA_BYTES {
            return Err("Portable images exceed the 400 MB limit.".into());
        }
        media_manifest.push(MediaManifestEntry {
            id: file.id.clone(),
            mime: file.mime.clone(),
            bytes: file.bytes.len(),
            sha256: checksum(&file.bytes),
        });
    }

    let manifest = Manifest {
        format: "pm-app-portable".into(),
        version: 2,
        snapshot_bytes: snapshot.len(),
        snapshot_sha256: checksum(snapshot),
        media: media_manifest,
    };
    let mut writer = ZipWriter::new(output);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    writer
        .start_file("manifest.json", options)
        .map_err(|_| "Cannot begin portable export.".to_owned())?;
    writer
        .write_all(&serde_json::to_vec(&manifest).map_err(|_| "Cannot encode export manifest.")?)
        .map_err(|_| "Cannot write export manifest.".to_owned())?;
    writer
        .start_file("workspace.sqlite", options)
        .map_err(|_| "Cannot add workspace to export.".to_owned())?;
    writer
        .write_all(snapshot)
        .map_err(|_| "Cannot write workspace export.".to_owned())?;
    for file in media {
        writer
            .start_file(media_name(&file.id, &file.mime)?, options)
            .map_err(|_| "Cannot add image to export.".to_owned())?;
        writer
            .write_all(&file.bytes)
            .map_err(|_| "Cannot write image export.".to_owned())?;
    }
    writer
        .finish()
        .map_err(|_| "Cannot finish portable export.".to_owned())
}

pub(crate) fn encode(snapshot: &[u8], media: &[MediaFile]) -> Result<Vec<u8>, String> {
    let bytes = write_archive(snapshot, media, Cursor::new(Vec::new()))?.into_inner();
    if bytes.len() > MAX_ARCHIVE_BYTES {
        return Err("Portable export exceeds the archive limit.".into());
    }
    Ok(bytes)
}

pub(crate) fn encode_to_file(
    snapshot: &[u8],
    media: &[MediaFile],
    path: &Path,
) -> Result<(), String> {
    let result = (|| {
        let file = File::create(path).map_err(|_| "Cannot create portable export.".to_owned())?;
        let file = write_archive(snapshot, media, file)?;
        if file
            .metadata()
            .map_err(|_| "Cannot inspect portable export.".to_owned())?
            .len()
            > MAX_ARCHIVE_BYTES as u64
        {
            return Err("Portable export exceeds the archive limit.".into());
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}

pub(crate) fn decode(bytes: &[u8]) -> Result<PortableData, String> {
    if bytes.is_empty() || bytes.len() > MAX_ARCHIVE_BYTES {
        return Err("Choose a portable export smaller than 550 MB.".into());
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| "Cannot open this portable export.".to_owned())?;
    let mut manifest_bytes = Vec::new();
    archive
        .by_name("manifest.json")
        .map_err(|_| "Portable export has no manifest.".to_owned())?
        .take(65_537)
        .read_to_end(&mut manifest_bytes)
        .map_err(|_| "Cannot read portable manifest.".to_owned())?;
    if manifest_bytes.len() > 65_536 {
        return Err("Portable manifest is too large.".into());
    }
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| "Portable manifest is invalid.".to_owned())?;
    if manifest.format != "pm-app-portable"
        || !matches!(manifest.version, 1 | 2)
        || manifest.snapshot_bytes == 0
        || manifest.snapshot_bytes > MAX_SNAPSHOT_BYTES
        || (manifest.version == 1 && !manifest.media.is_empty())
    {
        return Err("Portable export format is not supported.".into());
    }
    let mut expected = HashSet::from(["manifest.json".to_owned(), "workspace.sqlite".to_owned()]);
    let mut total_media = 0_usize;
    for item in &manifest.media {
        let name = media_name(&item.id, &item.mime)?;
        if !expected.insert(name) || item.bytes > crate::media::MAX_IMAGE_BYTES {
            return Err("Portable export contains a repeated or oversized image.".into());
        }
        total_media = total_media.saturating_add(item.bytes);
    }
    if total_media > MAX_MEDIA_BYTES || archive.len() != expected.len() {
        return Err("Portable export contains unexpected files.".into());
    }
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|_| "Cannot inspect portable export.".to_owned())?;
        if file.is_dir() || !expected.remove(file.name()) {
            return Err("Portable export has an unsafe or repeated file name.".into());
        }
    }
    if !expected.is_empty() {
        return Err("Portable export is missing files.".into());
    }
    let mut snapshot = Vec::new();
    archive
        .by_name("workspace.sqlite")
        .map_err(|_| "Portable export has no workspace.".to_owned())?
        .take((MAX_SNAPSHOT_BYTES + 1) as u64)
        .read_to_end(&mut snapshot)
        .map_err(|_| "Cannot read portable workspace.".to_owned())?;
    if snapshot.len() != manifest.snapshot_bytes || checksum(&snapshot) != manifest.snapshot_sha256
    {
        return Err("Portable export failed its integrity check.".into());
    }
    let mut media = Vec::with_capacity(manifest.media.len());
    for item in manifest.media {
        let mut image = Vec::new();
        archive
            .by_name(&media_name(&item.id, &item.mime)?)
            .map_err(|_| "Portable image is missing.".to_owned())?
            .take((crate::media::MAX_IMAGE_BYTES + 1) as u64)
            .read_to_end(&mut image)
            .map_err(|_| "Cannot read portable image.".to_owned())?;
        if image.len() != item.bytes || checksum(&image) != item.sha256 {
            return Err("Portable image failed its integrity check.".into());
        }
        media.push(MediaFile {
            id: item.id,
            mime: item.mime,
            bytes: image,
        });
    }
    Ok(PortableData { snapshot, media })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_corruption_check() {
        let id = Uuid::new_v4().to_string();
        let image = MediaFile {
            id: id.clone(),
            mime: "image/png".into(),
            bytes: b"image".to_vec(),
        };
        let bytes = encode(b"snapshot", &[image]).unwrap();
        let decoded = decode(&bytes).unwrap();
        assert_eq!(decoded.snapshot, b"snapshot");
        assert_eq!(decoded.media[0].bytes, b"image");
        assert_eq!(decoded.media[0].id, id);
        assert!(decode(b"not a zip").is_err());
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        archive
            .start_file("../workspace.sqlite", SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"unsafe").unwrap();
        assert!(decode(&archive.finish().unwrap().into_inner()).is_err());
    }

    #[test]
    fn file_export_roundtrips_without_building_an_archive_buffer() {
        let path = std::env::temp_dir().join(format!("pm-app-export-{}.zip", Uuid::new_v4()));
        let image = MediaFile {
            id: Uuid::new_v4().to_string(),
            mime: "image/png".into(),
            bytes: b"image bytes".to_vec(),
        };

        encode_to_file(b"legacy snapshot", &[image], &path).unwrap();
        let decoded = decode(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(decoded.snapshot, b"legacy snapshot");
        assert_eq!(decoded.media[0].bytes, b"image bytes");
        fs::remove_file(path).unwrap();
    }
}
