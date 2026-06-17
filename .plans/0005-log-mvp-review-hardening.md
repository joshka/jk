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

1. Add Betamax coverage for Right expansion and Left collapse.
1. Refresh while expanded and preserve expansion when the selected change still exists.
1. Collapse cleanly when the expanded change disappears after refresh.
1. Handle expansion on rows without details.
1. Check selection and scroll behavior when expanded content wraps.
1. Verify title and status truncation on narrow terminals.

## Done When

- The MVP change is either intentionally kept together or split into reviewable pieces.
- `just test`, `just clippy`, `just lint-md`, and `just betamax` pass for the reviewed slice.
- The remaining edge cases are either fixed or explicitly moved to later numbered plans.
