# jk

`jk` is a jj-native terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

It keeps `jj` output as the source of truth and adds an interactive workbench around it: inspect
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

The supported surface includes:

- log view backed by `jj`;
- manual refresh and movement by change, line, page, and edge;
- ordered revision marks;
- inline expansion of the selected change description;
- selected-change show, diff, evolog, and repository status inspection;
- root `jk diff`, `jk show`, and `jk status` entry points;
- diff search, file/hunk navigation, horizontal scrolling, folding, file list, and View Options;
- `:` command mode for direct `jj` commands with captured output;
- in-memory Command History with copy, details, and operation links;
- Workspaces and Operation Log screens;
- command previews for describe, abandon, new, edit, undo, and redo;
- generated help and searchable command discovery.

Current limitations:

- command history is in-memory for the current `jk` session;
- rebase, squash, split, restore, bookmarks, fetch, and push are still planned workflows;
- direct `a`, `n`, and `e` mutation keys are dogfood shortcuts until the broader action menu exists.

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
jk -R /path/to/repo -n 20
```

Bare `jk` follows `jj`'s configured `ui.default-command` when that command is log-like enough for
navigation. Use `jk log` for the explicit log path.

## Roadmap

The detailed roadmap lives in the repository docs:

- [product plan](https://github.com/joshka/jk/blob/main/docs/product-plan.md);
- [issue-sized roadmap](https://github.com/joshka/jk/blob/main/docs/roadmap.md).

Near-term work stabilizes this dogfoodable workbench for release before adding rebase-specific
behavior.

See the repository README for the current status and development workflow.
