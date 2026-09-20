# Using jk

This guide shows the shortest paths through the current `jk` TUI. It is not a complete key
reference. Press `?` inside `jk` for the full key list for the active screen.

## Start From The Log

Run:

```sh
jk
```

Use the log as the place to keep context while inspecting or changing the repository:

- `Enter` opens `jj show` for the selected change.
- `d` opens `jj diff` for the selected change.
- `v` opens the selected change's evolog.
- `s` opens repository status.
- `Backspace` or `Esc` returns to the previous view.
- `r` refreshes the active view.

`jk log -T <template>` changes the rendered log template, but the navigation pass still uses `jk`'s
internal template so movement and selection stay stable.

## Review A Diff

Open a diff from the log with `d`, or start from the command line:

```sh
jk diff -r <revision>
jk diff --from <from> --to <to>
jk diff --stat
```

Use the smallest set of controls that gets you through review:

- `[` and `]` move between files.
- `{` and `}` move between hunks.
- `f` opens the file list.
- `/`, `n`, and `N` search visible diff text.
- `h` and `l` fold or unfold the current file.
- `V` changes diff output format, such as patch, stat, summary, name-only, git, or color-words.

The diff view still renders `jj diff`; `jk` adds navigation, folding, search, current-file context,
and format switching around that output.

## Open The Action Menu

Press `a` from the log to open actions that are meaningful for the selected revision. The menu
groups revision changes separately from history and recovery, labels safety and execution behavior,
and routes each selection through the existing command runner:

- `m` opens an inline description editor for the selected revision; `Enter` saves it immediately.
- `n` runs New change inside the action menu from marks or the selected revision.
- `e` runs Edit change inside the action menu.
- `s` previews Squash after resolving ordered marks as sources and the cursor as destination.
- `r` previews copying all paths from the selected revision into `@`.
- `u` runs `jj undo` inside the action menu.
- `U` runs `jj redo` inside the action menu.

Inside the menu, press an action key or select a row with `j`/`k` and `Enter`. In particular, `a a`
checks whether the selected revision is empty before abandoning it: empty revisions run
immediately, while non-empty revisions open the destructive preview. Describe, New, Edit, Undo,
and Redo are menu-only there. Restore rejects revision marks as ambiguous, resolves the selected
source to an exact commit, and shows source, destination, and `all paths` scope before confirmation.
Inspection views remain read-only and do not offer repository actions.

Squash currently operates on whole changes. Mark one or more source revisions in the order you want
them shown, move the cursor to a distinct destination, then press `a s`. The confirmation labels
each role, shows the exact command, keeps the destination description, and supports cancellation.
File and hunk selection are intentionally deferred.

For actions that open a preview:

- `Enter` runs the displayed command.
- `y` copies the displayed command line.
- `Esc` cancels.
- Arrow keys and Page Up/Down scroll long previews without hiding the confirmation controls.

After a successful action, `jk` refreshes the log and records the command and resulting operation in
Command History. The normal footer controls remain visible alongside a short success or error
message. After `jj new` succeeds, selection moves to the newly created working copy. When jj reports
a resulting operation id, Command History can open the exact `jj op show` view.

The first restore slice intentionally supports only all-path content copying from one selected
revision into the working copy. Fileset selection, alternate destinations, `--changes-in`, and hunk
restore remain follow-up work. Immutable destinations fail through jj's normal check; `jk` keeps
stderr inspectable in Command History and does not expose `--ignore-immutable` in this workflow.

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

Enter opens a separate, exact-command preview; it does not execute from the picker. Enter again in
the preview confirms. Escape cancels without a graph mutation (while searching, the first Escape
exits search). Commands use full commit IDs so later selection changes cannot retarget the preview.
Immutable revisions are never overridden: jj failures remain inspectable in Command History.

## Run Direct Commands

Press `:` to run a direct `jj` command without leaving the TUI:

```text
:status
:log -r 'mine()'
:describe -m "new message" @
```

Command mode accepts an optional `jj` prefix. It parses argv-like input, does not invoke a shell,
captures stdout and stderr, and records the result in Command History.

From command output, press `e` to reopen command mode with the previous input.

Press `!` to run an external executable with explicit argv:

```text
!printf '%s\n' 'hello from jk'
!rg -n 'CommandHistory' crates
!sh -c 'printf "shell requested explicitly\n" | cat'
```

External command mode never adds a shell. Quotes and backslashes group argv for `jk`; shell
metacharacters such as `$`, `|`, `>`, and `&&` remain literal arguments. Run `sh -c` (or another
shell) explicitly when shell expansion, pipes, redirects, or built-ins are intentional.

This first external-command mode is captured and noninteractive: stdin is closed, stdout and stderr
are retained in the output view, and the exit code or terminating signal is recorded in Command
History. Commands that require a foreground terminal, password prompt, or full-screen TUI are not
supported in this mode; run them after leaving `jk`. This keeps `jk`'s terminal state intact while a
future foreground/cancellable runner can own terminal suspension and restoration explicitly.

Failures remain inspectable. Press `e` to edit or retry the same `!` input, `C` to inspect its
redacted history record, or `y` in Command History to copy the redacted command line. Obvious
secret-looking argv, stdout, and stderr values use the same redaction policy as `jj` commands.

## Inspect History And Operations

Press `C` for Command History. Use it to inspect what `jk` ran, copy exact commands, and follow
operation links.

Useful history path:

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

Missing workspace roots are reported inside `jk` instead of pushing a broken view.

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
multiple targets. Command mode routes bookmark mutations, fetch, and push back to these workflows.

Fetch and push dry-run output is shown in a retained command-output view and recorded in Command
History. Fetch refreshes the underlying bookmark list before displaying output; a failed refresh
preserves the last usable snapshot. Push dry-run may contact the selected remote but does not update
its refs. Real push remains unavailable.

## Command Entry Points

The current root commands are:

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
jk -R /path/to/repo -n 20
```

`jk` also has in-app paths for evolog, workspaces, operation views, command previews, and command
history.

## Current Limits

- Command History is in-memory for the current `jk` session.
- Split, file/hunk restore, and real remote pushes remain planned workflows.
- Squash is whole-change only; file and hunk selection remain planned.
- The action menu covers log revision changes, recovery, and workspace lifecycle actions.
  Rebase has a direct `R` entry; bookmark actions use the bookmark screen.
- Public README, crates.io, and website media still need a release-media refresh.
