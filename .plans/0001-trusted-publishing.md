# Trusted Publishing

## Why

The `jk`, `jk-cli`, `jk-core`, and `jk-tui` crate names are now published. The bootstrap API token
has done its job and should not remain as a long-lived release credential.

crates.io Trusted Publishing lets GitHub Actions exchange its OIDC identity for a short-lived
publishing token. `release-plz` supports that flow directly, so this repo should not add
`rust-lang/crates-io-auth-action` or keep `CARGO_REGISTRY_TOKEN` in the workflow.

References:

- <https://crates.io/docs/trusted-publishing>
- <https://release-plz.dev/docs/github/quickstart>

## Current Repo State

- `.github/workflows/release-plz.yml` already uses `environment: release`.
- The `release` job already has `permissions.id-token: write`.
- The workflow does not set `CARGO_REGISTRY_TOKEN`.
- First manual publishes are complete, so all four crates can now be configured for trusted
  publishing on crates.io.

## Steps

1. In crates.io, add a GitHub Actions trusted publisher for each crate:
   - `jk`
   - `jk-cli`
   - `jk-core`
   - `jk-tui`
1. Use these trusted publisher claims for each crate:
   - owner: `joshka`
   - repository: `jk`
   - workflow: `release-plz.yml`
   - environment: `release`
1. Confirm the GitHub repository has a `release` environment. Add required reviewers there if
   manual release gating is desired.
1. Remove any `CARGO_REGISTRY_TOKEN` GitHub secret after a release run successfully publishes via
   trusted publishing.
1. Revoke/delete the local crates.io API token used for the bootstrap publish.

## Done When

- A release-plz publish succeeds without any crates.io API token secret.
- The four crates show trusted publishing configured for this repository and workflow.
- No long-lived crates.io token remains in GitHub secrets or local shell/session state.
