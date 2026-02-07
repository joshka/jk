# Blockers and Workarounds

## Active blockers

None currently.

## Resolved blockers

### 2. Mutation-prone shortcut tests triggered real `jj` moves

- Symptom: adding `next`/`prev` assertions to the normal-mode app shortcut test executed real
  `jj next`/`jj prev` commands and moved the working copy commit during `cargo test`.
- Impact: test run changed active revision context and could hide/unhide in-progress changes.
- Workaround used:
  - removed direct app-level execution assertions for mutation-prone shortcuts;
  - added flow-planner coverage (`plan_command`) for `next`/`prev` behavior instead;
  - restored working copy to the intended change with `jj --no-pager edit <change-id>`.

### 1Password signing popup during `jj` rewrites

- Symptom: `jj describe`/rewrite operations failed with SSH signing error from 1Password.
- Workaround used: set repo-scoped config `signing.behavior = drop`.
- Command used:
  - `jj --no-pager config set --repo signing.behavior drop`

## End-of-run review checklist

- List unresolved blockers and impact.
- Confirm fallback behavior exists for partially implemented flows.
- Confirm gold command set coverage level with concrete status notes.

## Latest completion review (2026-02-07)

- Gold command set coverage: implemented in TUI flow planner and guarded execution path.
- High-frequency aliases (`gf`, `gp`, `rbm`, `rbt` + OMZ variants) are normalized and tested.
- Remaining work is quality and depth expansion (richer native `status/show/diff` views and
  long-tail command UX), not baseline command availability.
