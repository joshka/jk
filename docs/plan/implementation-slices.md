# Implementation Slices

These slices are ordered so a future Codex session can take one slice, implement it, validate it,
and return with a clear result. Each slice should be small enough for one focused change unless the
code reveals an unexpected dependency.

## Slice 0: Source Integration Spike

Goal: determine whether `jk` can render high-fidelity log rows through `jj_cli`/`jj_lib` instead of
deepening stdout/ANSI parsing.

Recommended scope:

- inspect and prototype against `jj_cli` template, formatter, graphlog, revset utility, and command
  setup paths;
- inspect and prototype against `jj_lib` repo, revset, graph, diff, view, and transaction semantics
  only where needed;
- try to capture row identity, graph text, styled spans, and searchable text from code paths rather
  than reparsing terminal output;
- compare against subprocess `jj log` for default config and at least one customized template or
  graph setting;
- update integration docs and the fragility register with the result.

Acceptance criteria:

- the spike states whether `jj_cli` is viable for log row rendering in `jk` today;
- the required dependency surface is named explicitly;
- private assumptions, copied code, or unstable public APIs are listed;
- the next log-row implementation path is clear: code-native, structured-template, or narrowed
  subprocess parser;
- no production behavior changes unless the prototype is clean enough to keep.

Validation:

- a small prototype test or scratch module if useful;
- comparison notes against `jj log`;
- updated docs for the chosen next step.

## Slice 1: Log Row Contract

Goal: make log selection, search, copy, refresh, and future actions operate on stable row semantics
rather than visual lines.

Recommended scope:

- introduce or refine a log row model with change id, commit id, graph/display spans, selectable
  state, searchable text, and action identity;
- use the Slice 0 result to choose code-native rendering, structured-template data, or a narrowed
  subprocess parser;
- keep rendered jj output as the presentation baseline even if the semantic path changes;
- preserve current log rendering and movement behavior;
- record parser assumptions in the fragility register if new assumptions are added.

Acceptance criteria:

- selection moves by revision item, not raw line;
- copy actions use row semantics;
- refresh preserves selected row by change id when possible;
- search can highlight rendered row text without changing row identity;
- malformed or non-selectable rows remain visible and non-actionable.

Validation:

- focused unit tests for row parsing/modeling;
- snapshot or inline tests for representative graph output;
- `cargo test`.

## Slice 2: View Mode Infrastructure

Goal: add named log view modes without changing the default home behavior.

Recommended scope:

- add view mode enum/state for default work, trunk work, recent work, all/repo overview, and custom
  revset;
- keep default mode as plain user-configured `jj`/`jj log`;
- implement mode switching in log state;
- display active mode in title/status chrome;
- preserve selected change id when switching modes if possible.

Recommended command shapes:

- default work: no app-owned revset override;
- trunk work: `trunk().. | trunk()`;
- recent work: `latest(mutable(), 20) | @ | trunk()`;
- all/repo overview: `all()`;
- custom revset: user-provided string.

Acceptance criteria:

- default startup output remains unchanged;
- switching modes reloads log with the selected mode;
- current mode is visible;
- selection preservation is attempted by semantic identity;
- invalid custom revset reports an error without corrupting the current view.

Validation:

- unit tests for mode command construction;
- app tests for mode switching and selection preservation;
- `cargo test`.

## Slice 3: Generated Help And Keymap

Goal: make shortcut consistency visible and reduce drift between bindings and help text.

Recommended scope:

- generate help/keymap rows from command binding metadata where practical;
- show global bindings and current-view bindings first;
- include direct actions and preview/confirmation actions as distinct categories;
- include view-mode switching once Slice 2 exists;
- keep help as overlay or dedicated screen without changing prior view state.

Acceptance criteria:

- `?` opens help/keymap from every view;
- closing help restores prior selection/scroll;
- keymap content reflects actual bindings;
- direct versus preview/confirmation actions are visually distinguishable.

Validation:

- unit tests for binding-to-help projection;
- snapshot tests for compact help rendering;
- `cargo test`.

## Slice 4: Direct `jj git fetch`

Goal: add the first low-risk direct mutation flow.

Recommended scope:

- add a fetch command action available from status and log/global context;
- run `jj git fetch` with clear command output or error capture;
- refresh the current screen after success;
- keep failure output visible without changing selection unexpectedly.

Acceptance criteria:

- fetch can be triggered with the recommended key from status and log/global context;
- success refreshes the current view;
- failure displays useful command output;
- no confirmation is required for the default command shape;
- unusual remote selection remains out of scope and can fall back to command mode.

Validation:

- command-construction tests;
- app-level tests with mocked command runner if available;
- manual run in a disposable jj repo if practical;
- `cargo test`.

## Slice 5: Direct `jj new trunk`

Goal: add the first common direct graph mutation flow.

Recommended scope:

- resolve/validate `trunk()` as the target for direct new-from-trunk;
- run `jj new trunk` or equivalent command shape;
- refresh the log after success;
- move selection to the new working-copy change when possible;
- show an undo hint after success.

Acceptance criteria:

- action is direct when trunk is exact;
- ambiguous or missing trunk does not run blindly;
- success makes the new working-copy change visible;
- failure leaves the prior view readable;
- status/help text makes `jj undo` available as the recovery path.

Validation:

- command-construction tests;
- target-validation tests;
- app refresh/selection tests with mocked output;
- manual disposable-repo test if practical;
- `cargo test`.

## Slice 6: Status Screen First Pass

Goal: make working-copy triage useful without overcommitting to file actions.

Recommended scope:

- add status screen entry and refresh;
- render `jj status` output faithfully;
- support section-aware scroll or simple document scroll;
- add fetch entry point if Slice 4 exists;
- defer mutation-capable file actions until exact path contracts exist.

Acceptance criteria:

- status opens from a stable shortcut and command mode;
- refresh preserves scroll/section when possible;
- clean, dirty, and conflict output remain readable;
- fetch can be launched from status when implemented;
- no file mutation action depends on parsed prose.

Validation:

- rendering/scroll tests;
- command-construction tests;
- `cargo test`.

## Slice 7: Operation Log First Pass

Goal: add recovery visibility before risky mutation flows.

Recommended scope:

- add operation log screen;
- model operation rows with exact operation id where possible;
- support movement, copy operation id, refresh, and drill-down placeholders;
- defer restore/revert until previews and confirmations exist.

Acceptance criteria:

- operation log opens from shortcut/command mode;
- movement is operation-item based;
- copy operation id works when id is known;
- refresh preserves selected operation when possible;
- restore/revert actions are not available until exact ids and confirmations exist.

Validation:

- operation-row parsing/model tests;
- screen movement tests;
- `cargo test`.

## Slice 8: Bookmark List First Pass

Goal: make bookmark state legible before bookmark mutations.

Recommended scope:

- add bookmark list screen;
- support movement, copy bookmark name/target, refresh, and open target where possible;
- model bookmark exact names separately from rendered labels;
- defer set/move/delete/track actions until exact tracking state is available.

Acceptance criteria:

- bookmarks open from shortcut/command mode;
- selected bookmark has exact name where available;
- copy name/target works;
- refresh preserves selected bookmark name;
- mutation actions are unavailable or clearly previewed until semantic state exists.

Validation:

- bookmark row tests;
- screen movement/copy tests;
- `cargo test`.

## Slice 9: File List And File Show

Goal: add file inspection around show/diff/status without mutation-capable path actions.

Recommended scope:

- add file list view for a revision or current context;
- add file show document view for selected path;
- preserve exact path identity;
- support search, copy path, refresh, and back;
- defer track/untrack/chmod until exact path/fileset contracts exist.

Acceptance criteria:

- file list opens from show/diff/status context where exact target exists;
- file show opens selected exact path;
- search and copy work;
- refresh preserves selected path when possible;
- no mutation action depends on display labels.

Validation:

- path identity tests;
- file-list movement tests;
- file-show document tests;
- `cargo test`.

## Slice 10: Action Menu And Multi-Select

Goal: prepare for risky graph-attached mutation flows.

Recommended scope:

- add action menu or command-role prompt for selected revision(s);
- add multi-select only in log and only where action roles need it;
- keep role assignment explicit for rebase/squash-like flows;
- surface safety tier and preview requirement in the action UI.

Acceptance criteria:

- one or more selected revisions carry exact identities;
- action menu shows only actions valid for the current selection;
- ambiguous roles require explicit prompt;
- selection survives refresh where possible;
- no risky action executes without preview/confirmation.

Validation:

- selection model tests;
- action availability tests;
- role assignment tests;
- `cargo test`.

## Slice 11: Push Preview Flow

Goal: add the first publication flow with preview and confirmation.

Recommended scope:

- launch from status/bookmark/log context;
- determine destination bookmark/remote explicitly;
- show preview command/output before running;
- confirm before push;
- refresh status/bookmarks/log after success.

Acceptance criteria:

- push never runs directly from a vague context;
- preview shows destination and affected refs;
- confirmation is required;
- success/failure output remains visible;
- refresh preserves meaningful context.

Validation:

- command-construction tests;
- preview-state tests;
- mocked app-flow tests;
- manual disposable-remote test only if practical;
- `cargo test`.

## Slice 12: Rebase Preview Flow

Goal: add the first risky graph rewrite using the action-selection model.

Recommended scope:

- select source revision(s) and destination explicitly;
- preview the command shape and graph effect where possible;
- confirm before execution;
- refresh log after success;
- show undo path.

Acceptance criteria:

- source and destination roles are explicit;
- action does not infer roles from visual order when ambiguous;
- preview/confirmation required;
- success refreshes and preserves or moves selection to the affected stack;
- failure leaves prior graph readable.

Validation:

- role-selection tests;
- command-construction tests;
- preview/confirmation tests;
- manual disposable-repo test if practical;
- `cargo test`.

## Execution Rule

Before starting a slice:

1. Read the owning screen/workflow docs.
1. Check `recommended-approach.md` for defaults.
1. Check `interaction-model.md` for shortcut and safety policy.
1. Check `integration-strategy.md` for semantic/rendering contract expectations.
1. Check `integration-feasibility.md` for source-backed API candidates.
1. Add or update fragility-register entries for new parser assumptions.

After finishing a slice:

1. Run focused tests.
1. Run `cargo test`.
1. Run Panache checks if docs changed.
1. Report any remaining ambiguity as a specific decision, not a broad concern.
