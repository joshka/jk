# jk

`jk` is a pager-driven TUI for `jj` workflows. It starts from the familiar `jj log` experience and
turns that screen into the command hub for day-to-day repository navigation and actions.

## Status

This repository is in early development. The TUI behavior described here is the intended design
target, not a complete implementation yet.

## Vision

- Default view matches `jj log` output and config as closely as possible.
- TUI commands are thin, interactive wrappers around existing `jk` command behavior and flags.
- Interaction stays terminal-native: minimal chrome, minimal panes, minimal visual noise.
- The log is not just output; it is the jumping-off point for inspect, diff, rebase, bookmark, and
  workflow actions.

## Design Principles

### 1. Zero-surprise defaults

If you already know `jj log`, you should recognize `jk` immediately. Templates, colors, and revset
semantics should come from the same config sources by default.

### 2. Pager-first interaction

The UI should feel closer to a powerful pager than a dashboard app. Prefer focused prompts and
transient overlays over persistent multi-panel layouts.

### 3. Command parity

For each `jk` command, the TUI variant should keep parameter names and meanings aligned whenever
possible. Interactive affordances should reduce typing, not change command semantics.

### 4. Shared design language

CLI and TUI should read as one tool: same naming, same mental model, same defaults, same outcomes.

## Command Model

The expected interaction model is:

1. Open a log-centric screen that behaves like `jj log`.
2. Move selection through commits and operations.
3. Trigger command actions from the current selection.
4. Pass through familiar flags/revsets/options where applicable.

Planned convention:

- Non-interactive command: `jk <command> [flags...]`
- Interactive command: `jk tui <command> [flags...]`
- In-session actions use the same command names as the CLI variants.

## Configuration

By default, `jk` should inherit relevant `jj` config for:

- log templates
- color and style behavior
- revset aliases
- user/repo defaults that affect command results

`jk`-specific config should be additive and scoped to interaction behavior (keys, prompts, and
layout density), not semantic differences in command results.

## First Milestone (Draft)

- Build a log view that matches `jj log` content and ordering.
- Add keyboard navigation and selection state.
- Add action dispatch from the selected change/operation.
- Implement a first set of TUI command wrappers with shared parameters.
- Confirm config compatibility with common `jj` setups.

## Development

### Build

```bash
cargo build
```

### Run

```bash
cargo run
```

As implementation grows, this README will be updated with concrete key bindings, command matrices,
and end-to-end usage examples.
