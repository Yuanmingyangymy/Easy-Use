# DropLite Plans

## Active Plan: v0.2 Desktop to Phone Transfer

Branch:

```text
feat/droplite-desktop-to-phone
```

Goal:

Add desktop -> phone temporary transfer while preserving the stable v0.1 phone -> desktop flow.

## Phase Status

### Phase 1: Outbox Backend Model

Status: implemented

Summary:

- In-memory `OutboxItem` model.
- Token-protected outbox APIs.
- Safe backend-only file references.
- No local absolute paths returned to phone.

### Phase 2: Desktop Send UI

Status: implemented

Summary:

- Desktop `Send to phone` panel.
- Add text to outbox.
- Choose files.
- Drag files.
- Outbox list on desktop.

### Phase 2.5: Configurable Receive Folder

Status: implemented

Summary:

- Allow user to change phone -> desktop receive directory.
- Persist setting locally.
- Open and reset folder.
- Do not mix `receive_dir` with outbox source files.

### Phase 3: Mobile Receive UI

Status: implemented

Summary:

- Phone polls `/api/outbox`.
- Show desktop-sent text/file cards.
- Copy text.
- Download files.
- Handle expired session.
- Keep mobile UI simple.
- Improved after real phone QA with mobile tabs, download fallback guidance, and outbox item de-duplication.

### Phase 4: Reliability and Regression QA

Status: in progress

Summary:

- v0.1 regression.
- First upload reliability.
- Desktop -> phone download reliability.
- iOS/Android browser behavior.
- Windows firewall/private network docs.
- WeChat in-app browser download fallback guidance.
- Manual QA checklist for bidirectional v0.2 validation.
- Manual QA details live in [`v0.2-manual-qa.md`](v0.2-manual-qa.md).

### Phase 5: v0.2 Preview Release

Status: planned

Summary:

- Tests.
- Build.
- Installer validation.
- Release notes.
- Tag and GitHub Release.

## Do Not Do In v0.2

- cloud relay
- accounts
- public sharing links
- WebRTC
- database
- permanent history
- phone app
- chat system
- multi-device contact list
