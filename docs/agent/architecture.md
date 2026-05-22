# Architecture Guidance For Agents

This document explains the shape of `jk` for agentic tooling. Load it when a change touches command
execution, view behavior, rendering, navigation, search, copying, or terminal lifecycle.

This is the canonical active guidance for current structure and ownership. Use
[`workflow.md`](workflow.md) for packet shape and completion criteria, and use
[`../reference/README.md`](../reference/README.md) for the current product-facing reference surface.

## Product Boundary

`jk` is a Ratatui TUI over the `jj` command-line interface. It should feel like an interactive
wrapper around the user's configured `jj`, not like a separate repository model.

The most important architecture rule is that rendered `jj` output is the default presentation
source:

- Preserve user templates, colors, graph symbols, diff styles, and jj wording.
- Shell out to `jj` for rendered views and command behavior instead of reconstructing repo state
  through a lower-level model.
- Parse only the small amount of structure required for selection, navigation, sticky file context,
  search, and copy actions.
- Prefer code or structured contracts over parsed CLI output for semantic state.
- Prefer honest pass-through behavior over silently normalizing or reformatting jj output.

If a change would make `jk` disagree with the `jj` CLI, treat that as a design problem unless the
user explicitly asked for an app-level behavior that differs.

## Integration Contracts

Rendered `jj` output is the default presentation source, but it is not the preferred source of
semantic data. When a feature depends on underspecified output shape, reconstructed terminal output,
or duplicated `jj` behavior, treat that as an explicit integration choice.

Choose the strongest honest contract that preserves `jj` fidelity without recreating `jj` locally:

- rendered output as-is when the feature only needs presentation;
- rendered output plus a narrow parser when the need is presentation-adjacent and tightly tested;
- structured output or purpose-built narrow templates when `jj` can expose exact data directly;
- shared semantic, rendering, config, template, graph, or style APIs when the feature needs both
  exact semantics and user-configured display fidelity;
- `jj_cli` or `jj_lib` when command behavior, repository state, revset/fileset semantics,
  transactions, or template/config behavior would otherwise be reimplemented in `jk`;
- future upstream UI-facing contracts when external tools need stronger guarantees than subprocess
  rendering can provide.

Record parser assumptions, inferred structure, duplicated semantics, and the preferred mitigation in
the owning feature docs, tests, or source comments. Favor contracts that fail loudly when `jj`
changes: compile errors, schema failures, focused parser tests, or snapshot diffs.

Also consider pipeline fragility. The path from `jj` semantics to stdout, ANSI parsing, intermediate
spans, and Ratatui items can lose information even when the rendered output looks correct. A direct
code path that shares `jj` template, config, graph, and styling behavior can be less fragile, but
only if that behavior is actually shared rather than locally copied.

The key rationale is that high-fidelity `jj` presentation is hard to preserve through subprocesses
without either rerunning `jj` for every semantic need or duplicating `jj`'s template, config, graph,
and formatting logic inside `jk`. When that pressure appears, treat it as evidence for stronger
`jj`-side abstractions rather than as a reason to let `jk` silently grow its own shadow model of
`jj`.

When a feature needs semantic meaning that `jj` already has internally, do not make rendered CLI
output the first design choice. Prefer structured output, narrow machine-oriented templates, shared
semantic/rendering/config APIs, `jj_cli`, `jj_lib`, or an upstream API. Use rendered-output parsing
for semantic state only when the scope is narrow, tested, and recorded as a soft agreement.

The strongest API shape exposes both semantic and view information. A rendered row often contains
interactive pieces such as change id, commit id, graph position, labels, and styled template output.
`jk` should avoid recomputing those same display decisions when `jj` can expose them through shared
code.

Before duplicating template parsing, config interpretation, revset/fileset semantics, graph layout,
transaction behavior, conflict modeling, or bookmark tracking behavior, check whether rendered
output, shared semantic/rendering APIs, `jj_cli`, `jj_lib`, structured output, or an upstream API
would be more honest and maintainable.

## Module Ownership

Shape the code as feature roots plus shared infrastructure. The first question for any move is not
"what kind of code is this?" It is "what product concept owns this decision?" Put each rule where a
maintainer would look when the user-visible concept changes.

Feature roots should own the decisions that change together for a user-visible surface:

- view state and view-local bindings;
- row models, row interpretation, and rendered-output assumptions;
- selection, search, copy, refresh, reveal, and drill-down behavior;
- feature-specific action availability and action target resolution;
- feature tests and user-visible contracts.

Shared infrastructure should own only cross-cutting mechanics that two feature owners can use
without understanding each other's domain:

- `app`: event loop, mode dispatch, navigation, action lifecycle, refresh/reveal orchestration, and
  services;
- `jj`: process execution, syntax quoting, command construction, and view specs;
- `actions`: command plans and execution contracts after a feature has chosen an action;
- `tui`: shared chrome, modal rendering, menus, status hints, and theme primitives;
- `selection`, `search`, `clipboard`, and similar helpers when the rule is domain-neutral.

Avoid letting `jj`, `actions`, `rendered_rows`, `menus`, `tui`, or `view_state` become dumping
grounds for feature policy. A shared module is the right home only when two feature owners would use
the code without learning each other's product rules.

That gives a practical split between feature policy and shared mechanics. Feature roots answer
questions such as "what does this surface show, select, copy, or recover from?" and "when is this
action available for the selected rows?" Shared modules answer questions such as "how is an exact
revset quoted?", "how does a modal list render?", or "how is an already-built action preview
executed?" If a rule changes because `operation_log`, `bookmarks`, `status`, `files`, or `log`
changes as a product surface, prefer the feature owner even when the code shape resembles an
existing shared helper.

A plausible current shape is:

- `log` owns the default graph/log view, log rows, log selection, and log-local action availability.
- `operation_log` owns operation rows, undo/redo/restore/revert target policy, operation detail
  navigation, and operation-log tests.
- `bookmarks` owns bookmark rows, bookmark metadata and pairing, bookmark mutation target policy,
  and bookmark tests.
- `status` owns status rows, exact path action policy, status navigation, and status tests.
- `files` owns file-list, file-show, and file-action policy that is specific to file-oriented
  surfaces.
- `documents` owns reusable rendered-document mechanics such as sticky headings, rendered line
  structure, and document search when that lowers reader burden more than separate helpers.
- `actions` owns cross-view action command plans, not view-specific availability. For example,
  rewrite plans can own argv, preview, and execution for rebase, squash, and absorb after a feature
  has selected targets; the log or status feature owns whether those actions are offered from its
  selected rows.

The exact names can change. The invariant is that a maintainer can start from a feature such as
`operation_log` or `bookmarks` and find the local row model, view behavior, action availability, and
tests without first understanding global buckets.

Do not introduce a `slices/` or migration-phase folder. Refactoring should move toward feature roots
plus shared infrastructure:

```text
src/
  app/
    mod.rs
    dispatch.rs
    effects.rs
    input/
    navigation/
    reducers/
    services/
    status_line.rs
    actions/
      entry/
      preview/
      completion/
      input/
      pane.rs
      shared.rs

  log/
    mod.rs
    rows/
    view/
    tests.rs

  operation_log/
    mod.rs
    rows.rs
    actions.rs
    detail.rs
    view/
    tests.rs

  bookmarks/
    mod.rs
    rows/
    actions/
    targets/
    view/
    tests.rs

  status/
    mod.rs
    view/
    rows.rs
    actions.rs
    tests.rs

  files/
    mod.rs
    list/
    show/

  documents/
    mod.rs
    rendered/
    sticky/

  actions/
    rewrite/
    working_copy/
    files/
    git_sync/
    describe/
    abandon/

  jj/
    command/
    process/
    syntax.rs
    view_spec/

  menus/
    model/
    revision_actions/
  tui/
    chrome.rs
    overlays/
    status_hints.rs
    theme.rs
```

This sketch is a direction, not a required final tree. A refactor packet should move code only when
the move shortens the maintainer path for a concrete behavior. For example, `operation_log` should
eventually be the starting point for operation row interpretation, operation selection/copy, and
undo/redo/restore/revert target policy. `bookmarks` should be the starting point for bookmark row
state, local/remote pairing, mutation target resolution, and bookmark-specific action availability.

Shared action modules should begin after a feature has chosen a target. `actions/rewrite/mod.rs` can
own argv, preview, and run contracts for rebase, squash, and absorb. The log, status, or bookmark
feature still owns whether that action is offered from its selected rows. Apply the same split to
working-copy, file, sync, describe, and abandon flows.

Current ownership:

- `app/mod.rs` owns terminal event loop, app-level key dispatch, pending key-prefix state, refresh,
  and the normal-key entry point. It should read as the app orchestration table of contents and
  route screen, action, service, and view-selection details to their owner modules.
- `app/dispatch.rs` owns prefix dispatch and binding execution flow after the event loop has chosen
  a key path.
- `app/effects.rs` owns `ViewEffect` interpretation.
- `app/navigation/mod.rs` owns startup parsing, view-stack transitions, top-level view-menu actions,
  diff-format application, and custom log revset mode changes through `startup`, `stack`, and
  `view_menu`.
- `app/input/mod.rs` owns active modal and prompt key reducers, including copy-menu opening and
  prompt acceptance/cancellation behavior.
- `app/actions/mod.rs` owns action-menu opening, prompt-to-preview setup, immediate action execution
  such as default fetch and new-from-trunk, and confirmed action result handling.
- `app/actions/input/mod.rs` owns common action-preview key flow between pending result panes and
  action confirmation.
- `app/actions/shared.rs` owns only the shared status/result/reveal helpers used by preview and
  completion families. It is intentionally shared because it centralizes identical lifecycle wording
  and refresh policy, not because it is a generic action bucket.
- `app/services/mod.rs` owns the app side-effect seam for tests. App submodules call that narrow
  service surface directly for jj/view effects, and `App` keeps only the small wrappers that must
  couple those effects to current app-owned state such as the active `ViewState`.
- `modes/mod.rs` owns app-level modal and prompt state, including help, copy, view-format,
  action-menu, role-prompt, text-prompt, action-preview/result, push-remote, operation-action, and
  working-copy navigation screens. It projects the current `InteractionMode` into status-line text
  and `tui::Overlay` values through `projection.rs`.
- `app/status_line.rs` owns status-line construction, status kind, title/message/hint storage, and
  per-view item-count wording.
- `app/actions/pane.rs` owns action preview/result body projection, scroll state, visible-line
  calculation, and preview/result key handling. `app/mod.rs` decides what an accepted or cancelled
  action means; `app/actions/pane.rs` decides how output panes move.
- `command/mod.rs` owns binding metadata and the command/effect vocabulary shared between app-level
  dispatch and individual views.
- `menus/mod.rs` owns shared menu vocabulary, safety markers, role prompts, action-menu items, and
  follow-up payload models. Feature roots own whether a selected row offers an action and which
  target values it carries.
- `actions/mod.rs` owns preview-first `jj` action and mutation plans after target selection. It is
  intentionally left as a top-level vocabulary and re-export boundary for action families rather
  than as a place for feature-owned availability rules.
- `jj/mod.rs` owns view-spec command construction, direct process helpers, diff-format arguments,
  and command/navigation target provenance.
- `jj/syntax.rs` owns exact revset/fileset/string quoting helpers and argv label helpers shared by
  `actions` and related command builders.
- `rendered_rows/mod.rs` owns only shared row-helper mechanics. It should not own command identity,
  navigation provenance, document loading, or feature-specific row policy.
- `operation_log/mod.rs` owns the operation-log feature root. `operation_log/view/mod.rs` owns the
  operation-log surface, selection/copy/search, and recovery availability. `operation_log/rows.rs`
  owns rendered operation-log row grouping, operation-id metadata parsing and pairing, and metadata
  drift tests. `operation_log/actions.rs` owns undo/redo and exact operation restore/revert argv
  construction, preview wording, and run contracts, while `actions/mod.rs` re-exports the app-facing
  names. `operation_log/detail.rs` owns the detail document surface.
- `bookmarks/mod.rs` owns the bookmarks feature root. `bookmarks/view/mod.rs` owns bookmark
  selection/copy/search and action availability; `bookmarks/rows/mod.rs` owns bookmark row metadata
  and local/remote state classification; `bookmarks/targets/mod.rs` owns safe bookmark mutation
  target resolution; `bookmarks/actions/mod.rs` owns bookmark mutation argv construction, preview
  summaries, exact-name quoting, and rename validation.
- `log/mod.rs` owns the default/log view, log row loading, log-row selection, log search, and
  log-to-detail navigation. `log/view/mod.rs` owns the log surface; `log/rows/mod.rs` owns rendered
  `jj log` row grouping, revision metadata pairing, compact log context, and the `LogItem` row
  contract.
- `status/mod.rs` owns the status feature root. `status/view/mod.rs` owns status
  selection/search/copy/refresh, `status/rows.rs` owns rendered status rows and exact path policy,
  and `status/actions.rs` owns status-file action target contracts.
- `show/mod.rs` and `diff/mod.rs` own their view behavior and should stay distinct even when they
  share document mechanics.
- `documents/mod.rs` owns shared rendered-file document mechanics for show, diff, status, file-show,
  and operation-detail surfaces: loading rendered document lines, sticky heading projection, file
  jumping, scroll state, search, and render helpers.
- `documents/rendered/mod.rs` owns lightweight structure over rendered jj lines, including file
  heading detection and sticky projection inputs.
- `search/mod.rs`, `selection.rs`, and `clipboard.rs` own narrow support concepts and should not
  accumulate view policy. `menus/model/copy.rs` owns copy-menu payload vocabulary.
- `tui/mod.rs` owns shared chrome only: layout, status/header rendering, overlays, and modal
  presentation.

Add a module only when it gives a real concept a local home. Do not split code just to make files
smaller if the resulting reader path becomes less direct.

## Screen And Action Contracts

Every active app screen should have one explicit owner for each part of its contract:

- Keys: `app/mod.rs` owns event-loop entry and normal-mode routing; `app/dispatch.rs` owns prefix
  dispatch; view modules own view-local bindings; `app/actions/pane.rs` owns scrolling keys inside
  action preview/result output.
- Screen state: `modes/mod.rs` owns modal and prompt variants. New prompt or overlay state should
  start there unless it is view-local state that belongs in a view module.
- Overlay projection: `modes/mod.rs` converts screen state to `tui::Overlay`; `tui/mod.rs` renders
  the overlay chrome without deciding app behavior.
- Status projection: `app/status_line.rs` constructs durable ready/error status lines from the
  active view; `modes/mod.rs` supplies transient prompt status text while a mode is active.
- Command execution: `actions/mod.rs` owns or re-exports action-plan command contracts for confirmed
  mutation flows; feature-owned action modules such as `operation_log/actions.rs` and
  `bookmarks/actions/mod.rs` own their local argv, preview, and run contracts. `jj/mod.rs` owns the
  shared `jj` process helpers and view-spec command construction. `app/actions/mod.rs` owns when
  action commands are run, how results refresh or reveal views, and what status/result screen
  follows.
- View behavior: view modules execute `ViewCommand` into `ViewEffect`; `app/mod.rs` routes global
  effects such as opening screens, copying, pushing views, refreshing, or changing search state to
  the app submodule that owns the detailed policy.

Future UI packets should name the smallest owner that matches the contract. For example, a new
action-result scroll key belongs in `app/actions/pane.rs`; a new modal projection belongs in
`modes/mod.rs` plus `tui/mod.rs`; a new log navigation behavior belongs in `log/view/mod.rs` or
`view_state/mod.rs`; and only the orchestration glue should land in `app/mod.rs`.

## View Architecture

Feature views should expose a small, boring surface:

- `load` constructs a view from a `ViewSpec` or equivalent input.
- `render` draws the view using already-owned state.
- `bindings` returns static bindings for view-local commands.
- `execute` translates `ViewCommand` plus `CommandContext` into a `ViewEffect`.
- `refresh` reloads external state and preserves or clamps local navigation.
- `clamp` keeps selection or scroll state valid for the current content.

The app owns global mode. Views should not know about help, copy menus, view menus, the stack, or
terminal polling. A view may request an effect; `app/mod.rs` and `app/effects.rs` decide how that
effect changes global state.

Prefer explicit effect values over callbacks or shared mutable app context. The current `ViewEffect`
shape is intentionally small because it keeps command flow auditable.

## Navigation Rules

`jj` workflows are change-centric. Graph rows may display both commit IDs and change IDs, but
navigation to `show` or `diff` should prefer change IDs from the selected row.

Maintain these distinctions:

- Use change IDs for app navigation targets.
- Keep commit IDs available for copy actions when jj printed them.
- Preserve original command-line args for direct startup views.
- When a navigated detail view has an explicit target, display a shortened target in app labels
  without changing the real jj arguments.

When parsing log rows, be conservative. If the output cannot be understood, the view should degrade
gracefully instead of inventing a target.

## Rendering Rules

Rendered jj text should keep its Ratatui spans and styles. Avoid converting styled lines into plain
strings except for narrow matching or parsing tasks.

For rendered file-oriented documents:

- Sticky file headings come from rendered jj output, not regenerated labels.
- Sticky projections should preserve enough blank-line context to still look like jj output.
- Search highlights should layer on top of displayed lines without changing the underlying document.
- Scroll math should be saturating and clamp against document length.

Keep shared chrome in `tui/mod.rs`. Do not let each view invent its own title bar, status line,
overlay style, or modal layout unless the app design genuinely changes.

## Terminal And Process Boundaries

Terminal lifecycle belongs in `app/mod.rs` and Ratatui setup. View and process command execution
helpers belong in `jj/mod.rs`; preview-first mutation command plans belong in `actions/mod.rs` until
a narrower feature owner is the better reader path. Clipboard integration belongs in `clipboard.rs`.

Make side effects visible:

- A function that shells out to `jj` should say so through its module or name.
- A function that writes to the clipboard should not look like pure formatting.
- A function that refreshes state should handle errors without corrupting the current view.

Avoid background work, global state, threads, async runtimes, or persistent processes unless there
is a concrete reason and a clear ownership model.

## Compatibility Bias

Favor jj CLI compatibility over clever app features. Before adding behavior that depends on a
specific jj output shape, ask what happens when the user has custom templates, different colors,
unusual graph symbols, or a future jj version.

Document any residual assumption near the parser or behavior that depends on it. A small explicit
parser with tests is preferable to broad string rewriting.
