use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const FALLBACK_NAME: &str = "file";
const MAX_FILENAME_CHARS: usize = 120;

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
) -> String {
    let extension = extension_for_mime(mime)
        .or_else(|| original_name.and_then(extension_from_name))
        .unwrap_or("bin");

    if let Some(original_name) = original_name {
        let sanitized = sanitize_filename(original_name);
        if should_keep_original_name(&sanitized) {
            return ensure_extension(&sanitized, extension);
        }
    }

    let prefix = fallback_prefix(mime);
    format!("{prefix}-{}.{}", compact_timestamp(received_at), extension)
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

fn should_keep_original_name(filename: &str) -> bool {
    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .trim();
    let has_extension = path.extension().is_some();

    if !has_extension || stem.is_empty() || filename == FALLBACK_NAME {
        return false;
    }

    if stem.len() > 80 {
        return false;
    }

    !looks_like_hash(stem)
}

fn looks_like_hash(value: &str) -> bool {
    value.len() >= 24 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn ensure_extension(filename: &str, extension: &str) -> String {
    if Path::new(filename).extension().is_some() {
        return truncate_filename(filename, MAX_FILENAME_CHARS);
    }

    truncate_filename(&format!("{filename}.{extension}"), MAX_FILENAME_CHARS)
}

fn extension_from_name(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
}

fn fallback_prefix(mime: Option<&str>) -> &'static str {
    let mime = mime.unwrap_or("").to_ascii_lowercase();
    if mime.starts_with("image/") {
        "image"
    } else if mime.starts_with("video/") {
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
    let (year, month, day, hour, minute, second) = utc_parts(timestamp);
    format!("{year:04}{month:02}{day:02}-{hour:02}{minute:02}{second:02}")
}

fn utc_parts(timestamp: u64) -> (i32, u32, u32, u32, u32, u32) {
    let days = (timestamp / 86_400) as i64;
    let seconds_of_day = timestamp % 86_400;
    let (year, month, day) = civil_from_days(days);

    (
        year,
        month,
        day,
        (seconds_of_day / 3_600) as u32,
        ((seconds_of_day % 3_600) / 60) as u32,
        (seconds_of_day % 60) as u32,
    )
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let days = days_since_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };

    (year as i32, month as u32, day as u32)
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
            upload_filename(None, Some("image/jpeg"), 1_778_106_301),
            "image-20260506-222501.jpg"
        );
        assert_eq!(
            upload_filename(Some("blob"), Some("video/mp4"), 1_778_106_301),
            "video-20260506-222501.mp4"
        );
        assert_eq!(
            upload_filename(Some("file"), None, 1_778_106_301),
            "file-20260506-222501.bin"
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
}
