# jk

`jk` is a jj-native terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

It keeps a `jj` log-like view open today, lets you refresh in place, and adds interactive navigation
for reviewing change descriptions and selected-change diffs.

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
- manual refresh;
- movement by change, page, and edge;
- inline expansion of the selected change description;
- selected-change diff inspection from the log or with `jk diff [REVISION]`;
- diff search, file/hunk navigation, horizontal scrolling, and folding;
- mode-specific help overlays;
- retryable empty/error states for selected diffs.

## Commands

```sh
jk
jk log
jk diff
jk diff <revision>
jk -R /path/to/repo -n 20
```

Bare `jk` follows `jj`'s configured `ui.default-command` when that command is log-like enough for
navigation. Use `jk log` for the explicit log path.

## Roadmap

The detailed roadmap lives in the repository docs:

- [product plan](https://github.com/joshka/jk/blob/main/docs/product-plan.md);
- [issue-sized roadmap](https://github.com/joshka/jk/blob/main/docs/roadmap.md).

Near-term work preserves jj-rendered output while adding command-shaped inspection, `show`,
`status`, command mode, command history, workspaces, command previews, and operation recovery.

See the repository README for the current status and development workflow.

## License

`jk` is dual-licensed under either
[MIT](https://github.com/joshka/jk/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/joshka/jk/blob/main/LICENSE-APACHE).
