use rusqlite::Connection;
use serde_json::Value;
use uuid::Uuid;

const MAX_DOCUMENT_BYTES: usize = 2_097_152;

/// Keep editor JSON bounded and restricted to the nodes supported by the app.
pub(crate) fn encode_document(document: Option<&Value>) -> Result<Option<String>, String> {
    let Some(document) = document else {
        return Ok(None);
    };
    let mut normalized = document.clone();
    if normalized.get("version").and_then(Value::as_u64) != Some(1) {
        return Err("Unsupported description document version.".into());
    }
    if let Some(content) = normalized.get_mut("content") {
        strip_transient_image_sources(content);
    }
    let content = normalized
        .get("content")
        .ok_or("Description document has no content.")?;
    validate_node(content, 0)?;
    if content.get("type").and_then(Value::as_str) != Some("doc") {
        return Err("Description document must have a root document node.".into());
    }
    let encoded = serde_json::to_string(&normalized)
        .map_err(|_| "Cannot encode the description document.".to_owned())?;
    if encoded.len() > MAX_DOCUMENT_BYTES {
        return Err("Description document is too large.".into());
    }
    Ok(Some(encoded))
}

pub(crate) fn encode_document_for_storage(
    connection: &Connection,
    document: Option<&Value>,
) -> Result<Option<String>, String> {
    let encoded = encode_document(document)?;
    crate::media::validate_references(connection, document)?;
    Ok(encoded)
}

fn strip_transient_image_sources(node: &mut Value) {
    if node.get("type").and_then(Value::as_str) == Some("image") {
        if let Some(attrs) = node.get_mut("attrs").and_then(Value::as_object_mut) {
            if attrs.get("mediaId").and_then(Value::as_str).is_some() {
                attrs.remove("src");
            }
        }
    }
    if let Some(children) = node.get_mut("content").and_then(Value::as_array_mut) {
        for child in children {
            strip_transient_image_sources(child);
        }
    }
}

fn validate_node(node: &Value, depth: usize) -> Result<(), String> {
    if depth > 32 {
        return Err("Description document is nested too deeply.".into());
    }
    let kind = node.get("type").and_then(Value::as_str).unwrap_or("");
    if ![
        "doc",
        "paragraph",
        "text",
        "heading",
        "blockquote",
        "bulletList",
        "orderedList",
        "listItem",
        "taskList",
        "taskItem",
        "codeBlock",
        "horizontalRule",
        "hardBreak",
        "image",
    ]
    .contains(&kind)
    {
        return Err("Description document contains an unsupported block.".into());
    }
    if node.as_object().is_some_and(|object| {
        object.keys().any(|key| {
            !matches!(
                key.as_str(),
                "type" | "text" | "attrs" | "marks" | "content"
            )
        })
    }) || (kind != "text" && node.get("text").is_some())
        || (kind == "text" && node.get("text").and_then(Value::as_str).is_none())
    {
        return Err("Description document contains an invalid block.".into());
    }
    if kind == "image" {
        let attrs = node
            .get("attrs")
            .and_then(Value::as_object)
            .ok_or("Image has no attributes.")?;
        if attrs
            .keys()
            .any(|key| !matches!(key.as_str(), "src" | "alt" | "mediaId"))
        {
            return Err("Image contains an unsupported attribute.".into());
        }
        if let Some(id) = attrs.get("mediaId").and_then(Value::as_str) {
            Uuid::parse_str(id).map_err(|_| "Image has an invalid media identifier.")?;
        } else {
            let src = attrs.get("src").and_then(Value::as_str).unwrap_or("");
            let supported = ["png", "jpeg", "gif", "webp"]
                .iter()
                .any(|format| src.starts_with(&format!("data:image/{format};base64,")));
            if !supported
                || src.len() > 1_500_000
                || !src.chars().all(|character| {
                    character.is_ascii_alphanumeric() || "+/=:;,".contains(character)
                })
            {
                return Err("Image must be a supported embedded image under 1 MB.".into());
            }
        }
        if attrs
            .get("alt")
            .and_then(Value::as_str)
            .is_some_and(|alt| alt.chars().count() > 200)
        {
            return Err("Image description is too long.".into());
        }
    }
    if let Some(marks) = node.get("marks") {
        let marks = marks
            .as_array()
            .ok_or("Description marks must be a list.")?;
        for mark in marks {
            let mark_type = mark.get("type").and_then(Value::as_str).unwrap_or("");
            if !["bold", "italic", "strike", "code", "link"].contains(&mark_type) {
                return Err("Description document contains unsupported formatting.".into());
            }
            if mark_type == "link" {
                let href = mark
                    .get("attrs")
                    .and_then(|attrs| attrs.get("href"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if !(href.starts_with("https://") || href.starts_with("http://")) {
                    return Err("Description links must use http or https.".into());
                }
            }
        }
    }
    if let Some(children) = node.get("content") {
        let children = children
            .as_array()
            .ok_or("Description content must be a list.")?;
        for child in children {
            validate_node(child, depth + 1)?;
        }
    }
    Ok(())
}

pub(crate) fn decode_document(encoded: Option<String>) -> Result<Option<Value>, rusqlite::Error> {
    encoded
        .map(|value| serde_json::from_str(&value).map_err(|_| rusqlite::Error::InvalidQuery))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_links_and_unknown_blocks() {
        let unsafe_link = serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "content": [{
                "type": "paragraph", "content": [{
                    "type": "text", "text": "Open", "marks": [{
                        "type": "link", "attrs": {"href": "javascript:alert(1)"}
                    }]
                }]
            }]}
        });
        assert!(encode_document(Some(&unsafe_link)).is_err());

        let unknown_block = serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "content": [{"type": "image", "attrs": {"src": "file:///private"}}]}
        });
        assert!(encode_document(Some(&unknown_block)).is_err());

        let invisible_text = serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "text": "hidden", "content": []}
        });
        assert!(encode_document(Some(&invisible_text)).is_err());
    }

    #[test]
    fn accepts_embedded_images_and_rejects_external_sources() {
        let image = serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "content": [{
                "type": "image", "attrs": {"src": "data:image/png;base64,iVBORw0KGgo=", "alt": "Diagram"}
            }]}
        });
        assert!(encode_document(Some(&image)).is_ok());

        let external = serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "content": [{
                "type": "image", "attrs": {"src": "https://example.com/track.png"}
            }]}
        });
        assert!(encode_document(Some(&external)).is_err());
    }

    #[test]
    fn managed_image_persists_only_its_id_and_alt_text() {
        let id = Uuid::new_v4().to_string();
        let document = serde_json::json!({
            "version": 1,
            "content": {"type": "doc", "content": [{
                "type": "image",
                "attrs": {
                    "mediaId": id,
                    "src": format!("data:image/png;base64,{}", "A".repeat(2_500_000)),
                    "alt": "Diagram"
                }
            }]}
        });
        let encoded = encode_document(Some(&document)).unwrap().unwrap();
        assert!(encoded.contains(&id));
        assert!(!encoded.contains("data:image"));
        assert!(encoded.contains("Diagram"));
    }
}
