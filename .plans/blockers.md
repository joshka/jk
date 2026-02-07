# Blockers and Workarounds

## Active blockers

None currently.

## Resolved blockers

### 1Password signing popup during `jj` rewrites

- Symptom: `jj describe`/rewrite operations failed with SSH signing error from 1Password.
- Workaround used: set repo-scoped config `signing.behavior = drop`.
- Command used:
  - `jj --no-pager config set --repo signing.behavior drop`

## End-of-run review checklist

- List unresolved blockers and impact.
- Confirm fallback behavior exists for partially implemented flows.
- Confirm gold command set coverage level with concrete status notes.
