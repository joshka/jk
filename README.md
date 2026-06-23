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

The useful current workflows are:

- inspect selected changes with `show`, `diff`, `evolog`, and `status`;
- review diffs with file/hunk movement, folding, search, file list, and View Options;
- run direct `jj` commands from `:` command mode and keep captured output in the TUI;
- preview local mutations before running describe, abandon, new, edit, undo, and redo;
- use Command History and Operation Log to inspect what ran and recover through `jj op` views;
- inspect sibling jj workspaces, including workspace-scoped log/status/diff views, without leaving
  the TUI.

The current implementation intentionally treats rendered `jj` output as the source of truth. The
TUI parses only enough structure to support navigation, search, sticky headers, folding, command
recording, and recovery-oriented links.

Use `?` inside `jk` for the full key list for the active screen. For task-oriented examples, see
[Workbench Workflows](docs/workbench.md).

Current limitations:

- command history is in-memory for the current `jk` session;
- rebase, squash, split, restore, bookmarks, fetch, and push are still planned workflows;
- direct `a`, `n`, and `e` bindings are dogfood shortcuts until the broader action menu exists;
- public README and website media still need a release-media refresh for the broader workbench
  surface.

## First Useful Paths

Start with `jk` or `jk log`, then use:

- `Enter`, `d`, `v`, and `s` to inspect the selected change;
- `:` to run a direct `jj` command without dropping TUI context;
- `m`, `a`, `n`, `e`, `u`, or `U` to preview a local mutation before it runs;
- `C` and `o` to inspect Command History and Operation Log;
- `W` to inspect other jj workspaces.

![jk diff view](https://www.joshka.net/jk-screenshots/assets/jk-diff-v3.gif)

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
jk workspaces
jk -R /path/to/repo -n 20
jk -R /path/to/repo log --limit 20
```

Bare `jk` follows `jj`'s configured `ui.default-command` when that command is log-like enough for
the semantic navigation pass. Use `jk log` when you want the explicit log command path.

## Roadmap

The detailed product and engineering plan lives in
[docs/product-plan.md](docs/product-plan.md). The shorter
[docs/roadmap.md](docs/roadmap.md) turns that plan into issue-sized milestones.

The current stabilization direction is:

- stabilize the current workbench surface before starting rebase;
- keep README, crate README, changelog, website, and media aligned with released behavior;
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

By default this local development task writes to `target/dogfood-artifacts/readme-media/`. Public
README, crates.io, and release-note media is published later from the separate
[`joshka/jk-screenshots`](https://github.com/joshka/jk-screenshots) repository and served from
`https://www.joshka.net/jk-screenshots/`. Keeping generated media out of this source repo avoids
making jj handle Git LFS-heavy screenshot and GIF churn.

## Crates

The workspace is split into narrow crates:

- `jk`: binary crate and terminal lifecycle;
- `jk-cli`: temporary `jj` process integration;
- `jk-core`: shared records and small data types;
- `jk-tui`: Ratatui state, rendering, and input actions.

Those boundaries keep `jj` presentation decisions at the edge while the TUI owns interaction state.
