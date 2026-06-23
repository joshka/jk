# Searchable Command Discovery

Status: draft

Owner: `vibe-searchable-command-discovery` workspace spike

Scope: first searchable help and command-discovery implementation chunk

## Problem

`jk` now has the foundations needed to make command discovery useful:

- command specs can describe read-only `jj` commands and titles;
- keymap metadata generates contextual hotbar and help text;
- a view stack and mode stack can host transient prompts;
- log, diff, show, and status inspection views exist;
- the log template popup selector proves centered overlay selection behavior.

The next roadmap slice should make `?` help searchable before adding `:` command mode and
workspaces. Users should be able to ask "what can I do here?" and quickly find actions by key,
action name, current screen, and `jj` command family. This must not execute commands yet and must
not replace the future `:` command mode.

## Goals

- Enhance `?` from static contextual help into a searchable discovery overlay.
- Search current-screen actions by key text, action name, help text, screen, and `jj` command
  family tags.
- Include contextual rows for log, diff, show, and status/inspection screens.
- Reuse existing keymap metadata and help overlay patterns where practical.
- Keep command execution, command parsing, and command history out of this slice.
- Keep the implementation internal to `jk-tui` and `jk` unless a current caller needs a public API.
- Produce tests and Betamax evidence that prove the discovery interaction, not just rendering.

## Non-Goals

- Do not add `:` command mode.
- Do not add `!` external command mode.
- Do not execute a selected action from the discovery overlay in the first slice.
- Do not add user-configurable keymaps.
- Do not add workspace commands or workspace screens.
- Do not build a general fuzzy-finder framework.
- Do not fetch or parse `jj help` output at runtime.
- Do not move command specs, inspection queries, or view stack state into new public crates.
- Do not change existing key meanings outside the help/discovery overlay.

## Current Dependencies

This plan assumes the current spike state rather than only the earlier written plans:

1. `0001-command-spec.md`: `JjCommandSpec` exists for read-only `jj` command metadata.
1. `0002-keymap-help-data.md`: `crates/jk-tui/src/keymap.rs` owns generated help and hotbar
   binding rows.
1. `0003-view-stack-foundation.md`: `crates/jk/src/main.rs` owns `ViewStack`, `ModeStack`, and
   transient input modes.
1. `0004-inspection-foundation.md`: diff/show/status query sources and pushed inspection views
   exist in the current spike.
1. `0005-log-template-selection.md`: the log template popup selector provides a nearby overlay and
   selection precedent.

The searchable discovery slice should extend those pieces instead of replacing them.

## User Experience

Pressing `?` opens a searchable command-discovery overlay for the active context. The first slice
can keep the overlay visually close to the existing help popup: centered, bordered, black
background, and rendered above the active screen.

The overlay should include:

- a title such as `Command discovery`;
- a filter prompt row, for example `? diff`;
- rows with key text, action label, screen/context, and optional `jj` family;
- a short footer with overlay controls.

Initial row examples:

```text
> d      Open diff              log          jj diff
  enter  Open show              log          jj show
  s      Open status            log          jj status
  /      Search output          diff         search
  T      Switch log template    log          jj log
  r      Refresh                current      refresh
```

Exact spacing can change during implementation, but rows should stay scan-friendly on a normal
80-column terminal. On narrow terminals, the rightmost metadata should clip before key or action
labels become unreadable. Keep the first slice current-context only: opening help from the log shows
log actions, opening help from diff shows diff actions, and opening help from show/status shows
inspection actions.

## Search Behavior

Filtering should be simple, deterministic, and case-insensitive.

Each row should match when every whitespace-separated query token appears in at least one searchable
field. Searchable fields should include:

- `keys`, such as `d`, `enter`, `?`, or `Ctrl-j/k`;
- action label, such as `open diff`;
- help text, such as `open selected-change diff`;
- context label, such as `log`, `diff`, `show`, `status`, or `inspection`;
- command family tags, such as `jj log`, `jj diff`, `jj show`, or `jj status`;
- optional aliases, such as `details` for show or `current screen` for contextual rows.

Examples:

- `d` matches rows that expose key `d` and rows whose text contains `d`;
- `diff` matches log `d` and diff navigation rows tagged `jj diff`;
- `jj show` matches the log `enter` row and show/inspection rows tagged `jj show`;
- `status s` matches status actions and the log `s` row;
- `template log` matches `T switch log template`.

Do not add scoring in the first slice. Keep existing row order after filtering.

## Key Behavior

The discovery overlay is an input mode. While it is active:

- printable character input appends to the filter;
- `Backspace` removes one filter character when the filter is non-empty;
- `Backspace` closes the overlay when the filter is empty;
- `Esc` closes the overlay;
- `q` closes the overlay;
- `?` closes the overlay;
- up/down arrows and `j/k` move the highlighted row when at least one row exists;
- row movement clamps at the first and last visible rows;
- `Enter` closes the overlay without executing anything;
- page keys can be ignored in the first slice unless overlay height requires scrolling.

`Enter` intentionally behaves like "done reading" in the first slice. It does not execute the
highlighted action, does not push a view, and does not run a `jj` command. That keeps searchable
help distinct from the future `:` command mode while still making keyboard dismissal predictable.

The global `?` key should keep working from log, diff, show, and status contexts. Existing
`q`/`Esc` close-help-before-quit behavior should remain true through the new mode.

## Data Model

Extend the current TUI-local keymap metadata rather than introducing a separate registry.

Suggested shape in `crates/jk-tui/src/keymap.rs`:

```rust
pub(crate) struct DiscoveryRow {
    pub keys: &'static str,
    pub action: &'static str,
    pub help: &'static str,
    pub context: BindingContext,
    pub command_family: Option<CommandFamily>,
    pub aliases: &'static [&'static str],
}

pub(crate) enum CommandFamily {
    JjLog,
    JjDiff,
    JjShow,
    JjStatus,
    Search,
    Refresh,
    Navigation,
}
```

The exact names can change, but the registry should expose enough data for both static help lines
and searchable rows. Avoid duplicating the same action text in a second table.

`KeyBinding` should gain metadata similar to:

- action label, for example `Open diff`;
- command family, optional;
- aliases, optional;
- whether the binding is relevant to discovery;
- whether the binding is only contextual to one active view.

Rows should be built from the active context only. The filter must still expose the current context
and command-family fields. A small follow-up can add "related but inactive" rows if usage shows that
users expect cross-screen discovery.

## Context Ownership

The source of truth should remain split the same way it is today:

- `crates/jk-tui/src/keymap.rs` owns visible binding metadata and discovery filtering helpers;
- `crates/jk-tui/src/chrome.rs` owns generic centered overlay rendering;
- log/diff/rendered views own only enough state or rendering hooks to display the overlay;
- `crates/jk/src/main.rs` owns terminal key dispatch and transient `InputMode` state;
- `crates/jk-cli` and `jk-core` should not change for this slice.

Suggested new or changed internals:

- add `DiscoveryState` or `InputMode::CommandDiscovery { context, query, selected }` in `main.rs`;
- add `discovery_rows(context)` and `filter_discovery_rows(context, query)` in `keymap.rs`;
- add a reusable `render_discovery_overlay` helper in `chrome.rs` or a small new TUI module;
- add render methods that can draw a caller-owned discovery overlay on top of an active view.

The current log-template selector already renders caller-owned popup lines through active views.
Searchable discovery can start with the same shape: the application owns query and selected-index
state, `jk-tui` owns filtered row construction and popup line formatting, and each active view only
needs a render hook that layers the popup over its existing content.

Keep `BindingContext` internal unless the binary crate needs to ask the active view for it. If the
binary needs the context, prefer a small public method or public enum from `jk-tui` over leaking the
whole binding registry.

## Public API Decision

No new public crate API is required by default. This can stay internal to `jk-tui` and `jk` because
the binary already owns dispatch and the TUI crate already owns visible help metadata.

Only make APIs public when they cross the existing crate boundary:

- `BindingContext` may need `pub` visibility if `main.rs` selects discovery rows directly;
- discovery row structs can remain `pub(crate)` if rendering stays inside `jk-tui`;
- `LogAction`, `DiffAction`, and `RenderedAction` should not gain execute-from-help variants in
  the first slice.

Do not move discovery metadata into `jk-core` yet. It is UI metadata, not a durable command model.

## Implementation Chunks

### Chunk 1: enrich keymap metadata

Files:

- `crates/jk-tui/src/keymap.rs`

Acceptance:

- each binding row has a stable action label suitable for discovery;
- log `enter`, `d`, `s`, `T`, `H/L`, and `r` have command-family or alias metadata;
- diff rows expose `jj diff`, search, navigation, fold, file, and hunk metadata;
- inspection rows expose show/status-compatible search, navigation, refresh, and return metadata;
- filtering is case-insensitive and token-based;
- static `help_lines` and `hotbar` output remain unchanged.

### Chunk 2: render a searchable discovery overlay

Files:

- `crates/jk-tui/src/chrome.rs`
- `crates/jk-tui/src/keymap.rs`
- optionally a new `crates/jk-tui/src/discovery.rs`
- `crates/jk-tui/src/lib.rs` only if a new module must be exported inside the crate

Acceptance:

- the overlay renders a prompt, filtered rows, current selection marker, and footer;
- empty filters show the active context rows;
- zero-result filters render an intentional empty state;
- narrow terminals clip or wrap predictably without panicking;
- existing help overlay tests still pass or are updated to the new discovery contract.

### Chunk 3: wire `?` to discovery mode

Files:

- `crates/jk/src/key.rs`
- `crates/jk/src/main.rs`
- `crates/jk-tui/src/log_view.rs`
- `crates/jk-tui/src/diff_view.rs`
- `crates/jk-tui/src/rendered_view.rs`

Acceptance:

- `?` opens command discovery from log, diff, show, and status contexts;
- typing filters visible rows;
- up/down and `j/k` move the selected row when rows exist;
- `Backspace` deletes filter text, then closes the overlay when empty;
- `Esc` and `q` close the overlay;
- `Enter` closes or no-ops without executing an action;
- existing search prompts and log template selector still take precedence when active;
- no command execution occurs from discovery.

Stop there for the first review unit.

## Acceptance Criteria

The first implementation is complete when these user-visible behaviors work:

- pressing `?` on the log opens searchable discovery instead of a static-only key list;
- typing `diff` on the log narrows to diff-related rows, including `d open diff`;
- typing `show` on the log shows the `enter open show` row;
- typing `status` on the log shows the `s open status` row;
- pressing `?` in a diff shows diff-context navigation, fold, file, hunk, search, refresh, and
  return rows;
- pressing `?` in show/status inspection shows inspection-context search, navigation, refresh, and
  return rows;
- arrow keys and `j/k` move the highlighted row when filtered rows exist;
- `Backspace`, `Esc`, `?`, and `q` cancel in the specified ways;
- `Enter` does not execute a command;
- `:` command mode remains unimplemented and unaffected;
- `jk-cli` command execution paths are unchanged.

## Tests

Add focused unit tests before relying on Betamax evidence.

Required `jk-tui` tests:

- discovery rows are generated from binding metadata for each `BindingContext`;
- filter matching is case-insensitive;
- multi-token filtering requires every token to match;
- key, action label, context, alias, and command-family fields are searchable;
- `help_lines` and `hotbar` remain stable after metadata enrichment;
- overlay rendering shows prompt text, rows, selection marker, and zero-result text.

Required `jk` tests:

- `?` enters discovery mode from log, diff, show, and status root views;
- printable keys update the discovery filter;
- `Backspace` deletes one filter character before closing the overlay;
- `Esc` and `q` close discovery;
- `j/k` and arrows move selection only when rows exist;
- `Enter` closes or no-ops without pushing a view or running a command.

Suggested validation:

```sh
cargo test -p jk-tui keymap
cargo test -p jk-tui discovery
cargo test -p jk
just lint-md
```

## Betamax Evidence Expectations

Add assertion-first validation evidence, not README or website media.

Minimum tape coverage:

- start in log, press `?`, type `diff`, assert the overlay includes the selected-change diff row;
- clear or reopen discovery, type `show`, assert the selected-change show row appears;
- start in diff, press `?`, type `file`, assert file navigation rows appear;
- start in show or status, press `?`, type `jj status` or `search`, assert contextual rows appear;
- press `j` or down arrow, assert the selection marker moves;
- press `Backspace` and `Esc`, assert the overlay closes without changing the underlying view;
- press `Enter` on a row and assert no command execution or view push occurs.

Use `tapes/validation/` if that taxonomy exists when the implementation lands. Otherwise add the
smallest local tape beside the existing `tapes/jk-log.tape` and `tapes/jk-diff.tape` patterns. Do
not generate README, crates.io, website, or release-note media in this slice.

## Risks

- Treating discovery as command mode would over-scope the slice. Keep it read-only and make
  execution a later decision.
- Duplicating action labels outside `keymap.rs` would reintroduce help drift. Enrich the existing
  binding rows instead.
- Showing inactive global rows may confuse users if pressing the key would do nothing in the active
  screen. Start with active-context rows and add related rows later only with clear labeling.
- Search scoring can hide deterministic behavior and create review churn. Token filtering is enough
  for the first slice.
- `Enter` can imply execution. The footer and tests must make the first-slice behavior explicit.
- The current `LogAction` is reused as a broad action adapter for multiple views. Avoid using this
  slice to redesign that adapter unless discovery cannot be wired cleanly otherwise.

## Follow-Ups

- Add related inactive rows with labels such as `available from log` after the active-context
  discovery is proven.
- Let `Enter` explain or preview a row once command preview UI exists.
- Connect discovery rows to `JjCommandSpec` previews after command history and command mode have a
  shared command surface.
- Add `:` command mode with optional `jj` prefix as a separate roadmap slice.
- Add workspace command discovery once the workspace screen and actions exist.
- Generate reference docs from the same metadata after key configuration is in scope.
