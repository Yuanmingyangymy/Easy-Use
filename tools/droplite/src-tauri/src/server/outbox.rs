use crate::{errors::AppError, storage::filename::sanitize_filename};
use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::now_epoch_secs;

const OUTBOX_ID_LEN: usize = 24;

#[derive(Clone, Debug)]
pub struct OutboxEntry {
    pub item: OutboxItem,
    pub content: OutboxContent,
}

#[derive(Clone, Debug)]
pub enum OutboxContent {
    Text(String),
    File { path: PathBuf },
}

#[derive(Clone, Debug, Serialize)]
pub struct OutboxItem {
    pub id: String,
    pub kind: OutboxKind,
    pub display_name: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub created_at: String,
    pub status: OutboxStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutboxKind {
    File,
    Image,
    Text,
    Video,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutboxStatus {
    Downloaded,
    Expired,
    Pending,
}

#[derive(Clone, Debug, Default)]
pub struct OutboxState {
    items: HashMap<String, OutboxEntry>,
}

impl OutboxState {
    pub fn add_text(&mut self, content: String) -> Result<OutboxItem, AppError> {
        let content = content.trim().to_string();
        if content.is_empty() {
            return Err(AppError::BadRequest("Text is empty.".to_string()));
        }

        let item = OutboxItem {
            id: new_outbox_id(),
            kind: OutboxKind::Text,
            display_name: "Text".to_string(),
            mime_type: Some("text/plain".to_string()),
            size_bytes: Some(content.as_bytes().len() as u64),
            created_at: created_at_string(),
            status: OutboxStatus::Pending,
            content: Some(content.clone()),
        };

        self.items.insert(
            item.id.clone(),
            OutboxEntry {
                item: item.clone(),
                content: OutboxContent::Text(content),
            },
        );

        Ok(item)
    }

    pub fn add_file(&mut self, path: PathBuf) -> Result<OutboxItem, AppError> {
        let metadata = std::fs::metadata(&path)?;
        if metadata.is_dir() {
            return Err(AppError::BadRequest(
                "Outbox file path points to a directory.".to_string(),
            ));
        }
        if !metadata.is_file() {
            return Err(AppError::BadRequest(
                "Outbox path is not a regular file.".to_string(),
            ));
        }

        let canonical_path = std::fs::canonicalize(&path)?;
        let display_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .map(sanitize_filename)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "file.bin".to_string());
        let mime_type = mime_guess::from_path(&canonical_path)
            .first()
            .map(|value| value.essence_str().to_string());
        let kind = kind_for_path(mime_type.as_deref(), &canonical_path);

        let item = OutboxItem {
            id: new_outbox_id(),
            kind,
            display_name,
            mime_type,
            size_bytes: Some(metadata.len()),
            created_at: created_at_string(),
            status: OutboxStatus::Pending,
            content: None,
        };

        self.items.insert(
            item.id.clone(),
            OutboxEntry {
                item: item.clone(),
                content: OutboxContent::File {
                    path: canonical_path,
                },
            },
        );

        Ok(item)
    }

    pub fn list(&self) -> Vec<OutboxItem> {
        let mut items: Vec<_> = self
            .items
            .values()
            .map(|entry| entry.item.clone())
            .collect();
        items.sort_by(|left, right| left.created_at.cmp(&right.created_at));
        items
    }

    pub fn get(&self, id: &str) -> Option<OutboxEntry> {
        self.items.get(id).cloned()
    }

    pub fn acknowledge(&mut self, id: &str) -> Result<OutboxItem, AppError> {
        let entry = self
            .items
            .get_mut(id)
            .ok_or_else(|| AppError::NotFound("Outbox item was not found.".to_string()))?;
        entry.item.status = OutboxStatus::Downloaded;
        Ok(entry.item.clone())
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

pub fn kind_for_path(mime_type: Option<&str>, path: &Path) -> OutboxKind {
    if mime_type.is_some_and(|value| value.starts_with("image/")) {
        return OutboxKind::Image;
    }
    if mime_type.is_some_and(|value| value.starts_with("video/")) {
        return OutboxKind::Video;
    }

    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" | "gif") => OutboxKind::Image,
        Some("mp4" | "mov" | "webm") => OutboxKind::Video,
        _ => OutboxKind::File,
    }
}

pub fn content_disposition(display_name: &str) -> String {
    let safe_name = sanitize_filename(display_name);
    let ascii_name: String = safe_name
        .chars()
        .map(|character| match character {
            '"' | '\\' => '_',
            character if character.is_ascii_graphic() || character == ' ' => character,
            _ => '_',
        })
        .collect();

    format!("attachment; filename=\"{ascii_name}\"")
}

fn new_outbox_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(OUTBOX_ID_LEN)
        .map(char::from)
        .collect()
}

fn created_at_string() -> String {
    chrono::DateTime::from_timestamp(now_epoch_secs() as i64, 0)
        .map(|datetime| datetime.to_rfc3339())
        .unwrap_or_else(|| "1970-01-01T00:00:00+00:00".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    fn test_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("droplite-outbox-{name}-{}", now_epoch_secs()))
    }

    #[test]
    fn creates_text_outbox_item() {
        let mut outbox = OutboxState::default();

        let item = outbox
            .add_text("hello phone".to_string())
            .expect("text item");

        assert_eq!(item.kind, OutboxKind::Text);
        assert_eq!(item.status, OutboxStatus::Pending);
        assert_eq!(item.content.as_deref(), Some("hello phone"));
        assert_eq!(outbox.list().len(), 1);
    }

    #[test]
    fn rejects_empty_text() {
        let mut outbox = OutboxState::default();

        assert!(matches!(
            outbox.add_text("   ".to_string()),
            Err(AppError::BadRequest(_))
        ));
    }

    #[test]
    fn creates_file_outbox_item_without_exposing_path() {
        let mut outbox = OutboxState::default();
        let dir = test_path("file-dir");
        fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("file.pdf");
        fs::write(&path, b"pdf").expect("write");

        let item = outbox.add_file(path.clone()).expect("file item");

        assert_eq!(item.kind, OutboxKind::File);
        assert_eq!(item.display_name, "file.pdf");
        assert_eq!(item.size_bytes, Some(3));
        assert!(serde_json::to_string(&item)
            .expect("json")
            .contains("file.pdf"));
        assert!(!serde_json::to_string(&item)
            .expect("json")
            .contains(&path.display().to_string()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_missing_file_and_directory() {
        let mut outbox = OutboxState::default();
        let missing = test_path("missing.txt");
        assert!(outbox.add_file(missing).is_err());

        let dir = test_path("directory");
        fs::create_dir_all(&dir).expect("dir");
        assert!(matches!(
            outbox.add_file(dir.clone()),
            Err(AppError::BadRequest(_))
        ));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_media_kinds() {
        assert_eq!(
            kind_for_path(Some("image/jpeg"), Path::new("anything.bin")),
            OutboxKind::Image
        );
        assert_eq!(
            kind_for_path(Some("video/mp4"), Path::new("anything.bin")),
            OutboxKind::Video
        );
        assert_eq!(
            kind_for_path(None, Path::new("photo.png")),
            OutboxKind::Image
        );
        assert_eq!(
            kind_for_path(None, Path::new("clip.mov")),
            OutboxKind::Video
        );
        assert_eq!(
            kind_for_path(None, Path::new("report.pdf")),
            OutboxKind::File
        );
    }

    #[test]
    fn ack_marks_downloaded() {
        let mut outbox = OutboxState::default();
        let item = outbox.add_text("hello".to_string()).expect("item");

        let updated = outbox.acknowledge(&item.id).expect("ack");

        assert_eq!(updated.status, OutboxStatus::Downloaded);
    }

    #[test]
    fn clear_removes_old_session_items() {
        let mut outbox = OutboxState::default();
        outbox.add_text("hello".to_string()).expect("item");

        outbox.clear();

        assert!(outbox.list().is_empty());
    }

    #[test]
    fn content_disposition_sanitizes_filename() {
        assert_eq!(
            content_disposition("../bad:name.pdf"),
            "attachment; filename=\"bad_name.pdf\""
        );
        assert_eq!(
            content_disposition("报告.pdf"),
            "attachment; filename=\"__.pdf\""
        );
    }
}
