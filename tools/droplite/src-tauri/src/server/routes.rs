use std::sync::Arc;

use axum::{
    extract::{Multipart, Path as AxumPath, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, options, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    errors::AppError,
    storage::filename::{unique_temp_path, unique_upload_path},
};

use super::{
    debug_log, now_epoch_secs,
    upload::{
        ensure_receive_dir, ensure_upload_size, rename_complete_upload, LimitedFileWriter,
        TempFileGuard,
    },
    AppState, ReceivedItem, ReceivedKind,
};

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
        .route("/api/received/:id/preview", get(received_preview))
        .route(
            "/api/upload/text",
            post(upload_text).options(options_handler),
        )
        .route(
            "/api/upload/file",
            post(upload_file).options(options_handler),
        )
        .route("/*path", options(options_handler))
        .layer(axum::extract::DefaultBodyLimit::max(max_body))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
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
        id: state.next_received_id("text"),
        kind: ReceivedKind::Text,
        name: "Text".to_string(),
        text: Some(text),
        path: None,
        preview_url: None,
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
    debug_log("upload request received");
    let token = query.token.as_deref().ok_or(AppError::TokenInvalid)?;
    state.validate_token(token)?;
    debug_log("upload token validated");

    let mut received_items = Vec::new();

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|error| AppError::Multipart(error.to_string()))?
    {
        let item = save_upload_field(&state, &mut field).await?;
        state.add_received(item.clone())?;
        debug_log("received item emitted");
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

async fn save_upload_field(
    state: &Arc<AppState>,
    field: &mut axum::extract::multipart::Field<'_>,
) -> Result<ReceivedItem, AppError> {
    ensure_receive_dir(&state.config().receive_dir).await?;
    let original_name = field.file_name().map(|value| value.to_string());
    let content_type = field.content_type().map(|value| value.to_string());
    debug_log(&format!(
        "multipart field received: filename={}, content_type={}",
        original_name.as_deref().unwrap_or("<missing>"),
        content_type.as_deref().unwrap_or("<missing>")
    ));
    let received_at = now_epoch_secs();
    let target_path = unique_upload_path(
        &state.config().receive_dir,
        original_name.as_deref(),
        content_type.as_deref(),
        received_at,
    );
    let display_name = target_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("file.bin")
        .to_string();
    let temp_path = unique_temp_path(&state.config().receive_dir);
    let mut temp_guard = TempFileGuard::new(temp_path);
    let mut output = LimitedFileWriter::create(temp_guard.path().to_path_buf()).await?;
    debug_log(&format!(
        "temp file created: {}",
        temp_guard
            .path()
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("<unknown>")
    ));
    let mut total_size = 0_u64;

    loop {
        let chunk = match field.chunk().await {
            Ok(Some(chunk)) => chunk,
            Ok(None) => break,
            Err(error) => {
                output.abort().await;
                return Err(AppError::Multipart(format!(
                    "Upload was interrupted before the file was fully received: {error}"
                )));
            }
        };

        if let Err(error) = output
            .write_chunk(&chunk, state.config().max_upload_bytes)
            .await
        {
            output.abort().await;
            return Err(error);
        }
        total_size = total_size.saturating_add(chunk.len() as u64);
    }

    let written_size = output.finish().await?;
    debug_log(&format!("upload bytes written: {written_size}"));
    if written_size != total_size {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            "Saved file size did not match uploaded bytes.",
        )));
    }

    debug_log(&format!("final filename resolved: {display_name}"));
    rename_complete_upload(temp_guard.path(), &target_path).await?;
    temp_guard.keep();
    debug_log("temp file renamed to final file");

    let saved_size = fs::metadata(&target_path).await?.len();
    if saved_size != written_size {
        let _ = fs::remove_file(&target_path).await;
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            "Saved file size did not match uploaded bytes.",
        )));
    }

    let mime = content_type.or_else(|| {
        mime_guess::from_path(&target_path)
            .first()
            .map(|value| value.essence_str().to_string())
    });
    let kind = received_kind_for(mime.as_deref(), &target_path);
    let id = state.next_received_id("file");
    let preview_url = if matches!(kind, ReceivedKind::Image) {
        Some(state.preview_url(&id)?)
    } else {
        None
    };

    let item = ReceivedItem {
        id,
        kind,
        name: target_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(&display_name)
            .to_string(),
        text: None,
        path: Some(target_path.display().to_string()),
        preview_url,
        size: Some(saved_size),
        mime,
        received_at,
    };
    debug_log("received item ready to emit");
    Ok(item)
}

fn received_kind_for(mime: Option<&str>, path: &std::path::Path) -> ReceivedKind {
    if mime.is_some_and(|value| value.starts_with("image/")) {
        return ReceivedKind::Image;
    }
    if mime.is_some_and(|value| value.starts_with("video/")) {
        return ReceivedKind::Video;
    }

    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" | "gif") => ReceivedKind::Image,
        Some("mp4" | "mov" | "webm") => ReceivedKind::Video,
        _ => ReceivedKind::File,
    }
}

async fn received_preview(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
    Query(query): Query<AuthQuery>,
) -> Result<impl IntoResponse, AppError> {
    let token = query.token.as_deref().ok_or(AppError::TokenInvalid)?;
    state.validate_token(token)?;

    let item = state
        .received_item(&id)?
        .ok_or_else(|| AppError::NotFound("Received file was not found.".to_string()))?;

    if !matches!(item.kind, ReceivedKind::Image) {
        return Err(AppError::NotFound(
            "Preview is only available for received images.".to_string(),
        ));
    }

    let path = item
        .path
        .as_deref()
        .ok_or_else(|| AppError::NotFound("Received file path was not found.".to_string()))?;
    let path = std::path::PathBuf::from(path);
    let receive_dir = fs::canonicalize(&state.config().receive_dir).await?;
    let file_path = fs::canonicalize(&path).await?;

    if !file_path.starts_with(&receive_dir) {
        return Err(AppError::NotFound(
            "Preview is outside the receive directory.".to_string(),
        ));
    }

    let bytes = fs::read(&file_path).await?;
    let content_type = item
        .mime
        .as_deref()
        .map(str::to_string)
        .or_else(|| {
            mime_guess::from_path(&file_path)
                .first()
                .map(|value| value.essence_str().to_string())
        })
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));

    Ok((headers, bytes))
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
      button { align-items: center; background: #17352b; border: 0; border-radius: 8px; color: #fff; display: inline-flex; font: inherit; font-weight: 700; justify-content: center; min-height: 44px; padding: 0 16px; width: 100%; }
      button:disabled { opacity: .55; }
      .topline { align-items: center; display: flex; justify-content: space-between; gap: 12px; }
      select { background: #fff; border: 1px solid #cfdbd0; border-radius: 8px; color: #17352b; font: inherit; min-height: 36px; padding: 0 10px; }
      .choice-grid { display: grid; gap: 10px; grid-template-columns: repeat(2, minmax(0, 1fr)); }
      .choice-button { background: #eef4ee; color: #17352b; min-height: 58px; }
      input[type=file] { height: 1px; opacity: 0; position: absolute; width: 1px; }
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
        <div class="topline">
          <h1 data-i18n="title">Drop to this computer</h1>
          <select id="languageSelect" aria-label="Language">
            <option value="en">English</option>
            <option value="zh-CN">简体中文</option>
          </select>
        </div>
        <p id="sessionText" data-i18n="checkingSession">Checking session...</p>
      </header>

      <div id="status" class="status" hidden></div>

      <section>
        <label for="text" data-i18n="sendText">Send text</label>
        <textarea id="text" data-i18n-placeholder="textPlaceholder" placeholder="Paste or type text here"></textarea>
        <p class="muted" data-i18n="textHelp">Paste text directly, then send it to the computer.</p>
        <button id="sendText" type="button" data-i18n="sendText">Send text</button>
      </section>

      <section>
        <label data-i18n="chooseWhat">Choose what to send</label>
        <div class="choice-grid">
          <button id="sendPhoto" class="choice-button" type="button" data-i18n="sendPhoto">Send photo</button>
          <button id="takePhoto" class="choice-button" type="button" data-i18n="takePhoto">Take photo</button>
          <button id="sendVideo" class="choice-button" type="button" data-i18n="sendVideo">Send video</button>
          <button id="sendFile" class="choice-button" type="button" data-i18n="sendFile">Send file</button>
        </div>
        <div id="dropZone" class="drop-zone" data-i18n="dropZone">Drop files here, or choose files above</div>
        <input id="photoInput" type="file" accept="image/*" multiple />
        <input id="takePhotoInput" type="file" accept="image/*" capture="environment" />
        <input id="videoInput" type="file" accept="video/*" multiple />
        <input id="fileInput" type="file" multiple />
        <p class="muted" data-i18n="localOnly">Local network only. No cloud upload.</p>
      </section>
    </main>

    <script>
      const params = new URLSearchParams(location.search);
      const token = params.get("token") || "";
      const dictionaries = {
        en: {
          title: "Drop to this computer",
          checkingSession: "Checking session...",
          sessionExpired: "Session expired. Please scan again.",
          networkUnreachable: "Network unreachable.",
          connected: "Connected to {device}. Session expires at {time}.",
          sendText: "Send text",
          textPlaceholder: "Paste or type text here",
          textHelp: "Paste text directly, then send it to the computer.",
          chooseWhat: "Choose what to send",
          sendPhoto: "Send photo",
          takePhoto: "Take photo",
          sendVideo: "Send video",
          sendFile: "Send file",
          dropZone: "Drop files here, or choose files above",
          localOnly: "Local network only. No cloud upload.",
          emptyText: "Text is empty.",
          chooseFiles: "Choose files first.",
          tooLarge: "{name} is too large. Maximum size is {max}.",
          uploading: "Uploading... Keep this page open until it finishes.",
          sent: "Sent successfully",
          sentCount: "Sent {count} file(s) successfully",
          sendFailed: "Send failed.",
          uploadFailed: "Upload failed."
        },
        "zh-CN": {
          title: "投递到这台电脑",
          checkingSession: "正在检查会话...",
          sessionExpired: "会话已过期，请重新扫码。",
          networkUnreachable: "网络不可达。",
          connected: "已连接到 {device}，会话将在 {time} 过期。",
          sendText: "发送文字",
          textPlaceholder: "在这里粘贴或输入文字",
          textHelp: "可直接粘贴文字，然后发送到电脑。",
          chooseWhat: "选择要发送的内容",
          sendPhoto: "发送图片",
          takePhoto: "拍照发送",
          sendVideo: "发送视频",
          sendFile: "发送文件",
          dropZone: "把文件拖到这里，或点击上方入口选择",
          localOnly: "仅在本地网络传输，不经过云端。",
          emptyText: "文字内容为空。",
          chooseFiles: "请先选择文件。",
          tooLarge: "{name} 太大。单文件最大 {max}。",
          uploading: "正在上传，请保持页面打开直到完成。",
          sent: "发送成功",
          sentCount: "已成功发送 {count} 个文件",
          sendFailed: "发送失败。",
          uploadFailed: "上传失败。"
        }
      };
      let language = normalizeLanguage(params.get("lang") || navigator.language || "en");
      const statusBox = document.getElementById("status");
      const sessionText = document.getElementById("sessionText");
      const textInput = document.getElementById("text");
      const photoInput = document.getElementById("photoInput");
      const takePhotoInput = document.getElementById("takePhotoInput");
      const videoInput = document.getElementById("videoInput");
      const fileInput = document.getElementById("fileInput");
      const sendText = document.getElementById("sendText");
      const sendPhoto = document.getElementById("sendPhoto");
      const takePhoto = document.getElementById("takePhoto");
      const sendVideo = document.getElementById("sendVideo");
      const sendFile = document.getElementById("sendFile");
      const languageSelect = document.getElementById("languageSelect");
      const dropZone = document.getElementById("dropZone");
      let maxUploadBytes = Number.POSITIVE_INFINITY;
      let sessionReady = false;
      let uploadPending = false;

      function devLog(message) {
        if (new URLSearchParams(location.search).get("debug") === "1") console.debug("[droplite]", message);
      }

      function normalizeLanguage(value) {
        return String(value || "").toLowerCase().startsWith("zh") ? "zh-CN" : "en";
      }

      function t(key, values) {
        let text = (dictionaries[language] || dictionaries.en)[key] || dictionaries.en[key] || key;
        for (const [name, value] of Object.entries(values || {})) text = text.split("{" + name + "}").join(String(value));
        return text;
      }

      function applyLanguage() {
        document.documentElement.lang = language;
        languageSelect.value = language;
        document.querySelectorAll("[data-i18n]").forEach((node) => {
          node.textContent = t(node.getAttribute("data-i18n"));
        });
        document.querySelectorAll("[data-i18n-placeholder]").forEach((node) => {
          node.setAttribute("placeholder", t(node.getAttribute("data-i18n-placeholder")));
        });
      }

      function setStatus(message, type) {
        statusBox.hidden = false;
        statusBox.className = "status " + type;
        statusBox.textContent = message;
      }

      function formatBytes(bytes) {
        if (bytes >= 1024 * 1024) return (bytes / 1024 / 1024).toFixed(1) + " MB";
        if (bytes >= 1024) return (bytes / 1024).toFixed(1) + " KB";
        return bytes + " B";
      }

      function uploadNameFor(file) {
        const original = (file && file.name ? file.name : "").trim();
        if (original && original !== "blob" && original !== "image" && original !== "unknown") return original;

        const type = file && file.type ? file.type.toLowerCase() : "";
        const extension = type === "image/png" ? "png"
          : type === "image/webp" ? "webp"
          : type === "image/heic" ? "heic"
          : type === "image/heif" ? "heif"
          : type === "video/mp4" ? "mp4"
          : type === "video/quicktime" ? "mov"
          : type === "video/webm" ? "webm"
          : type.startsWith("image/") ? "jpg"
          : "bin";
        const prefix = type.startsWith("image/") ? "image" : type.startsWith("video/") ? "video" : "file";
        return prefix + "." + extension;
      }

      async function parseResponse(response) {
        const text = await response.text();
        if (!text) return {};
        try {
          return JSON.parse(text);
        } catch (_) {
          return { error: text };
        }
      }

      function disableUploads(disabled) {
        textInput.disabled = disabled;
        photoInput.disabled = disabled;
        takePhotoInput.disabled = disabled;
        videoInput.disabled = disabled;
        fileInput.disabled = disabled;
        sendText.disabled = disabled;
        sendPhoto.disabled = disabled;
        takePhoto.disabled = disabled;
        sendVideo.disabled = disabled;
        sendFile.disabled = disabled;
      }

      async function checkSession() {
        if (!token) {
          sessionText.textContent = t("sessionExpired");
          disableUploads(true);
          return;
        }

        const response = await fetch("/api/session?token=" + encodeURIComponent(token));
        const data = await response.json();
        if (!data.valid) {
          sessionText.textContent = t("sessionExpired");
          setStatus(data.reason || t("sessionExpired"), "error");
          disableUploads(true);
          return;
        }

        const expiresAt = new Date(data.expires_at * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
        maxUploadBytes = data.max_upload_bytes;
        sessionText.textContent = t("connected", { device: data.device_name, time: expiresAt });
        sessionReady = true;
        disableUploads(false);
        devLog("session ready");
      }

      async function sendTextValue() {
        if (!sessionReady) {
          setStatus(t("checkingSession"), "error");
          return;
        }
        if (uploadPending) return;
        const text = textInput.value.trim();
        if (!text) {
          setStatus(t("emptyText"), "error");
          return;
        }

        uploadPending = true;
        sendText.disabled = true;
        devLog("text upload started");
        try {
          const response = await fetch("/api/upload/text?token=" + encodeURIComponent(token), {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ text })
          });
          const data = await parseResponse(response);
          devLog("text upload response received: " + response.status);
          if (!response.ok) throw new Error(data.error || t("sendFailed"));
          textInput.value = "";
          setStatus(t("sent"), "ok");
          devLog("text upload success");
        } catch (error) {
          setStatus(error.message || t("networkUnreachable"), "error");
          devLog("text upload failed");
        } finally {
          uploadPending = false;
          sendText.disabled = false;
        }
      }

      async function uploadFiles(fileList, sourceInput) {
        if (!sessionReady) {
          setStatus(t("checkingSession"), "error");
          if (sourceInput) sourceInput.value = "";
          return;
        }
        if (uploadPending) {
          if (sourceInput) sourceInput.value = "";
          return;
        }
        const files = Array.from(fileList || []);
        devLog("file input changed: " + files.length + " file(s)");
        if (files.length === 0) {
          setStatus(t("chooseFiles"), "error");
          if (sourceInput) sourceInput.value = "";
          return;
        }
        const oversized = files.find((file) => file.size > maxUploadBytes);
        if (oversized) {
          setStatus(t("tooLarge", { name: oversized.name, max: formatBytes(maxUploadBytes) }), "error");
          if (sourceInput) sourceInput.value = "";
          return;
        }

        uploadPending = true;
        disableUploads(true);
        devLog("upload started");
        setStatus(t("uploading"), "ok");
        const form = new FormData();
        for (const file of files) form.append("files", file, uploadNameFor(file));

        try {
          const response = await fetch("/api/upload/file?token=" + encodeURIComponent(token), {
            method: "POST",
            body: form
          });
          const data = await parseResponse(response);
          devLog("upload response received: " + response.status);
          if (!response.ok) throw new Error(data.error || t("uploadFailed"));
          photoInput.value = "";
          takePhotoInput.value = "";
          videoInput.value = "";
          fileInput.value = "";
          setStatus(t("sentCount", { count: files.length }), "ok");
          devLog("upload success");
        } catch (error) {
          setStatus(error.message || t("networkUnreachable"), "error");
          devLog("upload failed");
        } finally {
          if (sourceInput) sourceInput.value = "";
          uploadPending = false;
          disableUploads(false);
        }
      }

      languageSelect.addEventListener("change", () => {
        language = normalizeLanguage(languageSelect.value);
        applyLanguage();
        checkSession().catch(() => undefined);
      });
      sendText.addEventListener("click", sendTextValue);
      sendPhoto.addEventListener("click", () => photoInput.click());
      takePhoto.addEventListener("click", () => takePhotoInput.click());
      sendVideo.addEventListener("click", () => videoInput.click());
      sendFile.addEventListener("click", () => fileInput.click());
      photoInput.addEventListener("change", (event) => uploadFiles(event.currentTarget.files, event.currentTarget));
      takePhotoInput.addEventListener("change", (event) => uploadFiles(event.currentTarget.files, event.currentTarget));
      videoInput.addEventListener("change", (event) => uploadFiles(event.currentTarget.files, event.currentTarget));
      fileInput.addEventListener("change", (event) => uploadFiles(event.currentTarget.files, event.currentTarget));
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

      applyLanguage();
      disableUploads(true);
      devLog("upload page loaded");
      devLog("token parsed: " + (token ? "present" : "missing"));
      checkSession().catch(() => {
        sessionText.textContent = t("networkUnreachable");
        disableUploads(true);
      });
    </script>
  </body>
</html>"#;
