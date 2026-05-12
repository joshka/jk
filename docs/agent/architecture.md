# Architecture Guidance For Agents

This document explains the shape of `jk` for agentic tooling. Load it when a
change touches command execution, view behavior, rendering, navigation, search,
copying, or terminal lifecycle.

## Product Boundary

`jk` is a Ratatui TUI over the `jj` command-line interface. It should feel like
an interactive wrapper around the user's configured `jj`, not like a second
implementation of jj repository logic.

The most important architecture rule is that rendered `jj` output is canonical:

- Preserve user templates, colors, graph symbols, diff styles, and jj wording.
- Shell out to `jj` for repository information instead of reconstructing repo
  state through a lower-level model.
- Parse only the small amount of structure required for selection, navigation,
  sticky file context, search, and copy actions.
- Prefer honest pass-through behavior over silently normalizing or reformatting
  jj output.

If a change would make `jk` disagree with the `jj` CLI, treat that as a design
problem unless the user explicitly asked for an app-level behavior that differs.

## Module Ownership

Keep modules aligned with user-visible concepts:

- `app.rs` owns terminal event loop, app-level modes, key dispatch, modal state,
  view stack, refresh, and cross-view transitions.
- `command.rs` owns binding metadata and the command/effect vocabulary shared
  between app-level dispatch and individual views.
- `jj.rs` owns `jj` command construction, view specs, diff-format arguments, and
  conversion from rendered CLI output into the minimal structures `jk` needs.
- `graph.rs` owns the default/log graph view, graph-row selection, graph search,
  and graph-to-detail navigation.
- `show.rs` and `diff.rs` own their view behavior and should stay distinct even
  when they share document mechanics.
- `sticky_file_view.rs` owns shared show/diff document scrolling, file jumping,
  sticky heading projection, and document search.
- `rendered_jj.rs` owns lightweight structure over rendered jj lines, including
  file heading detection and sticky projection inputs.
- `search.rs`, `selection.rs`, `copy.rs`, and `clipboard.rs` own narrow support
  concepts and should not accumulate view policy.
- `tui.rs` owns shared chrome only: layout, status/header rendering, overlays,
  and modal presentation.

Add a module only when it gives a real concept a local home. Do not split code
just to make files smaller if the resulting reader path becomes less direct.

## View Architecture

Feature views should expose a small, boring surface:

- `load` constructs a view from a `ViewSpec` or equivalent input.
- `render` draws the view using already-owned state.
- `bindings` returns static bindings for view-local commands.
- `execute` translates `ViewCommand` plus `CommandContext` into a `ViewEffect`.
- `refresh` reloads external state and preserves or clamps local navigation.
- `clamp` keeps selection or scroll state valid for the current content.

The app owns global mode. Views should not know about help, copy menus, view
menus, the stack, or terminal polling. A view may request an effect; `app.rs`
decides how that effect changes global state.

Prefer explicit effect values over callbacks or shared mutable app context. The
current `ViewEffect` shape is intentionally small because it keeps command flow
auditable.

## Navigation Rules

`jj` workflows are change-centric. Graph rows may display both commit IDs and
change IDs, but navigation to `show` or `diff` should prefer change IDs from
the selected row.

Maintain these distinctions:

- Use change IDs for app navigation targets.
- Keep commit IDs available for copy actions when jj printed them.
- Preserve original command-line args for direct startup views.
- When a navigated detail view has an explicit target, display a shortened
  target in app labels without changing the real jj arguments.

When parsing graph rows, be conservative. If the output cannot be understood,
the view should degrade gracefully instead of inventing a target.

## Rendering Rules

Rendered jj text should keep its Ratatui spans and styles. Avoid converting
styled lines into plain strings except for narrow matching or parsing tasks.

For show/diff documents:

- Sticky file headings come from rendered jj output, not regenerated labels.
- Sticky projections should preserve enough blank-line context to still look
  like jj output.
- Search highlights should layer on top of displayed lines without changing the
  underlying document.
- Scroll math should be saturating and clamp against document length.

Keep shared chrome in `tui.rs`. Do not let each view invent its own title bar,
status line, overlay style, or modal layout unless the app design genuinely
changes.

## Terminal And Process Boundaries

Terminal lifecycle belongs in `app.rs` and Ratatui setup. Command execution
belongs in `jj.rs`. Clipboard integration belongs in `clipboard.rs`.

Make side effects visible:

- A function that shells out to `jj` should say so through its module or name.
- A function that writes to the clipboard should not look like pure formatting.
- A function that refreshes state should handle errors without corrupting the
  current view.

Avoid background work, global state, threads, async runtimes, or persistent
processes unless there is a concrete reason and a clear ownership model.

## Compatibility Bias

Favor jj CLI compatibility over clever app features. Before adding behavior
that depends on a specific jj output shape, ask what happens when the user has
custom templates, different colors, unusual graph symbols, or a future jj
version.

Document any residual assumption near the parser or behavior that depends on
it. A small explicit parser with tests is preferable to broad string rewriting.
