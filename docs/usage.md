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

## Preview A Local Mutation

Mutation shortcuts open a preview instead of running immediately:

- `m` previews `jj describe` for the selected revision.
- `a` previews `jj abandon <revision>`.
- `n` previews `jj new <parents>` from marks or the selected revision.
- `e` previews `jj edit <revision>`.
- `u` previews `jj undo`.
- `U` previews `jj redo`.

In the preview:

- `Enter` runs the displayed command.
- `y` copies the displayed command line.
- `Esc` cancels.

After a confirmed mutation, `jk` refreshes the log and records the result in Command History. When
`jj` reports a resulting operation id, Command History can open the exact `jj op show` view.

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
- `u` runs `jj workspace update-stale` when the selected workspace is stale.
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
- Rebase, squash, split, restore, bookmarks, fetch, and push are planned workflows.
- Direct mutation keys are dogfood shortcuts until the broader action menu exists.
- Public README, crates.io, and website media still need a release-media refresh.
