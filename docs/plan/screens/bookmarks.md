# Bookmark Screen

## Purpose

The bookmark screen is a focused utility view for bookmark state and bookmark-related actions.

## Source Commands

- `jj bookmark list`
- related guided flows: `set`, `create`, `move`, `rename`, `delete`, `forget`, `track`, `untrack`

## View Model

- list-first screen
- inline detail for selected bookmark is acceptable
- optional local split could be justified later if it materially helps tracking remote/local state

## Priority

Priority 1. Bookmarks are useful once status and sync flows begin to matter. They should stay a
focused ref-state utility surface.

## Core Information

- bookmark name
- local versus tracked/remote state
- target revision identity
- stale/divergent or other high-signal tracking state

## Primary Interactions

- move between bookmarks
- copy bookmark names or targets
- launch set-related flows
- launch rename/delete/track/untrack flows
- refresh
- go back

## Selection Model

- selection unit: bookmark item
- bookmark name and target identity must be exact for actions
- tracking, remote, stale, and divergent state should be semantic fields, not parsed decorations

## Interaction Details

- Movement: move by bookmark item.
- Inline detail: selected bookmark may expand to show local target, remote target, tracking state,
  divergence, and suggested actions.
- Set/move/create: flows should accept a selected bookmark and selected revision target when
  launched from graph or bookmark screens.
- Rename/delete/forget: destructive or confusing ref actions need confirmation.
- Track/untrack: tracking actions should describe local and remote names explicitly.
- Refresh: preserve selected bookmark name when possible.

## Shortcut Candidates

- `j`/`k`, arrows: move bookmark selection
- `Enter`, `s`, `l`, `Right`: open target in show when available
- `bc`: create flow from graph or status
- `r`: refresh
- `br`: rename local bookmark flow
- `x`: delete local bookmark flow
- `t`: track/untrack flow
- `y`: copy bookmark name or target
- `h`, `Left`: back

## Integration Notes

Use rendered bookmark output for inspection. Local, remote, tracking, stale, and divergent state can
be subtle; actions that depend on exact bookmark semantics should use structured data or `jj_lib`
instead of duplicating bookmark logic from rendered text.

The preferred contract exposes bookmark name, local target, remote target, tracking relationship,
conflict/divergence state, and renderable row segments together.

## Acceptance Criteria

- bookmarks become legible without leaving `jk`
- actions feel attached to bookmark context rather than to a global command launcher
- exact tracking semantics are not inferred from decoration text
