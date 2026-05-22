# Help And Keymap Screens

## Purpose

Help and keymap surfaces make `jk` discoverable without turning command mode into the only teaching
mechanism.

## View Model

- compact dedicated screens or overlays
- low chrome
- scrollable if needed
- not a free-form command palette

## Priority

Priority 0. Help/keymap should arrive with the core loop so `jk` remains discoverable while the
shortcut set is still changing.

## Scope

The help surface should explain:

- the core navigation loop
- currently available view-local actions
- the distinction between read surfaces and guided mutation flows
- mutation safety tiers: direct, prompted, confirmed, and preview-first
- graph action menu flow from log and which actions stay preview-first
- common direct workflows such as fetch and new-from-trunk

The keymap surface should explain:

- exact active bindings
- view-specific differences where relevant
- action menu (`a`) and current selection contract on log (`Space`)

## Primary Interactions

- open from anywhere
- press a listed command key to execute it and close the menu
- dismiss cleanly without executing a command

## Interaction Details

- Context: help opened from a view should show only commands available for that view.
- Currentness: keymap content should be generated from binding metadata where possible, not copied
  into static prose that can drift.
- Scope: help should explain the active screen, navigation loop, search, copy, refresh, and
  available guided flows. It should not become a replacement for `jj help`.
- Grouping: command rows should be grouped by user operation rather than raw key order.
- Consistency: help should make shared key meanings obvious across screens, especially movement,
  search, refresh, copy, open, back, and mutation prefixes or confirmations.
- Dismissal: closing help returns to the exact prior view state without changing selection or
  scroll.
- Preview-first flows: help should show that rewrite-like operations require preview output and role
  prompt preparation before execution.

## Shortcut Candidates

- `?`: open help/keymap
- `/`: search within help if help becomes long
- listed command keys: execute that command and close the menu
- `a` (log): open the log action menu
- `Esc`, `q`, `?`: dismiss

## Acceptance Criteria

- a new user can learn the app without reading raw CLI docs
- power users can confirm exact bindings quickly
