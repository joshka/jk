# Contributing

Thanks for helping improve `jk`. This project is early and intentionally keeps changes small,
reviewable, and easy to validate.

## Development Setup

Install the Rust toolchain declared in `Cargo.toml`, plus the local tools used by CI:

```sh
cargo install just
cargo install cargo-deny
cargo install cargo-udeps
```

The formatting task uses nightly rustfmt:

```sh
rustup toolchain install nightly --component rustfmt
```

## Local Checks

Use `just` for common tasks:

```sh
just --list
just check
just test
just clippy
just lint-md
```

Before opening a release-facing or broad PR, run:

```sh
just release-check
```

For narrow documentation-only changes, `just lint-md` is usually sufficient.

## Pull Requests

Keep pull requests focused on one behavior, workflow, or documentation topic. A good PR includes:

- a short summary of the user-visible or maintainer-facing change;
- the validation commands that were run;
- release or crates.io impact, when relevant;
- screenshots or terminal captures only for meaningful TUI rendering changes.

This repository uses `jj` for local version control, but GitHub PRs work normally from pushed
bookmarks or branches.

## Security Reports

Please do not report suspected vulnerabilities in public issues. Use GitHub Security Advisories as
described in [SECURITY.md](SECURITY.md).
