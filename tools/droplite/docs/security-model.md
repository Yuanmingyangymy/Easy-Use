# DropLite Security Model

## Current Boundary

DropLite 0.1 is a local-network HTTP transfer tool. It is designed for quick phone-to-desktop transfer on a trusted LAN.

It does not use:

- Accounts.
- Third-party cloud storage.
- Remote relay servers.
- Telemetry.
- Ads.
- Transfer history.

## Security Assumptions

The MVP assumes:

- The phone and desktop are on the same trusted local network.
- The QR code is only shown to the user or trusted people nearby.
- The local network is not actively hostile.
- The receiving computer is allowed to save files from the phone.

## What Is Protected

- Uploads require the current session token.
- The token is random and short-lived.
- Refreshing the session invalidates the previous token.
- Expired sessions cannot upload.
- Filenames are sanitized before saving.
- Existing files are not overwritten.
- Single-file uploads are limited to 200 MB by default.
- Text receive history is memory-only and disappears when the app closes.

## What Is Not Protected

Local HTTP is not encrypted. A device on the same network may be able to observe traffic depending on the network environment.

The token is present in the QR URL. Anyone who can view or capture that URL during the valid session may attempt to upload to the desktop.

DropLite 0.1 does not authenticate individual phone devices. The token is the session gate.

## Windows Private Network Note

On Windows, a Wi-Fi network marked as Public may block phones from opening the DropLite QR link, even when both devices are on the same Wi-Fi. Marking a trusted home or office Wi-Fi as Private allows local devices to reach services running on the computer.

Only use Private for networks you trust. Do not mark public, hotel, conference, or cafe Wi-Fi as Private just to use DropLite. If Windows Firewall prompts for access, allow DropLite on private networks only.

## When It Is Reasonable To Use

DropLite 0.1 is reasonable on:

- A home network you trust.
- A private office network you trust.
- A direct local network where you control the devices.

## When Not To Use It

Avoid using this MVP on:

- Public Wi-Fi.
- Shared conference networks.
- Networks where other devices should not be trusted.
- Any environment where local HTTP traffic exposure is unacceptable.

## Why Data Does Not Go Through The Cloud

The phone sends data directly to the local HTTP server running on the receiving computer. The QR code points to a local IP address and port. There is no remote API endpoint in the MVP.

## Image Preview Access

Image thumbnails are loaded through a local DropLite preview route, not by exposing arbitrary local filesystem paths to the WebView. The route requires the current session token, looks up the requested item in the in-memory received list, and verifies the canonical file path stays inside the DropLite receive directory.

This means the preview route is limited to files the current DropLite session already received. It is not a general file browser.

## Local Build Boundary

A locally built DropLite app has the same security model as development mode: local-network HTTP, temporary token, no cloud, and no account. Local builds are not automatically safer than development builds.

Current local builds are not code signed. Windows may show a security warning, and Windows Firewall may ask for network permission. Allow private network access only on trusted networks.

## Configurable Receive Directory

Status: implemented on the v0.2 feature branch.

Users can choose where phone-to-desktop received files are saved. DropLite stores this receive directory preference only on the local computer in a small config file under the user's app config area.

Security notes:

- DropLite does not upload the receive directory path to any cloud service.
- The configured path is used only by the local Rust backend.
- New received files are saved to the current configured receive directory.
- Old files are not moved when the directory changes.
- If the configured directory is unavailable, DropLite falls back to the default receive directory.
- This setting does not affect desktop-to-phone outbox source files. Outbox file paths remain backend-only references and are not shown to the phone.

## Future Security Improvements

Potential upgrades:

- Local HTTPS with generated certificates and clear pairing UX.
- End-to-end encryption where the QR code carries a one-time encryption key.
- Optional per-transfer confirmation on the desktop.
- Optional stricter content limits.
- Optional relay mode for cross-network transfer, but only with transparent UI that states when data leaves the local network.

## v0.2 Outbox Security Considerations

Status: Phase 1 backend model and Phase 2 desktop send UI implemented on the `feat/droplite-desktop-to-phone` branch. The mobile receive UI is still not implemented.

The v0.2 desktop-to-phone flow keeps the same Local-first / No account / No cloud boundary:

- Every outbox list, acknowledgement, and download API validates the current token.
- Expired tokens are rejected for outbox list and download access.
- Refreshing the session clears the current outbox so old pending items are not available under a new pairing context.
- Files must come from explicit desktop user actions, such as choosing a file or dragging it into DropLite.
- Adding outbox items is available only through Tauri commands inside the desktop app, not through a LAN HTTP endpoint.
- The phone-side HTTP API can list, download, and acknowledge outbox items with a valid token, but it cannot add new outbox items.
- The backend stores safe server-side references and never exposes arbitrary local paths to the phone.
- The desktop outbox panel also renders only display names and metadata, not local absolute paths.
- Download endpoints resolve only files registered in the current in-memory outbox.
- Filenames used for downloads are sanitized before being placed in `Content-Disposition`.
- Text sent to the phone should remain memory-only and should disappear when the app closes or the session is refreshed.
- v0.2 should not add accounts, cloud relay, tracking, a database, or permanent transfer history.

The transport would still be local-network HTTP in v0.2. It remains inappropriate for public or untrusted Wi-Fi unless a future version adds stronger transport security.
