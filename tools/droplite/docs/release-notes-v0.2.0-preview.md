# DropLite v0.2.0 Preview

DropLite v0.2.0 Preview adds desktop-to-phone transfer on top of the existing phone-to-desktop flow, making DropLite a lightweight two-way local transfer tool.

DropLite v0.2.0 Preview 在原有“手机传电脑”的基础上，新增“电脑传手机”，让 DropLite 成为一个轻量的本地双向临时投递工具。

This is a preview release for testers. It is not a final stable release.

## What Is New

- Send text from the desktop app to the phone browser.
- Send desktop files to the phone browser.
- Drag files into the desktop app to send them to the phone.
- Use the new mobile `Receive from desktop` tab to copy text or download files.
- Copy a token-protected download link when the browser cannot download directly.
- See clearer guidance for WeChat's in-app browser.
- Change the phone-to-desktop receive folder from the desktop UI.
- Keep the receive folder setting across app restarts.
- Use mobile send/receive tabs so long receive lists do not hide upload actions.

## Existing Phone to Desktop Flow

The v0.1 flow remains available:

- Send text from a phone browser to the desktop.
- Send photos, take a photo, send videos, and send generic files to the desktop.
- Use timestamp-based received filenames.
- View recent received items on the desktop.
- Copy received text.
- Open the current receive folder.

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

## Security and Privacy

- No account.
- No cloud.
- No tracking.
- Local network first.
- Temporary token-based sessions.
- No permanent transfer history.
- Received files are saved locally.
- Desktop-to-phone outbox items exist only in memory for the current session.
- Refreshing the session invalidates old session-scoped data.

## Known Limitations

- Local HTTP is not encrypted.
- Do not use DropLite on public or untrusted Wi-Fi.
- WeChat's in-app browser may block direct file downloads. Use the top-right menu to open the page in the system browser, then download there.
- Phone browser download behavior depends on the phone OS and browser.
- DropLite does not guarantee automatic saving to the phone photo gallery.
- The app is not code signed yet, so Windows may show a warning.
- This preview is tested primarily on Windows.

## Feedback Wanted

When reporting issues, please include:

- Windows version.
- Phone OS and browser.
- Whether the phone page was opened in a system browser or WeChat.
- File type and size.
- Whether phone-to-desktop transfer works.
- Whether desktop-to-phone text copy works.
- Whether desktop-to-phone file download works.
- Network setup details, especially Wi-Fi, VPN, firewall, and Private/Public network state.
