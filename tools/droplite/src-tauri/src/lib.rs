pub mod errors;
pub mod network;
pub mod security;
pub mod server;
pub mod storage;

#[cfg(not(test))]
use tauri::Manager;

#[cfg(not(test))]
use std::sync::Arc;

#[cfg(not(test))]
use server::{AppConfig, AppState, DesktopState};

#[cfg(not(test))]
#[tauri::command]
async fn get_desktop_state(state: tauri::State<'_, Arc<AppState>>) -> Result<DesktopState, String> {
    state.desktop_state().map_err(|error| error.to_string())
}

#[cfg(not(test))]
#[tauri::command]
async fn refresh_session(state: tauri::State<'_, Arc<AppState>>) -> Result<DesktopState, String> {
    state.refresh_session().map_err(|error| error.to_string())?;
    state.desktop_state().map_err(|error| error.to_string())
}

#[cfg(not(test))]
#[tauri::command]
async fn open_receive_folder(state: tauri::State<'_, Arc<AppState>>) -> Result<(), String> {
    let path = state.config().receive_dir.clone();
    open::that(path).map_err(|error| format!("Could not open receive folder: {error}"))
}

#[cfg(not(test))]
pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            let config = AppConfig::load()?;
            let state = Arc::new(AppState::new(config)?);
            state.set_app_handle(app.handle().clone());

            let server_state = Arc::clone(&state);
            tauri::async_runtime::spawn(async move {
                if let Err(error) = server::start(server_state).await {
                    eprintln!("DropLite local server stopped: {error}");
                }
            });

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_desktop_state,
            refresh_session,
            open_receive_folder
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("failed to run DropLite: {error}");
        std::process::exit(1);
    }
}
