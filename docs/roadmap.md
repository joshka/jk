# jk Roadmap

This is the execution map for the deeper [product plan](product-plan.md). Keep this page short
enough to drive issues, PRs, and release planning. Use the product plan for product principles,
keymap rationale, architecture details, docs/site strategy, and release policy.

The standalone [CLI surface addendum](plans/cli-surface-addendum.md) is the compatibility audit for
`jj 0.42.0` command families and flags. Do not inline it here; use it to keep this roadmap ordered
around reusable flag families instead of one-off command forms.

## North Star

`jk` is a jj-native terminal UI: focused screens, jj-shaped commands, config-faithful
rendering, safe previews, operation recovery, and first-class workspaces.

The product test is simple: `jk` should feel like interactive jj, not a Git dashboard for jj.

## Milestones

### 0.3 Foundation

Build the architecture needed before the keymap and workflows grow.

- Add `JjCommandSpec`, `GlobalOptions`, execution mode, safety class, and refresh plan.
- Replace hard-coded return paths with a view stack and mode stack.
- Move keybindings toward data-backed bindings that generate help and hotbar text.
- Add independent viewport scrolling and ordered marks.
- Reserve `V` for reusable View Options and standalone `v` for `evolog` before adding more
  display shortcuts.
- Add the cancellable task model needed for slow previews and refreshes.
- Start a generated command/flag manifest from installed `jj help` or `jj util markdown-help`
  output, and treat it as compatibility input rather than runtime UI.
- Expand [Betamax](https://www.joshka.net/betamax/) validation coverage for current log and diff
  workflows.

### 0.4 jj-shaped inspection

Make inspection workflows match jj's command model.

- Support canonical `jk diff -r REV`, `jk diff --from A --to B`, and stat variants.
- Resolve `d` and `S` from cursor plus ordered marks.
- Add selected-change `show` and repository `status` screens.
- Add a reusable `V` View Options overlay for patch, stat, summary, name-only, templates, graph
  options, and related diff formats.
- Add standalone `v` for selected-change `evolog` after the shared inspection source model exists.
- Add first-class diff search, file-list navigation, and two-revision comparison. The current
  implementation has the first diff file selector (`f`) for jumping within the active diff;
  searchable file lists and range comparison still need follow-up work.
- Add docs and [Betamax](https://www.joshka.net/betamax/) tapes for log, diff, show, and status
  flows.

### 0.5 Command Mode And Workspaces

Add the two features that turn `jk` from an inspection helper into a daily jj TUI.

- Add `:` jj command mode with optional `jj` prefix.
- Add `!` external command mode without shell interpretation by default.
- Add searchable contextual help so users can find actions by key, name, and jj command family.
- Record command history with argv, output, status, duration, and resulting operation.
- Add `W` workspace screen backed by `jj workspace list`.
- Add workspace actions for status, diff, update-stale, add, and forget.
- Add a `multi-workspace-repo` fixture and Betamax workspace tapes.

### 0.6 Safe Mutation Core

Introduce mutating workflows only through command preview and recovery.

- Add a reusable Run Options drawer for global options and advanced safety toggles before mutating
  wizards depend on them.
- Add inline and editor describe flows. The current implementation prefills the inline `m` prompt
  from the selected revision's full description and supports `Ctrl-u` clear before preview; editor
  describe and before/after review remain follow-up work.
- Add new, commit, edit, rebase, and abandon flows. The current implementation has direct
  selected-revision `a` -> `jj abandon REV`, `n` -> `jj new PARENT...`, and
  `e` -> `jj edit REV` previews. `jj new` already uses ordered marks as parents when present and
  falls back to the selected revision; the durable action-menu shape still belongs with the broader
  mutation selector work.
- Add rebase destination search and command preview before any graph mutation.
- Add undo/redo and operation log entry points.
- Log every mutation in command history.
- Make operation recovery visible after every mutation.

### 0.7 Content Workflows

Bring file and hunk workflows into the same command-shaped model.

- Add shared selector models for revisions, filesets, operations, bookmarks, tags, remotes, and
  workspaces before expanding content command forms.
- Add squash, split, restore, diffedit, and absorb flows.
- Support file selection first, then hunk-aware paths where jj/editor support is strong enough.
- Add conflict and resolve affordances after the file model is stable.

### 0.8 Refs And Remotes

Make bookmarks, tags, fetch, and push first-class screens.

- Add bookmark and tag list screens with scoped actions.
- Add fetch and push flows, with push dry-run first.
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
- Keep release gates boring: Rust checks, markdown, config-fidelity tests, and Betamax suites.

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

These priorities summarize direct `jk` surface area. Lower-priority families should still work
through `:` command mode once command mode exists.

- P0: `log`, `diff`, `show`, `status`, `describe`, `new`, `commit`, `edit`, `rebase`,
  `squash`, `split`, `restore`, operation log/show/diff, undo/redo, bookmarks, `git fetch`,
  `git push`, and workspaces.
- P1: `evolog`, `interdiff`, file list/show/annotate/search, `diffedit`, `absorb`, `fix`,
  `resolve`, tags, git remotes, and git import/export/clone/init.
- P2: `metaedit`, `revert`, `duplicate`, `parallelize`, `simplify-parents`, sparse, config
  inspection, and operation abandon/integrate.
- P3: sign/unsign, bisect, Gerrit, config editing, and forge/ticket/AI integrations.

## Issue Candidates

### Add CommandSpec And Preview Scaffold

Create the shared command model that every jj-shaped action will use.

- Scope: `jk-core`, `jk-cli`, and current log/diff command construction.
- Acceptance: commands can render argv and preview text without shell quoting mistakes, preserve
  `GlobalOptions`, and keep global flags before the jj command family.
- Tests: command string/argv unit tests, global-option ordering tests, and one integration fixture.

### Generate jj Command And Flag Manifest

Use installed jj help output as compatibility input for planning, tests, and future manifests.

- Scope: `jj help` or `jj util markdown-help` export, generated snapshots, and compatibility checks.
- Acceptance: supported command specs reference flags present in the manifest, and drift errors name
  the command family and flag.
- Tests: manifest parser fixtures and one generated-help snapshot.

### Introduce ViewStack And ModeStack

Replace the current log/diff return behavior with a general stack.

- Scope: `jk-tui` state and input handling.
- Acceptance: existing log to diff to back flow still works, and `Backspace` pops one view.
- Tests: state tests for preserved selection, scroll, marks, and transient overlay handling.

### Move Keymap Help And Hotbar To Data

Make `?` and the future hotbar generate from the same binding registry.

- Scope: key definitions, help overlay, and conflict tests.
- Acceptance: adding or remapping a key updates contextual help without hand-editing text.
- Tests: key conflict tests and generated help snapshots.

### Add Searchable Command Discovery

Make help and command discovery searchable without replacing `:` command mode.

- Scope: help overlay, action registry metadata, command family tags, and filter input.
- Acceptance: users can filter actions by key, action name, current screen, and jj command family.
- Tests: action metadata tests, filter tests, and Betamax searchable-help tape.

### Add Independent Scrolling And Ordered Marks

Separate object selection from viewport scrolling, then add ordered revision marks.

- Scope: log state, selected-row state, and rendering affordances.
- Acceptance: `Ctrl-j/Ctrl-k` scroll without changing selection, and `Space` toggles marks.
- Tests: selection/scroll state tests and Betamax log-mark validation tape.

### Add Graph Search And Filter Backtracking

Make long histories and far-away targets practical to navigate.

- Scope: graph search, revset filter input, search match navigation, and previous-filter state.
- Acceptance: `/` finds visible graph text, filter changes can return to the previous view, and the
  selected change is preserved when still visible.
- Tests: search/filter state tests and Betamax search-to-target tape.

### Make Diff Show And Status jj-shaped

Align inspection commands with jj's canonical argument shapes.

- Scope: CLI parsing, diff resolver, show/status screens, and docs.
- Acceptance: marks plus cursor resolve to clear `jj diff`, `jj show`, and `jj status` commands.
- Tests: command resolver tests, fixture integration tests, and Betamax inspection tapes.

### Add Reusable View Options Overlay

Expose display and template flags as reusable view state instead of per-command popups.

- Scope: `V` overlay, diff/display format state, graph/list state, template choices, and command
  spec regeneration.
- Acceptance: log, diff, show, status, evolog, and operation views can share display options without
  taking standalone command keys.
- Tests: view-option state tests, command-spec regeneration tests, and Betamax view-options tapes.

### Add Selected-change Evolog

Make change evolution visible without stealing display-option keys.

- Scope: standalone `v`, `jj evolog -r`, evolog view state, and follow-up interdiff affordances.
- Acceptance: `v` opens evolution history for the selected change, while `V` remains View Options.
- Tests: evolog command tests and Betamax evolog tape.

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

Keep the UI responsive while slow previews, external tools, or large diffs are running.

- Scope: async runner, cancellation tokens, stale-result handling, status messages, and refresh
  policy.
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

### Add Command Mode And Command History

Let users run jj-shaped commands and see everything `jk` has run.

- Scope: command input overlay, process runner, output view, and command history storage.
- Acceptance: `:` runs jj commands, `!` runs external commands, failures keep output visible.
- Tests: command parser tests, failed-command tests, and command-mode Betamax tape.

### Add Workspace Screen

Make workspaces visible early because they are a normal daily jj workflow.

- Scope: `W` screen, workspace provider, workspace actions, and fixture setup.
- Acceptance: users can list workspaces, identify the current workspace, inspect status/diff, and
  update stale workspaces.
- Tests: multi-workspace fixture, command resolver tests, and `docs-workspaces` media tape.

### Add Run Options Drawer

Expose global execution context and advanced safety flags consistently.

- Scope: repository, working-copy policy, operation time travel, operation integration,
  immutability override, config overlays, and output policy.
- Acceptance: previews show global flags in jj syntax, remote/network commands do not silently use
  local-operation simulation, and time-travel screens clearly show working-copy policy.
- Tests: command-spec tests for global flag ordering and Betamax run-options tape.

### Add Shared Selector Models

Prevent command-family forms from inventing one-off selectors.

- Scope: revision, fileset, operation, bookmark, tag, remote, and workspace selector state.
- Acceptance: role pickers and command previews reuse selector output across inspection, mutation,
  refs, remotes, and workspace workflows.
- Tests: selector resolution tests and Betamax selector-family tapes.

### Add Operation Log And Recovery

Make recovery part of the normal mutation loop.

- Scope: op log screen, op show/diff, undo/redo, restore/revert previews.
- Acceptance: every mutation leaves a visible path to command history and operation recovery.
- Tests: operation fixture, op command tests, and Betamax op-log tape.

### Add Rebase Destination Picker And Preview

Make graph mutation safer by resolving roles visibly before running jj.

- Scope: source/destination role resolver, destination search, insert-before/after modes,
  multi-parent destinations, command preview, and later ghost preview.
- Acceptance: users can search to a destination, see the exact rebase command, cancel safely, and
  reach operation recovery after success.
- Tests: role resolver tests, rebase fixture tests, and Betamax rebase-preview tape.

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

Promote [Betamax](https://www.joshka.net/betamax/) from media helper to project infrastructure.

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

Make visual/interactive workflow evidence part of review and release, not an optional attachment.

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
