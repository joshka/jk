# Pull request previews

The `Betamax` workflow builds the PR's `jk`, prepares a disposable jj history, and records
`preview.tape` on Ubuntu 24.04. The gallery contains a GIF and PNG previews of the log, diff, and
contextual help. Generated media stays in workflow artifacts, outside the tracked repository.

The separate `Betamax report` workflow updates one PR comment with the gallery link. It must be
merged into the default branch before GitHub can trigger it. It uses the built-in Actions token;
no attachment token or other secret is required. Downloading the gallery requires GitHub sign-in.

To run the same tape locally with `jj`, `betamax`, and Rust installed, start at the checkout root:

```sh
cargo build --locked -p jk
bash scripts/prepare-preview.sh
BETAMAX_WORKING_DIRECTORY="$PWD" betamax run tapes/ci/preview.tape \
  -o target/betamax-preview/preview.gif
```

The fixture script creates a new repository under `target/betamax-preview/` on every invocation.
It does not change the checkout's jj history. Screenshots are written beside the fixture and can
be removed with the rest of `target/`.

Add portable preview tapes under `tapes/ci/`; the other tapes retain their local dogfooding setup.
Keep setup hidden and wait for expected screen text before capturing each view. Both workflows
pin the same [Betamax Action](https://github.com/joshka/betamax-action) commit; update them together.
