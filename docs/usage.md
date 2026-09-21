# Using jk

Press `?` inside `jk` for the full key list for the active screen.

## Start From The Log

Start `jk` in a jj repository:

```sh
jk
```

From the log:

- `Enter` opens `jj show` for the selected change.
- `d` opens `jj diff` for the selected change.
- `v` opens the selected change's evolog.
- `s` opens repository status.
- `Backspace` returns to the previous view. `Esc` quits from ordinary rendered inspection or command
  output; dialogs and list views handle it separately.
- `r` refreshes the active view.

`jk log -T <template>` changes the rendered log template, but the navigation pass still uses `jk`'s
internal template so movement and selection stay stable.
Loading or refreshing the log snapshots working-copy changes, as ordinary `jj log` does.

## Review A Diff

Open a diff from the log with `d`, or start from the command line:

```sh
jk diff -r <revision>
jk diff --from <from> --to <to>
jk diff --stat
```

In the diff view:

- `[` and `]` move between files.
- `{` and `}` are hunk-navigation bindings, but normal diff formats do not produce recognized hunks.
- `f` opens the file list.
- `/`, `n`, and `N` search visible diff text.
- `h` and `l` fold or unfold the current file.
- `V` changes diff output format, such as patch, stat, summary, name-only, git, or color-words.

The diff view still renders `jj diff`; `jk` adds navigation, folding, search, current-file context,
and format switching around that output.

Choose color-words for file navigation and folding. Git-format patches render, but the file parser
does not recognize their headers, so the file picker reports no files. Hunk navigation and folding
require both color-words file headers and Git-style hunk headers; ordinary output provides one or
the other. Use scrolling or search to inspect content within a file.

## Open The Action Menu

Press `a` from the log to open revision actions and recovery commands. Each row indicates whether
the command runs immediately or opens a confirmation:

- `m` edits the selected revision's description. `Ctrl-j` inserts a newline; `Enter` saves.
- `n` creates a new change from ordered marks or the selected revision.
- `e` edits the selected revision.
- `a` checks whether the selected revision is empty before abandoning it.
- `s` previews Squash after resolving ordered marks as sources and the cursor as destination.
- `r` previews copying all paths from the selected revision into `@`.
- `u` runs `jj undo`.
- `U` runs `jj redo`.

Inside the menu, press an action key or select a row with `j`/`k` and `Enter`. These mutation keys
are active only while the menu is open. Show, diff, status, and evolog views do not offer repository
actions.

Squash operates on whole changes. Mark one or more source revisions in the order you want
them shown, move the cursor to a distinct destination, then press `a s`. The confirmation labels
each role, shows the exact command, keeps the destination description, and supports cancellation.
File and hunk selection are not yet available.

Command previews other than [Abandon](#abandon-a-change) use these controls:

- `Enter` runs the displayed command.
- `y` copies the displayed command line.
- `Esc` cancels.
- Arrow keys and Page Up/Down scroll long previews without hiding the confirmation controls.
- `o` opens [Run options](run-options.md) for rebase, squash, and restore previews.

After a successful action, `jk` refreshes the log and records the command and resulting operation in
Command History. The normal footer controls remain visible alongside a short success or error
message. After `jj new` succeeds, selection moves to the newly created working copy. When jj reports
a resulting operation id, Command History can open the exact `jj op show` view.

Restore copies all paths from the selected revision into `@`. It rejects revision marks as ambiguous,
resolves the source to an exact commit, and shows both operands and the `all paths` scope before
confirmation. Fileset selection, alternate destinations, `--changes-in`, and hunk restore are not
yet available. Immutable destinations fail through jj's normal check unless you explicitly allow
rewriting in Run options. Failures remain inspectable in Command History.

## Abandon A Change

Press `a`, then `a` from the log. Empty revisions are abandoned immediately. Non-empty revisions,
or a failed emptiness check, open a confirmation with Cancel selected.

Use Left/Right or Tab to choose View diff, Cancel, or Abandon, then press Enter. From the summary,
`y` confirms abandonment; `n` or Escape cancels. In the embedded diff, Enter or Escape returns to
the summary, and `n` cancels. A terminal too small to show the decision must be enlarged before
confirmation is available.

The embedded patch uses Git-format output with jk's addition/deletion colors. It does not yet
inherit configured jj diff formatting or colors. To inspect that configured output before
abandoning, open the normal diff view with `d` from the log.

## Rebase A Revision

Press `R` from the log. With no marks, the cursor is the source. With one mark, that mark is the
source and the cursor suggests a destination. Multiple marks are rejected rather than assigned
implicit roles. The picker searches visible revisions by description or commit ID with `/`.

The default moves the selected revision onto the destination (`-r`, `-o`), reconnecting its
descendants to its old parents. Choose `b` for a branch relative to the destination, including
applicable ancestors and descendants, or `s` for the source and its descendants. Placement `A` inserts
after the destination and also rewrites its existing descendants; `B` inserts before it and rewrites
the destination and its descendants. These broader modes can affect other workspaces: review their
scope carefully. `o` returns to onto placement.

Enter opens the command preview; Enter again confirms. Escape cancels without a graph mutation
(while searching, the first Escape exits search). Commands use full commit IDs so later selection
changes cannot retarget the preview.
Immutable revisions remain protected unless explicitly overridden in Run Options. jj failures
remain inspectable in Command History.

## Dialog appearance

Dialogs have a distinct filled surface over the repository view. jk detects a light or dark terminal
at startup. Use `jk --dialog-theme light` or `jk --dialog-theme dark` to override detection; `auto`
is the default. Unsupported or slow terminals use the dark surface. This setting changes
jk dialogs only; it preserves jj's configured graph, diff colors, and terminal background.

## Run Direct Commands

Press `:` to run a direct `jj` command without leaving the TUI:

```text
:status
:log -r 'mine()'
:describe -m "new message" @
```

Command mode accepts an optional `jj` prefix. It parses argv-like input, does not invoke a shell,
captures stdout and stderr, and records the result in Command History.

From command output, press `e` to reopen command mode with the previous input. `Backspace` returns to
the preceding view; `Esc` quits jk.

Press `!` to run an external executable with explicit argv:

```text
!printf '%s\n' 'hello from jk'
!rg -n 'CommandHistory' crates
!sh -c 'printf "shell requested explicitly\n" | cat'
```

External command mode never adds a shell. Quotes and backslashes group argv for `jk`; shell
metacharacters such as `$`, `|`, `>`, and `&&` remain literal arguments. Run `sh -c` (or another
shell) explicitly when shell expansion, pipes, redirects, or built-ins are intentional.

Both command modes are noninteractive: stdin is closed, stdout and stderr are retained in the
output view, and the exit code or terminating signal is recorded in Command History. Commands that
require a foreground terminal, password prompt, or full-screen TUI must run after leaving `jk`.

Press `e` to edit or retry the same `!` input, `C` to inspect its history record, or `y` in Command
History to copy the recorded command line. History redacts recognized secret patterns in argv,
stdout, and stderr using the same rules as `jj` commands.

## Inspect History And Operations

1. Press `C`.
1. Press `Enter` to inspect argv, output, status, duration, and operation metadata.
1. Press `o` to open the recorded operation. If the selected record has no operation id, `jk` opens
   Operation Log instead.

Press `o` from the log to open Operation Log directly. Operation show and diff views behave like
other rendered inspection views: search, page, refresh, and return work the same way.

## Inspect Workspaces

Press `W` to list jj workspaces, or start there with `jk workspaces`. From there:

- `l` opens log for the selected workspace.
- `Enter` or `s` opens status for the selected workspace.
- `d` opens diff for the selected workspace.
- `u` previews `jj workspace update-stale` for the selected workspace.
- `a` opens workspace actions: `a` add, `r` rename, `f` forget metadata, and `u` update stale.
- Add and rename first collect input, then show the exact command for confirmation.
- Forget removes only jj workspace metadata; the workspace directory and files remain on disk.
- `r` refreshes the workspace list.

If a workspace directory is missing, `jk` displays an error and keeps the workspace list open.

## Inspect Bookmarks And Remote Safety

Press `B` from the log to inspect local and remote bookmark rows. The bookmark view keeps remote,
tracking, deleted, and conflicted rows visible while limiting local mutations to unambiguous local
bookmarks.

- `c` opens a create-bookmark prompt, then a confirmation preview.
- `m` edits the selected bookmark's target and opens a move preview.
- `x` opens a destructive delete preview.
- `F` lists configured remotes, including remotes with no bookmarks yet. Choose one and press
  `Enter` to preview fetch; confirm separately to run it.
- `P` starts from a local bookmark with one target. Choose a configured remote, then review and
  confirm the scoped push dry-run. Deleted, conflicted, and remote-only rows cannot start a push.

Pattern-based jj arguments use exact matching so a bookmark or remote name cannot expand into
multiple targets. Command mode rejects `bookmark create`, `bookmark move`, `bookmark delete`,
`git fetch`, and `git push` with instructions to use the bookmark screen. Other bookmark commands
remain available through command mode.

Fetch and push dry-run output is shown in a retained command-output view and recorded in Command
History. Fetch refreshes the underlying bookmark list before displaying output; a failed refresh
preserves the last usable snapshot. Push dry-run may contact the selected remote but does not update
its refs. Real push remains unavailable.

## Command Entry Points

Command-line entry points include:

```sh
jk
jk log [-n <limit>] [-T <template>]
jk diff -r <revision>
jk diff --from <from> --to <to>
jk diff --stat
jk diff --name-only
jk diff --git
jk diff --color-words
jk show <revision>...
jk status [fileset]...
jk workspaces
jk -R /path/to/repo -n 20
```

`jk` also has in-app paths for evolog, bookmarks, operation views, command previews, and command
history.

## Current Limits

- Command History is in-memory for the current `jk` session.
- Split, file/hunk restore, and real remote pushes remain planned workflows.
- Squash is whole-change only; file and hunk selection remain planned.
- The action menu covers log revision changes, recovery, and workspace lifecycle actions.
  Rebase has a direct `R` entry; bookmark actions use the bookmark screen.
- Public README, crates.io, and website media still need a release-media refresh.
