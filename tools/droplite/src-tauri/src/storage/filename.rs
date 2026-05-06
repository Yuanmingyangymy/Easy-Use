use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const FALLBACK_NAME: &str = "file";
const MAX_FILENAME_CHARS: usize = 120;
const MAX_EXTENSION_CHARS: usize = 10;

pub fn sanitize_filename(input: &str) -> String {
    let base = input
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(FALLBACK_NAME)
        .trim();

    let sanitized: String = base
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            character if character.is_control() => '_',
            character => character,
        })
        .collect();

    let sanitized = sanitized.trim_matches([' ', '.']).trim();

    if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
        FALLBACK_NAME.to_string()
    } else {
        truncate_filename(&sanitized, MAX_FILENAME_CHARS)
    }
}

pub fn upload_filename(
    original_name: Option<&str>,
    mime: Option<&str>,
    received_at: u64,
    sequence: u16,
) -> String {
    let extension = original_name
        .and_then(extension_from_name)
        .or_else(|| extension_for_mime(mime).map(str::to_string))
        .unwrap_or_else(|| "bin".to_string());
    let prefix = fallback_prefix(mime, &extension);
    sanitize_filename(&format!(
        "{prefix}-{}-{sequence:03}.{extension}",
        compact_timestamp(received_at)
    ))
}

pub fn extension_for_mime(mime: Option<&str>) -> Option<&'static str> {
    match mime
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        "image/heic" => Some("heic"),
        "image/heif" => Some("heif"),
        "image/gif" => Some("gif"),
        "video/mp4" => Some("mp4"),
        "video/quicktime" => Some("mov"),
        "video/webm" => Some("webm"),
        "application/pdf" => Some("pdf"),
        "application/zip" => Some("zip"),
        "text/plain" => Some("txt"),
        _ => None,
    }
}

pub fn unique_path(directory: &Path, requested_name: &str) -> PathBuf {
    let safe_name = sanitize_filename(requested_name);
    let candidate = directory.join(&safe_name);

    if !candidate.exists() {
        return candidate;
    }

    let path = Path::new(&safe_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(FALLBACK_NAME);
    let extension = path.extension().and_then(|value| value.to_str());

    for index in 1..10_000 {
        let filename = match extension {
            Some(extension) => format!("{stem} ({index}).{extension}"),
            None => format!("{stem} ({index})"),
        };
        let candidate = directory.join(filename);
        if !candidate.exists() {
            return candidate;
        }
    }

    directory.join(format!("{stem} ({})", chrono_like_timestamp()))
}

pub fn unique_upload_path(
    directory: &Path,
    original_name: Option<&str>,
    mime: Option<&str>,
    received_at: u64,
) -> PathBuf {
    for sequence in 1..=999 {
        let filename = upload_filename(original_name, mime, received_at, sequence);
        let candidate = directory.join(filename);
        if !candidate.exists() {
            return candidate;
        }
    }

    unique_path(
        directory,
        &upload_filename(original_name, mime, received_at, 1_000),
    )
}

pub fn unique_temp_path(directory: &Path) -> PathBuf {
    for index in 0..10_000 {
        let filename = format!(".droplite-upload-{}-{index}.part", chrono_like_timestamp());
        let candidate = directory.join(filename);
        if !candidate.exists() {
            return candidate;
        }
    }

    directory.join(format!(".droplite-upload-{}.part", chrono_like_timestamp()))
}

fn extension_from_name(filename: &str) -> Option<String> {
    let sanitized = sanitize_filename(filename);
    Path::new(&sanitized)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.trim().trim_start_matches('.'))
        .filter(|value| {
            !value.is_empty()
                && value.len() <= MAX_EXTENSION_CHARS
                && value
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
        .map(|value| value.to_ascii_lowercase())
}

fn fallback_prefix(mime: Option<&str>, extension: &str) -> &'static str {
    let mime = mime.unwrap_or("").to_ascii_lowercase();
    if mime.starts_with("image/")
        || matches!(
            extension,
            "jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" | "gif"
        )
    {
        "image"
    } else if mime.starts_with("video/") || matches!(extension, "mp4" | "mov" | "webm") {
        "video"
    } else {
        "file"
    }
}

fn truncate_filename(filename: &str, max_chars: usize) -> String {
    if filename.chars().count() <= max_chars {
        return filename.to_string();
    }

    let path = Path::new(filename);
    let extension = path.extension().and_then(|value| value.to_str());
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(FALLBACK_NAME);

    match extension {
        Some(extension) => {
            let suffix = format!(".{extension}");
            let stem_limit = max_chars.saturating_sub(suffix.chars().count()).max(1);
            let truncated_stem: String = stem.chars().take(stem_limit).collect();
            format!("{truncated_stem}{suffix}")
        }
        None => filename.chars().take(max_chars).collect(),
    }
}

fn compact_timestamp(timestamp: u64) -> String {
    chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .map(|datetime| {
            datetime
                .with_timezone(&chrono::Local)
                .format("%Y%m%d-%H%M%S")
                .to_string()
        })
        .unwrap_or_else(|| "19700101-000000".to_string())
}

fn chrono_like_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    fn test_dir(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("droplite-{name}-{}", chrono_like_timestamp()));
        fs::create_dir_all(&path).expect("create temp test dir");
        path
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn sanitize_handles_normal_and_unsafe_names() {
        assert_eq!(sanitize_filename("photo.jpg"), "photo.jpg");
        assert_eq!(sanitize_filename("../../secret.txt"), "secret.txt");
        assert_eq!(sanitize_filename("..\\secret.txt"), "secret.txt");
        assert_eq!(sanitize_filename("a:b*c?.png"), "a_b_c_.png");
        assert_eq!(sanitize_filename("..."), FALLBACK_NAME);
        assert_eq!(sanitize_filename(""), FALLBACK_NAME);
        assert_eq!(sanitize_filename("no-extension"), "no-extension");
    }

    #[test]
    fn sanitize_truncates_long_names_but_keeps_extension() {
        let name = format!("{}.mp4", "a".repeat(200));
        let sanitized = sanitize_filename(&name);

        assert!(sanitized.len() <= MAX_FILENAME_CHARS + 4);
        assert!(sanitized.ends_with(".mp4"));
    }

    #[test]
    fn infers_extensions_from_mime() {
        assert_eq!(extension_for_mime(Some("image/jpeg")), Some("jpg"));
        assert_eq!(extension_for_mime(Some("image/png")), Some("png"));
        assert_eq!(extension_for_mime(Some("video/mp4")), Some("mp4"));
        assert_eq!(extension_for_mime(Some("video/quicktime")), Some("mov"));
        assert_eq!(extension_for_mime(Some("application/pdf")), Some("pdf"));
        assert_eq!(extension_for_mime(Some("application/octet-stream")), None);
    }

    #[test]
    fn builds_readable_fallback_names() {
        assert_eq!(
            upload_filename(None, Some("image/jpeg"), 1_778_106_301, 1),
            format!("image-{}-001.jpg", compact_timestamp(1_778_106_301))
        );
        assert_eq!(
            upload_filename(Some("blob"), Some("video/mp4"), 1_778_106_301, 1),
            format!("video-{}-001.mp4", compact_timestamp(1_778_106_301))
        );
        assert_eq!(
            upload_filename(Some("file"), None, 1_778_106_301, 1),
            format!("file-{}-001.bin", compact_timestamp(1_778_106_301))
        );
    }

    #[test]
    fn names_common_types_with_timestamp_and_sequence() {
        let timestamp = 1_778_106_301;

        assert_eq!(
            upload_filename(None, Some("image/jpeg"), timestamp, 1),
            format!("image-{}-001.jpg", compact_timestamp(timestamp))
        );
        assert_eq!(
            upload_filename(None, Some("video/mp4"), timestamp, 1),
            format!("video-{}-001.mp4", compact_timestamp(timestamp))
        );
        assert_eq!(
            upload_filename(None, Some("application/pdf"), timestamp, 1),
            format!("file-{}-001.pdf", compact_timestamp(timestamp))
        );
        assert_eq!(
            upload_filename(Some("clip"), Some("video/mp4"), timestamp, 1),
            format!("video-{}-001.mp4", compact_timestamp(timestamp))
        );
        assert_eq!(
            upload_filename(None, Some("application/octet-stream"), timestamp, 1),
            format!("file-{}-001.bin", compact_timestamp(timestamp))
        );
    }

    #[test]
    fn names_mobile_photo_edge_cases() {
        let timestamp = 1_778_106_301;

        assert_eq!(
            upload_filename(Some(""), Some("image/jpeg"), timestamp, 1),
            format!("image-{}-001.jpg", compact_timestamp(timestamp))
        );
        assert_eq!(
            upload_filename(Some("blob"), Some("image/png"), timestamp, 1),
            format!("image-{}-001.png", compact_timestamp(timestamp))
        );
        assert_eq!(
            upload_filename(Some("image"), Some("image/heic"), timestamp, 1),
            format!("image-{}-001.heic", compact_timestamp(timestamp))
        );
    }

    #[test]
    fn ignores_wechat_camera_original_name() {
        assert_eq!(
            upload_filename(
                Some("wx_camera_1778063309104.jpg"),
                Some("image/jpeg"),
                1_778_106_301,
                1
            ),
            format!("image-{}-001.jpg", compact_timestamp(1_778_106_301))
        );
    }

    #[test]
    fn keeps_original_extension_when_available() {
        assert_eq!(
            upload_filename(Some("../evil.mp4"), None, 1_778_106_301, 1),
            format!("video-{}-001.mp4", compact_timestamp(1_778_106_301))
        );
    }

    #[test]
    fn unique_path_appends_number_without_overwriting() {
        let temp = test_dir("unique");
        let first = temp.join("file.mp4");
        fs::write(&first, b"existing").expect("write");
        fs::write(temp.join("file (1).mp4"), b"existing").expect("write second");

        let next = unique_path(&temp, "file.mp4");

        assert_eq!(
            next.file_name().and_then(|value| value.to_str()),
            Some("file (2).mp4")
        );
        cleanup(&temp);
    }

    #[test]
    fn unique_upload_path_increments_timestamp_sequence() {
        let temp = test_dir("timestamp-sequence");
        let timestamp = 1_778_106_301;
        fs::write(
            temp.join(upload_filename(None, Some("image/jpeg"), timestamp, 1)),
            b"existing",
        )
        .expect("write first");
        fs::write(
            temp.join(upload_filename(None, Some("image/jpeg"), timestamp, 2)),
            b"existing",
        )
        .expect("write second");

        let next = unique_upload_path(&temp, None, Some("image/jpeg"), timestamp);

        assert_eq!(
            next.file_name().and_then(|value| value.to_str()),
            Some(upload_filename(None, Some("image/jpeg"), timestamp, 3).as_str())
        );
        cleanup(&temp);
    }

    #[test]
    fn unique_upload_path_increments_multiple_photos_same_second() {
        let temp = test_dir("photo-sequence");
        let timestamp = 1_778_106_301;
        let first_name = upload_filename(Some("blob"), Some("image/jpeg"), timestamp, 1);
        fs::write(temp.join(first_name), b"existing").expect("write first");

        let next = unique_upload_path(&temp, Some("blob"), Some("image/jpeg"), timestamp);

        assert_eq!(
            next.file_name().and_then(|value| value.to_str()),
            Some(upload_filename(Some("blob"), Some("image/jpeg"), timestamp, 2).as_str())
        );
        cleanup(&temp);
    }
}
