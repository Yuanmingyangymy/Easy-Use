# DropLite

DropLite is the first tool in the Easy-Use collection. It is a minimal, account-free, local-first temporary drop tool for sending text, images, and files from a phone browser to a desktop.

No account. No cloud. Local-first. Temporary session. Open source. Phone browser to desktop.

## Screenshot

Product screenshots will be added after the first packaged preview build. The MVP interface contains a desktop QR screen, session countdown, safety note, and a recent receive list.

## Core Features

- Desktop app built with Tauri, Rust, React, TypeScript, and Vite.
- Starts a temporary local-network HTTP server on an available port.
- Detects the local LAN IP and shows a QR code for the phone.
- Phone opens a browser upload page without installing an app.
- Sends text, images, and files from phone to desktop.
- Saves received files to the user's Downloads/DropLite folder by default.
- Shows received text immediately and provides copy action.
- Shows image thumbnails in the desktop receive list.
- Uses a random session token with a default 10-minute expiry.
- Does not use accounts, cloud services, telemetry, ads, or transfer history.

## MVP 0.1 Scope

Supported:

- Phone to desktop transfer.
- Text upload.
- Image upload.
- File upload.
- Drag-and-drop upload where the mobile browser supports it.
- Paste text on the phone page.
- Token validation and expiry.
- Single-file upload limit of 200 MB by default.
- Safe filename handling and non-overwriting save behavior.

Not supported in this MVP:

- Desktop to phone transfer.
- Public internet transfer.
- User accounts.
- Cloud sync.
- Chat.
- Friend lists.
- Device address books.
- Phone apps.
- Folder sync.
- Clipboard sync.
- Multi-user rooms.
- Transfer history.
- Complex settings.

## Security and Privacy

DropLite 0.1 uses local-network HTTP. Data does not go through a third-party cloud service, and the app does not collect analytics or tracking data.

The QR code contains a local address, port, and short-lived session token:

```text
http://<local-ip>:<port>/?token=<session-token>
```

Requests must include the token. The token is generated at startup, can be refreshed manually, and expires after 10 minutes by default.

Important boundary: local HTTP is not encrypted. Do not use this MVP on untrusted public networks. Anyone who can see the QR code or token during the valid session may attempt to upload to the receiving computer.

See [docs/security-model.md](docs/security-model.md) for the full security model.

## Local Development

Requirements:

- Node.js 18+
- Rust stable
- Platform dependencies required by Tauri

Install dependencies:

```bash
npm install
```

Run the desktop app in development:

```bash
npm run tauri:dev
```

Run only the Vite frontend:

```bash
npm run dev
```

## Build

Build the frontend:

```bash
npm run build
```

Build the Tauri app:

```bash
npm run tauri:build
```

## Tests

Run Rust tests:

```bash
cd src-tauri
cargo test
```

Run frontend tests:

```bash
npm test
```

The Rust tests cover token generation, token validation, token expiry, filename sanitization, unique save path behavior, and upload size limits.

## FAQ

### Does DropLite upload my files to the cloud?

No. The MVP starts a local HTTP server on the receiving computer. The phone sends data directly to that computer on the local network.

### Do I need a phone app?

No. Scan the QR code and use the phone browser.

### Why does the session expire?

The short-lived token limits accidental exposure. Use Refresh session on the desktop to generate a new QR code.

### Where are files saved?

By default, files are saved to `Downloads/DropLite`. If the downloads directory cannot be found, DropLite falls back to an app data directory.

### Is local HTTP encrypted?

No. This MVP is intended for trusted local networks. Avoid public or untrusted Wi-Fi.

## Roadmap

See [docs/roadmap.md](docs/roadmap.md).

## Contributing

Please read the repository-level [CONTRIBUTING.md](../../CONTRIBUTING.md).

## License

MIT License. See [../../LICENSE](../../LICENSE).
