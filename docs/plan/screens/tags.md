# Tag Screen

## Purpose

The tag screen is a low-frequency utility surface for tag inspection and mutation.

## Source Commands

- `jj tag list`
- related guided flows: `tag set`, `tag delete`

## View Model

- simple list-first screen
- inline detail if helpful, but likely no need for a split

## Priority

Priority 3. Tags are useful but lower frequency than bookmarks and working-copy triage.

## Primary Interactions

- navigate tags
- copy tag names or targets
- launch set/delete flows
- refresh
- go back

## Selection Model

- selection unit: tag item
- tag name and target revision identity must be exact for mutation flows
- tag actions are single-item by default

## Interaction Details

- Movement: move by tag item.
- Inline detail: selected tag may show target identity and related labels if useful.
- Set/delete: tag mutation flows should confirm exact tag name and target.
- Refresh: preserve selected tag name when possible.

## Shortcut Candidates

- `j`/`k`, arrows: move tag selection
- `Enter`: open target in log/show when available
- `s`: set flow
- `d`: delete flow
- `y`: copy tag name or target
- `r`: refresh
- `h`, `Left`: back

## Integration Notes

Use rendered tag output for inspection. Tag actions should use exact tag names and target revision
identity from structured data or a narrow template before becoming mutation-capable.

## Acceptance Criteria

- tag management exists as a coherent utility surface without pretending tags are a core navigation
  model
- tag actions do not infer names or targets from ambiguous display text
