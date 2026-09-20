# jk

`jk` is a jj-native terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

It keeps the parts of `jj` you already trust: the graph, colors, wording, revsets, templates, and
diff output still come from `jj`. `jk` adds an interactive review loop around that output so you can
keep context open while an editor, shell, or coding agent changes the repository.

The media below shows the published release; current integration proof is described in
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

This source checkout supports inspection, local history editing, workspace management, and sharing
previews with consistent controls and command previews:

- inspect changes with log, show, diff, evolog, and status;
- navigate, search, fold, and compare diffs without losing the selected revision;
- use `:` for captured jj commands and `!` for shell-free captured external commands;
- describe inline and use the action menu for New, Edit, Undo, Redo, and Abandon;
- use `R` to choose rebase roles and a destination, `a s` for whole-change squash, and `a r` to
  restore all paths from the selected commit into the working copy;
- inspect exact mutation commands before confirmation, with command-local Run Options on supported
  previews, recorded results, and links to the resulting operation;
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
- Run Options currently covers working-copy policy, eligible historical operations, and immutable
  override for a pending rebase, squash, or restore command. Repository and config editing remain
  outside that drawer.

Use `?` inside `jk` for contextual help. See [Using jk](docs/usage.md) for workflows and
[command coverage](docs/command-coverage.md) for implemented forms and the next priorities.

## First Useful Paths

Start with `jk` or `jk log`, then use:

- `Enter`, `d`, `v`, and `s` to inspect the selected change;
- `:` to run a direct `jj` command or `!` to run an external executable without shell
  interpretation;
- `a` to open actions for the selected change, then `a` again to abandon it after an empty check
  and, when content would be discarded, a confirmation;
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

Bare `jk` follows `jj`'s configured `ui.default-command` when that command is log-like enough for
the semantic navigation pass. Use `jk log` when you want the explicit log command path.

## Roadmap

The detailed product and engineering plan lives in
[docs/product-plan.md](docs/product-plan.md). The shorter
[docs/roadmap.md](docs/roadmap.md) turns that plan into issue-sized milestones.

The current stabilization direction is:

- stabilize the integrated rebase, squash, restore, and workspace workflows;
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
