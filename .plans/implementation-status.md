# Implementation Status Tracker

## Legend

- `not-started`: no implementation work yet.
- `in-progress`: currently being built/refined.
- `blocked`: waiting on decision or dependency.
- `done`: implemented and validated by tests.

## Current snapshot (2026-02-07)

### Documentation and planning

- Master plan (`.plans/jk-tui-command-plan.md`): `done`
- Research context (`.plans/research-context.md`): `done`
- Per-command deep plan (`.plans/command-deep-plans.md`): `done`
- Gold command detail matrix (`.plans/gold-command-details.md`): `done`
- ADR scaffolding: `done`

### Foundation implementation

- CLI entrypoint (`jk` == `jk log`): `done`
- Alt-screen + raw mode runtime loop: `done`
- Keybinding system + TOML defaults: `done`
- Command registry and alias normalization: `done` (includes OMZ alias baseline + 44-command registry)
- Log view rendering/parsing bridge: `done` (metadata-backed row mapping + fallback parsing)

### Core command flows (Phase 1 target)

- `log`, `status`, `show`, `diff`: `done` (log-first baseline + selection-aware show/diff)
- `new`, `describe`, `commit`, `next`, `prev`, `edit`: `done` (selection-aware planning wired)
- `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`: `done` (guided flows + danger confirms)
- `bookmark` core subset: `done` (list/create/set/move/track/untrack prompts wired)
- `git fetch`, `git push`: `done` (guided prompts + alias coverage + push confirmation)

### Extended guided flows (Phase 2 seed)

- `restore`, `revert`: `done` (guided prompt defaults wired)
- `bookmark rename`, `bookmark delete`, `bookmark forget`: `done` (guided prompts wired)
- `operation` / `workspace` defaults: `done` (`operation log` and `workspace list`)
- `operation show`, `operation diff`, `operation restore`, `operation revert`: `done` (direct read
  flows for show/diff, guided prompts for restore/revert)
- `workspace add`, `workspace forget`, `workspace rename`: `done` (guided prompts)

### Testing baseline

- Unit test harness for parsing/alias normalization: `done`
- `insta` visual snapshots for main screens: `done`
- Command assembly tests (`FlowAction`/`PromptKind`): `done`
- Safety-guard tests for Tier C commands: `done`

## Implementation checklist

1. Build runtime loop and rendering pipeline.
2. Add keybinding config loading and default mapping.
3. Implement read-only command flows.
4. Implement mutation flows with preview/confirm safety layers.
5. Add alias compatibility (`gf`/`gp`/`rbm`/`rbt` + OMZ variants).
6. Add tests and snapshots.
7. Validate with `cargo fmt`, `cargo test`, and markdown lint.

## Notes

- Keep this file updated after each meaningful implementation step.
- Plan-to-implementation handoff rule:
  - when work starts from a Plan Mode proposal, write the handoff entry here before any
    implementation edits.
- Every status transition should include tests or clear rationale in commit/body text.
- Current handoff for module refactor:
  - source plan: command-first module architecture with soft 300/hard 500 LOC and co-located tests.
  - full durable execution plan written to `.plans/module-refactor-execution-plan.md`.
  - refactor execution is now active and will proceed in atomic jj commits by subsystem.
- Latest checkpoint:
  - expanded OMZ alias support and `rbm` default-to-`main` behavior
  - added guided flows for rewrite/recovery, bookmark, and remote commands in `src/flows.rs`
  - retained confirmation gating in `src/app.rs` for Tier C commands
  - added explicit test coverage for dangerous-command gating and alias argument fidelity
  - added explicit top-level `jj` command registry and safety-tier lookup in `src/commands.rs`
  - expanded guided prompts for `restore`, `revert`, and additional bookmark mutations
  - improved revision selection with metadata-backed row mapping for `jj log` output
  - added in-app `:commands` registry view from command metadata
  - routed startup `jk <command>` through the same flow planner used by command mode
  - added startup regression tests for confirm-gated and render-only startup actions
  - added confirmation preview rendering with `git push --dry-run` support
  - added filtered command registry lookup via `:commands <query>` and `:help <query>`
  - added guided operation subcommand prompts and operation/workspace safety overrides
  - added guided workspace prompts with direct handling for `workspace root` and `update-stale`
  - added confirmation previews for `operation restore` and `operation revert`
  - aligned command registry modes so `operation`/`workspace`/`restore`/`revert` report guided
    coverage
  - expanded alias and flow tests for OMZ high-frequency variants (`jjgfa`, `jjgpt`, `jjgpa`,
    `jjgpd`, `jjst`, `jjl`)
  - documented command-registry parity and safety routing in `docs/adr/0003-command-registry-parity.md`
  - broadened Tier C confirmation previews to cover rebase/squash/split/abandon/restore/revert,
    bookmark mutations, and undo/redo operation context
  - added Tier C fallback preview to operation log when a command has no custom preview renderer
  - improved `rbm`/`rbt` alias normalization to support optional destination overrides without
    duplicate destination args
  - added default normal-mode shortcuts for high-frequency remote/rebase flows (`F`, `P`, `M`,
    `T`)
  - added default normal-mode action shortcuts (`n`, `c`, `D`, `b`, `a`) for new/commit/describe,
    bookmark set, and abandon flows
  - added log patch toggle shortcut (`p`) with argument-preserving `log` patch on/off behavior
  - added normal-mode rewrite/recovery shortcuts (`B`, `S`, `X`, `O`, `R`, `u`, `U`) that route
    through existing prompt/confirm safety flows
  - added `s` status shortcut and sectioned status rendering for improved in-app scanability
  - surfaced high-frequency alias hints directly in unfiltered `:commands`/`:help` output
  - switched `:` command parsing to shell-style tokenization with invalid-quote status feedback
  - added native header/shortcut wrappers for `show` and `diff` outputs to improve scanability
  - added normal-mode `?` shortcut that opens the in-app command registry/help view
  - added command-mode history navigation (`Up`/`Down`) with draft restoration
  - added normal-mode `.` shortcut to re-run the last executed command
  - added native wrapper view for `root` output with workspace-path-focused presentation
  - added native wrapper views for `bookmark list` and `operation log` outputs
  - added normal-mode `]`/`[`/`e` shortcuts for `next`/`prev`/`edit` revision actions
  - added explicit gold-set flow contract test covering all core commands in one matrix
  - added explicit OMZ gold-alias flow contract test matrix for high-frequency compatibility
  - added `insta` snapshot coverage for bookmark-list and operation-log wrapper views
  - added quick read-mode shortcuts (`o`, `L`, `w`) for operation log/bookmark list/root views
  - aligned `workspace root` rendering with the native root wrapper presentation
  - added in-app `:aliases` catalog view with optional query filtering
  - improved `show`/`diff` wrapper rendering with automatic section spacing between file blocks
  - added in-app `:keys` keymap view with query filtering and startup support (`jk keys`)
  - added normal-mode `K` shortcut for direct keymap access
  - added `:commands` tips that point to `:aliases` and `:keys` discovery views
  - added normal-mode `A` shortcut for direct alias-catalog access
  - refined `status`/`operation log` wrappers with compact summaries and section-spacing rules
  - added `insta` snapshot coverage for the updated status wrapper presentation
  - surfaced local discovery views (`aliases`, `keys`, `keymap`) in the `:commands` catalog
  - added native wrapper views for `workspace list` and `operation show` output
  - added `insta` snapshot coverage for workspace-list and operation-show wrapper presentation
  - added native wrapper views for `file list` and `tag list` output
  - added `insta` snapshot coverage for file-list and tag-list wrapper presentation
  - defaulted `file` and `tag` command groups to `file list` and `tag list` flows
  - defaulted `resolve` command group to `resolve -l` for list-first conflict inspection
  - switched `operation show` and `operation diff` to direct read execution without prompt
  - added `tag` short-subcommand canonicalization (`l`/`s`/`d`) in flow planning
  - refined command safety mapping for read-only `file`/`tag` subcommands
  - refined command safety mapping for read-only `resolve -l`
  - surfaced command-group default hints in `:commands` unfiltered tips
  - added normal-mode quick-read shortcut for `resolve -l` (`v`)
  - added normal-mode quick-read shortcuts for `file list` (`f`) and `tag list` (`t`)
  - added native wrapper view and snapshot coverage for `resolve -l` output
  - added native wrapper view and snapshot coverage for `operation diff` output
  - added native wrapper views and coverage tests for `git fetch` and `git push` output
  - added `insta` snapshot coverage for `git fetch` and `git push` wrapper views
  - added native wrapper views and coverage for `file show`, `file search`, and `file annotate`
    output
  - added `insta` snapshot coverage for `file show`, `file search`, and `file annotate` wrappers
  - added guided prompt flows for `file track`, `file untrack`, and `file chmod`
  - added native wrapper views and tests for `file track`, `file untrack`, and `file chmod`
    output
  - added `insta` snapshot coverage for `file track`, `file untrack`, and `file chmod` wrappers
  - added guided prompt flows for `tag set` and `tag delete` with selected-revision defaults
    for tag set
  - aligned command-registry metadata so `file` and `tag` now report `guided` mode
  - added native wrapper views for `bookmark` mutation flows
    (`create/set/move/track/untrack/delete/forget/rename`)
  - added native wrapper views for `workspace` mutation flows
    (`add/forget/rename/update-stale`)
  - added native wrapper views for `operation restore` and `operation revert`
  - added native wrapper views for top-level mutation commands:
    `new`, `describe`, `commit`, `edit`, `next`, `prev`, `rebase`, `squash`, `split`, `abandon`,
    `undo`, `redo`, `restore`, `revert`
  - added targeted decorator/render tests and snapshots for
    `bookmark set`, `workspace add`, and `operation restore`
  - added targeted decorator/render tests and snapshots for
    top-level `commit` and `rebase` wrapper rendering
  - refined `rbm`/`rbt` alias normalization to preserve explicit destination flags (`-d`/`--to`)
    without forcing default destinations
  - expanded alias-catalog parity with the installed OMZ `jj` plugin aliases (`jjbc`, `jjbd`,
    `jjbf`, `jjbr`, `jjcmsg`, `jjdmsg`, `jjgcl`, `jjla`) and added direct catalog coverage tests
  - reran full validation checkpoint with green `markdownlint-cli2`, `cargo fmt`, `cargo check`,
    `cargo test` (199 passed), and strict `cargo clippy`
- Workflow order for each change:
  1. write/update docs first when design context changes;
  2. lint Markdown immediately;
  3. write code;
  4. run `cargo fmt` + `cargo check`;
  5. run targeted tests for changed behavior.
- At each larger checkpoint, run full `cargo test` and
  `cargo clippy --all-targets --all-features -- -D warnings`.
- Latest pass:
  - added native wrapper views for high-frequency mutation outputs:
    `bookmark` mutations, `workspace` mutations, and `operation restore/revert`.
  - added targeted decorator/render tests and `insta` snapshots for:
    `bookmark set`, `workspace add`, and `operation restore` wrapper rendering.
- Latest pass:
  - added native wrappers for top-level daily/rewrite mutation commands so command results stay in
    the same scannable in-app format as the rest of the gold command set.
- Latest pass:
  - added broad regression tests ensuring every top-level mutation command routes through a native
    wrapper.
  - added command-specific tip assertions for `commit`, `undo`, `rebase`, and `next` wrapper
    output.
- Latest pass:
  - refined top-level mutation wrapper summaries to prefer signal-first action lines (for example
    rebase/undo/restore results) before falling back to generic output-line counts.
  - updated top-level `commit` and `rebase` snapshots to lock in signal-first summary rendering.
- Latest pass:
  - added a gold-command wrapper matrix regression test over
    `status/show/diff`, top-level mutation commands, bookmark core commands, and `git fetch/push`
    wrapper headers.
  - updated the gold command detail contract to require this wrapper matrix coverage.
- Latest pass:
  - added snapshot coverage for remaining mutation wrapper variants:
    bookmark `create/move/track/untrack`, workspace `forget/rename/update-stale`, and operation
    `revert`.
- Latest pass:
  - added snapshot coverage for the remaining bookmark mutation variants:
    `bookmark delete`, `bookmark forget`, and `bookmark rename`.
- Latest pass:
  - added snapshot coverage for additional top-level mutation wrappers:
    `new`, `undo`, `abandon`, and `restore`.
- Latest pass:
  - added snapshot coverage for the remaining top-level mutation wrappers:
    `describe`, `edit`, `next`, `prev`, `squash`, `split`, `redo`, and `revert`.
- Latest pass:
  - added mutation wrapper routing matrix tests for bookmark and workspace mutation subcommands.
  - added explicit `operation revert` wrapper routing coverage in decorator tests.
- Latest pass:
  - reran full validation checkpoint with green `markdownlint-cli2`, `cargo fmt`, `cargo check`,
    `cargo test` (199 passed), and strict `cargo clippy`.
- Latest pass:
  - aligned alias normalization with core `jj` defaults so `b`, `ci`, and `op` canonicalize to
    `bookmark`, `commit`, and `operation` for consistent in-app flow routing.
- Latest pass:
  - added core `jj` default aliases (`b`, `ci`, `desc`, `op`, `st`) to the in-app alias catalog.
  - added focused flow-planner and alias-catalog tests for core `jj` default alias behavior.
- Latest pass:
  - added top-level default-alias annotation in `:commands` output
    (`bookmark (b)`, `commit (ci)`, `describe (desc)`, `operation (op)`, `status (st)`).
  - added command-registry tests for default-alias rendering and alias-based filtering.
- Latest pass:
  - added startup-path regression coverage for core `jj` default aliases
    (`jk st`, `jk ci`, `jk desc`, `jk b`, `jk op`) so startup routing stays aligned with command
    mode behavior.
- Latest pass:
  - refined bookmark/workspace/operation mutation wrappers to prefer signal-first summary lines
    (for example `Moved bookmark ...`, `Created workspace ...`, `Restored/Reverted operation ...`)
    before falling back to output-line counts.
  - added focused summary-heuristic unit tests and refreshed affected `insta` snapshots for
    bookmark/workspace/operation mutation wrapper views.
- Latest pass:
  - refined `git push`/`git fetch` wrappers to prefer signal-first summary lines when command
    output includes clear action lines, while preserving no-change and fallback count summaries.
  - added focused `git fetch`/`git push` summary-heuristic tests and refreshed the `git push`
    wrapper snapshot.
- Latest pass:
  - added a native wrapper for `version` command output with concise summary/tip presentation to
    keep low-risk utility flows consistent with the in-app design language.
  - added targeted render/decorator tests plus an `insta` snapshot for the version wrapper view.
- Latest pass:
  - added startup-action regression coverage for high-frequency OMZ aliases
    (`jk jjst`, `jk jjl`, `jk jjd`, `jk jjgf`, `jk jjgp`, `jk jjrbm`, `jk jjc`, `jk jjds`).
- Latest pass:
  - lifted `absorb`, `duplicate`, and `parallelize` from deferred behavior to selection-aware
    guided flows in the planner (`absorb --from <selected>`, `duplicate <selected>`,
    `parallelize <selected>`).
  - aligned command-registry execution modes so all three now report `guided` coverage.
- Latest pass:
  - grouped in-app help and keymap output into intent-first sections
    (`Navigation`, `Views`, `Actions`, `Safety`) while preserving compact two-column density.
  - removed legacy ASCII underline heading rows from wrapper-rendered views for cleaner text-only
    presentation.
  - simplified the header to `jk [MODE]` with a muted mode badge and removed duplicated
    command-context noise.
  - added a concrete "First 5 Minutes in `jk`" walkthrough to `README.md`.
  - updated `docs/tutorial-vhs.md` to embed dynamic GIFs directly with narrative descriptions.
  - refreshed all affected `insta` snapshots and reran full validation checkpoint:
    `cargo test`, strict `cargo clippy`, and `markdownlint-cli2`.
- Latest pass:
  - implemented directional UX improvements from
    `.plans/improvements/2026-02-08-015556-directional-ux-docs-summary.md`.
  - added normal-mode footer signals for onboarding progress, primary next action, log quick
    actions, and back/forward history context.
  - added workflow-scoped help presets (`:help inspect|rewrite|sync|recover`) with in-app
    narratives and recent-intent context.
  - added command-mode ranked suggestions using session-local frequency + recency scoring.
  - added confirm-mode dry-run key (`d`) for dangerous commands when preview strategies exist.
  - added `docs/workflows.md` with inspect/rewrite/sync/recover narratives linked to screenshots
    and GIFs.
  - added workflow-focused snapshot coverage for app and command-workflow help rendering.
- Latest pass:
  - completed a greenfield documentation coverage pass for `README.md` and non-test Rust modules.
  - added module docs and item-level contract docs across `src/` with full non-test coverage.
  - recorded remediation backlog and coverage ledger in
    `.plans/docs-greenfield-coverage-2026-02-08.md`.
- Latest pass:
  - completed follow-up docs work in the same change: added `docs/contributing-tests.md`, added
    `docs/glossary.md`, linked both from `README.md`, and linked glossary terminology from command
    docs.
  - added compact rustdoc examples for planner and builder entrypoints in
    `src/flow/planner.rs` and `src/flow/builders.rs`.
  - added native top-level mutation wrappers plus summary-signal coverage and `insta` snapshots
    for `absorb`, `duplicate`, and `parallelize`.
- Latest pass:
  - reran full validation checkpoint with green `markdownlint-cli2`, `cargo fmt`, `cargo check`,
    `cargo test` (205 passed), and strict `cargo clippy`.
- Latest pass:
  - lifted `interdiff` and `evolog` from passthrough to selection-aware guided defaults:
    `interdiff --from @- --to <selected>` and `evolog -r <selected>`.
  - lifted `metaedit` to a guided prompt flow using selected-revision default:
    `metaedit -m <message> <selected>`.
  - added native wrapper presentation and snapshot coverage for `interdiff`, `evolog`, and
    `metaedit` outputs.
- Latest pass:
  - reran full validation checkpoint with green `markdownlint-cli2`, `cargo fmt`, `cargo check`,
    `cargo test` (211 passed), and strict `cargo clippy`.
- Latest pass:
  - lifted `simplify-parents` from passthrough to a selection-aware guided default:
    `simplify-parents <selected>`.
  - added native top-level mutation wrapper coverage (including snapshot) for
    `simplify-parents` output so rewrite flows stay presentation-consistent.
- Latest pass:
  - reran full validation checkpoint with green `markdownlint-cli2`, `cargo fmt`, `cargo check`,
    `cargo test` (212 passed), and strict `cargo clippy`.
- Latest pass:
  - lifted `fix` from passthrough to a selection-aware guided default:
    `fix -s <selected>`.
  - aligned command metadata and safety tier so `fix` is treated as a confirm-gated rewrite flow.
  - added native top-level mutation wrapper coverage (including snapshot) for `fix` output.
- Latest pass:
  - reran full validation checkpoint with green `markdownlint-cli2`, `cargo fmt`, `cargo check`,
    `cargo test` (213 passed), and strict `cargo clippy`.
- Latest pass:
  - lifted `resolve` metadata to `guided` to match the existing list-first flow behavior.
  - added native wrapper presentation for non-list `resolve` output, while preserving the
    `resolve -l` list wrapper behavior.
  - added decorator and snapshot coverage for resolve action wrapper rendering.
- Latest pass:
  - reran full validation checkpoint with green `markdownlint-cli2`, `cargo fmt`, `cargo check`,
    `cargo test` (215 passed), and strict `cargo clippy`.
- Latest pass:
  - lifted `diffedit` from passthrough to a selection-aware guided default:
    `diffedit -r <selected>`.
  - aligned command metadata and safety tier so `diffedit` is confirm-gated as a rewrite flow.
  - added native top-level mutation wrapper coverage (including snapshot) for `diffedit` output.
- Latest pass:
  - reran full validation checkpoint with green `markdownlint-cli2`, `cargo fmt`, `cargo check`,
    `cargo test` (216 passed), and strict `cargo clippy`.
- Latest pass:
  - aligned command rendering with `jj` color semantics by forcing colorized subprocess output in
    the `jj` runner for interactive command execution.
  - kept metadata parsing deterministic by using a plain-color runner path for internal log metadata
    queries.
  - hardened ANSI handling in TUI rendering/parsing (width trimming, revision extraction, and
    signal-summary detection) with focused regression tests.
- Latest pass:
  - reran full validation checkpoint with green `markdownlint-cli2`, `cargo fmt`, `cargo check`,
    `cargo test` (218 passed), and strict `cargo clippy`.
- Latest pass:
  - reset terminal color after each rendered row to prevent ANSI style bleed when long colorized
    lines are width-trimmed in the TUI.
  - reran `cargo fmt`, `cargo check`, focused ANSI regression tests, and strict `cargo clippy`.
- Latest pass:
  - added a dedicated normal-mode home binding (`l`) that jumps directly to `log`, making it
    possible to leave transient wrappers (for example `status` and `:commands`) with one keypress.
  - threaded `normal.log` through keybind config loading, keymap rendering, shortcut routing,
    and focused tests.
  - updated README onboarding to document the new one-key return-to-log flow.
- Latest pass:
  - began module-architecture refactor by converting `src/app.rs` into `src/app/` directory
    modules with explicit files for `mod`, `selection`, `preview`, `view`, and `terminal`.
  - moved all `insta` snapshots to `src/app/snapshots` to keep snapshot discovery aligned with the
    new Rust module path.
  - extracted the large app test block into `src/app/tests.rs` and rewired imports to module-local
    helpers.
  - added durable refactor execution plan file:
    `.plans/module-refactor-execution-plan.md`.
- Latest pass:
  - completed `flow` module split into `src/flow/mod.rs`, `planner.rs`, `prompt_kind.rs`,
    `builders.rs`, and `tests.rs`; rewired app/main imports from `crate::flows` to `crate::flow`.
  - split `commands` into `src/commands/spec.rs` and `src/commands/overview.rs` with a thin
    `src/commands/mod.rs` facade.
  - split `alias` into `src/alias/normalize.rs` and `src/alias/catalog.rs` with a thin
    `src/alias/mod.rs` facade.
  - reran `cargo fmt --all`, `cargo check`, and focused `flow`, `alias`, and `commands` tests.
- Latest pass:
  - split `src/app/mod.rs` into focused state-only `mod.rs` plus `runtime.rs`, `input.rs`, and
    `history.rs` impl modules, preserving `App::new` and `App::run`.
  - split `src/app/view.rs` into command-aligned modules under `src/app/view/` while keeping the
    same `super::view::*` function surface used by app logic and tests.
  - reran `cargo fmt --all`, `cargo check`, and full `app::tests::` coverage (167 passed).
- Latest pass:
  - moved `src/config.rs` to `src/config/mod.rs` and extracted schema/merge internals into
    `src/config/raw.rs`.
  - kept `KeybindConfig::load()` and runtime keybinding structs stable for existing call sites.
  - moved config tests to `src/config/tests.rs` and reran `cargo fmt`, `cargo check`,
    `cargo test`, and strict `cargo clippy`.
- Latest pass:
  - continued vertical-slice refactoring by converting `src/app/input.rs` into
    `src/app/input/` with mode-focused modules:
    `mod.rs`, `normal.rs`, `command.rs`, `prompt.rs`, and `confirm.rs`.
  - kept command execution/planning and confirmation flows behavior-identical while reducing
    per-file complexity in the input subsystem.
  - reran `cargo fmt`, `cargo check`, full `cargo test` (218 passed), and strict `cargo clippy`.
- Latest pass:
  - added baseline GitHub CI in `.github/workflows/ci.yml` with format, check, test, clippy, and
    markdown lint jobs.
  - enabled weekly dependency automation via `.github/dependabot.yml` for Cargo and GitHub Actions.
  - added release readiness audit document:
    `docs/release-readiness-audit-2026-02-08.md`.
- Latest pass:
  - finalized dual-license setup using `MIT OR Apache-2.0` with `LICENSE-MIT` and
    `LICENSE-APACHE`.
  - added `SECURITY.md` and generated `CHANGELOG.md` via `git-cliff`.
  - expanded CI to macOS/Windows matrix coverage and added dependency gates
    (`cargo audit`, `cargo deny`).
- Latest pass:
  - migrated runtime rendering from raw crossterm line drawing to ratatui frame rendering.
  - introduced a styled top header bar and mode-aware footer/status bar in the TUI.
  - kept command/view behavior and snapshots stable while removing legacy ASCII underline rows in
    live rendering.
- Latest pass:
  - switched log selection navigation from raw line stepping to revision-item stepping so
    `j`/`k` and arrow movement lands on the next logical revision entry.
  - added viewport paging controls (`PageUp`/`PageDown`, plus `Ctrl+u/d` and `Ctrl+b/f`) and
    surfaced these in the footer/help output.
  - reorganized command help with a common-screens-first section and added
    `docs/navigation-behavior-checklist.md` for terminal/readline/vim UX expectations.
- Latest pass:
  - tightened log selection so graph rows without explicit commit hashes still resolve revision
    anchors and preserve item-based movement.
  - added explicit screen history traversal with back/forward bindings
    (`Left`/`Right`, `Ctrl+o`/`Ctrl+i`) and visible footer discoverability.
  - reordered command help around day-one tutorial flows and kept full command coverage below.
- Latest pass:
  - added command-help spacing snapshots for the new day-one-first `:commands` layout and checked
    in `src/commands/snapshots/` coverage for default and filtered help output.
  - added canonical UX docs: `docs/screens.md` and flow-validation checklist updates in
    `docs/navigation-behavior-checklist.md`.
  - added VHS tapes under `docs/vhs/` and rendered flow GIFs to `target/vhs/`.
- Latest pass:
  - codified UX/navigation guardrails in `AGENTS.md`, including terminal navigation parity,
    log item-based movement requirements, help ordering expectations, and vertical-slice execution
    guidance.
  - reran full validation checkpoint with green `cargo fmt --all`, `cargo check`, `cargo test`
    (226 passed), strict `cargo clippy`, and `markdownlint-cli2`.
- Latest pass:
  - expanded VHS coverage so every documented tutorial flow has a dedicated tape under `docs/vhs/`
    and generated GIF in `target/vhs/`.
  - added `docs/tutorial-vhs.md` as a single tutorial-to-GIF index and linked it from
    `README.md`.
- Latest pass:
  - added a compact tutorial gallery section to `README.md` with embedded local previews for
    key day-one flows and a direct link to full coverage in `docs/tutorial-vhs.md`.
- Latest pass:
  - replaced short per-key tutorial GIFs with a mixed capture strategy:
    static screenshots (`static-*.png`) plus longer dynamic scenario GIFs.
  - switched all VHS tapes to the `Aardvark Blue` theme for stronger visual contrast and clearer
    color differentiation.
  - added scenario documentation in `docs/vhs/scenarios.md` and rewrote
    `docs/tutorial-vhs.md` to map tutorial behaviors to static/dynamic assets.
  - rerendered all tapes with the new scenario set and verified docs lint is green.
- Latest pass:
  - hardened color behavior for captured runs by forcing color-friendly subprocess env in `jj`
    execution (`NO_COLOR` removed, `CLICOLOR_FORCE=1`, `COLORTERM=truecolor`,
    `TERM=xterm-256color`) and in VHS startup commands.
  - slowed tape pacing across all scenarios (longer sleeps and slower typing speed) to avoid
    flicker and make transitions readable.
  - added `docs/vhs/tutorial-full-catalog.tape` and generated a full 25-item screenshot catalog
    (`tutorial-01-*.png` through `tutorial-25-*.png`) so no tutorial item is missing.
  - expanded tutorial docs narrative to include both story-level GIF scenarios and per-item
    catalog mapping.
- Latest pass:
  - fixed log selection/paging semantics to operate on explicit item-start boundaries rather than
    stepping by repeated item count per page.
  - added focused regression tests for viewport-based log paging and a snapshot sequence that locks
    item navigation markers across `Down`, `PageDown`, and `PageUp`.
  - rerendered all VHS assets and tutorial screenshots after the selection fix.
- Latest pass:
  - added durable compaction plan:
    `.plans/readme-ui-compaction-2026-02-08.md`.
  - muted chrome styling in live TUI rendering by unifying header/footer palettes to
    dark-gray background + white foreground and switching selection emphasis to
    marker/foreground highlight (no dark full-row selection fill).
  - reduced footer duplication by hiding routine status messages when they repeat header context.
  - condensed `:commands` and `:keys` into compact two-column layouts with reduced label
    repetition and updated snapshot/test coverage.
  - added a dedicated keymap layout snapshot:
    `src/app/snapshots/jk__app__tests__snapshot_renders_condensed_keymap_layout.snap`.
  - rewrote `README.md` around a workflow-first narrative and moved implementation-heavy details to
    `docs/architecture.md`.
  - reran full validation checkpoint (`cargo fmt`, `cargo check`, `cargo test` 228 passed,
    strict clippy, markdownlint) and regenerated all VHS gifs/screenshots.
- Latest pass:
  - simplified user-facing command terminology by replacing `mode/tier` columns in `:commands`
    with behavior labels (`runs now`, `opens prompt`, `runs as jj`, `asks confirmation`).
  - rewrote `docs/glossary.md` to behavior-first language and removed remaining tier wording from
    `README.md` and `docs/screens.md`.
  - captured the next simplification backlog in
    `.plans/improvements/2026-02-08-022120-jj-aligned-simplification-ideas.md`.
  - reran validation checkpoint (`cargo fmt --all`, `cargo test` 239 passed, strict clippy, and
    markdownlint).
- Latest pass:
  - fixed log item navigation to prioritize visible graph commit rows for selection movement,
    preventing collapsed movement when metadata revision ids repeat.
  - hardened graph-row detection for ANSI-colored log output by stripping ANSI codes before
    commit-row symbol detection.
  - added critical navigation regression coverage:
    - three-entry up/down command-history traversal and snapshot sequence.
    - three-item log down/down/up/up round-trip assertions and snapshot sequence.
    - screen-transition snapshot sequence for commands/keys/aliases/help with back/forward history.
  - reran validation checkpoint (`cargo fmt --all`, `cargo test` 245 passed, `cargo check`, strict
    clippy).
- Latest pass:
  - made help/catalog screens (`commands`, `help`, `keys`, `aliases`) scroll-only in normal mode:
    `Up/Down` now scroll the viewport and no row-selection cursor is shown.
  - stabilized command/help two-column alignment by auto-sizing first-column width from content
    before packing paired rows.
  - added explicit regression coverage for help scrolling semantics and a new help-scroll snapshot.
  - expanded transition snapshots to reflect scroll-only help behavior and reran validation
    (`cargo fmt --all`, `cargo test` 247 passed, `cargo check`, strict clippy, markdownlint).
