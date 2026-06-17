# JJ Integration Cleanup

## Why

The MVP uses a temporary stdout/template bridge to get both CLI-equivalent rendered output and
semantic records. That is acceptable for the first slice only if the boundary stays narrow and the
missing upstream contract remains explicit.

## Cleanup Targets

1. Keep rendered output handling isolated from semantic record parsing.
1. Preserve user templates, graph symbols, color behavior, and revset defaults.
1. Avoid teaching the TUI to reimplement `jj` display decisions.
1. Replace shell-out or template parsing when `jj-cli` / `jj-lib` exposes a stable better contract.

## Questions To Answer

- What exact data does `jk` need for log navigation, expansion, and diff targeting?
- Which fields are semantic state, and which bytes are presentation that should stay opaque?
- Can one `jj` call provide both renderable output and semantic records without drift?
- Where should parser fixtures live so future `jj` output changes fail clearly?

## Done When

- The current integration boundary is documented in code or tests, not only in plans.
- Parser assumptions are covered by realistic fixtures.
- Any move toward `jj-cli` / `jj-lib` reduces duplicated display behavior rather than hiding it.
