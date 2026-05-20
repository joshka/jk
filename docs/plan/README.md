# Planning Index

This directory is the durable planning surface for `jk`. It exists to separate product planning from
implementation details and to keep roadmap decisions close to the codebase.

The planning model has fourteen parts:

1. [`command-inventory.md`](command-inventory.md): broad `jj` command coverage matrix and first-pass
   classification.
1. [`recommended-approach.md`](recommended-approach.md): opinionated defaults and tradeoffs that can
   be approved or overridden directly.
1. [`implementation-slices.md`](implementation-slices.md): ordered execution slices with acceptance
   criteria and validation expectations.
1. [`next-implementation-slices.md`](next-implementation-slices.md): delegation-ready continuation
   packets after the completed initial implementation slices.
1. [`view-model.md`](view-model.md): cross-screen UI framing, especially around hybrid views and the
   role of inline detail versus split layouts.
1. [`interaction-model.md`](interaction-model.md): shared shortcut vocabulary, mutation safety
   tiers, revset/view modes, and post-action feedback rules.
1. [`integration-strategy.md`](integration-strategy.md): how `jk` should choose between rendered
   output, subprocess parsing, structured output, `jj_cli`, `jj_lib`, future RPC APIs, and upstream
   extraction.
1. [`integration-feasibility.md`](integration-feasibility.md): concrete source findings from the
   adjacent `jj` checkout and the recommended source-integration spike.
1. [`fragility-register.md`](fragility-register.md): current and planned soft agreements with `jj`
   that need tests, degradation behavior, or stronger contracts.
1. [`screen-priority.md`](screen-priority.md): priority-ordered screen roadmap and implementation
   focus by stage.
1. [`screens.md`](screens.md): stable screen definitions and screen-by-screen specs.
1. [`workflows.md`](workflows.md): user workflow groupings that cut across command namespaces.
1. [`phases.md`](phases.md): implementation cut lines and suggested order of work.
1. [`open-questions.md`](open-questions.md): unresolved decisions that need explicit answers before
   implementation grows.

## Planning Rules

- Treat `jj` command parity as a coverage matrix, not as the product model.
- A `jj` command may map to a screen, a guided flow, a prompt, a confirmation step, a shortcut, or a
  passthrough command-mode entry.
- Stable screen definitions come before shortcut design.
- Keyboard shortcuts should use the shared vocabulary in
  [`interaction-model.md`](interaction-model.md). Final bindings come after command ownership and
  screen transitions are clear.
- Read-first surfaces should become native screens before mutation-heavy flows.
- Hybrid screens are preferred over fixed pane dashboards. Expand in place first, split only when
  the split clearly improves terminal ergonomics.
- Treat rendered `jj` output as the default presentation source, but record every fragile parser or
  inferred structure in [`fragility-register.md`](fragility-register.md).
- Prefer code or structured contracts over parsed CLI output for semantic data. Parsing terminal
  output should be presentation-adjacent, narrow, and tested.
- Prefer contracts that expose semantic data and renderable view information together, so `jk` can
  preserve user-configured `jj` views without reverse-engineering terminal text.
- When a planning decision conflicts with rendered-`jj` fidelity, prefer `jj` fidelity unless there
  is a clear, evidenced product benefit.
- Treat external-tool versus upstream integration as a question to test through implementation, not
  as a settled premise.

## How To Use This Plan

When considering a new capability:

1. Find the related `jj` command in [`command-inventory.md`](command-inventory.md).
1. Check the default recommendation in [`recommended-approach.md`](recommended-approach.md).
1. Check whether it is already an executable slice in
   [`implementation-slices.md`](implementation-slices.md).
1. For post-Slice-12 work, check the next delegation packet in
   [`next-implementation-slices.md`](next-implementation-slices.md).
1. Check whether the view should be single-surface, inline-expanded, or optionally split in
   [`view-model.md`](view-model.md).
1. Check shared shortcut and mutation-safety policy in
   [`interaction-model.md`](interaction-model.md).
1. Choose the least duplicative non-lossy integration that preserves the needed semantics using
   [`integration-strategy.md`](integration-strategy.md).
1. Check whether the current `../jj` source already exposes a usable code path in
   [`integration-feasibility.md`](integration-feasibility.md).
1. If the feature parses output or duplicates `jj` behavior, update
   [`fragility-register.md`](fragility-register.md).
1. Check the feature's priority and dependencies in [`screen-priority.md`](screen-priority.md).
1. Confirm which workflow it belongs to in [`workflows.md`](workflows.md).
1. Check whether it already has a screen home in [`screens.md`](screens.md).
1. Check whether the current phase in [`phases.md`](phases.md) says it should be native, guided, or
   passthrough.
1. If the answer is still ambiguous, add the question to [`open-questions.md`](open-questions.md)
   instead of guessing in code.

## Promotion To Issues

This directory is the source of truth for planning. GitHub issues should be created only for
concrete execution units after the relevant screen, workflow, or phase definition is stable enough
to implement.

## Upstream Feedback Loop

The planning docs should preserve evidence about where `jk` is easy or hard to keep aligned with
`jj`. If a screen works well by showing rendered output as-is for presentation, that supports the
external-tool path. If a feature requires parsing underspecified output for semantic state,
duplicating `jj` internals, or performing multi-step operations that should be one transaction,
record that as evidence for stronger `jj` APIs, extracted libraries, structured output, RPC support,
or possible in-tree work.

The goal is not to prove the original upstream TUI proposal right or wrong in advance. The goal is
to build the tool in a way that makes the answer clearer over time.

## Detailed Specs

Per-screen specs live in [`screens/`](screens/). Current first-pass files:

- [`screens/log.md`](screens/log.md)
- [`screens/show.md`](screens/show.md)
- [`screens/diff.md`](screens/diff.md)
- [`screens/status.md`](screens/status.md)
- [`screens/operation-log.md`](screens/operation-log.md)
- [`screens/help-keymap.md`](screens/help-keymap.md)
- [`screens/bookmarks.md`](screens/bookmarks.md)
- [`screens/files.md`](screens/files.md)
- [`screens/resolve.md`](screens/resolve.md)
- [`screens/tags.md`](screens/tags.md)
- [`screens/workspaces.md`](screens/workspaces.md)

Per-workflow specs live in [`workflows/`](workflows/). Current first-pass files:

- [`workflows/inspect.md`](workflows/inspect.md)
- [`workflows/rewrite.md`](workflows/rewrite.md)
- [`workflows/sync.md`](workflows/sync.md)
- [`workflows/recover.md`](workflows/recover.md)
- [`workflows/refs-and-workspaces.md`](workflows/refs-and-workspaces.md)
