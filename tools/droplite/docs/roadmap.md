# DropLite Roadmap

## 0.1 Phone to Desktop Drop

- QR connection flow.
- Temporary session token.
- Phone browser upload page.
- Text, image, video, and file upload.
- Separate phone choices for text, photos, videos, and files.
- English and Simplified Chinese UI.
- Timestamp-based received filenames.
- Image previews in the desktop receive list.
- Local receive directory.
- Basic desktop receive list.
- Honest local HTTP security model.

## 0.2 Desktop to Phone Drop

- Select text or files on the desktop.
- Phone page can download from the desktop.
- Keep the same temporary-session model.

## 0.3 Bidirectional Transfer

- Combine phone-to-desktop and desktop-to-phone flows.
- Keep transfers temporary and explicit.
- Avoid chat, history, or sync behavior.

## 0.4 Tray Mode

- Optional tray presence for quick access.
- Clear indication when a session is active.
- No hidden background receiving without user awareness.

## 0.5 Packaging

- More complete Windows, macOS, and Linux packaging.
- Release checklist and signed build guidance where practical.
- Optional code signing.
- GitHub Release packaging.
- Optional auto-update after a release process exists.

## Later Product Polish

- More complete settings page.
- Optional setting to preserve original filenames.
- Better thumbnail generation for very large images.

## 0.6 Optional End-to-End Encryption

- Explore pairing-based encryption.
- Keep security claims tied to the actual implementation.
- Preserve the simple QR workflow.

## 0.7 Optional Relay Mode

- Investigate cross-network transfer only as an explicit optional mode.
- The UI must clearly state when data leaves the local network.
- Relay mode must not become the default MVP behavior.
