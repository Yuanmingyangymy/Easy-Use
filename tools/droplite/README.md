# DropLite

DropLite is an Easy-Use tool for temporary cross-device transfer over the local network. It is designed to stay simple: no account, no cloud, no tracking, no permanent transfer history.

## v0.2.0 Preview

DropLite v0.2.0 Preview adds desktop-to-phone transfer on top of the existing phone-to-desktop flow, making DropLite a lightweight two-way local transfer tool.

DropLite v0.2.0 Preview 在原有“手机传电脑”的基础上，新增“电脑传手机”，让 DropLite 成为一个轻量的本地双向临时投递工具。

This is a preview release, not a final stable release. It still uses local-network HTTP and is intended for trusted private Wi-Fi networks.

## Core Features

- Desktop app built with Tauri, Rust, React, TypeScript, and Vite.
- QR-code based temporary local session.
- Phone browser to desktop transfer for text, photos, videos, and files.
- Desktop to phone transfer for text and files.
- Mobile tabs for `Send to desktop` and `Receive from desktop`.
- Desktop `Send to phone` panel with text input, file picker, and file drag-and-drop.
- Configurable receive folder for phone-to-desktop files.
- Timestamp-based received filenames such as `image-20260506-225801-001.jpg`.
- Recently received list on desktop.
- Copy actions for received text.
- Token-protected outbox list, download, and acknowledgement APIs.
- English and Simplified Chinese UI.
- No account, no cloud service, no telemetry, no ads.

## Current Preview Scope

Supported:

- Phone to desktop text upload.
- Phone to desktop photo upload.
- Take photo and send.
- Phone to desktop video upload.
- Phone to desktop generic file upload.
- Desktop to phone text transfer.
- Desktop to phone file transfer.
- Desktop file drag-and-drop into the outbox.
- Phone-side copy for desktop-sent text.
- Phone-side download or download-link fallback for desktop-sent files.
- Configurable receive folder with reset to default.
- Token validation, session refresh, and session expiry.
- Single-file upload limit of 200 MB by default.
- Safe filename handling and non-overwriting save behavior.
- Windows preview build.

Not supported:

- Public internet transfer.
- User accounts.
- Cloud sync or relay.
- WebRTC.
- Database-backed transfer history.
- Chat.
- Friend lists or device address books.
- Phone app.
- Folder sync.
- Clipboard sync.
- Multi-user rooms.
- Automatic saving to the phone photo gallery.
- Code signing.

## For Testers on Windows

1. Download the installer from the GitHub Release page.
2. Install and open DropLite.
3. If Windows shows a warning, it is because this preview build is not code signed yet.
4. Make sure your phone and computer are on the same Wi-Fi.
5. On Windows, set this Wi-Fi network to Private.
6. If the firewall asks for permission, allow Private networks.
7. Scan the QR code with your phone browser.
8. Use `Send to desktop` on the phone, or `Send to phone` on the desktop.

普通 Windows 试用步骤：

1. 从 GitHub Release 页面下载安装包。
2. 安装并打开 DropLite。
3. 如果 Windows 出现安全提醒，这是因为当前预览版尚未代码签名。
4. 确认手机和电脑连接到同一个 Wi-Fi。
5. 在 Windows 中将当前 Wi-Fi 设置为“专用网络”。
6. 如果防火墙请求权限，请允许“专用网络”访问。
7. 用手机浏览器扫码。
8. 可以从手机发送到电脑，也可以从电脑发送到手机。

Use the installer from the Release assets for normal testing. The bare `src-tauri/target/release/droplite.exe` is a build artifact for developers and should not be treated as the primary distribution file.

DropLite is a GUI app. A normal preview build should not open an extra black console window. If a console window appears, please report whether you launched DropLite from the installer, Start menu, installation folder, or the bare build artifact.

## Security and Privacy

DropLite uses local-network HTTP. Data does not go through a third-party cloud service, and the app does not collect analytics or tracking data.

The QR code contains a local address, port, and short-lived session token:

```text
http://<local-ip>:<port>/?token=<session-token>
```

Requests must include the token. The token is generated at startup, can be refreshed manually, and expires after 10 minutes by default.

Important boundary: local HTTP is not encrypted. Do not use this preview on untrusted public networks. Anyone who can see the QR code or token during the valid session may attempt to access the current transfer session.

Desktop-to-phone outbox items exist only in memory for the current session. File download APIs do not expose local absolute paths and only serve files explicitly added to the outbox from the desktop app.

See [docs/security-model.md](docs/security-model.md) for the full security model.

## Phone Browser Download Notes

For best desktop-to-phone download behavior, open the QR page in the phone's system browser, such as Android Chrome or iOS Safari.

WeChat's in-app browser may block direct file downloads. If that happens, tap the top-right menu, choose to open the page in the system browser, and download there. DropLite also provides a `Copy download link` fallback. The copied link still contains the temporary session token and expires with the session.

DropLite does not promise automatic saving to the phone photo gallery. Final save/open behavior is controlled by the phone browser and operating system.

## Receive Folder

Phone-to-desktop files are saved to `Downloads/DropLite` by default. In v0.2, the desktop app can change this folder, open it, or reset it to the default. The setting is stored locally on the computer and is not uploaded anywhere.

Changing the receive folder affects only future phone-to-desktop files. It does not move existing files and does not affect desktop-to-phone outbox source files.

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
npm run tauri build
```

Local builds generate installable desktop artifacts under `src-tauri/target/release/bundle/`. The preview build targets a Windows NSIS installer by default, usually under `src-tauri/target/release/bundle/nsis/`. MSI packaging is not enabled by default because it requires the WiX toolchain and can fail on first build if the WiX download times out.

Current local builds are not code signed and are not official releases. On Windows, the first run may show a security warning, and the firewall may ask whether to allow local network access. For DropLite's LAN workflow, allow private network access only on trusted networks.

Do not commit build outputs from `dist/`, `build/`, `target/`, or installer folders.

## Tests

Run frontend tests:

```bash
npm test
```

Run frontend type checks:

```bash
npm run typecheck
```

Run Rust tests:

```bash
cd src-tauri
cargo test
```

The test suite covers token handling, session expiry, filename safety, upload size limits, configurable receive folder behavior, outbox APIs, mobile receive rendering, download fallback behavior, and desktop drag-and-drop de-duplication.

## Manual QA

Before a v0.2 preview release, follow [docs/v0.2-manual-qa.md](docs/v0.2-manual-qa.md). It covers:

- Windows private network setup.
- Phone to desktop regression.
- Desktop to phone regression.
- System browser download behavior.
- WeChat in-app browser limitations.
- Refresh session and token expiry.
- Receive folder configuration.

## Release Notes

See [CHANGELOG.md](CHANGELOG.md), [docs/release-notes-v0.1.0-preview.md](docs/release-notes-v0.1.0-preview.md), and [docs/release-notes-v0.2.0-preview.md](docs/release-notes-v0.2.0-preview.md).

## Language

DropLite currently supports English and Simplified Chinese. The desktop app follows the system or browser language by default, falls back to English, and lets the user switch language in the header. The selected language is stored locally in `localStorage`.

The QR code includes a `lang` parameter so the phone page follows the desktop language. The phone page also has a small language selector.

## File Naming

DropLite uses receive-time names by default:

- `image-YYYYMMDD-HHMMSS-001.jpg`
- `video-YYYYMMDD-HHMMSS-001.mp4`
- `file-YYYYMMDD-HHMMSS-001.pdf`

The prefix comes from MIME type or extension. The extension is taken from the original filename when reliable, then inferred from MIME type, then falls back to `.bin`. If multiple files arrive in the same second, DropLite increments the sequence and never overwrites existing files.

## FAQ

### Does DropLite upload my files to the cloud?

No. DropLite starts a local HTTP server on the desktop. The phone sends data directly to that computer on the local network, and desktop-to-phone downloads also come from that local server.

### Do I need a phone app?

No. Scan the QR code and use the phone browser.

### Why does the session expire?

The short-lived token limits accidental exposure. Use `Refresh session` on the desktop to generate a new QR code.

### Is local HTTP encrypted?

No. This preview is intended for trusted local networks. Avoid public or untrusted Wi-Fi.

### My phone cannot open the QR link. What should I check?

Make sure the phone and computer are connected to the same Wi-Fi. Avoid guest Wi-Fi because it often blocks devices from reaching each other.

Turn off VPNs on the phone and computer, then try again.

On Windows, confirm the current Wi-Fi is set to Private network. If Windows Firewall asks whether to allow DropLite, allow access for private networks. Do not switch a public or untrusted Wi-Fi to Private just to use DropLite.

### Why does WeChat not download files directly?

WeChat's in-app browser may block direct file downloads. Open the page in the system browser from WeChat's top-right menu, then download there. You can also copy the download link, but the link is temporary and expires with the session.

### Should I see `.part` files in the receive folder?

No. DropLite may create hidden `.droplite-upload-*.part` files while an upload is in progress, but a successful upload is renamed to the final timestamped name. Current builds remove temporary files when uploads fail because of size limits, token expiry, network interruption, or write errors.

### How should I report preview issues?

Please include the DropLite version, Windows version, phone OS and browser, whether you used a system browser or WeChat, Wi-Fi or firewall notes, file type and size, and the exact steps to reproduce the issue.

## Roadmap

See [docs/roadmap.md](docs/roadmap.md).

## Contributing

Please read the repository-level [CONTRIBUTING.md](../../CONTRIBUTING.md).

## License

MIT License. See [../../LICENSE](../../LICENSE).
