# Keymap Help Data Foundation

Status: draft  
Owner: `vibe` workspace spike  
Scope: first keymap/help implementation chunk

## Problem

`jk` currently has three separate sources of keybinding truth:

- `crates/jk/src/key.rs` maps terminal keys to actions;
- `crates/jk-tui/src/chrome.rs` stores hard-coded status hotbar text;
- `crates/jk-tui/src/log_view.rs` and `crates/jk-tui/src/diff_view.rs` store hard-coded help rows.

That works for the current log and diff views, but it will drift as command discovery, view stack,
mode stack, command previews, and workspaces are added. The product plan calls for contextual help
and hotbar text generated from the active keymap.

## Goals

- Introduce a small data model for visible keybinding metadata.
- Generate current log and diff hotbar/help text from the same binding rows.
- Preserve current key dispatch behavior.
- Keep the change small enough to review independently.
- Make the next searchable-help slice possible without moving user config into scope.

## Non-Goals

- Do not add user-configurable keymaps.
- Do not add searchable help or command mode.
- Do not change terminal key dispatch semantics.
- Do not introduce view stack or mode stack.
- Do not add a new crate.

## Proposed Shape

Add a TUI-local key help module:

```rust
pub struct KeyBindingHelp {
    keys: &'static str,
    summary: &'static str,
    hotbar: bool,
}

impl KeyBindingHelp {
    pub const fn new(keys: &'static str, summary: &'static str) -> Self;
    pub const fn hotbar(self) -> Self;
    pub fn help_line(&self) -> String;
}
```

The TUI crate should own visible help text because it owns the view contracts and rendered chrome.
The binary crate can keep owning `crossterm` dispatch for now.

Suggested file ownership:

- `crates/jk-tui/src/key_help.rs`: metadata type plus formatting helpers.
- `crates/jk-tui/src/chrome.rs`: render hotbar text from `KeyBindingHelp` rows.
- `crates/jk-tui/src/log_view.rs`: replace `LOG_HELP` and `LOG_STATUS` with generated text.
- `crates/jk-tui/src/diff_view.rs`: replace `DIFF_HELP` and `DIFF_STATUS` with generated text.

## First Implementation Chunk

Acceptance:

- Log help overlay still contains the same user-facing actions.
- Diff help overlay still contains the same user-facing actions.
- Log and diff status rows are generated from binding rows marked as hotbar-visible.
- Tests prove changing a binding row changes both help and hotbar output through one data source.
- `cargo check` and targeted `jk-tui` tests pass.

Keep formatting simple. Help rows can keep fixed-width key columns for now because the current
overlay depends on compact scan-friendly rows. The model only needs to make the source shared.

## Follow-Up Chunks

- Move binary key dispatch toward shared action metadata after the visible help source is stable.
- Add command-family tags for searchable help.
- Add screen identifiers so the help surface can filter by active view.
- Add generated keymap docs after the binding registry is broad enough to be useful.

## Risks

- Moving dispatch and help at the same time would be easy to over-scope. This chunk should only
  unify visible metadata.
- The current log and diff views share many bindings but not all semantics. Prefer separate binding
  rows per view until a shared registry removes real duplication without hiding differences.
- Hotbar text must remain short enough for normal terminals; generated text should not blindly show
  every binding.
