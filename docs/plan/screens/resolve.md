# Resolve Screen

## Purpose

The resolve screen is the conflict-oriented utility surface.

## Source Command

- `jj resolve`

## View Model

- list-first surface
- conflict details may open into dedicated follow-up views or external tools

## Priority

Priority 2. Resolve should follow status and file screens, because it needs exact conflict and path
semantics before guided resolution can be safe.

## Core Information

- conflicted paths
- enough context to identify the conflict target quickly
- obvious next actions for resolution

## Primary Interactions

- move between conflict items
- inspect selected conflict context
- launch resolution-related flows
- refresh
- go back

## Selection Model

- selection unit: conflict item or conflicted path
- conflict identity, path state, and available resolution actions must be semantic data
- multi-select is not a first-wave need; resolve flows should start with one conflict at a time

## Interaction Details

- Movement: move by conflict item.
- Inspect: open selected conflict context in a dedicated detail view or external tool.
- Launch resolution: resolution flows should clearly state which path/conflict they target.
- Refresh: preserve selected conflicted path when possible.
- Completion: after a resolution action, refresh and either stay on remaining conflicts or return to
  status if all conflicts are resolved.

## Shortcut Candidates

- `j`/`k`, arrows: move conflict selection
- `Enter`: inspect conflict context
- `r`: refresh
- `R`: launch resolution flow
- `y`: copy path
- `h`, `Esc`: back

## Integration Notes

Conflict state is too semantic to duplicate casually. Use rendered output for the first list view,
but prefer `jj` APIs or structured data before building guided resolution flows that need exact
conflict contents, path state, or merge behavior.

The preferred contract exposes conflicted path, conflict kind, available resolution state,
renderable row text, and launchable resolution targets together.

## Acceptance Criteria

- conflict state is visible and actionable
- the screen stays focused on resolution rather than becoming a generic file browser
- guided resolution does not infer conflict semantics from rendered prose
