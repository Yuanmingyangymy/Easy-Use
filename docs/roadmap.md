# Easy-Use Roadmap

## Current Focus

### DropLite v0.2 Development

DropLite v0.1.0 Preview has been released. DropLite v0.2 is now on a feature branch and will add desktop-to-phone transfer while preserving the local-first, no-account, no-cloud model.

Current v0.2 status:

- Phase 1 backend outbox model is implemented on the v0.2 branch.
- Phase 2 desktop send UI is implemented on the v0.2 branch.
- The mobile receive UI is still planned.

Planned v0.2 direction:

- Desktop to phone text transfer.
- Desktop to phone file transfer.
- Phone-side copy and download actions.
- Token-protected local outbox.
- No cloud relay, account system, database, or permanent history.

### DropLite v0.1.0 Preview

The released v0.1.0 Preview focuses on phone-to-desktop temporary transfer over the local network:

- Send text from a phone browser to the desktop.
- Send photos, take a photo, send videos, and send generic files from a phone browser to the desktop.
- Use a short-lived local-network QR session token.
- Provide basic English and Simplified Chinese UI.
- Provide a Windows preview build.
- Avoid accounts, cloud services, transfer history, and telemetry.

## Next DropLite Directions

Possible future DropLite work:

- Desktop to phone transfer.
- Better receive and download flow on mobile.
- More polished settings.
- Optional keep-original-filename behavior.
- Better onboarding and diagnostics.
- Cross-platform packaging for macOS and Linux.
- GitHub Actions release pipeline.
- Code signing research.
- Auto-update research.

No timeline is promised. Future work should keep DropLite focused, temporary, local-first by default, and honest about security boundaries.

## Future Tool Directions

These are possible directions, not completed features:

- Quick OCR for local image-to-text extraction.
- Image Compressor for simple local image optimization.
- PDF Toolbox for focused, privacy-aware PDF operations.
- Small conversion tools for everyday file and text tasks.

Future tools should follow the same standard: focused, local-first where practical, documented, tested, and honest about tradeoffs.
