# jk Roadmap

The [product plan](product-plan.md) defines the product direction and command model. The
[coverage audit](command-coverage.md) records implemented command forms and their limits in the
2026-09-20 integration. The work below remains planned unless that audit marks it implemented.

The [CLI surface addendum](plans/cli-surface-addendum.md) audits jj 0.42.0. Check installed jj help
before adding newer commands or flags.

## North Star

`jk` should feel like interactive jj: focused screens, configured jj output, visible commands and
consequences, operation recovery, and workspace scope. The [TUI design contract](tui-design.md)
requires borderless content, readable hierarchy, and consistent keyboard controls.

## Current integration and next work

The integration includes rebase with one destination, whole-change squash, all-path restore,
workspace lifecycle, bookmark mutations, confirmed fetch, push dry-run, captured external commands,
cancellable log refresh, shared selector handoffs, and command-local Run Options. This is unreleased
integration scope. Milestone numbers describe dependency order, not release versions.

Next work, in dependency order:

1. Add foreground jj editor/tool handoff, starting with editor describe and restoring the terminal
   on success and failure. Reuse it for split, diffedit, and resolve.
1. Add shared fileset selection for commit/split, then partial squash/restore. Keep commit scoped to
   the working copy and show the selected and remaining content.
1. Add final push confirmation after dry-run, retaining the chosen bookmark and remote. Local
   operation undo cannot reverse publication.

Native interdiff can proceed independently. Validate each workflow on fresh disposable repositories
with paired Betamax PNG and terminal-state checkpoints.

## Milestone Goals

### 0.3 Foundation

Command specs, global options, safety classes, ordered marks, independent viewport scrolling, and
cancellable log refresh are implemented. Remaining foundation work:

- Replace hard-coded return paths with a view stack and mode stack.
- Extend data-backed bindings across help and hotbar text.
- Extend cancellation to slow previews.
- Generate a command/flag manifest from installed `jj help` or `jj util markdown-help` output as
  compatibility input.

### 0.4 jj-shaped inspection

Revision and from/to diff forms, selected-change show, status, evolog, diff search, and file/hunk
navigation are implemented. Remaining inspection work:

- Extend reusable `V` View Options across views and configured display options.
- Add searchable file lists, fileset operands, and richer comparisons.
- Expand configuration-fidelity fixtures and paired Betamax checkpoints as these forms grow.

### 0.5 Command Mode And Workspaces

Captured `:` jj commands and shell-free `!` external commands are implemented, with command history,
contextual help, workspace inspection, and workspace lifecycle actions. Remaining work:

- Add foreground editor/tool handoff with terminal restoration on success and failure.
- Add searchable command discovery outside the Help overlay.
- Expose workspace creation parents, messages, and sparse options.

### 0.6 Safe Mutation Core

Introduce mutating workflows through recorded command execution, using command previews and recovery
for safety-sensitive actions.

- Extend the command-local [Run Options drawer](run-options.md) beyond working-copy policy,
  eligible historical operations, and immutable override only when scope and recovery are explicit.
- Add editor describe and before/after review. The inline `m` prompt is implemented: it prefills
  the selected revision's full description, shows an insertion cursor, supports `Ctrl-u` clear,
  and saves directly on `Enter`.
- Complete native commit and advanced new forms. New, edit, inline describe, and abandon already
  have native actions. New uses ordered marks as parents, falling back to the selected revision.
  Abandon checks emptiness and opens destructive confirmation for non-empty revisions.
- Preserve explicit operand roles in whole-change squash, all-path restore, and workspace lifecycle
  previews. Their implemented limits are recorded in the coverage audit.
- Rebase destination search and exact-command confirmation are implemented through `R`.
  The default is revision-only onto; branch, source-and-descendants, and insertion modes are explicit.
  Multi-parent destinations and ghost previews remain follow-up work.
- Extend the implemented undo/redo, operation views, and command-history links to each new mutation
  workflow so recovery stays visible.

### 0.7 Content Workflows

Bring file and hunk workflows into the same command-shaped model.

- Extend the shared revision selector foundation to filesets, operations, bookmarks, tags, remotes,
  and workspaces before expanding content command forms.
- Whole-change squash is implemented. Split, diffedit, and absorb remain. Restore previews an explicit
  selected-commit source, `@` destination, and all-path scope; fileset and hunk restore remain.
- Support file selection first, then hunk-aware paths where jj/editor support is strong enough.
- Add conflict and resolve affordances after the file model is stable.

### 0.8 Refs And Remotes

Make bookmarks, tags, fetch, and push first-class screens.

- Bookmarks now have inspection and create/move/delete previews; dedicated tags remain.
- Explicit remote fetch and scoped push dry-run are implemented. Real push remains deferred until
  its final confirmation and remote-result recovery contract are complete.
- Add remote management only after the common fetch/push path is solid.

### 0.9 Hybrid Rendering Beta

Improve graph interaction without giving up jj config fidelity.

- Add config-fidelity fixtures for templates, graph styles, wrapping, colors, and aliases.
- Introduce hybrid/native graph previews only where tests can prove compatibility.
- Add rebase ghost previews, large-repo performance checks, and slow-preview cancellation checks.

### 1.0 Obvious Daily Driver

Stabilize the default user experience.

- Polish the core workflows across inspect, workspaces, command mode, mutation, recovery, refs, and
  remotes.
- Complete README, docs, website, generated media, release notes, and install paths.
- Run Rust checks, Markdown lint, config-fidelity tests, and Betamax suites before release.

## Dependency Order

Reusable primitives come before broad workflow coverage:

- Command specs must carry `GlobalOptions` and command/flag metadata before mutation previews,
  command history, or command-mode output try to format commands independently.
- `V` View Options must exist before adding separate display toggles for log, diff, show, evolog,
  operation log, or templates.
- Run Options must exist before exposing advanced safety flags such as `--ignore-working-copy`,
  `--at-operation`, `--no-integrate-operation`, `--ignore-immutable`, `--config`, and
  `--config-file`.
- Shared selector models must exist before command families grow bespoke prompts for revsets,
  filesets, operations, bookmarks, tags, remotes, or workspaces.
- Workspaces stay early core scope. The current implementation covers workspace list,
  selected-workspace log/status/diff, and stale-workspace updates before broad history-editing
  polish because multi-workspace state changes how log, status, diff, operation, and sparse
  workflows are shown.
- Betamax validation should be organized by flag families as well as command journeys, so one tape
  can prove shared behavior such as `-R` propagation, View Options formats, Run Options safety, or
  selector resolution across multiple commands.

## Command Family Priorities

These are priorities for native workflows. Typed noninteractive commands cover some forms today;
command mode does not provide foreground terminal handoff, and refs mutations are restricted to their
dedicated flows. See the coverage audit before treating typed command access as native support.

- P0: `log`, `diff`, `show`, `status`, `describe`, `new`, `commit`, `edit`, `rebase`,
  `squash`, `split`, `restore`, operation log/show/diff, undo/redo, bookmarks, `git fetch`,
  `git push`, and workspaces.
- P1: `evolog`, `interdiff`, file list/show/annotate/search, `diffedit`, `absorb`, `fix`,
  `resolve`, tags, git remotes, and git import/export/clone/init.
- P2: `metaedit`, `revert`, `duplicate`, `parallelize`, `simplify-parents`, sparse, config
  inspection, and operation abandon/integrate.
- P3: sign/unsign, bisect, Gerrit, config editing, and forge/ticket/AI integrations.

## Issue Candidates

### Generate jj Command And Flag Manifest

Use installed jj help output as compatibility input for planning, tests, and future manifests.

- Scope: `jj help` or `jj util markdown-help` export, generated snapshots, and compatibility checks.
- Acceptance: supported command specs reference flags present in the manifest, and drift errors name
  the command family and flag.
- Tests: manifest parser fixtures and one generated-help snapshot.

### Add Graph Search And Filter Backtracking

Make long histories and far-away targets practical to navigate.

- Scope: graph search, revset filter input, search match navigation, and previous-filter state.
- Acceptance: `/` finds visible graph text, filter changes can return to the previous view, and the
  selected change is preserved when still visible.
- Tests: search/filter state tests and Betamax search-to-target tape.

### Add Reusable View Options Overlay

Expose display and template flags as reusable view state instead of per-command popups.

- Scope: `V` overlay, diff/display format state, graph/list state, template choices, and command
  spec regeneration.
- Acceptance: log, diff, show, status, evolog, and operation views can share display options without
  taking standalone command keys.
- Tests: view-option state tests, command-spec regeneration tests, and Betamax view-options tapes.

### Add Rich Diff And File Navigation

Make diff inspection useful for large changes without forcing users into another tool.

- Scope: file list, sticky revision/file context, diff search, next/previous file actions,
  two-revision diffs, horizontal overflow controls, edge-case fixtures, and file-only/details mode.
- Current implementation status: diff views support `[ ]` file movement, `{ }` hunk movement,
  search, horizontal scroll, sticky/current-file context, diff format View Options, and an `f` file
  selector that jumps to the chosen file in the active diff.
- Acceptance: users can search within a diff, jump between files, inspect name-only/stat/detail
  variants, compare two selected revisions with the shown jj command, recover from empty or failed
  diff loads without leaving the TUI, and use a searchable file list once the shared file selector
  model exists.
- Tests: diff navigation state tests, command resolver tests, large-diff fixture, and Betamax
  rich-diff tape.

### Add Cancellable Preview Runner

Cancellable log refresh is integrated. Extend cancellation to slow previews and large diffs while
preserving navigation and ignoring superseded results.

- Scope: preview execution, cancellation, stale-result handling, and status messages.
- Acceptance: changing selection cancels or marks stale preview work, and slow commands never block
  graph navigation.
- Tests: fake slow command tests, stale-preview tests, large-diff Betamax tape.

### Define Auto-Refresh Policy

Design automatic refresh before enabling it by default.

- Scope: debounce/coalescing policy, refresh mode status, external-change detection, preservation
  rules, and failure display.
- Acceptance: manual refresh remains predictable, auto-refresh does not steal focus, and selection,
  scroll, expansion, marks, and folded diff state preserve consistently across refresh.
- Tests: debounce tests, external-edit fixture, disappeared-selection fixture, and Betamax
  auto-refresh tape.

### Extend Run Options

The [Run Options drawer](run-options.md) already covers working-copy policy, eligible historical
operations, and immutable override. Add further flags only when their command scope and recovery
behavior are explicit.

- Scope: config overlays, output policy, and additional operation/global-option forms.
- Acceptance: previews show global flags in jj syntax, remote/network commands do not silently use
  local-operation simulation, and time-travel screens clearly show working-copy policy.
- Tests: command-spec tests for global flag ordering and Betamax run-options tape.

### Extend Shared Selectors

Extend the existing typed selector handoffs for file and hunk workflows.

- Scope: fileset selection first, then additional revision, operation, tag, and workspace operands.
- Acceptance: role pickers and command previews reuse selector output across inspection, mutation,
  refs, remotes, and workspace workflows.
- Tests: selector resolution tests and Betamax selector-family tapes.

### Extend Rebase Destinations And Previews

The implemented picker resolves one source and destination with explicit role and placement
choices. Extend it without changing those command meanings.

- Scope: multi-parent destinations and, once validated against jj, graph previews.
- Acceptance: users can search to a destination, see the exact rebase command, cancel safely, and
  reach operation recovery after success.
- Tests: role resolver tests, rebase fixture tests, and Betamax rebase-preview tape.
- Implemented scope: one exact source, visible-destination search, explicit roles, cancellation,
  shared confirmation, recorded operation, and refreshed log. Combined evidence is reproducible with
  `just betamax-workflows`; multi-parent selection and ghost previews remain deferred.

### Harden jj Integration Boundary

Keep the current jj bridge narrow until replacing it reduces duplicated jj behavior.

- Scope: rendered-output provider, semantic template pass, parser fixtures, error messages, and a
  short `jj-cli` or upstream log-core spike.
- Acceptance: rendered output stays opaque, semantic parsing is fixture-backed, drift errors name
  the jj/config contract involved, and no direct integration copies jj presentation logic just to
  avoid shelling out.
- Tests: rendered/semantic alignment fixtures, custom template fixtures, and version/config-drift
  error tests.

### Prove Release Publishing And Install Assets

Make release infrastructure match the project plan before relying on it.

- Scope: crates.io trusted publishing for all four crates, required status checks, GitHub Release
  archive naming, cargo-binstall smoke, and Homebrew formula smoke.
- Acceptance: release-plz can publish without `CARGO_REGISTRY_TOKEN`, `Check`/`Markdown`/`MSRV`
  are required on `main`, and release archives plus `.sha256` files install through downstream
  paths.
- Tests: release dry-run or first release readback, cargo-binstall install smoke, Homebrew formula
  smoke, and branch-protection/ruleset readback.

### Establish Betamax Tape Taxonomy

Organize [Betamax](https://www.joshka.net/betamax/) tapes by validation purpose and shared command
behavior, with separate media pacing where needed.

- Scope: `tapes/validation`, `tapes/media`, flag-family matrix, fixture scripts, and `just`
  recipes.
- Acceptance: validation tapes run in CI-friendly form, media tapes produce reviewable artifacts.
- Tests: `just betamax-validation`, `just betamax-media-smoke`, and family tapes for global options,
  selectors, view options, run options, workspaces, and push safety.

### Build Betamax-driven Docs

Make docs, README media, website media, and release demos come from repeatable terminal stories.

- Scope: guide template, media tape naming, screenshot/GIF publishing, and docs references.
- Acceptance: each user guide has a concept explanation, jj equivalent, jk interaction sequence,
  Betamax-rendered artifact, and command-mode equivalent where useful.
- Tests: at least one docs media tape proves the pipeline from fixture to generated artifact.

### Define Betamax And jk Co-development Contract

Capture the requirements `jk` should push into [Betamax](https://www.joshka.net/betamax/) and the
requirements Betamax should push back into `jk`.

- Scope: deterministic demo mode, fixture stability, state JSON filtering, artifact manifests,
  theme/font stability, failed-checkpoint screenshots, and command history export.
- Acceptance: missing Betamax capabilities are tracked as explicit follow-up issues, and new `jk`
  UI workflows expose stable screen titles, hotbar text, and test-only overrides where needed.
- Tests: one intentionally failing validation tape preserves a useful screenshot/state artifact.

### Add Betamax PR And Release Gates

Require repeatable interaction evidence for UI-changing PRs and release candidates.

- Scope: PR template, CI artifacts, `just betamax-validation`, and `just betamax-media-smoke`.
- Acceptance: UI-changing PRs include validation tape updates and attach or link artifacts;
  release candidates run the stable Betamax suite before publishing.
- Tests: validation artifacts include State JSON and screenshots; media artifacts include GIF or
  MP4 where appropriate.

### Add First ADRs

Capture durable decisions before the code grows around them.

- Scope: `docs/adr/0001-command-spec.md`, `0002-keymap-principles.md`, and
  `0003-rendering-strategy.md`.
- Acceptance: each ADR records context, decision, consequences, alternatives, rollout, and
  compatibility notes.

## Issue Template Fields

Every major workflow issue should include:

- jj command or command family involved;
- relevant flag families;
- object roles and how cursor plus marks resolve them;
- selector models used;
- proposed keys and scope;
- prior-art or community-demand signal, when relevant;
- preview behavior;
- recovery behavior;
- docs, tests, and Betamax tapes needed.

## Website And Release Visibility

Product-visible `jk` changes should trigger a website pass in `/Users/joshka/local/jk-website`
when they change install instructions, released features, key workflows, screenshots, GIFs, command
examples, or README/crates.io positioning.

README, crates.io, website, and release-note media should come from
[Betamax](https://www.joshka.net/betamax/) tapes. Store generated README and crates.io media in the
separate [`jk-screenshots`](https://github.com/joshka/jk-screenshots) repository, not in this source
tree.
