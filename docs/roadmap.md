# jk Roadmap

This is the execution map for the deeper [golden plan](golden-plan.md). Keep this page short
enough to drive issues, PRs, and release planning. Use the golden plan for product principles,
keymap rationale, architecture details, docs/site strategy, and release policy.

## North Star

`jk` is a jj-native terminal workbench: focused screens, jj-shaped commands, config-faithful
rendering, safe previews, operation recovery, and first-class workspaces.

The product test is simple: `jk` should feel like interactive jj, not a Git dashboard for jj.

## Milestones

### 0.3 Foundation

Build the architecture needed before the keymap and workflows grow.

- Add `JjCommandSpec`, execution mode, safety class, and refresh plan.
- Replace hard-coded return paths with a view stack and mode stack.
- Move keybindings toward data-backed bindings that generate help and hotbar text.
- Add independent viewport scrolling and ordered marks.
- Expand [Betamax](https://www.joshka.net/betamax/) validation coverage for current log and diff
  workflows.

### 0.4 jj-shaped inspection

Make inspection workflows match jj's command model.

- Support canonical `jk diff -r REV`, `jk diff --from A --to B`, and stat variants.
- Resolve `d` and `S` from cursor plus ordered marks.
- Add selected-change `show` and repository `status` screens.
- Add view-format choices for patch, stat, summary, name-only, and related diff formats.
- Add docs and [Betamax](https://www.joshka.net/betamax/) tapes for log, diff, show, and status
  flows.

### 0.5 Command Mode And Workspaces

Add the two features that turn `jk` from an inspection helper into a daily workbench.

- Add `:` jj command mode with optional `jj` prefix.
- Add `!` external command mode without shell interpretation by default.
- Record command history with argv, output, status, duration, and resulting operation.
- Add `W` workspace screen backed by `jj workspace list`.
- Add workspace actions for status, diff, update-stale, add, and forget.
- Add a `multi-workspace-repo` fixture and Betamax workspace tapes.

### 0.6 Safe Mutation Core

Introduce mutating workflows only through command preview and recovery.

- Add inline and editor describe flows.
- Add new, commit, edit, rebase, and abandon flows.
- Add undo/redo and operation log entry points.
- Log every mutation in command history.
- Make operation recovery visible after every mutation.

### 0.7 Content Workflows

Bring file and hunk workflows into the same command-shaped model.

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
- Add rebase ghost previews and large-repo performance checks.

### 1.0 Obvious Daily Driver

Stabilize the default user experience.

- Polish the core workflows across inspect, workspaces, command mode, mutation, recovery, refs, and
  remotes.
- Complete README, docs, website, generated media, release notes, and install paths.
- Keep release gates boring: Rust checks, markdown, config-fidelity tests, and Betamax suites.

## Issue Candidates

### Add CommandSpec And Preview Scaffold

Create the shared command model that every jj-shaped action will use.

- Scope: `jk-core`, `jk-cli`, and current log/diff command construction.
- Acceptance: commands can render argv and preview text without shell quoting mistakes.
- Tests: command string/argv unit tests and one integration fixture.

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

### Add Independent Scrolling And Ordered Marks

Separate object selection from viewport scrolling, then add ordered revision marks.

- Scope: log state, selected-row state, and rendering affordances.
- Acceptance: `Ctrl-j/Ctrl-k` scroll without changing selection, and `Space` toggles marks.
- Tests: selection/scroll state tests and Betamax log-mark validation tape.

### Make Diff Show And Status jj-shaped

Align inspection commands with jj's canonical argument shapes.

- Scope: CLI parsing, diff resolver, show/status screens, and docs.
- Acceptance: marks plus cursor resolve to clear `jj diff`, `jj show`, and `jj status` commands.
- Tests: command resolver tests, fixture integration tests, and Betamax inspection tapes.

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

### Add Operation Log And Recovery

Make recovery part of the normal mutation loop.

- Scope: op log screen, op show/diff, undo/redo, restore/revert previews.
- Acceptance: every mutation leaves a visible path to command history and operation recovery.
- Tests: operation fixture, op command tests, and Betamax op-log tape.

### Establish Betamax Tape Taxonomy

Promote [Betamax](https://www.joshka.net/betamax/) from media helper to project infrastructure.

- Scope: `tapes/validation`, `tapes/media`, fixture scripts, and `just` recipes.
- Acceptance: validation tapes run in CI-friendly form, media tapes produce reviewable artifacts.
- Tests: `just betamax-validation` and `just betamax-media-smoke`.

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
- object roles and how cursor plus marks resolve them;
- proposed keys and scope;
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
