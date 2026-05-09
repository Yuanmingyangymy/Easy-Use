# AGENTS.md - DropLite

## Product Identity

DropLite is an Easy-Use tool for temporary cross-device transfer. It should remain simple, fast, local-first, and privacy-conscious.

## Core Principles

- No account.
- No cloud.
- No tracking.
- No ads.
- Local network first.
- Temporary sessions.
- No persistent transfer history.
- Be honest about security boundaries.

## Current Stable Release

DropLite v0.1.0 Preview supports phone browser -> desktop transfer:

- text
- photos
- take photo and send
- videos
- generic files
- QR token session
- timestamped filenames
- configurable language
- Windows preview installer

Do not break this v0.1 flow.

## Current Development Branch

v0.2 desktop -> phone work is on:

```text
feat/droplite-desktop-to-phone
```

## v0.2 Direction

The v0.2 goal is desktop -> phone transfer:

- Desktop can add text/files to outbox.
- Phone can poll outbox.
- Phone can copy text or download files.
- Continue using local HTTP + token.
- Do not introduce WebRTC yet.
- Do not introduce a database.
- Do not introduce cloud relay or public sharing.

## Security Rules

- All LAN HTTP APIs must validate the session token.
- Expired tokens must be rejected.
- Refresh session invalidates old token and clears or invalidates session-scoped data.
- Never expose local absolute paths to the phone.
- Download APIs must only serve registered outbox items.
- Prevent path traversal.
- Sanitize filenames used in `Content-Disposition`.
- Do not log full tokens or transferred content.

## Receive Directory Rules

- The receive directory is for phone -> desktop files.
- The outbox is for desktop -> phone files.
- Do not mix these concepts.
- User-configured receive directory must be persisted locally only.
- Do not upload or expose receive directory paths.

## UI Rules

- Keep UI simple and polished.
- Avoid turning DropLite into a chat app.
- Avoid complex settings pages unless necessary.
- Maintain English and Simplified Chinese i18n.
- Do not hardcode major UI strings.
- Mobile UI must stay small-screen friendly.

## Test Commands

For DropLite changes, run when applicable:

```bash
cd tools/droplite
npm test
npm run typecheck

cd tools/droplite/src-tauri
cargo test
```

Run `npm run tauri:dev` when UI, Tauri commands, capabilities, or runtime behavior changed.

Run `npm run tauri build` only for release/build-related changes.

## Documentation

Update these when behavior changes:

- `tools/droplite/README.md` for released user-facing behavior
- `tools/droplite/docs/architecture.md`
- `tools/droplite/docs/security-model.md`
- `tools/droplite/docs/roadmap.md`
- `tools/droplite/docs/v0.2-desktop-to-phone-design.md` for v0.2 work
- `tools/droplite/docs/PLANS.md` for active implementation plan status

Do not claim unreleased v0.2 features are available in the v0.1 release.
