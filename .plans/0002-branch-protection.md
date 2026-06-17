# Branch Protection

## Why

The reset line should only merge changes that pass the release baseline and Markdown checks. CI is
already split into stable job names so GitHub branch protection can require them for automerge.

## Current Repo State

- `.github/workflows/ci.yml` exposes:
  - `Check`
  - `Markdown`
  - `MSRV`
- The workflow also runs on `merge_group`, so the same statuses can protect merge queue batches.
- GitHub branch protection previously returned a plan/visibility `403`, so the repo-side workflow
  shape is ready even if the setting cannot be applied yet.

## Steps

1. Once GitHub settings allow it, enable branch protection or rulesets for `main`.
1. Require these status checks:
   - `Check`
   - `Markdown`
   - `MSRV`
1. Enable automerge or merge queue using those same checks.
1. Keep `Markdown` separate from `Check`; formatting failures should stay quick and obvious.

## Done When

- A pull request cannot merge unless `Check`, `Markdown`, and `MSRV` pass.
- Merge queue, if enabled, runs the same required checks through `merge_group`.
