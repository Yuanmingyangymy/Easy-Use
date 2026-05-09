use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use super::{default_receive_dir, ensure_dir};

const CONFIG_FILE_NAME: &str = "config.json";
const WRITE_TEST_FILE: &str = ".droplite-write-test";

#[derive(Clone, Debug)]
pub struct ReceiveDirectoryConfig {
    config_path: PathBuf,
    default_dir: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
struct PersistedConfig {
    receive_dir: PathBuf,
}

impl ReceiveDirectoryConfig {
    pub fn load_default() -> io::Result<Self> {
        Ok(Self::new(default_config_path(), default_receive_dir()?))
    }

    pub fn new(config_path: PathBuf, default_dir: PathBuf) -> Self {
        Self {
            config_path,
            default_dir,
        }
    }

    pub fn default_dir(&self) -> &Path {
        &self.default_dir
    }

    pub fn get_receive_dir(&self) -> io::Result<PathBuf> {
        let Some(config) = self.read_config()? else {
            return self.ensure_default_dir();
        };

        match validate_receive_dir(&config.receive_dir) {
            Ok(path) => Ok(path),
            Err(_) => self.ensure_default_dir(),
        }
    }

    pub fn set_receive_dir(&self, path: PathBuf) -> io::Result<PathBuf> {
        let path = validate_receive_dir(&path)?;
        self.write_config(&PersistedConfig {
            receive_dir: PathBuf::from(display_path(&path)),
        })?;
        Ok(path)
    }

    pub fn reset_receive_dir(&self) -> io::Result<PathBuf> {
        let path = self.ensure_default_dir()?;
        self.write_config(&PersistedConfig {
            receive_dir: PathBuf::from(display_path(&path)),
        })?;
        Ok(path)
    }

    fn ensure_default_dir(&self) -> io::Result<PathBuf> {
        validate_receive_dir(&self.default_dir)
    }

    fn read_config(&self) -> io::Result<Option<PersistedConfig>> {
        match fs::read_to_string(&self.config_path) {
            Ok(contents) => Ok(serde_json::from_str(&contents).ok()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(_) => Ok(None),
        }
    }

    fn write_config(&self, config: &PersistedConfig) -> io::Result<()> {
        if let Some(parent) = self.config_path.parent() {
            ensure_dir(parent)?;
        }

        let contents = serde_json::to_string_pretty(config)
            .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
        fs::write(&self.config_path, contents)
    }
}

pub fn display_path(path: &Path) -> String {
    normalize_windows_verbatim_prefix(&path.display().to_string())
}

pub fn default_config_path() -> PathBuf {
    dirs_next::config_dir()
        .or_else(dirs_next::data_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Easy-Use")
        .join("DropLite")
        .join(CONFIG_FILE_NAME)
}

fn normalize_windows_verbatim_prefix(path: &str) -> String {
    const UNC_PREFIX: &str = r"\\?\UNC\";
    const DRIVE_PREFIX: &str = r"\\?\";

    if let Some(rest) = path.strip_prefix(UNC_PREFIX) {
        return format!(r"\\{rest}");
    }

    if let Some(rest) = path.strip_prefix(DRIVE_PREFIX) {
        return rest.to_string();
    }

    path.to_string()
}

fn validate_receive_dir(path: &Path) -> io::Result<PathBuf> {
    if path.exists() && !path.is_dir() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "The selected path is not a folder.",
        ));
    }

    ensure_dir(path)?;
    let canonical = fs::canonicalize(path)?;
    ensure_writable(&canonical)?;
    Ok(canonical)
}

fn ensure_writable(path: &Path) -> io::Result<()> {
    let probe = path.join(WRITE_TEST_FILE);
    fs::write(&probe, b"ok")?;
    let _ = fs::remove_file(probe);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_root(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("droplite-receive-config-{name}-{nonce}"))
    }

    fn test_store(name: &str) -> (ReceiveDirectoryConfig, PathBuf) {
        let root = test_root(name);
        let default_dir = root.join("Downloads").join("DropLite");
        let config_path = root.join("config").join(CONFIG_FILE_NAME);
        (
            ReceiveDirectoryConfig::new(config_path, default_dir.clone()),
            root,
        )
    }

    #[test]
    fn default_receive_dir_is_created() {
        let (store, root) = test_store("default");

        let dir = store.get_receive_dir().expect("default dir");

        assert_eq!(
            dir,
            fs::canonicalize(root.join("Downloads").join("DropLite")).unwrap()
        );
        assert!(dir.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn set_receive_dir_saves_and_loads_config() {
        let (store, root) = test_store("set");
        let custom = root.join("Custom");

        let saved = store.set_receive_dir(custom.clone()).expect("set dir");
        let loaded = store.get_receive_dir().expect("load dir");

        assert_eq!(saved, loaded);
        assert_eq!(loaded, fs::canonicalize(custom).unwrap());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn config_survives_store_recreation() {
        let (store, root) = test_store("restart");
        let custom = root.join("Custom");
        let saved = store.set_receive_dir(custom).expect("set dir");
        let recreated = ReceiveDirectoryConfig::new(
            store.config_path.clone(),
            root.join("Downloads").join("DropLite"),
        );

        assert_eq!(recreated.get_receive_dir().expect("load dir"), saved);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reset_receive_dir_restores_default() {
        let (store, root) = test_store("reset");
        store
            .set_receive_dir(root.join("Custom"))
            .expect("custom dir");

        let reset = store.reset_receive_dir().expect("reset");

        assert_eq!(
            reset,
            fs::canonicalize(root.join("Downloads").join("DropLite")).unwrap()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn damaged_config_falls_back_to_default() {
        let (store, root) = test_store("damaged");
        fs::create_dir_all(store.config_path.parent().unwrap()).expect("config parent");
        fs::write(&store.config_path, "{bad json").expect("bad config");

        let dir = store.get_receive_dir().expect("fallback");

        assert_eq!(
            dir,
            fs::canonicalize(root.join("Downloads").join("DropLite")).unwrap()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_receive_dir_is_created() {
        let (store, root) = test_store("missing");
        let missing = root.join("Missing").join("DropLite");

        let dir = store.set_receive_dir(missing.clone()).expect("set missing");

        assert_eq!(dir, fs::canonicalize(missing).unwrap());
        assert!(dir.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn file_path_is_rejected() {
        let (store, root) = test_store("file");
        fs::create_dir_all(&root).expect("root");
        let file = root.join("not-a-folder.txt");
        fs::write(&file, b"not dir").expect("file");

        let result = store.set_receive_dir(file);

        assert_eq!(
            result.expect_err("file rejected").kind(),
            ErrorKind::InvalidInput
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn display_path_removes_windows_extended_drive_prefix() {
        assert_eq!(
            normalize_windows_verbatim_prefix(r"\\?\C:\Users\ymy\Desktop"),
            r"C:\Users\ymy\Desktop"
        );
        assert_eq!(
            normalize_windows_verbatim_prefix(r"\\?\C:\Users\ymy\Downloads\DropLite"),
            r"C:\Users\ymy\Downloads\DropLite"
        );
    }

    #[test]
    fn display_path_removes_windows_extended_unc_prefix() {
        assert_eq!(
            normalize_windows_verbatim_prefix(r"\\?\UNC\server\share\folder"),
            r"\\server\share\folder"
        );
    }

    #[test]
    fn display_path_keeps_normal_path_unchanged() {
        assert_eq!(
            normalize_windows_verbatim_prefix(r"C:\Users\ymy\Desktop"),
            r"C:\Users\ymy\Desktop"
        );
    }

    #[test]
    fn config_file_saves_display_friendly_path() {
        let (store, root) = test_store("display-save");
        let custom = root.join("Custom");

        let saved = store.set_receive_dir(custom).expect("set dir");
        let contents = fs::read_to_string(&store.config_path).expect("config");

        assert!(!contents.contains(r"\\?\"));
        assert!(contents.contains(&display_path(&saved).replace('\\', "\\\\")));
        let _ = fs::remove_dir_all(root);
    }
}
