use std::sync::Arc;

use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, options, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    errors::AppError,
    storage::filename::{sanitize_filename, unique_path},
};

use super::{now_epoch_secs, upload::ensure_upload_size, AppState, ReceivedItem, ReceivedKind};

#[derive(Deserialize)]
struct AuthQuery {
    token: Option<String>,
}

#[derive(Deserialize)]
struct TextUpload {
    text: String,
}

#[derive(Serialize)]
struct SessionResponse {
    valid: bool,
    expires_at: u64,
    device_name: String,
    max_upload_bytes: u64,
    reason: Option<String>,
}

#[derive(Serialize)]
struct UploadResponse {
    ok: bool,
    items: Vec<ReceivedItem>,
}

pub fn router(state: Arc<AppState>) -> Router {
    let max_body = state.config().max_upload_bytes.saturating_add(1024 * 1024) as usize;

    Router::new()
        .route("/", get(upload_page))
        .route("/api/session", get(session_info))
        .route("/api/upload/text", post(upload_text).options(options_handler))
        .route("/api/upload/file", post(upload_file).options(options_handler))
        .route("/*path", options(options_handler))
        .layer(axum::extract::DefaultBodyLimit::max(max_body))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state)
}

async fn options_handler() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

async fn upload_page() -> Html<&'static str> {
    Html(MOBILE_UPLOAD_HTML)
}

async fn session_info(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuthQuery>,
) -> Result<Json<SessionResponse>, AppError> {
    let token = query.token.unwrap_or_default();
    let validation = state.validate_token(&token);
    let view = state.session_view()?;

    Ok(Json(SessionResponse {
        valid: validation.is_ok(),
        expires_at: view.expires_at,
        device_name: view.device_name,
        max_upload_bytes: view.max_upload_bytes,
        reason: validation.err().map(|error| error.to_string()),
    }))
}

async fn upload_text(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuthQuery>,
    Json(payload): Json<TextUpload>,
) -> Result<Json<UploadResponse>, AppError> {
    let token = query.token.as_deref().ok_or(AppError::TokenInvalid)?;
    state.validate_token(token)?;

    let text = payload.text.trim().to_string();
    if text.is_empty() {
        return Err(AppError::BadRequest("Text is empty.".to_string()));
    }

    ensure_upload_size(0, text.as_bytes().len(), state.config().max_upload_bytes)?;

    let item = ReceivedItem {
        id: format!("text-{}", now_epoch_secs()),
        kind: ReceivedKind::Text,
        name: "Text".to_string(),
        text: Some(text),
        path: None,
        size: None,
        mime: Some("text/plain".to_string()),
        received_at: now_epoch_secs(),
    };

    state.add_received(item.clone())?;

    Ok(Json(UploadResponse {
        ok: true,
        items: vec![item],
    }))
}

async fn upload_file(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuthQuery>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    let token = query.token.as_deref().ok_or(AppError::TokenInvalid)?;
    state.validate_token(token)?;

    let mut received_items = Vec::new();

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|error| AppError::Multipart(error.to_string()))?
    {
        let original_name = field.file_name().unwrap_or("upload.bin").to_string();
        let safe_name = sanitize_filename(&original_name);
        let content_type = field.content_type().map(|value| value.to_string());
        let target_path = unique_path(&state.config().receive_dir, &safe_name);
        let mut output = fs::File::create(&target_path).await?;
        let mut total_size = 0_u64;

        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|error| AppError::Multipart(error.to_string()))?
        {
            total_size = match ensure_upload_size(total_size, chunk.len(), state.config().max_upload_bytes) {
                Ok(size) => size,
                Err(error) => {
                    drop(output);
                    let _ = fs::remove_file(&target_path).await;
                    return Err(error);
                }
            };
            output.write_all(&chunk).await?;
        }

        output.flush().await?;

        let mime = content_type.or_else(|| {
            mime_guess::from_path(&target_path)
                .first()
                .map(|value| value.essence_str().to_string())
        });
        let kind = if mime.as_deref().is_some_and(|value| value.starts_with("image/")) {
            ReceivedKind::Image
        } else {
            ReceivedKind::File
        };
        let item = ReceivedItem {
            id: format!("file-{}-{}", now_epoch_secs(), received_items.len()),
            kind,
            name: target_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or(&safe_name)
                .to_string(),
            text: None,
            path: Some(target_path.display().to_string()),
            size: Some(total_size),
            mime,
            received_at: now_epoch_secs(),
        };

        state.add_received(item.clone())?;
        received_items.push(item);
    }

    if received_items.is_empty() {
        return Err(AppError::BadRequest("No file was uploaded.".to_string()));
    }

    Ok(Json(UploadResponse {
        ok: true,
        items: received_items,
    }))
}

const MOBILE_UPLOAD_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>DropLite</title>
    <style>
      :root { color: #18201c; background: #f5f7f4; font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
      * { box-sizing: border-box; }
      body { margin: 0; min-height: 100vh; padding: 22px; }
      main { margin: 0 auto; max-width: 520px; }
      header { margin: 10px 0 24px; }
      h1 { font-size: 2rem; line-height: 1.08; margin: 0 0 8px; }
      p { color: #53645a; margin: 0; }
      section { background: #fff; border: 1px solid #dfe7df; border-radius: 8px; margin-bottom: 14px; padding: 16px; }
      label { color: #253c32; display: block; font-weight: 700; margin-bottom: 8px; }
      textarea { border: 1px solid #cfdbd0; border-radius: 8px; font: inherit; min-height: 132px; padding: 12px; resize: vertical; width: 100%; }
      input[type=file] { width: 100%; }
      button { align-items: center; background: #17352b; border: 0; border-radius: 8px; color: #fff; display: inline-flex; font: inherit; font-weight: 700; justify-content: center; min-height: 44px; padding: 0 16px; width: 100%; }
      button:disabled { opacity: .55; }
      .drop-zone { border: 1px dashed #9eb0a4; border-radius: 8px; color: #53645a; padding: 22px; text-align: center; }
      .drop-zone.dragging { background: #edf4ed; border-color: #17352b; color: #17352b; }
      .status { border-radius: 8px; margin: 12px 0; padding: 12px; }
      .ok { background: #e9f6ed; color: #1f6b3d; }
      .error { background: #fff1ed; color: #8d2f24; }
      .muted { color: #64766b; font-size: .9rem; margin-top: 8px; }
    </style>
  </head>
  <body>
    <main>
      <header>
        <h1>Drop to this computer</h1>
        <p id="sessionText">Checking session...</p>
      </header>

      <div id="status" class="status" hidden></div>

      <section>
        <label for="text">Send text</label>
        <textarea id="text" placeholder="Paste or type text here"></textarea>
        <p class="muted">Paste text directly, then send it to the computer.</p>
        <button id="sendText" type="button">Send text</button>
      </section>

      <section>
        <label for="files">Send files</label>
        <div id="dropZone" class="drop-zone">Drop files here, or choose files below</div>
        <input id="files" type="file" multiple />
        <p class="muted">Images and files are saved on the receiving computer.</p>
        <button id="sendFiles" type="button">Choose files</button>
      </section>
    </main>

    <script>
      const params = new URLSearchParams(location.search);
      const token = params.get("token") || "";
      const statusBox = document.getElementById("status");
      const sessionText = document.getElementById("sessionText");
      const textInput = document.getElementById("text");
      const filesInput = document.getElementById("files");
      const sendText = document.getElementById("sendText");
      const sendFiles = document.getElementById("sendFiles");
      const dropZone = document.getElementById("dropZone");

      function setStatus(message, type) {
        statusBox.hidden = false;
        statusBox.className = "status " + type;
        statusBox.textContent = message;
      }

      function disableUploads(disabled) {
        textInput.disabled = disabled;
        filesInput.disabled = disabled;
        sendText.disabled = disabled;
        sendFiles.disabled = disabled;
      }

      async function checkSession() {
        if (!token) {
          sessionText.textContent = "Session expired. Please scan again.";
          disableUploads(true);
          return;
        }

        const response = await fetch("/api/session?token=" + encodeURIComponent(token));
        const data = await response.json();
        if (!data.valid) {
          sessionText.textContent = "Session expired. Please scan again.";
          setStatus(data.reason || "Session expired. Please scan again.", "error");
          disableUploads(true);
          return;
        }

        const expiresAt = new Date(data.expires_at * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
        sessionText.textContent = "Connected to " + data.device_name + ". Session expires at " + expiresAt + ".";
      }

      async function sendTextValue() {
        const text = textInput.value.trim();
        if (!text) {
          setStatus("Text is empty.", "error");
          return;
        }

        sendText.disabled = true;
        try {
          const response = await fetch("/api/upload/text?token=" + encodeURIComponent(token), {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ text })
          });
          const data = await response.json();
          if (!response.ok) throw new Error(data.error || "Send failed.");
          textInput.value = "";
          setStatus("Sent successfully", "ok");
        } catch (error) {
          setStatus(error.message || "Network unreachable.", "error");
        } finally {
          sendText.disabled = false;
        }
      }

      async function uploadFiles(fileList) {
        const files = Array.from(fileList || []);
        if (files.length === 0) {
          setStatus("Choose files first.", "error");
          return;
        }

        sendFiles.disabled = true;
        const form = new FormData();
        for (const file of files) form.append("files", file, file.name);

        try {
          const response = await fetch("/api/upload/file?token=" + encodeURIComponent(token), {
            method: "POST",
            body: form
          });
          const data = await response.json();
          if (!response.ok) throw new Error(data.error || "Upload failed.");
          filesInput.value = "";
          setStatus("Sent successfully", "ok");
        } catch (error) {
          setStatus(error.message || "Network unreachable.", "error");
        } finally {
          sendFiles.disabled = false;
        }
      }

      sendText.addEventListener("click", sendTextValue);
      sendFiles.addEventListener("click", () => filesInput.click());
      filesInput.addEventListener("change", () => uploadFiles(filesInput.files));
      document.addEventListener("paste", (event) => {
        const text = event.clipboardData && event.clipboardData.getData("text/plain");
        if (text && document.activeElement !== textInput) textInput.value = text;
      });

      ["dragenter", "dragover"].forEach((name) => dropZone.addEventListener(name, (event) => {
        event.preventDefault();
        dropZone.classList.add("dragging");
      }));
      ["dragleave", "drop"].forEach((name) => dropZone.addEventListener(name, (event) => {
        event.preventDefault();
        dropZone.classList.remove("dragging");
      }));
      dropZone.addEventListener("drop", (event) => uploadFiles(event.dataTransfer.files));

      checkSession().catch(() => {
        sessionText.textContent = "Network unreachable.";
        disableUploads(true);
      });
    </script>
  </body>
</html>"#;
