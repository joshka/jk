# Contributing Tests And Snapshots

This repository uses focused unit tests plus `insta` snapshots for wrapper/render regressions.

## When To Add Snapshot Tests

Add snapshot tests when output text shape is part of behavior, especially for:

- wrapper headers, summaries, and follow-up tips
- spacing and section breaks in rendered views
- command output decoration routing for a command/subcommand

Prefer regular assertions for pure token-building or branching logic.

## Updating Snapshots

1. Run the targeted snapshot test first:

   ```bash
   cargo test app::tests::snapshot_renders_status_wrapper_view
   ```

1. If snapshot output changed intentionally, update snapshots:

   ```bash
   INSTA_UPDATE=always cargo test
   ```

1. Re-run tests without update flags to verify clean state:

   ```bash
   cargo test
   ```

## Review Expectations

- Keep snapshot diffs small and scoped to the behavior change.
- Confirm changed lines match intended UX text, spacing, and summaries.
- Add or update targeted unit assertions when logic changes drive snapshot updates.
- Avoid batching unrelated snapshot churn with functional changes.
