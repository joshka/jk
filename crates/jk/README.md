# jk

`jk` is a jj-native terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

It keeps `jj` output as the source of truth and adds an interactive jj TUI around it: inspect
changes, run safe command previews, review command history, and recover through operation views
without losing terminal context.

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

Use `?` inside the TUI for full screen-specific key help. The repository's
[Using jk](https://github.com/joshka/jk/blob/main/docs/usage.md) guide has
task-oriented examples for the current jk surface.

## Commands

```sh
jk
jk log
jk log -T builtin_log_compact_full_description
jk diff
jk diff -r <revision>
jk diff --from <revision> --to <revision>
jk diff --stat
jk show <revision>
jk status
jk workspaces
jk -R /path/to/repo -n 20
```

Bare `jk` follows `jj`'s configured `ui.default-command` when that command is log-like enough for
navigation. Use `jk log` for the explicit log path.

## Roadmap

The detailed roadmap lives in the repository docs:

- [product plan](https://github.com/joshka/jk/blob/main/docs/product-plan.md);
- [issue-sized roadmap](https://github.com/joshka/jk/blob/main/docs/roadmap.md).

Near-term work stabilizes the integrated rebase, squash, restore, workspace, and refs workflows.
The next command slices are foreground editor/tool handoff, shared fileset selection, and completing
publication from the existing push dry-run flow.

See the repository README for the current status and development workflow.

## License

`jk` is dual-licensed under either
[MIT](https://github.com/joshka/jk/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/joshka/jk/blob/main/LICENSE-APACHE).
