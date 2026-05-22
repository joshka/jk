# Source Cleanup Audit

This audit records mechanical measurements for the maintainability cleanup wave. Measurements are
evidence for choosing what to read next; they are not a refactoring order by themselves.

Captured: `2026-05-22 11:11:16 PDT`.

## Current Measurements

- Rust source total from `find src -maxdepth 4 -name '*.rs' -print0 | xargs -0 wc -l`: `37,055`
  lines.
- Visibility entries from `rg "^pub(\\(| |$)|pub\\(crate\\)|pub\\(super\\)" src -n | wc -l`: `737`.
- Inline `#[cfg(test)] mod tests { ... }` blocks from `rg -U`: `24`.

## Largest Files

The largest files are now dominated by tests. The list below is still a prompt for reading, not an
automatic split queue.

```text
1196 src/app/tests/bookmark_actions.rs
 778 src/app/tests/working_copy_actions.rs
 648 src/bookmarks/tests.rs
 608 src/log/tests.rs
 596 src/app/tests/command_navigation.rs
 531 src/app/tests/detail_restore_actions.rs
 529 src/app/tests/rewrite_actions.rs
 485 src/app/tests/sync_actions.rs
 466 src/app/tests/support/services.rs
 465 src/bookmarks/rows/tests.rs
 421 src/tui/tests.rs
 413 src/bookmarks/actions/plan.rs
 408 src/jj/view_spec/mod.rs
 389 src/bookmarks/targets/resolver.rs
 377 src/operation_log/detail.rs
 369 src/workspaces/rows.rs
 355 src/app/tests/operation_actions.rs
 351 src/app/tests/describe_commit_actions.rs
 336 src/jj/view_spec/tests.rs
 327 src/terminal_process/mod.rs
```

## What The Measurements Mean Now

- The cleanup wave has largely shifted source-size pressure away from mixed production owners and
  into explicit test modules.
- The remaining large production files are mostly coherent roots or feature-local owners, so future
  work should start with reader pain rather than with file size alone.
- Remaining inline test blocks should move only when the production module becomes harder to scan or
  when a sibling test file would better preserve feature locality.

## Candidate Future Reads

- `src/bookmarks/actions/plan.rs`: still the largest production action-plan file.
- `src/jj/view_spec/mod.rs`: still a central shared boundary with startup and navigation policy.
- `src/bookmarks/targets/resolver.rs`: still a dense feature-owned target policy file.
- `src/operation_log/detail.rs`: still a larger rendered-document feature surface.
- `src/workspaces/rows.rs`: still a non-trivial feature-owned row and metadata pairing surface.
- `src/terminal_process/mod.rs`: still a shared effect boundary worth re-reading before future
  process-handling changes.

## Process Guidance

- Re-measure before choosing another broad cleanup packet.
- Prefer a no-move decision when a large file is coherent and well owned.
- Use focused validation for the surface being changed, then rerun the broader gate when deciding
  whether the cleanup objective is actually complete.
