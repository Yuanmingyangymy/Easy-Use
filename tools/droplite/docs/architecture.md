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

## Desktop and Phone Flow

1. DropLite starts on the desktop.
2. Rust creates a new session token with a 10-minute expiry.
3. Rust binds the local HTTP server to `0.0.0.0:0`, allowing the OS to choose a free port.
4. Rust detects a local IP address and builds a connection URL.
5. The desktop UI requests the current session through a Tauri command.
6. The desktop UI renders the QR code.
7. The phone scans the QR code and opens the local mobile upload page.
8. The phone sends text or files to the local HTTP API with the token.
9. Rust validates the token and expiry before accepting the upload.
10. Rust emits a Tauri event so the desktop receive list updates immediately.

## Local HTTP Service

The MVP service exposes:

- `GET /` for the mobile upload page.
- `GET /api/session?token=...` for phone-side session validation.
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
- Empty or dangerous names fall back to `upload`.
- Existing files are never overwritten.
- Duplicate names receive a suffix such as `photo (1).png`.

Received text is kept only in memory for the current app session. Files remain because the user intentionally received them.

## Future Extensions

Possible future versions can add:

- Desktop to phone transfer by adding a desktop send queue and phone download page.
- Bidirectional transfer by separating send and receive channels.
- HTTPS for local transport through generated local certificates or pairing-based trust.
- Optional end-to-end encryption where the QR code carries a pairing key and payloads are encrypted before upload.

Those additions should remain optional and clearly explained to avoid turning DropLite into a complex sync or chat product.
