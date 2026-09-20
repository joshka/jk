# jk

`jk` is a jj-native terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

Its log and inspection views render `jj` output, including configured graph styles, colors, revsets,
and templates. `jk` adds navigation, search, repository actions, and command history.

The media below shows the published release.

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

Use `?` for the active screen's keys. See
[Using jk](https://github.com/joshka/jk/blob/main/docs/usage.md) for workflows and examples.

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

Bare `jk` follows `jj`'s configured `ui.default-command`. The configured command must accept log
templates so `jk` can identify revisions for navigation. Use `jk log` to open the log explicitly.

## Roadmap

The detailed roadmap lives in the repository docs:

- [product plan](https://github.com/joshka/jk/blob/main/docs/product-plan.md);
- [issue-sized roadmap](https://github.com/joshka/jk/blob/main/docs/roadmap.md).

Near-term work stabilizes rebase, squash, restore, workspace, and bookmark workflows. The next
priorities are foreground editor/tool handoff, shared fileset selection, and push after dry-run.

See the repository README for the current status and development workflow.

## License

`jk` is dual-licensed under either
[MIT](https://github.com/joshka/jk/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/joshka/jk/blob/main/LICENSE-APACHE).
