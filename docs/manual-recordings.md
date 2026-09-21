# Manual recordings and review evidence

The manual scenarios exercise the workflows registered in
[`manual-coverage.json`](manual-coverage.json). Every pull request records the entire collection,
including proposed tape changes. Each scenario gets a gallery and a separate evidence archive.
Scenarios live under `tapes/manual/`, grouped by the task they teach. Adding a registered tape adds
it to the next PR run automatically; CI does not maintain a smaller smoke-test list.

## Run the complete collection locally

Install `jj`, Betamax, and the fonts and ffmpeg required by Betamax, then run:

```sh
python3 scripts/manual-recordings.py run
```

The runner builds this checkout with `cargo build --locked -p jk` and uses that executable by absolute
path. It never selects an installed `jk` from `PATH`. To reuse a binary you have just built:

```sh
python3 scripts/manual-recordings.py run --binary "$PWD/target/debug/jk"
```

Each run creates a fresh directory below `target/manual-recordings/`. Its `index.html` links every
scenario, including failures. The runner continues after a failed scenario and exits unsuccessfully
if any scenario or evidence check fails. A failed build keeps `build.log` and `collection.json`.
The default timeout is ten minutes per scenario; use `--timeout` to change it locally.

During iteration, select one or more scenarios without changing the full CI collection:

```sh
python3 scripts/manual-recordings.py run \
  --binary "$PWD/target/debug/jk" \
  --scenario change-lifecycle \
  --scenario workspaces-lifecycle
```

Fixtures create disposable repositories and isolated jj identity, configuration, and signing. The
source checkout is never the repository under test. `manual-fixture.sh` exposes `repo` and related
fixture paths inside the hidden setup section. The runner supplies `JK_SOURCE_REPO`, `JK_BIN`,
`JK_MANUAL_ARTIFACT_DIR`, and `JK_MANUAL_SCENARIO` to every tape.

Historical Run Options scenarios declare `# PreparedFixture run-options`. The runner creates that
disposable fixture before recording, keeps `fixture-preparation.log` and explicit `fixture-env.sh`,
and substitutes its real initial operation ID for `{{INITIAL_OPERATION}}` in the prepared tape.
The tape reuses that exact fixture through `JK_MANUAL_PREPARED_FIXTURE`; it does not type a fabricated
ID or create a different repository when recording starts.

## What a scenario preserves

Each scenario evidence directory contains:

- `source.tape` and `capture.tape`, with the original interaction and the exact output paths used.
- `source-scripts/`, retaining the fixture, assertion, runner, and supporting shell/Python scripts
  byte-for-byte, with original paths and hashes recorded in the metadata.
- An MP4 recording with readable typing and pauses after semantic screen assertions.
- PNG checkpoints and same-stem terminal-state JSON, including text and styled spans.
- `fixture.json`, recording the disposable repository and its starting commit and operation.
- One or more `*-assertions.json` files proving cancellation, successful mutations, failures, or
  recovery against repository state. Every stage must pass; a later success cannot hide a failure.
- `capture-metadata.json`, with source hashes, binary hashes, executable versions, timestamps, result,
  file sizes, and artifact hashes. CI also includes PR/merge SHA and workflow-run identifiers.
- `render.log` locally, or `action-diagnostics/` with action logs and the render manifest in CI.
- `checkpoint-summary.json` and `review.html`, placing pixels beside their terminal text and styles.

CI builds `jk` once and distributes that exact binary to all scenarios. Each render job checks its
hash and source hashes against `build-provenance.json`. The original binary, build log, and provenance
remain in the `manual-source-binary` artifact. CI uses jj 0.45.1 and checksum-verified Betamax 0.1.21;
local runs record the actual installed versions so differences are visible.

The evidence verifier rejects missing or empty media, missing PNG/State pairs, empty terminal text,
missing styled spans, absent or failed repository assertions, failed rendering, and gallery size
violations. These checks do not certify visual quality. The metadata explicitly leaves visual review
pending until someone inspects the actual output.

The runner checks helper hashes again when capture ends. Edits during a recording fail its evidence
check; freeze authoring and rerun the affected scenarios. Completed evidence remains valid when the
checkout changes later because its source bytes are retained. Older recordings with hashes alone are
explicitly labeled as lacking source snapshots. They can be backfilled only when the original hashes
match the available bytes:

```sh
python3 scripts/manual-recordings.py snapshot-sources --directory PATH_TO_SCENARIO_EVIDENCE
```

If backfill reports a mismatch, rerun the scenario; do not copy newer helpers into older evidence.

## Review pixels and repository behavior

Inspect each video's transitions, readability, and pacing. Playback or decoded sample frames with
tape timing can establish the sequence and pauses; play sections where samples leave uncertainty.
Record the method used. Read every PNG alongside the terminal text and
styles. Show PNG proof inline in the Codex rollout, using the new run's path to avoid stale cached
images. Open generated images in a browser only when requested.

For a dialog, check that its surface is opaque, its text remains readable, and actual repository
content is visible around it. Check operand names, selected revisions, destination, affected paths,
and the consequence of confirmation. Look for clipped text, competing emphasis, unnecessary steps,
missing feedback, and navigation that makes the next action hard to discover.

Review the matching assertion stages before declaring the workflow sound. A success message alone
does not prove that the intended revision, files, bookmark, remote, or workspace changed. A cancel
checkpoint needs unchanged repository state; recovery needs restored state. Record design findings
against scenario and command IDs in the coverage ledger. Keep substantial redesigns as proposals
with the observed failure and supporting checkpoint.

## Add or extend a scenario

1. Add a tape under the appropriate `tapes/manual/<family>/` directory with a globally unique
   basename using lowercase letters, digits, or hyphens. The family-prefixed gallery name must fit
   within 40 characters; the planner checks it.
1. Declare exactly one MP4 `Output`. Pair every PNG `Screenshot` with a same-stem JSON `State` after
   a semantic `Wait+Screen` assertion. Use distinct checkpoint names, readable pauses, and hidden
   fixture setup and cleanup.
1. Source `manual-fixture.sh` with a suitable fixture and use its absolute `bin` and disposable `repo`.
1. Check repository outcomes with `manual-assert.sh` after cancellation, completion, and recovery as
   appropriate. Assertion JSON records the scenario ID, stage, status, and named boolean checks.
1. Register its path and supported forms in `docs/manual-coverage.json`. The planner rejects omitted
   tapes, nonexistent registered tapes, duplicate basenames, and unpaired checkpoint names.
1. Run the scenario, inspect pixels and state, then run the complete collection before review.

Validate the collection and runner without recording:

```sh
python3 scripts/check-manual-coverage.py
python3 scripts/manual-recordings.py plan
python3 -m unittest discover -s scripts -p 'test_manual_recordings.py'
```

## PR galleries and security boundary

`.github/workflows/betamax.yml` runs on every PR with read-only repository permissions and no secrets.
It builds the exact source binary, discovers every registered scenario, and runs a matrix with
failure isolation. Job labels and gallery names include the family and scenario. The pinned Betamax
action publishes one gallery per scenario and its diagnostics. An unconditional upload preserves the
complete evidence directory, including partial outputs from a failed or timed-out tape. Cancellation
or infrastructure failure can prevent artifact upload; rerun that job if its archive is absent.

One scenario corresponds to one render invocation. The planner permits at most 19 PNG checkpoints
plus one MP4: the action limit is 20 media files, 10 MiB per file, and 40 MiB total per invocation.
The verifier enforces the byte limits after rendering. Split an oversized scenario at a meaningful
task boundary rather than dropping its checkpoints. The matrix supports up to 256 scenarios.

`.github/workflows/betamax-report.yml` runs separately after rendering, with Actions read and PR write
permissions. It uses only the pinned report action; it never checks out PR code, restores PR caches,
or executes fixture scripts. Artifact mode groups all matrix galleries in one PR comment. Inline
attachment mode would impose a 20-file and 40-MiB aggregate cap across the whole collection, so it is
not used for the manual suite.

CI evidence expires after the configured 30-day retention, subject to repository policy. Permanent
manual media belongs to the website repository's `public/assets/` directory and follows that
repository's LFS rules. The website must use its published asset URLs, never expiring Actions links
or signed blob URLs. Copy only reviewed recordings to the website and retain the matching source
scenario, assertions, provenance, and audit links in the coverage ledger.
