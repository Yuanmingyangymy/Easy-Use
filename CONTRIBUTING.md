# Contributing to Easy-Use

Thank you for helping make small tools better.

## What Fits This Project

Easy-Use welcomes focused utilities that solve a clear user pain. A good tool is small, understandable, documented, tested, and honest about what it does not do.

Before proposing a new tool, please consider:

- What single problem does it solve?
- Can it stay useful without accounts, tracking, or unnecessary cloud services?
- Can its safety and privacy boundaries be explained simply?
- Will it remain maintainable as an open source project?

## Issues

Use issues for bug reports, focused feature requests, documentation gaps, and security-adjacent questions that do not expose sensitive details.

Please include:

- The tool name and version or commit.
- Your operating system and environment.
- Steps to reproduce for bugs.
- Expected and actual behavior.
- Screenshots or logs when useful.

## Pull Requests

Pull requests should be narrow and reviewable. Avoid mixing unrelated refactors with feature work.

Before opening a PR:

- Keep the change scoped to one tool or one documentation area.
- Add or update tests for behavior changes.
- Update README or docs when user-facing behavior changes.
- Avoid adding heavy dependencies without a clear reason.
- Do not introduce accounts, telemetry, ads, or cloud services unless the tool explicitly requires it and the docs explain the tradeoff.

## Code Quality

- Prefer clear module boundaries over clever abstractions.
- Handle errors deliberately.
- Keep user-facing messages short and useful.
- Do not store more data than the tool needs.
- Document security boundaries honestly.

## Documentation

Docs are part of the product. Every tool should explain what it does, what it does not do, how to run it locally, how to test it, and what safety assumptions it makes.
