# Manual coverage contract

The [coverage manifest](manual-coverage.json) lists jk commands, their manual chapters, executable
scenarios, recordings, and [UX findings](manual-ux-audit.md). The public manual lives in the companion
website repository. Keep the inventory and scenarios beside the source so command changes trigger
documentation review.

Coverage follows the implementation. A new native command, CLI form, selector role, or execution
option needs a manual entry and a scenario. The inventory includes all nine rebase scope/placement
combinations, both diff query shapes with all seven formats, every log template, multi-parent new,
multi-source squash, workspace operations, and the different abandon safety paths.

## Read the evidence accurately

Each command row names its starting state, affected objects, entry points, result, verification,
cancellation, failure, and recovery. Its source anchors identify the current builder or interaction
that establishes support. `manual_slug` locates its task chapter. `scenario_ids` names exact examples;
each scenario has a tape, video, and PNG/terminal-state pairs. `finding_ids` links concerns that need
review instead of treating confusing behavior as a documentation problem.

Support has three meanings:

- `native`: a dedicated jk command, control, startup option, or interaction implements the form.
- `command-mode`: the captured `:` interface forwards an explicitly noninteractive jj form. jk does
  not implement a second native version of that command. `coverage_scope: forwarding-example` marks
  syntax examples without an exact-form recording; these have no scenario IDs.
- `unavailable`: the requested interaction needs a facility that jk does not provide, such as a
  foreground editor or a native publication action. The manual gives the shell continuation.

A mapped scenario starts with `recording.status: planned`. Change it to `verified` after checking
repository outcomes and reviewing the exact recording's pixels, terminal text/styles, and video.
The evidence index retains these reviews and the source commit. Changing a tape resets its status.

Implemented controls can still have defects. `implementation_status: partially-working` identifies
such rows while keeping them in native coverage. For example, ordinary diff output does not produce
recognized hunk sections, and automatic container resolution fails with tested jj 0.45.1. Their
scenarios record the limitation or workaround; their findings describe the proposed fixes.

The [recording pipeline](manual-recordings.md) checks repository outcomes and retains the binary,
version metadata, logs, PNGs, state JSON, and media together. Website media is copied into the website
repository after review. PR artifacts are temporary evidence; their expiring URLs are not manual
assets.

## Review and prepare recordings for import

Download each CI evidence archive with its directory structure intact. Keep failed captures for
diagnosis. Inspect each PNG with its paired terminal state, then review the video for readable
captions, pacing, transitions, and the final frame. A sampled video review must state the sampling
method and timing checks. Record only observations made on those files.

Write review ledgers under `target/manual-recordings/reviews/`. Each ledger has `schema_version: 1`
and a `reviews` array. Each review contains:

- `scenario_id`, `source_tape_sha256`, and `capture_metadata_sha256` from the reviewed archive.
- `checkpoints`, with each checkpoint's `id`, `png_sha256`, and `state_sha256`.
- `pixel_review` and `text_style_review` on every checkpoint, each with `status`, `reviewer`,
  `reviewed_at`, `summary`, and any `findings` IDs.
- `video_review`, with the same review fields and the MP4's `sha256`.

Use `status: reviewed` for accepted evidence, `pending` for an unfinished review, or
`needs-rerecording` for a capture defect. A readable recording may show a product defect; name the
finding in the review. Misleading captions or a missing intended state require a new recording.
Conflicting reviews of identical bytes remain blocked while any review says `needs-rerecording`.
Resolve the disagreement in the ledger before preparing the index.

Build a report after adding reviews:

```sh
python3 scripts/prepare-manual-evidence.py --root . \
  --artifacts target/manual-recordings/ci-RUN_ID \
  --reviews target/manual-recordings/reviews \
  --output target/manual-recordings/reviews/candidate-evidence-index.json
```

Replace `RUN_ID` with the downloaded run. Repeat `--artifacts` to use several runs. The report prefers
an already reviewed capture whose source, tape, and artifact bytes still match. Otherwise it selects
a valid capture awaiting review, then a failed capture for diagnosis. Within each group, later
inputs take precedence; capture timestamps determine priority within an input. Repeat `--reviews`
for other ledger locations. Generated index files are ignored as review inputs.

The report rehashes the archived files, compares the full compiled-source inventory with the
checkout, checks preserved helper snapshots, and reads repository assertions. Reviews must match
the metadata, tape, PNG/state pair, and video hashes. `source_commit` is the CI build's `GITHUB_SHA`;
`pr_head_commit` separately records `JK_PR_HEAD_SHA`, since a PR build may use a synthetic merge.
Later helper edits do not invalidate a capture whose original helpers were preserved and unchanged
during recording.

The report lists missing, blocked, and unfinished reviews. Once all scenarios are `ready`, finalize
the coverage manifest before writing the durable index. Set each scenario's `recording` fields to
`status: verified`, `evidence_index: docs/manual-evidence-index.json`, the accepted `source_commit`,
and `pixel_review`/`text_style_review` references to its index entry. Take these values from the
accepted report. Keep a copy of the previous manifest until final index generation succeeds.

Generate the final index from those completed coverage bytes with a versioned website destination:

```sh
python3 scripts/prepare-manual-evidence.py --root . \
  --artifacts target/manual-recordings/ci-RUN_ID \
  --reviews target/manual-recordings/reviews \
  --website-version manual-VERSION \
  --require-ready --output docs/manual-evidence-index.json
```

`--require-ready` refuses to write the index while any scenario is unfinished. If it fails, restore
the previous coverage manifest and resolve the reported problem. The helper never changes coverage
status itself. Choose `VERSION` for the accepted collection; `website.path` becomes
`assets/manual/manual-VERSION/SCENARIO`.

Before importing, check that the final index describes the final coverage file:

```sh
python3 - <<'PY'
import hashlib
import json
from pathlib import Path

coverage = Path("docs/manual-coverage.json").read_bytes()
index = json.loads(Path("docs/manual-evidence-index.json").read_text())
assert index["coverage_manifest_sha256"] == hashlib.sha256(coverage).hexdigest()
assert all(row["status"] == "ready" for row in index["scenarios"])
PY
```

This is a final packaging check. Later tape edits may change the manifest while unchanged scenarios
retain their accepted reviews. Run the ordinary coverage check and record replacements in CI, then
prepare a new final index before the next import.

Pass the final index to the website importer through `--review-index`; the importer rechecks its
hashes and copies the accepted reviews beside the permanent assets. Keep the index in the source
repository and the videos, PNGs, states, and archive files in the website repository.

## Maintain the inventory

Run the source and collection check from the jk checkout:

```sh
python3 scripts/check-manual-coverage.py
```

When the companion checkout is available, check that every chapter also exists:

```sh
python3 scripts/check-manual-coverage.py --website-root ../jk-website
```

Use the actual website path if the repositories have a different directory layout.

After adding or changing a tape, synchronize its media paths and reset its review state:

```sh
python3 scripts/check-manual-coverage.py --sync-tapes
```

`--sync-tapes` registers the authored collection. It does not infer that a scenario covers a command;
review and update the command's `scenario_ids` separately. Keep those links exact. A screenshot of a
selector with one source does not prove multi-source execution, and captured command output does
not prove a native interaction exists.

The checker compares the declared public command modules with `jk-cli`, validates source markers
and line anchors, and fingerprints command builders, CLI arguments, Run options, command mode, and
key discovery metadata. Changes to these surfaces require a coverage review. After checking the
changed behavior and updating the manual/scenarios, accept the reviewed source version:

```sh
python3 scripts/check-manual-coverage.py --refresh-source-lock
```

The fingerprints exclude conventional trailing unit-test modules. They deliberately cover the
command contract rather than the whole application. Source anchors elsewhere still fail when a
symbol disappears or its recorded line moves. Do not refresh the lock solely to silence a failure.

`--allow-incomplete` is for concurrent authoring while scenario and finding files are being written.
It permits unfinished mappings; it must not be used as the CI gate.

## Coverage boundaries

The collection covers jk's implemented surface. Arbitrary jj commands accepted by generic command
mode do not create an unbounded requirement to record the whole jj CLI. Record the alternatives
needed to complete the manual's workflows, and label other illustrative syntax without claiming
exact-form evidence.

Internal probes and structured metadata passes are source dependencies of a user-facing form, not
extra user commands. For example, abandon's emptiness query belongs to the abandon paths; workspace
root detection belongs to workspace inspection. Source locking includes these builders so changes
to their behavior still trigger review.

The checker verifies that declarations remain connected. It cannot establish the meaning of a key
sequence, inspect pixels, or prove repository outcomes. Those are separate checks in the recording
pipeline and UX audit. Substantial workflow redesigns remain proposals until approved.
