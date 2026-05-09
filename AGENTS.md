# AGENTS.md - Easy-Use

## Project Identity

Easy-Use is an open-source toolkit. Each tool should solve one clear user problem with a simple, polished, local-first experience where possible.

## Repository Structure

- Root files describe the Easy-Use toolkit.
- Individual tools live under `tools/<tool-name>/`.
- DropLite lives under `tools/droplite/`.
- Do not place tool implementation files directly in the repository root.

## Git Workflow

- `master` is the stable release baseline.
- Do not develop large features directly on `master`.
- Use `feat/*`, `fix/*`, `docs/*`, or `chore/*` branches.
- Current DropLite v0.2 work happens on `feat/droplite-desktop-to-phone`.
- Do not force push `master`.
- Do not move published tags.
- Use clear, scoped commits.
- Push the current feature branch after completing a task if credentials are available.

## Quality Expectations

- Keep code maintainable.
- Avoid unnecessary dependencies.
- Update docs when behavior changes.
- Add or update tests for meaningful changes.
- Do not remove tests to make the suite pass.
- Do not hide errors by using broad `any`, `unwrap`, or equivalent shortcuts without a good reason.
- Prefer existing project patterns over new abstractions.

## Git Ignore / Artifacts

Never commit:

- `node_modules`
- `target`
- `dist`
- `build`
- installers
- release artifacts
- `SHA256SUMS.txt`
- received test files
- `.part` temporary files
- local config files
- secrets or tokens

## Release Rules

- Source code goes to Git.
- Installers and `SHA256SUMS.txt` are GitHub Release assets, not repository files.
- Tags represent release snapshots.
- Previews should be marked as pre-release.
- Do not push, tag, or publish releases unless the user explicitly asks.

## Communication

At the end of each task, report:

- current branch
- files changed
- tests run and results
- commit hash
- push status
- known limitations
- next recommended step
