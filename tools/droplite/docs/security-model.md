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

## Future Security Improvements

Potential upgrades:

- Local HTTPS with generated certificates and clear pairing UX.
- End-to-end encryption where the QR code carries a one-time encryption key.
- Optional per-transfer confirmation on the desktop.
- Optional stricter content limits.
- Optional relay mode for cross-network transfer, but only with transparent UI that states when data leaves the local network.
