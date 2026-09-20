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
- `o` opens [Run options](run-options.md) for rebase, squash, and restore previews.

After a successful action, `jk` refreshes the log and records the command and resulting operation in
Command History. The normal footer controls remain visible alongside a short success or error
message. After `jj new` succeeds, selection moves to the newly created working copy. When jj reports
a resulting operation id, Command History can open the exact `jj op show` view.

The first restore slice intentionally supports only all-path content copying from one selected
revision into the working copy. Fileset selection, alternate destinations, `--changes-in`, and hunk
restore remain follow-up work. Immutable destinations fail through jj's normal check unless you
explicitly allow rewriting in Run options. Failures remain inspectable in Command History.

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

## Run A Direct jj Command

Press `:` to run a direct `jj` command without leaving the TUI:

```text
:status
:log -r 'mine()'
:describe -m "new message" @
```

Command mode accepts an optional `jj` prefix. It parses argv-like input, does not invoke a shell,
captures stdout and stderr, and records the result in Command History.

From command output, press `e` to reopen command mode with the previous input.

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
- Split, file/hunk restore, bookmarks, fetch, and push are planned workflows.
- Squash is whole-change only; file and hunk selection remain planned.
- The action menu covers log revision changes, recovery, and workspace lifecycle actions.
  Rebase has a direct `R` entry; refs and remote actions remain planned.
- Public README, crates.io, and website media still need a release-media refresh.
