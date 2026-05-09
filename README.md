# Easy-Use

Easy-Use is an open source collection of tiny, focused, high-quality tools. Each tool is designed to solve one clear everyday problem without turning into a heavy all-in-one product.

## Vision

Easy-Use exists to make practical utilities feel simple, trustworthy, and pleasant. The project favors local-first behavior, honest security boundaries, no forced accounts, and careful product polish over feature sprawl.

## Tools

| Tool | Status | Purpose |
| --- | --- | --- |
| [DropLite](tools/droplite/README.md) | v0.1.0 Preview | Send text, photos, videos, and files from a phone browser to a desktop over the local network. |

## DropLite

DropLite is the first Easy-Use tool. It is a minimal, account-free, local-network-first temporary drop tool for moving content from a phone to a computer.

Open DropLite on the computer, scan the QR code with a phone, then send text, photos, videos, or files through the phone browser. No phone app, no cloud service, no account, and no transfer history.

## Design Principles

- One tool solves one clear problem.
- Default to simple workflows.
- Do not force login for local utility use cases.
- Prefer local-first behavior where practical.
- Avoid meaningless complexity and heavy dependencies.
- Make safety boundaries clear instead of overselling them.
- Treat documentation, tests, and maintainability as product features.

See [Tool Design Principles](docs/tool-design-principles.md) for the full project standard.

## Development

The root of this repository contains the Easy-Use project documentation and tool index. Individual tools live under `tools/`.

Before starting larger changes, create a branch from `master`. See [Git Workflow](docs/git-workflow.md).

Codex and maintainer guidance:

- [AGENTS.md](AGENTS.md): repository-level Codex/project instructions.
- [tools/droplite/AGENTS.md](tools/droplite/AGENTS.md): DropLite-specific Codex instructions.
- [tools/droplite/docs/PLANS.md](tools/droplite/docs/PLANS.md): active DropLite implementation plan.

To work on DropLite:

```bash
cd tools/droplite
npm install
npm run tauri:dev
```

## Contributing

Issues, documentation improvements, tests, and focused tool ideas are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

## License

Easy-Use is released under the [MIT License](LICENSE).
