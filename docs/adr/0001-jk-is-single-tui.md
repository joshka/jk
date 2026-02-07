# ADR 0001: `jk` is a single log-first TUI

## Status

Accepted

## Context

The product goal is to keep users inside `jk` for most version-control tasks, with log navigation as
its home context. A separate `jk tui` subcommand would duplicate entrypoints and split mental models.

## Decision

- `jk` is the primary command and enters the TUI directly.
- `jk log` is an explicit alias of the same entry behavior.
- `jk <command> [args...]` starts the same TUI, preloading a command flow.

## Consequences

- Users learn one entrypoint and one interaction style.
- Command parity with `jj` remains visible without leaving the app.
- CLI/TUI drift risk is reduced because one runtime owns both behaviors.
