# Release Assets

## Why

The release workflow already builds `.tgz` archives and `.sha256` files for cargo-binstall and a
future Homebrew tap. The next step is proving those assets work from a real GitHub Release, not just
from a local package script.

## Current Repo State

- `crates/jk/Cargo.toml` has cargo-binstall metadata.
- `scripts/package-release-archive.sh` packages a root-level `jk` binary.
- `.github/workflows/release-plz.yml` uploads macOS and Linux archives for `x86_64` and `aarch64`.
- Homebrew formula maintenance is intentionally deferred, but the release assets should already be
  consumable by a tap formula.

## Steps

1. After the first release-plz-managed release, inspect the GitHub Release assets:
   - `jk-<version>-x86_64-apple-darwin.tgz`
   - `jk-<version>-aarch64-apple-darwin.tgz`
   - `jk-<version>-x86_64-unknown-linux-gnu.tgz`
   - `jk-<version>-aarch64-unknown-linux-gnu.tgz`
   - matching `.sha256` files
1. Test `cargo binstall jk` against the published release assets.
1. Draft a Homebrew formula from the release archive URLs and checksums, but do not add tap
   automation until release cadence and artifact naming have settled.
1. Keep archive naming in sync across:
   - `crates/jk/Cargo.toml`
   - `.github/workflows/release-plz.yml`
   - `scripts/package-release-archive.sh`

## Done When

- `cargo binstall jk` installs from a GitHub Release archive instead of compiling.
- A Homebrew formula can install `jk` from release assets and smoke-test `jk --version`.
