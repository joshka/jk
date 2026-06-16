# jk

> Prototype archive: `joshka/prototype-1-rewrite` is the first cleaned-up rewrite
> experiment. It keeps the surface intentionally narrow around log, show, diff,
> refresh, search, copy, and sticky file context. Use it as the most reviewable
> baseline for product shape, but treat its rendered CLI parsing as something to
> replace with stronger `jj` library/rendering contracts before a curated release.

`jk` is an experimental Rust TUI for [Jujutsu](https://github.com/jj-vcs/jj). It is a log-first
interface for keeping `jj` visible while work happens in an editor, another terminal, or an agent
session, then refreshing in place instead of repeatedly quitting and rerunning `jj log`.

The current implementation shells out to `jj` and treats rendered `jj` output as canonical. It
preserves user colors, templates, graph symbols, wording, and diff style wherever possible. The
long-term direction is a Rust-based jj interface that feels close to the existing CLI UI while
making it cheap to navigate between related jj concepts.

## Direction

`jk` is intentionally not a pane-heavy repository dashboard. The preferred interaction model is one
active view at a time:

- start in a compact `jj` or `jj log` graph view;
- move with vimish keys;
- open the selected change as `show` or `diff`;
- move back to the previous view;
- refresh the current view in place after external changes;
- keep revsets, templates, colors, and command names aligned with jj.

This follows the direction explored in [`jj-vcs/jj#9319`](https://github.com/jj-vcs/jj/pull/9319): a
native, Rust-shaped, jj-centered TUI with view-centric navigation. Splits and previews may be useful
later, but panes should be presentation choices rather than the core mental model.

## Current State

Today `jk` supports:

- default graph view through `jj`;
- explicit `log`, `show`, and `diff` startup commands;
- graph navigation by selected change id;
- sticky file context in `show` and `diff` views;
- text search within the current view;
- copy menus for selected revisions or file labels;
- switching between default jj diff output and `--git` diff output;
- refresh-in-place for the current view.

This is still a prototype. It is useful enough to exercise the architecture, but the product surface
is intentionally narrow.

## Install And Run

Prerequisites:

- Rust toolchain with edition 2024 support;
- `jj` available on `PATH`;
- nightly Rust for formatting through the repository `rustfmt.toml`.

Run the TUI from a jj repository:

```sh
just run
```

You can also pass a supported startup command:

```sh
cargo run -- log
cargo run -- log -r 'mine() & mutable()'
cargo run -- show @
cargo run -- diff -r @
```

## Keybindings

Global keys:

  | Key         | Action                                |
  | ----------- | ------------------------------------- |
  | `q`, `Esc`  | Quit                                  |
  | `?`         | Toggle help                           |
  | `r`         | Refresh the current view              |
  | `/`         | Search within the current view        |
  | `n`, `N`    | Move to next or previous search match |
  | `y`         | Open the copy menu                    |
  | `v`         | Open the diff-format menu             |
  | `h`, `Left` | Go back                               |
  | `L`         | Switch to `jk log`                    |
  | `J`         | Switch to the default `jk` view       |

Graph view:

  | Key               | Action                              |
  | ----------------- | ----------------------------------- |
  | `j`, `Down`       | Move down                           |
  | `k`, `Up`         | Move up                             |
  | `g`, `Home`       | Move to first item                  |
  | `G`, `End`        | Move to last item                   |
  | `l`, `Right`, `s` | Open `show` for the selected change |
  | `d`               | Open `diff` for the selected change |

Show and diff views:

  | Key                               | Action                   |
  | --------------------------------- | ------------------------ |
  | `j`, `Down`                       | Scroll down              |
  | `k`, `Up`                         | Scroll up                |
  | `Space`, `PageDown`, `Ctrl-f`     | Page down                |
  | `Shift-Space`, `PageUp`, `Ctrl-b` | Page up                  |
  | `g`, `Home`                       | Scroll to top            |
  | `G`, `End`                        | Scroll to bottom         |
  | `]`                               | Jump to next file        |
  | `[`                               | Jump to previous file    |
  | `s`                               | Switch from diff to show |
  | `d`                               | Switch from show to diff |

## Development

Use the repository `just` commands:

```sh
just fmt
just md-fmt
just md-check
just test
just check
```

`just check` formats Rust with nightly rustfmt, checks Markdown with Panache, then runs
`cargo check` and `cargo test`.

The architecture notes in [`docs/agent/architecture.md`](docs/agent/architecture.md) describe the
current internal boundaries. The most important rule is that `jk` should stay close to jj rather
than quietly becoming a second implementation of jj repository logic.
