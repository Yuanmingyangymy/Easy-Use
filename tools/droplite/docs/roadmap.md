# DropLite Roadmap

## v0.1.0 Preview

- QR connection flow.
- Temporary session token.
- Phone browser upload page.
- Phone to desktop transfer.
- Text, image, video, and file upload.
- Take photo and send.
- Separate phone choices for text, photos, videos, and files.
- English and Simplified Chinese UI.
- Timestamp-based received filenames.
- Image previews in the desktop receive list.
- Local receive directory.
- Basic desktop receive list.
- Honest local HTTP security model.
- Windows Preview build.

## v0.2 Planned: Desktop to Phone Transfer

Status: Phase 1 backend model, Phase 2 desktop send UI, Phase 2.5 configurable receive folder, and Phase 3 mobile receive UI implemented on the v0.2 feature branch. Phase 4 reliability and mobile download polish is in progress. Full v0.2 is not released yet.

- Desktop to phone transfer.
- In-memory outbox backend model.
- Token-protected outbox list, download, and ack APIs.
- Desktop `Send to phone` panel.
- Native file picker and Tauri file drop support for adding outbox files.
- Mobile `Receive from desktop` tab for copying text and downloading files.
- Download fallback guidance for restricted mobile browsers.
- De-duplication for dragged files and phone outbox rendering.
- Configurable receive folder for phone-to-desktop files.
- Manual QA checklist for bidirectional regression and WeChat/system-browser download behavior.
- Better receive and download flow on mobile.
- More polished settings.
- Optional keep original filename behavior.
- Better onboarding and diagnostics.

See [v0.2 desktop-to-phone design](v0.2-desktop-to-phone-design.md) and [v0.2 manual QA](v0.2-manual-qa.md).

## v0.3

- Cross-platform packaging for macOS and Linux.
- GitHub Actions release pipeline.
- Code signing research.
- Auto-update research.

## Later Product Polish

- Bidirectional transfer polish after desktop to phone exists.
- Optional tray presence for quick access.
- More complete settings page.
- Better thumbnail generation for very large images.

## Optional Security Enhancements

- Explore pairing-based encryption.
- Keep security claims tied to the actual implementation.
- Preserve the simple QR workflow.

## Optional Relay Mode

- Investigate cross-network transfer only as an explicit optional mode.
- The UI must clearly state when data leaves the local network.
- Relay mode must not become the default MVP behavior.
