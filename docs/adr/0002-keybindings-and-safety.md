# ADR 0002: Keybindings are TOML-configurable and safety-first

## Status

Accepted

## Context

The UI should feel vim-like by default while remaining configurable and predictable. History-rewrite
and remote commands require stronger safeguards than read-only commands.

## Decision

- Use alt-screen + raw mode for an immersive TUI loop.
- Load keybindings from a default TOML file, optionally overridden by user TOML.
- Use mode-aware key precedence: input fields, overlays, then normal-mode bindings.
- Gate dangerous commands behind explicit confirmation with command preview.

## Consequences

- Defaults are fast for power users and still user-overridable.
- Keybinding behavior is deterministic and testable.
- Risky commands are less likely to execute accidentally.
