# DropLite Architecture

## Overview

DropLite is a Tauri desktop app with a React desktop interface and a Rust backend. The Rust backend owns the local session, starts the temporary LAN HTTP server, receives uploads, saves files, and notifies the desktop UI.

The phone side is intentionally simple. It is a mobile web page served by the local Rust HTTP server at the QR code URL.

## Components

- Desktop UI: React, TypeScript, Vite, and Tauri commands/events.
- Local service: Rust HTTP server using Axum.
- Session layer: random token, expiry timestamp, refresh behavior.
- Storage layer: receive directory selection, filename sanitization, unique save paths.
- Network layer: local IP detection and automatic available port binding.
- I18n layer: small English and Simplified Chinese dictionaries for the desktop React UI and the mobile upload page.

## Desktop and Phone Flow

1. DropLite starts on the desktop.
2. Rust creates a new session token with a 10-minute expiry.
3. Rust binds the local HTTP server to `0.0.0.0:0`, allowing the OS to choose a free port.
4. Rust detects a local IP address and builds a connection URL.
5. The desktop UI requests the current session through a Tauri command.
6. The desktop UI renders the QR code.
7. The phone scans the QR code and opens the local mobile upload page. The QR URL includes `lang=en` or `lang=zh-CN`.
8. The phone sends text or files to the local HTTP API with the token.
9. Rust validates the token and expiry before accepting the upload.
10. Rust emits a Tauri event so the desktop receive list updates immediately.

## Local HTTP Service

The MVP service exposes:

- `GET /` for the mobile upload page.
- `GET /api/session?token=...` for phone-side session validation.
- `GET /api/received/:id/preview?token=...` for image previews in the desktop receive list.
- `POST /api/upload/text?token=...` for text payloads.
- `POST /api/upload/file?token=...` for multipart file uploads.
- `OPTIONS` handlers for browser compatibility.

The service is bound to all local interfaces so the phone can reach it from the same LAN. The QR code uses the detected LAN IP, not `localhost`.

## Token Generation and Validation

Each session has:

- A random 32-character token.
- A creation timestamp.
- An expiry timestamp.
- The currently bound server port.

Every upload request must include the token. Validation rejects missing, invalid, or expired tokens. Refreshing the session generates a new token while keeping the current server port.

## File Storage

DropLite saves files to `Downloads/DropLite` by default. If the downloads directory is unavailable, it falls back to an app data directory.

Before saving:

- Path separators are removed.
- Illegal filename characters are replaced.
- Empty or dangerous names fall back to generated names.
- Existing files are never overwritten.
- Files are named by type and receive time, such as `image-20260506-225801-001.jpg`.
- Multiple files received in the same second increment the sequence number.
- Extensions are taken from the original filename when reliable, then inferred from MIME type, then `.bin`.

Uploads are written to a temporary `.part` file first. DropLite flushes and syncs the file, then renames it to the final timestamped filename only after the upload is complete. Failed or oversized uploads remove the partial file.

Received text is kept only in memory for the current app session. Files remain because the user intentionally received them.

## Receive Directory Configuration

Status: implemented on the v0.2 feature branch. This is a product polish improvement for the existing phone-to-desktop receive flow, not part of the desktop-to-phone outbox.

The default receive directory remains `Downloads/DropLite`. Users can change it from the desktop UI with `Change folder`, open it with `Open folder`, or restore the default with `Reset`.

The Rust backend persists only the selected receive directory in a small local JSON config file:

```text
<system config dir>/Easy-Use/DropLite/config.json
```

On Windows this resolves under the user's app config area, such as AppData. The config is local to the computer and is not written into the repository or the receive directory.

Startup behavior:

- If no config exists, DropLite uses `Downloads/DropLite`.
- If the configured directory exists, DropLite uses it.
- If the configured directory is missing, DropLite attempts to create it.
- If the config is damaged or the directory is unusable, DropLite falls back to the default receive directory.

Runtime behavior:

- Upload saving reads the current backend receive directory.
- `Open folder` opens the current configured directory.
- `Refresh session` does not reset the receive directory.
- Changing the receive directory does not move old files.
- The setting applies only to new phone-to-desktop received files.
- Desktop-to-phone outbox files keep backend-only references to the user's chosen source files and are not copied into the receive directory.

## Image Previews

The desktop receive list does not expose raw Windows file paths to the WebView. For received images, Rust provides a session-token-protected preview URL:

```text
http://127.0.0.1:<port>/api/received/<id>/preview?token=<token>
```

The preview route only serves files that are already in the current in-memory received list and whose canonical path stays inside the DropLite receive directory.

## I18n

The desktop UI stores translations in `src/i18n/locales/en.ts` and `src/i18n/locales/zh-CN.ts`. The provider detects the local language, falls back to English, and persists manual choices in `localStorage`.

The mobile page has a small embedded dictionary because it is served directly by the Rust local HTTP server. It reads the QR `lang` parameter and also provides a compact language selector.

## Future Extensions

Possible future versions can add:

- Desktop to phone transfer by adding a desktop send queue and phone download page.
- Bidirectional transfer by separating send and receive channels.
- HTTPS for local transport through generated local certificates or pairing-based trust.
- Optional end-to-end encryption where the QR code carries a pairing key and payloads are encrypted before upload.

Those additions should remain optional and clearly explained to avoid turning DropLite into a complex sync or chat product.

## v0.2 Outbox Architecture: Desktop to Phone

Status: Phase 1 backend model, Phase 2 desktop send UI, Phase 2.5 configurable receive folder, and Phase 3 mobile receive UI are implemented on the `feat/droplite-desktop-to-phone` branch. This is still v0.2 branch work and is not part of the v0.1.0 Preview release.

The v0.2 approach keeps the existing local HTTP service and adds an in-memory desktop outbox. The desktop UI adds text or files to the Rust backend through Tauri commands, the phone page polls a token-protected outbox endpoint, and the phone can copy text or download files through token-protected routes.

Planned flow:

```text
Desktop UI
-> Tauri/Rust backend
-> in-memory outbox
-> phone browser polling
-> phone receive cards
-> token-protected file download
```

Implemented API shape:

- `GET /api/outbox?token=...` lists current pending desktop-to-phone items.
- `GET /api/outbox/:id/download?token=...` downloads a registered file item.
- `POST /api/outbox/:id/ack?token=...` marks an item as claimed or downloaded.

Implemented internal backend methods:

- `add_outbox_text(content)` creates a memory-only text item and rejects empty text.
- `add_outbox_file(path)` creates a file, image, or video item from a backend-only canonical path, rejects directories and missing files, and never returns the local path to the phone.

Desktop UI integration:

- The desktop WebView calls Tauri commands, not public HTTP endpoints, to add outbox items.
- `add_outbox_text` adds text entered in the desktop panel.
- `add_outbox_file` adds file paths selected by the native Tauri dialog or received from Tauri file drop events.
- `list_outbox_items` lets the desktop panel show the current memory-only outbox.
- File paths are used only as command inputs and backend references. The rendered outbox list shows display names, type, size, time, and status, but not absolute paths.

Mobile UI integration:

- The mobile page keeps the existing `Send to desktop` upload controls.
- The mobile page uses tabs for `Send to desktop` and `Receive from desktop` so a long desktop outbox cannot bury the phone-to-desktop upload controls.
- A `Receive from desktop` tab polls `GET /api/outbox?token=...` about every 1.5 seconds while the page is visible.
- Polling replaces the list from the server and de-duplicates by item id before rendering.
- Text items render as copyable cards. Copying text calls `POST /api/outbox/:id/ack?token=...`.
- File, image, and video items render as download cards. Download uses `GET /api/outbox/:id/download?token=...` and then acknowledges the item after a download/open action is triggered.
- Embedded browser environments that may block downloads show a system-browser and copy-link fallback rather than silently failing.
- WeChat's in-app browser is detected by user agent only for UX messaging. It shows a dedicated hint to open the page in the system browser. This detection is not a security mechanism.
- File cards always expose `Download` and `Copy download link`. Copying a link does not acknowledge the item because it does not prove the file was downloaded.
- The phone page uses item ids and the session token only. It never receives or displays desktop absolute paths.
- Manual bidirectional QA for this development branch is tracked in [`v0.2-manual-qa.md`](v0.2-manual-qa.md).
- If the token expires, polling stops and the page shows the session expired state.

Desktop drag-and-drop integration de-duplicates paths within a single drop event and ignores duplicate drop events from the same gesture for a short window. Intentional repeated drops are still allowed as separate user actions.

Polling is preferred over WebSocket for v0.2 because it is simpler, broadly compatible with phone browsers, easier to debug, and less likely to destabilize the v0.1 upload path. A future version can revisit WebSocket if the product need becomes clear.

Outbox state should remain memory-only, expire with the session, and be cleared when the app closes or the user refreshes the session.

Refresh session behavior is explicit: the old token is invalidated and the current outbox is cleared. This prevents an old phone page from downloading newly queued desktop content.
