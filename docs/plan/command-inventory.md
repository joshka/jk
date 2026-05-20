# Command Inventory

This document maps the `jj` CLI surface to current `jk` homes. The goal is not to make every command
a first-class screen immediately. The goal is to decide, deliberately, which commands are already
shipped, which deserve guided flows, and which should stay passthrough or deferred.

## Classification

- `native screen`: persistent read surface or central navigation home.
- `utility screen`: focused list/detail screen that supports a narrower task.
- `guided flow`: prompt and confirmation-driven flow where `jk` adds context or safety over raw CLI
  use.
- `direct action`: immediate action with clear output and refresh, without a preview/confirm loop.
- `passthrough`: run with the regular `jj` CLI outside `jk` until a native flow ships.
- `planned`: intended `jk` home or flow, but not shipped yet.
- `defer`: intentionally out of near-term scope for `jk`.

`passthrough` is a holding pattern, not a promise to build in-app command mode. Commands stay there
when `jk` adds little value beyond explicit native flows. `planned` means the workflow is named and
anticipated, but the home does not exist yet.

## Prioritization Signals

The first-pass priority order here is informed by three inputs:

- the current `jk` product direction
- the old `prototype` branch and VHS artifacts
- common `jj` aliases collected in the Oh My Zsh `jj` plugin work, including `bookmark`, `commit`,
  `diff`, `git fetch`, `git push`, `log`, `new`, `rebase`, `restore`, `root`, `split`, `squash`, and
  `status`

That alias surface is not a perfect usage survey, but it is a useful signal for what experienced
users consider common enough to shorten aggressively.

## Command Mode Policy

`jk` should not try to mirror the whole `jj` CLI in-app yet. Low-frequency or advanced commands can
remain passthrough when the app adds no clearer home. Dangerous commands need a stronger guided flow
or stay deferred until the exact target, preview, and recovery story is obvious.

## Core Read Surface

- `log`: `native screen`. Current home screen.
- `show`: `native screen`. Drill-down from graph.
- `diff`: `native screen`. Drill-down from graph or show.
- `status`: `native screen`. High-frequency triage surface.
- `evolog`: `planned`. Likely a later utility screen.
- `interdiff`: `passthrough`. Can become a derived detail view later.
- `version`: `passthrough`. No dedicated screen needed.

## Navigation And Working Copy

- `new`: `guided flow`. Exact-parent graph flow is shipped; the trunk shortcut is a separate direct
  action.
- `edit`: `guided flow`. Exact-row graph launch.
- `next`: `guided flow`. Preview-first `--edit` movement relative to `@`.
- `prev`: `guided flow`. Preview-first `--edit` movement relative to `@`.
- `commit`: `guided flow`. Targets `@`.
- `describe`: `guided flow`. Targets exact graph rows or `@` from status.
- `metaedit`: `passthrough`. No dedicated `jk` home yet.
- `root`: `planned`. Utility screen when root/workspace context is worth surfacing.
- `workspace`: `planned`. Focused list and subflows would need a dedicated home.
- `sparse`: `passthrough`. Advanced CLI-first surface.

## Rewrite And Recovery

- `rebase`: `guided flow`. Shipped with preview-first command planning.
- `squash`: `guided flow`. Shipped with explicit source and destination roles.
- `split`: `planned`. Good candidate, but not shipped.
- `abandon`: `guided flow`. Shipped from exact graph targets with strong confirmation.
- `duplicate`: `planned`. Useful, but not first-wave.
- `parallelize`: `passthrough`. Niche until the rewrite model matures.
- `simplify-parents`: `passthrough`. Niche advanced flow.
- `absorb`: `guided flow`. Shipped, but intentionally narrow.
- `restore`: `guided flow`. Shipped from exact graph or file contexts.
- `revert`: `guided flow`. Shipped for exact revisions only.
- `undo`: `guided flow`. Global repo-cursor recovery from the operation log.
- `redo`: `guided flow`. Global repo-cursor recovery from the operation log.
- `operation log`: `native screen`. Central recovery surface.
- `operation show`: `utility screen`. Drill-down from operation log.
- `operation diff`: `utility screen`. Drill-down from operation log.
- `operation restore`: `guided flow`. Recovery from an exact selected operation-log id.
- `operation revert`: `guided flow`. Recovery from an exact selected operation-log id.
- `operation integrate`: `passthrough`. Specialized.
- `operation abandon`: `defer`. Too dangerous for early UI.

## Bookmarks, Tags, And Related Refs

- `bookmark list`: `utility screen`. Strong fit from prototype ideas.
- `bookmark set`: `guided flow`. Shipped from graph exact rows and status (`=`) using current `@`.
- `bookmark create`: `guided flow`. Shipped from graph exact rows and status (`@`) as bookmark name
  prompts.
- `bookmark move`: `guided flow`. Shipped from graph exact rows and status (`@`) as bookmark name
  prompts.
- `bookmark rename`: `planned`. Not a current home.
- `bookmark delete`: `guided flow`. Shipped from bookmark rows in bookmarks view; local-only and
  confirmation-worthy.
- `bookmark forget`: `planned`. Useful, but not shipped.
- `bookmark track`: `planned`. Tracking-state metadata still needs a stronger contract.
- `bookmark untrack`: `planned`. Same tracking-state gap as track.
- `bookmark advance`: `passthrough`. Probably not a first-class flow.
- `tag`: `planned`. Lower-frequency utility surface.

## Files And Resolve

- `file list`: `utility screen`. Useful companion to show and diff.
- `file show`: `utility screen`. Drill-down from file list.
- `file search`: `planned`. Useful if scoped well.
- `file annotate`: `planned`. Later read surface.
- `file track`: `planned`. Needs exact path ownership from status or file list.
- `file untrack`: `planned`. Same exact-path requirement as track.
- `file chmod`: `planned`. Lower-frequency file action.
- `resolve`: `utility screen`. List-first conflict surface.

## Git And Remote Sync

- `git fetch`: `direct action`. Common and low-risk enough to stay direct.
- `git push`: `guided flow`. Preview and confirmation matter.
- other `jj git` commands: `passthrough`. Likely too broad for early native UI.

## Inspection But Probably Not Core

- `bisect`: `defer`. Large workflow, later if ever.
- `arrange`: `defer`. Likely incompatible with the current single-view model without more design.
- `diffedit`: `defer`. Editor-centric and high-complexity.
- `fix`: `passthrough`. Can remain CLI-first.
- `config`: `passthrough`. Not a product-center task.
- `sign`: `passthrough`. Advanced.
- `unsign`: `passthrough`. Advanced.
- `gerrit`: `defer`. Host-specific.
- `util`: `defer`. Explicitly not a first-class surface.
- `help`: `passthrough`. `jk` needs its own help, not a full CLI mirror.

## Current Planning Bias

Short version:

- Shipped native screens: `log`, `show`, `diff`, `status`, `operation log`.
- Shipped utility screens: `bookmarks`, `file list/show`, `resolve`, `operation show`,
  `operation diff`.
- Shipped guided flows: `edit`, `next`, `prev`, `describe`, `commit`, `rebase`, `squash`, `abandon`,
  `restore`, `revert`, `operation restore`, `operation revert`, `absorb`,
  `bookmark set/create/move/delete`, `undo`, `redo`, `git push`.
- Shipped direct actions: `jj git fetch` and `jj new trunk`.
- Planned utility or guided work: `evolog`, `root`, `workspace`, `split`, `duplicate`,
  `bookmark rename/forget/track/untrack`, `tag`, `file search`, `file annotate`,
  `file track/untrack/chmod`.
- Everything else stays passthrough or deferred until the core loop is strong.

When a command needs native structure, check [`integration-strategy.md`](integration-strategy.md)
before deciding whether to use rendered output, a narrow parser, structured output, `jj_cli`,
`jj_lib`, or an upstream API.
