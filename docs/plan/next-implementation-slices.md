# Next Implementation Slices

This document extends the completed Slice 0-12 plan with the next implementation packets for `jk`.
It is optimized for delegation: each packet names its owning concept, likely files, non-goals,
acceptance criteria, validation, documentation updates, model routing, and review prompt.

Use this after checking [`progress.md`](progress.md),
[`fragility-register.md`](fragility-register.md), and the workflow specs for the area being changed.
The packet order favors near-term daily value and enabling infrastructure before broader command
coverage.

## How To Use This

1. Pick the earliest packet whose dependencies are satisfied.
1. Copy the packet fields into a bounded task prompt using the template below.
1. Keep implementation write sets disjoint when running parallel agents.
1. Run focused tests while editing and the packet's validation before handoff.
1. Update progress, fragility, and user-facing docs when the packet changes behavior or assumptions.

## Execution Contract

These rules apply to every packet below.

- Keep each packet bounded to one product behavior and one owning concept. If the implementation
  discovers a necessary second behavior, record it as a follow-on instead of expanding the packet.
- Do not assign overlapping write sets to concurrent implementation workers. Parallel work is safe
  for read-only exploration, documentation review, and code review, but shared Rust files should
  have one implementor at a time.
- Route work by risk. Use smaller workers for narrow documentation, command-construction, or
  compile-repair tasks. Use stronger workers for cross-module behavior, mutation flows, parser
  contracts, and final acceptance review.
- Keep `jj` command exactness visible. Command builders should preserve user-provided arguments,
  prefer exact change/bookmark/operation ids over rendered labels, and make every mutation target
  explicit in preview/result text.
- Preserve compilation during Rust work. After nontrivial edits, run `cargo check` or focused tests
  before stacking more changes.
- Add tests at the behavior boundary: view-level tests for modal/output/navigation behavior,
  command-construction tests for `jj` invocations, parser tests for rendered output contracts, and
  disposable `jj` repo/manual proofs for mutation flows and screen-flow checks that need actual
  writes and cannot be proved with unit tests. Those proofs must use an isolated repo under `/tmp`
  and run each write operation with that repo as the process working directory (`cwd`) so the
  checkout under `/Users/joshka/local/jk` is not mutated.
- Update documentation truthfully. User-facing docs should describe shipped behavior only; planning
  docs can describe future behavior when clearly marked as planned.
- Update [`fragility-register.md`](fragility-register.md) whenever a packet parses rendered `jj`
  output, infers semantic state from presentation, duplicates `jj` command behavior, or relies on
  CLI output wording.
- Update [`progress.md`](progress.md) after implementation packets land with files changed,
  validation run, remaining risk, and the next recommended packet.
- Preserve output/result visibility. Mutations should not collapse rich command output into a
  transient one-line status when the user needs to inspect success, failure, or recovery context.
- Do not store generated GIFs, screenshots, demo repositories, or rendered tutorial media in the
  repo. Put generated assets under ignored `target/vhs` or publish/host them externally.

## Subagent Task Template

```text
Goal: <one-sentence packet goal>
Ownership/write set: <specific files/modules or docs>
Non-goals: <nearby behavior to leave alone>
Acceptance criteria:
- <observable user-visible or maintainer-visible result>
- <edge case or failure behavior>
Validation:
- <focused tests/checks>
- <manual disposable jj repo proof, if needed (run in `/tmp` repo with `cwd` set there)>
Docs/fragility:
- <progress, fragility-register, README, tutorial, or screen docs updates>
Model/agent choice: <implementation, exploration, review, and model strength>
Review prompt:
Review <files/modules> against packet <N> acceptance criteria, command exactness,
view behavior, tests, docs, and residual risk. Report findings first with file/line
references and state whether the packet is acceptable.
```

## Ordering And Parallelism

Packets 13 and 14 are enabling UI infrastructure and can run in sequence with minimal product
ambiguity. Packet 15 should wait for Packet 13 so abandon results stay inspectable. Packets 16 and
17 both use operation-log context; implement 16 first so undo/redo can reuse detail/output
presentation patterns. Packet 18 can run after 13 and does not require operation-log work.

Documentation/demo packets 20 and 21 can be drafted in parallel with implementation work if they do
not document unshipped behavior as complete. Packet 21 should wait to capture final media until the
status bar and action output surface are stable. Packet 22 and later mutation packets should use the
post-Packet-13 output surface and should receive deeper review before acceptance.

## Packet 13: Scrollable Action Output Overlay

- Goal: make long preview/result/error output readable after direct and guided actions.
- Owner concept: shared modal/action output presentation in `app.rs`, `tui.rs`, and any existing
  action-flow types.
- Expected write set: `src/app.rs`, `src/tui.rs`, `src/action_menu.rs` if shared state needs a
  clearer owner, focused tests in touched modules, `docs/plan/progress.md`, and
  `docs/plan/fragility-register.md` only if output assumptions change.
- Non-goals: no new mutation commands, no command-mode surface, no redesign of all modals, and no
  persistent operation-output history beyond the active result.
- Acceptance criteria: action previews, successful results, and failures can show multi-line output;
  output scrolls in small terminals; close/confirm/cancel keys remain clear; prior view selection
  survives opening and closing the overlay; existing push and rebase result output use the shared
  surface.
- Validation: focused view/app tests for scroll boundaries, close behavior, and preserving prior
  selection; focused tests for existing push/rebase modal text; `cargo check`; `cargo test` when
  practical.
- Docs/fragility updates: record any new dependency on `jj` output wording in
  `fragility-register.md`; update `progress.md` after landing.
- Suggested agent/model: stronger implementation worker or one implementation worker plus stronger
  review, because this crosses modal state, rendering, and existing mutation flows.
- Review prompt: review the output overlay for small-terminal behavior, accessibility of command
  output, key handling, selection preservation, and whether existing push/rebase flows still expose
  their recovery path.

## Packet 14: Declutter Status Bar

- Goal: make the status bar calmer by moving exhaustive bindings into generated help and keeping
  status focused on current mode, selection, action state, and errors.
- Owner concept: shared chrome and command/help projection.
- Expected write set: `src/tui.rs`, `src/command.rs`, `src/view_state.rs` or view files only where
  status labels are owned, focused rendering tests, `docs/plan/progress.md`.
- Non-goals: no shortcut remapping, no new help categories unless needed to preserve
  discoverability, and no visual redesign outside the status/header/help surfaces.
- Acceptance criteria: status text remains useful at narrow widths; high-frequency transient state
  is visible; less common keys are discoverable through `?`; generated help remains the source of
  truth for full key coverage; mutation previews and result overlays are not duplicated into noisy
  status text.
- Validation: snapshot-style rendering tests for narrow and normal widths; help projection tests if
  wording changes; `cargo check`; focused tests for status/header rendering; `cargo test` when
  practical.
- Docs/fragility updates: update `progress.md`; update user-facing docs only if key discovery or
  status semantics are documented there in Packet 20.
- Suggested agent/model: narrow implementation worker is acceptable, with review focused on terminal
  ergonomics.
- Review prompt: review the status/header/help changes for discoverability regressions, narrow
  terminal readability, and whether output that belongs in overlays stayed out of the status bar.

## Packet 15: General Abandon From Exact Change Targets

- Goal: add a safe guided `jj abandon` flow from contexts where the selected change target is exact.
- Owner concept: action flow and `jj abandon` command construction for exact change targets.
- Expected write set: `src/graph.rs`, `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`,
  focused tests, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no multi-select abandon, no operation restore/revert.
- Acceptance criteria: the action is available only when an exact change target is available and
  disabled for ambiguous targets; empty changes use a standard low-friction confirmation path;
  non-empty changes use a stronger confirmation step and explicitly surface the exact change, title,
  and effect in preview; success refreshes the active view and keeps `jj undo` visible; failures
  keep full output readable.
- Validation: command-construction tests for `jj abandon <change>`; separate empty-change and
  non-empty confirmation-path tests; view-level preview/confirm/cancel/result tests; disposable
  `/tmp` `jj` repo proof for abandoning an empty change, a non-empty change, and undoing it with the
  operation executed from that repo; `cargo check`; focused tests; `just check` before handoff when
  practical.
- Docs/fragility updates: record the exact-target selection and non-empty confirmation contracts and
  any output parsing in `fragility-register.md`; update `progress.md`.
- Suggested agent/model: stronger implementation worker plus stronger review because this is the
  first destructive graph action.
- Review prompt: review exact target selection, empty-vs-non-empty confirmation behavior, preview/
  result wording, undo visibility, `/tmp` disposable-repo proof, and whether non-empty or ambiguous
  changes are blocked.

## Packet 16: Operation Show/Diff Detail

- Goal: let users drill from operation log rows into `jj operation show` and `jj operation diff`
  detail/output.
- Owner concept: operation-log screen and shared document/detail view behavior.
- Expected write set: `src/operation_log.rs`, `src/view_state.rs`, `src/app.rs`, `src/jj.rs`,
  `src/tui.rs`, focused tests, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no undo/redo execution, no operation restore/revert, no attempt to parse full
  transaction semantics, and no operation history editing.
- Acceptance criteria: operation detail opens from the selected operation id; show and diff detail
  preserve rendered `jj` output and styles; output scroll/search/copy behavior matches existing
  document views where practical; missing operation ids degrade to disabled actions; back/refresh
  preserve useful operation context.
- Validation: command-construction tests for operation show/diff; operation-row tests for missing
  ids; view-level navigation and scroll tests; `cargo check`; focused operation-log tests; full
  `cargo test` when practical.
- Docs/fragility updates: record any additional operation output assumptions in
  `fragility-register.md`; update `progress.md`.
- Suggested agent/model: stronger implementation worker, because this touches cross-view navigation
  and operation-id contracts.
- Review prompt: review operation id usage, rendered-output preservation, disabled-action behavior,
  navigation/back semantics, and whether detail views avoid over-parsing operation output.

## Packet 17: Undo/Redo From Operation Log

- Goal: make `jj undo` and `jj redo` accessible from recovery context with inspectable output and a
  clear refresh path.
- Owner concept: operation-log guided recovery actions.
- Expected write set: `src/operation_log.rs`, `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`,
  `src/tui.rs`, focused tests, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no operation restore/revert, no arbitrary operation selection for undo semantics unless
  `jj` supports the exact command shape, and no history rewriting beyond `undo`/`redo`.
- Acceptance criteria: undo and redo actions are explicit about what `jj` will do; preview or
  confirmation matches risk; success refreshes the current view and operation log; result output is
  scrollable; `redo` is disabled or reports clearly when unavailable; ambiguous target assumptions
  are not hidden.
- Validation: command-construction tests; view-level preview/confirm/result tests; disposable `/tmp`
  `jj` repo proof for undo and redo after a simple mutation (run from that repo's `cwd`);
  `cargo   check`; focused operation-log tests; `just check` when practical.
- Docs/fragility updates: record whether the flow relies on global last-operation semantics or exact
  operation ids; update `progress.md`.
- Suggested agent/model: stronger implementation worker plus stronger review because recovery flows
  must be exact.
- Review prompt: review undo/redo command semantics against `jj` help, output visibility, refresh
  behavior, disabled states, and manual proof from `/tmp` repos.

## Packet 18: `jj new` From Selected Parents

- Goal: support creating a new working-copy change from the selected log change or selected multiple
  parent changes.
- Owner concept: graph selection and `jj new` command planning.
- Expected write set: `src/graph.rs`, `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`,
  focused tests, `docs/plan/progress.md`, and `docs/plan/fragility-register.md` if target inference
  changes.
- Non-goals: no bookmark creation, no description prompt, no automatic rebase/squash, and no complex
  revset editor.
- Acceptance criteria: single selected parent runs an exact `jj new <change>` shape; multi-select
  parents run an exact multi-parent command shape supported by `jj`; preview lists all exact parent
  ids; ambiguous or non-selectable rows are rejected; success reveals the new working copy with a
  recent-mode fallback when needed; undo hint is visible.
- Validation: command-construction tests for one and multiple parents; graph multi-select tests for
  selected parent ordering and disabled states; view-level result refresh tests; disposable `/tmp`
  `jj` repo proof for single-parent and merge-parent new changes with `cwd` set to the proof repo;
  `cargo check`; `cargo test`.
- Docs/fragility updates: update `progress.md`; record any reliance on graph selection ordering in
  `fragility-register.md`.
- Suggested agent/model: stronger implementation worker, because graph selection and command
  semantics must stay aligned.
- Review prompt: review exact parent command construction, multi-select role wording, refresh and
  selection behavior, disabled states, and `/tmp` disposable-repo proof.

## Packet 19: Push Flow Simplification

- Goal: make simple and obvious `jj git push` flows faster while keeping preview safety for
  non-obvious targets.
- Owner concept: sync guided action from status, graph, and bookmark context.
- Expected write set: `src/app.rs`, `src/jj.rs`, `src/bookmarks.rs`, `src/tui.rs`,
  `src/action_menu.rs` if action routing changes, focused tests, `docs/plan/progress.md`, and
  `docs/plan/fragility-register.md`.
- Non-goals: no host-specific dashboard, no credential handling, no force-push shortcut, and no
  broad remote-management UI.
- Acceptance criteria: status-context push uses `jj` default resolution only when the preview says
  so explicitly; bookmark or selected-change push targets are exact; obvious single-target cases can
  reduce prompting without hiding the command; failure and success output remain scrollable; fetch
  and push remain visually distinct.
- Validation: command-construction tests for default, bookmark, revision, and remote-target shapes;
  view-level preview/result tests; disposable `/tmp` `jj` repo or remote-less proof for
  disabled/error paths (run from `/tmp` repo when applicable); `cargo check`; focused push tests;
  `cargo test` when practical.
- Docs/fragility updates: update existing push-targeting entries in `fragility-register.md` if
  target inference changes; update `progress.md`.
- Suggested agent/model: stronger implementation worker plus review, because push behavior has prior
  acceptance history and remote defaults are subtle.
- Review prompt: review push target explicitness, default-resolution honesty, result visibility,
  command construction, and regression risk from the existing push-preview flow.

## Packet 20: README/User Docs Refresh

- Goal: make the public README and user-facing docs match the current app without overstating future
  command coverage.
- Owner concept: user-facing documentation and release-readiness framing.
- Expected write set: `README.md`, `docs/plan/progress.md`, and possibly a new small user guide
  under `docs/` if the README would become too long.
- Non-goals: no generated media checked in, no code changes, no planned behavior described as
  shipped, and no broad rewrite of planning docs.
- Acceptance criteria: README includes a concise app description, installation/run instructions,
  current hotkeys or a pointer to generated help behavior, current screenshots/media placeholder
  policy, safety notes for mutation flows, and links into relevant planning docs for contributors;
  docs distinguish shipped direct/guided flows from planned packets.
- Validation: `just md-check`; manual read-through against current command inventory and progress;
  no Rust validation required unless examples invoke code-generated text.
- Docs/fragility updates: update `progress.md`; no fragility entry unless docs add claims about
  parser or command contracts.
- Suggested agent/model: narrow documentation worker plus maintainer-style review for truthfulness.
- Review prompt: review README claims against current shipped behavior, command coverage,
  installation/run accuracy, and Markdown style.

## Packet 21: VHS Specs Without Committed GIFs

- Goal: add tracked capture specifications and deterministic demo setup while keeping generated
  media out of the repository.
- Owner concept: demo/tutorial capture workflow.
- Expected write set: a tracked `docs/demo/` or `demo/` spec directory, optional `justfile` demo
  recipes, `.gitignore` only if a new generated path outside ignored `target/` is required,
  `docs/plan/progress.md`, and README links if Packet 20 has landed.
- Non-goals: no committed GIFs, no committed generated demo repositories, no brittle local terminal
  screenshots, and no marketing page.
- Acceptance criteria: tracked VHS `.tape` or equivalent specs can generate README/tutorial media
  into ignored `target/vhs`; deterministic demo repo setup writes under `target/demo-repos`; capture
  scripts hide setup noise, use a dark theme, stay at or below 1200 px width, include enough dwell
  time, and focus on one strong workflow at a time; docs say generated media should be hosted or
  attached externally.
- Validation: run the setup script or `just demo-*` gate if added; run VHS locally only if
  available; otherwise state that capture execution was skipped because the tool is missing;
  `just md-check`.
- Docs/fragility updates: update `progress.md`; no fragility entry unless demo setup depends on
  unstable `jj` output in assertions.
- Suggested agent/model: narrow documentation/tooling worker; use review for reproducibility and
  repository hygiene.
- Review prompt: review demo specs against the Ratatui release and showcase guidance, generated
  asset paths, deterministic setup, and whether media remains out of normal repository history.

Ratatui sources for this packet:

- <https://ratatui.rs/recipes/apps/release-your-app/> recommends README basics, hotkeys, media, VHS
  capture, dark terminal themes, width around 1200 px or less, `Hide`/`Show` for setup noise, dwell
  time, and avoiding committed screenshots/GIFs.
- <https://ratatui.rs/recipes/apps/submitting-to-the-showcase/> emphasizes legibility when resized
  or phone-sized, contrast, visual hierarchy, signal over coverage, calm motion, and media review
  before submission.

## Packet 22: Squash Preview Flow

- Goal: add a graph-driven `jj squash` preview flow for moving changes into an explicit target.
- Owner concept: graph action roles and rewrite command planning.
- Expected write set: `src/action_menu.rs`, `src/graph.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`,
  focused tests, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no interactive patch selection, no split flow, no absorb behavior, and no automatic
  target guessing without a visible role prompt.
- Acceptance criteria: source and destination roles are explicit; preview shows exact command and
  affected changes; confirmation is required; success refreshes and reveals the affected target or
  source context; failure keeps full output visible; undo path remains visible.
- Validation: command-construction tests for supported `jj squash` shapes; action-role tests;
  view-level preview/confirm/cancel/result tests; disposable `/tmp` `jj` repo proof with undo (`cwd`
  set to the proof repo); `cargo check`; focused tests; `just check` when practical.
- Docs/fragility updates: add or update rewrite-flow fragility entries for squash preview output and
  role inference; update `progress.md`.
- Suggested agent/model: stronger implementation worker plus stronger review.
- Review prompt: review role selection, command exactness, preview honesty, result visibility,
  refresh/selection behavior, tests, and `/tmp`-scoped manual proof.

## Packet 23: Describe And Commit Flows

- Goal: support common metadata and finalization flows for the current or selected change.
- Owner concept: guided text input and working-copy mutation actions.
- Expected write set: `src/app.rs`, `src/jj.rs`, `src/tui.rs`, `src/command.rs`, graph/status files
  only for action entry points, focused tests, `docs/plan/progress.md`, and
  `docs/plan/fragility-register.md` if command output assumptions are added.
- Non-goals: no full editor integration, no template editor, no amend/squash coupling, and no broad
  command mode.
- Acceptance criteria: describe prompts for a new description and runs exact `jj describe` target
  command; commit finalizes the current working-copy change with explicit description behavior and
  advances according to `jj` semantics; empty/cancelled input is handled deliberately; success and
  failure output are inspectable; undo hint remains visible for mutations.
- Validation: command-construction tests for describe and commit; prompt/input tests; disposable
  `/tmp` `jj` repo proof for describe, commit, and undo (proof run from `/tmp` repo);
  `cargo   check`; focused tests; full `cargo test` when practical.
- Docs/fragility updates: update `progress.md`; record command-output or working-copy inference
  assumptions if any are introduced.
- Suggested agent/model: stronger implementation worker because prompt state and mutation semantics
  cross several modules.
- Review prompt: review prompt lifecycle, exact targets, empty/cancel behavior, output visibility,
  and `/tmp` disposable-repo proof.

## Packet 24: Bookmark Mutation Flows

- Goal: add safe guided bookmark set/create/move/delete/track/untrack flows from bookmark and graph
  context.
- Owner concept: bookmark screen and sync/ref command planning.
- Expected write set: `src/bookmarks.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`,
  `src/action_menu.rs` for graph-launched flows, focused tests, `docs/plan/progress.md`, and
  `docs/plan/fragility-register.md`.
- Non-goals: no host-specific pull request workflow, no tags, no force-push shortcut, and no attempt
  to fully model remote tracking unless the packet first establishes a robust contract.
- Acceptance criteria: bookmark names and targets are exact; destructive actions require
  confirmation; set/create/move previews show source and destination; track/untrack behavior is
  disabled unless tracking state is known; refresh preserves selected bookmark when possible; output
  is scrollable.
- Validation: command-construction tests for each supported bookmark command; bookmark parser tests
  for local/remote/tracking rows; view-level preview/result tests; disposable `/tmp` `jj` repo proof
  for create/move/delete and any feasible tracking flow (run from repo `cwd`); `cargo check`;
  focused tests.
- Docs/fragility updates: update bookmark metadata entries in `fragility-register.md`; update
  `progress.md`.
- Suggested agent/model: exploration first for tracking-state contract, then stronger implementation
  worker and review.
- Review prompt: review exact bookmark names, tracking-state gating, destructive confirmations,
  command construction, parser assumptions, and manual proof from isolated `/tmp` repos.

## Packet 25: Absorb Preview Flow

- Goal: make `jj absorb` available as a guided flow only when the preview can explain what will
  change.
- Owner concept: rewrite action planning and output review.
- Expected write set: `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`, focused tests,
  `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no automatic broad absorb from arbitrary context, no silent execution, and no
  line-level patch UI.
- Acceptance criteria: the flow explains source context and candidate descendants before
  confirmation; command shape is exact; users can cancel after preview; success/failure output is
  scrollable; ambiguous or unsupported absorb cases remain disabled or fall back to CLI use.
- Validation: command-construction tests; preview/result view tests; disposable `jj` repo proof with
  a simple absorb and undo in `/tmp` with `cwd` set to that repo; `cargo check`; focused tests;
  `just check` when practical.
- Docs/fragility updates: add absorb-specific rewrite fragility for preview/output assumptions;
  update `progress.md`.
- Suggested agent/model: exploration plus stronger implementation and review, because absorb
  semantics are subtle.
- Review prompt: review whether the preview honestly explains absorb effects, whether unsupported
  cases are blocked, and whether command/output assumptions are documented.

## Packet 26: Rebase Polish And Before/After Graph

- Goal: improve the existing rebase flow with clearer graph effect preview and post-action review.
- Owner concept: rewrite preview presentation.
- Expected write set: `src/action_menu.rs`, `src/app.rs`, `src/graph.rs`, `src/jj.rs`, `src/tui.rs`,
  focused tests, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no new rebase command variants unless the before/after preview requires a separately
  accepted sub-scope, and no in-memory reimplementation of `jj` graph semantics.
- Acceptance criteria: preview distinguishes current graph context from expected command effect;
  before/after information is either produced by `jj`/a disposable preview mechanism or clearly
  labeled as command summary, not simulated truth; success keeps affected stack visible; result
  output remains scrollable.
- Validation: view-level preview tests; command-construction regression tests; disposable `jj` repo
  proof for one representative rebase and undo (`cwd` set to an isolated `/tmp` repo);
  `cargo   check`; focused rebase tests; full `cargo test`.
- Docs/fragility updates: update the existing rebase preview entry with the chosen before/after
  contract; update `progress.md`.
- Suggested agent/model: stronger exploration before implementation, then stronger review, because
  graph preview can easily duplicate `jj` semantics.
- Review prompt: review whether before/after graph claims are sourced from `jj` or safely labeled,
  whether selection refresh still works, and whether the implementation avoids silent semantic
  reconstruction.

## Packet 27: Restore/Revert Guided Flows

- Goal: add preview-first restore and revert flows from graph, show/diff, or file context where
  targets are exact.
- Owner concept: file/revision rewrite command planning.
- Expected write set: `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`,
  `src/file_list.rs` or document views only for exact file-entry points, focused tests,
  `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no interactive patch editor, no broad pathspec parser beyond exact selected paths, and
  no operation restore/revert.
- Acceptance criteria: revision and path targets are explicit; destructive paths require
  confirmation; preview explains whether content is restored or reverse-applied; success/failure
  output is scrollable; undo hint is visible; unsupported ambiguous file paths are disabled.
- Validation: command-construction tests for revision-only and path-scoped shapes; parser tests for
  exact path extraction if touched; view-level preview/result tests; disposable `/tmp` `jj` repo
  proof for restore, revert, and undo with `cwd` set to that repo; `cargo check`; focused tests.
- Docs/fragility updates: record exact path/revision assumptions; update `progress.md`.
- Suggested agent/model: stronger implementation worker plus review; use exploration first if path
  contracts are not strong enough.
- Review prompt: review target exactness, path handling, destructive confirmation, command
  construction, manual proof from a `/tmp` working repo, and fragility documentation.

## Packet 28: Resolve Screen And Conflict Flow

- Goal: provide a focused conflict list and safe entry points for resolution work.
- Owner concept: resolve utility screen.
- Expected write set: new `src/resolve.rs` if needed, `src/main.rs`, `src/view_state.rs`,
  `src/app.rs`, `src/jj.rs`, `src/tui.rs`, `src/command.rs`, focused tests,
  `docs/plan/screens/resolve.md`, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no full merge editor, no automatic conflict resolution, and no destructive file rewrite
  without exact path contracts.
- Acceptance criteria: resolve screen lists conflict state using rendered `jj` output or a narrow
  contract; navigation and copy work for exact paths when known; actions that require external
  editors or exact resolution semantics are clearly deferred or launched explicitly; refresh
  preserves selection when possible.
- Validation: parser/rendering tests for conflict rows; navigation tests; command-construction tests
  for any launched commands; disposable conflicted `/tmp` `jj` repo proof if feasible (`cwd` set to
  temporary repo); `cargo check`; focused tests; `cargo test`.
- Docs/fragility updates: add resolve parser/contract entries; update screen spec and `progress.md`.
- Suggested agent/model: exploration first, then stronger implementation worker and review.
- Review prompt: review conflict-state contract, exact path handling, deferred actions, screen
  ownership, and safe degradation for surprising `jj` output.

## Packet 29: Day-To-Day Tutorial Set

- Goal: create concise tutorials/examples for the workflows `jk` actually supports.
- Owner concept: user documentation and demo scripts.
- Expected write set: `README.md`, `docs/tutorials/` or similar, existing VHS/demo specs if Packet
  21 landed, and `docs/plan/progress.md`.
- Non-goals: no generated GIFs/images in repo, no tutorial for unimplemented flows as if shipped,
  and no broad marketing rewrite.
- Acceptance criteria: tutorials cover the current daily loop: inspect log, show/diff, status,
  fetch/push, create new work, abandon change flow, operation recovery, and the implemented rewrite
  flows; each tutorial has deterministic setup or clear prerequisites; demo media references are
  hosted externally or generated under `target/vhs`.
- Validation: `just md-check`; run any demo setup commands if they exist; optional VHS execution if
  installed; manual truthfulness check against `progress.md`.
- Docs/fragility updates: update `progress.md`; no fragility entry unless tutorials encode parser or
  output assumptions as test fixtures.
- Suggested agent/model: documentation worker plus review for product truthfulness.
- Review prompt: review tutorials for shipped-behavior accuracy, reproducibility, generated media
  hygiene, and alignment with Ratatui media guidance.

## Packet 30: Edit/Next/Prev Navigation Flows

- Goal: add high-frequency working-copy navigation commands for moving edit focus through a stack.
- Owner concept: graph-guided working-copy navigation.
- Expected write set: `src/graph.rs`, `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`,
  focused tests, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no rebase/squash coupling, no automatic commit, and no hidden stack traversal beyond
  exact `jj` command semantics.
- Acceptance criteria: edit/next/prev actions show exact target or `jj` default semantics before
  running; success refreshes and reveals the new working copy; failures keep output readable; unsafe
  or ambiguous contexts are disabled or prompt explicitly.
- Validation: command-construction tests; view-level action/result tests; disposable `/tmp` `jj`
  repo proof for edit, next, prev, and undo where applicable (run from that repo's `cwd`);
  `cargo check`; focused tests.
- Docs/fragility updates: record any reliance on `jj next`/`prev` default target resolution; update
  `progress.md`.
- Suggested agent/model: stronger implementation worker plus review.
- Review prompt: review target/default-semantics wording, refresh behavior, output visibility, and
  manual proof from `/tmp` disposable repos.

## Packet 31: Command Coverage Audit And Passthrough Policy

- Goal: tighten the command inventory after the next guided flows land and decide which remaining
  commands stay passthrough.
- Owner concept: planning and command coverage documentation.
- Expected write set: `docs/plan/command-inventory.md`, `docs/plan/workflows.md`, relevant
  `docs/plan/workflows/*.md`, `docs/plan/progress.md`, and possibly `README.md` if user-facing
  coverage language changes.
- Non-goals: no code changes, no new command implementation, and no speculative support claims.
- Acceptance criteria: common day-to-day commands are mapped to native screen, guided flow, or
  passthrough with current evidence; unsupported dangerous flows have explicit rationale; follow-on
  packets are created or amended for any newly promoted commands.
- Validation: `just md-check`; manual consistency check against `progress.md` and implemented
  command bindings.
- Docs/fragility updates: update `progress.md`; fragility register usually unchanged unless the
  audit identifies a new planned soft contract.
- Suggested agent/model: planning/documentation worker; stronger review if command policy changes
  product scope.
- Review prompt: review command classifications for truthfulness, product focus, dangerous-command
  safety, and consistency with shipped behavior.

## Packet 32: Strong Command-Coverage Follow-Through

Goal: turn Packet 31's command coverage audit into bounded implementation packets. This packet is a
docs-only follow-through plan, not a claim that any listed behavior is shipped.

The sequence below promotes only command groups where `jk` can add product value through exact
context, preview, readable result output, or a focused read surface. It intentionally does not
schedule a generic command mode or broad CLI mirror. Shipped daily flows such as `new`, `edit`,
`next`, `prev`, `describe`, `commit`, `rebase`, `squash`, `abandon`, `restore`, `revert`, `absorb`,
`undo`, `redo`, bookmark create/set/move/delete, fetch, and push stay in maintenance mode unless a
specific regression or polish packet is accepted later.

### Packet 33: Operation Restore/Revert From Operation Log

- Goal: add explicit recovery flows for `jj operation restore` and `jj operation revert` from exact
  operation-log rows.
- Owner concept: operation-log guided recovery actions and operation command construction.
- Expected write set: `src/operation_log.rs`, `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`,
  `src/tui.rs`, focused tests, `docs/plan/progress.md`, `docs/plan/fragility-register.md`, and
  `docs/plan/workflows/recover.md` only if shipped wording changes.
- Non-goals: no `operation abandon`, no `operation integrate`, no transaction graph simulation, and
  no arbitrary operation id text entry.
- Acceptance criteria: actions appear only when the selected row has an exact operation id; previews
  distinguish restore from revert and show the exact operation id and command; confirmation is
  required; success refreshes operation log and the active repo view where practical; failure and
  result output remain scrollable; undo/redo recovery wording stays visible without implying the
  selected row is used by global undo/redo.
- Validation: command-construction tests for both operation commands; operation-log disabled-state
  tests for rows without ids; view-level preview, cancel, confirm, and result-output tests; output
  tests that preserve multiline success/failure text; disposable `/tmp` jj repo proof for restore,
  revert, failure or disabled behavior, and recovery, with every mutation command run from the proof
  repo's `cwd`; `cargo check`; focused operation tests; full `cargo test` and `just check` when
  practical.
- Docs/fragility updates: add or update operation recovery entries for exact operation-id command
  semantics and any output assumptions; update `progress.md`.
- Suggested agent/model routing: gpt-5.5 high implementation plus gpt-5.5 high review, because
  recovery semantics are easy to overstate and failures must stay inspectable.
- Review prompt: review Packet 33 for operation id exactness, restore/revert wording, command
  construction, disabled states, output preservation, `/tmp` proof quality, and whether recovery
  semantics are claimed only where `jj` proves them.

### Pre-Packet-34 Interruption: Maintainability And UI Repair Wave

Packet 33 is accepted. Packet 34 remains the next rewrite-feature packet, but it is intentionally
postponed behind this interruption wave. The interruption exists because the next work should reduce
maintainer load before adding another guided rewrite flow: `app.rs` is carrying too many unrelated
responsibilities, several screen-level contracts are implicit, and small UI regressions now point to
missing ownership boundaries.

Run interruption packet A first. Later interruption packets should be promoted only after packet A
defines the owning module and screen/action contract for that area. Do not run two interruption
packets in parallel when their expected write sets overlap. If packet A introduces narrower modules,
future prompts should use those new owner files instead of sending unrelated work back through
`src/app.rs`.

After the first Packet A extraction is accepted, run a bounded module-coherence audit before
starting broad follow-up refactors. The audit should look for the same low-cognitive-load and
high-coherence pressure that motivated the `app.rs` extraction, not reopen the already accepted
screen, status, and action-output split.

#### Interruption Packet A: App Decomposition And Screen Contracts

- Goal: decompose the oversized app orchestration surface and define screen/action contracts before
  adding Split Guided Flow.
- Current implementation note: first extraction has landed for modal/screen projection, status-line
  construction, and action-output key handling. Remaining Packet A follow-ups should consider
  command-runner injection, startup parsing, and the app test module only if they preserve behavior
  and strengthen the same ownership map.
- Owner concept: app-level event dispatch, screen ownership, action execution contracts, prompt and
  overlay lifecycle, and maintainability guidance for future UI work.
- Expected write set: `src/app.rs`, new focused app/screen/action modules if introduced,
  `src/view_state.rs`, `src/command.rs` only for contracts that must move with dispatch, focused
  screen-level tests in touched modules, `docs/agent/architecture.md`,
  `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`.
- Non-goals: no new user-visible commands, no Packet 34 split implementation, no broad visual
  redesign, no command-key remapping except where required to preserve existing behavior during the
  extraction, and no cleanup of unrelated parser or `jj` command semantics.
- Acceptance criteria: each active screen has an explicit home for keys, actions, prompts/overlays,
  command execution, refresh/back behavior, and tests; app-level dispatch no longer owns detailed
  policy for every screen; action output, prompts, help/leader behavior, and screen transitions have
  named contracts; existing shipped Packet 33 behavior and earlier daily flows still pass focused
  regression tests; future packets can name a non-overlapping owner/write set without defaulting to
  `src/app.rs`.
- Validation: focused tests for every moved screen/action contract; screen-level regression tests
  for graph, status, bookmarks, operation log, action output, and existing guided flows that move;
  full `cargo test`; `cargo clippy -- -D warnings` or the repository-approved no-warning equivalent;
  `cargo run` smoke check with no warnings when practical; `rustup run nightly cargo fmt --check`;
  `just md-check`; `just check` before handoff when the local wrapper is healthy, otherwise report
  the known wrapper failure and equivalent direct checks.
- Docs/fragility updates: update architecture docs with the new ownership map and update
  `progress.md`; update `fragility-register.md` only if extraction changes a parser, rendered-output
  assumption, or command semantic contract.
- Suggested agent/model routing: gpt-5.5 high implementation and gpt-5.5 high review. This is
  cross-module architecture work with high cognitive-load risk, and the user explicitly requested
  that this refactoring/design pass use a high-capability model rather than a mini model.
- Review prompt: review Interruption Packet A for concept coherence, reduced cognitive load,
  behavior preservation, screen/action contract clarity, test ownership, documentation updates,
  warning discipline, and whether follow-up packets can proceed without broad `src/app.rs` edits.

#### Interruption Packet A Follow-Up: Module Coherence Audit

- Goal: inspect other large or concept-mixed modules for the same maintainability pressure surfaced
  by `app.rs`, and turn only well-evidenced split candidates into future bounded packets.
- Owner concept: maintainability planning, module ownership, and cognitive-load reduction across
  current large files.
- Expected write set: planning and architecture docs first:
  `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`,
  `docs/process-observations.md`, and optionally `docs/agent/architecture.md` if the audit records
  durable owner-map updates. Rust edits are not part of the audit unless the user explicitly
  promotes a split candidate into its own implementation packet.
- Candidate order: start with `src/jj.rs` because it is the largest remaining non-`app.rs` module by
  current line count and mixes command construction, rendered-output loading, metadata contracts,
  and parsing-adjacent helpers. Inspect `src/tui.rs`, `src/graph.rs`, `src/command.rs`,
  `src/action_menu.rs`, and `src/view_state.rs` next as secondary candidates. Treat
  `src/rendered_jj.rs`, `src/operation_log.rs`, and `src/bookmarks.rs` as optional context only if
  the primary candidates expose ownership dependencies on them.
- Non-goals: no immediate broad refactor, no behavior change, no Packet 34 Split Guided Flow, no
  warning-cleanup sweep, no line-count target, no moving code only because a file is large, and no
  blocking acceptance of the first Packet A extraction.
- Acceptance criteria: the audit identifies each candidate module's owning concepts, concept seams
  that are already named in code or docs, possible split candidates, reasons to avoid splitting, and
  validation needs for any promoted implementation packet. It should explicitly distinguish
  high-value splits from files that are large but conceptually coherent enough to leave alone.
- Validation: documentation-only audit uses `just md-check` and a manual consistency check against
  current module ownership notes. Any future implementation packet promoted by the audit must define
  its own focused Rust tests and validation before code is changed.
- Docs/fragility updates: update `progress.md` and `process-observations.md`; update
  `fragility-register.md` only if the audit records a parser, rendered-output, or command semantic
  assumption that should guide a future implementation packet.
- Suggested agent/model routing: gpt-5.5 high for the audit and gpt-5.5 high review. This is design
  and review work over module boundaries, not mechanical line-count cleanup.
- Review prompt: review the module-coherence audit for evidence, boundedness, owner concept clarity,
  non-goals, validation requirements, and whether each promoted split candidate is small enough for
  a subagent without overlapping the current Packet A extraction.

Audit result from 2026-05-20:

- `src/jj.rs`: high-value split candidate. The module currently owns action/mutation command plans
  and preview text, `ViewSpec` and view-mode command shape, direct `jj` process execution, rendered
  row item models, metadata-template loading, row grouping, and parsers in one file. The
  cognitive-load issue is not line count alone; future command packets must scan unrelated
  rendered-output and view-spec code to review an action-plan change, while parser or row-loading
  changes must share a test module with every command-builder contract. Promote the command-plan
  extraction below first.
- `src/tui.rs`: large but coherent enough for now. It owns shared chrome and overlay rendering as
  documented. The main pressure is repeated action-output overlay arms and repeated selected-list
  styling, which can be cleaned locally when a UI packet touches overlay presentation. Do not split
  it before a concrete rendering change.
- `src/graph.rs`: large but conceptually coherent. It owns graph selection, search, refresh
  preservation, graph-mode switching, exact change-id selection, and graph-to-action-menu context.
  The action-menu handoff is view policy rather than a separate owner today. Leave it intact until
  Packet B navigation work proves a smaller view-entry contract is needed.
- `src/command.rs`: moderately mixed but acceptable. It combines command/effect vocabulary,
  key-pattern metadata, and generated-help projection, which are intentionally coupled by the
  current architecture. Revisit only if Packet B adds multi-character grammar or help projection
  grows enough to make command vocabulary harder to audit.
- `src/action_menu.rs`: coherent. It owns preview-first action vocabulary, exact action contexts,
  role prompts, menu labels, and follow-up values without executing commands. Watch it as more
  mutation families arrive, but do not split before the next command-plan extraction clarifies the
  boundary with `jj` command builders.
- `src/view_state.rs`: coherent dispatcher with one growth point. The routing methods are repetitive
  by design, while `push_target`, `bookmark_target`, `selected_local_bookmark_name`, and
  `exact_restore_revert_context` are cross-view target contracts. Keep them here for now; consider a
  later target-contract extraction only if Packet B or later action packets add several more
  cross-view target methods.

#### Interruption Packet A1: Extract `jj` Action Plans

- Goal: move preview-first `jj` action/mutation command plans out of `src/jj.rs` while preserving
  the existing public API through compatibility re-exports if needed.
- Owner concept: `jj` command-plan contracts for preview-first actions, including argv shape,
  labels, preview summaries, direct execution, exact change-id revsets, exact bookmark patterns, and
  fallback result wording.
- Expected write set: `src/jj.rs`, one new focused module such as `src/jj_actions.rs` or an
  equivalent owner-named file, `src/main.rs` for the module declaration, focused moved tests from
  the current `src/jj.rs` test module, `docs/agent/architecture.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`.
- Non-goals: no command semantic changes, no output/parser extraction, no `ViewSpec` redesign, no
  broad call-site churn, no new user-visible commands, no Packet 34 Split Guided Flow, and no
  warning-cleanup sweep.
- Acceptance criteria: action-plan types and methods for operation recovery/target actions, git
  push, new, describe, commit, edit/next/prev, restore, revert, bookmark mutations, rebase, squash,
  absorb, and abandon have a coherent owner outside the row-loading/parser code; existing imports
  still compile or are updated mechanically; command argv, preview text, fallback messages, and
  exact quoting behavior are unchanged; `src/jj.rs` remains a smaller facade for view specs,
  loading, process boundaries, and rendered-output conversion until a later packet moves those
  concepts.
- Validation: focused command-construction tests moved with the command-plan owner; focused abandon,
  restore/revert, bookmark, operation, push, and working-copy navigation tests; full `cargo test`;
  `cargo check`; `rustup run nightly cargo fmt --check`; `just md-check`; `just check` when the
  local wrapper is healthy, otherwise report the known wrapper failure and equivalent checks.
- Docs/fragility updates: update architecture ownership notes and progress/process docs. Update
  `fragility-register.md` only if the extraction changes a parser, rendered-output assumption, or
  command semantic contract; a behavior-preserving move should leave it unchanged.
- Suggested agent/model routing: gpt-5.5 high implementation and gpt-5.5 high review because this is
  a cross-module Rust extraction with many mutation command contracts and high regression risk.
- Review prompt: review Packet A1 for behavior preservation, command argv exactness, preview/result
  wording stability, public API churn, test ownership, and whether `src/jj.rs` now separates command
  plans from rendered-output loading enough for later packets.

#### Interruption Packet A2: Extract `jj` Rendered Row Loading

- Goal: after Packet A1 lands, separate rendered `jj` row item models, metadata pairing, loaders,
  and parsers from remaining `ViewSpec` and process-boundary code.
- Owner concept: conversion from rendered `jj` output plus narrow metadata-template streams into the
  selectable rows used by graph, bookmark, file-list, resolve, and operation-log views.
- Expected write set: `src/jj.rs`, one new focused module such as `src/jj_rows.rs`, `src/main.rs`
  for the module declaration, moved parser/row tests from `src/jj.rs`, `docs/agent/architecture.md`,
  `docs/plan/progress.md`, `docs/plan/fragility-register.md` only for newly clarified or changed
  soft contracts, and `docs/process-observations.md`.
- Non-goals: no command-plan changes, no `rendered_jj.rs` sticky-file extraction, no lower-level
  `jj_lib` integration, no new parser assumptions, no behavior changes to row grouping or metadata
  pairing, and no Packet 34 Split Guided Flow.
- Acceptance criteria: `LogItem`, `BookmarkItem`, `FileListItem`, `ResolveEntry`, and
  `OperationLogItem` plus their load/group/parse helpers live under one row-loading owner; row
  grouping, bookmark metadata pairing, operation-id pairing, resolve JSON parsing, file-list path
  preservation, and ANSI-to-Ratatui conversion behavior are unchanged; `ViewSpec` continues to own
  command identity and navigation target provenance; parser tests move with the parser owner and
  still document graceful degradation.
- Validation: focused row/parser tests for log grouping, bookmark pairing, operation-log id
  grouping, resolve JSON degradation, file-list path preservation, and view-spec navigation
  contracts left in `src/jj.rs`; full `cargo test`; `cargo check`;
  `rustup run nightly cargo fmt --check`; `just md-check`.
- Docs/fragility updates: review existing fragility entries for row-order metadata pairing and
  rendered-output parsing. Update only if the extraction changes wording, adds a new assumption, or
  reveals a current undocumented soft contract with precise code/test evidence.
- Suggested agent/model routing: gpt-5.5 high implementation and gpt-5.5 high review because this
  touches rendered-output contracts and parser tests.
- Review prompt: review Packet A2 for parser behavior preservation, metadata pairing exactness,
  graceful degradation, test locality, and whether `src/jj.rs` now reads as a small
  process/view-spec facade rather than a mixed parser and command-plan module.

#### Interruption Packet B: Navigation And View Entry Contracts

- Goal: make view entry and directional navigation coherent before new rewrite flows add more keys.
- Owner concept: command grammar, generated key metadata, view entry routing, and screen-local
  navigation contracts.
- Expected write set: `src/command.rs`, `src/view_state.rs`, screen owner modules identified by
  packet A, focused tests, `docs/plan/progress.md`, and user-facing docs only if shipped keys
  change.
- Non-goals: no Split Guided Flow, no command mode, no broad shortcut redesign, and no mutation
  command changes.
- Acceptance criteria: status is selectable from log with `S`; bookmark and operation-log view entry
  keybinds that are documented or expected by the app work consistently; a view menu can select log,
  status, bookmarks, operation log, and other shipped views without pretending future screens exist;
  multi-character command grammar is designed for commands such as `bc` bookmark create and `gf` git
  fetch; right/`l` expands or opens where a selected item has expandable detail, while left/`h`
  collapses or backs out according to the screen contract; generated help and dispatch agree.
- Validation: command metadata tests; generated-help tests; screen-level dispatch tests for `S`,
  bookmarks, operation log, view menu selection, multi-character prefixes/timeouts/cancel behavior,
  and `h`/`l` expansion/collapse; `cargo check`; full `cargo test`; warning-free/clippy check when
  the packet changes Rust.
- Docs/fragility updates: update `progress.md`; update command inventory or tutorials only after
  behavior ships; no fragility update unless new rendered-output assumptions are added.
- Suggested agent/model routing: gpt-5.5 high exploration and implementation if multi-character
  grammar is included; split out smaller subpackets if the grammar and view-menu work cannot share a
  single owner after packet A.
- Review prompt: review navigation contracts for key conflicts, generated-help truthfulness,
  screen-local ownership, prefix ambiguity, and regression coverage for shipped view entry keys.

#### Interruption Packet C: Help Leader Menu

- Goal: turn help from a passive key listing into a keyboard-driven leader-style command surface.
- Owner concept: help/leader menu state, command grouping, and dispatch back into existing commands.
- Expected write set: help/menu owner module from packet A, `src/command.rs`, `src/tui.rs` only for
  help rendering if it remains the owner, focused tests, and `docs/plan/progress.md`.
- Non-goals: no generic command palette, no fuzzy search, no new mutation commands, and no free-form
  command mode.
- Acceptance criteria: pressing a listed option executes that command and closes the menu; the menu
  has an explicit close option that closes without executing; entries are grouped by user operation
  such as navigation, view switching, search/copy, action previews, and recovery rather than sorted
  only by key; unavailable commands are hidden or disabled according to the screen contract; the
  help/leader surface remains keyboard-only friendly.
- Validation: view-level help/leader tests for execute-and-close, close-only, disabled command
  behavior, grouping text, and selected-screen command availability; generated-help metadata tests;
  `cargo check`; focused tests and full `cargo test` when practical; warning-free/clippy check for
  Rust changes.
- Docs/fragility updates: update `progress.md`; update user docs/tutorials only after the shipped
  interaction changes.
- Suggested agent/model routing: gpt-5.5 high implementation if the leader menu executes commands
  through shared dispatch; smaller implementation is acceptable only if packet A already isolated
  the menu state cleanly.
- Review prompt: review the leader menu for command grouping, execute-and-close semantics, close
  affordance, disabled-state correctness, generated metadata reuse, and keyboard-only usability.

#### Interruption Packet D: Action Menu, Popovers, And Selection Presentation

- Goal: make action and popover surfaces keyboard-driven, themed, and visibly connected to the
  current selection.
- Owner concept: action menu interaction, popover rendering, selected-row presentation, and theme
  fallback policy.
- Expected write set: `src/action_menu.rs`, popover/rendering owner modules identified by packet A,
  `src/tui.rs` only where rendering remains centralized, focused rendering and interaction tests,
  `docs/plan/progress.md`, and `docs/plan/fragility-register.md` only if color/style assumptions
  depend on rendered `jj` output.
- Non-goals: no new action commands, no broad theme system, no terminal color configurator, and no
  rewrite of every screen outside the selection/action/popover surfaces.
- Acceptance criteria: selected items in lists have a visible highlight or color treatment; the
  action menu is driven by keys instead of requiring menu-style selection movement for common
  commands; popovers use colors and fallback styles that fit `jj` output and the active terminal
  theme better than black-and-white defaults; small terminals keep labels readable; selected context
  remains clear when action previews, popovers, or overlays open and close.
- Validation: rendering snapshots or snapshot-style tests for selected rows, action menu, popovers,
  narrow terminals, and no-color/low-color fallback where practical; interaction tests for
  key-driven action execution and close/cancel behavior; `cargo check`; full `cargo test`;
  warning-free/clippy check for Rust changes.
- Docs/fragility updates: update `progress.md`; add a fragility entry only if style decisions depend
  on parsing or inferring colors from rendered `jj` output.
- Suggested agent/model routing: gpt-5.5 high implementation or review because visual state,
  terminal compatibility, and action routing cross ownership boundaries.
- Review prompt: review selection visibility, keyboard-driven action behavior, popover theme
  coherence, small-terminal rendering, no-color fallback, and absence of new command semantics.

#### Interruption Packet E: Status File Actions

- Goal: make status files selectable and action-capable through exact file-path contracts.
- Owner concept: status screen file selection, exact path ownership, and file-scoped action routing.
- Expected write set: `src/status.rs`, file action owner modules identified by packet A, `src/jj.rs`
  only for exact file command construction, focused tests, `docs/plan/progress.md`, and
  `docs/plan/fragility-register.md`.
- Non-goals: no patch editor, no broad fileset input, no conflict resolver, and no path inference
  from rendered sticky headings.
- Acceptance criteria: status rows support file selection where exact repo-relative paths are known;
  file-scoped actions are enabled only for exact paths; unsupported or ambiguous status rows degrade
  to disabled state with readable status; selected file actions preserve full command output and
  refresh selection safely.
- Validation: status parser/path ownership tests, screen-level status selection tests, command
  construction tests for each promoted file action, result-output tests, disposable `/tmp` jj repo
  proof for any mutation, `cargo check`, full `cargo test`, and warning-free/clippy check.
- Docs/fragility updates: update exact-path and status-output assumptions in
  `fragility-register.md`; update `progress.md`.
- Suggested agent/model routing: gpt-5.5 high implementation and review because exact file paths are
  mutation-critical.
- Review prompt: review status file exactness, disabled ambiguous rows, action availability, command
  construction, refresh behavior, output preservation, and `/tmp` proof quality.

#### Interruption Packet F: Fetch Remote Selection

- Goal: let fetch use the default behavior quickly while also supporting an explicit remote choice.
- Owner concept: sync/fetch action planning and remote selection UI.
- Expected write set: sync/fetch owner module from packet A, `src/jj.rs`, `src/command.rs` only for
  key metadata, focused tests, `docs/plan/progress.md`, and `docs/plan/fragility-register.md` if
  remote-list parsing changes.
- Non-goals: no host dashboard, no credential management, no push redesign, and no remote editing.
- Acceptance criteria: fetch offers an option to fetch from a specific remote; one-remote and
  default-fetch paths remain low friction; no-remote and credential/error output stay readable;
  preview/result wording distinguishes default fetch from remote-specific fetch.
- Validation: command-construction tests for default and remote-specific fetch; remote-list parser
  or selection tests; view-level preview/result/error tests; disposable `/tmp` or local disposable
  remote proof where feasible; `cargo check`; full `cargo test`; warning-free/clippy check.
- Docs/fragility updates: update remote-list/fetch assumptions if they change; update `progress.md`.
- Suggested agent/model routing: gpt-5.5 high if remote selection shares the same picker/action
  surface as push; otherwise a narrower implementation with stronger review is acceptable.
- Review prompt: review fetch target wording, remote-list exactness, default behavior preservation,
  credential/error output preservation, and remote proof coverage.

#### Interruption Packet G: File Viewing And Wrap Modes

- Goal: improve file viewing with formatting-preserving display and an explicit no-wrap mode.
- Owner concept: file show/document view rendering, wrap policy, and per-view display mode state.
- Expected write set: `src/file_show.rs`, `src/sticky_file_view.rs`, `src/rendered_jj.rs` only if
  formatting preservation needs parser support, focused tests, `docs/plan/progress.md`, and
  `docs/plan/fragility-register.md` if rendered-output assumptions change.
- Non-goals: no editor, no syntax-highlighting engine unless separately justified, no mutation, and
  no global terminal reflow redesign.
- Acceptance criteria: users can view files with formatting preserved where `jj` output already
  provides it; a no-wrap view is available for content such as Markdown where wrapping obscures
  structure; wrap/no-wrap state is discoverable through generated help or the leader menu; scroll,
  search, sticky file context, and copy continue to behave predictably.
- Validation: document-view rendering tests for wrapped and no-wrap modes, Markdown-like long-line
  examples, horizontal/vertical scroll boundaries, search/copy behavior, and sticky context;
  `cargo check`; full `cargo test`; warning-free/clippy check for Rust changes.
- Docs/fragility updates: update `progress.md`; update fragility entries if the implementation
  parses additional file headings, ANSI styles, or rendered formatting markers.
- Suggested agent/model routing: gpt-5.5 high implementation or review because wrap policy affects
  document navigation and presentation.
- Review prompt: review formatting preservation, no-wrap ergonomics, scroll/search/copy behavior,
  sticky context, generated help, and parser fragility.

#### Interruption Packet H: Validation And Contribution Discipline

- Goal: make warning-free builds and commit-message discipline part of the finished packet contract.
- Owner concept: repository validation workflow, agent handoff expectations, and contribution
  guidance.
- Expected write set: `Justfile`, repo-local agent/process docs, relevant planning docs, and CI or
  lint config only if the repo already has a clear home for it.
- Non-goals: no Rust behavior changes, no dependency churn unless required by the chosen lint gate,
  no Git workflow examples, and no rewrite of historical progress entries that were true at the
  time.
- Acceptance criteria:
  - finished implementation packets require a warning-free `cargo run` smoke when practical, or a
    clear reason why it was skipped;
  - Rust packets include `cargo clippy -- -D warnings` or the repository's documented equivalent;
  - if baseline blockers remain, handoffs list the exact blockers and whether they are unchanged;
  - commit descriptions follow the 50-character title and 72-column body guidance already used in
    the repo's jj workflow;
  - handoffs report validation proof, skipped checks, and no-warning status explicitly.
- Validation: `just md-check`; run any new or changed validation recipe; no Rust tests required
  unless validation tooling touches Rust build configuration.
- Docs/fragility updates: update `progress.md`; no fragility-register update unless validation
  begins depending on unstable external output.
- Suggested agent/model routing: documentation/tooling worker with maintainer review; use gpt-5.5
  high review if the packet changes the standard finished-check contract for all future agents.
- Review prompt: review validation discipline for command accuracy, jj terminology, warning policy,
  commit-message rules, exact blocker reporting, and whether future packets have a mechanical way to
  prove or explicitly skip no-warning behavior.

### Packet 34a: Split Process-Boundary Spike

- Goal: decide the process boundary for `jj split` before implementing a product split flow, because
  command planning is clear but interactive editor handoff through the current captured-process
  runner is not proven.
- Owner concept: rewrite command planning and terminal/editor process lifecycle.
- Expected write set: `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`,
  `docs/plan/fragility-register.md`, and `docs/process-observations.md`. Rust edits are allowed only
  for a compile-breaking docs reference, and any mutation proof must use a disposable `/tmp` jj repo
  with the proof repo as command `cwd`.
- Non-goals: no product split command, no `JjSplitPlan`, no terminal runner refactor, no in-app
  patch editor, no generated media, and no mutation proof in this repository.
- Command contract: preserve the planned command shapes exactly: use bare `jj split` for the
  visible/current `@` path, and use `jj split --revision exactly(change_id("<id>"), 1)` for an exact
  graph target.
- Process boundary: no-fileset split delegates patch selection to `jj`'s diff editor and may also
  invoke description editing. `jk` must not describe this as an in-app patch editor or imply that it
  can choose split hunks without handing control to `jj`.
- Acceptance criteria: Packet 34 records that the current captured output runner is not proven to
  support real interactive editor handoff; Packet 34 either depends on an interactive
  process/terminal suspension runner or explicitly ships only preview/readable failure semantics;
  output preservation remains part of the contract.
- Validation: `just md-check`; no active-repo mutation proof. If additional proof is needed, create
  a disposable `/tmp` jj repo and run every mutation with that repo as process `cwd`, then record
  the exact commands and output summary in progress/process docs.
- Docs/fragility updates: add or update the split editor/process-boundary entry in
  `fragility-register.md`; update progress and process observations with the explorer evidence and
  remaining risk.
- Suggested agent/model routing: docs implementation can be narrow, but review should use stronger
  model scrutiny because the decision gates a risky interactive rewrite flow.
- Review prompt: review Packet 34a for command-shape accuracy against `jj split --help`,
  process-boundary honesty, evidence quality, docs consistency, and whether Packet 34 is now
  safer/better bounded.

### Packet 34c: Interactive Split Process Runner

- Goal: add and prove the smallest process-boundary primitive needed to hand interactive `jj split`
  from the Ratatui app to jj's diff editor.
- Owner concept: terminal suspension in `app.rs` / Ratatui setup and inherited-stdio `jj` process
  execution in `jj.rs`.
- Expected write set: likely `src/app.rs`, `src/app/services.rs`, `src/jj.rs`, or a narrowly named
  terminal/process helper if that is clearer; focused tests near the owner; `docs/plan/progress.md`
  and `docs/process-observations.md`.
- Non-goals: no product split action, no `JjSplitPlan`, no action-menu split flow, no in-app patch
  editor, no broad terminal lifecycle rewrite, and no active-repo mutation proof.
- Acceptance criteria:
  - a confirmed interactive command path leaves raw mode and the alternate screen before spawning
    the child process, then restores the Ratatui terminal before returning to the event loop;
  - the child process inherits stdin, stdout, and stderr instead of using `Command::output()`;
  - failures while spawning, waiting, or restoring the terminal return readable runner status
    without leaving the app in raw/alternate-screen limbo;
  - inherited child stdout/stderr is treated as live terminal handoff output, not captured result
    text that remains visible after the alternate screen is restored;
  - the runner is testable without launching a real editor, with unit tests covering command
    construction, inherited-stdio intent, and restore-on-error behavior where practical;
  - the final manual proof uses only disposable `/tmp` jj repositories and runs every mutation from
    the proof repo's `cwd`.
- Validation: `cargo check`; focused owner tests; `rustup run nightly cargo fmt --check`;
  `just md-check`; manual `/tmp` proof that a noninteractive failure still returns cleanly and, if
  practical in a real terminal, that an inherited-stdio split editor handoff can be cancelled or
  completed without corrupting the TUI terminal state.
- Docs/fragility updates: update `progress.md`; update the split fragility entry only if the runner
  introduces new assumptions about jj output wording or terminal escape behavior.
- Suggested agent/model routing: gpt-5.5 high implementation or review, because the packet crosses
  terminal lifecycle, process stdio, and mutation safety.
- Review prompt: review Packet 34c for terminal lifecycle correctness, inherited stdio, restoration
  on success/failure/panic-adjacent paths, exact `/tmp` proof cwd discipline, tests, and whether it
  is small enough for Packet 34 to depend on without absorbing split UI behavior.

### Packet 34: Split Guided Flow

- Goal: add a bounded `jj split` flow for the current or exact selected change when the UI can
  explain the editor handoff and result path honestly.
- Owner concept: rewrite action planning and editor/process-boundary policy.
- Expected write set: `src/action_menu.rs`, `src/app.rs`, `src/graph.rs`, `src/jj.rs`, `src/tui.rs`,
  focused tests, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no in-app patch editor, no line-level split UI, no automatic split decisions, no broad
  `diffedit` support, and no noninteractive fake split.
- Prerequisite: Packet 34a and the Packet 34b follow-up spike chose a dedicated implementation
  boundary. Packet 34c must first add and prove a terminal-suspension / inherited-stdio runner for
  real editor handoff. Until that lands, no-fileset interactive split must remain preview-only or
  absent from the product flow rather than executing through the captured runner.
- Command contract: preserve the planned command shapes exactly: bare `jj split` for the
  visible/current `@`, and `jj split --revision exactly(change_id("<id>"), 1)` for exact graph
  targets.
- Acceptance criteria: the flow targets an exact change or visible `@` according to launch context;
  preview shows the exact command and says when `jj` will launch an editor or interactive selector;
  no-fileset split is described as delegating patch selection to `jj`'s diff editor and possible
  description editor, not as an in-app patch editor; cancel before execution is supported;
  post-command success and failure status/results are readable through an explicit app surface,
  because inherited child output may no longer be visible after Ratatui restores the alternate
  screen; the graph refresh reveals the relevant working copy or split result when possible;
  unsupported noninteractive environments fail through readable app status/result text and may also
  print `jj` output while the terminal is suspended.
- Validation: command-construction tests for supported split shapes; view-level preview, cancel,
  confirm, and result tests; result/status tests that distinguish clean child nonzero status from
  terminal restore failures; either terminal/editor handoff proof through the Packet 34c runner or
  explicit tests proving the command stays preview-only until that runner exists; disposable `/tmp`
  jj repo proof for any feasible mutation or failure path, with mutations run from the proof repo's
  `cwd`; `cargo check`; focused rewrite tests; full `cargo test` and `just check` when practical.
- Docs/fragility updates: record the editor/process boundary and any command-output assumptions;
  update `progress.md`.
- Suggested agent/model routing: gpt-5.5 high implementation and review, because Packet 34a showed
  command shape is clear while interactive editor semantics remain the key risk.
- Review prompt: review Packet 34 for honest split/editor semantics, exact target handling,
  noninteractive failure behavior, explicit post-command status/result visibility, and evidence that
  the flow does not pretend to be an in-app patch editor.

### Packet 35: Duplicate Guided Flow

- Goal: add a preview-first `jj duplicate` flow for exact selected changes.
- Owner concept: graph rewrite command planning.
- Expected write set: `src/action_menu.rs`, `src/app.rs`, `src/graph.rs`, `src/jj.rs`, `src/tui.rs`,
  focused tests, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no bulk/range duplicate, no duplicate-and-rebase combo, no bookmark movement, and no
  target guessing from rendered labels.
- Acceptance criteria: the action is available only for exact graph or detail-view revision targets;
  preview lists every exact source if multi-source support is deliberately accepted, otherwise it
  allows one source only; confirmation is required; success refreshes and reveals the duplicated
  change or a recent-work fallback; failures preserve full `jj` output; `jj undo` is visible.
- Validation: command-construction tests; view-level preview, cancel, confirm, and result tests;
  output/result tests for success and failure; disposable `/tmp` jj repo proof for duplicate and
  undo from the proof repo's `cwd`; `cargo check`; focused rewrite tests; full `cargo test` and
  `just check` when practical.
- Docs/fragility updates: record duplicate result-selection assumptions if refresh/reveal depends on
  rendered output or operation output; update `progress.md`.
- Suggested agent/model routing: gpt-5.5 high implementation plus review; use a smaller helper only
  for command-help exploration.
- Review prompt: review Packet 35 for exact source targeting, bounded source count, refresh/reveal
  behavior, command construction, output preservation, and `/tmp` undo proof.

### Packet 36: Bookmark Tracking Metadata Contract

- Goal: extend bookmark row state with explicit remote and tracking metadata so later tracking
  mutations do not infer state from rendered labels.
- Owner concept: bookmark utility screen metadata contract.
- Expected write set: `src/bookmarks.rs`, `src/jj.rs`, `src/tui.rs` if row labels need minor
  metadata presentation, focused parser/view tests, `docs/plan/progress.md`, and
  `docs/plan/fragility-register.md`.
- Non-goals: no track/untrack mutation, no bookmark rename/forget, no push behavior change, and no
  host-specific remote UI.
- Acceptance criteria: local, remote, tracked, untracked, and ambiguous bookmark rows have explicit
  internal state where `jj` exposes it; rows with insufficient metadata degrade to non-actionable
  state; rendered labels remain preserved for presentation; existing create/set/move/delete behavior
  is unchanged; the contract is documented before write actions depend on it.
- Validation: bookmark metadata parser tests for local-only, remote-only, tracked, untracked,
  missing, and extra metadata rows; view tests for disabled action projections; regression tests for
  existing bookmark delete localness; `cargo check`; focused bookmark tests; full `cargo test` when
  practical.
- Docs/fragility updates: update bookmark metadata and tracking-state entries; update `progress.md`.
- Suggested agent/model routing: gpt-5.5 high exploration and implementation, because this is a
  semantic contract that gates future mutations; review can be gpt-5.5 high or maintainer-led.
- Review prompt: review Packet 36 for metadata truthfulness, row-order pairing risk, graceful
  degradation, preservation of existing local bookmark mutations, and whether tracking actions stay
  disabled until exact state is known.

### Packet 37: Bookmark Rename Flow

- Goal: add a safe bookmark rename flow for exact local bookmark rows.
- Owner concept: bookmark utility screen local rename action.
- Expected write set: `src/bookmarks.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`, focused tests,
  `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no bookmark forget, no track/untrack, no remote deletion, no bookmark advance, no push,
  and no generic ref editor.
- Acceptance criteria: rename is offered only for rows with one exact local bookmark name; the
  prompt collects one new exact bookmark name; empty, unchanged, or invalid names are rejected
  before execution; preview shows old name, new name, and command; confirmation is required;
  duplicate-name and command failures preserve full `jj` output; existing delete and future forget
  wording stay distinct from rename.
- Validation: command-construction tests for quoted old and new names; prompt, cancel, confirm,
  invalid-input, duplicate-name failure, and result-output tests; disposable `/tmp` jj repo proof
  for rename, duplicate-name failure, and undo/recovery where `jj` supports it, with mutations run
  from the proof repo's `cwd`; `cargo check`; focused bookmark tests; full `cargo test` and
  `just check` when practical.
- Docs/fragility updates: update bookmark exact-name and rename entries; update `progress.md` and
  workflow docs after shipping.
- Suggested agent/model routing: gpt-5.5 high implementation plus gpt-5.5 high review, because exact
  ref-name handling and prompt lifecycle are mutation-critical.
- Review prompt: review Packet 37 for local exact-name targeting, new-name validation, command
  construction, prompt lifecycle, output preservation, and isolated proof coverage.

### Packet 38: Bookmark Forget Flow

- Goal: add a safe bookmark forget flow after Packet 36 proves exact remote/tracking metadata.
- Owner concept: bookmark utility screen remote/tracking ref hygiene action.
- Expected write set: `src/bookmarks.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`, focused tests,
  `docs/plan/progress.md`, `docs/plan/fragility-register.md`, and
  `docs/plan/workflows/refs-and-workspaces.md`.
- Non-goals: no bookmark rename, no track/untrack, no local bookmark delete replacement, no bookmark
  advance, no push, and no generic ref editor.
- Acceptance criteria: forget is offered only where Packet 36 metadata proves the selected row is a
  forgettable remote or tracking-state target; local-only and ambiguous rows stay disabled; preview
  names the exact bookmark, remote/tracking state, and command; confirmation is required; failures
  preserve full `jj` output; existing local delete remains visually and textually distinct from
  forget.
- Validation: command-construction tests for exact remote/tracking target shapes; metadata-gated
  disabled-state tests; view-level preview, cancel, confirm, and result-output tests; disposable
  `/tmp` jj repo proof for forget where feasible, unsupported-state failure, and recovery where `jj`
  supports it, with mutations run from the proof repo's `cwd`; `cargo check`; focused bookmark
  tests; full `cargo test` and `just check` when practical.
- Docs/fragility updates: update bookmark forget/tracking-state entries; update `progress.md` and
  refs/workspaces workflow docs after shipping.
- Suggested agent/model routing: gpt-5.5 high implementation plus gpt-5.5 high review, because
  delete/forget wording and remote-state exactness are high-risk UX details.
- Review prompt: review Packet 38 for metadata-gated target exactness, delete-versus-forget clarity,
  command construction, disabled states, output preservation, and isolated proof coverage.

### Packet 39: Bookmark Track/Untrack Flows

- Goal: add bookmark track and untrack flows after Packet 36 proves exact tracking metadata.
- Owner concept: bookmark remote-tracking guided actions.
- Expected write set: `src/bookmarks.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`, focused tests,
  `docs/plan/progress.md`, `docs/plan/fragility-register.md`, and `docs/plan/workflows/sync.md`.
- Non-goals: no host dashboard, no branch protection or credential handling, no force-push, and no
  inference from rendered `name@remote` text.
- Acceptance criteria: track/untrack actions are enabled only for rows with exact metadata; preview
  names local bookmark, remote bookmark, remote, and exact command; confirmation is required for
  untrack; success refreshes bookmark rows and keeps output readable; unsupported remote states
  remain disabled with clear status.
- Validation: command-construction tests for track and untrack shapes; metadata-gated disabled-state
  tests; view-level preview, cancel, confirm, and result-output tests; disposable `/tmp` jj repo
  proof with a local disposable remote where feasible, with all mutations run from the proof repo's
  `cwd`; `cargo check`; focused bookmark/sync tests; full `cargo test` and `just check` when
  practical.
- Docs/fragility updates: update bookmark tracking-state and sync-flow entries; update
  `progress.md`.
- Suggested agent/model routing: gpt-5.5 high implementation and review; use exploration first if
  installed `jj` tracking semantics differ from the documented command shapes.
- Review prompt: review Packet 39 for exact tracking metadata, remote-state gating, command
  construction, result visibility, disposable remote proof, and whether host-specific behavior stays
  out.

### Packet 40: File Track/Untrack/Chmod Actions

- Goal: add exact-path file hygiene actions for `jj file track`, `jj file untrack`, and
  `jj file chmod`.
- Owner concept: file/status exact-path action planning.
- Expected write set: `src/file_list.rs`, `src/file_show.rs`, `src/status.rs`, `src/app.rs`,
  `src/jj.rs`, `src/tui.rs`, focused tests, `docs/plan/progress.md`, and
  `docs/plan/fragility-register.md`.
- Non-goals: no broad fileset editor, no recursive glob UI, no path inference from sticky headings
  alone, no conflict resolver, and no patch editor.
- Acceptance criteria: actions launch only from contexts that own exact repo-relative paths; preview
  shows each exact path and command; track/untrack/chmod command shapes are tested for paths with
  spaces and metacharacters; chmod mode choices are explicit and bounded to supported `jj` modes;
  result output stays scrollable; refresh preserves or clamps file selection safely.
- Validation: command-construction tests for track, untrack, executable/non-executable chmod, and
  unusual paths; parser/path-ownership tests for status and file contexts; view-level preview,
  cancel, confirm, and result-output tests; disposable `/tmp` jj repo proof for each file mutation
  and recovery where supported, with mutations run from the proof repo's `cwd`; `cargo check`;
  focused file/status tests; full `cargo test` and `just check` when practical.
- Docs/fragility updates: update exact-path and fileset/path quoting entries; update `progress.md`
  and file workflow docs after shipping.
- Suggested agent/model routing: gpt-5.5 high implementation plus review because path exactness is
  mutation-critical.
- Review prompt: review Packet 40 for exact path ownership, path quoting, chmod mode bounds,
  disabled ambiguous contexts, output preservation, and `/tmp` file-mutation proof.

### Packet 41: Workspace And Root Utility Surface

- Goal: add a focused read-first workspace/root surface for `jj root` and `jj workspace` context.
- Owner concept: workspace/root utility view.
- Expected write set: new `src/workspaces.rs` if warranted, `src/main.rs`, `src/command.rs`,
  `src/view_state.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`, focused tests,
  `docs/plan/screens/workspaces.md`, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`
  if output is parsed.
- Non-goals: no workspace add/rename/forget/update-stale mutations in the first packet, no worktree
  manager dashboard, and no replacement for shell navigation.
- Acceptance criteria: users can inspect repo root and workspace list/context; startup or global
  entry opens the view without mutating the repo; exact workspace names are separated from rendered
  labels if future actions will use them; copy exposes root and workspace names; missing or old `jj`
  workspace support degrades readably.
- Validation: command-construction tests for root/workspace read commands; parser tests if row
  structure is extracted; view navigation/search/copy/refresh tests; `cargo check`; focused
  workspace tests; full `cargo test` when practical; no `/tmp` mutation proof required unless the
  packet adds writes, but a disposable read proof is useful.
- Docs/fragility updates: add root/workspace output assumptions if parsed; update `progress.md` and
  screen docs.
- Suggested agent/model routing: gpt-5.5 high or gpt-5.4-mini implementation depending on parser
  complexity; gpt-5.5 high review if a new screen and parser are introduced.
- Review prompt: review Packet 41 for read-only scope, workspace/root command compatibility,
  graceful degradation, copy/search behavior, and whether mutation commands were kept out.

### Packet 42: Tag List Read Surface

- Goal: add a lower-priority read-only tag utility surface with exact tag list rows.
- Owner concept: refs utility view for tags.
- Expected write set: new `src/tags.rs` if warranted, `src/main.rs`, `src/command.rs`,
  `src/view_state.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`, focused tests,
  `docs/plan/screens/tags.md`, `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no tag set, no tag delete, no host release workflow, no signed-tag management, no
  push/tag sync policy, and no broad ref dashboard.
- Acceptance criteria: tag list is useful as a read surface before mutations are considered; exact
  tag names and target ids are carried separately from rendered labels; search, copy, refresh, and
  back behavior match other utility screens; ambiguous remote or signed-tag behavior stays
  CLI-first; tag set/delete remain deferred to a future separate packet or parking-lot review.
- Validation: list command and parser tests; view navigation/search/copy/refresh tests;
  `cargo check`; focused tag tests; full `cargo test` and `just check` when practical; no `/tmp`
  mutation proof is required because this packet is read-only.
- Docs/fragility updates: add tag output and exact-name assumptions if parsed; update `progress.md`
  and screen docs.
- Suggested agent/model routing: gpt-5.4-mini implementation is acceptable if the parser is narrow;
  use gpt-5.5 high review if exact tag identity depends on a new metadata contract.
- Review prompt: review Packet 42 for read-only scope, exact tag identity, rendered-output
  preservation, utility-screen behavior, and whether tag mutations stayed out of the packet.

### Packet 43: File Search Utility

- Goal: add a focused file search/read surface when it can improve navigation without replacing
  shell tools.
- Owner concept: file utility search and navigation.
- Expected write set: `src/file_list.rs`, `src/file_show.rs` or a new focused module if needed,
  `src/command.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`, focused tests, `docs/plan/progress.md`,
  and `docs/plan/fragility-register.md` if output is parsed.
- Non-goals: no full grep replacement, no indexer, no mutation, no fuzzy file manager, and no search
  over unbounded history unless scoped deliberately.
- Acceptance criteria: the search scope is explicit; result rows preserve exact paths and useful
  rendered context; opening a result uses exact path/revision data where known; empty and huge
  result sets stay readable; copy and refresh behavior match other file utility screens.
- Validation: command-construction tests for the chosen search command shape; parser tests for
  paths, line numbers, empty output, and unusual characters if parsed; view navigation/search/copy
  tests; `cargo check`; focused file tests; full `cargo test` when practical.
- Docs/fragility updates: record search-output assumptions if any; update `progress.md`.
- Suggested agent/model routing: gpt-5.5 high exploration to choose scope, then gpt-5.4-mini or
  gpt-5.5 high implementation based on parser complexity.
- Review prompt: review Packet 43 for bounded search scope, exact path handling, large-output
  behavior, navigation correctness, and whether it avoids becoming a general grep UI.

### Packet 44: File Annotate Read Surface

- Goal: add a read-only `jj file annotate` surface for exact file paths.
- Owner concept: file provenance detail view.
- Expected write set: new `src/file_annotate.rs` if warranted, `src/file_list.rs`,
  `src/file_show.rs`, `src/command.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`, focused tests,
  `docs/plan/progress.md`, and `docs/plan/fragility-register.md`.
- Non-goals: no mutation, no blame analytics dashboard, no cross-file provenance browser, and no
  parsing of annotation semantics beyond navigation/copy needs.
- Acceptance criteria: annotate opens only when an exact file path is available; rendered `jj`
  output and styles are preserved; search, scroll, copy, and back behavior match document views;
  copied ids prefer explicit semantic fields only if a narrow contract proves them, otherwise copy
  rendered text; missing paths or unsupported output degrade readably.
- Validation: command-construction tests for exact file paths and revision context if supported;
  parser tests only for the narrow fields used by navigation/copy; view-level
  scroll/search/copy/back tests; `cargo check`; focused file annotate tests; full `cargo test` when
  practical.
- Docs/fragility updates: record annotate output assumptions if parsed; update `progress.md`.
- Suggested agent/model routing: gpt-5.4-mini implementation is acceptable for a rendered-document
  first pass; use gpt-5.5 high review if semantic id extraction is added.
- Review prompt: review Packet 44 for read-only scope, exact path command construction, rendered
  output preservation, navigation/copy behavior, and minimal parsing.

### Packet 45: Evolog Utility Screen

- Goal: add a later read-only `jj evolog` utility screen for inspecting change evolution from an
  exact graph row.
- Owner concept: change-history detail view.
- Expected write set: new `src/evolog.rs` if warranted, `src/graph.rs`, `src/command.rs`,
  `src/app.rs`, `src/jj.rs`, `src/tui.rs`, focused tests, `docs/plan/progress.md`, and
  `docs/plan/fragility-register.md`.
- Non-goals: no mutation from evolog, no operation recovery replacement, no semantic reconstruction
  of evolution edges, and no whole-repo evolog dashboard.
- Acceptance criteria: evolog opens from an exact selected change; rendered output is preserved;
  search, scroll, copy, refresh, and back behavior follow existing document/detail views; missing
  exact targets disable the action; docs explain that operation log remains the recovery anchor.
- Validation: command-construction tests; view-level open/back/refresh/search/copy tests; parser
  tests only if semantic fields are extracted; `cargo check`; focused evolog tests; full
  `cargo test` when practical.
- Docs/fragility updates: add evolog rendered-output assumptions if parsed; update `progress.md`.
- Suggested agent/model routing: gpt-5.4-mini implementation with gpt-5.5 high review unless parser
  or cross-view navigation complexity expands.
- Review prompt: review Packet 45 for exact change targeting, read-only boundary, rendered-output
  preservation, operation-log distinction, and bounded parser assumptions.

### Packet 46: Low-Value Command Parking-Lot Review

- Goal: keep low-value commands deliberately passthrough or deferred after the promoted packets are
  scheduled.
- Owner concept: command inventory and workflow policy documentation.
- Expected write set: `docs/plan/command-inventory.md`, `docs/plan/workflows.md`, relevant
  `docs/plan/workflows/*.md`, `docs/plan/progress.md`, and `docs/process-observations.md`.
- Non-goals: no code changes, no command implementation, no command mode, and no promotion of a
  command just because `jj` supports it.
- Acceptance criteria: passthrough/deferred rationale is explicit for low-frequency or poor-fit
  commands such as `interdiff`, `metaedit`, `parallelize`, `simplify-parents`, `sparse`, `fix`,
  `config`, `sign`, `unsign`, `operation integrate`, `bookmark advance`, `diffedit`, `arrange`,
  `bisect`, `gerrit`, and `util`; dangerous commands have stronger rationale than "later"; planned
  packets above remain the only near-term promoted work.
- Validation: `just md-check`; manual consistency check against `command-inventory.md`,
  `workflows.md`, workflow-specific docs, and `progress.md`; no Rust validation required.
- Docs/fragility updates: update `progress.md`; leave `fragility-register.md` unchanged unless the
  review introduces a new planned soft contract.
- Suggested agent/model routing: gpt-5.5 high documentation/review worker because this is product
  prioritization, not prose cleanup.
- Review prompt: review Packet 46 for classification consistency, explicit passthrough/deferred
  rationale, no overclaiming, and whether the parking lot avoids turning `jk` into a full CLI clone.

### Packet 32 Scheduling Notes

- Immediate next recommended packet: Pre-Packet-34 Interruption Packet A, because Packet 33 shipped
  operation-log recovery for exact operation ids and the user requested a maintainability/UI
  interruption before Split Guided Flow. Packet 34 is postponed until the interruption wave either
  lands or is explicitly reprioritized.
- Packets 34 and 35 should stay separate. `split` has editor/process uncertainty, while `duplicate`
  is a graph rewrite with different target and refresh behavior.
- The interruption packets should not be treated as command-coverage parity work. Their purpose is
  maintainability, screen-level contracts, keyboard/UI coherence, and validation discipline before
  more rewrite features are added.
- Packets 36, 38, and 39 must not be collapsed. Forget and tracking mutations need explicit metadata
  first; rendered bookmark labels are not a mutation-grade contract.
- Packet 37 may run after the existing local exact bookmark-name contract is confirmed. It should
  stay separate from Packet 38 because rename targets a local old name and new name, while forget
  targets metadata-gated remote/tracking state.
- Packets 38 and 39 may run only after Packet 36 lands, and they should not be implemented in
  parallel with other bookmark action work because they share metadata, action routing, and
  `src/bookmarks.rs`.
- Packet 40 should wait until an implementor re-reads the exact path contracts from file list, file
  show, status, and resolve. It is one file-hygiene packet because the commands share path ownership
  and result refresh behavior.
- Packets 41 through 45 are read-surface or utility packets. They are valuable, but lower priority
  than recovery and high-frequency mutation gaps unless user work shows a stronger need.
- Packet 46 is a policy/doc packet after the next implementation wave or whenever classifications
  drift. It exists to prevent parity pressure from silently becoming product direction.

## Documentation And Demo Media Packets

The documentation/demo work is not a polish afterthought. It is the way users will understand which
`jj` flows are safe, direct, previewed, or deliberately left to the CLI.

- Packet 20 should make the README accurate and useful before public-facing media is captured.
- Packet 21 should add reusable capture specs and deterministic demo setup, with generated media
  under ignored `target/vhs` or hosted externally.
- Packet 29 should turn those specs into tutorial coverage for the shipped daily loop.

Media review should follow Ratatui guidance:

- Use a dark theme, high contrast, and clear hierarchy.
- Keep captures at or below 1200 px wide and verify legibility when resized or phone-sized.
- Hide shell setup noise and show the app quickly.
- Prefer signal over feature coverage; a single clear workflow is better than a crowded tour.
- Use calm motion and enough dwell time for viewers to read each state.
- Do not commit generated GIFs/screenshots to normal repository history.

## Day-To-Day Workflow Coverage Map

  | Workflow area            | Current/next packet coverage                        | Still missing or deferred                                      |
  | ------------------------ | --------------------------------------------------- | -------------------------------------------------------------- |
  | Inspect log/show/diff    | Completed Slice 0-3 and existing show/diff behavior | README/tutorial refresh in Packets 20 and 29                   |
  | Status and fetch         | Completed Slice 4 and 6, Packet 14                  | status docs/media after decluttering                           |
  | Action output visibility | Packet 13                                           | persistent action history remains out of scope                 |
  | Abandon change flow      | Packet 15                                           | multi-select and advanced/bulk abandon flows                   |
  | Operation recovery       | Completed Slice 7, Packets 16, 17, and 33           | operation integrate                                            |
  | Create new work          | Completed Slice 5, Packet 18                        | describe-on-create and richer revset prompt                    |
  | Push/sync                | Completed Slice 11, Packet 19                       | host-specific flows, force pushes, deeper tracking UI          |
  | README and release docs  | Packets 20, 21, and 29                              | published hosted media after captures are reviewed             |
  | Squash/rebase rewrites   | Completed Slice 12, Packets 22 and 26               | split, duplicate, parallelize, simplify-parents                |
  | Describe/commit          | Packet 23                                           | editor integration and advanced metadata editing               |
  | Bookmarks                | Completed Slice 8, Packet 24                        | tags and advanced remote-tracking semantics                    |
  | Absorb                   | Packet 25                                           | patch-level explanations beyond CLI output                     |
  | Restore/revert           | Packets 27 and 33                                   | patch editor                                                   |
  | Resolve conflicts        | Packet 28                                           | full merge editor                                              |
  | Working-copy navigation  | Packet 30                                           | richer stack-aware movement policy                             |
  | Strong command coverage  | Packets 31 and 32                                   | low-frequency passthrough commands remain intentionally scoped |
  | Maintainability/UI       | Pre-Packet-34 interruption wave                     | Packet 34 waits behind app decomposition and UI repair         |

## Follow-Up Planning Backlog

This backlog captures likely follow-up waves after Packets 13-32. These are not implementation
packets yet. Promote an item into a packet only after the owner, target semantics, confirmation
level, tests, and proof path can be written without hiding uncertainty.

For every write operation below, promotion requires disposable-repo proof under `/tmp`, with the
write command run from that repository's `cwd`. The proof should cover the normal path, cancel or
disabled behavior when relevant, failure output visibility, and recovery through `jj undo` or the
operation-log flow when `jj` supports that recovery shape.

### Mutation Safety And Promotion Rules

- Prerequisites: Packet 13 output visibility, Packet 14 status/help clarity, exact target identity
  carried through the launching screen, and a documented safety tier for direct, low-friction,
  previewed, confirmed, or passthrough-only execution.
- Promotion evidence: command-construction tests, view/action tests for preview and result
  presentation, a fragility-register entry for every parsed or inferred contract, and an isolated
  `/tmp` write proof for mutations.
- Packet shape: one command family per packet unless two commands share one exact target contract
  and one user workflow. Avoid grab-bag command parity packets.

### Abandon Follow-Ups

- Generalized abandon should grow only after Packet 15 proves exact-target abandon from one context.
  Follow-ups can add bulk abandon, multi-select graph abandon, range/revset abandon, and
  abandon-from-detail views.
- Prerequisites: stable multi-select ordering, visible source context for every target, and a
  stronger confirmation rule for any non-empty target or bulk operation.
- Promotion evidence: tests proving ambiguous rows stay disabled, preview text enumerates every
  exact target, and `/tmp` proofs cover empty, non-empty, multiple-target, failure, and undo paths.

### Operation Recovery Follow-Ups

- Packet 33 implemented `operation restore` and `operation revert` from the operation-log context.
  Consider `operation integrate` later. Keep `operation abandon` deferred unless there is a strong
  product reason and a much stricter safety contract.
- Prerequisites: operation detail/diff views, exact operation ids, result output that remains
  inspectable, and clear language distinguishing restore, revert, undo, redo, and integrate.
- Promotion evidence: exploration against `jj operation --help`, command-construction tests,
  operation-log disabled-state tests, and `/tmp` repos proving restore/revert semantics and
  recovery.

### Rewrite Expansion Follow-Ups

- Candidate commands: `split`, `duplicate`, `parallelize`, `simplify-parents`, `unsquash`, and
  deeper `absorb`/`squash` variants. Treat `diffedit` or external patch editing as a later spike
  unless the current command-mode policy and editor lifecycle are settled.
- Prerequisites: source/destination role prompts from Packet 22, rebase-preview lessons from Packet
  26, exact selected revision identity, and a decision on whether the flow can rely on `jj` preview
  output or needs structured command planning.
- Promotion evidence: a small proof matrix for each command's actual `jj` semantics, tests showing
  role clarity and cancel behavior, fragility entries for any graph-effect claims, and `/tmp` proof
  with undo for each promoted rewrite.

### Richer `jj new` Follow-Ups

- Build on Packet 18 with description-on-create, insert-before/after, revset or custom-parent
  prompts, workspace-aware `new`, and a clearer distinction between direct `new trunk` and guided
  parent selection.
- Prerequisites: text-input lifecycle from Packet 23, exact parent identities from graph selection,
  workspace/root context from the utility screens, and a policy for low-friction creation versus
  previewed custom parent shapes.
- Promotion evidence: command-construction tests for every supported shape, tests for cancelled or
  empty descriptions, `/tmp` proofs for single-parent, multi-parent, insert-before/after, and
  workspace-aware creation, and refresh tests proving the new working copy is visible.

### Refs, Tags, Workspaces, And Root Follow-Ups

- Extend beyond Packet 24 with bookmark rename/forget/advance if justified, tag set/delete only
  after a separate read-only tag list packet proves exact identity, workspace
  list/add/rename/forget/update-stale, and a small root/workspace-info surface.
- Prerequisites: bookmark tracking-state contract, exact names separated from rendered labels,
  workspace root detection, and a decision on which low-frequency commands remain passthrough.
- Promotion evidence: parser or structured-data tests for local/remote/tracking rows, command tests
  for quoted names and exact targets, `/tmp` write proofs for ref and workspace mutations, and docs
  that keep host-specific handoff out unless the scope is explicit.

### File Workflow Follow-Ups

- Extend file workflows beyond Packet 27 with `file track`, `file untrack`, `file annotate`
  provenance, `file chmod`, and polish for `file list` and `file show` navigation, search, sticky
  context, and copy.
- Prerequisites: exact path contracts from file list/show, status file-group ownership, conflict
  path handling from Packet 28, and a fileset/path quoting policy that does not depend on rendered
  labels.
- Promotion evidence: parser tests for unusual paths, command-construction tests for track/untrack
  and chmod, view tests for file list/show refresh and sticky context, and `/tmp` proofs for file
  mutations from the temporary repo's `cwd`.

### Sync Follow-Ups

- Build on Packet 19 with remote selection for fetch, selected-bookmark push, selected-change push,
  track/untrack remote bookmarks, dry-run or preview policy, and host handoff only when scoped to a
  narrow action such as opening an already-known compare URL.
- Prerequisites: exact remote/bookmark metadata, a visible default-target policy, scrollable output,
  credential/error output preservation, and a product decision that `jk` is not becoming a remote
  dashboard.
- Promotion evidence: command-construction tests for remote, bookmark, revision, tracking, dry-run,
  and default shapes; remote-less failure tests; `/tmp` or disposable remote proofs for feasible
  write paths; and docs explaining what remains CLI passthrough.

### Conflict And Resolve Follow-Ups

- After Packet 28, consider conflict detail views, launch-to-external-resolver actions, mark
  resolved actions when exact paths are known, refresh-preserving conflict navigation, and conflict
  tutorial/demo coverage.
- Prerequisites: conflict state contract, exact conflicted paths, editor/process lifecycle policy,
  and a safe fallback when `jj` output does not expose enough state.
- Promotion evidence: conflicted `/tmp` repo fixtures, parser or structured-state tests, command
  tests for launched resolution actions, and a fragility-register update if conflict semantics come
  from rendered output.

### Command Mode And Passthrough Policy

- Packet 31 should decide command-mode safety tiers before long-tail promotion: read-only
  passthrough, write passthrough with preview, blocked-dangerous commands, and native-only guided
  flows for commands where raw passthrough would hide too much risk.
- Prerequisites: current command inventory, help/keymap clarity, action output overlay, and
  agreement on whether command mode is a launcher, fallback, or teaching surface.
- Promotion evidence: documented tiers, tests for blocked or confirmation-required command shapes,
  and proof that passthrough output remains readable without turning `jk` into a full CLI clone.

### Evaluation, Tutorial, Demo, And Media Follow-Ups

- After Packets 20, 21, and 29, add day-to-day tutorial expansions, deterministic eval scripts, demo
  repository refresh checks, hosted media review, and regression fixtures for capture flows.
- Prerequisites: shipped behavior only, ignored generated assets under `target/`, stable demo repo
  setup, and Ratatui media constraints for contrast, width, dwell time, and setup-noise hiding.
- Promotion evidence: `just md-check`, successful demo setup runs, optional VHS execution when
  installed, and a manual truthfulness pass against `progress.md` and the command inventory.

### Integration Contract Follow-Ups

- Track candidates for structured output, purpose-built templates, `jj_cli`, `jj_lib`, future RPC,
  or upstream extraction whenever a packet needs semantic state that `jj` already knows before
  rendering.
- Prerequisites: fragility-register evidence, a failing or awkward parser contract, and a small
  spike proving the stronger contract can preserve both semantic identity and user-configured view
  fidelity.
- Promotion evidence: before/after failure-mode notes, focused parser or schema tests, a written
  migration path for closing fragility-register entries, and a decision on whether the dependency is
  production-ready or only spike evidence.

### Performance And Large-Repo Follow-Ups

- Plan large-repo checks for log rendering, search, refresh, multi-select reconciliation, operation
  log loading, file list/show, and action-output scrolling.
- Prerequisites: representative large disposable repos or captured fixtures, explicit terminal size
  inputs, and instrumentation that does not change normal terminal behavior.
- Promotion evidence: benchmark or timed manual runs with stated repo size and terminal dimensions,
  tests for empty/huge outputs and narrow terminals, and documented decisions on paging, truncation,
  streaming, or lazy loading.

### Accessibility And Terminal Compatibility Follow-Ups

- Extend Packet 14 and Packet 13 with checks for narrow terminals, no-color or low-color
  environments, high-contrast themes, keyboard-only operation, copyable exact ids, readable modal
  labels, and terminal resize behavior.
- Prerequisites: stable status/help/output surfaces, a small compatibility matrix, and decisions on
  how `NO_COLOR`, ANSI style loss, and alternate terminal capabilities should degrade.
- Promotion evidence: rendering snapshots for narrow and normal widths, manual terminal checks where
  automation is insufficient, and docs that state compatibility limits without overpromising.
