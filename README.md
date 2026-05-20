# jk

`jk` is a log-first Rust TUI for [Jujutsu](https://github.com/jj-vcs/jj). It shells out to `jj`, so
the app keeps the user's templates, colors, graph symbols, diff formatting, and CLI wording instead
of recreating a separate repository model. The current focus is the everyday loop of scanning
history, drilling into `show`/`diff`, checking status, and returning without leaving the terminal.

## What Ships Today

`jk` currently ships these surfaces:

- a default graph view and explicit `log`, `show`, `diff`, `status`, `bookmarks`, and
  `operation-log` startup views;
- one-active-view navigation with `show`/`diff` drill-down, back, refresh, and search;
- focused utility views for files, bookmarks, status, and operation history;
- copy menus for selected revisions, files, bookmarks, and operation-log rows;
- direct `jj` actions for fetch (`f`) and graph `c` / `jj new trunk`;
- preview/result-gated (guided) mutation flows for push, operation undo/redo, graph action-menu
  `new`, abandon, rebase, and related action-menu flows where implemented.

This README only describes shipped behavior. Planned packets live in
[`docs/plan/next-implementation-slices.md`](docs/plan/next-implementation-slices.md), and shipped
progress is tracked in [`docs/plan/progress.md`](docs/plan/progress.md).

Short walkthroughs for the supported daily loops live in
[`docs/tutorials/`](docs/tutorials/README.md). The tracked demo setup used by some of those examples
is documented in [`docs/demo/`](docs/demo/README.md).

## Install And Run

Prerequisites:

- a recent Rust toolchain with edition 2024 support;
- `jj` on `PATH`;
- `just` if you want the helper commands.

Run `jk` from a `jj` repository:

```sh
just run
```

The helper runs `cargo run` from the repository root. You can also invoke Cargo directly:

```sh
cargo run
cargo run -- log
cargo run -- log -r 'mine() & mutable()'
cargo run -- show @
cargo run -- diff -r @
cargo run -- status
cargo run -- bookmarks
cargo run -- operation-log
```

## Help And Keys

Press `?` for the generated in-app help overlay. It reflects the current bindings for the active
view, which is the quickest way to see the exact key surface without consulting the source.

Global keys:

- `q` or `Esc` quits;
- `/` starts search;
- `n` and `N` move between search matches;
- `r` refreshes the current view;
- `h` or `Left` goes back;
- `L` switches to `jk log`;
- `J` switches back to the default `jk` view;
- `S` opens status;
- `B` opens bookmarks;
- `O` opens operation log;
- `f` fetches;
- `p` opens push;
- `y` opens the copy menu;
- `v` opens the diff-format menu;
- `W` prompts for a custom graph revset.

Graph view:

- `j` and `k` move;
- `g` and `G` jump to the first or last item;
- `s`, `l`, or `Right` open `show`;
- `d` opens `diff`;
- `Space` toggles exact revision selection for preview-first actions;
- `a` opens the action menu;
- `c` creates a new working-copy change from trunk;
- `w` cycles the graph log mode.

Show and diff views:

- `j` and `k` scroll;
- `Space`, `PageDown`, and `Ctrl-f` page down;
- `Shift-Space`, `PageUp`, and `Ctrl-b` page up;
- `[` and `]` jump between files;
- `l` opens the file list;
- `s` switches from diff to show;
- `d` switches from show to diff.

Bookmarks and operation log are item-based views. Bookmarks open the selected change with `s` or
`Enter` when a target change id is present. Operation log opens operation `show` with `s` or
`Enter`, operation `diff` with `d`, and global repo recovery with `u` for undo and `Ctrl-r` for
redo.

## Safety Notes

- Direct low-friction mutation actions (`f` and `c`) execute immediately and show the resulting
  status/output from `jj`; where recovery is available, `jj`-level undo is still the intended
  fallback.
- Guided flows (for example push, operation undo/redo, graph action-menu `new`, abandon, and rebase)
  show preview or result text before execution.
- Where `jj` exposes an exact target, `jk` keeps that target exact instead of guessing from a label.
- Successful mutations keep the undo path visible when `jj` supports one.
- Operation-log undo and redo act on the repo cursor, not on the selected row.
- Push is previewed and does not ship a force-push shortcut.
- If `jj` rejects an action or a target is unavailable, `jk` shows a readable result or status error
  instead of pretending success.

## Media And Captures

- Generated screenshots, GIFs, and demo repos are not committed.
- The tracked capture specs and setup notes live in [`docs/demo/`](docs/demo/README.md).
- Tutorial walkthroughs live in [`docs/tutorials/`](docs/tutorials/README.md).
- If capture artifacts are added later, keep the generated output under ignored `target/vhs` or host
  it externally.

## Contributor References

- [`docs/product-direction.md`](docs/product-direction.md) for the product shape.
- [`docs/plan/progress.md`](docs/plan/progress.md) for shipped packet history.
- [`docs/plan/next-implementation-slices.md`](docs/plan/next-implementation-slices.md) for planned
  packets and their boundaries.
- [`docs/agent/documentation.md`](docs/agent/documentation.md) for doc style and truthfulness.
