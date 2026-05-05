# Tool Design Principles

## One Tool, One Problem

Each Easy-Use tool should solve one clear problem. The best version of a tool is usually the smallest version that fully handles the real workflow.

## Default to Minimal

Minimal does not mean unfinished. It means the main path is clear, the interface is quiet, and secondary options do not bury the thing the user came to do.

## No Forced Login

Local utility workflows should not require accounts by default. If a future tool truly needs identity, the reason must be clear and documented.

## Local First When Practical

Prefer local processing and local transfer when it fits the problem. If a tool uses cloud services, it must say so plainly and explain why.

## No Meaningless Complexity

Avoid feature lists that only make the project harder to understand. Do not add chat, sync, dashboards, accounts, or background services unless they are central to the tool.

## User Experience Comes First

The interface should feel focused and respectful. Good defaults, fast startup, useful errors, and clear empty states matter.

## Honest Security Boundaries

Security claims must match the implementation. If a tool uses local HTTP, say local HTTP. If traffic is not encrypted, say so. Users should know when a public or untrusted network is a bad idea.

## Maintainability Is Product Quality

Open source tools need readable code, tests for core behavior, structured documentation, and conservative dependency choices.
