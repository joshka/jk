# View Stack Foundation

Status: draft  
Owner: `dogfood-view-stack` workspace pass  
Scope: first view-stack/mode-stack implementation chunk

## Problem

`jk` currently hard-codes one return path in the binary:

- `AppView::Log(LogView)` is the normal graph view.
- `AppView::Diff { log: Option<LogView>, diff: Box<DiffView> }` stores one optional previous log.
- Returning from diff swaps that optional log back into the active view.

That preserves today's log-to-diff behavior, but it is already a special case. It cannot naturally
support `show`, `status`, command previews, workspace screens, graph filters, or nested picker
flows. It also makes initial `jk diff REV` awkward because there is no previous log to return to.

Search input is a second hard-coded path. The terminal loop owns `InputMode::DiffSearch`, renders a
temporary status override, and manually dispatches the finished query to the active diff. That is a
reasonable precursor for command mode, but it should become a small mode stack before `:`, `!`,
filter search, and picker prompts add more prompt-like states.

## Goals

- Replace the optional previous-log field with a general view stack.
- Add a tiny mode stack that initially only models the current diff search prompt.
- Preserve current user behavior:
  - log to selected-change diff returns to the same log state, including selection and scroll;
  - initial `jk diff REV` has a single root diff view, so back is a no-op and does not crash;
  - `q` still quits, except when closing help as it does today;
  - `Esc` keeps its current quit/close-help behavior outside active prompt modes;
  - search entry, search submit, `n`, and `N` keep current diff behavior.
- Add `Backspace` as the explicit back key from the roadmap.
- Keep the first implementation small enough to review as a foundation slice.

## Non-Goals

- Do not add new screens such as `show`, `status`, workspace, command history, or preview.
- Do not move key dispatch to the generated keymap registry in this slice.
- Do not add searchable help, graph filters, command mode, or external command mode.
- Do not introduce async loading, cancellation, or background preview runners.
- Do not make `DiffView` own log loading or make views perform I/O.
- Do not change the visible chrome, help rows, hotbar text, or current key meanings except adding
  `Backspace` for back.

## Proposed API And State Shape

Keep the first stack private to `crates/jk/src/main.rs` because the binary currently owns event
dispatch and I/O. The shape should still be backend-neutral enough to move into `jk-tui` once more
screens exist.

```rust
#[derive(Debug)]
struct AppState {
    views: ViewStack,
    modes: ModeStack,
}

#[derive(Debug)]
struct ViewStack {
    views: Vec<AppView>,
}

#[derive(Debug)]
enum AppView {
    Log(LogView),
    Diff(DiffView),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ModeStack {
    modes: Vec<InputMode>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum InputMode {
    DiffSearch { query: String },
}
```

Required `ViewStack` behavior:

- `ViewStack::new(root)` stores a non-empty stack.
- `active()` and `active_mut()` return the last view.
- `push(view)` opens a child screen.
- `pop()` returns `true` only when it removed a child view.
- `pop()` returns `false` at the root and leaves the root view unchanged.
- `replace_root(view)` is available for startup or future home/log root replacement, but the first
  chunk can avoid using it if current source-switching remains simpler.

Required `ModeStack` behavior:

- `active()` and `active_mut()` return the top prompt mode.
- `push(InputMode::DiffSearch { query })` starts the current search prompt.
- `pop()` closes the current prompt and reveals the previous mode, if any.
- The first implementation should not add mode IDs, command registries, or generic prompt
  callbacks. Search is the only real prompt today.

The terminal loop should render from `state.views.active_mut()` and the optional top mode:

- active log renders as today;
- active diff with no mode renders as today;
- active diff with `DiffSearch` renders with the `/{query}` status override.

Actions should return stack-oriented transitions:

```rust
enum AppTransition {
    Continue,
    Push(AppView),
    PopView,
    Quit,
}
```

`OpenDiff` becomes `Push(AppView::Diff(diff))`. `ReturnToLog` becomes `PopView`. A root-level pop is
a no-op, which preserves initial `jk diff REV` and failed initial diff behavior.

## First Implementation Chunk

Target size: less than 200 LoC changed, excluding test-only fixture text if needed.

Files:

- `crates/jk/src/main.rs`
- `crates/jk/src/key.rs`

Implementation steps:

1. Replace `AppView::Diff { log, diff }` with `AppView::Diff(DiffView)`.
1. Add private `AppState`, `ViewStack`, and `ModeStack` helpers in `main.rs`.
1. Initialize `AppState` with a single root view from the current CLI startup path.
1. Render through `state.views.active_mut()` and `state.modes.active()`.
1. Convert diff open to `ViewStack::push(AppView::Diff(diff))`.
1. Convert diff return to `ViewStack::pop()`, ignoring `false` at the root.
1. Move `InputMode::DiffSearch` into `ModeStack` without adding other modes.
1. Add `AppKey::Back` and map `Backspace` to it.
1. Dispatch `AppKey::Back` by closing the top mode first, otherwise popping one view.

Keep `JjLog` and `JjDiff` sources in the binary loop for this slice. The active log still refreshes
from the existing `source`, and the active diff still refreshes from `diff_source`. Per-view command
sources can wait until there are multiple log-like root screens.

## Tests

Add focused unit tests near the state helpers in `crates/jk/src/main.rs`:

- `view_stack_keeps_root_when_popped` proves initial `jk diff` back behavior is a no-op.
- `view_stack_returns_to_previous_log_state` proves pushing a diff and popping it restores the
  exact previous `LogView` value.
- `mode_stack_closes_search_before_popping_view` proves `Backspace` during search leaves the diff
  open and exits the prompt.
- `mode_stack_submit_search_restores_normal_mode` proves `Enter` applies search and pops the mode.

Extend `crates/jk/src/key.rs` tests:

- `backspace_maps_to_back` proves the new roadmap key is wired.

Keep existing `jk-tui` view tests unchanged. They already cover help closing, diff retry rendering,
empty diff rendering, and log/diff action contracts. This slice should add stack behavior without
changing those contracts.

Suggested validation:

```sh
cargo test -p jk
cargo test -p jk-tui log_view::tests::refresh_and_quit_actions_request_outer_loop_effects
cargo test -p jk-tui diff_view::tests::refresh_and_return_actions_request_outer_loop_effects
```

## Betamax Evidence Expectations

This planning slice does not add tapes, but the implementation PR should collect or update
user-visible evidence for the stack behavior.

Minimum validation tape expectations:

- start in log, move selection, press `d`, assert the diff title/content matches the selected
  change, press `Backspace`, and assert the original log selection is still visible;
- start with `jk diff REV`, press `Backspace`, and assert the diff remains visible;
- start in diff, press `/`, type a query, press `Backspace`, and assert the prompt closes while the
  diff remains visible;
- start in diff, search for text, press `n` and `N`, and assert existing search navigation still
  works.

Use existing `just betamax-diff` or a new short validation tape if the current tape does not cover
these assertions. Prefer assertion-heavy validation over media generation for this foundation
slice.

## Risks

- Moving the stack into `jk-tui` immediately may cause churn because `main.rs` still owns I/O,
  sources, key dispatch, and terminal lifecycle. Start private, then move the stable state surface.
- A generic mode abstraction can easily overbuild command mode before its requirements are known.
  Keep search as the only mode and make the stack mechanics obvious.
- `Esc` is already quit outside prompts. Adding back behavior to `Esc` would be a user-visible
  behavior change, so use `Backspace` for stack pop in this slice.
- Returning from root diff must be a no-op. Accidentally treating it as quit or log reload would
  break direct `jk diff REV` usage.
- Popping a view must preserve the stored log value, not reload the log. Reloading would lose the
  current selection and scroll state.

## Follow-Up Chunks

- Move `ViewStack`, `ModeStack`, and stack transition tests into `jk-tui` once another screen needs
  the same behavior.
- Add generated keymap/help rows for `Backspace` after the keymap registry owns visible binding
  metadata.
- Add log filter or graph search as the next mode-stack consumer, preserving previous filter state
  through the view stack.
- Add selected-change `show`, `status`, and stat/detail diff variants as ordinary pushed views.
- Attach command specs and refresh plans to view entries once command history and preview screens
  need per-view provenance.
- Expand Betamax validation into a stable `log-to-diff-back` tape and reuse it for README or site
  media only after the assertion tape is reliable.
