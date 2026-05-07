# DropLite Changelog

## DropLite v0.1.0 Preview

DropLite v0.1.0 Preview is the first usable preview release for sending text, photos, videos, and files from a phone browser to a desktop over the local network.

DropLite v0.1.0 Preview 是首个可用预览版，支持通过手机浏览器将文字、图片、视频和文件在本地网络内投递到电脑。

### Added

- Phone browser to desktop local transfer.
- Text upload.
- Photo upload.
- Take photo and send.
- Video upload.
- Generic file upload.
- QR-code based temporary session.
- 10-minute session expiration.
- Timestamp-based received file naming.
- Recently received list.
- Copy received text.
- Open receive folder.
- Local network safety hints.
- English and Simplified Chinese UI.
- Mobile upload shortcuts for text, photo, video, and file.

### Security and privacy

- No account.
- No cloud.
- No tracking.
- Local network only.
- Temporary token-based session.
- Received files are saved locally.
- Text history is not persisted after app close.

### Known limitations

- Only phone to desktop is supported.
- Desktop to phone is not supported yet.
- Local HTTP is not encrypted.
- Not recommended on public or untrusted Wi-Fi.
- Windows users may need to set Wi-Fi to Private network.
- App is not code signed yet, so Windows may show a warning.
- Tested primarily on Windows for this preview.
- Normal Windows preview launches should show only the DropLite GUI, not an extra console window.

### Feedback wanted

- Windows version.
- Phone OS and browser.
- File type and size.
- Whether first upload succeeds.
- Whether QR page opens smoothly.
- Whether firewall/network setup is confusing.
- If an extra console window appears, whether DropLite was launched from the installer, Start menu, installation folder, or bare build artifact.
