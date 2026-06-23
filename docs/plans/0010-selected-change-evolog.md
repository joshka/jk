# Selected-Change Evolog

Status: draft

Owner: current dogfood workspace pass

Scope: first selected-change evolution-log implementation slice

## Decision

Add lowercase `v` as a read-only selected-change `jj evolog` screen.

This slice should be deliberately small: pressing `v` on the log opens evolution history for the
selected revision/change, rendered by `jj evolog -r REV`, inside the existing view stack. It should
reuse `JjCommandSpec`, `GlobalOptions`, `InspectionSnapshot`, `RenderedView`, search, refresh, and
back behavior from the current show/status inspection path.

Do not add mutation, op-log recovery, interdiff, evolog version selection, patch display toggles,
or new View Options rows in this slice. Uppercase `V` remains the reusable View Options entry point.

## Command Shape Evidence

The installed `jj evolog --help` output in this workspace supports revision targeting:

```text
Usage: jj evolog [OPTIONS]

Options:
  -r, --revisions <REVSETS>
          Follow changes from these revisions

          [default: @]
```

Therefore the first generated command should be:

```text
jj evolog -r REV
```

where `REV` is the selected log change id or revision id already used by selected-change `show` and
`diff`. The typed spec still renders process argv as:

```text
jj --no-pager --color always [global options...] evolog -r REV
```

with global options before the command family. The title shown in the view should stay the
command-family preview, for example:

```text
jj evolog -r abc123
```

## Context

This slice follows the foundations already present in the current dogfood workspace:

1. `JjCommandSpec` and `GlobalOptions` are implemented in `jk-core`.
1. `JjShow` and `JjStatus` prove the `InspectionSnapshot` source pattern.
1. `RenderedView` and `RenderedState` provide generic rendered-output scrolling, search, refresh,
   retry, and error display.
1. `ViewStack` in `crates/jk/src/main.rs` already opens diff/show/status as focused stack entries.
1. `V` already opens View Options, and lowercase `v` is intentionally unbound for evolog.

The product plan and CLI addendum reserve `v` for evolution history and `V` for display options.
This slice should use that reservation without pulling display options into evolog yet.

## Goals

- Bind lowercase `v` on the log screen to open selected-change evolution history.
- Build a typed `jj evolog -r REV` command with `JjCommandSpec::render_read_only`.
- Preserve repository/global-option propagation through `GlobalOptions`.
- Reuse the current rendered inspection view stack rather than adding an evolog-specific viewer.
- Preserve existing show/status search, scroll, refresh, error, help, and back semantics.
- Add visible help and searchable discovery metadata for the log action.
- Add unit tests and one Betamax validation tape for the workflow.
- Keep the implementation small enough to review as a first read-only evolog slice.

## Non-Goals

- Do not add `jk evolog` as a root CLI subcommand in this slice.
- Do not add evolog View Options rows for `--patch`, `--stat`, `--summary`, `--no-graph`,
  `--template`, `--limit`, or `--reversed`.
- Do not add an evolog-specific selection model for historical versions.
- Do not add `Enter` from evolog to show a version.
- Do not add `d` from evolog to diff a version.
- Do not add `I` or any interdiff workflow.
- Do not add op-log recovery, undo/redo, operation restore, or operation time-travel UI.
- Do not mutate repository state or invoke any external editor/tool.
- Do not replace `jj`-rendered evolog output with native rendering.

## User Experience

From the log screen:

1. The user moves to a change.
1. The user presses `v`.
1. `jk` runs `jj evolog -r <selected change>`.
1. A focused rendered inspection view opens with title `jj evolog -r <selected change>`.
1. `Backspace`, `H`, or `L` returns to the previous log view without reloading it.

If no log revision is selected, `v` is a no-op. That matches the current selected-change `d` and
`Enter` behavior and avoids inventing status text for an impossible normal state.

The evolog screen uses the existing inspection bindings for the first slice:

- `j/k` or arrows scroll one line;
- `Ctrl-j/k` scroll one line;
- `space`, `b`, `Ctrl-f`, and `Ctrl-b` page;
- `g/G` or `Home/End` jump to top/bottom;
- `/`, `n`, and `N` search rendered output;
- `V` opens the placeholder View Options overlay only;
- `r` refreshes by rerunning the same `jj evolog -r REV` spec;
- `H`, `L`, or `Backspace` returns to the previous view;
- `?` opens searchable command discovery;
- `q` quits from the root or closes help as today.

`v` should not be active inside diff/show/status/evolog rendered views in this first slice. It is a
log-selected-change action, not a global nested inspection action yet.

## Source Shape

Add a new source module alongside show and status:

```rust
// crates/jk-cli/src/evolog.rs
pub struct EvologQuery {
    rev: String,
}

pub struct JjEvolog {
    repository: Option<PathBuf>,
}

impl JjEvolog {
    pub fn with_repository(self, repository: impl Into<PathBuf>) -> Self;
    pub fn load_query(&self, query: &EvologQuery) -> Result<InspectionSnapshot, JjEvologError>;
    pub fn spec_for(&self, query: &EvologQuery) -> JjCommandSpec;
}
```

`spec_for` should build this argv:

```text
["evolog", "-r", REV]
```

The implementation should mirror `JjShow` and `JjStatus`:

- build the spec first;
- run through `run_jj_spec`;
- return `InspectionSnapshot::new(query.target_label(), rendered).with_title(spec.title())`;
- preserve `with_repository` through `spec.with_repository(repository)`;
- return an `Io` error for spawn/read failures;
- return `CommandFailed(stderr)` for non-zero `jj` exits.

Re-export the source types from `crates/jk-cli/src/lib.rs`.

## Application Shape

Add a new `AppView` variant that follows show/status:

```rust
Evolog {
    view: RenderedView,
    query: EvologQuery,
}
```

Add `JjEvolog` to the terminal loop dependencies:

- create `evolog_source(&Args) -> JjEvolog`;
- pass `&JjEvolog` into `run_terminal`;
- add `push_selected_evolog(&mut AppState, &JjEvolog)`;
- add `apply_evolog_action` that maps to the same `RenderedAction` set as show/status;
- add `refresh_evolog` that reruns `source.load_query(query)`.

Do not add `Command::Evolog` to the CLI yet. A future root `jk evolog -r REV` subcommand can reuse
`EvologQuery` and `JjEvolog`, but the first slice should stay selected-change only.

## Keymap And Discovery

Add a binary key route:

```rust
AppKey::OpenEvolog
```

Map lowercase `v` to `OpenEvolog`. Keep uppercase `V` mapped to `OpenViewOptions`.

Add visible log metadata in `crates/jk-tui/src/keymap.rs`:

- action label: `Open evolog`;
- key: `v`;
- help: `open selected-change evolog`;
- command family: add `CommandFamily::JjEvolog` with label `jj evolog`;
- aliases: `evolution`, `history`, `change`, `version`;
- hotbar: include `v evolog` only if the log hotbar still fits normal terminals.

Discovery should match queries such as:

- `v`;
- `evolog`;
- `jj evolog`;
- `evolution history`;
- `selected change`.

Do not add evolog-specific rows to the inspection context in this slice. The evolog screen should
use the generic inspection rows until it gains version selection or interdiff actions.

## View Options

`V` remains the display/options key. On evolog, the first slice should behave like show/status:

```text
View Options

No view options in this slice.

esc close
```

Do not add rows for `--patch`, `--stat`, `--summary`, `--name-only`, `--no-graph`, `--template`,
`--limit`, or `--reversed`. Those belong to the reusable View Options model after this slice proves
the selected-change source, stack, refresh, and search path.

## Navigation, Back, Refresh, And Search

Use the current `RenderedView` behavior unchanged:

- opening evolog pushes a child view and preserves the stored log under it;
- `Backspace` closes an active prompt first, otherwise pops one view;
- popping evolog returns to the exact previous log state without refreshing;
- `H` and `L` in a rendered inspection view request the same return-to-log action as show/status;
- root-level pop remains a no-op, even though this slice does not add a root evolog command;
- `r` refreshes by rerunning the stored `EvologQuery`;
- successful refresh replaces rendered output and clears stale status messages;
- failed refresh keeps the previous rendered body and shows the error in the status line;
- `/` starts the existing inspection search prompt;
- `Enter` submits search and restores normal mode;
- `Backspace` or `Esc` closes search without leaving evolog;
- `n` and `N` navigate the current search matches.

The evolog view should not try to preserve a selected historical version across refresh because this
slice has no historical-version selection model.

## Status And Error Behavior

Initial load failure from the log should leave the user on the log and show the error there. That
matches selected-change diff/show behavior and avoids pushing an empty child view for a transient
bad revset.

Direct root error rendering can wait until a root `jk evolog` command exists. If implementation
shares a helper that can build an error view, it must still not expose a root CLI route in this
slice.

Inside an evolog view, refresh failure should use `RenderedView::show_error`. The status line should
contain the `JjEvologError` text. The body should remain the previous successful output.

Empty successful output should use the existing rendered inspection empty-state text:

```text
No output for REV.

The jj command produced no visible text.
```

## Tests

Add focused unit tests for the command source:

- `JjEvolog::spec_for(EvologQuery::new("abc123"))` produces
  `["evolog", "-r", "abc123"]`;
- the spec title is `jj evolog -r abc123`;
- the spec is `RenderReadOnly`, `ReadOnly`, and `ReRunSpec`;
- repository options render before `evolog` in process argv;
- non-zero `jj evolog` exits become `JjEvologError::CommandFailed`.

Add binary/input tests:

- lowercase `v` maps to `AppKey::OpenEvolog`;
- uppercase `V` still maps to `AppKey::OpenViewOptions`;
- pressing `v` from a log with a selected change pushes `AppView::Evolog`;
- pressing `v` outside the log is ignored;
- `refresh_evolog` replaces the rendered view after success and preserves the query;
- `refresh_evolog` keeps the old body and sets status on failure;
- `Backspace` from evolog pops to the preserved log;
- `Backspace` while evolog search is active closes search before popping the view.

Add TUI metadata tests:

- log help includes `v` as selected-change evolog;
- log discovery finds `jj evolog`;
- inspection help remains generic and does not advertise evolog-specific version actions;
- the View Options placeholder still appears for inspection contexts.

Suggested validation:

```sh
cargo test -p jk-cli evolog
cargo test -p jk key::tests::lowercase_v_opens_evolog
cargo test -p jk-tui keymap::tests
```

Run broader tests if the implementation touches shared rendered-view behavior:

```sh
cargo test -p jk
cargo test -p jk-tui rendered_view rendered_state keymap
```

## Betamax Evidence

Add one validation tape after the implementation lands. The tape should use a fixture where one
change has more than one evolution entry, such as a change that was described, amended, or rebased.

Minimum assertions:

- start in log;
- move to the prepared change;
- press `v`;
- assert the title contains `jj evolog -r`;
- assert the rendered body contains evolution-log output for that change;
- press `/`, search for text known to exist in the evolog output, and assert search status;
- press `r` and assert the title/body still represent the same command;
- press `Backspace` and assert the original log view returns with selection preserved.

The first tape should be validation-oriented, not README media. Media can wait until evolog has
View Options, historical-version selection, or interdiff behavior worth demonstrating publicly.

## Dependency Order

This slice depends on:

1. `0001-command-spec.md`, because evolog must use `JjCommandSpec`.
1. `0003-view-stack-foundation.md`, because evolog opens as a pushed focused view.
1. `0004-inspection-foundation.md`, because evolog should reuse the show/status source pattern.
1. `0006-searchable-command-discovery.md`, because `v` should be discoverable by command family.
1. `0008-view-options-overlay.md`, because `V` must already own display options before `v` lands.
1. `0009-global-run-options-foundation.md`, because evolog must preserve `GlobalOptions`.

This slice should land before:

- evolog View Options rows;
- historical-version selection inside evolog;
- interdiff from evolog;
- workspace-aware evolog actions;
- operation log and operation recovery;
- mutation preview or command history.

The immediate next implementation chunk should be the `JjEvolog` source plus `v` key routing into a
pushed `RenderedView`. That proves the command, stack, refresh, and search path before adding any
display options or version-aware behavior.

## Risks

- Adding View Options rows too early would duplicate the planned reusable display-options model.
  Keep evolog display flags out of this slice.
- Treating evolog output as semantic graph data would overfit before a version-selection model
  exists. Keep the body opaque and rendered by `jj`.
- Reusing `H / L` as return-to-log is slightly odd from an evolog child view, but it is consistent
  with the current generic inspection view. A future stack-wide back label can clean this up.
- Using commit ids where change ids are expected can make evolog less useful. Start with the same
  selected-change identifier used by `show` and `diff`, then adjust only if command evidence shows a
  better stable target.
