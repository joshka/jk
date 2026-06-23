# Adaptive Hotbar Slice

Status: implemented in the current dogfood work

Owner: current dogfood workspace pass

Scope: first width-aware hotbar/status implementation chunk

## Problem

Recent inspection slices added more high-value commands to the log hotbar:

- `enter show`;
- `d diff`;
- `v evolog`;
- `s status`;
- `V options`;
- mark and clear-mark commands.

The visible hotbar is now generated from `crates/jk-tui/src/keymap.rs`, but
`crates/jk-tui/src/chrome.rs` still renders the generated status text as one fixed line. At normal
Betamax widths, the log hotbar clips on the right edge. That can hide `q quit`, and it makes the
footer feel broken exactly where the product plan says contextual help and discovery should remain
visible.

This is a presentation problem, not a keymap problem. The next slice should make the existing
hotbar adapt to terminal width using the current keymap metadata and ranks.

## Decision

Add a small adaptive hotbar formatter that chooses which ranked keymap labels fit in the available
status-row width.

The first slice should:

- keep the existing key meanings;
- keep `? help` and `q quit` visible whenever the terminal can fit them;
- preserve high-value context commands before low-value movement reminders;
- use an explicit overflow marker when lower-priority labels are omitted;
- keep full discovery available through `?`;
- avoid moving dispatch, user-configurable keymaps, command mode, or workspace UI into scope.

The implementation should make the footer intentionally shorter on narrow terminals instead of
letting Ratatui clip a long string.

## Context

This plan assumes the current dogfood workspace state:

1. `crates/jk-tui/src/keymap.rs` owns `BindingContext`, hotbar labels, `hotbar_rank`, generated
   help rows, and searchable discovery rows.
1. The log context now includes selected-change `show`, `diff`, `evolog`, `status`, View Options,
   marks, refresh, home/log switching, help, and quit.
1. Diff and generic inspection contexts already have shorter hotbars, but they should use the same
   adaptive formatter so future diff, evolog, show, status, and operation views do not repeat this
   bug.
1. `crates/jk-tui/src/chrome.rs` renders the status row from a caller-provided string and does not
   know the terminal width when the string is built.
1. Betamax artifacts for selected-change evolog show an inspection-width hotbar that fits, while
   the expanded log hotbar is the risky case after `v evolog` lands.

The product plan calls for contextual hotbars and generated help as discovery mechanisms. The
adaptive formatter should support that direction by making the visible footer honest about what
fits while keeping the complete command list one keypress away.

## Goals

- Make hotbar/status text fit the status row at normal and narrow terminal widths.
- Use the keymap hotbar ranks as the source of display order and omission priority.
- Always reserve space for a quit/help affordance before optional labels.
- Keep log, diff, and inspection behavior consistent even when their label sets differ.
- Make omitted commands discoverable through the overflow marker and `?` help.
- Add focused tests for rank selection, pinning, truncation, and tiny-width behavior.
- Add Betamax evidence for the clipped log case and at least one inspection context.

## Non-Goals

- Do not change key meanings or add new bindings.
- Do not add user-configurable keymaps.
- Do not move terminal key dispatch into `jk-tui`.
- Do not add `:` command mode, `!` external command mode, or workspaces.
- Do not add multi-row footers, a permanent command palette, or a pane layout.
- Do not introduce per-view hand-written hotbar strings.
- Do not hide or remove searchable help/discovery.

## Desired Behavior

The status row should be generated from the active `BindingContext` and the available row width.
The formatter should return one plain status string that `ViewChrome` can render normally.

Rules:

- If all labels fit, render the full ranked hotbar exactly as today.
- If labels do not fit, keep pinned help and quit labels visible first.
- Fill remaining space with the lowest-rank labels that fit.
- Preserve rank order among rendered non-pinned labels.
- Show an overflow marker when one or more labels were omitted.
- Never let a partially rendered label imply that a command exists under a different key.
- If the terminal is too narrow for the preferred labels, degrade to the shortest useful labels.
- If the terminal is too narrow for both help and quit, prefer a help affordance because it exposes
  quit and the rest of the keymap.

Suggested default labels:

```text
? help
q quit
...
```

Suggested tiny-width fallbacks:

```text
?
q
…
```

Use ASCII `...` in the first implementation unless the surrounding UI already uses a Unicode
ellipsis in this surface. The exact marker can change, but it must be deliberate and tested.

## Log Context

The log screen has the largest hotbar and should drive the first implementation.

Priority should follow the current ranks, with help and quit pinned:

1. `? help`
1. `H home  L log`
1. `r refresh`
1. `enter show`
1. `d diff`
1. `v evolog`
1. `s status`
1. `V options`
1. `space mark`
1. `c clear`
1. `j/k move`
1. `q quit`

At wide widths, the full row can render.

At normal Betamax widths where the full row does not fit, prefer inspection and view commands over
movement reminders. A reasonable adapted row is:

```text
? help  H home  L log  r refresh  enter show  d diff  v evolog  ...  q quit
```

If there is room for one more command, `s status` should appear before mark or movement reminders.
If marks are active and the implementation later makes hotbar rows state-aware, `c clear` may gain
priority only when marks exist. That state-aware behavior is a follow-up, not part of this slice.

## Diff Context

The diff hotbar currently fits normal widths, but it should still use the adaptive formatter:

```text
? help  V options  r refresh  j/k line  space/b page  q quit
```

When width is constrained, preserve:

- `? help`;
- `q quit`;
- `V options`;
- `r refresh`.

Navigation reminders such as `j/k line` and `space/b page` can be omitted first because a user can
open help for the complete movement list. Future file, hunk, fold, and horizontal-scroll hotbar
labels should follow the same rank-based rule instead of receiving ad hoc truncation logic.

## Inspection Contexts

Show, status, and evolog currently share generic inspection bindings. Their adaptive behavior should
match diff:

```text
? help  V options  r refresh  j/k line  space/b page  q quit
```

On narrow terminals, preserve help, quit, view options, and refresh. The inspection body title
already carries the active `jj show`, `jj status`, or `jj evolog` command, so the footer does not
need to repeat command-family details.

Evolog should not gain its own hotbar rows in this slice. Version selection, interdiff, and
evolog-specific View Options belong to later slices.

## Overflow And Truncation Rules

Do not rely on terminal clipping for normal operation.

The formatter should build a candidate list from keymap metadata:

1. Split labels into pinned and optional labels.
1. Sort optional labels by `hotbar_rank`.
1. Add labels while the joined string plus pinned labels and overflow marker fit.
1. If any labels are omitted, include the overflow marker.
1. Keep the final rendered order stable and rank-based.

Pinned labels:

- help: any hotbar label from the help command family, currently `? help`;
- quit: any hotbar label from the quit command family, currently `q quit`.

When there is enough room, render pinned labels in their ranked positions. When there is not enough
room, pin help at the left edge and quit at the right edge, with the overflow marker between the
visible optional labels and quit.

The formatter should avoid ambiguous half-labels. If a single label is longer than the available
width, use its tiny fallback or omit it. A low-level `truncate_to_width` helper is acceptable only
for final defensive clipping after semantic label selection has already happened.

## Implementation Slice

Keep the first implementation small and local to the TUI crate unless existing callers require a
binary change.

Suggested files:

- `crates/jk-tui/src/keymap.rs`
- `crates/jk-tui/src/chrome.rs`
- active view render call sites only if `ViewChrome` needs `BindingContext` or width-aware input

Suggested API shape:

```rust
pub fn hotbar(context: BindingContext, width: u16) -> String;
```

or, if preserving the existing zero-argument caller is clearer:

```rust
pub fn hotbar(context: BindingContext) -> String;
pub fn adaptive_hotbar(context: BindingContext, width: u16) -> String;
```

The formatter needs access to structured binding metadata, not only the already-joined string. If
needed, add a private `HotbarItem` derived from `KeyBinding`:

```rust
struct HotbarItem {
    label: &'static str,
    short_label: &'static str,
    rank: u8,
    pinned: HotbarPin,
}
```

The first slice can use simple built-in short labels for help, quit, and overflow only. Do not
require every binding to define a short label unless tests prove that normal widths still fail
after ranked omission.

`ViewChrome::render` already receives `ChromeAreas`, including status-row width. It is a natural
place to either accept a preformatted adaptive string or to call a width-aware helper before
rendering.

## Tests

Add focused unit tests in `crates/jk-tui/src/keymap.rs` or a nearby module:

- wide log width renders the full current hotbar;
- normal log width keeps `? help`, high-value inspection commands, overflow, and `q quit`;
- normal log width omits low-priority movement or mark labels before selected-change inspection
  labels;
- tiny width degrades to help before quit if both cannot fit;
- diff width keeps `? help`, `V options`, `r refresh`, and `q quit`;
- inspection width uses the same adaptive behavior as diff;
- every hotbar binding still has help/discovery metadata;
- no rendered adaptive hotbar exceeds the requested width.

Add a regression test for the width that clipped in Betamax. Use the terminal width from the
artifact or tape that exposed the bug rather than a guessed number.

Suggested validation:

```sh
cargo test -p jk-tui keymap::tests
```

Run broader tests if `ViewChrome` call sites change shape:

```sh
cargo test -p jk-tui
cargo test -p jk
```

## Betamax Evidence

Add or update a validation tape after the implementation lands.

Minimum evidence:

- start in the log screen at the normal Betamax width that currently clips;
- assert the status row contains `? help`;
- assert the status row contains `q quit`;
- assert the status row contains an overflow marker when optional labels are omitted;
- assert high-value log commands such as `d diff`, `v evolog`, and `s status` remain visible when
  the width allows them;
- open selected-change evolog and assert the inspection hotbar still fits;
- open `?` from a context where the hotbar omitted labels and assert discovery still lists omitted
  commands.

The tape should be assertion-oriented. README or website media can wait until the adaptive footer is
stable and the surrounding workflows are polished.

## Future Work

This slice supports command mode and workspaces by keeping the footer as a ranked, contextual
summary instead of a hard-coded string. Future screens can add command-mode, workspace, operation,
bookmark, and mutation-preview bindings to the same metadata and rely on the adaptive formatter to
show the most valuable subset.

Do not let adaptive truncation become hidden functionality. The footer is a reminder, not the whole
manual. `?` must remain visible because it is the durable discovery route for commands that do not
fit, commands that are intentionally not in the hotbar, and future user-configurable bindings.

## Recommended Immediate Chunk

Implement `adaptive_hotbar(context, width)` in `jk-tui`, update `ViewChrome` callers to pass the
status-row width, and add the log-width regression tests before changing any key labels. That chunk
fixes the visible clipping without reopening the keymap, command-discovery, or command-mode design.
