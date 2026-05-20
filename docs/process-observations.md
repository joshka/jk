# Process Observations

Tracked observations about model and worker behavior during this project. Record only facts that can
be supported by the work log, repo state, or direct transcript evidence.

## Observations

### 2026-05-19

- Slice / task: Bootstrap first tracked process-observations doc

- Worker / model: `019e436d-8c85-7b21-bb66-cb30be4b31af` / `gpt-5.4-mini`

- Scope given: Draft a concise tracked Markdown doc under `docs/`; do not touch `jj` or Git history.

- Observable outcome: Produced the first tracked `docs/process-observations.md` draft with a short
  purpose statement, an observations section, excluded-evidence guidance, and a maintenance note.

- Evidence basis:
  - Thread: `019e436d-8c85-7b21-bb66-cb30be4b31af`
  - Date: `2026-05-19` (local checkout)
  - Transcript: subagent completion message for the doc-draft task
  - Files: `docs/process-observations.md`

- Slice / task: Concurrent-edit handling during doc bootstrap

- Worker / model: main thread orchestration

- Scope given: Avoid reverting or overwriting other work in the checkout.

- Observable outcome: Preserved unrelated modifications already present in `src/app.rs`,
  `src/bookmarks.rs`, `src/command.rs`, `src/graph.rs`, `src/jj.rs`, `src/tui.rs`, and
  `src/view_state.rs`.

- Evidence basis:
  - Thread: `019e4370-f815-7d30-b00f-40ff208958a4`
  - Date: `2026-05-19`
  - Command: `jj --no-pager status`
  - Files: `src/app.rs`, `src/bookmarks.rs`, `src/command.rs`, `src/graph.rs`, `src/jj.rs`,
    `src/tui.rs`, `src/view_state.rs`

- Slice / task: Pre-fix Slice 11 validation pass

- Worker / model: `019e436d-8c39-7592-9ffd-4eed7517b7e5` / `gpt-5.3-codex-spark`

- Scope given: Verify the landed push-preview flow in the current working copy, fix issues if
  needed, and keep the code compiling between edits where practical.

- Observable outcome: Reported the pre-fix Slice 11 patch as building and testing cleanly, with full
  `cargo test` passing at `153` tests before the later review-driven fix pass landed.

- Evidence basis:
  - Thread: `019e436d-8c39-7592-9ffd-4eed7517b7e5`
  - Date: `2026-05-19`
  - Transcript: subagent completion message for the validation task
  - Commands: `cargo check`, `cargo test`, `cargo test -q`, `cargo check --quiet`

- Slice / task: Slice 11 review before acceptance

- Worker / model: `019e436d-8ccf-7c51-a575-bd2a1b49cd78` / `gpt-5.5`

- Scope given: Review the push-preview implementation for logic, contract compliance, and
  maintainability, and review the evidence standard for the new observations doc.

- Observable outcome: Found concrete issues that changed the accepted patch shape: Git-backed remote
  discovery was wrong for jj repos, push results were reduced to one-line status text, and the
  observations doc used non-durable evidence labels.

- Evidence basis:
  - Thread: `019e436d-8ccf-7c51-a575-bd2a1b49cd78`
  - Date: `2026-05-19`
  - Transcript: subagent review findings with file references
  - Files: `src/jj.rs`, `src/app.rs`, `src/tui.rs`, `docs/process-observations.md`

### 2026-05-19 (bounded Slice-11 patch)

- Slice / task: Replace git remote discovery in push flow, make push-result preview visible, and
  make status pushes explicit.

- Worker / model: `019e4370-f815-7d30-b00f-40ff208958a4` / `gpt-5.3-codex-spark`

- Scope given: apply a bounded fix pass for Slice 11 and process-doc durability.

- Observable outcome: switched remotes lookup to `jj git remote list`, updated push preview flow to
  remain visible after confirm, and documented explicit status push target context in push overlay.

- Evidence basis:
  - Thread: `019e4370-f815-7d30-b00f-40ff208958a4`
  - Date: `2026-05-19`
  - Transcript: subagent completion message for the bounded fix pass
  - Commands:
    - `jj --no-pager git remote --help`
    - `jj --no-pager git remote list`
    - `cargo test`
    - `just md-check`
  - Files: `src/jj.rs`, `src/app.rs`, `src/tui.rs`, `AGENTS.md`, `docs/process-observations.md`

### 2026-05-19 (Slice-11 doc refresh)

- Slice / task: Update Slice 11 planning docs for the landed push-preview flow.

- Worker / model: `019e4373-c824-7332-b6a8-bacf0526161f` / `gpt-5.4-mini`

- Scope given: keep the doc edits tight and record the shipped behavior, validation, and residual
  risk.

- Observable outcome: documented the landed push-preview flow in the progress log, added the two
  remaining push-related fragility entries, and left the rest of the planning surface unchanged.

- Evidence basis:
  - Thread: `019e4373-c824-7332-b6a8-bacf0526161f`
  - Date: `2026-05-19`
  - Commands:
    - `sed -n '1,220p' docs/plan/progress.md`
    - `sed -n '1,220p' docs/plan/fragility-register.md`
    - `cargo test push_preview`
    - `cargo test git_push`
    - `just md-check`
  - Files: `docs/plan/progress.md`, `docs/plan/fragility-register.md`,
    `docs/process-observations.md`

### 2026-05-19 (Slice-11 re-review)

- Slice / task: Re-review the updated Slice 11 push-preview implementation and process-observations
  doc after the latest fixes.

- Worker / model: `019e4373-2987-7e30-8b0c-6e20b5829940` / `gpt-5.5`

- Scope given: Focus on `src/app.rs`, `src/jj.rs`, `src/tui.rs`, `AGENTS.md`, and
  `docs/process-observations.md`; check jj-backed remote discovery, push result visibility, explicit
  status-context messaging, and durable evidence references.

- Observable outcome: Reviewed the requested files, checked local `jj git remote list` output,
  verified focused push and remote parser tests, and checked Markdown formatting and linting.

- Evidence basis:
  - Thread: `019e4373-2987-7e30-8b0c-6e20b5829940`
  - Date: `2026-05-19`
  - Commands:
    - `jj --no-pager status`
    - `jj --no-pager diff -- src/app.rs src/jj.rs src/tui.rs AGENTS.md docs/process-observations.md`
    - `jj --no-pager git remote list`
    - `cargo test push -- --nocapture`
    - `cargo test parses_git_remotes -- --nocapture`
    - `just md-check`
  - Files: `src/app.rs`, `src/jj.rs`, `src/tui.rs`, `AGENTS.md`, `docs/process-observations.md`

## Excluded Evidence

This page excludes speculation about cost, quality, intent, or future outcomes. It also excludes
unverified attributions for why a worker chose a path and any general project claims that are not
tied to a concrete command, file state, or transcript.

## Maintenance

Update this file on each turn, as requested by the user, with any new provable observations that
belong here.
