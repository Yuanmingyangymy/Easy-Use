pub mod routes;
pub mod session;
pub mod upload;

use crate::{errors::AppError, network::local_ip::detect_local_ip, storage};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, RwLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};
use tokio::net::TcpListener;

use self::session::Session;

pub const DEFAULT_SESSION_TTL_SECS: u64 = 10 * 60;
pub const DEFAULT_MAX_UPLOAD_BYTES: u64 = 200 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub device_name: String,
    pub local_ip: String,
    pub max_upload_bytes: u64,
    pub receive_dir: PathBuf,
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
    use super::ReceivedKind;

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
}

pub struct AppState {
    config: AppConfig,
    session: RwLock<Session>,
    received: RwLock<Vec<ReceivedItem>>,
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

        Ok(Self {
            device_name,
            local_ip,
            max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
            receive_dir: storage::default_receive_dir()?,
            ttl_secs: DEFAULT_SESSION_TTL_SECS,
        })
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Result<Self, AppError> {
        Ok(Self {
            session: RwLock::new(Session::new(config.ttl_secs, 0)),
            received: RwLock::new(Vec::new()),
            app_handle: Mutex::new(None),
            id_counter: AtomicU64::new(1),
            config,
        })
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

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

        if let Ok(slot) = self.app_handle.lock() {
            if let Some(handle) = slot.as_ref() {
                let _ = handle.emit("droplite://received", item);
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
            receive_dir: self.config.receive_dir.display().to_string(),
            security_note: if self.config.local_ip == "127.0.0.1" {
                "No LAN IP was detected. Phone access may not work until a local network is available.".to_string()
            } else {
                "No account. No cloud. No history. Local HTTP is visible to devices on this network.".to_string()
            },
            started_at: session.created_at,
        })
    }

    fn session_snapshot(&self) -> Result<Session, AppError> {
        self.session
            .read()
            .map_err(|_| AppError::LockFailed("session"))
            .map(|session| session.clone())
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
