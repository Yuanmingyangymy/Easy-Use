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

## Future Security Improvements

Potential upgrades:

- Local HTTPS with generated certificates and clear pairing UX.
- End-to-end encryption where the QR code carries a one-time encryption key.
- Optional per-transfer confirmation on the desktop.
- Optional stricter content limits.
- Optional relay mode for cross-network transfer, but only with transparent UI that states when data leaves the local network.
