# Contributing

Thanks for helping improve `jk`. This project is early and intentionally keeps changes small,
reviewable, and easy to validate.

## Contribution Status

`jk` is still early and maintainer-led. The backlog is larger than the review bandwidth, and the
design is moving quickly, so broad issues and implementation PRs are usually premature right now.

Good contributions today are usually:

- small fixes for current `log` / `diff` behavior;
- documentation corrections for behavior that exists now;
- visual or accessibility feedback with screenshots;
- validation improvements, tests, or Betamax tapes for current workflows;
- product-direction feedback connected to the product plan or roadmap.

Please open an issue before starting larger implementation work. New workflows, mutation support,
keybindings, command mode, workspace screens, and architecture changes need design alignment before
code.

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

If a pull request makes sense, keep it focused on one current behavior, workflow, or documentation
topic. A good PR includes:

- a short summary of the user-visible or maintainer-facing change;
- the validation commands that were run;
- release or crates.io impact, when relevant;
- screenshots or terminal captures only for meaningful TUI rendering changes.

This repository uses `jj` for local version control, but GitHub PRs work normally from pushed
bookmarks or branches.

The product plan is intentionally ahead of the released implementation. Please do not treat every
planned feature as ready for a drive-by PR.

## Security Reports

Please do not report suspected vulnerabilities in public issues. Use GitHub Security Advisories as
described in [SECURITY.md](SECURITY.md).
