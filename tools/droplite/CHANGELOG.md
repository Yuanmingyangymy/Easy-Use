# DropLite Changelog

## DropLite v0.2.0 Preview

DropLite v0.2.0 Preview adds desktop-to-phone transfer on top of the existing phone-to-desktop flow, making DropLite a lightweight two-way local transfer tool.

DropLite v0.2.0 Preview 在原有“手机传电脑”的基础上，新增“电脑传手机”，让 DropLite 成为一个轻量的本地双向临时投递工具。

### Added

- Desktop to phone transfer.
- Send text from desktop to phone.
- Send files from desktop to phone.
- Drag files on desktop to send to phone.
- Mobile `Receive from desktop` tab.
- Copy desktop-sent text on phone.
- Download desktop-sent files on phone.
- Copy download link fallback.
- WeChat in-app browser download guidance.
- Configurable receive folder for phone-to-desktop files.
- Receive folder persists across app restarts.
- Mobile send/receive tab layout.

### Improved

- Mobile layout no longer stacks send and receive sections endlessly.
- Outbox polling deduplicates items by id.
- Desktop drag-and-drop avoids duplicate outbox records.
- Download ack behavior is more conservative.
- Windows receive folder path display no longer shows `\\?\` prefix.

### Security and privacy

- Still no account.
- Still no cloud.
- Still local network first.
- Outbox APIs require session token.
- Download APIs do not expose local absolute paths.
- Refresh session invalidates old session-scoped data.
- Receive folder config is local only.

### Known limitations

- WeChat in-app browser may block direct file downloads.
- Use system browser for best phone download experience.
- Local HTTP is not encrypted.
- Public/untrusted Wi-Fi is not recommended.
- App is not code signed yet.
- Desktop-to-phone browser download behavior depends on mobile OS/browser.

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
