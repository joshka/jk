# Log MVP Review And Hardening

## Why

The log-first MVP working copy is now a substantial change. Before expanding scope, decide whether
it is reviewable as one coherent product slice or whether cleanup, tooling, and test-support pieces
should be split out.

## Current Inputs

- `0004-log-first-mvp.md` is the product boundary.
- Unit tests cover the latest key behavior, including Right toggling and Left collapsing.
- The Betamax tape still exercises Enter expansion but not Right/Left.
- The implementation intentionally stops at manual refresh and inline log expansion.

## Review Pass

1. Read the diff as a maintainer, not as an implementer.
1. Identify any mixed concerns that should become separate changes.
1. Confirm `jk-cli`, `jk-core`, and `jk-tui` boundaries are earning their keep.
1. Check whether local dogfood install instructions are good enough to run the MVP outside
   `cargo run`.
1. Keep notes in this plan only until they become accepted docs, tests, or source comments.

## Hardening Queue

- [x] Add Betamax coverage for Right expansion and Left collapse.
- [x] Refresh while expanded and preserve expansion when the selected change still exists.
- [x] Collapse cleanly when the expanded change disappears after refresh.
- [x] Handle expansion on rows without details.
- [x] Check selection and scroll behavior when expanded content wraps.
- [x] Verify title and status truncation on narrow terminals.

## Review Result

Keep the log-first MVP as one reviewable product slice for now. The current crate split is earning
its keep: `jk-cli` owns the temporary `jj` bridge, `jk-core` owns the narrow semantic snapshot, and
`jk-tui` owns interaction and rendering state without reimplementing `jj` graph presentation.

The hardening pass tightened expansion state rather than adding new product scope. Empty-detail rows
no longer enter a hidden expanded mode, refresh preserves expansion only when the expanded change
still has details, and refresh collapses when the expanded change disappears.

## Validation

- `just test`
- `just clippy`
- `just lint-md`
- `just betamax`

## Done When

- The MVP change is either intentionally kept together or split into reviewable pieces.
- `just test`, `just clippy`, `just lint-md`, and `just betamax` pass for the reviewed slice.
- The remaining edge cases are either fixed or explicitly moved to later numbered plans.
