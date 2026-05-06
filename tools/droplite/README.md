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
- Provides separate phone entry points for text, photos, videos, and files.
- Saves received files to the user's Downloads/DropLite folder by default.
- Names received files by type and receive time, such as `image-20260506-225801-001.jpg`.
- Shows received text immediately and provides copy action.
- Shows image thumbnails in the desktop receive list.
- Supports English and Simplified Chinese UI.
- Uses a random session token with a default 10-minute expiry.
- Does not use accounts, cloud services, telemetry, ads, or transfer history.

## MVP 0.1 Scope

Supported:

- Phone to desktop transfer.
- Text upload.
- Image upload.
- Video upload.
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

Run the desktop app in development. This mode is for contributors and debugging:

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
npm run tauri build
```

Local builds generate installable or runnable desktop artifacts under `src-tauri/target/release/bundle/`. The exact installer path depends on the platform and Tauri CLI output.

Current local builds are not code signed and are not official releases. On Windows, the first run may show a security warning, and the firewall may ask whether to allow local network access. For DropLite's LAN workflow, allow private network access only on trusted networks.

Do not commit build outputs from `dist/`, `build/`, `target/`, or installer folders.

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

Run frontend type checks:

```bash
npm run typecheck
```

The Rust tests cover token generation, token validation, token expiry, timestamp-based filename generation, filename sanitization, unique save path behavior, received item kinds, and upload size limits.

## Language

DropLite currently supports English and Simplified Chinese. The desktop app follows the system or browser language by default, falls back to English, and lets the user switch language in the header. The selected language is stored locally in `localStorage`.

The QR code includes a `lang` parameter so the phone upload page follows the desktop language. The phone page also has a small language selector.

## File Naming

DropLite uses receive-time names by default:

- `image-YYYYMMDD-HHMMSS-001.jpg`
- `video-YYYYMMDD-HHMMSS-001.mp4`
- `file-YYYYMMDD-HHMMSS-001.pdf`

The prefix comes from MIME type or extension. The extension is taken from the original filename when reliable, then inferred from MIME type, then falls back to `.bin`. If multiple files arrive in the same second, DropLite increments the sequence and never overwrites existing files.

## Manual Acceptance

Before a release, verify the MVP with real devices:

- Start DropLite with `npm run tauri:dev`.
- Confirm the desktop app opens and shows a QR code.
- Put the phone and computer on the same trusted Wi-Fi.
- On Windows, set the current Wi-Fi network to Private and allow firewall access if prompted.
- Scan the QR code from the phone and open the upload page.
- Send text and confirm it appears in Recently received.
- Send a JPG or PNG image and confirm the desktop list shows an image record and thumbnail.
- Send a small MP4 video under 50 MB and confirm the saved video plays through completely.
- Send a PDF or ZIP and confirm it appears as a file record and opens from the receive folder.
- Send two files with the same name and confirm DropLite creates `file (1).ext` instead of overwriting.
- Let the token expire and confirm uploads are rejected with a friendly message.

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

### My phone cannot open the QR link. What should I check?

Make sure the phone and computer are connected to the same Wi-Fi. Avoid guest Wi-Fi because it often blocks devices from reaching each other.

Turn off VPNs on the phone and computer, then try again.

On Windows, confirm the current Wi-Fi is set to Private network. If Windows Firewall asks whether to allow DropLite, allow access for private networks. Do not switch a public or untrusted Wi-Fi to Private just to use DropLite.

DropLite 0.1 uses local-network HTTP, so it is not recommended on public or untrusted Wi-Fi.

### Why does Send photo or Send video still open a file manager on my phone?

DropLite uses standard browser file inputs. `Send photo` uses `accept="image/*"` and `Send video` uses `accept="video/*"`, which usually opens a gallery or media picker. Some browsers, embedded webviews, or chat-app browsers may still show a file manager. That behavior is controlled by the browser and operating system.

### Why should the first upload work immediately?

DropLite shows the QR code only after the local server is listening, and the phone upload page keeps upload buttons disabled until the session is confirmed. A success message means the desktop has already saved the file and created the received record.

### My phone opens the QR page, but the first upload fails. What should I try?

Check that Wi-Fi is stable, turn off VPNs, and scan a fresh QR code. If the issue is repeatable, please open an issue with the file type, phone browser, operating system, and any DropLite dev logs. Do not rely on repeated retries as the expected workflow.

### Should I see `.part` files in the receive folder?

No. DropLite may create hidden `.droplite-upload-*.part` files while an upload is in progress, but a successful upload is renamed to the final timestamped name, such as `image-YYYYMMDD-HHMMSS-001.jpg`, `video-YYYYMMDD-HHMMSS-001.mp4`, or `file-YYYYMMDD-HHMMSS-001.pdf`.

If a `.part` file remains, the upload was interrupted or an earlier version hit a cleanup bug. Current builds remove temporary files when uploads fail because of size limits, token expiry, network interruption, or write errors.

## Roadmap

See [docs/roadmap.md](docs/roadmap.md).

## Contributing

Please read the repository-level [CONTRIBUTING.md](../../CONTRIBUTING.md).

## License

MIT License. See [../../LICENSE](../../LICENSE).
