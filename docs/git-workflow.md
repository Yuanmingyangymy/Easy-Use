# Git Workflow

Easy-Use keeps `master` stable so released tools remain easy to test, build, and share.

## `master` Branch

- `master` should always be usable.
- Code on `master` should pass the relevant tests.
- `master` should represent the latest usable version or a release candidate state.
- Do not develop large features directly on `master`.

Small documentation or release housekeeping changes may land on `master` when they are low risk and easy to review.

## Branch Names

Use short, scoped branch names:

- `feat/<scope>-<short-description>`
- `fix/<scope>-<short-description>`
- `docs/<scope>-<short-description>`
- `chore/<scope>-<short-description>`
- `release/<version>` when a dedicated release branch is useful

Examples:

- `feat/droplite-desktop-to-phone`
- `fix/droplite-mobile-upload`
- `docs/droplite-release-notes`
- `chore/droplite-ci-cleanup`

## Development Flow

1. Start from the latest `master`.
2. Create a focused branch.
3. Develop and commit on that branch.
4. Run the relevant local tests.
5. Confirm the user-facing behavior when the change affects UI or transfer flow.
6. Push the branch.
7. Open a pull request into `master`.
8. Merge only after the branch is reviewed and `master` remains buildable.

Avoid mixing unrelated refactors, docs, and feature work in one pull request.

## Checks Before Merging

For DropLite changes, run at least:

```bash
cd tools/droplite
npm test
npm run typecheck
```

```bash
cd tools/droplite/src-tauri
cargo test
```

For important release work, also run:

```bash
cd tools/droplite
npm run tauri build
```

If a command cannot run in the current environment, document why in the pull request and run it in a suitable local or CI environment before merging risky changes.

## Release Flow

1. Validate `master`.
2. Create a version tag from the validated commit.
3. Push `master` and the tag.
4. Create a GitHub Release.
5. Upload only release assets, such as installers and `SHA256SUMS.txt`.

Tag examples:

- `v0.1.0-preview.1`
- `v0.1.1-preview.1`
- `v0.2.0-preview.1`

Do not commit installers, `target/`, `dist/`, `build/`, `node_modules/`, `SHA256SUMS.txt`, received files, or temporary `.part` files.

## Rollback

- If `master` has a serious problem, return to the most recent known-good tag.
- Treat tags as published snapshots. Do not move a published tag casually.
- Do not force push `master` unless the maintainers explicitly agree that it is necessary.
