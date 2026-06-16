# jk

> Prototype archive: `joshka/prototype-3-guided` is the more guided follow-up
> experiment. It narrows the product direction around a log-first, single-view
> TUI and adds stronger docs, tests, feature roots, and exact-target action
> checks. Mine it for tested behavior, module ownership ideas, and product
> language; do not treat the whole broad surface as reviewable enough for the
> first curated crate release.

`jk` is a terminal UI for [Jujutsu](https://github.com/jj-vcs/jj) that keeps your day-to-day `jj`
flow in one place.

It shells out to `jj` instead of rebuilding a separate repository model, so your templates, colors,
graph symbols, diff formatting, and wording stay intact. The goal is simple: stay in the terminal,
move through history quickly, inspect changes, run a few high-value actions, and get back to work.

## Why This Exists

If you already use `jj`, you know the rhythm: run a command, inspect output, run another command,
repeat. That works, but it also means a lot of shell ping-pong.

`jk` keeps that rhythm while making the common loop faster:

- start in a log-first home view;
- move by revision item instead of line noise;
- open `show` and `diff` without leaving the app;
- check status, bookmarks, workspaces, conflicts, or operation history in focused views;
- run selected `jj` actions when the target is exact and the flow is clear.

## What You Get

`jk` is a good fit if you want:

- a log-first interface that still feels like `jj`;
- one active view at a time instead of a dashboard full of panes;
- fast keyboard navigation for inspecting history and patches;
- search, copy, and drill-down flows around rendered `jj` output;
- safety-first mutation flows that prefer exact targets, previews, or staying disabled over
  guessing.

Today that includes:

- graph, `log`, `show`, `diff`, `status`, `resolve`, `bookmarks`, `workspaces`, and `operation-log`
  views;
- back, refresh, search, and view switching;
- copy menus for revisions, files, bookmarks, and operation-log rows;
- focused `jj` actions including fetch, push, describe, commit, bookmark mutation, undo and redo,
  and selected graph actions such as `new`, rebase, squash, absorb, abandon, and restore when `jk`
  has an exact target.

## Quick Start

Prerequisites:

- a recent Rust toolchain with edition 2024 support;
- `jj` on `PATH`;
- `just` if you want the helper commands.

Run `jk` inside a `jj` repository:

```sh
just run
```

You can also invoke Cargo directly:

```sh
cargo run
cargo run -- log
cargo run -- show @
cargo run -- diff -r @
cargo run -- status
cargo run -- bookmarks
cargo run -- workspaces
cargo run -- operation-log
```

## Your First Few Minutes

If you want to know whether `jk` is useful for your workflow, try this:

1. Start in the graph and move with `j` and `k`.
1. Press `s`, `l`, or `Right` to open `show` for the selected change.
1. Press `d` to switch to `diff`.
1. Press `h` or `Left` to go back.
1. Press `S` for status or `B` for bookmarks.
1. Press `?` anywhere to see the commands for the current view.

If that loop feels faster and calmer than bouncing between `jj log`, `jj show`, `jj diff`, and
`jj status`, `jk` is probably pulling its weight.

## Safety And Limits

`jk` is strongest as a navigation and inspection tool with a focused set of high-value actions
around it.

- Direct low-friction actions such as `f`, graph `gf`, and `c` execute immediately and show the
  resulting `jj` output.
- Riskier or more stateful flows prefer preview, confirmation, or an exact metadata-backed target.
- Bookmark mutation, restore, revert, and similar actions stay disabled when `jk` cannot identify
  the exact target it would pass to `jj`.
- Some `jj` workflows still belong in the CLI first. `jk` does not try to replace the full command
  line.

## Learn More

- [`docs/reference/README.md`](docs/reference/README.md) describes the current screens, workflows,
  and view model.
- [`docs/tutorials/README.md`](docs/tutorials/README.md) has short walkthroughs for common loops.
- [`docs/demo/README.md`](docs/demo/README.md) documents the tracked demo setup used by some
  examples.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) covers contributor workflow, repo references, and development
  commands.
