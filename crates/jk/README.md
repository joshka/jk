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

`jk` currently focuses on the daily inspect-and-recover loop:

- inspect changes through log, show, diff, evolog, and status views;
- review diffs with file/hunk navigation, folding, search, and View Options;
- run direct `jj` commands from `:` command mode with captured output;
- preview local mutations before describe, abandon, new, edit, undo, and redo;
- inspect Command History, Operation Log, and sibling jj workspaces, including workspace-scoped
  log/status/diff views.

Current limitations:

- command history is in-memory for the current `jk` session;
- rebase, squash, split, restore, bookmarks, fetch, and push are still planned workflows;
- direct `a`, `n`, and `e` mutation keys are dogfood shortcuts until the broader action menu exists.

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

Near-term work stabilizes this dogfoodable jk TUI for release before adding rebase-specific
behavior.

See the repository README for the current status and development workflow.

## License

`jk` is dual-licensed under either
[MIT](https://github.com/joshka/jk/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/joshka/jk/blob/main/LICENSE-APACHE).
