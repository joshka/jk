# Changelog

This changelog is written for people deciding whether to install, upgrade, demo, or review `jk`.
Entries are grouped by user workflow and release impact instead of following the commit history.

## Unreleased

## 0.2.7 - 2026-06-24

This patch release tightens log graph behavior and workspace discovery after the dogfoodable 0.2.6
release candidate.

### Log Graph Rendering

- Fix inline expanded details so they do not draw vertical continuation lanes through commit nodes.
- Preserve connector lanes for graph rows that attach upward to nodes, including `├───╯` shapes.
- Fix graph expansion links and elision drill-in column matching so hidden-revision navigation
  follows the selected graph lane.
- Shorten revision ids shown by log navigation actions.

### Workspace Discovery

- Handle `jk` started from jj no-working-copy container roots by selecting the active child
  workspace instead of failing at the container root.
- Handle stale jj working copies more gracefully when discovering and refreshing workspaces.

### Visual Polish

- Simplify selected-row highlighting so graph rows stay easier to scan.

## 0.2.6 - 2026-06-23

This release candidate turns `jk` from a log/diff inspection helper into a dogfoodable jj TUI.
The release cutoff is the current inspection, command, history, recovery, workspace, and diff-review
surface. Rebase and larger action menus remain intentionally deferred.

### Inspection And Navigation

- Keep `jj` as the source of truth for rendered log, diff, show, status, and evolog output while
  adding interactive navigation around that output.
- Support `jk`, `jk log`, `jk diff`, `jk show`, and `jk status` entry points, including repository
  and limit options shared with the top-level command.
- Add selected-change inspection from the log for show, diff, evolog, and repository status.
- Preserve log selection when moving into inspection views, refreshing them, and returning to the
  log.
- Add ordered revision marks so multi-revision workflows can select parents in the order the user
  chose them.

### Diff Review

- Add line, page, file, hunk, and horizontal movement for diff review without changing the rendered
  `jj diff` text.
- Add file and hunk folding, fold-all and unfold-all commands, and a file-list overlay for jumping
  within large diffs.
- Add sticky current-file context with diff stat suffixes and file position, such as `[file 2/7]`.
- Add diff search with forward and backward match navigation.
- Add View Options for switching the rendered diff format between patch, summary, stat, types,
  name-only, git, and color-words output.

### Command Mode And Command History

- Add `:` command mode for running direct `jj` commands from inside the TUI.
- Capture command output, errors, exit status, duration, command arguments, and operation ids in an
  in-memory Command History.
- Keep failed command output available for inspection instead of dropping the user back into an
  unexplained terminal state.
- Add history detail, copy, retry, and operation links so command results can be inspected after the
  command finishes.

### Safe Mutation Previews And Recovery

- Add preview-and-confirm flows for `jj describe`, `jj abandon`, `jj new`, `jj edit`, `jj undo`, and
  `jj redo`.
- Show the exact command before it runs, with cancel and copy paths available before confirmation.
- Prefill describe text from the selected change and support clearing the prompt before rewriting it.
- Use marked revisions as ordered `jj new` parents.
- Capture operation ids from mutating commands and expose a recovery footer with undo, redo,
  operation log, operation show, operation diff, and Command History entry points.

### Workspaces And Operations

- Add a Workspaces screen with list, refresh, selected-workspace log, selected-workspace status,
  selected-workspace diff, and stale-workspace update paths.
- Add Operation Log, operation show, and operation diff views so recent repository state changes are
  inspectable from inside the TUI.
- Keep workspace and operation inspection on the same rendered-output model as show, status, and
  diff views.

### Help, Discovery, And Menus

- Generate mode-specific help and hotbar labels from the keymap data used by the app.
- Add searchable command discovery from help so users can find commands without memorizing every
  binding.
- Add View Options and template selectors as menu overlays that wrap, start at the current
  selection, and support arrow-key movement.
- Order help content by workflow so common inspection, movement, mutation, recovery, and exit
  commands are easier to scan.
- Show the active log template name instead of a placeholder when a custom template is used.

### Documentation, Release Notes, And Media

- Update the repository README and crate README to describe the current jk surface rather
  than the earlier log/diff-only milestone.
- Add a release stabilization plan that treats changelog and release notes as written product
  artifacts based on a feature audit, not generated commit summaries.
- Document the release cutoff before rebase, the command-preview safety model, current known
  limitations, and the required public media refresh.
- Keep generated validation media local until release assets are prepared in the repositories that
  own public screenshots, GIFs, and website media.

### Known Limitations

- Command History is currently in-memory for the active `jk` session.
- Rebase, squash, split, restore, bookmark, fetch, and push workflows are still planned.
- Direct `a`, `n`, and `e` bindings are dogfood shortcuts until the broader action menu exists.
- Public README GIFs, crates.io media, and website media still need a release-media refresh before
  they should advertise the full TUI surface.
- The project website may intentionally lag a source release, but it should not pre-publish
  unreleased behavior as if it were already available.

## 0.2.5 And Earlier

Earlier releases established the public installation path and the initial `jj`-rendered terminal UI
foundation:

- log and selected-change diff views;
- README and crates.io media for the early log/diff experience;
- Homebrew, `cargo-binstall`, and `cargo install` installation paths;
- package homepage metadata and release automation cleanup.

The previous generated changelog entries for these releases were duplicated and commit-oriented.
They have been collapsed here into a reader-facing summary. Use the GitHub release and pull request
history for exact patch-level archaeology.
