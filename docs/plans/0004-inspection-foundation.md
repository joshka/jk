# Inspection Foundation

Status: draft

Owner: implementation spike

Scope: first `jj`-shaped inspection roadmap bucket after foundation

## Problem

`jk` can inspect a selected change today, but the model is still narrower than `jj`:

- `JjDiff::load(change_id)` always builds `jj diff -r CHANGE`.
- `jk diff REV` exists as positional compatibility sugar, while `jj diff` treats positional
  arguments as filesets.
- `DiffSnapshot` stores a selected-change identifier, not a reusable command/query source.
- `main.rs` can start with a direct root diff view, but refresh still asks the view for one change
  id.
- `show` and `status` do not exist as canonical `jk` entry points or pushed inspection views.

The product plan and roadmap both make the next bucket clear: inspection should match `jj` command
shapes before `jk` adds mutation flows, command mode, command history, or richer file navigation.
The work should keep `jj` responsible for rendered output and make `jk` responsible for command
specs, view stack ownership, titles, search, folding, refresh, and visible provenance.

## Goals

- Add canonical `jk diff -r REV`, `jk diff --from A --to B`, and stat forms.
- Keep `jk diff REV` only as compatibility sugar for `jk diff -r REV`.
- Add `jk show` and `jk status` as read-only inspection commands after canonical diff lands.
- Preserve `jj`-rendered patch, show, and status output for config fidelity.
- Store enough source/query data outside the rendered view to refresh direct root views correctly.
- Use `JjCommandSpec` for every read-only inspection command and title.
- Open inspection screens as direct root views or pushed `ViewStack` entries, not panes.
- Keep the first implementation chunk under 200 LoC changed so it is reviewable.

## Non-Goals

- Do not add mutation commands, mutation preview, command history, or command mode.
- Do not replace `jj` diff/show/status rendering with native rendering.
- Do not add a permanent split-pane dashboard or side preview.
- Do not implement hunk staging, split, restore, diffedit, or absorb.
- Do not add a provider crate or broad command-runner abstraction in this bucket.
- Do not make `jk diff` parse every future `jj diff` flag in the first chunk.
- Do not add one-off display toggles that bypass the reusable `V` View Options overlay.

## Current Dependencies

This plan assumes the foundation plans have landed or are landing in order:

1. `0001-command-spec.md`: `JjCommandSpec` exists and can render read-only argv previews.
1. `0002-keymap-help-data.md`: visible hotbar/help text can be generated from binding data.
1. `0003-view-stack-foundation.md`: root views and child views are managed by `ViewStack`.

The inspection bucket should not bypass those foundations. In particular, view titles and future
command previews should come from `JjCommandSpec`, and direct command entry points should become
root views in the same stack used by log-to-diff navigation.

## Proposed CLI Shape

Canonical command forms:

```text
jk diff -r REV
jk diff --from FROM --to TO
jk diff --stat -r REV
jk diff --stat --from FROM --to TO
jk show REV
jk show REV1 REV2
jk status
jk status FILESET...
```

Compatibility:

```text
jk diff REV
```

`jk diff REV` should resolve to the same internal query as `jk diff -r REV`. It should stay
documented as compatibility sugar rather than the preferred shape because `jj diff` positional
arguments are filesets.

Validation rules:

- `-r REV` and `--from/--to` are mutually exclusive.
- `--from` and `--to` must be provided together.
- omitted diff revision defaults to `@`, matching the current behavior.
- `--stat` changes the rendered command to a stat view, not a side panel.
- unsupported passthrough diff flags can wait until a later richer parser chunk.

## Proposed API Shape

Start with small query objects near current call sites. Move them into `jk-core` only after more
than one crate needs to construct them directly.

```rust
pub enum DiffQuery {
    Revision { rev: String, format: DiffFormat },
    FromTo { from: String, to: String, format: DiffFormat },
}

pub enum DiffFormat {
    Patch,
    Stat,
}

pub struct ShowQuery {
    revs: Vec<String>,
}

pub struct StatusQuery {
    filesets: Vec<String>,
}
```

Each query must have one command-spec constructor:

```rust
impl JjDiff {
    pub fn spec_for(&self, query: &DiffQuery) -> JjCommandSpec;
    pub fn load_query(&self, query: &DiffQuery) -> Result<DiffSnapshot, JjDiffError>;
}
```

`JjShow` and `JjStatus` should follow the same pattern once their chunks begin:

```rust
pub fn spec_for(&self, query: &ShowQuery) -> JjCommandSpec;
pub fn load_query(&self, query: &ShowQuery) -> Result<InspectionSnapshot, JjShowError>;

pub fn spec_for(&self, query: &StatusQuery) -> JjCommandSpec;
pub fn load_query(&self, query: &StatusQuery) -> Result<InspectionSnapshot, JjStatusError>;
```

The exact `InspectionSnapshot` name can change. The important contract is that direct root views do
not have to recover their refresh source by parsing a title or change id out of rendered text.

## Command Spec Usage

Every inspection load should build a `JjCommandSpec` first, then execute that spec through the
existing `jk-cli` adapter.

Required argv examples:

```text
jj diff -r @
jj diff -r abc123 --stat
jj diff --from main --to @
jj diff --from main --to @ --stat
jj show abc123
jj show abc123 def456
jj status
jj status src docs
```

Rules:

- The spec argv is the execution source of truth.
- The spec title is the view title source of truth.
- Direct error views use the same spec title as successful views.
- `JjCommandSpec::preview()` remains display-only; execution uses argv.
- The adapter continues to add `--no-pager`, `--color always`, repository metadata, and color env
  cleanup outside the spec argv.
- Read-only inspection specs use `ExecutionMode::RenderReadOnly`, `SafetyClass::ReadOnly`, and
  `RefreshPlan::ReRunSpec`.

## Title And Preview Behavior

Titles should be full command labels, not informal object labels:

- `jk diff -r abc123` root title: `jj diff -r abc123`.
- selected-change `d` title: `jj diff -r <selected change>`.
- marked two-revision diff title: `jj diff --from A --to B`.
- stat title: the same argv plus `--stat`.
- show title: `jj show REV...`.
- status title: `jj status` plus filesets when present.

When future preview or command-history UI appears, it should reuse the same command spec instead of
reformatting the command independently.

## ViewStack Behavior

Inspection screens should be ordinary stack entries:

- `jk diff ...` starts with a root diff view.
- `jk show ...` starts with a root show/details view.
- `jk status ...` starts with a root status view.
- pressing `Backspace` at a root inspection view is a no-op.
- pressing `d` from log pushes a diff child view.
- pressing `S` from log pushes a stat diff child view after the mark resolver exists.
- pressing `Enter` from log pushes a show/details child view.
- pressing `s` from log pushes a status child view.

Do not add a permanent log-left/diff-right pane. The product plan allows focused previews later,
but this bucket should keep one active screen with overlays or modal menus when needed.

## Dependency Order

### Chunk 1: canonical diff query and root CLI

Add `jk diff -r/--from/--to` without changing visual rendering.

Files:

- `crates/jk-cli/src/diff.rs`
- `crates/jk/src/main.rs`
- `crates/jk-core/src/lib.rs` only if the query must be shared immediately

Acceptance:

- `jk diff -r REV` opens a root diff view titled `jj diff -r REV`.
- `jk diff --from A --to B` opens a root diff view titled `jj diff --from A --to B`.
- `jk diff --stat -r REV` opens a root stat view titled `jj diff -r REV --stat`.
- `jk diff REV` still works and resolves to `DiffQuery::Revision`.
- refresh re-runs the original diff query, including `--from/--to` and `--stat`.
- selected-change `d` still opens `jj diff -r <selected change>` through `ViewStack::push`.

### Chunk 2: shared inspection source for refresh

Stop asking `DiffView` for a change id as the only refresh input.

Files:

- `crates/jk/src/main.rs`
- `crates/jk-tui/src/diff_view.rs`
- `crates/jk-core/src/lib.rs`

Acceptance:

- active diff views carry or are paired with their `DiffQuery`.
- retryable direct diff error views refresh the same failed query.
- `DiffSnapshot` no longer has to pretend every rendered diff is one selected change.
- existing file/hunk navigation, folding, search, empty diff, and error rendering still pass.

### Chunk 3: show/details root and pushed view

Add read-only `jj show` inspection.

Files:

- `crates/jk-cli/src/show.rs`
- `crates/jk-cli/src/lib.rs`
- `crates/jk/src/main.rs`
- `crates/jk-tui/src/show_view.rs` or a small shared rendered-inspection view
- `crates/jk-core/src/lib.rs`

Acceptance:

- `jk show REV` opens a root view titled `jj show REV`.
- `Enter` on a log revision pushes `jj show <selected change>`.
- rendered output remains produced by `jj show`.
- diff-like search and page movement work if a shared rendered-inspection view is used.
- no side pane is introduced.

### Chunk 4: status root and pushed view

Add read-only `jj status` inspection.

Files:

- `crates/jk-cli/src/status.rs`
- `crates/jk-cli/src/lib.rs`
- `crates/jk/src/main.rs`
- `crates/jk-tui/src/status_view.rs` or the shared rendered-inspection view
- `crates/jk-core/src/lib.rs`

Acceptance:

- `jk status` opens a root view titled `jj status`.
- `jk status FILESET...` passes filesets through after `status`.
- `s` from log pushes the repository status view.
- refresh re-runs the original status query.
- failed status loads stay visible and retryable inside the TUI.

### Chunk 5: mark-aware inspection resolver

Resolve graph cursor and ordered marks into inspection commands.

Files:

- `crates/jk/src/main.rs`
- `crates/jk-tui/src/log_view.rs`
- `crates/jk-tui/src/log_state.rs`
- `crates/jk-core/src/lib.rs` if ordered mark types are shared

Acceptance:

- no marks plus cursor and `d` resolves to `jj diff -r CURSOR`.
- one mark plus cursor and `d` resolves to `jj diff --from MARK --to CURSOR`.
- two ordered marks and `d` resolve to `jj diff --from MARK0 --to MARK1`.
- `S` uses the same resolver with `--stat`.
- ambiguous non-contiguous selections do not guess silently.

## First Implementation Chunk

Target size: less than 200 LoC changed, excluding generated lockfile churn if none is expected.

Implement only Chunk 1. Keep query types private to `crates/jk-cli/src/diff.rs` unless `main.rs`
must own them for refresh. If `main.rs` needs ownership, define the smallest shared type in
`jk-core` and avoid adding generic inspection abstractions.

Suggested implementation steps:

1. Add `DiffQuery` and `DiffFormat` with `spec_for`, `load_query`, and title tests.
1. Keep `JjDiff::load(change_id)` as a compatibility wrapper around `DiffQuery::Revision`.
1. Change `DiffArgs` to parse `-r/--revision`, `--from`, `--to`, `--stat`, and one optional
   compatibility positional revision.
1. Validate mutually exclusive arguments before entering the terminal.
1. Store the selected root diff query in `main.rs` so refresh does not degrade to `jj diff -r`.
1. Leave `DiffView` rendering unchanged.

Stop there. Do not add `show`, `status`, marks, View Options, or file-list navigation in the
first review unit.

## Tests

Unit tests should cover command construction and argument resolution before adding tapes.

Required tests:

- `DiffQuery::Revision` builds argv `diff -r REV`.
- `DiffQuery::FromTo` builds argv `diff --from A --to B`.
- stat variants append `--stat`.
- titles match the command spec preview for all diff forms.
- `jk diff REV` compatibility resolves to the same query as `jk diff -r REV`.
- `-r` with `--from` or `--to` is rejected.
- lone `--from` or lone `--to` is rejected.
- root diff refresh preserves `--from/--to`.
- selected-change `d` still opens and refreshes `jj diff -r <change>`.
- existing `DiffView` tests for search, folding, retryable errors, and empty diffs still pass.

Suggested validation commands:

```sh
cargo test -p jk-cli diff
cargo test -p jk
cargo test -p jk-tui diff_view
```

## Betamax Evidence Expectations

Chunk 1 should add or update validation tapes if Betamax is available in the checkout:

- direct `jk diff -r REV` opens a root view with title `jj diff -r REV`;
- direct `jk diff --from A --to B` opens a root view with that exact title;
- direct `jk diff --stat --from A --to B` renders stat output as the focused screen;
- pressing `Backspace` in a root diff view keeps the diff visible;
- pressing `r` in a `--from/--to` root diff view re-runs the same comparison;
- log-to-`d` still pushes selected-change diff and `Backspace` returns to the preserved log.

Prefer short validation tapes under `tapes/validation/` over media tapes for this bucket. Media can
wait until the command shape is stable enough for README or website examples.

## Risks

- Treating every diff as a change id will break refresh for `--from/--to`. Store query/source data
  outside the rendered view before expanding direct roots.
- Reformatting titles independently from argv will cause command preview and command history drift.
  Use `JjCommandSpec` as the source.
- Adding a generic rendered-output view too early may hide real diff/show/status differences. Share
  only the state that is genuinely common.
- Adding panes now would fight the product direction. Use focused views first.
- Passing through all `jj diff` flags in the first chunk could over-scope the parser. Start with
  the canonical forms and add passthrough coverage once tests pin down behavior.

## Follow-Up Chunks

- Add patch, stat, summary, name-only, and template display controls through the reusable `V` View
  Options overlay.
- Add file-list navigation as a focused overlay or direct view, not a permanent pane.
- Add richer current-revision/current-file context in the title or status line.
- Add two-revision comparison from ordered marks after the mark model lands.
- Add `show` metadata parsing only if it enables navigation without compromising jj-rendered
  output.
- Add status file actions after mutation preview and command history exist.
- Add command-mode routing so `:diff --from main --to @`, `:show @`, and `:status` open the same
  inspection views.
