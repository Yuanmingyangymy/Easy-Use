pub mod outbox;
pub mod routes;
pub mod session;
pub mod upload;

use crate::{
    errors::AppError, network::local_ip::detect_local_ip, storage::config::ReceiveDirectoryConfig,
};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, RwLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};
#[cfg(not(test))]
use tauri::{AppHandle, Emitter};
use tokio::net::TcpListener;

#[cfg(test)]
type AppHandle = ();

use self::outbox::{OutboxEntry, OutboxItem, OutboxState};
use self::session::Session;

pub const DEFAULT_SESSION_TTL_SECS: u64 = 10 * 60;
pub const DEFAULT_MAX_UPLOAD_BYTES: u64 = 200 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub device_name: String,
    pub local_ip: String,
    pub max_upload_bytes: u64,
    pub default_receive_dir: PathBuf,
    pub receive_dir: PathBuf,
    pub receive_config: ReceiveDirectoryConfig,
    pub ttl_secs: u64,
}

#[derive(Clone, Serialize)]
pub struct DesktopState {
    pub session: SessionView,
    pub received: Vec<ReceivedItem>,
}

#[derive(Clone, Serialize)]
pub struct SessionView {
    pub connection_url: String,
    pub device_name: String,
    pub expires_at: u64,
    pub is_ready: bool,
    pub local_ip: String,
    pub max_upload_bytes: u64,
    pub port: u16,
    pub receive_dir: String,
    pub security_note: String,
    pub started_at: u64,
}

#[derive(Clone, Serialize)]
pub struct ReceivedItem {
    pub id: String,
    pub kind: ReceivedKind,
    pub name: String,
    pub text: Option<String>,
    pub path: Option<String>,
    pub preview_url: Option<String>,
    pub size: Option<u64>,
    pub mime: Option<String>,
    pub received_at: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReceivedKind {
    File,
    Image,
    Text,
    Video,
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, AppState, ReceivedKind, DEFAULT_MAX_UPLOAD_BYTES};
    use crate::storage::config::ReceiveDirectoryConfig;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn test_root(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("droplite-app-state-{name}-{nonce}"))
    }

    fn test_state(name: &str) -> (AppState, PathBuf) {
        let root = test_root(name);
        let default_receive_dir = root.join("Downloads").join("DropLite");
        let receive_config =
            ReceiveDirectoryConfig::new(root.join("config.json"), default_receive_dir.clone());
        let state = AppState::new(AppConfig {
            device_name: "Test computer".to_string(),
            local_ip: "127.0.0.1".to_string(),
            max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
            default_receive_dir: default_receive_dir.clone(),
            receive_dir: default_receive_dir,
            receive_config,
            ttl_secs: 600,
        })
        .expect("state");

        (state, root)
    }

    #[test]
    fn received_kind_serializes_for_frontend() {
        assert_eq!(
            serde_json::to_string(&ReceivedKind::Text).expect("text"),
            "\"text\""
        );
        assert_eq!(
            serde_json::to_string(&ReceivedKind::Image).expect("image"),
            "\"image\""
        );
        assert_eq!(
            serde_json::to_string(&ReceivedKind::Video).expect("video"),
            "\"video\""
        );
        assert_eq!(
            serde_json::to_string(&ReceivedKind::File).expect("file"),
            "\"file\""
        );
    }

    #[test]
    fn receive_dir_can_change_and_survives_session_refresh() {
        let (state, root) = test_state("change");
        let custom = root.join("Custom");

        let updated = state.set_receive_dir(custom.clone()).expect("set dir");
        state.refresh_session().expect("refresh session");

        assert_eq!(state.receive_dir().expect("receive dir"), updated);
        assert_eq!(
            state.session_view().expect("session").receive_dir,
            updated.display().to_string()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn receive_dir_reset_restores_default() {
        let (state, root) = test_state("reset");
        state
            .set_receive_dir(root.join("Custom"))
            .expect("set custom");

        let reset = state.reset_receive_dir().expect("reset");

        assert_eq!(
            reset,
            fs::canonicalize(root.join("Downloads").join("DropLite")).unwrap()
        );
        assert_eq!(state.receive_dir().expect("receive dir"), reset);
        let _ = fs::remove_dir_all(root);
    }
}

pub struct AppState {
    config: AppConfig,
    receive_dir: RwLock<PathBuf>,
    session: RwLock<Session>,
    received: RwLock<Vec<ReceivedItem>>,
    outbox: RwLock<OutboxState>,
    #[cfg_attr(test, allow(dead_code))]
    app_handle: Mutex<Option<AppHandle>>,
    id_counter: AtomicU64,
}

impl AppConfig {
    pub fn load() -> Result<Self, AppError> {
        let local_ip = detect_local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let device_name = hostname::get()
            .ok()
            .and_then(|name| name.into_string().ok())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "This computer".to_string());
        let receive_config = ReceiveDirectoryConfig::load_default()?;
        let receive_dir = receive_config.get_receive_dir()?;
        let default_receive_dir = receive_config.default_dir().to_path_buf();

        Ok(Self {
            device_name,
            local_ip,
            max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
            default_receive_dir,
            receive_dir,
            receive_config,
            ttl_secs: DEFAULT_SESSION_TTL_SECS,
        })
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Result<Self, AppError> {
        Ok(Self {
            receive_dir: RwLock::new(config.receive_dir.clone()),
            session: RwLock::new(Session::new(config.ttl_secs, 0)),
            received: RwLock::new(Vec::new()),
            outbox: RwLock::new(OutboxState::default()),
            app_handle: Mutex::new(None),
            id_counter: AtomicU64::new(1),
            config,
        })
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn receive_dir(&self) -> Result<PathBuf, AppError> {
        self.receive_dir
            .read()
            .map_err(|_| AppError::LockFailed("receive_dir"))
            .map(|path| path.clone())
    }

    pub fn set_receive_dir(&self, path: PathBuf) -> Result<PathBuf, AppError> {
        let path = self.config.receive_config.set_receive_dir(path)?;
        let mut receive_dir = self
            .receive_dir
            .write()
            .map_err(|_| AppError::LockFailed("receive_dir"))?;
        *receive_dir = path.clone();
        Ok(path)
    }

    pub fn reset_receive_dir(&self) -> Result<PathBuf, AppError> {
        let path = self.config.receive_config.reset_receive_dir()?;
        let mut receive_dir = self
            .receive_dir
            .write()
            .map_err(|_| AppError::LockFailed("receive_dir"))?;
        *receive_dir = path.clone();
        Ok(path)
    }

    #[cfg(not(test))]
    pub fn set_app_handle(&self, handle: AppHandle) {
        if let Ok(mut slot) = self.app_handle.lock() {
            *slot = Some(handle);
        }
    }

    pub fn set_port(&self, port: u16) -> Result<(), AppError> {
        let mut session = self
            .session
            .write()
            .map_err(|_| AppError::LockFailed("session"))?;
        session.port = port;
        Ok(())
    }

    pub fn refresh_session(&self) -> Result<(), AppError> {
        let current_port = self.session_snapshot()?.port;
        let mut session = self
            .session
            .write()
            .map_err(|_| AppError::LockFailed("session"))?;
        *session = Session::new(self.config.ttl_secs, current_port);
        self.clear_outbox()?;
        Ok(())
    }

    pub fn validate_token(&self, token: &str) -> Result<(), AppError> {
        self.session_snapshot()?.validate(token)
    }

    pub fn add_received(&self, item: ReceivedItem) -> Result<(), AppError> {
        {
            let mut received = self
                .received
                .write()
                .map_err(|_| AppError::LockFailed("received"))?;
            received.insert(0, item.clone());
            received.truncate(100);
        }

        #[cfg(not(test))]
        {
            if let Ok(slot) = self.app_handle.lock() {
                if let Some(handle) = slot.as_ref() {
                    let _ = handle.emit("droplite://received", item);
                }
            }
        }

        Ok(())
    }

    pub fn received_item(&self, id: &str) -> Result<Option<ReceivedItem>, AppError> {
        let received = self
            .received
            .read()
            .map_err(|_| AppError::LockFailed("received"))?;
        Ok(received.iter().find(|item| item.id == id).cloned())
    }

    pub fn next_received_id(&self, prefix: &str) -> String {
        let counter = self.id_counter.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}-{}-{counter}", now_epoch_secs())
    }

    pub fn preview_url(&self, id: &str) -> Result<String, AppError> {
        let session = self.session_snapshot()?;
        Ok(format!(
            "http://127.0.0.1:{}/api/received/{}/preview?token={}",
            session.port, id, session.token
        ))
    }

    pub fn add_outbox_text(&self, content: String) -> Result<OutboxItem, AppError> {
        let mut outbox = self
            .outbox
            .write()
            .map_err(|_| AppError::LockFailed("outbox"))?;
        outbox.add_text(content)
    }

    pub fn add_outbox_file(&self, path: PathBuf) -> Result<OutboxItem, AppError> {
        let mut outbox = self
            .outbox
            .write()
            .map_err(|_| AppError::LockFailed("outbox"))?;
        outbox.add_file(path)
    }

    pub fn outbox_items(&self) -> Result<Vec<OutboxItem>, AppError> {
        let outbox = self
            .outbox
            .read()
            .map_err(|_| AppError::LockFailed("outbox"))?;
        Ok(outbox.list())
    }

    pub fn outbox_entry(&self, id: &str) -> Result<Option<OutboxEntry>, AppError> {
        let outbox = self
            .outbox
            .read()
            .map_err(|_| AppError::LockFailed("outbox"))?;
        Ok(outbox.get(id))
    }

    pub fn acknowledge_outbox_item(&self, id: &str) -> Result<OutboxItem, AppError> {
        let mut outbox = self
            .outbox
            .write()
            .map_err(|_| AppError::LockFailed("outbox"))?;
        outbox.acknowledge(id)
    }

    pub fn clear_outbox(&self) -> Result<(), AppError> {
        let mut outbox = self
            .outbox
            .write()
            .map_err(|_| AppError::LockFailed("outbox"))?;
        outbox.clear();
        Ok(())
    }

    pub fn desktop_state(&self) -> Result<DesktopState, AppError> {
        let mut received = self
            .received
            .read()
            .map_err(|_| AppError::LockFailed("received"))?
            .clone();
        self.attach_preview_urls(&mut received)?;

        Ok(DesktopState {
            session: self.session_view()?,
            received,
        })
    }

    pub fn session_view(&self) -> Result<SessionView, AppError> {
        let session = self.session_snapshot()?;
        let is_ready = session.port != 0;
        let connection_url = format!(
            "http://{}:{}/?token={}",
            self.config.local_ip, session.port, session.token
        );

        Ok(SessionView {
            connection_url,
            device_name: self.config.device_name.clone(),
            expires_at: session.expires_at,
            is_ready,
            local_ip: self.config.local_ip.clone(),
            max_upload_bytes: self.config.max_upload_bytes,
            port: session.port,
            receive_dir: self.receive_dir()?.display().to_string(),
            security_note: if self.config.local_ip == "127.0.0.1" {
                "No LAN IP was detected. Phone access may not work until a local network is available.".to_string()
            } else {
                "No account. No cloud. No history. Local HTTP is visible to devices on this network.".to_string()
            },
            started_at: session.created_at,
        })
    }

    pub(crate) fn session_snapshot(&self) -> Result<Session, AppError> {
        self.session
            .read()
            .map_err(|_| AppError::LockFailed("session"))
            .map(|session| session.clone())
    }

    #[cfg(test)]
    pub(crate) fn set_session_for_test(&self, session: Session) -> Result<(), AppError> {
        let mut current = self
            .session
            .write()
            .map_err(|_| AppError::LockFailed("session"))?;
        *current = session;
        Ok(())
    }

    fn attach_preview_urls(&self, items: &mut [ReceivedItem]) -> Result<(), AppError> {
        for item in items {
            item.preview_url = if matches!(item.kind, ReceivedKind::Image) {
                Some(self.preview_url(&item.id)?)
            } else {
                None
            };
        }

        Ok(())
    }
}

pub async fn start(state: Arc<AppState>) -> Result<(), AppError> {
    let listener = TcpListener::bind(("0.0.0.0", 0)).await?;
    let port = listener.local_addr()?.port();
    state.set_port(port)?;
    debug_log(&format!("local server listening on port {port}"));

    let router = routes::router(Arc::clone(&state));
    axum::serve(listener, router).await?;
    Ok(())
}

#[cfg(debug_assertions)]
pub fn debug_log(message: &str) {
    eprintln!("[droplite] {message}");
}

#[cfg(not(debug_assertions))]
pub fn debug_log(_message: &str) {}

pub fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
