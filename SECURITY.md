# Security Policy

Easy-Use is a collection of local-first open source tools. Each tool must document its own security model, assumptions, and limits.

## Reporting a Vulnerability

If you find a vulnerability, please open a private security advisory if the repository host supports it. If private reporting is not available, contact the maintainers through the project issue tracker with a minimal public description and offer to share sensitive details privately.

Please include:

- The affected tool and version or commit.
- A short impact summary.
- Reproduction steps or proof-of-concept details.
- Any known mitigations.

## Project-Wide Expectations

- Tools should avoid collecting data they do not need.
- Local-first tools should clearly state when data stays local and when it does not.
- Authentication, encryption, and network boundaries must not be overstated.
- Security-sensitive code should include focused tests where practical.

## Tool-Specific Security

DropLite documents its MVP security model in [tools/droplite/docs/security-model.md](tools/droplite/docs/security-model.md).
