# DropLite v0.1.0 Preview Release Notes

## English

DropLite v0.1.0 Preview is the first usable preview release for sending text, photos, videos, and files from a phone browser to a desktop over the local network.

This is a preview release. It currently supports phone to desktop transfer only. DropLite uses local-network HTTP by default, does not require an account, does not use cloud services, does not track users, and does not persist transfer history after the app closes.

### Highlights

- Send text from a phone browser to the desktop.
- Send photos, take a photo, send videos, and send generic files.
- Scan a QR code to join a temporary 10-minute session.
- Save received files locally with readable timestamp-based names.
- See received items in the desktop Recently received list.
- Copy received text and open the receive folder.
- Use the UI in English or Simplified Chinese.

### Security and privacy

- No account.
- No cloud.
- No tracking.
- Local network only.
- Temporary token-based session.
- Received files are saved locally.
- Text history is memory-only and is not persisted after app close.

### Known limitations

- Desktop to phone transfer is not supported yet.
- Local HTTP is not encrypted.
- Avoid public or untrusted Wi-Fi.
- On Windows, set the current trusted Wi-Fi network to Private and allow firewall access for private networks.
- This preview build is not code signed, so Windows may show a warning.
- Tested primarily on Windows.

### Feedback wanted

Please include your Windows version, phone OS and browser, network setup, file type and size, whether the first upload succeeded, and clear reproduction steps.

## 简体中文

DropLite v0.1.0 Preview 是首个可用预览版，支持通过手机浏览器将文字、图片、视频和文件在本地网络内投递到电脑。

这是预览版。目前只支持手机到电脑传输。DropLite 默认使用局域网 HTTP，不需要账号，不经过云端，不做追踪，也不会在应用关闭后保留传输历史。

### 主要能力

- 从手机浏览器发送文字到电脑。
- 发送图片、拍照发送、发送视频、发送普通文件。
- 扫描二维码加入 10 分钟临时会话。
- 按接收时间保存文件，文件名更容易识别。
- 在桌面端 Recently received 列表查看接收记录。
- 复制接收到的文字，打开接收文件夹。
- 支持 English 和 简体中文界面。

### 安全与隐私

- 不需要账号。
- 不经过云端。
- 不做追踪。
- 仅用于本地网络。
- 使用临时 token 会话。
- 接收到的文件只保存在本机。
- 文字记录只保存在内存中，关闭应用后不保留。

### 已知限制

- 暂不支持电脑到手机传输。
- 局域网 HTTP 未加密。
- 不建议在公共或不可信 Wi-Fi 下使用。
- Windows 用户需要将当前可信 Wi-Fi 设置为“专用网络”，并允许防火墙的专用网络访问。
- 当前预览版未做代码签名，Windows 可能出现安全提醒。
- 本预览版主要在 Windows 上测试。

### 反馈时请提供

请尽量附上 Windows 版本、手机系统和浏览器、网络环境、文件类型和大小、第一次上传是否成功，以及清晰的复现步骤。
