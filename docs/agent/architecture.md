# Architecture Guidance For Agents

This document explains the shape of `jk` for agentic tooling. Load it when a change touches command
execution, view behavior, rendering, navigation, search, copying, or terminal lifecycle.

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

Use [`../plan/integration-strategy.md`](../plan/integration-strategy.md) when deciding between:

- rendered output as-is;
- rendered output plus a narrow parser;
- structured output or a purpose-built template;
- shared semantic, rendering, config, template, graph, or style APIs;
- `jj_cli` or `jj_lib`;
- a future UI/RPC API;
- upstream extraction or in-tree work.

Use [`../plan/fragility-register.md`](../plan/fragility-register.md) to record parser assumptions,
inferred structure, duplicated semantics, and the preferred mitigation. Favor contracts that fail
loudly when `jj` changes: compile errors, schema failures, focused parser tests, or snapshot diffs.

Also consider pipeline fragility. The path from `jj` semantics to stdout, ANSI parsing, intermediate
spans, and Ratatui items can lose information even when the rendered output looks correct. A direct
code path that shares `jj` template, config, graph, and styling behavior can be less fragile, but
only if that behavior is actually shared rather than locally copied.

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

Keep modules aligned with user-visible concepts:

- `app.rs` owns terminal event loop, app-level key dispatch, pending key-prefix state, refresh, and
  `ViewEffect` routing. It should read as the app orchestration table of contents and route screen,
  action, service, and view-selection details to their owner modules.
- `app/navigation.rs` owns startup argument parsing, view-stack transitions, top-level view-menu
  actions, diff-format application, and custom log revset mode changes.
- `app/mode_input.rs` owns active modal and prompt key reducers, including copy-menu opening and
  prompt acceptance/cancellation behavior.
- `app/action_lifecycle.rs` owns action-menu opening, prompt-to-preview setup, immediate action
  execution such as default fetch and new-from-trunk, and confirmed action result handling.
- `app/action_flow.rs` owns common action-preview key flow between pending result panes and action
  lifecycle confirmation.
- `app/services.rs` owns the app side-effect seam for tests and forwards app-owned jj/view effects
  through one narrow service surface instead of scattering runner fields through `App`.
- `app_screen.rs` owns app-level modal and prompt state, including help, copy, view-format,
  action-menu, role-prompt, text-prompt, action-preview/result, push-remote, operation-action, and
  working-copy navigation screens. It projects the current `InteractionMode` into status-line text
  and `tui::Overlay` values.
- `app_status.rs` owns status-line construction, status kind, title/message/hint storage, and
  per-view item-count wording.
- `action_output.rs` owns action preview/result body projection, scroll state, visible-line
  calculation, and preview/result key handling. `app.rs` decides what an accepted or cancelled
  action means; `action_output.rs` decides how output panes move.
- `command.rs` owns binding metadata and the command/effect vocabulary shared between app-level
  dispatch and individual views.
- `jj_actions.rs` owns preview-first `jj` action and mutation plans, including argv construction,
  labels, preview summaries, direct run methods, and fallback result wording for user-confirmed
  mutation flows.
- `jj.rs` owns view-spec command construction, direct process helpers, diff-format arguments, and
  command/navigation target provenance.
- `jj_syntax.rs` owns exact revset/fileset/string quoting helpers and argv label helpers shared by
  `jj_actions.rs` and related command builders.
- `jj_rows.rs` owns selectable rendered row models, row loaders, narrow metadata templates, metadata
  pairing, row grouping, resolve-entry parsing, file-list path preservation, and conversion from
  rendered ANSI output into Ratatui row items. It may call the `jj.rs` process helpers, but it
  should not own command identity or navigation provenance.
- `graph.rs` owns the default/log graph view, graph-row selection, graph search, and graph-to-detail
  navigation.
- `show.rs` and `diff.rs` own their view behavior and should stay distinct even when they share
  document mechanics.
- `sticky_file_view.rs` owns shared rendered-file document mechanics for show, diff, status,
  file-show, and operation-detail surfaces: sticky heading projection, file jumping, scroll state,
  search, and render helpers.
- `rendered_jj.rs` owns lightweight structure over rendered jj lines, including file heading
  detection and sticky projection inputs.
- `search.rs`, `selection.rs`, `copy.rs`, and `clipboard.rs` own narrow support concepts and should
  not accumulate view policy.
- `tui.rs` owns shared chrome only: layout, status/header rendering, overlays, and modal
  presentation.

Add a module only when it gives a real concept a local home. Do not split code just to make files
smaller if the resulting reader path becomes less direct.

## Screen And Action Contracts

Every active app screen should have one explicit owner for each part of its contract:

- Keys: `app.rs` owns global dispatch and mode transitions; view modules own view-local bindings;
  `action_output.rs` owns scrolling keys inside action preview/result output.
- Screen state: `app_screen.rs` owns modal and prompt variants. New prompt or overlay state should
  start there unless it is view-local state that belongs in a view module.
- Overlay projection: `app_screen.rs` converts screen state to `tui::Overlay`; `tui.rs` renders the
  overlay chrome without deciding app behavior.
- Status projection: `app_status.rs` constructs durable ready/error status lines from the active
  view; `app_screen.rs` supplies transient prompt status text while a mode is active.
- Command execution: `jj_actions.rs` owns action-plan command contracts for confirmed mutation
  flows; `jj.rs` owns the shared `jj` process helpers and view-spec command construction.
  `app/action_lifecycle.rs` owns when action commands are run, how results refresh or reveal views,
  and what status/result screen follows.
- View behavior: view modules execute `ViewCommand` into `ViewEffect`; `app.rs` routes global
  effects such as opening screens, copying, pushing views, refreshing, or changing search state to
  the app submodule that owns the detailed policy.

Future UI packets should name the smallest owner that matches the contract. For example, a new
action-result scroll key belongs in `action_output.rs`; a new modal projection belongs in
`app_screen.rs` plus `tui.rs`; a new graph navigation behavior belongs in `graph.rs` or
`view_state.rs`; and only the orchestration glue should land in `app.rs`.

## View Architecture

Feature views should expose a small, boring surface:

- `load` constructs a view from a `ViewSpec` or equivalent input.
- `render` draws the view using already-owned state.
- `bindings` returns static bindings for view-local commands.
- `execute` translates `ViewCommand` plus `CommandContext` into a `ViewEffect`.
- `refresh` reloads external state and preserves or clamps local navigation.
- `clamp` keeps selection or scroll state valid for the current content.

The app owns global mode. Views should not know about help, copy menus, view menus, the stack, or
terminal polling. A view may request an effect; `app.rs` decides how that effect changes global
state.

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

When parsing graph rows, be conservative. If the output cannot be understood, the view should
degrade gracefully instead of inventing a target.

## Rendering Rules

Rendered jj text should keep its Ratatui spans and styles. Avoid converting styled lines into plain
strings except for narrow matching or parsing tasks.

For show/diff documents:

- Sticky file headings come from rendered jj output, not regenerated labels.
- Sticky projections should preserve enough blank-line context to still look like jj output.
- Search highlights should layer on top of displayed lines without changing the underlying document.
- Scroll math should be saturating and clamp against document length.

Keep shared chrome in `tui.rs`. Do not let each view invent its own title bar, status line, overlay
style, or modal layout unless the app design genuinely changes.

## Terminal And Process Boundaries

Terminal lifecycle belongs in `app.rs` and Ratatui setup. View and process command execution helpers
belong in `jj.rs`; preview-first mutation command plans belong in `jj_actions.rs`. Clipboard
integration belongs in `clipboard.rs`.

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
