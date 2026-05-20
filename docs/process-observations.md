# Process Observations

Tracked observations about model and worker behavior during this project. Record only facts that can
be supported by the work log, repo state, or direct transcript evidence.

## Observations

### 2026-05-19 (Packet 13 action output overlay)

- Slice / task: Implement Packet 13 scrollable action output overlay.

- Worker / model: `gpt-5.5` (high reasoning). Main thread orchestrated/reviewed.

- Scope given: keep the write set bounded to the action output surface, avoid jj/git commands,
  preserve other workers' edits, make push and rebase preview/result/error output readable in small
  terminals, and update progress and process notes after validation.

- Observable outcome: added a small `ActionOutput` modal-state type, routed push and rebase
  previews/results/errors through the shared scrollable overlay, kept footer keys visible while the
  body scrolls, and added focused app, state, and rendering tests for scroll bounds, close behavior,
  selection preservation, and existing push/rebase completion behavior.

- Evidence basis:
  - Thread: `019e43ee-31d3-78e2-91d9-6d87a434c31f`
  - Date: `2026-05-19`
  - Commands:
    - `printenv CODEX_THREAD_ID`
    - `date +%F`
    - `cargo check`
    - `cargo test action_output`
    - `cargo test push_preview`
    - `cargo test rebase_preview`
    - `cargo test`
    - `rustup run nightly cargo fmt`
    - `just md-check`
  - Files: `src/action_output.rs`, `src/app.rs`, `src/main.rs`, `src/tui.rs`,
    `docs/plan/progress.md`, `docs/process-observations.md`

### 2026-05-20 (Packet 13 confirm completion repair)

- Scope given: fix the rebase confirm path where reveal failure left the overlay in a `pending`
  state and close to a second confirm after command success.

- Fix outcome: `confirm_rebase` now always replaces the pending rebase preview with a completed
  `ActionOutput` after successful `run` + `refresh`, including reveal failure context
  (`reveal failed: ...`) in the completed output text. Enter now closes that state.

- Evidence basis:
  - Thread: `019e43f9-63bd-7b91-8d87-078bece5ce8c`
  - Date: `2026-05-20`
  - Commands:
    - `printenv CODEX_THREAD_ID`
    - `date +%F`
    - `cargo check`
    - `cargo test rebase_`
    - `cargo test action_output`
    - `cargo test push_preview`
    - `rustup run nightly cargo fmt`
  - Files: `src/app.rs`, `docs/process-observations.md`

### 2026-05-19 (Packet 15 planning follow-up)

- Slice / task: Refine Packet 15 contract language and disposable-repo execution discipline

- Worker / model: `019e43d5-1799-78a1-92b7-fb709d7d640c` / `gpt-5.3-codex-spark`

- Scope given: rewrite Packet 15 as a general exact-target abandon flow, require stronger
  confirmation for non-empty changes, forbid ambiguous targets, and require manual proof/write
  testing to use isolated disposable repos under `/tmp` instead of the repo under active
  development.

- Observable outcome: updated `docs/plan/next-implementation-slices.md` to broaden Packet 15 to all
  exact-target abandon contexts, distinguish empty/non-empty confirmation behavior, and require
  `/tmp`-scoped manual proof with `cwd` set to the temporary repo; also added a follow-up
  observation entry explaining the correction.

- Evidence basis:
  - Thread: `019e43d5-1799-78a1-92b7-fb709d7d640c`
  - Date: `2026-05-19`
  - Commands:
    - `printenv CODEX_THREAD_ID`
    - `rg --files docs/plan docs/process-observations.md`
    - `rg -n "Packet 15|disposable|abandon" docs/plan/next-implementation-slices.md`
    - `sed -n '1,240p' docs/plan/next-implementation-slices.md`
    - `sed -n '1,160p' docs/process-observations.md`
    - `just md-check`
  - Files: `docs/plan/next-implementation-slices.md`, `docs/process-observations.md`

### 2026-05-19 (Packet 14 status-bar declutter)

- Slice / task: Implement Packet 14 status-bar declutter.

- Worker / model: `019e4400-56e3-79c0-81fc-d0c4c93f9d07` / `gpt-5.4-mini` (high reasoning). Main
  thread orchestrated/reviewed.

- Scope given: keep the status bar calmer by moving exhaustive binding discovery to generated help,
  keep status focused on current mode, selection/action state, errors, and high-frequency keys,
  avoid remapping shortcuts, and update only the touched Rust and packet-doc surfaces.

- Observable outcome: replaced the long per-view status hint wall with a message-first status line
  and a smaller width-aware hint set, kept the generated help overlay as the exhaustive binding
  source, and added snapshot-style chrome tests for narrow and normal widths.

- Evidence basis:
  - Thread: `019e4400-56e3-79c0-81fc-d0c4c93f9d07`
  - Date: `2026-05-19`
  - Commands:
    - `printenv CODEX_THREAD_ID`
    - `date +%F`
    - `cargo test tui -- --nocapture`
    - `cargo check`
    - `cargo test`
    - `rustup run nightly cargo fmt`
    - `just md-check`
  - Files: `src/app.rs`, `src/tui.rs`, `docs/plan/progress.md`, `docs/process-observations.md`

### 2026-05-19 (future follow-up planning expansion)

- Slice / task: Expand the next implementation-slices plan with downstream follow-up waves.

- Worker / model: `019e43d7-783b-7e40-86bf-6f8805f95a80` / `gpt-5.5` (high reasoning). Main thread
  orchestrated and reviewed this turn.

- Scope given: edit only `docs/plan/next-implementation-slices.md` and
  `docs/process-observations.md`, avoid source code, `AGENTS.md`, `docs/development`, generated
  files, README, the plan index, and jj/git commands, and broaden the plan beyond the current
  near-term implementation packets.

- Observable outcome: added a follow-up backlog after the workflow coverage map that covers abandon,
  operation recovery, rewrite, richer `jj new`, refs/tags/workspaces, file workflows, sync, conflict
  resolution, command-mode policy, tutorial/demo/media, integration contracts, performance, and
  accessibility/terminal compatibility follow-ups. The backlog records likely prerequisites,
  promotion evidence, and the continuing requirement that write-operation proof use isolated `/tmp`
  repositories.

- Evidence basis:
  - Thread: `019e43d7-783b-7e40-86bf-6f8805f95a80`
  - Date: `2026-05-19`
  - Commands:
    - `sed -n '1,260p' docs/plan/next-implementation-slices.md`
    - `sed -n '260,620p' docs/plan/next-implementation-slices.md`
    - `sed -n '1,260p' docs/process-observations.md`
    - `sed -n '260,620p' docs/process-observations.md`
    - `sed -n '1,220p' docs/agent/documentation.md`
    - `sed -n '1,220p' docs/plan/command-inventory.md`
    - `sed -n '1,220p' docs/plan/workflows.md`
    - `sed -n '1,260p' docs/plan/integration-strategy.md`
    - `sed -n '1,180p' docs/plan/fragility-register.md`
    - `printenv CODEX_THREAD_ID`
  - Files: `docs/plan/next-implementation-slices.md`, `docs/process-observations.md`

### 2026-05-19

- Slice / task: Slice 12 rebase preview flow implementation

- Worker / model: `019e4378-4fca-7202-bedc-7ba0df298487` / `gpt-5.4-mini`

- Scope given: implement the first rebase preview flow only, keep source and destination roles
  explicit, require preview and confirmation, refresh the graph after success, and avoid touching
  jj/git history.

- Observable outcome: threaded the graph action menu into a dedicated rebase preview modal with
  explicit source/destination role extraction, synthetic command/effect preview text, confirm/cancel
  behavior, and post-run refresh messaging that keeps `jj undo` visible.

- Evidence basis:
  - Thread: `019e4378-4fca-7202-bedc-7ba0df298487`
  - Date: `2026-05-19`
  - Commands:
    - `cargo check`
    - `cargo test rebase -- --nocapture`
    - `cargo test selected_sources_and_destination_prompt_with_explicit_roles -- --nocapture`
    - `cargo test`
  - Files: `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`

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

### 2026-05-19 (Slice-12 implementation review)

- Slice / task: Review the current Slice 12 rebase-preview implementation against the
  implementation-slice acceptance criteria.

- Worker / model: `019e437e-13a5-7ba3-9b9a-f029dc3bf178` / `gpt-5.5`

- Scope given: Focus on `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`, and tests
  added in `src/app.rs` and `src/jj.rs`; check explicit roles, visual-order inference,
  preview/confirmation, success selection behavior, failure readability, preview honesty, and slice
  scope.

- Observable outcome: Reviewed the requested files, checked the Slice 12 text, verified the current
  unit test suite, and consulted `jj rebase --help` for the exact `-r` graph semantics.

- Evidence basis:
  - Thread: `019e437e-13a5-7ba3-9b9a-f029dc3bf178`
  - Date: `2026-05-19`
  - Commands:
    - `jj --no-pager status`
    - `rg -n "Slice 12|stack|rebase|source|destination|acceptance"       docs/plan/implementation-slices.md`
    - `rg -n "action_menu|Move|rebase|source|destination|preview|confirm|stack"       src/action_menu.rs src/app.rs src/jj.rs src/tui.rs`
    - `cargo test`
    - `jj --no-pager rebase --help`
  - Files: `docs/plan/implementation-slices.md`, `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`,
    `src/tui.rs`, `docs/process-observations.md`

### 2026-05-19 (Slice-12 implementation and repair)

- Slice / task: Implement the first rebase preview flow while keeping the shared tree buildable.

- Worker / model: `019e4378-4fca-7202-bedc-7ba0df298487` / `gpt-5.4-mini`,
  `019e437a-4153-72d2-88cf-57a2d13d1bdb` / `gpt-5.3-codex-spark`, and
  `019e437b-9ac8-72e2-b3f7-7b5dc414bdc7` / `gpt-5.3-codex-spark`

- Scope given: implement Slice 12 code in the graph action-menu path, then restore compilation
  immediately if the shared working copy stopped building.

- Observable outcome: the mini worker eventually reported a complete Slice 12 implementation; in
  parallel, a Spark worker landed the shared `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, and
  `src/tui.rs` rebase-preview patch in the working copy, and a second Spark worker narrowed a broken
  intermediate state to concrete compile blockers after a syntax repair.

- Evidence basis:
  - Threads:
    - `019e4378-4fca-7202-bedc-7ba0df298487`
    - `019e437a-4153-72d2-88cf-57a2d13d1bdb`
    - `019e437b-9ac8-72e2-b3f7-7b5dc414bdc7`
  - Date: `2026-05-19`
  - Transcript: subagent completion messages for the implementation and compile-repair tasks
  - Commands:
    - `cargo check`
    - `cargo test rebase -- --nocapture`
    - `cargo test`
    - `jj --no-pager status`
  - Files: `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`

### 2026-05-19 (Slice-12 acceptance fix)

- Slice / task: Close the 5.5 review gap on post-rebase selection behavior.

- Worker / model: main thread orchestration and local patching after `gpt-5.5` review

- Scope given: ensure successful rebase refreshes and preserves or moves selection to the affected
  stack before accepting Slice 12.

- Observable outcome: updated `confirm_rebase()` to reveal a rebased source change after refresh,
  using the same recent-mode fallback pattern already used by the `jj new trunk` flow, then reran
  focused rebase tests, full `cargo test`, and a disposable-repo `jj rebase` plus `jj undo` proof.

- Evidence basis:
  - Thread: `019e437e-13a5-7ba3-9b9a-f029dc3bf178`
  - Date: `2026-05-19`
  - Commands:
    - `cargo test rebase -- --nocapture`
    - `cargo test`
    - `jj --no-pager rebase --help`
    - disposable-repo `jj --no-pager rebase -r <source> -o <dest>`
    - disposable-repo `jj --no-pager undo`
  - Files: `src/app.rs`, `docs/plan/progress.md`, `docs/plan/fragility-register.md`,
    `docs/process-observations.md`

### 2026-05-19 (Slice-12 confirmatory re-review)

- Slice / task: Confirm that the post-rebase selection fix closes the only substantive Slice 12
  review gap.

- Worker / model: `019e437e-13a5-7ba3-9b9a-f029dc3bf178` / `gpt-5.5`

- Scope given: re-review the updated `src/app.rs` success path only and confirm whether the earlier
  acceptance finding is resolved.

- Observable outcome: confirmed that successful rebase now captures a rebased source id before
  execution and reveals that change after refresh with a `LogViewMode::Recent` fallback, leaving no
  substantive acceptance gaps.

- Evidence basis:
  - Thread: `019e437e-13a5-7ba3-9b9a-f029dc3bf178`
  - Date: `2026-05-19`
  - Transcript: subagent follow-up completion message after the selection fix
  - Commands:
    - `cargo test rebase -- --nocapture`
  - Files: `src/app.rs`, `docs/process-observations.md`

### 2026-05-19 (session review audit)

- Slice / task: Review landed slices, model usage evidence, development-rule adherence, and product
  state.

- Worker / model: main thread review pass

- Scope given: read the copied `docs/development` rule files up front, inspect session history and
  landed code, compare subagent model usage with rework, and assess current `jk` product
  completeness.

- Observable outcome: found and corrected the Slice 12 model attribution for
  `019e4378-4fca-7202-bedc-7ba0df298487`; counted explicit parent-thread subagent spawn requests by
  model/role; inspected the current jj stack and main workflow files; reran the full Rust unit test
  suite.

- Evidence basis:
  - Thread: `019e42d3-ba3c-78a1-9623-d684a45bcc39`
  - Date: `2026-05-19`
  - Commands:
    - `rg --files docs/development | sort`
    - `jq -r 'select(.type=="response_item" and .payload.type=="function_call" and .payload.name=="spawn_agent") | (.payload.arguments | fromjson? // empty) | [.model // "inherited", .agent_type // "default"] | @tsv' ~/.codex/sessions/2026/05/19/rollout-2026-05-19T17-40-32-019e42d3-ba3c-78a1-9623-d684a45bcc39.jsonl`
    - `jj --no-pager log -r 'main..@'`
    - `cargo test`
  - Files: `docs/development/`, `docs/process-observations.md`, `docs/plan/progress.md`,
    `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`

### 2026-05-19 (next implementation planning)

- Slice / task: Write the next implementation-slice plan after completed Slices 0-12.

- Worker / model: main thread orchestration plus `gpt-5.5` subagents:
  `019e43b8-34ab-7263-9c25-066c5d46b2c0` / explorer for product and workflow packet planning,
  `019e43b8-5352-79f0-ba9b-6d9185d25fb8` / explorer for documentation, demo, and VHS planning, and
  `019e43bb-b4ec-7180-99b8-4f75f4a3fbf1` / worker for the tracked docs update.

- Scope given: create or update `docs/plan/next-implementation-slices.md`, index it from
  `docs/plan/README.md`, record this planning turn here, keep the write set narrow, and avoid jj/git
  commands.

- Observable outcome: two read-only planning agents proposed product/workflow and docs/demo packets;
  the docs worker then drafted delegation-ready Packets 13-32 with bounded ownership, acceptance,
  validation, docs/fragility requirements, model/review routing, Ratatui-grounded demo guidance, and
  a day-to-day workflow coverage map.

- Evidence basis:
  - Thread: `019e43bb-b4ec-7180-99b8-4f75f4a3fbf1`
  - Date: `2026-05-19`
  - Commands:
    - `sed -n '1,240p' docs/plan/README.md`
    - `sed -n '1,260p' docs/plan/implementation-slices.md`
    - `sed -n '1,260p' docs/plan/progress.md`
    - `sed -n '1,260p' docs/plan/command-inventory.md`
    - `sed -n '1,260p' docs/plan/workflows/rewrite.md`
    - `sed -n '1,260p' docs/plan/workflows/recover.md`
    - `sed -n '1,260p' docs/plan/workflows/sync.md`
    - `sed -n '1,260p' docs/plan/workflows/refs-and-workspaces.md`
    - `sed -n '1,320p' docs/plan/fragility-register.md`
    - `sed -n '1,260p' docs/agent/testing.md`
    - `sed -n '1,260p' docs/agent/documentation.md`
    - `sed -n '1,260p' docs/process-observations.md`
    - `printenv CODEX_THREAD_ID`
    - `panache format docs/plan/next-implementation-slices.md docs/plan/README.md docs/process-observations.md`
    - `just md-check`
  - Web sources:
    - `https://ratatui.rs/recipes/apps/release-your-app/`
    - `https://ratatui.rs/recipes/apps/submitting-to-the-showcase/`
  - Files: `docs/plan/next-implementation-slices.md`, `docs/plan/README.md`,
    `docs/process-observations.md`

## Analysis From This Session

This section records process guidance derived from the observations above. It is not active
`AGENTS.md` policy yet. Treat it as candidate guidance for the next subagent-heavy implementation
run, with the supporting evidence kept nearby so the rule can be accepted, revised, or rejected
later.

### Evidence Summary

- The parent session explicitly requested many subagents. Counting explicit `spawn_agent` requests
  by model and role in the parent session produced:
  - `gpt-5.4-mini` / worker: 27 requests.
  - `gpt-5.3-codex-spark` / worker: 19 requests.
  - `gpt-5.5` / default: 9 requests.
  - `gpt-5.5` / explorer: 7 requests.
  - `gpt-5.5` / worker: 9 requests.
  - inherited-model requests: 11 total across default, explorer, and worker roles.
- The count above is only a spawn-request count. It is not a cost total, runtime total, quality
  score, or proof that every spawned worker completed useful work.
- Slice 11 shows a concrete pattern where a fast validation pass reported the pre-fix push-preview
  flow as building and testing cleanly, but later `gpt-5.5` review found acceptance-shaping issues:
  Git-backed remote discovery was wrong for a jj repo, push results were too compressed, and process
  evidence labels were not durable.
- Slice 11 also shows a positive bounded-fix pattern: a Spark worker, after review narrowed the
  target, switched remote discovery to `jj git remote list`, kept push results visible, improved
  status-context messaging, and ran the relevant checks.
- Slice 12 shows the risk of broad implementation tasks against overlapping write sets. A mini
  worker was assigned a full rebase-preview implementation, a Spark worker landed overlapping code
  in the shared files, and a second Spark worker then had to narrow compile blockers in a broken
  intermediate state.
- Slice 12 also shows the value of a deeper acceptance review. The `gpt-5.5` review found that
  successful rebase did not yet preserve or move selection to the affected stack; the main thread
  then patched `confirm_rebase()` and a follow-up `gpt-5.5` review confirmed that gap was closed.
- The final implementation passed the broad local proof target available at the time: `cargo check`,
  focused rebase tests, full `cargo test`, manual disposable-repo rebase plus undo, formatter check,
  and Markdown checks. The latest review pass reran full `cargo test` with 162 passing tests.

### Candidate Project Guidance

These are candidate rules for future subagent-heavy work in this repository. They are intentionally
recorded here instead of `AGENTS.md` until the maintainer decides which should become durable
instructions.

- Give implementation subagents bounded, well-specified work. A worker task should name the owned
  files or modules, explicit non-goals, expected behavior, tests to add or run, and handoff format.
  If that contract cannot be written clearly, run exploration or design review before
  implementation.
- Avoid overlapping implementation write sets by default. Parallel agents can inspect or review the
  same area, but simultaneous code-writing workers should own disjoint files or responsibilities.
  When the write set is shared, prefer one implementor and one or more read-only reviewers.
- Route models by risk and task shape. Use faster/smaller workers for docs, narrow local patches,
  and compile repair. Use stronger workers for cross-module implementation or design-heavy work. Use
  `gpt-5.5` review for acceptance gates on risky mutation flows and cross-module behavior.
- Treat Spark as a quick-fix tool unless the task is exceptionally narrow. The observed successful
  Spark work had explicit concrete fixes; the observed problematic Spark work was broader and landed
  code that needed compile repair.
- Do not treat "tests pass" from a fast worker as acceptance for a risky flow. Require review
  against the slice acceptance criteria, user-visible behavior, module ownership, test gaps, and
  documented residual risk.
- Keep the tree compiling during multi-agent implementation. Workers should run `cargo check` early
  after nontrivial Rust edits. If the tree stops compiling and the fix is not immediate, hand off a
  narrow compile-repair task with the current compiler error.
- Define done for each slice as code plus docs plus tests plus recorded residual risk. A handoff
  should state what passed, what was not run, and what remains risky.
- Prefer view/app-level tests for TUI behavior when the contract includes both content and
  presentation. Command construction tests are necessary for `jj` flows, but they are not enough for
  modal behavior, refresh behavior, or user-visible result presentation.
- Watch for module-size pressure during slice work. When `app.rs` or another owner starts collecting
  several different workflow ideas, consider a small concept-owned extraction before adding the next
  flow.
- Record model/thread attribution from transcript or spawn records. If the model is unknown, record
  it as unknown instead of inferring from memory.

### Next-Run Preflight

Use this before launching a subagent-heavy slice:

- Restate the slice goal in one sentence and list the owned files or modules.
- Decide which work is read-only, which is implementation, and which is review.
- Mark the non-goals explicitly, especially neighboring features and unrelated refactors.
- Identify the acceptance evidence up front: tests, manual checks, and review target.
- Check for existing edits in the owned files and avoid overlapping write sets.
- Decide whether the first step is exploration, implementation, or compile repair.

### Next Slice-Planning Pass

For the next task-list rewrite, the useful output is not only a new ordered roadmap. Ask the planner
to produce implementation packets that are ready to delegate:

- Read current product, progress, fragility, and process-observation docs before rewriting the
  slices.
- Start by identifying enabling refactors or shared test harness work that would reduce risk in the
  next several slices.
- For each proposed slice, name the owning concept, expected write set, explicit non-goals,
  acceptance criteria, validation plan, and residual-risk doc updates.
- Mark which slices are safe for small implementation workers and which require stronger
  implementation or design review.
- Mark where a read-only explorer should run before implementation because ownership, upstream
  behavior, or output shape is still unclear.
- Include a review prompt for each high-risk slice so the acceptance gate is planned before coding
  starts.

### Subagent Task Template

Use a bounded prompt that includes all of these fields:

```text
Goal: <one-sentence slice goal>
Owned files/modules: <specific paths or concepts>
Behavior to change: <what should be true when done>
Non-goals: <what to leave alone>
Constraints: <policy, scope, or compatibility limits>
Tests/checks: <commands to run or add>
Handoff evidence: <what files changed, what passed, what remains risky>
```

Good prompts keep the work local. They name the exact ownership boundary, say what success looks
like, and make it obvious when the task is too broad for one worker.

### Reviewer Checklist

Use deeper review agents to confirm the slice against the actual acceptance criteria:

- Does the change stay inside the intended files or module boundary?
- Does the behavior match the requested flow, including edge cases and failure paths?
- Is the user-visible presentation honest about preview, confirmation, and result state?
- Are the tests at the right level for the behavior that changed?
- Did the author report what was run, what passed, what was skipped, and why?
- Is any residual risk written down plainly enough for the next pass?

### Agent Routing Note

Choose the worker shape from the task shape:

- Exploration: use when the slice is still fuzzy, the ownership boundary is unclear, or the next
  implementation step depends on understanding existing behavior.
- Implementation: use when the task can be bounded to files, behavior, and checks, and the tree can
  stay compiling while the worker edits.
- Compile repair: use when the tree is already broken and the immediate goal is to restore the
  current compiler or test failure before broader work resumes.

## Sanitized Guidance Candidate

The following is a project-neutral version that can be reused in another repository or shared in a
public discussion. It deliberately avoids this project's file names, thread ids, and model-count
details.

### Multi-Agent Implementation Guidance

Use subagents for parallel work only when their tasks are small enough to specify precisely. A good
implementation task names:

- the owned files or subsystem;
- the behavior to implement;
- explicit non-goals;
- expected tests or checks;
- the required handoff evidence.

Avoid assigning multiple code-writing agents to the same files at the same time. If a change needs
parallelism, split by ownership boundary. If that is not possible, use one implementor and parallel
read-only reviewers.

Use lighter models for bounded execution: documentation updates, local refactors, test fixes, and
compile repair. Use stronger models for ambiguous design, cross-module implementation, and final
review. Fast models can save time, but only when the task shape prevents them from inventing missing
architecture.

Treat fast compile or test success as evidence, not acceptance. For risky or user-facing behavior,
review against the actual acceptance criteria: behavior, edge cases, user-visible presentation,
module ownership, documentation truth, and residual risk.

Keep the repository buildable during multi-agent work. Run a cheap build check early after
nontrivial edits. If the shared tree stops compiling, stop broad work and issue a narrow
compile-repair task with the current error.

Define done before implementation starts. For feature work, done usually means:

- code changed in the owning module;
- tests prove the behavior at the right level;
- user-facing or maintainer-facing docs are updated when behavior or assumptions changed;
- validation commands were run and reported;
- known residual risks are written down.

Keep process notes factual. Record what task was assigned, which worker handled it, what changed,
what checks ran, what review found, and what rework followed. Separate those facts from later
opinions about cost effectiveness or model quality.

For planning work, ask for implementation packets, not just a roadmap. Each packet should name the
owning subsystem, write boundary, non-goals, acceptance criteria, validation, and review needs. This
makes the next delegation step mechanical and exposes slices that are still too vague to implement.

## Excluded Evidence

This page excludes speculation about cost, quality, intent, or future outcomes. It also excludes
unverified attributions for why a worker chose a path and any general project claims that are not
tied to a concrete command, file state, or transcript.

## Maintenance

Update this file on each turn, as requested by the user, with any new provable observations that
belong here.
