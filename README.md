# jk

`jk` is a log-first terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

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

## What Works Today

`jk` currently focuses on the inspection loop:

```text
log -> details -> diff -> back -> refresh
```

The supported surface includes:

- a full-screen log view backed by `jj`;
- explicit `jk log` and default-command `jk` entry points;
- manual refresh without leaving the TUI;
- movement by change, page, and edge;
- inline expansion of the selected change description;
- selected-change diff inspection with `jk diff [REVISION]` or `d` from the log;
- return from diff to log without losing log selection;
- line, page, file, hunk, and horizontal movement in the diff view;
- file folding, fold-all/unfold-all, and hunk folding;
- sticky current-file context with diff stat suffix and `[file x/y]`;
- diff search with `/`, `n`, and `N`;
- retryable empty/error states for selected diffs;
- mode-specific help overlays with `?`;
- Betamax visual tapes for the log and diff workflows.

The current implementation intentionally treats rendered `jj` output as the source of truth. The
TUI parses only enough structure to support navigation, search, sticky headers, and folding.

## Diff Review

The diff view preserves `jj diff` output while adding review controls around it.

![jk diff view](https://www.joshka.net/jk-screenshots/assets/jk-diff-v3.gif)

Useful bindings in the diff view:

| Key                           | Action                       |
| ----------------------------- | ---------------------------- |
| `j` / `k`                     | scroll one line              |
| `Space` / `b`                 | page down / page up          |
| `g` / `G`                     | jump to top / bottom         |
| `[` / `]`                     | previous / next file         |
| `{` / `}`                     | previous / next hunk         |
| `h` / `l`                     | fold / unfold current file   |
| `Ctrl-Left` / `Ctrl-Right`    | fold / unfold all files      |
| `-` / `+`                     | fold / unfold current hunk   |
| `<` / `>`                     | horizontal scroll            |
| `/`, `n`, `N`                 | search, next match, previous |
| `r`                           | refresh                      |
| `H` / `L`                     | return to the log            |
| `?`                           | show mode-specific help      |
| `q` / `Esc`                   | close help, then quit        |

## Commands

```sh
jk
jk log
jk diff
jk diff <revision>
jk -R /path/to/repo -n 20
jk log --repository /path/to/repo --limit 20
```

Bare `jk` follows `jj`'s configured `ui.default-command` when that command is log-like enough for
the semantic navigation pass. Use `jk log` when you want the explicit log command path.

## Roadmap

The next good diff-review improvements are:

- file list or jump overlay for large diffs;
- search highlighting;
- a clearer current-hunk visual marker;
- fold-state indicators on expanded file and hunk headers;
- fold/unfold all hunks for the current file or whole diff;
- optional wrapping mode for prose-heavy or very wide diffs;
- more intentional binary, rename, conflict, mode-change, and permission-change presentation;
- a command palette if the keymap keeps growing;
- visual regression checks over Betamax screenshots or captured state;
- mouse support for file/hunk selection and wheel scrolling.

Broader application directions:

- automatic refresh when repository state changes;
- selected-change `show` view;
- operation log inspection;
- repository status and bookmark views;
- copy/export helpers for review notes;
- carefully scoped mutation actions once target selection and confirmation semantics are boring.

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
