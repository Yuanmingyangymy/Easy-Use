use std::path::{Path, PathBuf};

const FALLBACK_NAME: &str = "upload";

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
        sanitized.chars().take(180).collect()
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

fn chrono_like_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sanitize_removes_path_traversal() {
        assert_eq!(sanitize_filename("../../secret.txt"), "secret.txt");
        assert_eq!(sanitize_filename("..\\secret.txt"), "secret.txt");
        assert_eq!(sanitize_filename("a:b*c?.png"), "a_b_c_.png");
        assert_eq!(sanitize_filename("..."), FALLBACK_NAME);
    }

    #[test]
    fn unique_path_appends_number_without_overwriting() {
        let temp = tempfile::tempdir().expect("temp dir");
        let first = temp.path().join("photo.png");
        fs::write(&first, b"existing").expect("write");

        let next = unique_path(temp.path(), "photo.png");

        assert_eq!(next.file_name().and_then(|value| value.to_str()), Some("photo (1).png"));
    }
}
