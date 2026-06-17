# jk

`jk` is a log-first terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

The first release is intentionally narrow: keep a `jj` log-like view open, refresh it, move by
change, and expand the selected change's description inline. The project is being rebuilt from a
clean root so the published crate can be small, reviewed, and tested.

The long-form project rationale, design lineage, and release notes live in the repository
[`README.md`](https://github.com/joshka/jk#readme).

## Current Status

This crate is the binary package for the reviewed application surface. The workspace also contains
small library crates for shared log records, `jj` process integration, and Ratatui view state.

Bare `jk` follows `jj`'s configured `ui.default-command` when that command is log-like enough for
the semantic template pass. Use `jk log` to open the explicit log path.

## Commands

```sh
jk
jk log
jk -R /path/to/repo -n 20
jk log --repository /path/to/repo --limit 20
```

The TUI currently supports manual refresh, movement by change, page and edge movement, switching
between the configured default `jj` view and explicit `jj log`, and inline expansion for the
selected change's description.
