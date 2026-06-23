# View Options Overlay

Status: implemented in the `vibe` spike

Owner: `vibe-view-options-overlay` workspace spike

Scope: first reusable View Options overlay chunk, centered on moving log template selection under
`V`

## Problem

The pre-slice implementation had the pieces needed to add the reusable View Options entry point,
but the in-app key surface was still temporary:

- uppercase `T` opens a standalone log-template popup through `LogAction::SwitchTemplate`,
  `ActionResult::SwitchTemplate`, `AppTransition::OpenTemplateSelector`, and
  `InputMode::LogTemplate`;
- the log template model already lives in `JjLog` and correctly keeps the rendered log template
  separate from the semantic JSON template used for navigation;
- help, hotbar, and searchable discovery currently advertise `T template`;
- lowercase `v` has no current product meaning and is reserved for standalone `jj evolog`;
- the CLI surface addendum reserves uppercase `V` for reusable View Options across display,
  graph/list, template, and diff-format options.

Leaving template selection on standalone `T` while adding more display toggles would have created
the wrong product shape. This slice introduces the reusable `V` overlay first, then hangs the
existing log-template selector from a `Template` row inside that overlay.

## Goals

- Add uppercase `V` as the View Options entry point.
- Keep lowercase `v` reserved for future standalone evolog and do not bind it in this slice.
- Move log template switching under `V` without changing the existing template application
  semantics.
- Reuse the existing log-template options, selector rows, selection movement, apply behavior, and
  refresh path.
- Preserve selection, marks, scroll, and navigation usability after applying a template.
- Update help, hotbar, and searchable discovery to advertise `V options`, not `T template`.
- Establish a small view-options state shape that can later host graph/list and diff-format rows.
- Keep the first slice local to view options and log templates, not the full display-options
  roadmap.

## Non-Goals

- Do not add evolog behavior.
- Do not add diff format toggles such as stat, summary, name-only, git, or color-words.
- Do not add graph/list toggles such as reversed, no-graph, limit editing, or count-only.
- Do not add persistent user template configuration.
- Do not add editable custom-template input beyond the existing startup custom template behavior.
- Do not change `jk log -T TEMPLATE` command-line behavior.
- Do not change the semantic JSON `LOG_TEMPLATE` pass.
- Do not introduce a broad command registry or replace the current keymap metadata model.

## Pre-Slice State

This plan was written from the pre-slice `vibe` workspace state:

1. `crates/jk-cli/src/log.rs` already owns `LogTemplateSelection`, built-in template aliases,
   startup custom templates, and `JjLog::template_options`.
1. `crates/jk/src/main.rs` owns `ModeStack`, `InputMode::LogTemplate`,
   `open_template_selector`, `handle_template_mode`, and `apply_log_template_selection`.
1. `crates/jk/src/key.rs` maps uppercase `T` to `LogAction::SwitchTemplate`.
1. `crates/jk-tui/src/log_view.rs` exposes `LogAction::SwitchTemplate` and
   `ActionResult::SwitchTemplate`.
1. `crates/jk-tui/src/keymap.rs` generates log help, hotbar, and discovery rows from metadata,
   currently with `T template`.
1. `?` already opens searchable command discovery through the shared keymap metadata.
1. Ordered revision marks and rendered-line scrolling already exist, so template switching must
   continue preserving those state paths through `LogState::refresh`.

The slice extends these seams rather than redesigning input dispatch or rendering. After
implementation, `V` owns the visible View Options route and the temporary `T template` path is
removed from dispatch, help, hotbar metadata, and command discovery.

## Desired Key Surface

The durable key surface should be:

- `V`: open View Options;
- `v`: reserved for selected-change evolog and intentionally unbound in this slice;
- `T`: remove from visible help, hotbar, and discovery before review-ready work.

Recommended treatment of `T`:

- prefer removing the `T` binding from `crates/jk/src/key.rs` in the implementation slice;
- if the implementation keeps `T` briefly as a hidden transitional alias, it must not appear in
  help, hotbar, discovery, or Betamax evidence;
- do not let `T` remain the only path to template selection.

Visible metadata should say `V options`. Searchable discovery should match queries such as
`view`, `options`, `template`, and `jj log`.

## User Experience

On the log screen, pressing `V` opens a centered View Options overlay above the current log:

```text
View Options

> Template      configured

j/k or arrows move   enter open   esc close
```

The first slice only needs one active row:

- `Template`: opens the existing log-template selector behavior.

Pressing Enter on `Template` closes or covers the View Options overlay with the existing template
selector:

```text
Log template

> Configured         jj configured template
  Comfortable        builtin_log_comfortable
  Compact            builtin_log_compact
  Full description   builtin_log_compact_full_description
  Detailed           builtin_log_detailed
  Oneline            builtin_log_oneline
  Redacted           builtin_log_redacted

j/k or arrows move   enter apply   esc cancel
```

The exact spacing can remain close to the current selector. The important behavior is that the user
enters through `V`, chooses `Template`, and then gets the same selector semantics as today.

## Non-Log Views

Wire `V` as the common View Options entry point even before diff/show/status have real options. For
this first slice, non-log views should show a small placeholder overlay:

```text
View Options

No view options in this slice.

esc close
```

This is preferable to ignoring `V` because it reserves the reusable key visibly and avoids another
future keymap migration when diff/show/status options arrive. The placeholder must not imply that
diff-format or show/status toggles already work.

If implementation risk is higher than expected, a narrower fallback is acceptable: handle `V` only
on the log screen and ignore it elsewhere. That fallback must keep non-log help and discovery from
advertising `V options` until those contexts actually handle it.

## State And Model Shape

Add a general view-options mode instead of making template selection the top-level action:

```rust
enum InputMode {
    ViewOptions {
        context: BindingContext,
        selected: usize,
    },
    LogTemplate {
        options: Vec<LogTemplateSelection>,
        selected: usize,
    },
    // existing modes...
}
```

Suggested row model in `crates/jk-tui/src/keymap.rs` or a small adjacent TUI module:

```rust
pub enum ViewOptionRow {
    LogTemplate,
    Placeholder,
}

pub struct ViewOptionsState {
    pub context: BindingContext,
    pub selected: usize,
}
```

The exact names can change, but the ownership should stay small:

- `crates/jk/src/key.rs` maps uppercase `V` to a semantic open-options action.
- `crates/jk/src/main.rs` owns mode transitions, row selection, and applying a selected row.
- `crates/jk-tui/src/keymap.rs` owns visible `V options` metadata and discovery tags.
- TUI rendering helpers own line formatting for the View Options overlay.
- The existing `LogTemplateSelection` model remains in `jk-cli`; do not duplicate template strings
  in the overlay model.

Do not put graph/list/diff settings in this slice. Add only enough enum shape that those rows can
be appended later without another input-mode rewrite.

## Interaction Flow

### Opening View Options

1. User presses `V`.
1. `AppKey` maps it to an open-view-options action.
1. The terminal loop pushes `InputMode::ViewOptions` with the active `BindingContext`.
1. The active view renders normally.
1. A centered `View Options` overlay renders above the active view.

### Moving In View Options

- `j/k` and up/down arrows move the selected row.
- Movement clamps at the first and last row.
- On log, the first slice has one row, so movement is a no-op.
- On placeholder contexts, movement is a no-op.

### Opening Template Selection

1. User presses Enter on the log `Template` row.
1. The app replaces or pushes the mode with `InputMode::LogTemplate`.
1. The existing `open_template_selector` path supplies `source.template_options()` and selects the
   active template.
1. The selector renders with the existing `Log template` title and rows.

Replacing the View Options mode is simpler than keeping it underneath. Returning to View Options
after applying or canceling the selector is not required in this slice.

### Applying A Template

Template application should keep the current behavior:

1. Capture the selected `LogTemplateSelection`.
1. Close the selector mode.
1. Build `next_source` from the current `JjLog`, force explicit `JjLogCommand::Log`, and apply the
   selected rendered template.
1. Load the replacement log before mutating active state.
1. On success, assign `source = next_source` and call `log.refresh(snapshot)`.
1. On failure, keep the previous rendered log visible and show the error in the status line.

The refresh path must keep selected-change navigation usable and should preserve ordered marks and
scroll state through the existing `LogState::refresh` behavior.

### Closing

- `Esc`, `Backspace`, and `q` close View Options without changing state.
- `Esc`, `Backspace`, and `q` keep closing the log-template selector without changing the template.
- `Enter` on a placeholder View Options overlay closes it or does nothing; choose the simpler
  implementation and test it.

## Implementation Phases

### Phase 1: key and action plumbing

Files:

- `crates/jk/src/key.rs`
- `crates/jk/src/main.rs`
- `crates/jk-tui/src/log_view.rs`

Acceptance:

- uppercase `V` opens a View Options mode from the log screen;
- lowercase `v` remains unbound or ignored;
- the old standalone `T` path is removed or kept only as a hidden transitional alias;
- `LogAction::SwitchTemplate` is replaced by a more general view-options action, or kept only as an
  internal helper that is no longer directly bound to `T`.

### Phase 2: View Options overlay rendering

Files:

- `crates/jk/src/main.rs`
- `crates/jk-tui/src/keymap.rs`
- optionally `crates/jk-tui/src/chrome.rs` or a small new TUI module

Acceptance:

- the log View Options overlay renders a `Template` row;
- `j/k` and arrow movement are handled even when there is only one row;
- `Esc`, `Backspace`, and `q` close the overlay;
- non-log views either show the documented placeholder overlay or intentionally ignore `V` without
  advertising it in their visible metadata.

### Phase 3: Template row integration

Files:

- `crates/jk/src/main.rs`
- `crates/jk-tui/src/keymap.rs`

Acceptance:

- Enter on `Template` opens the existing template selector behavior;
- selector navigation still supports `j/k` and up/down arrows;
- Enter applies the highlighted template;
- canceling the selector does not change `JjLog` source state;
- applying a template keeps selection, ordered marks, and navigation usable when the selected change
  still exists.

### Phase 4: visible discovery cleanup

Files:

- `crates/jk/src/key.rs`
- `crates/jk-tui/src/keymap.rs`

Acceptance:

- log hotbar includes `V options`;
- log help includes `V` as View Options;
- searchable discovery finds `V options` through `view`, `options`, `template`, and `jj log`;
- visible help, hotbar, discovery, and Betamax evidence no longer present `T template` as the
  primary path.

## Tests

Required unit tests:

- `AppKey::from_crossterm` maps uppercase `V` to opening View Options.
- `AppKey::from_crossterm` does not map lowercase `v` to template selection.
- if `T` is removed, `AppKey::from_crossterm` ignores uppercase `T`; if retained, tests document
  that it is a hidden transitional alias.
- opening View Options from log pushes `InputMode::ViewOptions` with `BindingContext::Log`.
- Enter on the log `Template` row opens `InputMode::LogTemplate` with the current template selected.
- `Esc`, `Backspace`, and `q` close View Options without changing source state.
- applying a template through the `V` path calls the same refresh behavior as the existing selector.
- template switch failure keeps the previous log and shows an error.
- log help and hotbar snapshots contain `V options` and do not contain visible `T template`.
- discovery filtering finds the options row for `view`, `options`, and `template log`.
- non-log `V` behavior is tested according to the chosen first-slice policy: placeholder overlay or
  ignored key with no visible metadata.

Suggested validation:

```sh
cargo test -p jk key
cargo test -p jk app
cargo test -p jk-tui keymap
cargo test -p jk-tui log
```

Run the broader package tests if the action enums or public TUI API change more than expected:

```sh
cargo test -p jk
cargo test -p jk-tui
```

The implementation includes deterministic unit coverage for the `V` overlay, the route from the
`Template` row into the existing selector, source preservation on cancellation, and the reload
failure path. Betamax covers the successful user-facing path from `V` through selector application
because that path intentionally crosses the real terminal and `jj` rendering boundary.

## Betamax Evidence Expectations

Collect focused validation tape evidence after unit tests pass:

- start on `jk log`;
- press `V` and assert the `View Options` overlay appears with a `Template` row;
- press Enter and assert the existing `Log template` selector appears;
- move to a different built-in template and apply it;
- assert the rendered log title or visible body changes consistently with the selected template;
- assert `j/k` navigation still moves the selected change after the template switch;
- assert the hotbar or help surface advertises `V options`;
- assert `T template` is not the primary visible route.

Keep tapes under `tapes/validation/` unless the repo has a more specific convention by the time the
implementation lands. Do not regenerate README, crates.io, or website media in this slice.

## Risks

- Keeping `T` visible after adding `V` preserves the temporary key shape and makes a later tags
  screen migration harder.
- Treating View Options as log-only can make the next diff/show/status display slice repeat input
  and overlay work.
- Moving template selection can accidentally apply the user-facing rendered template to the
  semantic JSON pass. Keep `LOG_TEMPLATE` independent.
- Replacing the selector mode instead of layering it may surprise a future nested-options flow, but
  it keeps this slice simpler and does not block adding a return-to-options behavior later.
- Placeholder overlays on non-log views could look like unfinished product. Keep the copy explicit
  and remove it as soon as real diff/show/status options land.

## How This Prevents Rework

This slice changes the key topology before the option set grows. After `V` exists, later work can
add graph/list rows, diff-format rows, show patch toggles, operation-log display options, and
template-aware rendering under the same overlay model instead of moving each one from a standalone
key. It also frees `T` for the future tags screen and keeps lowercase `v` available for evolog,
matching the CLI surface addendum before those screens are implemented.

## Acceptance Criteria

- `V` opens a View Options popup while the log view is active.
- The log View Options popup includes a `Template` row.
- Enter on `Template` opens the existing template selector behavior.
- Selecting a template through the `V` path reloads the rendered log and keeps
  selection/navigation usable.
- Help, hotbar, and searchable discovery reflect `V options`.
- `v` remains reserved for evolog and is not used for template or view-options behavior.
- `T` is removed from the visible key surface or explicitly documented and tested as a hidden
  transitional alias.
- Non-log views either show the documented placeholder View Options popup or ignore `V` with a clear
  implementation note and no visible metadata.
- No code path applies the rendered template selection to the semantic JSON `LOG_TEMPLATE` pass.
