# jk

`jk` is a log-first terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

The first release is intentionally narrow: keep a `jj` log-like view open, refresh it, select a
change, and inspect the relevant `show` or `diff` output. The project is being rebuilt from a clean
root so the published crate can be small, reviewed, and tested.

The long-form project rationale, design lineage, and release notes live in the repository
[`README.md`](https://github.com/joshka/jk#readme).

## Current Status

This crate is not yet a full TUI. It is the binary package that will grow the reviewed application
surface first. Placeholder library crates exist for possible future boundaries, but those
boundaries will stay empty until there is a concrete implementation reason to make them real.
