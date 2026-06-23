# jk

`jk` is a jj-native terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

It keeps the parts of `jj` you already trust: the graph, colors, wording, revsets, templates, and
diff output still come from `jj`. `jk` adds an interactive review loop around that output so you can
keep context open while an editor, shell, or coding agent changes the repository.

![jk log view](https://www.joshka.net/jk-screenshots/assets/jk-log-v3.gif)

## Installation

Install with Homebrew:

```sh
brew trust --formula joshka/tap/jk
brew install joshka/tap/jk
```

The trust command scopes Homebrew's non-official tap trust to the `jk` formula.

Install prebuilt release assets with [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):

```sh
cargo binstall jk
```

Or build from the crates.io source package:

```sh
cargo install jk --locked
```

## Current Status

`jk` now covers the daily inspection and recovery loop:

```text
log -> inspect -> preview command -> run -> history -> operation recovery
```

The supported surface includes:

- a full-screen log view backed by `jj`;
- explicit `jk log` and default-command `jk` entry points;
- manual refresh without leaving the TUI;
- movement by change, line, page, and edge;
- ordered revision marks for multi-revision workflows;
- inline expansion of the selected change description;
- selected-change `show`, `diff`, `evolog`, and repository `status` inspection;
- root `jk diff`, `jk show`, and `jk status` entry points;
- return from inspection views without losing log selection;
- `:` command mode for direct `jj` commands with captured output;
- in-memory Command History with argv, output, status, duration, copy, and operation links;
- Operation Log, operation show, and operation diff views for recovery-oriented inspection;
- Workspaces screen with workspace status, diff, refresh, and stale-workspace update;
- command previews for describe, abandon, new, edit, undo, and redo;
- post-mutation recovery footer with undo, redo, operation, and history entry points;
- line, page, file, hunk, and horizontal movement in the diff view;
- file folding, fold-all/unfold-all, and hunk folding;
- sticky current-file context with diff stat suffix and `[file x/y]`;
- diff search with `/`, `n`, and `N`;
- diff View Options for patch, summary, stat, types, name-only, git, and color-words output;
- file list overlay for jumping within the active diff;
- generated help and searchable command discovery with `?`;
- retryable empty/error states for selected diffs;
- [Betamax](https://www.joshka.net/betamax/) visual tapes for the log and diff workflows.

The current implementation intentionally treats rendered `jj` output as the source of truth. The
TUI parses only enough structure to support navigation, search, sticky headers, folding, command
recording, and recovery-oriented links.

Current limitations:

- command history is in-memory for the current `jk` session;
- rebase, squash, split, restore, bookmarks, fetch, and push are still planned workflows;
- direct `a`, `n`, and `e` bindings are dogfood shortcuts until the broader action menu exists;
- public README and website media still need a release-media refresh for the broader workbench
  surface.

## Diff Review

The diff view preserves `jj diff` output while adding review controls around it.

![jk diff view](https://www.joshka.net/jk-screenshots/assets/jk-diff-v3.gif)

Useful bindings in the diff view:

| Key                           | Action                       |
| ----------------------------- | ---------------------------- |
| `j` / `k`                     | scroll one line              |
| `Space` / `b`                 | page down / page up          |
| `g` / `G`                     | jump to top / bottom         |
| `f`                           | open file list               |
| `[` / `]`                     | previous / next file         |
| `{` / `}`                     | previous / next hunk         |
| `h` / `l`                     | fold / unfold current file   |
| `Ctrl-Left` / `Ctrl-Right`    | fold / unfold all files      |
| `-` / `+`                     | fold / unfold current hunk   |
| `<` / `>`                     | horizontal scroll            |
| `/`, `n`, `N`                 | search, next match, previous |
| `V`                           | choose diff output format    |
| `r`                           | refresh                      |
| `H` / `L`                     | return to the log            |
| `?`                           | show mode-specific help      |
| `q` / `Esc`                   | close help, then quit        |

## Safe Command Loop

Mutating workflows run through command preview. The preview shows the exact `jj` command before it
runs, supports copy with `y`, cancels with `Esc`, and records the result in Command History after
confirmation.

Useful log bindings:

| Key       | Action                                                |
| --------- | ----------------------------------------------------- |
| `Enter`   | show selected revision                                |
| `d`       | diff selected revision                                |
| `v`       | show selected revision's evolog                       |
| `s`       | show repository status                                |
| `m`       | preview `jj describe` for the selected revision       |
| `a`       | preview `jj abandon REV`                              |
| `n`       | preview `jj new`; marked revisions become parents     |
| `e`       | preview `jj edit REV`                                 |
| `u` / `U` | preview undo / redo                                   |
| `o`       | open operation log                                    |
| `C`       | open Command History                                  |
| `W`       | open Workspaces                                       |
| `:`       | run a direct `jj` command and capture output          |
| `V`       | open View Options                                     |
| `?`       | open generated help and searchable command discovery  |

## Commands

```sh
jk
jk log
jk log -T builtin_log_compact_full_description
jk diff
jk diff -r <revision>
jk diff --from <revision> --to <revision>
jk diff --stat
jk diff --summary
jk show <revision>
jk status
jk status <fileset>
jk -R /path/to/repo -n 20
jk log --repository /path/to/repo --limit 20
```

Bare `jk` follows `jj`'s configured `ui.default-command` when that command is log-like enough for
the semantic navigation pass. Use `jk log` when you want the explicit log command path.

## Roadmap

The detailed product and engineering plan lives in
[docs/product-plan.md](docs/product-plan.md). The shorter
[docs/roadmap.md](docs/roadmap.md) turns that plan into issue-sized milestones.

The current stabilization direction is:

- stabilize the current workbench surface before starting rebase;
- scrub the implementation history and release notes into product-facing language;
- update README, crate README, changelog, website, and media so they agree about released behavior;
- keep mutating workflows behind command previews, command history, and operation recovery;
- make [Betamax](https://www.joshka.net/betamax/) tapes the source for regression tests,
  README/site media, and release demos.

## Development

This repository uses `jj` for version control and `just` for local tasks.

```sh
just --list
just check
just test
just clippy
just lint-md
```

Visual README media is generated with:

```sh
just readme-media
```

The generated screenshots and GIFs live in the separate
[`joshka/jk-screenshots`](https://github.com/joshka/jk-screenshots) repository and are served from
`https://www.joshka.net/jk-screenshots/`.

## Crates

The workspace is split into narrow crates:

- `jk`: binary crate and terminal lifecycle;
- `jk-cli`: temporary `jj` process integration;
- `jk-core`: shared records and small data types;
- `jk-tui`: Ratatui state, rendering, and input actions.

Those boundaries keep `jj` presentation decisions at the edge while the TUI owns interaction state.
