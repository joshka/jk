# ADR 0003: Command registry mirrors jj and drives routing

## Status

Accepted

## Context

`jk` aims to be a TUI layer for `jj`, not an unrelated command set. Without an explicit command
registry, feature work drifts: command-mode help, safety behavior, and actual flow routing diverge
over time.

This project also needs clear guardrails for risky operations (`rebase`, `git push`, operation
restore/revert), while keeping fast paths for read-only commands.

## Decision

- Maintain an explicit top-level command registry aligned to current `jj --help` output.
- Annotate each command with:
  - execution mode (`native`, `guided`, `passthrough`, `defer`);
  - safety tier (`A`, `B`, `C`).
- Use the registry for in-app command coverage views (`:commands` / `:help`) and as the default
  safety fallback.
- Allow subcommand safety overrides where risk differs from top-level command defaults (for example
  `git push` and `operation restore`/`revert`).

## Consequences

- Command coverage remains inspectable and testable as `jj` evolves.
- Safety behavior is consistent for startup commands and in-session command mode.
- New command work can start by updating one table and associated tests before UI polish.
- Users get predictable confirmations for risky actions and fast execution for low-risk actions.
