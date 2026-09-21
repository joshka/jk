# jk

`jk` is a jj-native terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

Its log and inspection views render `jj` output, including configured graph styles, colors, revsets,
and templates. `jk` adds navigation, search, and repository actions so you can keep changes in view
while an editor, shell, or coding agent works on the repository.

The media below shows the published release. Source-checkout validation is recorded in
[workspace coherence](docs/workspace-coherence.md).

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

In this source checkout, you can:

- inspect changes with log, show, diff, evolog, and status;
- navigate, search, fold, and compare diffs without losing the selected revision;
- use `:` for captured jj commands and `!` for shell-free captured external commands;
- describe inline and use the action menu for New, Edit, Undo, Redo, and Abandon;
- use `R` to choose rebase roles and a destination, `a s` for whole-change squash, and `a r` to
  restore all paths from the selected commit into the working copy;
- review rebase, squash, and restore commands before confirmation, adjust their Run Options, and
  inspect recorded results and links to the resulting operation;
- use `W` to inspect, add, rename, forget, and update stale workspaces;
- use `B` for bookmarks, confirmed fetch from an explicit remote, and scoped push dry-runs.

The main canvas inherits terminal colors. A separate gutter identifies the cursor and marked
changes without replacing jj's graph symbols or semantic text colors. Explicit log refresh runs
in the background, can be cancelled, and preserves the view's context.
Dialogs use a distinct opaque light or dark surface with the repository still visible around them.
Use `--dialog-theme light` or `--dialog-theme dark` to override automatic terminal detection.

Current limitations:

- Command History lasts for the current session.
- Rebase selects one source and one destination from the visible log; multi-parent destinations and
  graph predictions remain planned.
- Squash and restore operate on whole changes; fileset and hunk selection remain planned.
- Native commit, split, absorb, resolve, and foreground editor/tool handoff remain planned.
  Captured command modes do not provide an interactive terminal to child processes.
- Push stops at dry-run. Dedicated tags and advanced remote workflows remain planned.
- Run Options changes working-copy policy and immutable protection for a pending rebase, squash,
  or restore command. Historical operation IDs are available for rebase and squash. Repository
  and config editing remain outside that drawer.

Use `?` inside `jk` for contextual help. See [Using jk](docs/usage.md) for workflows and
[command coverage](docs/command-coverage.md) for implemented forms and the next priorities.

## First Useful Paths

Start with `jk` or `jk log`, then use:

- `Enter`, `d`, and `v` to inspect the selected change; `s` for repository status;
- `:` to run a direct `jj` command or `!` to run an external executable without shell
  interpretation;
- `a`, then `a` to abandon the selected change: empty changes run immediately; other changes open
  a confirmation with Cancel selected;
- `a`, then `r` to preview restoring all paths from the selected revision into `@`;
- `a`, then `m` to edit and save a description inline; choose New, Edit, Undo, or Redo from `a`
  to run them immediately;
- `C` and `o` to inspect Command History and Operation Log;
- `R` to select a rebase destination and review the command;
- `o` inside a supported mutation preview to edit its Run Options;
- `B` for bookmarks and remote workflows;
- `W` for workspace inspection and lifecycle actions.

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

Bare `jk` follows `jj`'s configured `ui.default-command`. The configured command must accept log
templates so `jk` can identify revisions for navigation. Use `jk log` to open the log explicitly.

## Roadmap

The [product plan](docs/product-plan.md) and [roadmap](docs/roadmap.md) track these priorities:

- stabilize rebase, squash, restore, workspace, and bookmark workflows;
- keep README, crate README, changelog, website, and media aligned with released behavior;
- use confirmations for consequential mutations and keep command history and operation recovery
  available after every mutation;
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

This task writes PNGs and GIFs to `assets/` in the sibling
[`joshka/jk-screenshots`](https://github.com/joshka/jk-screenshots) repository. Set
`JK_SCREENSHOTS_REPO` if that repository is elsewhere. Each recording retains its terminal-state
JSON, tape, and binary metadata under `target/dogfood-artifacts/betamax/runs/`.

Publish README, crates.io, and release-note media from `jk-screenshots`, which tracks it with Git LFS
and serves it from `https://www.joshka.net/jk-screenshots/`. Generated media stays out of this source
repository.

## Crates

The workspace contains:

- `jk`: binary crate and terminal lifecycle;
- `jk-cli`: temporary `jj` process integration;
- `jk-core`: shared records and small data types;
- `jk-tui`: Ratatui state, rendering, and input actions.
