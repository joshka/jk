# Screen Specs

This document defines the stable screens `jk` should grow around. A screen is a persistent,
navigable surface with a clear purpose. It is not merely a command result; it is a place in the app.

Detailed per-screen specs live in [`screens/`](screens/). Priority order and dependency guidance
live in [`screen-priority.md`](screen-priority.md).

## Screen Template

Use this template when adding or refining a screen:

- purpose
- source `jj` command(s)
- why this is a screen instead of a prompt or passthrough
- selection unit
- entry points
- primary actions
- back/exit behavior
- refresh behavior
- empty/error states
- shortcut candidates
- open questions
- acceptance criteria

## Stable Core Screens

### Log

- purpose: home screen for stack inspection and navigation.
- source `jj` command(s): `jj`, `jj log`.
- why screen: this is the primary navigation surface.
- selection unit: logical revision item, not raw rendered line.
- entry points: startup default, `L`, `J`, back-to-home actions.
- primary actions: open `show`, open `diff`, search, copy ids, refresh.
- back/exit behavior: quitting exits; back from deeper views returns here.
- refresh behavior: preserve selection when possible and clamp honestly when history shape changes.
- shortcut candidates: `j`, `k`, `g`, `G`, `l`, `s`, `d`, `/`, `n`, `N`.
- acceptance criteria: graph feels stable, item-based, and cheap to scan.

### Show

- purpose: inspect one change with commit context and file-aware scrolling.
- source `jj` command(s): `jj show`.
- why screen: it is the natural read drill-down from log.
- selection unit: scroll offset with active file projection.
- entry points: from log, direct startup, from diff.
- primary actions: file-to-file jump, search, copy, switch to diff, back.
- back/exit behavior: returns to previous screen.
- refresh behavior: reload output and preserve/clamp scroll position.
- acceptance criteria: sticky file context works and does not feel like a second pane.

### Diff

- purpose: inspect patch content for one change.
- source `jj` command(s): `jj diff`.
- why screen: it is the natural patch-focused drill-down from log or show.
- selection unit: scroll offset with active file projection.
- entry points: from log, direct startup, from show.
- primary actions: file jump, search, copy, switch to show, back.
- back/exit behavior: returns to previous screen.
- refresh behavior: reload output and preserve/clamp scroll position.
- acceptance criteria: patch navigation is cheap and file boundaries stay clear.

### Status

- purpose: inspect working-copy state and choose the next action.
- source `jj` command(s): `jj status`.
- why screen: this is a high-frequency triage surface, not just a command dump.
- selection unit: probably line-oriented or section-oriented, depending on final design.
- entry points: dedicated shortcut and command mode.
- primary actions: refresh, file-related actions, fetch/push entry points, return to log.
- back/exit behavior: back returns to prior screen, often log.
- acceptance criteria: makes “what changed locally?” cheaper than shell ping-pong.

### Operation Log

- purpose: inspect operation history for audit and recovery.
- source `jj` command(s): `jj op log`.
- why screen: recovery is important enough to deserve a stable home.
- selection unit: logical operation item.
- entry points: dedicated shortcut; command mode once that surface exists.
- primary actions: open op-show, open op-diff, undo, redo, restore, revert.
- back/exit behavior: back returns to previous screen.
- acceptance criteria: recovery paths are visible and confidence-building.

## Likely Utility Screens

These are not the product center, but they fit the single-view model well:

- bookmark list
- file list
- file show
- file search
- file annotate
- resolve list
- tag list
- workspace root
- workspace list
- compact help
- compact keymap

These screens should stay focused, low-chrome, and history-aware. They should not turn into a
multi-pane repository dashboard.

## Non-Screens By Default

These should not become persistent screens unless experience proves otherwise:

- generic command palette as the main model
- broad config editing
- Git/Gerrit plumbing
- sparse management
- signature management
- advanced graph surgery such as `arrange`

They may still be supported through command mode or guided flows.

## Screen Dependencies

Use [`screen-priority.md`](screen-priority.md) for the current priority order. In short:

1. core loop: log, show, diff, help/keymap
1. daily triage: status, operation log, bookmarks
1. file and conflict utility: file list/show, resolve, annotate/search later
1. low-frequency utility: tags, workspaces, evolog, interdiff
1. guided mutation attachments

Mutation flows should attach to existing screens where possible instead of inventing mutation-first
screens.
