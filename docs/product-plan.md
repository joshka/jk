# jk Product Plan

> A single document of record for building `jk` into the obvious terminal UI for Jujutsu (`jj`).
>
> Status: product and engineering north star, plus appendix corrections from planning review.  
> Date: 2026-06-22.  
> Audience: maintainers, contributors, Codex agents, release managers, docs/site maintainers.

## How to read this plan

This document is an explanation, roadmap, and decision record. It is intentionally broader than the
current implementation. The README and crate README should continue to describe what `jk` does
today; this plan describes the product direction, design contracts, and reviewable slices needed to
get there.

Treat strong claims in this document as one of three things:

- **Current behavior:** backed by the live repo, README, crate docs, or existing tapes.
- **Design contract:** a rule future implementation work should preserve unless an ADR changes it.
- **Roadmap target:** planned behavior that needs implementation, tests, docs, and Betamax proof.

When current `jj` behavior matters, verify against the installed `jj help <COMMAND>` output and the
user's configured `jj` version. External docs and prior-art READMEs are useful references, but `jk`
compatibility work should not outrank the local `jj` command surface.

The standalone [CLI surface addendum](plans/cli-surface-addendum.md) is the command/flag audit for
`jj 0.42.0`. It sharpens this plan without replacing it: use the addendum for reusable flag-family
primitives, `v`/`V` key assignments, selector coverage, and command-family priority updates.

Useful source links for `jj` compatibility and ecosystem context:

- [Jujutsu CLI reference](https://docs.jj-vcs.dev/latest/cli-reference/);
- [Jujutsu configuration reference](https://docs.jj-vcs.dev/latest/config/);
- [Jujutsu templating language](https://docs.jj-vcs.dev/latest/templates/);
- [Jujutsu community-built tools](https://docs.jj-vcs.dev/latest/community_tools/).

## 0. Executive thesis

`jk` should become the terminal workbench that feels like the interactive form of `jj` itself.

The product should not be positioned as "a lazygit for jj" and should not be constrained by the
narrow first-screen framing. The default screen can remain the revision graph because that is where
many jj workflows begin, but the durable positioning is broader:

> `jk` is a jj-native terminal workbench. It follows Jujutsu's command model and configuration, then
> adds fast navigation, selection, previews, safe mutations, operation recovery, command history,
> and command mode.

The two product contracts are separate and equally important:

1. **Command fidelity.** Every user action should be explainable as a jj-shaped command or a short
   jj-shaped command sequence. Mutations should show the command before running it.
2. **Presentation fidelity.** `jk` should honor jj configuration: default command, revsets,
   templates, template aliases, graph style, node templates, colors, diff format, wrapping,
   editor/diff-editor settings, immutable rules, bookmarks/tags/remotes, and operation history
   semantics.

Rendering can begin by delegating to `jj` and may evolve toward native rendering. The invariant is
not "jj renders every byte forever." The invariant is:

> Same jj config. Same jj command model. Better interactive surface.

## 1. Product north star

The goal is to be so good that a terminal-oriented jj user instinctively reaches for `jk` first, and
a new jj user learns jj faster because `jk` makes the underlying commands visible.

### 1.1 What winning looks like

A user can:

- Open a repo and immediately see the revision graph, working copy, bookmarks, tags, remote state,
  and current operation context.
- Move by revision, line, section, file, hunk, operation, bookmark, or search result without losing
  context.
- Mark revisions/files/hunks/operations/bookmarks and build jj-shaped commands from those marks.
- Diff, show, interdiff, evolog, status, and op-log inspect without leaving the TUI.
- Rebase, squash, split, absorb, restore, abandon, describe, edit, new, commit, bookmark, tag,
  fetch, push, undo, redo, and recover from operations with clear previews.
- Jump to `$EDITOR` or the configured diff editor when jj itself would, then return to a refreshed
  TUI.
- Use `:` to run a jj command and `!` to run an external command without leaving the app.
- See a command history of everything `jk` ran, including duration, exit status, output, and
  resulting operation.
- Use `?` anywhere and see accurate, generated help for the current mode and keymap.
- Configure keybindings later without docs/help drifting.

### 1.2 Product personality

`jk` should feel:

- **jj-native.** It uses jj language: revisions, changes, revsets, filesets, bookmarks, operations,
  working-copy commit, conflicts, immutable commits.
- **Fast.** Navigation must be immediate, even in large repositories.
- **Safe.** Mutations show preview commands and preserve a route to op-log recovery.
- **Transparent.** Users can copy the exact jj command or command sequence that an action would run.
- **Discoverable.** The footer/hotbar says what matters now; `?` explains the rest.
- **Powerful by composition.** Object under cursor + marks + current screen = resolved command.

## 2. Prior art and what to borrow

The jj ecosystem already includes multiple TUIs and interactive tools. The official
[Jujutsu community-built tools](https://docs.jj-vcs.dev/latest/community_tools/) page lists tools
such as [JJ TUI](https://github.com/faldor20/jj_tui),
[LazyJJ](https://github.com/Cretezy/lazyjj), [Jujutsu UI](https://github.com/idursun/jjui),
[jj-hunk](https://github.com/laulauland/jj-hunk), [JJ-FZF](https://github.com/tim-janik/jj-fzf),
[LightJJ](https://github.com/chronologos/lightjj), and
[JJ View](https://github.com/brychanrobot/jj-view), while noting that community tools are not
reviewed, endorsed, or guaranteed by the jj project. `jk` should learn from them but use jj's
CLI/config semantics as the compatibility oracle.

The installed `jj help <COMMAND>` output should remain the command-compatibility oracle. The
generated jj CLI reference is useful, but the plan should not assume generated docs are more
authoritative than the installed `jj` version a user is actually running.

Implementation languages are worth tracking in this section because they shape what each project
proves. A Go TUI, an OCaml/Dune graph renderer, and Rust/Ratatui tools each validate different
architecture and ecosystem choices.

### 2.1 jjui

Language / stack: Go.

`jjui` demonstrates broad jj-specific feature coverage: revset autocomplete, rebase, squash,
revision details, bookmarks, op log, preview, git fetch/push, undo/redo, evolog, and
jump-to-revision.

Borrow:

- Revset input with completion/signature help.
- Object preview: revision -> `jj show`, file -> `jj diff`, operation -> `jj op show`.
- First-class op log.
- Details view with file-level actions.

Do not copy directly:

- A flat global keymap where lowercase/uppercase letters carry too much overloaded meaning.

### 2.2 JJ TUI / jju

Languages / stacks:

- `faldor20/jj_tui`: OCaml / Dune.
- `jju`: Rust.

`jj_tui` and `jju` show that visual rebase previews, prefix menus, multi-select, and graph
interaction are valuable.

Borrow:

- Visual rebase destination-pick mode.
- Prefix menus for collision-heavy command families.
- Multi-select with `Space`.
- Configurable key sequences eventually.
- Neighborhood/zoom ideas as later graph-navigation features.

Be cautious with:

- Reimplementing graph rendering without first defining config-fidelity tests.
- Requiring mouse/drag interaction for core flows.

### 2.3 lazyjj / blazingjj

Language / stack: Rust / Ratatui.

`lazyjj` and `blazingjj` show a lazygit-style jj UI with tabs, side-panel details, command box,
command log, bookmarks, files, fetch/push, and configurable keys.

Borrow:

- Command log.
- `:` command box where `jj` prefix is optional.
- Bookmarks and files as first-class screens.
- Configurable keys later.

Change for `jk`:

- Keep `d = diff`; use `m/M` for describe/message.
- Keep `P = push`; leave `p` available for temporary preview/details views.
- Put abandon/absorb/squash/split/restore/revert under an action prefix instead of global
  `a/A/s/S/r/R` overloads.

### 2.4 GitButler TUI

Language / stack: Tauri + Svelte/TypeScript UI, Rust backend and CLI.

GitButler's TUI is especially useful as a terminal-workbench reference. It has a full workspace TUI,
focused diff TUI, stage/hunk picker, command mode, generated help, hotbar, marked commits,
inline/editor rewording, external commands, undo/redo, and operation recovery.

Borrow heavily:

- A contextual hotbar showing the most useful actions in the current mode.
- `?` generated help as source of truth.
- `:` native command mode and `!` external command mode.
- Clear distinction between full workspace TUI, focused diff TUI, and focused hunk/stage picker.
- Successful command reloads state; failed command keeps output visible.
- Inline message flow plus external editor flow.
- Marked object workflows.

Keep `jk` distinct:

- `jk` should be jj-native, not a Git branch/forge/workspace product first.
- Forge/ticket/AI integrations are optional future layers, not part of the core 1.0 promise.

### 2.5 Other broad jj TUIs

`madicen/jj-tui` is a Go / Bubble Tea / Lip Gloss project with a broader dashboard-style surface:
graph, files, GitHub PRs, tickets, settings, command history, mouse, external editor, branches,
and remotes.

Borrow:

- Evidence that broad workflow coverage can live in a terminal UI.
- Command history and external editor ideas.
- The value of making remotes and related project context discoverable.

Do not copy directly:

- A forge/ticket/productivity dashboard as the default `jk` identity.
- A permanent pane-first layout.

### 2.6 Prior-art implementation notes

These notes should stay in the plan because they explain what each project proves technically, not
only what it looks like as a product.

Direct project links:

- [jjui](https://github.com/idursun/jjui);
- [faldor20/jj_tui](https://github.com/faldor20/jj_tui);
- [lazyjj](https://github.com/Cretezy/lazyjj);
- [blazingjj](https://github.com/blazingjj/blazingjj);
- [jju](https://github.com/praveenperera/jju);
- [madicen/jj-tui](https://github.com/madicen/jj-tui);
- [GitButler](https://github.com/gitbutlerapp/gitbutler).

```text
jjui
  Language / stack: Go
  Notes: Feature-rich jj TUI with revset autocomplete, rebase, squash,
  details, bookmarks, op log, preview, git fetch/push, undo/redo.
  Source signal: README documents:
    go install github.com/idursun/jjui/cmd/jjui@latest
    go build ./cmd/jjui

faldor20/jj_tui
  Language / stack: OCaml / Dune
  Notes: Reimplements jj graph rendering for snappy interaction and visual
  rebase previews; has configurable keymaps and command submenus.
  Source signal: README points to .ml keymap/config internals and Dune build.

lazyjj
  Language / stack: Rust / Ratatui
  Notes: LazyGit-like jj TUI with log, files, bookmarks, command log,
  command box, and configurable keybindings.
  Source signal: README says it is built in Rust with Ratatui and uses jj CLI.

blazingjj
  Language / stack: Rust / Ratatui
  Notes: Fork of lazyjj with similar layout and key model, plus absorb and
  yanking change IDs.
  Source signal: README says it is a Rust/Ratatui lazyjj fork using jj CLI.

jju
  Language / stack: Rust
  Notes: Visual rebase preview, prefix menus, configurable key sequences,
  bookmark actions, non-interactive hunk splitting, daily workflow focus.
  Source signal: README installs with:
    cargo install jju
  It also documents TOML keybindings with command IDs.

madicen/jj-tui
  Language / stack: Go / Bubble Tea / Lip Gloss
  Notes: Broad product surface: graph, files, GitHub PRs, tickets,
  settings, command history, mouse, external editor, branches, remotes.
  Source signal: README documents Go installation/build commands.

GitButler / but tui
  Language / stack: Tauri + Svelte/TypeScript UI, Rust backend and CLI
  Notes: Not a jj TUI, but relevant for hotbar, generated help, command mode,
  operation history, undo/redo, external command handling, and focused flows.
  Source signal: README describes a Tauri/Svelte/TypeScript desktop app with
  a Rust backend, and a CLI using the same Rust backend engine.
```

### 2.7 User demand signals from prior art

The planning implication is not "copy the most active TUI." It is that repeated user requests
cluster around a small number of workflow pressures. These should steer issue priority, docs,
website demos, release notes, and Betamax tapes.

Important framing:

- Treat the project author's theme/dark-mode and
  [jj-vcs/jj#9319](https://github.com/jj-vcs/jj/pull/9319) design material as product intent,
  not as independent demand evidence.
- Preserve the "interactive jj" stance. Users praise tools that extend jj and teach the underlying
  command, while pane-first tools are criticized when it is unclear which region or object is active.
- Do not optimize for a permanent pane grid. The demand is for fast focus changes, previews,
  pickers, overlays, and stable context.
- Workspaces show up as normal daily jj usage. They should stay early core scope.

Signals to incorporate:

- **TUI adoption pressure is real, but blessing one tool is contentious.** Some users want an
  obvious default recommendation because newcomers ask which TUI to use. Others worry that a blessed
  UI would narrow experimentation. `jk` should win by being clearly jj-native, well documented, and
  demonstrably safer rather than by claiming official status.
- **Command discovery needs to be searchable.** `?` should not only be a static help sheet. Users
  ask for a command palette or filterable command list that can find actions by name, key, and jj
  command shape. `:` remains command entry; searchable help is a separate discovery surface.
- **Revision navigation needs more than cursor movement.** Long histories and rebase targets far
  away from the cursor need `/` search, jump-to-revision, quick filters, and a way to return to the
  previous filter or default revset without losing context.
- **Diff and file browsing are a major product differentiator.** Users ask for repo-at-revision file
  browsing, searchable file lists, sticky revision/file context, next/previous file navigation,
  diff search, range diffs, two-revision diffs, and easy toggles between patch, stat, summary, and
  file-only views.
- **Rebase and graph manipulation should be visual before they mutate.** Strong demand clusters
  around multi-parent change creation, rebase destination picking, insert-before/after, selection
  order that is hard to get wrong, and live or ghost previews of the resulting graph.
- **Large repositories must stay responsive.** Existing TUIs struggle with large monorepos, large
  diffs, slow external diff tools, preview panes that keep rendering after the cursor moves, and
  stale state after outside commands. Cancellable preview tasks and refresh discipline are not
  polish; they are table stakes.
- **Customization has to cover actions, not only colors.** Users ask for keymaps, command groups,
  custom commands with selected revision values, clipboard actions, template-aware log display, and
  theme/color overrides that preserve legibility.
- **Maintenance and jj compatibility matter.** Prior tools have broken across jj releases or forked
  when releases stalled. `jk` should prefer structured jj data where possible, isolate CLI parsing,
  test against jj config variants, and document compatibility gaps explicitly.
- **Docs and demos need to show day-to-day workflows, not only feature lists.** The strongest demos
  should be short stories: inspect a stack, search to a target, preview a rebase, recover through
  the op log, use a workspace, and see the jj command behind the action.

Related upstream discussion anchors:

- [jj-vcs/jj#8836](https://github.com/jj-vcs/jj/pull/8836): TUI for editing history.
- [jj-vcs/jj#5121](https://github.com/jj-vcs/jj/issues/5121): refreshable jj log.
- [jj-vcs/jj#2765](https://github.com/jj-vcs/jj/issues/2765): interactive log/op-log viewing.
- [jj-vcs/jj#6903](https://github.com/jj-vcs/jj/issues/6903): interactive command defaults.
- [jj-vcs/jj#7045](https://github.com/jj-vcs/jj/issues/7045): split TUI documentation confusion.
- [jj-vcs/jj#8947](https://github.com/jj-vcs/jj/pull/8947): terminal color compatibility.

## 3. Non-negotiable design principles

### 3.1 Command-shaped actions

Every action resolves to one of these:

- A jj command to render.
- A jj command to preview and run.
- A jj command sequence to preview and run.
- An internal view operation that is not a repo mutation, such as scrolling, marking, folding,
  filtering, switching a focused region, or opening an overlay.

Mutating actions always go through a command preview unless they are explicitly configured as
no-confirm.

### 3.2 Config-faithful presentation

`jk` should honor jj config by design. Especially:

- `[ui].default-command`
- `[revsets].log`
- `[revsets].log-graph-prioritize`
- `[revsets].op-diff-changes-in`
- `[templates].log`, `show`, `evolog`, `op_log`, `op_show`
- `[templates].log_node`, `op_log_node`
- `[template-aliases]`
- `[ui].graph.style`
- `[ui].log-word-wrap`
- diff format and diff colors
- editor, diff editor, merge tool
- immutable revset behavior
- bookmark/tag ordering and push/fetch defaults

Native rendering is allowed when it improves interaction, but it must preserve these config
expectations or clearly document the gap.

### 3.3 Separate selection, scrolling, and marking

Object movement and viewport movement are different.

- `j/k`: move selected object in object screens.
- `Ctrl-j/Ctrl-k`: scroll viewport by one line without changing selection.
- `Ctrl-d/Ctrl-u`: half-page viewport scroll.
- `Ctrl-f/Ctrl-b`: full-page viewport scroll.
- `Space`: mark/unmark selected object in object screens.

Diff/details screens are text-reading screens first, so `j/k` scroll text there, while file/hunk
movement uses `[ ]` and `{ }`.

### 3.4 `?` is help everywhere

`?` opens generated contextual help. The help content and hotbar are generated from the active
keymap and view state. If a keybinding changes, help changes automatically.

Help rows should be ordered for the reader: primary view actions first, then mutation, recovery, and
command actions, then view/refresh controls, navigation, and close/help commands. The order should
not merely mirror implementation or dispatch registration order.

Help should also become searchable. The help surface and command palette are related but distinct:

- `?` explains what is currently possible.
- typing inside help filters by action name, key, screen, and jj command family.
- `:` runs jj commands.
- command-mode suggestions may reuse the same action registry, but should not replace searchable
  help.

### 3.5 Safe mutation loop

All rewrite/destructive/network actions should follow:

1. Resolve object roles.
2. Build `JjCommandSpec`.
3. Show preview: command, object summary, consequences, affected marks.
4. `Enter` runs, `e` edits command, `y` copies command, `Esc` cancels.
5. Restore terminal if the command opens editor/diff-editor/merge-tool.
6. Refresh state after success.
7. Show result, command history entry, and operation-recovery actions.

Current implementation status: selected-revision `jj abandon REV`, parented `jj new`, and
selected-revision `jj edit REV` have the first mutation previews. Pressing `a` on the log previews
`jj abandon REV`; pressing `n` previews `jj new PARENT...`, using ordered marks as parents when
present and otherwise using the selected revision; pressing `e` previews `jj edit REV`. Enter runs
the command, successful runs refresh the graph, command history records the resulting operation id,
and the recovery footer surfaces undo/redo/operation/history actions. The direct `a` binding is a
dogfood shortcut until the long-term action-menu prefix exists.

### 3.6 Recovery is first-class

After every mutation:

- Show a short success/failure status.
- Offer `o` to open operation log.
- Offer `u` undo and `U` redo where valid.
- Offer command-history view.

### 3.7 Key collisions are solved by scope and prefix menus

Direct keys are reserved for navigation, safe inspection, common mode changes, and the most central
workflows. Collision-heavy jj verbs live in a visible action menu.

### 3.8 Focused workflows, not pane-first dashboards

`jk` should be a view-stack / focused-workflow TUI, not a permanent pane grid. The active screen
owns the interaction model. Secondary information appears as inline expansion, modal preview,
full-screen details, or temporary overlays when useful.

Preferred product language:

```text
screen
view
overlay
modal
preview
detail screen
view stack
focused flow
```

Good examples:

```text
Log / graph screen
  -> Enter opens show/details screen
  -> d opens diff screen
  -> R opens rebase destination-pick mode
  -> a opens action menu overlay
  -> : opens command mode overlay

Diff screen
  -> file/hunk navigation inside the same focused view
  -> optional file picker overlay
  -> optional full-screen stat/summary/name-only variant

Operation log screen
  -> Enter opens operation show
  -> d opens operation diff
  -> r/v open confirm overlays
```

Avoid making the core product shape feel like:

```text
left pane: log
right pane: diff
bottom pane: command log
tabs: files/bookmarks/config/etc.
```

An optional split preview can exist later, but it should be a focused view mode, not the default
mental model.

### 3.9 Responsive by construction

Interaction should stay responsive even when jj, a diff tool, or a large repository is slow. This is
a product principle, not only an optimization pass:

- expensive preview work should be cancellable when selection changes;
- long-running commands should show visible progress or a pending state;
- stale external state should be detected and explained where possible;
- slow diff rendering should not block graph navigation;
- refresh should be explicit, predictable, and tied to operation changes where possible.

## 4. Core mental model

The product model is:

```text
current screen + cursor object + ordered marks + active mode
  -> role resolver
  -> command spec or view action
  -> preview/render/run
  -> refresh and preserve state
```

### 4.1 Object families

`jk` must model these object types:

- **Revision/change:** change ID, commit ID, parents, bookmarks, tags, author, description, flags,
  conflict/empty/immutable/divergent state.
- **Operation:** operation ID, description, user, time, parents, current flag, snapshot flag,
  workspace.
- **File:** repo path, status/type, diff stats, file mode, conflict flag.
- **Hunk:** file path, hunk header/range, old/new spans, selected lines if supported.
- **Bookmark:** local/remote/tracking state, target revision, conflict/divergence state.
- **Tag:** local/remote, target revision.
- **Remote:** name, URL, push URL, tracked bookmarks.
- **Workspace:** workspace name, root path, current change, current commit, stale state, active
  flag, and relevant operation context.
- **Command:** argv, cwd/repo, stdin, execution mode, output, status, resulting operation.

### 4.2 Ordered marks

Marks are ordered by time of selection. This lets `jk` infer roles naturally:

- first mark = source/from
- second mark = destination/to
- cursor = implicit destination or fallback object

The role resolver must always show the resulting command before mutation. For inspection commands,
direct open is acceptable, but the title/status must still show the command.

### 4.3 Role resolution examples

Diff:

```text
cursor A, no marks      -> jj diff -r A
mark A, cursor B        -> jj diff --from A --to B
mark A, mark B          -> jj diff --from A --to B
marks A..D contiguous   -> jj diff -r 'A::D' or prompt
```

Show:

```text
cursor A                -> jj show A
marks A B C             -> jj show A B C
```

Rebase:

```text
mark A, cursor B, R     -> default preview jj rebase -b A -o B
source toggle b/s/r     -> --branch / --source / --revision
destination toggle o/A/B -> --onto / --insert-after / --insert-before
```

Squash:

```text
cursor A                -> jj squash -r A
mark A, cursor B        -> jj squash --from A --into B
marked files            -> append filesets
hunks selected          -> start with jj squash --interactive, later native hunk spec
```

Describe:

```text
m on A                  -> inline prompt -> jj describe -m "..." A
M on A                  -> external editor -> jj describe A
```

Push:

```text
P on bookmark X         -> jj git push --dry-run --bookmark X, then confirm without --dry-run
P with marks            -> dry-run selected bookmarks
```

## 5. Harmonized keymap

This is the proposed default keymap. It should be encoded as data and tested for conflicts. User
configuration may remap it later.

### 5.1 Global keys

| Key         | Meaning                                                                     |
| ----------- | --------------------------------------------------------------------------- |
| `?`         | Contextual help.                                                            |
| `Esc`       | Close overlay/prompt, clear transient mode/marks, or go back.               |
| `Backspace` | Go back one view in the view stack.                                         |
| `q`         | Quit the app, except when an overlay treats it as close.                    |
| `:`         | jj command mode, `jj` prefix optional.                                      |
| `!`         | External command mode, explicit non-shell command unless user runs `sh -c`. |
| `r`         | Refresh current view.                                                       |
| `u`         | Undo, preview/confirm unless configured otherwise.                          |
| `U`         | Redo, preview/confirm unless configured otherwise.                          |
| `Tab`       | Cycle focused regions within the active screen, if that screen has regions. |
| `V`         | Reusable View Options overlay for display, diff, graph, and template flags. |
| `h/l`       | Collapse/open or fold/unfold where sensible.                                |
| `Ctrl-c`    | Emergency quit; restore terminal.                                           |

### 5.2 Navigation keys for object screens

Object screens include graph, op log, bookmarks, tags, remotes, command history, file lists, and
pickers.

| Key             | Meaning                                                                                |
| --------------- | -------------------------------------------------------------------------------------- |
| `j/k`, arrows   | Move selection down/up.                                                                |
| `Ctrl-j/Ctrl-k` | Scroll viewport one line without changing selection.                                   |
| `Ctrl-d/Ctrl-u` | Scroll half page down/up.                                                              |
| `Ctrl-f/Ctrl-b` | Scroll page down/up.                                                                   |
| `g g`           | First item.                                                                            |
| `G`             | Last item.                                                                             |
| `z z`           | Center selection.                                                                      |
| `z t`           | Put selection at top.                                                                  |
| `z b`           | Put selection at bottom.                                                               |
| `/`             | Search/filter prompt.                                                                  |
| `Ctrl-n/Ctrl-p` | Next/previous search match.                                                            |
| `Space`         | Mark/unmark selected object.                                                           |
| `c`             | Clear marks when marks exist; otherwise screen-specific commit/create action if valid. |
| `y`             | Yank/copy ID/revset/path/command for current object.                                   |

Rationale: `n` remains available for `new`; search repeat uses `Ctrl-n/Ctrl-p` rather than stealing
`n/N`.

### 5.3 Revision graph / main workbench

| Key     | Meaning                                                                                  |
| ------- | ---------------------------------------------------------------------------------------- |
| `Enter` | Open selected revision show/details.                                                     |
| `p`     | Open/toggle temporary preview or detail view for selected object.                        |
| `d`     | Diff selected/marked revisions.                                                          |
| `S`     | Stat view for selected/marked revisions.                                                 |
| `v`     | Evolution log for selected revision: `jj evolog`.                                        |
| `s`     | Status screen.                                                                           |
| `f`     | Revset/filter input with completion.                                                     |
| `n`     | New change from cursor/marks: `jj new ...`.                                              |
| `N`     | New change with inline message.                                                          |
| `c`     | Commit working-copy change when selected/current object is `@`: `jj commit`.             |
| `e`     | Edit selected revision: `jj edit <rev>`; preview if immutable or surprising.             |
| `m`     | Inline describe/message: `jj describe -m ... <rev>`.                                     |
| `M`     | Describe in editor: `jj describe <rev>`.                                                 |
| `R`     | Rebase wizard/destination-pick mode.                                                     |
| `a`     | Action menu for rewrite/content/destructive commands.                                    |
| `o`     | Operation log.                                                                           |
| `B`     | Bookmarks screen.                                                                        |
| `T`     | Tags screen.                                                                             |
| `W`     | Workspaces screen.                                                                       |
| `F`     | Fetch screen/default fetch.                                                              |
| `P`     | Push screen, dry-run first.                                                              |
| `V`     | View Options overlay for graph/list, template, patch, stat, and diff display flags.      |
| `@`     | Jump to working-copy commit.                                                             |

### 5.4 Action menu from revision graph

The `a` prefix opens a visible overlay. Keys are active only while the overlay is open.

Current implementation note: before this prefix menu exists, `a` directly previews
`jj abandon REV` for the selected log revision. Keep that implementation path command-spec based so
it can move under `a a` later without rewriting the mutation runner.

Current implementation note: `n` directly previews `jj new PARENT...` from the log. Ordered marks
become parents when present; otherwise the selected revision is the parent. Search-next remains
scoped to diff and rendered inspection views.

Current implementation note: `e` directly previews `jj edit REV` from the log. The same key still
reopens the command prompt when a command-output view is active.

| Key | jj command family               | Notes                                                      |
| --- | ------------------------------- | ---------------------------------------------------------- |
| `a` | `jj abandon`                    | Confirm; explain descendant behavior.                      |
| `b` | `jj absorb`                     | Review with `jj op show -p` after success.                 |
| `s` | `jj squash`                     | Source/destination resolver; filesets if file marks exist. |
| `S` | `jj split`                      | Uses configured diff editor initially.                     |
| `r` | `jj restore`                    | File/hunk/revision aware; confirm.                         |
| `v` | `jj revert`                     | Confirm destination.                                       |
| `d` | `jj diffedit`                   | External diff editor flow.                                 |
| `f` | `jj fix`                        | Preview and post-op review.                                |
| `m` | `jj metaedit`                   | Metadata editor/options.                                   |
| `M` | Merge/new-with-multiple-parents | `jj new <target> <source>` role picker.                    |
| `D` | `jj duplicate`                  | Confirm duplicates/targets.                                |
| `p` | `jj parallelize`                | Preview.                                                   |
| `P` | `jj simplify-parents`           | Preview.                                                   |
| `R` | `jj resolve`                    | Conflict resolver/tool flow.                               |
| `E` | `jj edit --ignore-immutable`    | Confirm strongly.                                          |

### 5.5 Diff/show/details screen

| Key                                         | Meaning                                                                |
| ------------------------------------------- | ---------------------------------------------------------------------- |
| `j/k`, arrows                               | Scroll one line.                                                       |
| `Ctrl-d/Ctrl-u`                             | Scroll half page.                                                      |
| `Ctrl-f/Ctrl-b`, `Space/PageDown`, `PageUp` | Page down/up.                                                          |
| `g g` / `G`                                 | Top/bottom.                                                            |
| `[ / ]`                                     | Previous/next file.                                                    |
| `{ / }`                                     | Previous/next hunk.                                                    |
| `h/l`                                       | Fold/unfold current file.                                              |
| `- / +`                                     | Fold/unfold current hunk.                                              |
| `Ctrl-left/Ctrl-right`                      | Fold/unfold all files.                                                 |
| `< / >`                                     | Horizontal scroll.                                                     |
| `/`, `Ctrl-n`, `Ctrl-p`                     | Search.                                                                |
| `Space`                                     | Mark/unmark file or hunk when focus is on a selectable section.        |
| `f`                                         | File picker/jump overlay.                                              |
| `V`                                         | View Options overlay.                                                  |
| `S`                                         | Toggle/open stat view.                                                 |
| `o`                                         | Open full diff for current file/path from a focused summary or picker. |
| `O`                                         | Open selected file in external editor.                                 |
| `a`                                         | File/hunk action menu: squash, split, restore, diffedit.               |
| `Backspace/Esc`                             | Return to previous screen.                                             |

### 5.6 Status screen

| Key     | Meaning                                     |
| ------- | ------------------------------------------- |
| `Enter` | Open selected file diff.                    |
| `Space` | Mark/unmark file.                           |
| `c`     | Commit selected/all working-copy changes.   |
| `n`     | New change.                                 |
| `m/M`   | Describe working-copy change inline/editor. |
| `a s`   | Squash selected files.                      |
| `a S`   | Split selected files.                       |
| `a r`   | Restore selected files.                     |
| `a d`   | Diffedit selected files.                    |
| `B`     | Bookmark current working-copy change.       |

### 5.7 Operation log screen

| Key     | Meaning                                                                        |
| ------- | ------------------------------------------------------------------------------ |
| `Enter` | `jj op show <op>`.                                                             |
| `d`     | `jj op diff` for selected operation.                                           |
| `S`     | Stat operation diff/show.                                                      |
| `V`     | View Options overlay.                                                          |
| `l`     | Open revision graph at operation: `jj --at-op=<op> log --ignore-working-copy`. |
| `s`     | Status at operation.                                                           |
| `r`     | `jj op restore <op>`, confirmed.                                               |
| `v`     | `jj op revert <op>`, confirmed.                                                |
| `u/U`   | Undo/redo from current operation context.                                      |
| `y`     | Copy operation ID or command.                                                  |

### 5.8 Bookmarks screen

| Key     | Meaning                                    |
| ------- | ------------------------------------------ |
| `Enter` | Show target revision.                      |
| `d`     | Diff target revision.                      |
| `c`     | Create bookmark at cursor/target revision. |
| `s`     | Set bookmark to revision.                  |
| `m`     | Move bookmark.                             |
| `a`     | Advance bookmark.                          |
| `x`     | Delete bookmark, confirmed.                |
| `f`     | Forget bookmark.                           |
| `r`     | Rename bookmark.                           |
| `t`     | Track remote bookmark.                     |
| `T`     | Untrack bookmark.                          |
| `P`     | Push selected bookmark, dry-run first.     |
| `F`     | Fetch selected/tracked remote.             |
| `Space` | Mark/unmark bookmark.                      |

Rationale: `u` remains global undo; untrack uses `T`.

### 5.9 Tags screen

| Key     | Meaning                |
| ------- | ---------------------- |
| `Enter` | Show target revision.  |
| `d`     | Diff target revision.  |
| `s`     | Set tag.               |
| `x`     | Delete tag, confirmed. |
| `Space` | Mark/unmark tag.       |
| `y`     | Copy tag name/revset.  |

### 5.10 Git/fetch/push screens

| Key     | Meaning                                    |
| ------- | ------------------------------------------ |
| `F`     | Fetch; if ambiguous, open chooser.         |
| `P`     | Push; always dry-run first where possible. |
| `Enter` | Run selected dry-run/actual command.       |
| `e`     | Edit command before running.               |
| `r`     | Refresh remote/bookmark state.             |
| `B`     | Jump to bookmarks.                         |
| `R`     | Remote management screen.                  |

### 5.11 Workspaces screen

Workspaces are early core scope, not an advanced-only feature. They should be a focused screen and
picker reachable from anywhere with `W`, not a permanent sidebar.

The exact command coverage should start conservative and jj-shaped:

```text
jj workspace list
jj workspace add
jj workspace forget
jj workspace root
jj workspace update-stale
```

The current implementation covers list, selected-workspace log, selected-workspace status,
selected-workspace diff, refresh, and update-stale. The screen should answer:

```text
What workspaces exist?
Which one am I in?
What revision/change is each workspace on?
Which workspaces are stale?
Which workspace has uncommitted/current work?
What happens if I switch context or update stale state?
```

| Key             | Meaning                              |
| --------------- | ------------------------------------ |
| `j/k`           | Move workspace selection.            |
| `Ctrl-j/Ctrl-k` | Scroll without changing selection.   |
| `Enter`         | Choose or inspect workspace.         |
| `n`             | Create workspace.                    |
| `u`             | Update stale workspace.              |
| `f`             | Forget workspace, confirmed.         |
| `s`             | Status for workspace.                |
| `d`             | Diff for workspace.                  |
| `o`             | Operation log relevant to workspace. |
| `a`             | Workspace actions.                   |
| `?`             | Help.                                |
| `Esc`           | Back.                                |

### 5.12 Command mode

`:` opens jj command mode. The user types `log -r 'mine()'`, not necessarily `jj log ...`. `jk`
should accept either.

Rules:

- Run without a shell by default.
- Preserve terminal when needed.
- Reload current state after success.
- Keep output/error panel visible after failure.
- Record command history.

`!` opens external command mode.

Rules:

- Also run without a shell by default.
- Tell the user to use `sh -c '...'` when they need pipes, redirects, built-ins, or `&&`.
- Do not interpret `!` commands as jj commands.

## 6. Screen model and view transitions

Use a view stack, not hard-coded `H/L` return paths.

```text
Workbench / graph
  -> Show/details
  -> Diff
  -> Status
  -> Operation log
      -> Operation show
      -> Operation diff
      -> Graph at operation
  -> Bookmarks
      -> Bookmark target show/diff
  -> Tags
      -> Tag target show/diff
  -> Workspaces
      -> Workspace status/diff/op log
  -> Git fetch/push/remotes
  -> Command history
  -> Command preview
  -> External command runner
```

Navigation contract:

- `Backspace`: pop one view.
- `Esc`: close transient state first, then pop view if no transient state.
- `q`: quit app, not back, except overlays may close on q.
- View stack preserves selection, scroll, marks, and filter where sensible.

## 7. Feature design by jj command family

### 7.1 Inspection commands

| jj command                          | jk screen/action      | Priority | Notes                                                                              |
| ----------------------------------- | --------------------- | -------- | ---------------------------------------------------------------------------------- |
| `jj log`                            | Workbench graph       | P0       | Default screen. Honor `ui.default-command`, `revsets.log`, templates, graph style. |
| `jj diff`                           | `d`, Diff screen      | P0       | Support `-r`, `--from`, `--to`, filesets, display modes.                           |
| `jj show`                           | `Enter`, Show/details | P0       | Metadata + diff; View Options.                                                     |
| `jj status`                         | `s`, Status screen    | P0       | Working copy, conflicts, bookmarks, file list.                                     |
| `jj interdiff`                      | `I` later or evolog   | P1       | Useful for comparing evolution of a change; marks map to from/to.                  |
| `jj evolog`                         | `v`                   | P1       | Evolution history; useful for split/divergence.                                    |
| `jj file list/show/annotate/search` | File/detail screens   | P1       | Add after the file model and selectors are stable.                                 |

### 7.2 Revision and history edit commands

| jj command            | jk action                   | Priority | Notes                                               |
| --------------------- | --------------------------- | -------- | --------------------------------------------------- |
| `jj describe`         | `m/M`                       | P0       | Inline and editor flows.                            |
| `jj metaedit`         | `a m`                       | P2       | Advanced metadata.                                  |
| `jj new`              | `n/N`                       | P0       | From cursor/marks as parents; with/without message. |
| `jj commit`           | `c` where `@`/status valid  | P0       | Working-copy flow.                                  |
| `jj edit`             | `e`                         | P0/P1    | Direct but clear; `a E` for ignore immutable.       |
| `jj rebase`           | `R`                         | P0       | Visual role picker.                                 |
| `jj abandon`          | `a` now, `a a` later        | P0/P1    | Destructive preview and operation recovery.         |
| `jj revert`           | `a v`                       | P2       | Destination picker.                                 |
| `jj duplicate`        | `a D`                       | P2       | Preserve command preview.                           |
| `jj parallelize`      | `a p`                       | P2       | Stack cleanup.                                      |
| `jj simplify-parents` | `a P`                       | P2       | Stack cleanup.                                      |
| `jj prev/next`        | command mode or future goto | P3       | Lower priority; conflicts with app navigation.      |

### 7.3 Content movement and file/hunk commands

| jj command                    | jk action                | Priority | Notes                                                         |
| ----------------------------- | ------------------------ | -------- | ------------------------------------------------------------- |
| `jj squash`                   | `a s`                    | P0       | Revision, file, and later hunk-aware.                         |
| `jj split`                    | `a S`                    | P0       | Start with diff editor; later non-interactive file/hunk flow. |
| `jj restore`                  | `a r`                    | P0       | Revision/file/hunk-aware; confirm.                            |
| `jj diffedit`                 | `a d`                    | P1       | External diff editor flow.                                    |
| `jj absorb`                   | `a b`                    | P1       | Post-op `jj op show -p` review.                               |
| `jj fix`                      | `a f`                    | P1       | Integrate with config-defined fix tools.                      |
| `jj resolve`                  | `a R`                    | P1       | Conflict list and external merge-tool flow.                   |
| `jj file track/untrack/chmod` | status/files action menu | P1       | Scoped to file screens.                                       |

### 7.4 Operation log commands

| jj command                | jk action         | Priority | Notes                   |
| ------------------------- | ----------------- | -------- | ----------------------- |
| `jj op log`               | `o`               | P0       | First-class screen.     |
| `jj op show`              | `Enter` in op log | P0       | Show operation changes. |
| `jj op diff`              | `d` in op log     | P0       | Compare operations.     |
| `jj op restore`           | `r` in op log     | P0/P1    | Strong confirmation.    |
| `jj op revert`            | `v` in op log     | P0/P1    | Strong confirmation.    |
| `jj undo`                 | `u`               | P0       | Preview/confirm.        |
| `jj redo`                 | `U`               | P0       | Preview/confirm.        |
| `jj op abandon/integrate` | op action menu    | P2       | Advanced maintenance.   |

### 7.5 Bookmarks, tags, Git

| jj command                                       | jk screen/action          | Priority | Notes                                   |
| ------------------------------------------------ | ------------------------- | -------- | --------------------------------------- |
| `jj bookmark list`                               | `B`                       | P0       | Local/remote/tracking/conflict state.   |
| `jj bookmark create/set/move/advance`            | Bookmark screen           | P0       | Role picker for target revision.        |
| `jj bookmark delete/forget/rename/track/untrack` | Bookmark screen           | P0       | Scoped keys.                            |
| `jj tag list/set/delete`                         | `T`                       | P1       | Similar to bookmarks.                   |
| `jj git fetch`                                   | `F`                       | P0       | Default direct, chooser when ambiguous. |
| `jj git push`                                    | `P`                       | P0       | Dry-run first.                          |
| `jj git remote list/add/remove/rename/set-url`   | Git remote screen         | P1       | Remote setup and repair.                |
| `jj git import/export`                           | Git action menu           | P1/P2    | Useful in colocated repos.              |
| `jj git clone/init`                              | Welcome/onboarding screen | P1/P2    | Useful outside existing repositories.   |

### 7.6 Workspaces, sparse, config, signing, bisect, Gerrit

Workspaces should be available through command mode early and should get a dedicated screen before
advanced history surgery is considered complete. Sparse, config, signing, bisect, and Gerrit can
remain command-mode-first until the core workflows are mature.

| Family           | Roadmap                                                               |
| ---------------- | --------------------------------------------------------------------- |
| `jj workspace`   | P0 dedicated workspace screen with list/add/forget/root/update-stale. |
| `jj sparse`      | P2 sparse screen; command mode first.                                 |
| `jj config`      | P2 config inspection, P3 config editor integration.                   |
| `jj sign/unsign` | P3 action menu.                                                       |
| `jj bisect`      | P3 workflow screen.                                                   |
| `jj gerrit`      | P3 optional integration; command mode first.                          |
| forge/tickets/AI | P3+; not part of core jj-native promise.                              |

## 8. Architecture plan

### 8.1 Current useful foundation

The current repo already has good boundaries:

- `jk`: binary crate and terminal lifecycle.
- `jk-cli`: jj process integration.
- `jk-core`: shared records and data types.
- `jk-tui`: Ratatui state/rendering/input actions.

Keep that separation, but expand the model.

### 8.2 Target crate map

Proposed eventual workspace:

```text
crates/jk              binary, CLI args, terminal lifecycle, event loop
crates/jk-core         object IDs, app models, command specs, keymap types
crates/jk-jj           jj integration: CLI provider, config loading, command runner
crates/jk-render       rendering backends: jj-rendered, native graph, native diff helpers
crates/jk-tui          Ratatui views, state machines, layout, help/hotbar
crates/jk-test         fixtures, fake providers, golden-test helpers
```

This does not need to happen immediately. Split crates only when boundaries are proven.

### 8.3 Command spec

All repo-affecting actions must go through a typed command spec.

```rust
struct JjCommandSpec {
    argv: Vec<OsString>,
    cwd: Option<PathBuf>,
    repository: Option<PathBuf>,
    stdin: Option<String>,
    mode: ExecutionMode,
    title: String,
    explanation: CommandExplanation,
    refresh_plan: RefreshPlan,
    safety: SafetyClass,
}

enum ExecutionMode {
    RenderReadOnly,
    ConfirmMutation,
    ConfirmExternalTool,
    DryRunThenConfirm,
    CommandMode,
}

enum SafetyClass {
    ReadOnly,
    LocalMetadata,
    LocalRewrite,
    DestructiveLocal,
    NetworkRead,
    NetworkWrite,
    ExternalCommand,
}
```

Rules:

- Read-only specs can run immediately and render a view.
- Mutating specs show preview.
- External-tool specs restore terminal and run foreground.
- Network-write specs dry-run first when jj supports it.
- Command history records every spec.

### 8.4 Provider abstraction

```rust
trait RepositoryProvider {
    fn load_graph(&self, query: GraphQuery) -> Result<GraphModel>;
    fn load_status(&self) -> Result<StatusModel>;
    fn load_diff(&self, query: DiffQuery) -> Result<DiffModel>;
    fn load_show(&self, query: ShowQuery) -> Result<ShowModel>;
    fn load_operations(&self, query: OpQuery) -> Result<OpModel>;
    fn load_bookmarks(&self) -> Result<BookmarkModel>;
    fn load_tags(&self) -> Result<TagModel>;
    fn load_remotes(&self) -> Result<RemoteModel>;
}
```

Implementation stages:

1. CLI-backed provider using jj subprocesses and template side channels.
2. Hybrid provider using jj CLI for presentation-critical output and structured templates for
   semantics.
3. Native provider using jj crates where stable enough.

Do not reimplement jj's template language from scratch unless absolutely necessary. Prefer using
jj's own crates or `jj` itself as the evaluator.

### 8.5 Renderer abstraction

```rust
enum RenderBackend {
    JjRendered,
    Native,
    Hybrid,
}

trait ViewRenderer {
    fn render_graph(&self, model: &GraphModel, state: &GraphViewState) -> FrameModel;
    fn render_diff(&self, model: &DiffModel, state: &DiffViewState) -> FrameModel;
    fn render_op_log(&self, model: &OpModel, state: &OpViewState) -> FrameModel;
}
```

Rendering roadmap:

- **JjRendered:** preserve current behavior; jj renders graph/diff; jk aligns rows and adds
  interaction.
- **Hybrid:** jj evaluates templates into semantic/display cells; jk owns layout, selection, hotbar,
  line scrolling, marks, folding.
- **Native:** jk owns graph layout/rendering while honoring jj graph style/node templates and
  labels.

### 8.6 Keymap as data

The keymap should be data-driven:

```rust
struct KeyBinding {
    mode: ModeId,
    sequence: Vec<KeyStroke>,
    action: ActionId,
    label: &'static str,
    help: &'static str,
    visibility: VisibilityRule,
}
```

Tests:

- No conflicting default bindings within a mode.
- Every visible hotbar action has help text.
- Every help action maps to an implemented action.
- Every mutation action has a command spec and preview path.

### 8.7 View stack and mode stack

Maintain separate stacks:

- View stack: graph -> diff -> op log -> op show, etc.
- Mode stack: normal -> search -> action menu -> command preview.

`Esc` pops modes before views. `Backspace` pops views.

### 8.8 External tool runner

For editor/diff-editor/merge-tool commands:

1. Save app state.
2. Leave alternate screen / restore terminal.
3. Run command in foreground.
4. Re-enter alternate screen.
5. Refresh according to `RefreshPlan`.
6. Show success/failure with output and command history.

### 8.9 Command history

Command history should be a model and a screen.

Fields:

```text
id
time
cwd/repository
argv
stdin summary
execution mode
started_at / duration
exit status
stdout/stderr snippets
full output path if captured
operation id after success if discoverable
triggering action/key
```

User actions:

- `Enter`: open output.
- `y`: copy command.
- `Y`: copy command + output.
- `o`: open resulting operation when known.
- `r`: rerun, confirmed.

## 9. Detailed workflow specs

### 9.1 Diff workflow

From graph:

- `d` with no marks: `jj diff -r <cursor>`.
- `d` with one mark and cursor: `jj diff --from <mark> --to <cursor>`.
- `d` with two marks: `jj diff --from <mark0> --to <mark1>`.
- `S` uses same resolver plus `--stat`.
- `V` opens View Options.

Diff view:

- Preserve selected file/hunk across refresh if path/header still exists.
- Sticky current-file header.
- File picker overlay for large diffs. The current implementation includes the first `f` file
  selector for jumping within an active diff; the later target is searchable and shared with the
  broader fileset selector model.
- Search highlights and next/previous match navigation.
- Next/previous file navigation without reopening the picker.
- Toggle between patch, stat, summary, name-only, and file-only details.
- Keep revision metadata visible enough that users know which change/range they are inspecting.
- File/hunk marks feed squash/split/restore/diffedit.

Diff workflow should grow in three layers:

1. **jj-shaped inspection:** render `jj diff`, `jj show`, and `jj status` faithfully, with cursor and
   marks resolving to visible commands.
2. **File-centric navigation:** add a searchable file list, sticky revision/file context,
   next/previous file commands, and quick format toggles.
3. **Repository-at-revision browsing:** let users inspect the file tree for a selected revision,
   then jump from a file to its diff or full content without switching to another tool.

Range and comparison behavior should be explicit:

- one revision means `jj diff -r REV`;
- two ordered revisions mean `jj diff --from A --to B`;
- selected contiguous ranges may offer a range revset preview;
- non-contiguous selections should prompt or use an action menu rather than guessing.

Large diffs should never make graph navigation feel broken. If an expensive diff preview is still
rendering when selection changes, cancel it or mark it stale before starting the next preview.

Diff follow-up work should keep these details visible as separate reviewable slices:

- searchable file-list filtering once the shared fileset/file selector exists;
- horizontal overflow controls for wide diffs, with status text showing the shifted column;
- empty-diff and failed-load states that stay retryable inside the TUI instead of exiting before the
  user can recover;
- edge-case fixtures for binary files, renames, copies, conflicts, mode changes, symlinks, file
  permission changes, empty diffs, and very wide lines;
- clearer current-hunk markers and fold-state indicators for files and hunks;
- copy actions for path, hunk location, or selected line text after selection semantics are stable;
- optional mouse selection and wheel scrolling only after keyboard behavior is complete.

### 9.2 Show/details workflow

`Enter` opens `jj show <rev>` in details view.

Show view should include:

- Header metadata from configured template.
- Diff body with file/hunk navigation.
- Action menu scoped to selected revision/file/hunk.
- Toggle between inline details and fullscreen details.

### 9.3 Rebase workflow

`R` opens rebase mode.

Default role inference:

```text
source = first mark or cursor or @
destination = cursor if source is marked, otherwise prompt/picker
source mode = branch (-b)
destination mode = onto (-o)
```

UI:

- Highlight source revisions.
- Highlight destination candidate.
- Show ghost/preview summary if possible.
- Support search and jump while choosing a far-away destination.
- Preserve enough context that users can return to the source after previewing destinations.
- Footer shows role toggles:

```text
source: b branch / s source / r revision
dest: o onto / A after / B before
Enter preview   Esc cancel
```

Preview:

```text
jj rebase -b A -o B
```

Advanced:

- Multiple sources supported by repeating args where jj supports it.
- Multiple destinations supported for merges where jj supports repeated `-o`.
- Insert-before/insert-after should detect obviously inverted child/parent selection order and
  either swap roles safely or ask for confirmation.
- Multi-parent change creation should share the same role-picker vocabulary as rebase so users do
  not learn a separate model for merge-like changes.
- If source/destination invalid, show jj stderr or preflight validation.

Long-term rebase preview should be visual:

- render the current graph and the proposed result side by side only as a focused preview mode, not
  as the default app layout;
- show added, moved, and unchanged nodes distinctly;
- keep the exact jj command visible;
- make `Enter` run and `Esc` cancel;
- preserve the pre-run operation recovery path after success.

### 9.4 Describe/message workflow

`m` opens inline description prompt.

Prompt keys:

- `Enter`: run `jj describe -m <message> <rev>`.
- `Ctrl-e`: open editor with current text as seed if supported.
- `Ctrl-u`: clear the current inline prompt text.
- `Esc`: cancel.
- `Ctrl+s`: save when multiline editor is active.

Current implementation status: `m` opens the inline prompt prefilled with the selected revision's full
description, and `Ctrl-u` clears the prefilled text before preview. The prompt still submits through
command preview, command history, operation-id capture, log refresh, and recovery footer. It does
not yet support multiline editing, editor handoff, or a before/after review panel.

`M` opens configured editor directly:

```text
jj describe <rev>
```

After success, refresh graph and preserve selected change if still visible.

### 9.5 New/commit/edit workflow

`n`: create new change.

Role inference:

- If marks exist: marks are parents.
- Else cursor is parent.
- If cursor is absent: default jj behavior.

Current implementation status: direct `n` implements the marks-or-cursor parent rule and routes
through command preview, command history, operation-id capture, log refresh, and recovery footer. It
does not yet support inline messages or the later role-resolver overlay.

`N`: create new change with inline message.

`c`: commit working-copy change, only visible when current context is working-copy/status.

`e`: edit selected revision:

```text
jj edit <rev>
```

If revision is immutable or hidden, require stronger preview or route to action menu variant.

Current implementation status: direct `e` implements the selected-revision preview path and routes
through command preview, command history, operation-id capture, log refresh, and recovery footer. It
does not yet add immutable-specific warning copy or the stronger `a E` ignore-immutable variant.

### 9.6 Squash/split/restore/absorb

Initial implementation should delegate hunk-level editing to jj's configured diff editor. File-level
selection can be non-interactive earlier.

Squash:

- Revision context: `jj squash -r <rev>` or `jj squash --from <source> --into <dest>`.
- File marks: append filesets.
- Hunk marks: use `--interactive` until native hunk apply exists.

Split:

- Revision context: `jj split -r <rev>`.
- File marks: `jj split -r <rev> <filesets...>`.
- Hunk marks: use `--interactive` initially.

Restore:

- Revision/file context: `jj restore --from <from> --into <into> <filesets...>`.
- Prompt clearly, because restore can discard content.

Absorb:

- `jj absorb --from <rev>` or selected filesets.
- After success, show `jj op show -p` action because absorb may distribute changes across ancestors.

### 9.7 Operation recovery workflow

`o` opens op log.

After every mutation, show recovery footer:

```text
Ran: jj squash --from A --into B
u undo   o op log   p op show -p   y copy command
```

Op log actions:

- `Enter`: op show.
- `d`: op diff.
- `r`: restore selected op, strong confirmation.
- `v`: revert selected op, strong confirmation.
- `l`: inspect graph at op.

### 9.8 Bookmarks/tags workflow

Bookmarks screen should clearly distinguish:

- local bookmark
- remote bookmark
- tracked bookmark
- conflicted/diverged bookmark
- target revision

Moving or deleting a bookmark should not be confused with abandoning revisions. Confirm text should
say that bookmark delete does not abandon revisions.

Push from bookmarks:

- `P` dry-runs selected bookmarks.
- Confirm pushes actual bookmarks.

Tags are similar but smaller.

### 9.9 Git fetch/push workflow

Fetch:

- `F` runs default fetch if unambiguous.
- If multiple remotes or marked bookmarks/remotes, open fetch screen.
- Options should include tracked-only, specific bookmark, all remotes.

Push:

- `P` always opens push preview.
- Use `jj git push --dry-run ...` first where possible.
- Show remote, bookmarks, new/deleted bookmarks, and safety notes.
- `Enter` runs actual push.
- Failed push should suggest fetch/resolve bookmark conflicts where relevant.

### 9.10 Command mode workflow

`:` command examples:

```text
:log -r 'mine() & mutable()'
:diff --from main --to @
:rebase -b @ -o main
:git push --dry-run --bookmark feature-x
:op log --stat
```

Rules:

- `jk` should parse enough to know whether output should become a view, a command-output panel, or a
  mutation preview.
- Unknown commands still run through `jj` and show output.
- Mutating commands typed manually can either run after confirmation or be governed by a setting.

## 10. CLI surface for `jk`

`jk` CLI should mirror jj-shaped args whenever possible.

Canonical commands:

```text
jk
jk log [jj log args...]
jk diff [jj diff args...]
jk show [jj show args...]
jk status [filesets...]
jk op log [jj op log args...]
jk op show [jj op show args...]
jk op diff [jj op diff args...]
jk bookmark list [args...]
jk tag list [args...]
jk workspace list [args...]
jk git fetch [args...]
jk git push [args...]
```

Important correction from current behavior:

- `jk diff <revision>` can remain compatibility sugar.
- Canonical shape should be `jk diff -r <revision>`, `jk diff --from A --to B`, and `jk diff --stat
  --from A --to B` because jj's positional diff arguments are filesets.

Global options:

```text
-R, --repository <path>
--at-op <operation>
--ignore-working-copy
--config <KEY=VALUE> / --config-file maybe passthrough later
--renderer jj|hybrid|native
--no-watch
--debug-command-log
--tape/demo mode later
```

## 11. Configuration plan

Prefer jj config for user-visible `jk` settings, using `jk.*` keys, because jj users already manage
repo/user config there.

Example:

```toml
[jk]
renderer = "hybrid"
confirm-mutations = true
show-hotbar = true
command-history-limit = 500

[jk.keymap]
# later, schema-driven

[jk.ui]
preview-style = "overlay"
mouse = false
```

Use XDG config/state for app-only state:

```text
$XDG_CONFIG_HOME/jk/config.toml    optional app-local overrides
$XDG_STATE_HOME/jk/command-log     command history/output cache
$XDG_CACHE_HOME/jk                 cached expensive semantic loads if needed
```

Config precedence:

1. CLI flags.
2. Repo-local jk config.
3. User jk config.
4. jj config-derived defaults.
5. built-in defaults.

## 12. Rendering strategy

### 12.1 Phase A: rendered-output first

Keep current approach for stability:

- Run jj for rendered output.
- Run a semantic template pass for row identity and metadata.
- Align rendered lines to semantic records.
- Add selection/scroll/marks/hotbar outside the rendered body.

### 12.2 Phase B: hybrid rendering

Move to a row/cell model while still using jj to evaluate templates.

- Ask jj to emit structured JSON for fields `jk` needs.
- Ask jj to render user template fragments where fidelity matters.
- `jk` owns line wrapping, selection, marks, previews, and hotbar layout.

Structured data should be preferred whenever jj exposes it. Plain rendered output is still useful
for fidelity, but brittle string parsing should stay behind provider boundaries so jj compatibility
changes are isolated. When `json()` templates or other structured jj output can provide stable
identity, parentage, bookmarks, flags, and operation metadata, use that data for state and reserve
rendered text for presentation.

### 12.3 Phase C: native graph rendering

Native graph rendering is justified when it enables:

- Accurate line scrolling independent of selected row.
- Stable row identity.
- Multi-selection overlays.
- Rebase ghost previews.
- Section folding.
- Side-by-side graph/detail layouts.
- Mouse hit testing.

Acceptance criterion:

- For a fixture matrix of jj configs, native rendering must match jj output closely enough or
  clearly document differences.
- Graph style and node templates must be honored.
- Color labels and template aliases must be honored or evaluated by jj.

### 12.4 Diff rendering

Initial diff rendering can remain jj-rendered. Native diff enhancements may come later:

- File list with stats.
- Hunk selection.
- Syntax highlighting.
- Side-by-side diff.
- Conflict visualization.
- Sticky revision and file metadata.
- Searchable file list and inline diff search.
- Fast file-only/details mode for large changes.

Do not replace jj diff formatting until there is a strong reason and fidelity tests.

Diff rendering has a higher complexity ceiling than graph rendering because it touches diff
algorithms, word highlighting, syntax highlighting, split/diffedit behavior, conflict rendering,
binary files, and external diff tools. Treat native diff work as a series of narrowly scoped
enhancements, not a wholesale replacement.

### 12.5 jj integration cleanup

The current rendered-output bridge is acceptable only because it preserves jj's real CLI behavior
while the product is young. Treat it as a compatibility adapter, not as the desired end state.

The provider layer needs two different products from jj:

1. the exact terminal presentation jj would render for the user's config, template, graph style,
   terminal width, colors, revset defaults, default command behavior, and warnings;
1. semantic records for navigation, expansion, diff targeting, row identity, marks, and future
   mutation previews.

Those are not the same contract. `jj-lib` owns repository semantics, but it does not own the whole
CLI log presentation contract. Direct `jj-lib` integration is not a win if it forces `jk` to copy
`jj log` selection, template, graph, color, and warning behavior. In-process `jj-cli` integration is
worth a spike only if it reduces process overhead without changing product behavior.

Cleanup rules:

- keep rendered output opaque outside the provider boundary;
- keep semantic parsing narrow, fixture-backed, and version/config-drift aware;
- document parser failures in terms of the jj version or config contract that changed;
- prefer structured jj output where it gives stable identity, but do not make `jk` duplicate
  presentation behavior for aesthetic architecture reasons;
- revisit shell-out replacement only when the replacement reduces duplicated jj behavior;
- if semantic row access requires copying `cmd_log`, draft an upstream reusable log-core proposal
  instead of reimplementing jj log in `jk`.

## 13. Testing and quality gates

### 13.1 Unit tests

- Role resolver tests for every command shape.
- Keymap conflict tests.
- Command spec string/argv tests.
- View-stack/mode-stack tests.
- Selection/scroll/mark state tests.
- Diff file/hunk parser tests.

### 13.2 Integration tests

Use fixture jj repos for:

- Linear history.
- Merge history.
- Conflicts.
- Divergent changes.
- Empty changes.
- Hidden/abandoned changes.
- Immutable trunk/main.
- Bookmarks local/remote/tracked/conflicted.
- Tags.
- Multiple remotes.
- Operation log with undo/redo/revert/restore.
- Large repo/log performance fixture.

### 13.3 Config-fidelity tests

Fixture matrix:

- Default templates.
- Custom `templates.log`.
- Custom `templates.log_node`.
- `graph.style = curved/square/ascii/ascii-large`.
- `log-word-wrap = true/false`.
- custom `format_short_id` aliases.
- custom colors/labels.

Compare against jj output where possible. For native rendering, classify differences as
accepted/bug.

### 13.4 Betamax end-to-end TUI tests

[Betamax](https://www.joshka.net/betamax/) should be part of the `jk` product system, not just a
way to make GIFs. The source lives in [joshka/betamax](https://github.com/joshka/betamax). Every
important interaction should eventually have the same source artifact powering:

```text
developer regression tests
PR review artifacts
README/site demos
release smoke checks
user-facing workflow docs
```

[Betamax](https://www.joshka.net/betamax/) is a strong fit because it is Rust-first and tape-driven.
It runs commands in a PTY, feeds terminal output through `libghostty-vt`, rasterizes frames
in-process with `cosmic-text` and `swash`, and writes screenshots or animations. It should be
treated as both a capture tool and a terminal testing harness: tapes can run interactive terminal
programs, wait for expected screen text, capture screenshots, write structured terminal state, and
fail when expected output does not appear before a timeout.

The current repo already has the seed of this model:

```text
just betamax
just readme-media
just betamax-log
just betamax-diff
```

The README also says visual README media is generated with `just readme-media`. For unreleased
local work, generated media should stay under `target/dogfood-artifacts/`. Public README and
crates.io media is published later from the separate
[`joshka/jk-screenshots`](https://github.com/joshka/jk-screenshots) repository served from
`https://www.joshka.net/jk-screenshots/`. Website-specific media can live in `jk-website` when the
site is the only consumer. Keeping generated images and GIFs out of the main source repository is a
jj ergonomics decision as much as a repository-size decision: Git LFS-heavy media churn is easier to
manage in a purpose-built media or website repository. That setup should grow from "README media
helpers" into core project infrastructure.

Every meaningful `jk` workflow should eventually have two Betamax tapes:

```text
validation tape
  optimized for CI and regression confidence

review-media tape
  optimized for docs, website, release notes, and PR review
```

For `jk`, a user-visible feature is not done until it has:

- unit tests for state and command resolution;
- integration tests for jj command construction;
- a Betamax validation tape for the user journey;
- a Betamax review-media tape if it is user-visible enough for docs.

Suggested tape structure:

```text
tapes/
  validation/
    log-navigation.tape
    log-marks.tape
    diff-selected-revision.tape
    diff-from-to.tape
    show-details.tape
    command-mode.tape
    command-history.tape
    op-log.tape
    describe-inline.tape
    describe-editor.tape
    rebase-preview.tape
    workspace-list.tape
    bookmark-list.tape
    git-push-dry-run.tape

  media/
    readme-overview.tape
    docs-log-to-diff.tape
    docs-rebase-preview.tape
    docs-command-mode.tape
    docs-op-log-undo.tape
    docs-workspaces.tape
    release-highlight.tape

  fixtures/
    basic-repo.sh
    divergent-change-repo.sh
    bookmark-conflict-repo.sh
    multi-workspace-repo.sh
    conflict-repo.sh
    large-diff-repo.sh
```

Validation tapes should be short, deterministic, and assertion-heavy. Media tapes can be slower and
more readable. Review-media tapes should include readable dwell and, where useful, Betamax
presentation affordances such as keyboard overlays and captions. For `jk`, captions should read as
presentation metadata on the surrounding frame, not as terminal content, and must never cover the
TUI hotbar, footer, prompts, status text, or keyboard overlay. Captions and keyboard overlays should
occupy separate visual regions so they explain the action without competing with each other. Until
Betamax can guarantee that layout, `jk` media tapes should prefer keyboard overlays and avoid
captions.

Expand Betamax coverage for:

- Graph navigation and line scrolling.
- Marks and two-revision diff.
- Command preview.
- Rebase picker.
- Describe inline/editor fake.
- Op log restore/revert preview.
- Bookmarks screen.
- Push dry-run screen.
- Command mode.
- Help/hotbar generation.
- Workspace listing, stale workspace handling, and workspace-scoped status/diff.

#### 13.4.1 Validation assertions

Prefer semantic waits and structured state snapshots over sleeps. Good assertions include:

- expected hotbar actions for the current screen;
- generated help entries for the current screen;
- selected object identity;
- visible command preview;
- `Space` mark/unmark behavior;
- `Ctrl-j/Ctrl-k` scrolling without selection movement;
- command history entries;
- expected jj operation results.

Use Betamax's test-oriented primitives for this style of tape:

```text
Require
Wait+Screen
Screenshot
State
Output
Hide
```

Validation tapes should specifically assert flows like:

```text
The hotbar contains the expected actions for the current screen.
? opens generated help for the current screen.
Space marks/unmarks a revision.
Ctrl-j/Ctrl-k scroll without changing selection.
d opens the expected diff.
S opens stat mode.
: opens command mode.
R opens rebase preview, not an immediate mutation.
Enter on a command preview runs the command.
Esc cancels previews and clears marks in the expected order.
u/U show undo/redo confirmation or run safely according to policy.
```

The canonical machine-readable checkpoint should be Betamax State JSON. It is strict JSON designed
for snapshot testing: easy plain-text assertions, styled output when needed, and no repeated
per-cell default data. State JSON exposes:

- `viewport_text`;
- `scrollback_text`;
- styled viewport and scrollback spans;
- terminal size;
- cursor metadata;
- default style;
- non-default styles.

Avoid brittle assertions on elapsed timings, exact ANSI bytes, full graph layout in non-graph tests,
theme-dependent colors, unnormalized absolute paths, and unseeded random names.

Do not overfit validation tapes to volatile details unless the test is specifically about those
details. Prefer assertions on:

```text
visible command preview
selected object identity
screen title
hotbar actions
help entries
status/error text
presence of command in command history
expected jj operation result
```

Avoid assertions on:

```text
exact elapsed command timings
exact ANSI escape sequences
full graph layout for non-graph tests
theme-dependent color hex values
terminal size unless the workflow depends on it
random generated bookmark names unless seeded
absolute paths unless fixtures normalize them
```

#### 13.4.2 Fixture strategy

`jk` needs reproducible jj repositories for tapes. Each fixture script should:

```text
create a temp repo
run jj git init --colocate or jj init as appropriate
set deterministic jj config
create deterministic commits/bookmarks/workspaces
avoid timestamps in visible assertions where possible
print the repo path for the tape
```

Suggested fixtures:

```text
basic-repo
  few revisions, one bookmark, one working-copy diff

branchy-repo
  several mutable commits with merge/rebase opportunities

diff-repo
  multiple files, hunks, rename, binary/no-diff file if practical

workspace-repo
  multiple jj workspaces, one stale workspace, one active workspace

oplog-repo
  prepared operations suitable for op log, undo, redo, restore

conflict-repo
  conflict markers / conflicted revision for resolve flows

bookmark-repo
  local bookmark, remote-tracking bookmark, diverged bookmark

git-remote-repo
  local bare remote for fetch/push dry-run flows
```

Workspaces should be early in this suite, not advanced. Add `docs-workspaces.tape` as soon as `W`
exists.

#### 13.4.3 Betamax-driven documentation

The docs site should be generated around repeatable terminal stories. Each user guide should have:

```text
one short concept explanation
one jj command equivalent
one jk interaction sequence
one Betamax-rendered screenshot/GIF/video
one copyable command-mode equivalent where relevant
```

Example:

```text
Guide: Rebase a stack

CLI:
  jj rebase -b feature -o main

jk:
  Space mark source
  move to main
  R
  choose branch + onto
  Enter

Artifact:
  docs-rebase-preview.gif generated from tapes/media/docs-rebase-preview.tape
```

Docs should avoid hand-crafted screenshots. If a screenshot appears in the README or website, it
should be generated from a tape. Generated media should live outside the source tree as release or
site assets so it can render on GitHub and crates.io without storing generated media in this repo.

#### 13.4.4 PR review workflow

Every UI-changing PR should attach Betamax artifacts or link CI artifacts.

Recommended CI artifacts:

```text
target/dogfood-artifacts/betamax/validation/*.json
target/dogfood-artifacts/betamax/validation/*.png
target/dogfood-artifacts/betamax/media/*.gif
target/dogfood-artifacts/betamax/media/*.mp4, optional
```

PR template addition:

```markdown
## Betamax

- [ ] Added/updated validation tape
- [ ] Added/updated review-media tape, if user-visible
- [ ] Attached screenshots/GIFs or linked CI artifact
- [ ] Updated docs page that consumes the tape, if applicable
```

For Codex specifically: any change to input handling, view state, command preview, rendering, or
screen navigation should include a tape update unless the change is strictly internal and invisible.

#### 13.4.5 Release workflow

Release candidates should run a stable Betamax suite before publish.

Add release gates:

```text
just betamax-validation
just betamax-media-smoke
just release-check
```

Suggested `justfile` additions:

```make
betamax := env_var_or_default("BETAMAX", "betamax")

betamax-validation:
    {{betamax}} run tapes/validation/log-navigation.tape
    {{betamax}} run tapes/validation/diff-selected-revision.tape
    {{betamax}} run tapes/validation/command-mode.tape

betamax-media:
    {{betamax}} run tapes/media/readme-overview.tape
    {{betamax}} run tapes/media/docs-rebase-preview.tape

betamax-update:
    just betamax-validation
    just betamax-media
```

Use the installed `betamax` binary by default, and override `BETAMAX` when a local source checkout
or wrapper command is needed. Betamax currently supports installation through Homebrew and
`cargo-binstall`. Local development can still run basic smoke experiments from a Betamax checkout
with:

```sh
cargo run -- run examples/basic.tape
```

#### 13.4.6 Co-development contract between jk and Betamax

Because Betamax and `jk` are evolving together, `jk` should intentionally drive Betamax
requirements.

Likely Betamax feature pressure from `jk`:

```text
stable key input for Ctrl/Alt/Shift combinations
better terminal resize scripting
state JSON filtering / normalization helpers
snapshot comparison helpers
artifact manifest output
theme pinning for deterministic screenshots
font/layout stability across CI platforms
easy per-tape environment setup
first-class failed-checkpoint screenshot capture
caption and keyboard-overlay layout that avoids terminal content
```

Likely `jk` feature pressure from Betamax:

```text
deterministic demo mode
stable fixture repositories
optional animation-friendly delays/status text
test-only config overrides
screen titles and hotbar text that are easy to assert
consistent generated help text
command history export
```

This should be embraced. The projects can demonstrate how terminal tools can evolve with
browser-grade workflow testing, but with terminal-native artifacts.

Demo artifact rule:

```text
README GIF       <- tapes/media/readme-overview.tape
website hero     <- tapes/media/site-hero.tape
release GIF      <- tapes/media/release-vX.Y.tape
bug reproduction <- tapes/regressions/<issue>.tape
```

A demo should never be a separate manual artifact. It should be a tape.

If a bug report says "the UI did the wrong thing," the preferred regression artifact is a Betamax
tape that reproduces it.

### 13.5 Release gates

Minimum local gate:

```text
just release-check
```

Expand over time:

```text
just fixture-test
just betamax-validation
just betamax-media-smoke
just keymap-test
just config-fidelity-test
just package-smoke
```

CI should keep one stable required aggregate status, even as jobs split.

For the current repo, the required status baseline should stay explicit and boring:

- `Check`
- `Markdown`
- `MSRV`

If GitHub merge queue is enabled, it should run the same checks through `merge_group`. Keep the
Markdown check separate from Rust checks so docs and formatting failures stay quick to diagnose.

## 14. Performance plan

Performance goals:

- Startup under 100 ms for small repos after jj process overhead, or as close as practical.
- Smooth navigation independent of jj subprocess latency.
- Avoid reloading expensive views unless inputs changed.
- Preserve UI responsiveness during command execution with cancellable tasks.
- Keep graph navigation usable in large monorepos and histories with tens of thousands of changes.
- Keep large diffs and slow external diff tools from blocking unrelated navigation.

Implementation ideas:

- Async command runner with cancellation.
- Load current view first, then enrich side data.
- Cache parsed rendered snapshots by command/op/repo state where safe.
- Incremental refresh when operation ID changes.
- Optional file watcher/op watcher.
- Limit huge logs by default but make expansion easy.
- Use virtualized list rendering for large graphs.
- Debounce selection-driven previews and cancel stale preview work when the cursor moves.
- Prefer op-id-based refresh checks after mutating commands.
- Detect external repo state changes and explain when the user needs a refresh.
- Add a slow-command status line for operations that take longer than an interaction threshold.
- Include large-log, large-diff, slow-diff-tool, and external-change fixtures in validation.

Responsive behavior to test explicitly:

- moving through a graph while a previous diff preview is still rendering;
- opening a large change and immediately moving to another revision;
- running a mutation, refreshing by operation ID, and preserving cursor identity;
- editing the repo outside `jk`, then returning to a stale view;
- using a custom diff tool or template that makes preview rendering slow.

### 14.1 Auto-refresh policy

Auto-refresh belongs after the manual refresh model is solid in both log and selected-change diff
views. It should make editor, shell, and agent workflows smoother without turning repository writes
into flicker or focus theft.

Policy requirements:

- manual refresh remains available and predictable;
- auto-refresh has a debounce/coalescing window for editors, shells, and agents that write several
  files or operations in quick succession;
- selection, scroll, expansion, marks, and collapsed diff state follow the same preservation rules
  as manual refresh;
- refresh failures surface in status text or a retryable output view without stealing focus;
- the status bar makes refresh mode visible without adding noisy chrome;
- default/opt-in/configurable behavior is decided before enabling auto-refresh by default.

Tests should cover debounce timing, stale command results, external repo edits, disappeared selected
changes, and refresh while the user is scrolling or reading expanded details.

## 15. Safety and security

### 15.1 Mutation safety

Safety classes determine confirmation:

- Read-only: no confirmation.
- Local metadata: preview optional, but command visible.
- Local rewrite: preview required.
- Destructive local: preview + strong confirm.
- Network read: direct allowed if user initiated.
- Network write: dry-run/preview required where supported.
- External command: explicit preview.

Strong confirmation examples:

- `jj abandon`
- `jj op restore`
- `jj op revert`
- bookmark/tag delete
- restore file/hunks
- push actual remote updates
- external `!` commands if configured

### 15.2 Terminal safety

- Always restore terminal on panic/error.
- No shell interpolation unless `! sh -c` is explicit.
- Redact obvious tokens in command history output.
- Keep full raw output in state/cache only if user opts in or file permissions are safe.

## 16. Documentation and website plan

### 16.1 Documentation structure

Repository docs:

```text
README.md
CHANGELOG.md
CONTRIBUTING.md
SECURITY.md
CODE_OF_CONDUCT.md optional
LICENSE files already present
docs/
  product-plan.md
  architecture.md
  keymap.md
  command-mapping.md
  user-demand-signals.md
  workflows/
    inspect.md
    search-and-filter.md
    rebase.md
    describe.md
    squash-split-absorb.md
    operation-recovery.md
    bookmarks-tags.md
    fetch-push.md
    command-mode.md
  config.md
  rendering.md
  testing.md
  release.md
  adr/
    0001-command-spec.md
    0002-keymap-principles.md
    0003-rendering-strategy.md
```

### 16.2 README

README should stay concise:

- One-sentence positioning.
- Screenshot/GIF.
- Install commands.
- Current status.
- Five-minute tutorial.
- Key concepts: marks, command preview, `:`, operation recovery.
- Link to full docs.

### 16.3 Documentation modes

Keep one dominant documentation job per page:

- **README / crate README:** current behavior, install paths, short status, screenshots, and next
  links. Do not front-load maintainer roadmap depth here.
- **Quick start / tutorials:** teach one complete workflow by doing it, with the jj command
  equivalent and a Betamax-generated artifact.
- **Workflow guides:** help users complete tasks such as inspect, rebase, describe, squash/split,
  operation recovery, bookmarks/tags, fetch/push, command mode, and workspaces.
- **Reference pages:** state exact keys, commands, config keys, safety classes, and command mappings.
- **Explanation pages:** carry mental models, tradeoffs, compatibility principles, and prior-art
  comparison. Keep user-demand evidence factual and respectful: describe what users are trying to
  accomplish, not which tool is "wrong."
- **ADRs:** record durable decisions that change product direction, ownership boundaries,
  rendering strategy, command mode semantics, keymap policy, or stability guarantees.
- **Release docs:** state user-visible changes, keymap changes, config changes, safety changes,
  media updates, and known limitations.

Documentation that changes user-visible workflows should update examples, tests, fixtures, and
Betamax tapes in the same review unit where practical. When a follow-up is required, track it in the
roadmap or an issue rather than burying uncertainty in user docs.

### 16.4 Website

Website pages:

- Home: promise, GIF, install, why jk.
- Quick start: first five minutes.
- Workflows: inspect, search/filter, rebase, squash/split, op recovery, workspaces, bookmarks,
  push.
- Keymap: generated from source if possible.
- Command mapping: jk action -> jj command.
- Config fidelity: how jk honors jj config.
- Prior art/comparison: respectful, factual, lightweight.
- Roadmap.
- Releases/changelog.
- Contributing.

Website assets:

- Short GIFs for each hero workflow, generated from Betamax media tapes.
- Static screenshots for key screens, generated from Betamax media tapes.
- Copyable command examples.
- A "learn jj by using jk" section showing command previews.
- Short demos that make demand-backed differentiators obvious: searchable help, far-away rebase
  target search, visual rebase preview, workspace inspection, op-log recovery, and rich diff/file
  navigation.

### 16.5 Docs generation

Generate from source where possible:

- Keymap docs from keymap data.
- Hotbar/help tests from same data.
- CLI docs from clap.
- Command mapping table from command spec registry.
- Screenshots, GIFs, and release demos from Betamax tapes.

Docs must not drift from implementation.

The docs should make the product roadmap visible without overstating current behavior:

- current README and crate README describe what works now;
- roadmap and product plan describe planned work;
- workflow pages move from planned to current only when implementation, tests, and tapes exist;
- community demand notes should be preserved in maintainer docs or issues, but user-facing copy
  should speak in terms of user problems and workflows.

## 17. Release and distribution plan

### 17.1 Existing release foundation

Keep the current release foundation:

- `release-plz` for crates/releases.
- Binary archives for macOS Intel/Apple Silicon and Linux x86_64/aarch64.
- `.sha256` files.
- cargo-binstall-compatible asset naming.
- Homebrew tap bump.
- CI with Rust check/test/clippy/docs/package/install smoke, markdown, MSRV, actionlint, and GitHub
  Actions security checks.

Release infrastructure requirements:

- crates.io publishing should use Trusted Publishing through GitHub Actions OIDC for `jk`,
  `jk-cli`, `jk-core`, and `jk-tui`;
- the release workflow should use the `release` GitHub environment and `id-token: write`;
- the workflow should not require `CARGO_REGISTRY_TOKEN` once trusted publishing is proven;
- any bootstrap crates.io token should be revoked after an OIDC-backed release succeeds;
- GitHub Release archives and `.sha256` files should be smoke-tested through cargo-binstall and a
  Homebrew formula path;
- archive naming must stay consistent across `crates/jk/Cargo.toml`,
  `.github/workflows/release-plz.yml`, and `scripts/package-release-archive.sh`.

### 17.2 Versioning

Before 1.0:

- Minor versions may change keybindings and UI contracts, but every breaking keymap change must be
  in release notes.
- Patch versions fix bugs only.

After 1.0:

- SemVer applies to CLI flags, config keys, keymap action IDs, and core behavior.
- Visual layout can evolve, but destructive command semantics cannot change silently.

### 17.3 Proposed release milestones

| Version | Theme                       | Must-have deliverables                                                                     |
| ------- | --------------------------- | ------------------------------------------------------------------------------------------ |
| 0.3     | Foundation                  | CommandSpec, view stack, generated help/hotbar, line scrolling, mark model.                |
| 0.4     | jj-shaped inspection        | `jk diff -r/--from/--to`, show, status, View Options, diff search/file jump, evolog.       |
| 0.5     | Command mode and workspaces | `:`, `!`, searchable help, command output, rerun/copy, external runner, `W` workspace.     |
| 0.6     | Safe mutation core          | describe inline/editor, new, commit, edit, rebase picker, abandon, undo/redo, op log.      |
| 0.7     | Content workflows           | squash, split, restore, diffedit, absorb; file selection; initial hunk path via jj editor. |
| 0.8     | Refs and remotes            | bookmarks, tags, fetch, push dry-run, remote management.                                   |
| 0.9     | Native/hybrid graph beta    | config-fidelity tests, native graph previews, rebase ghost preview, large-repo checks.     |
| 1.0     | Obvious daily driver        | Stability, docs/site complete, release channels solid, broad workflows polished.           |
| 1.1+    | Advanced jj                 | sparse, bisect, fix, signing, Gerrit, forge integrations if desired.                       |

### 17.4 Release checklist

For each release:

- `just release-check` passes.
- Fixture tests pass.
- Keymap docs regenerated.
- CLI help regenerated.
- README updated if status changed.
- Website workflow screenshots/GIFs updated when UX changes and only after the behavior is released
  or release-gated.
- Betamax validation tapes pass for changed workflows.
- Betamax media smoke passes for README/site/release demos affected by the change.
- CHANGELOG is written for readers, grouped by workflow or user outcome, and describes
  user-visible changes, keymap changes, config changes, safety changes, and known limitations.
- Release notes are LLM-assisted or human-written from a feature audit, not generated from commit
  messages. Commit history is evidence, not the release-note structure.
- Release notes include install commands and known limitations.
- Smoke install via cargo, cargo-binstall, and Homebrew when applicable.

## 18. Contributor and Codex operating rules

Codex and contributors should follow these rules:

1. Preserve the two contracts: command fidelity and config fidelity.
2. Do not add a mutating action without `JjCommandSpec`, preview, command-history logging, tests,
   and docs/help entries.
3. Do not add a default keybinding without checking the keymap matrix and generated help.
4. Do not make a UI-only shortcut that cannot explain itself as a jj command unless it is pure
   navigation/view state.
5. Keep repo I/O outside `jk-tui`; views request effects, outer layers execute effects.
6. Tests must cover role resolution for marks/cursor.
7. External editor/tool commands must restore terminal and refresh after exit.
8. Prefer using jj's own behavior/config over duplicating jj logic.
9. Any native renderer work must include config-fidelity tests.
10. Update docs and tapes with user-visible workflow changes.

## 19. Issue labels and project board

Suggested labels:

```text
area-graph
area-diff
area-status
area-oplog
area-bookmarks
area-git
area-command-mode
area-keymap
area-rendering
area-config-fidelity
area-release
area-docs
kind-bug
kind-feature
kind-design
kind-refactor
kind-test
kind-docs
safety-mutation
safety-network
priority-p0
priority-p1
priority-p2
good-first-issue
needs-design
```

Project board lanes:

```text
Inbox
Needs design
Ready for implementation
In progress
Needs review
Needs docs/tape
Blocked
Done
```

Every major workflow should have an issue template with:

- jj command(s) involved
- object roles
- proposed keys
- preview behavior
- recovery behavior
- docs/tests/tapes needed

## 20. Design-decision process

Use ADRs for decisions that change product direction:

- Rendering strategy.
- Keymap principles.
- Command mode semantics.
- Native provider vs CLI provider.
- Config source and schema.
- 1.0 stability guarantees.

ADR template:

```text
# ADR N: Title

Status: proposed/accepted/rejected/superseded
Date:
Context:
Decision:
Consequences:
Alternatives considered:
Rollout plan:
Compatibility notes:
```

## 21. Acceptance criteria for "best jj TUI"

`jk` should not claim the crown until these are true:

- A daily jj user can do inspect, rebase, describe, new, commit, squash, split, absorb, restore,
  abandon, bookmarks, fetch, push, undo/redo, and op recovery comfortably.
- The default keymap feels coherent and does not rely on remembering arbitrary uppercase/lowercase
  collisions.
- `?` and the hotbar make the app self-teaching.
- Help and command discovery are searchable enough that users can find actions without reading a
  manual first.
- The command history teaches users the jj commands behind actions.
- Mutations are safer than shell usage because preview and recovery are built in.
- jj config-heavy users feel respected, not forced into a separate visual grammar.
- Large repos remain navigable.
- Large diffs remain inspectable without blocking graph navigation.
- Rebase and graph manipulation have preview paths that make role mistakes hard to commit.
- Installation is one command on major platforms.
- Docs/site explain the mental model in minutes.
- Prior-art users can see why `jk` is jj-native rather than lazygit-inspired.

## 22. First Codex-ready build sequence

Start here.

### Step 1: Add the product planning docs

- Add this document as `docs/product-plan.md`.
- Add `docs/keymap.md` with the default keymap table.
- Add `docs/architecture.md` with CommandSpec/ViewStack/Provider/Renderer.
- Link from README roadmap.

### Step 2: Introduce CommandSpec

- Add `JjCommandSpec`, `ExecutionMode`, `SafetyClass`, `RefreshPlan` to core.
- Move current diff/log command creation toward specs.
- Add tests for command strings/argv.

### Step 3: Introduce ViewStack and ModeStack

- Replace hard-coded log/diff return behavior with a stack.
- Keep current log->diff behavior working.
- Add `Backspace` as back.
- Preserve `q` quit.

### Step 4: Introduce keymap data

- Move keybindings from hard-coded match statements to data-backed bindings.
- Generate help/hotbar from bindings.
- Keep existing keys compatible where not conflicting.
- Change object-screen `Space` to mark/unmark; keep diff text `Space` as page down unless focus is
  on selectable file/hunk.

### Step 5: Add line scrolling

- Add viewport line scroll state independent of selection.
- Bind `Ctrl-j/Ctrl-k`, `Ctrl-d/Ctrl-u`, `Ctrl-f/Ctrl-b`.
- Add tests for selection not changing while scrolling.

### Step 6: Add ordered revision marks

- Mark/unmark with `Space`.
- Show marks in gutter or suffix.
- Preserve marks across refresh when change IDs still exist.
- Add clear marks behavior.

### Step 7: Make diff jj-shaped

- Support `jk diff -r REV`.
- Support `jk diff --from A --to B`.
- Keep `jk diff REV` only as compatibility sugar.
- `d` resolves from marks/cursor.
- `S` opens stat using same resolver.

### Step 8: Add show and status

- `Enter` opens `jj show <rev>`.
- `s` opens `jj status`.
- Reuse diff navigation where possible.
- Keep current diff navigation discoverable: `[ ]` for files, `{ }` for hunks, `f` for file list,
  `/` for search, and `V` for format options.

### Step 9: Add command history and `:`

- Record all commands.
- Implement command output panel.
- Implement `:` jj command mode.
- Add `!` external command mode.

### Step 10: Add the workspace screen

- `W` opens `jj workspace list` as a focused screen.
- Add workspace status and diff actions.
- Add update-stale and forget confirmation flows.
- Add `multi-workspace-repo` fixture and Betamax workspace tapes.

### Step 11: Add op log

- `o` opens `jj op log`.
- `Enter`, `d`, `S`, `r`, `v`, `l` as specified.
- Add restore/revert previews.

After these steps, `jk` will have the architecture needed for the rest of the roadmap.

## 23. The shortest possible north-star statement

> `jk` is the interactive terminal interface for jj. It honors jj config, explains every mutation as
> a jj command, gives you fast keyboard control over revisions, files, hunks, operations,
> bookmarks, and remotes, and makes operation recovery impossible to miss.

## 24. Appendix: planning review additions

These review additions are incorporated into the main plan, but are kept here as explicit guardrails
for future Codex work.

### 24.1 Revised positioning

```text
jk is a jj-native terminal workbench: focused screens, jj-shaped commands,
config-faithful rendering, safe previews, operation recovery, and first-class
workspaces.
```

An even shorter product test:

```text
jk is interactive jj, not a Git dashboard for jj.
```

### 24.2 Corrections incorporated

- Avoid pane-first positioning. Use screens, views, overlays, modals, previews, detail screens,
  the view stack, and focused flows as the core product language.
- Note prior-art implementation languages because they show which architecture and ecosystem
  choices each project validates.
- Treat workspaces as early core scope for daily jj users, not an advanced feature.
- Treat `jk` as a flag-family workbench: build shared selectors, `V` View Options, Run Options, and
  generated command/flag manifests before expanding one-off command forms.
- Reserve standalone `v` for `jj evolog` and use `V` for display, graph, diff, and template
  options.

### 24.3 Betamax product-spine rule

```text
Betamax is the canonical way jk proves, documents, and reviews terminal behavior:
every important workflow should become both a validation tape and, when useful,
a human-readable media tape.
```

### 24.4 Community demand guardrails

Community feedback should keep sharpening the roadmap, but public docs should express those signals
as user problems and workflow requirements:

- people want an obvious jj TUI recommendation, but `jk` should earn that through quality rather
  than relying on blessed-tool language;
- users respond strongly to TUIs that teach the underlying jj command instead of hiding it;
- searchable command discovery, graph search, filter backtracking, rich diff/file navigation,
  rebase preview, workspaces, and operation recovery should stay visible in issues and demos;
- large-repo and large-diff responsiveness should be validated early, not left as late polish;
- prior-art comparison should remain respectful, factual, and useful for design decisions.
