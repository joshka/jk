# Refs and Remotes Workflows

Status: implemented through push dry-run; real push remains gated

Owner: Luna

Implementation note: the completed workflow uses a small in-app remote chooser, exact-pattern
arguments, and local jj fixture repositories. Fetch refreshes bookmarks and retains command output.
Command mode rejects these mutation/network commands in favor of the preview paths, including
push dry-run. Real push remains disabled. The original design below records intended review and
acceptance criteria; implementation file boundaries differ where existing app patterns were reused.

Scope: bookmarks, explicit fetch, and push dry-run after the current mutation and history foundations

## Outcome

Implement the refs/remotes roadmap in five dependency-ordered review units:

1. inspect local and remote bookmarks and open their target commits;
1. preview and confirm local bookmark create, move, and delete operations;
1. choose one named remote and run fetch with durable output and command history;
1. choose one local bookmark and one named remote, then run and display push dry-run;
1. optionally enable a real push only after every gate in [Real Push Gate](#real-push-gate) passes.

The default plan stops after push dry-run. A real push is a separate final review unit, not an
implicit part of the dry-run change.

## Current Foundations

The implementation should reuse the foundations already present on `main`:

- `jk_core::JjCommandSpec` owns process argv, global options, titles, execution mode, safety class,
  and refresh policy.
- `jk_cli::JjCommandRunner` and `RecordingJjCommandRunner` provide injectable execution and bounded
  in-memory command history.
- `jk_tui::WorkspacesView` is the closest provider-neutral selectable object-screen pattern.
- `jk_tui::CommandPreviewView`, `jk::PendingCommandPreview`, and the mode stack own current local
  mutation previews.
- `jk::AppView::CommandOutput` and `command_mode_snapshot` already preserve successful and failed
  stdout/stderr in a focused rendered view.
- `jk::ViewStack` preserves the parent screen while target details or command output are open.

Do not copy code from another active workspace or depend on unlanded shared-selector work. Build the
small bookmark and remote selection state needed here from the landed patterns. A later change can
adopt a shared selector after that selector lands independently.

## Verified jj 0.44.0 Contract

The command forms below were verified against the installed `jj 0.44.0` help. Re-run the same help
checks before implementation in case Luna's installed version differs:

```text
jj bookmark list --all-remotes -T 'json(self) ++ "\n"'
jj bookmark create --revision REV NAME
jj bookmark move --to REV NAME
jj bookmark delete NAME
jj git remote list
jj git fetch --remote REMOTE
jj git push --remote REMOTE --bookmark BOOKMARK --dry-run
```

`json(self)` serializes one bookmark reference per line. In jj 0.44.0 the useful fields are:

```json
{"name":"main","target":["COMMIT_ID"]}
{"name":"main","remote":"origin","target":["COMMIT_ID"],"tracking_target":["COMMIT_ID"]}
```

The `target` array preserves jj's merge terms: added, removed, added, and so on. A normal bookmark
has one commit ID; a deleted target is `[null]`. A conflict can contain `null` terms and must stay
conflicted even when it has only one added commit ID. Keep added and removed IDs separate, retain
conflict state, and shorten full IDs only for display. The presence of `tracking_target` indicates
tracking even when its value is `[null]` after a local deletion. Compare the full target merge terms
when reporting whether a tracked remote is synchronized.

Important upstream behavior to preserve in UI text and tests:

- `bookmark move` cannot create a missing bookmark.
- Moving backwards or sideways fails unless `--allow-backwards` is explicitly supplied. Do not
  expose that flag in the first mutation slice.
- `bookmark delete` does not abandon revisions; it records a deletion that may propagate on a later
  push. `bookmark forget` is different and is out of scope.
- Fetch can make remote-unreachable commits abandoned in local remote state. It is a network read,
  but it is not a no-op on local repository metadata.
- Push does not derive its destination from tracked bookmarks. Always pass `--remote` explicitly.
- `--no-integrate-operation` is not a push simulation and must never be added to fetch or push.
- `git push --dry-run` may contact the remote but must not move, create, or delete remote refs.

## Safety Invariants

These invariants apply across all review units:

- Construct argv as `OsString` data and execute it directly. Never build or run a shell command
  string from a bookmark name, remote name, URL, or revset.
- Preserve global-option ordering through `JjCommandSpec::process_argv()`.
- Resolve bookmark and remote choices from loaded repository data, not from guessed defaults.
- Pass `--remote REMOTE` for every fetch, push dry-run, and possible real push.
- Pass exactly one `--bookmark BOOKMARK` in the first push slice.
- Keep `--all`, `--tracked`, `--deleted`, `--change`, `--named`, `--option`, and every `--allow-*`
  push flag out of the first push workflow.
- Do not expose bookmark mutation actions for remote-only rows.
- Do not allow a conflicted, deleted, or multi-target local bookmark into the first push workflow.
- Opening or canceling a prompt, chooser, or preview must spawn no command and add no history record.
- Record every confirmed bookmark mutation, fetch, and push dry-run through
  `RecordingJjCommandRunner`.
- Keep both successful and failed fetch/dry-run output visible in a pushed output/result view.
- Preserve the last usable bookmark snapshot when refresh or command execution fails.
- Tests and demos may use only temporary local fixture repositories and local bare remotes. Never
  fetch from or push to this checkout's configured remotes.
- Command mode must not remain a bypass around the refs/remotes safety policy.

## Architecture

### Process integration

Add two coherent `jk-cli` owners:

- `crates/jk-cli/src/bookmarks.rs`
  - `JjBookmarks`
  - `BookmarkListQuery`
  - `BookmarkSnapshot` and `BookmarkRef`
  - `BookmarkMutation`
  - list parsing and typed list/create/move/delete specs
- `crates/jk-cli/src/git_remote.rs`
  - `JjGitRemote`
  - `GitRemote`
  - remote-list parsing
  - typed fetch and push-dry-run specs
  - an optional real-push spec builder that is private until its gate passes

Export the public integration types from `crates/jk-cli/src/lib.rs`. Prefer `git_remote.rs` over a
broad `git.rs`: this module owns the named-remote fetch/push boundary, not every `jj git` command.

Use `serde_json` already present in `jk-cli` to parse the JSON-lines bookmark template. Return
structured `thiserror` errors that distinguish spawn failure, unsuccessful command output, malformed
JSON, unsupported bookmark shape, missing remote, and failed network command.

### Provider-neutral TUI state

Add these focused views under `jk-tui`:

- `bookmark_view.rs` for local/remote bookmark rows, selection, refresh preservation, target-open,
  and action availability;
- `remote_picker.rs` for an explicit list of remote names used by fetch and push;
- `push_preview_view.rs` only if `CommandPreviewView` cannot clearly show dry-run output and the
  frozen remote/bookmark selection without accumulating command-specific branches.

One bookmark-reference row should show:

- local, tracked remote, untracked remote, deleted, or conflicted status;
- bookmark name and remote name when present;
- all target commit IDs, visibly marking multi-target conflicts;
- tracking/synchronization state derived from the JSON row;
- disabled-action feedback rather than silently doing nothing.

Keep selection state in the view. Preserve selection by `(bookmark name, remote name)` on refresh,
then clamp to the nearest available row. Do not store repository or process objects in `jk-tui`.

### App state and orchestration

Add `AppView::Bookmarks` and the minimal prompt modes needed for:

- bookmark name input;
- target revset input;
- explicit remote selection;
- push dry-run result/confirmation state.

Prefer narrow enums over callbacks or a broad context object. For example, a bookmark input mode can
carry `BookmarkInputPurpose::{Create, Move}` and the already-selected source row. A remote chooser
can carry `RemoteChoicePurpose::{Fetch, PushDryRun}` and the selected bookmark when applicable.

Generalize the confirmed-command boundary just enough that a preview can return to and refresh its
owning view. The current `confirm_command_preview` assumes a log source and always refreshes the log.
Do not add bookmark commands to that function with more log-specific conditionals. Either:

1. make `PendingCommandPreview` own a small `PreviewOwner`/success transition enum; or
1. add a bookmark-specific confirmation function that uses the same recorder and error formatting.

Choose the smaller shape after writing the state-transition tests. Do not introduce a general
workflow framework in this roadmap slice.

### Command history identity

Extend `crates/jk-core/src/command_history/identity.rs` with explicit families and sources:

- command families: bookmark, git fetch, and git push;
- source views: bookmarks, fetch, and push preview/result;
- source actions: bookmark list, target inspection, create, move, delete, fetch, push dry-run, and
  optional confirmed push.

History should identify `jj git fetch` and `jj git push` separately rather than grouping both under
`Other("git")`.

### Command-mode resolver

Before exposing push dry-run, add a resolver in `crates/jk/src/command_mode.rs` that classifies the
in-scope command families before execution. Suggested result shape:

```rust
enum JjCommandResolution {
    RunReadOnly(JjCommandSpec),
    RequirePreview(JjCommandSpec),
    RouteToRefsWorkflow(RefsRoute),
    Reject(String),
}
```

The exact names can change. The required policy is:

- `bookmark create`, `bookmark move`, and `bookmark delete` cannot run immediately from `:`;
- `git fetch` cannot run immediately without explicit network confirmation;
- `git push` without `--dry-run` is rejected until the real-push gate lands;
- `git push --dry-run` must still require an explicit remote and exactly one bookmark, or route the
  user to the canonical `P` workflow;
- the resolver must classify flags without relying on their textual order;
- unknown commands keep the current command-mode behavior; broader mutation classification remains
  a separate roadmap concern.

This is a narrow closure of the new feature's bypass, not a claim that all arbitrary command-mode
mutations are now fully classified.

## Review Unit 1: Bookmark Inspection

Description:

```text
Add bookmark inspection

List local and remote bookmark references from jj's JSON template and add a
focused bookmark screen that opens normal target commits without depending on
unlanded selector work.
```

Implementation:

1. Add `JjBookmarks::list_spec()` using `bookmark list --all-remotes` and the verified JSON-lines
   template. Force color off for machine output while preserving repository/global options.
1. Parse every non-empty line independently so the error can report the failing line number.
1. Model missing, one, and multiple targets without discarding conflict information.
1. Add `BookmarksView` with provider-neutral rows, selection, refresh, error, and status behavior.
1. Add `B` as the global entry point and `AppView::Bookmarks` as a pushed view.
1. Let `Enter` open the normal target through the existing `JjShow`/`RenderedView` path using the
   full commit ID. On no target or multiple targets, keep the bookmark screen and show a specific
   status explaining why target inspection is unavailable.
1. Add bookmark help/chrome metadata and make `Esc`/`Backspace` return to the preserved parent.
1. Record list, refresh, and target-show commands with their real source view/action.

Acceptance tests:

- list specs keep global options before `bookmark list` and use color-free JSON output;
- JSON parsing covers local, tracked remote, untracked remote, deleted, and conflicted rows;
- malformed JSON reports its line and keeps the previous snapshot visible;
- empty repositories render a useful empty state;
- selection survives refresh by name and remote;
- `B` opens Bookmarks from every supported root/inspection screen;
- `Enter` opens exactly one normal full commit target;
- `Enter` on missing or conflicted targets runs nothing and shows a useful message;
- `Esc` returns to the prior screen with its state preserved;
- history attributes list/refresh/show to Bookmarks.

## Review Unit 2: Bookmark Mutation Previews

Description:

```text
Preview bookmark mutations

Resolve create, move, and delete into typed jj command specs and require a
visible confirmation before changing local bookmark metadata.
```

First-slice behavior:

- `c`: prompt for a new name, then a target revset defaulting to `@`, then preview
  `jj bookmark create --revision REV NAME`;
- `m`: only on a present local single-target row, prompt for a destination revset defaulting to
  `@`, then preview `jj bookmark move --to REV NAME`;
- `x`: only on a present local row, preview `jj bookmark delete NAME` with the explicit statement
  "Deletes the bookmark; does not abandon its revisions; may propagate on a later push.";
- do not expose `--allow-backwards`, multiple-name mutation, rename, advance, set, forget, track, or
  untrack.

Classify create and move as local metadata. Classify delete as destructive local metadata in the
preview even though jj can recover it through operation history; its future remote consequence is
important enough to make the warning visually distinct.

Acceptance tests:

- the resolver emits exact argv for create, move, and delete;
- names and revsets containing shell metacharacters remain one argv element and are never evaluated;
- empty names/targets remain in the prompt with validation text and run nothing;
- remote-only, deleted, and conflicted rows cannot move or delete;
- backwards movement is not silently enabled;
- opening/canceling each preview runs nothing and records nothing;
- confirming uses the recording runner once, refreshes Bookmarks on success, and preserves the
  selected identity when it still exists;
- create/move/delete failure keeps the old snapshot, displays stderr first, and records the failure;
- delete preview includes the non-abandonment and future-push consequence;
- command-mode attempts route to the preview workflow or are rejected instead of running directly.

## Review Unit 3: Explicit Fetch

Description:

```text
Add explicit fetch workflow

Require a named remote selection before fetch, retain command output in a
focused view, and record both successful and failed network reads.
```

Implementation:

1. Load remote names through `jj git remote list`. Parse only the name needed for selection; do not
   add remote URLs to durable app state in this slice.
1. Bind `F` to an explicit remote chooser. Even with one remote, show or preview the resolved remote
   rather than silently relying on jj defaults.
1. Build only `jj git fetch --remote REMOTE` in the first slice.
1. Add an explicit network-read confirmation policy. Prefer a new
   `ExecutionMode::ConfirmNetworkRead` and `CommandPreviewWarning::NetworkRead` over describing fetch
   as an ordinary local mutation.
1. On confirmation, record the command and push a persistent output view containing command, remote,
   exit status, stdout, and stderr for both success and failure.
1. Refresh bookmark data after a successful fetch without replacing the output view. Returning to
   Bookmarks should reveal the refreshed snapshot and preserve selection when possible.
1. If no remotes exist, run nothing and explain that remote management is a later workflow.

Acceptance tests:

- remote parsing covers ordinary names, URLs containing spaces after the name, empty output, and
  malformed lines;
- every fetch spec contains exactly one explicit `--remote` and is `NetworkRead`;
- choosing/canceling a remote runs nothing and records nothing;
- success and non-zero exit both remain visible with stdout/stderr and command context;
- spawn errors are distinct from unsuccessful exits;
- history records the chosen remote through argv without retaining a separately guessed default;
- command mode cannot execute fetch immediately around the confirmation boundary;
- successful fixture fetch updates the client's remembered remote bookmark;
- failed fixture fetch preserves the prior bookmark screen.

## Review Unit 4: Push Dry-Run

Description:

```text
Add bookmark push dry-run

Freeze one local bookmark and one named remote, run jj's push dry-run first,
and display its exact output without permitting a remote write.
```

Implementation:

1. Enable `P` only on a present, normal, non-conflicted local bookmark row.
1. Require an explicit remote choice and show both the bookmark and remote before execution.
1. Construct exactly:

   ```text
   jj git push --remote REMOTE --bookmark BOOKMARK --dry-run
   ```

1. Use `ExecutionMode::DryRunThenConfirm` only as state vocabulary; at this review unit there is no
   real-push transition. The UI should say "Dry-run only" and must have no key that runs a real push.
1. Store the frozen remote and bookmark with the dry-run result. Do not re-read the current selection
   later and assume it is the same intent.
1. Show command, remote, bookmark, exit status, stdout, stderr, and a concise explanation that no
   remote refs were changed.
1. A failed or canceled dry-run cannot advance to any confirm state.
1. Reject non-dry-run `git push` in command mode. Route valid dry-run requests to this workflow or
   require the same explicit remote/bookmark and preview boundary.

Acceptance tests:

- the resolver always includes exactly one remote, one bookmark, and one `--dry-run`;
- no first-slice path can add broad selection or safety-bypass flags;
- remote-only, deleted, and conflicted bookmarks cannot start push dry-run;
- cancel runs nothing and records nothing;
- success/failure output remains visible and history classifies the command as git push dry-run;
- command-mode flag order cannot bypass the non-dry-run rejection;
- `--no-integrate-operation` is absent even when global run options use it elsewhere;
- a local bare-remote integration test records the ref before and after dry-run and proves it is
  unchanged;
- application tests use a fake runner and assert that no non-dry-run push spec is submitted.

## Real Push Gate

Do not implement a real push in the first four review units. Create a fifth change only when all of
these statements are demonstrated by tests and visible evidence:

- The successful dry-run result owns a frozen remote name, bookmark name, bookmark target ID, and
  repository identity.
- The real-push spec is derived from the successful dry-run spec by removing only `--dry-run`.
- A unit test compares every other argv element and rejects any scope widening.
- The current bookmark target and current repository operation are re-read immediately before
  confirmation. Any difference invalidates the preview and requires another dry-run.
- Confirmation is unavailable after dry-run failure, cancellation, refresh failure, or selection
  change.
- The confirmation surface says the exact remote and bookmark that will change and identifies the
  command as a network write.
- The real command uses `SafetyClass::NetworkWrite`, an explicit remote, and exactly one bookmark.
- Success and failure output remain visible and are recorded in history independently from dry-run.
- A local bare-remote integration test proves the expected single ref moves and no other ref moves.
- Tests prove command mode cannot bypass this gate with reordered flags or a non-dry-run push.
- The Betamax journey shows dry-run, the distinct second confirmation, and the final result using
  only a temporary local bare remote.

If any condition is missing, leave real push disabled and file the missing condition as follow-up
work. In particular, do not treat a successful dry-run alone as authorization for a later push.

Suggested description only after the gate passes:

```text
Confirm guarded bookmark push

Permit one explicit bookmark push only from an unchanged successful dry-run,
with a second network-write confirmation and complete output history.
```

## Fixture Integration

Add a reusable test fixture under `crates/jk-cli/tests/support/` and behavior tests under
`crates/jk-cli/tests/refs_remotes.rs`.

The fixture should create, under one unique temporary root:

- a local bare Git repository used only as transport storage;
- a seed jj repository that creates and publishes deterministic commits/bookmarks to the bare
  remote;
- a client jj repository used by the code under test;
- optionally a second producer checkout that advances a remote bookmark for fetch tests.

Use jj for repository state, bookmarks, commits, fetch, push, and inspection. Using `git init
--bare` is acceptable only to create the fixture's bare transport store because jj does not provide
an equivalent bare-server initialization. Disable signing in every fixture and set deterministic
fixture identity. Include the process ID plus an atomic counter in temporary directory names and
remove only the exact fixture root on drop.

Required fixture assertions:

- initial bookmark list contains local, tracked, untracked, and remote-only examples;
- a producer advance is absent before client fetch and present afterward;
- push dry-run leaves the bare remote ref at the exact prior object ID;
- a deliberately missing remote produces a stable non-zero failure without network access;
- optional real-push tests move exactly one expected ref in the local bare remote.

Never inherit `origin` from this repository. Every integration test must pass `-R` or set the child
working directory to its fixture repository and must assert its configured remote URL is inside the
fixture root before any fetch or push command runs.

## Test Matrix

### `jk-core`

- command-family classification for bookmark, fetch, and push;
- source view/action round trips in command-history records;
- network-read and network-write preview warnings;
- `DryRunThenConfirm` confirmation semantics;
- redacted process previews preserve bookmark and remote argv boundaries.

### `jk-cli`

- every command resolver/spec builder and repository/global-option order;
- JSON-lines bookmark parser success, conflict, deletion, and line-number errors;
- remote-list parser and empty/malformed output;
- structured errors for spawn failure versus non-zero exit;
- local fixture list/fetch/dry-run behavior;
- dry-run remote ref immutability;
- optional real-push single-ref scope.

### `jk-tui`

- bookmark row labels and action availability;
- selection and scroll boundaries at empty, one-row, and many-row sizes;
- refresh selection preservation;
- conflicted/multi-target layout at compact and wide widths;
- remote chooser empty/one/many states;
- no wrapping or truncation of key, bookmark, remote, status, and short target columns;
- push dry-run result clearly distinguishes success, failure, and dry-run-only state.

### `jk`

- `B`, `F`, and `P` key mapping and contextual help;
- root and pushed-view routing;
- target-show pushes and back navigation;
- prompt input, cancellation, preview, confirmation, failure, and refresh state;
- fake-runner capture of exact specs and call counts;
- command-history source/action/result records;
- command-mode resolver policy and reordered-flag bypass cases;
- no real push invocation before the optional gate change.

## Documentation and Terminal Evidence

When the first four units are complete:

1. Update `README.md`, `crates/jk/README.md`, and `docs/usage.md` together. Replace only the planned
   bookmark/fetch/push-dry-run wording that is now implemented; keep real push described as
   unavailable unless its gated review unit lands.
1. Update the relevant checkboxes or milestone wording in `docs/roadmap.md` and
   `docs/plans/0017-dogfood-backlog-order.md` without rewriting historical plan context.
1. Add `tapes/refs-remotes.tape` plus a `just betamax-refs-remotes` recipe.
1. Build and create all fixture repositories while Betamax output is hidden. The visible sequence
   should start with `jk` already open or one clean `jk -R FIXTURE` command.
1. Capture new PNG names for each reviewed state so Codex and browser caches cannot show stale
   images:

   ```text
   target/dogfood-artifacts/betamax/refs-remotes-bookmarks.png
   target/dogfood-artifacts/betamax/refs-remotes-delete-preview.png
   target/dogfood-artifacts/betamax/refs-remotes-fetch-output.png
   target/dogfood-artifacts/betamax/refs-remotes-push-dry-run.png
   ```

1. The visible tape should demonstrate bookmark status/target, target inspection, one canceled local
   mutation preview, successful local-remote fetch output, failed fetch output, push dry-run output,
   and Command History. Do not perform a real push unless the fifth review unit is included, and
   then only against the tape's local bare remote.
1. Open the generated PNGs in Firefox for final visual inspection as required by the repository
   guidelines.

The public website and `jk-screenshots` repositories are outside this workspace. Record a follow-up
website/media pass in the handoff instead of editing those repositories from this task.

## Validation

Run focused checks after each review unit, then the broad gate after the complete stack:

```text
cargo test -p jk-core command
cargo test -p jk-cli bookmark
cargo test -p jk-cli --test refs_remotes
cargo test -p jk-tui bookmark
cargo test -p jk refs_remotes
just fmt-check
just check
just test
just clippy
just lint-md
just betamax-refs-remotes
just release-check
```

Use the actual focused test filters created by the implementation if these conceptual filters do
not match exact names. Do not run the Betamax tape or integration tests until their fixture remote
guard has verified that every remote path is inside the temporary fixture root.

For each change, also inspect its standalone diff and run its focused tests before creating the next
change. Every review unit should build and pass its relevant tests without depending on descendants.

## jj Stack Shape

Starting from this plan change, Luna should create this stack sequentially:

```text
Add bookmark inspection
Preview bookmark mutations
Add explicit fetch workflow
Add bookmark push dry-run
Confirm guarded bookmark push    # optional; only after the real-push gate passes
```

Use `jj new` for each unit and set its description before editing. Do not create bookmarks or push
the stack from this workspace. If a follow-up is inseparable from the change below it, squash only
after the descendant's focused tests pass; otherwise preserve the review boundary.

## Dependency and Merge Order

Merge in the same order as the stack:

1. bookmark inspection establishes the parsed ref model and selectable screen;
1. bookmark mutations consume the selected local ref and reusable confirmation boundary;
1. fetch adds remote selection, network-read confirmation, output retention, and refreshed refs;
1. push dry-run consumes both bookmark and remote selection plus persistent command output;
1. optional real push consumes the frozen dry-run state and must remain last.

Relative to other active `jk` workspaces, this stack depends only on current `main`. It must not
depend on `work/shared-selectors`, `work/run-options`, or other unlanded workspace changes. After
those stacks land, resolve overlap by rebasing this stack and adopting their public landed APIs in a
separate mechanical change only when that reduces duplicated concepts without mixing behavior into
the review units above.

## Non-Goals

- tag list or mutation;
- remote add/remove/rename/set-url management;
- bookmark advance, set, forget, rename, track, or untrack;
- multiple marked bookmarks or multiple remotes in one fetch/push;
- broad `--all`, `--tracked`, or `--deleted` fetch/push modes;
- generated push bookmarks through `--change` or `--named`;
- safety overrides such as allow-private, allow-empty-description, or allow-conflicts;
- conflicted bookmark resolution;
- persistent command history;
- a general selector framework or full arbitrary-command safety classifier;
- real network access in tests, tapes, or development proof;
- publication, bookmarks for this implementation stack, or a pull request.

## Completion Handoff

Luna's final handoff should include:

- exact workspace path;
- `jj status` and `jj log -r 'main..@'` showing the review-unit stack;
- focused tests per change and the final repository-level validation results;
- fixture guard evidence proving all fetch/push endpoints were local paths;
- PNG paths and the Betamax command used;
- whether real push was included or deferred, with every gate result listed;
- residual risks, especially jj-version-sensitive template/flag behavior and command-mode scope;
- dependency and merge order relative to any active workspace that landed while the work proceeded.
