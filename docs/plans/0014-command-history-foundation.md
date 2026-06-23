# Command History Foundation

Status: draft/spec

Owner: `vibe` workspace spike

Scope: first command-history implementation slice after Workspaces

## Decision

Add a durable command-history model, recorder, and first read-only history surface before adding
safe mutation preview, op-log recovery, or broad command-mode execution.

This slice should record commands that `jk` already runs through typed `JjCommandSpec` paths. It
should start with current read-only inspection sources plus the selected-workspace
`workspace update-stale` local-metadata mutation. It should not implement a general mutation
runner, command preview, `:` command mode, `!` external command mode, rerun, command editing, or
persistent cross-session storage yet.

The first reviewable implementation should:

- define a `CommandRecord` data model in the command-history layer;
- record command specs before execution and complete records after execution;
- retain enough stdout/stderr summary to debug failures without storing unlimited output;
- expose a focused Command History screen or overlay from the app;
- keep every record local to the running process unless local artifact capture is explicitly
  requested by a test or Betamax path;
- add tests and Betamax evidence proving the history surface and first recorded command families.

## Context

The roadmap places command history in the 0.5 Command Mode and Workspaces milestone, while safe
mutation core follows in 0.6. Existing specs have already put the lower-level prerequisites in
place:

1. `0001-command-spec.md` introduced `JjCommandSpec`, execution mode, safety class, titles, and
   refresh plans.
1. `0009-global-run-options-foundation.md` moved global options into the command spec so previews,
   execution, and history do not invent different argv formatting rules.
1. `0012-workspace-scope.md` added the `W` screen as a focused workspace view with status and diff
   actions.
1. `0013-workspace-update-stale.md` defined the first local-metadata mutation that is visible but
   intentionally lighter than future history rewrites.

That leaves an important gap. `jk` can now run several command-shaped actions, including one local
metadata mutation, but there is no durable record of what ran, which view or action triggered it,
what output came back, how long it took, or which operation it produced. Safe mutation preview and
operation recovery both need that record.

## Why This Comes Now

Command history belongs immediately after Workspaces for three reasons.

First, Workspaces are the first feature family where the active source view is no longer just "the
current log repository." Status and diff may run against a selected workspace root, while
`workspace update-stale` mutates local workspace metadata for that selected root. History needs to
record the resolved source view, selected action, cwd, repository/global options, and command title
before more cross-workspace commands appear.

Second, `workspace update-stale` is the first sanctioned local mutation that intentionally predates
the full Command Preview loop. Recording it makes that exception honest: users and tests can see
that `jk` ran `jj -R WORKSPACE_ROOT workspace update-stale`, how it exited, and what output it
returned.

Third, safe mutation preview and op-log recovery need a place to attach their results. If command
history waits until after rebase, abandon, describe, undo, or restore, those features will each
invent local status messages and transcript handling. A small recorder now lets later mutation
slices add preview and operation recovery without reshaping every command path again.

The ordering should therefore be:

1. read-only command specs and global options;
1. read-only inspection screens and Workspaces;
1. selected-workspace `update-stale` as the first local metadata mutation;
1. command-history foundation;
1. Command Preview, Run Options, and operation recovery for mutating workflows;
1. `:` and `!` command modes once the same recorder can capture user-entered commands.

## Goals

- Add a first command-history model that can describe every app-run command.
- Record argv and a display preview from `JjCommandSpec`, not from hand-built shell strings.
- Record command family, title, source view, triggering action, cwd, repository/global options,
  start/end time or duration, exit status, output summaries, safety class, execution mode, refresh
  plan, and operation id when available.
- Keep the first slice local-first and privacy-aware.
- Bound output retention by default and avoid writing transcripts to persistent state unless a
  test, debug flag, or Betamax artifact path asks for it.
- Add a focused history entry point that lets users inspect recent commands without changing
  existing command behavior.
- Start recording current read-only sources and `workspace update-stale`.
- Preserve current user-facing behavior except for the new visible history entry point and small
  status/title improvements needed to make records understandable.
- Leave the model ready for Command Preview, Run Options, command mode, external tools, rerun, and
  operation recovery.

## Non-Goals

- Do not add `:` jj command mode.
- Do not add `!` external command mode.
- Do not add command rerun, command editing, or copy-to-clipboard behavior in this slice.
- Do not add full persistent history under `$XDG_STATE_HOME`.
- Do not write full stdout/stderr for normal app runs by default.
- Do not add a reusable mutation confirmation or Command Preview screen.
- Do not add a visible Run Options drawer.
- Do not add op-log, undo, redo, restore, revert, or operation recovery actions.
- Do not change the semantics of read-only log, diff, show, status, evolog, workspace status, or
  workspace diff commands.
- Do not make `workspace update-stale` require a new confirmation just because it is recorded.
- Do not record secrets from config overlays, environment variables, stdin, or command output
  without redaction.
- Do not regenerate website media or public README media in this slice.

## Data Model

Add a command-history model close to the command-spec layer if it needs shared types, or in the app
crate if no other crate needs it yet. The model should be independent from rendering so tests can
assert records without terminal snapshots.

Suggested shape:

```rust
pub struct CommandRecord {
    pub id: CommandRecordId,
    pub command: CommandIdentity,
    pub source: CommandSource,
    pub context: CommandExecutionContext,
    pub timing: CommandTiming,
    pub result: CommandResultSummary,
    pub retention: OutputRetention,
    pub refresh: RefreshPlan,
    pub safety: SafetyClass,
    pub execution_mode: ExecutionMode,
    pub operation_id: Option<String>,
}

pub struct CommandIdentity {
    pub argv: Vec<OsString>,
    pub spec_preview: String,
    pub command_family: CommandFamily,
    pub title: String,
}

pub struct CommandSource {
    pub view: SourceView,
    pub action: SourceAction,
    pub key: Option<String>,
}

pub struct CommandExecutionContext {
    pub cwd: Option<PathBuf>,
    pub repository: Option<PathBuf>,
    pub global_options: GlobalOptionsSnapshot,
}

pub struct CommandTiming {
    pub started_at: SystemTime,
    pub ended_at: Option<SystemTime>,
    pub duration: Option<Duration>,
}

pub struct CommandResultSummary {
    pub exit_status: Option<ExitStatusSummary>,
    pub stdout: StreamSummary,
    pub stderr: StreamSummary,
}
```

The exact type and visibility names can change during implementation. The durable contract is that
one completed record can answer:

- what command `jk` intended to run;
- what context produced it;
- where it ran;
- whether it was read-only, local metadata, local rewrite, network, or external;
- whether it succeeded;
- what compact output was available;
- what should refresh afterward;
- what operation id resulted, if `jj` made one discoverable.

### Command Identity

Record two command forms:

- `argv`: the exact process argv after global options are applied, excluding environment variables;
- `spec_preview`: the display string from the same spec, suitable for titles and copy later.

The process argv should keep global options before the command family:

```text
jj --no-pager --color always --repository WORKSPACE_ROOT workspace update-stale
```

The title may stay shorter:

```text
jj -R WORKSPACE_ROOT workspace update-stale
```

Do not reconstruct either form from rendered title text.

### Command Family

Use a command-family enum or tag set that can represent current and near-term commands:

- configured default / bare `jj`;
- `jj log`;
- `jj diff`;
- `jj show`;
- `jj status`;
- `jj evolog`;
- `jj workspace`;
- future `jj op`;
- future `:` user-entered jj command;
- future `!` external command.

For nested families such as `jj workspace update-stale`, store both the broad family and enough
argv/title detail for filtering. A simple first slice can use one `JjWorkspace` family plus title
matching; a later slice can add subfamily tags if discovery needs them.

### Source View And Action

Record the source view and action that caused the command. This is distinct from command family
because the same `jj status` command can come from root status, a selected workspace row, command
mode, or a future operation-time-travel view.

Initial source views:

- Log;
- Diff;
- Show;
- Status;
- Evolog;
- Workspaces;
- WorkspaceStatus;
- WorkspaceDiff;
- CommandHistory.

Initial source actions:

- InitialLoad;
- Refresh;
- OpenDiff;
- OpenShow;
- OpenStatus;
- OpenEvolog;
- WorkspaceList;
- WorkspaceStatus;
- WorkspaceDiff;
- WorkspaceUpdateStale.

Use an `Unknown` or `Other(String)` escape hatch only for tests and future compatibility. Current
app-owned paths should use typed variants.

### Context

Record these context fields:

- `cwd`: process working directory if the runner sets one or relies on the current process cwd;
- `repository`: resolved `-R` / `--repository` path when present;
- `global_options`: a snapshot of repository, working-copy policy, operation policy, operation
  integration, immutability, output policy, debug, and config overlays;
- optional workspace name or view target label when the source action resolves a workspace row.

Paths may be absolute internally. UI rendering should use compact display paths where possible and
redact home-directory or fixture prefixes only when a test explicitly needs stable output. Do not
store environment variables in the command record.

### Timing And Exit Status

Each record should be created before command execution starts and completed when the command
finishes. Store:

- `started_at`;
- `ended_at` or `duration`;
- process exit code or signal summary when available;
- spawn/read errors as result failures even when no process exit status exists.

The first UI can show duration instead of raw start/end timestamps if that is easier to scan. Tests
should avoid asserting wall-clock timestamps directly.

### Output Summaries

Store bounded summaries for stdout and stderr:

```rust
pub struct StreamSummary {
    pub byte_len: usize,
    pub line_count: usize,
    pub snippet: String,
    pub truncated: bool,
    pub redacted: bool,
}
```

Default retention:

- keep a compact in-memory snippet for stdout and stderr;
- prefer stderr for failure summaries;
- preserve enough stdout for successful commands that only report useful state on stdout;
- mark snippets as truncated when output exceeds the configured limit;
- store the byte count and line count even when the snippet is empty.

Suggested first limits:

- 8 KiB per stream in memory;
- 40 display lines per stream for the output view;
- no persistent full-output file unless test/debug artifact capture asks for one.

These numbers can change in implementation. The important contract is bounded by default,
observable in tests, and not silently unbounded.

### Operation Id

Record an operation id when it is cheaply and reliably available. The first slice should not run
extra `jj op log` commands after every command just to discover one.

Allowed first-slice sources:

- parse a stable operation id from successful command output when `jj` prints one;
- accept an operation id returned by a future runner API;
- leave `operation_id = None` for read-only commands and for mutations that do not expose an id.

Do not block this slice on operation-id extraction. The record must have the field so future
Command Preview and op-log recovery can fill it.

## Persistence And Privacy

The first slice should be local-only and conservative.

Persist by default:

- recent command records in process memory;
- bounded stdout/stderr snippets in process memory;
- Betamax/test artifacts under `target/vibe-artifacts` when validation asks for them.

Do not persist by default:

- full stdout/stderr transcripts;
- command stdin bodies;
- environment variables;
- raw config overlay values that look secret-bearing;
- shell history;
- cross-session history under `$XDG_STATE_HOME`;
- anything outside the repo's local ignored artifact directories.

Future cross-session history can use the product-plan location:

```text
$XDG_STATE_HOME/jk/command-log
```

That is deliberately out of scope here. This spike should make the in-memory model and UI stable
before choosing file format, migration, retention limits, or user config keys.

### Redaction

Add a small redaction pass before records are displayed or persisted to test artifacts.

Redact obvious secret-bearing data in:

- argv values after flags such as `--config`, `--config-file` only when the value itself appears to
  contain a token, password, credential, or authorization header;
- inline config overlays with keys containing `token`, `secret`, `password`, `credential`, `auth`,
  or `key`;
- stdin summaries;
- stdout/stderr lines containing obvious `KEY=VALUE` credential patterns.

Do not over-redact normal jj paths, revsets, operation ids, commit ids, workspace names, or
template strings. Redaction should protect accidental leaks without making command history useless.

Use a visible marker:

```text
<redacted>
```

Do not store the unredacted value inside `CommandRecord` after redaction unless the value is already
held by a separate live command spec needed for execution. The record is the audit surface, so it
should be safe to render.

## Output Retention Policy

Define retention as part of the record rather than a hidden runner behavior:

```rust
pub enum OutputRetention {
    SummaryOnly {
        stdout_limit: usize,
        stderr_limit: usize,
    },
    Artifact {
        stdout_limit: usize,
        stderr_limit: usize,
        full_output_path: PathBuf,
    },
}
```

First-slice app runs should use `SummaryOnly`. Betamax and explicit debug paths may use `Artifact`
under:

```text
target/vibe-artifacts/command-history/
```

Artifact paths must stay local, ignored, and reproducible. Do not add committed media or committed
transcripts for this slice.

## UI Entry Point

Add one focused entry point for recent command history. Two UI shapes are acceptable; choose the one
that fits the current app stack with the least churn.

Preferred shape: a Command History screen.

- Key: `C` or another available key chosen during implementation after checking current bindings.
- View title: `Command History`.
- Rows: newest first or oldest first, but choose one and test it.
- Row fields: status marker, duration, source action, command title, compact repository/workspace.
- `Enter`: open command output/details as a read-only child view if details can reuse
  `RenderedView`.
- `Backspace`/`Esc`: return to the previous view.
- `?`: searchable help rows include the history actions.

Fallback shape: a Command History overlay.

- Opens above the active view.
- Shows recent rows and a selected-row details pane only if the current overlay helpers support it.
- Closes with `Esc`, `Backspace`, or `?`.
- Does not need to push a stack entry.

The screen is preferable because history is an object view like Workspaces, op log, bookmarks, and
future refs. The overlay is acceptable if a first screen would require broad app-state movement.

### Relationship To Existing Surfaces

`:` command mode:

- do not implement `:` in this slice;
- record metadata should already be able to represent future user-entered jj commands;
- command mode should later append records through the same recorder and reuse the same output
  details view.

Run Options:

- do not implement the Run Options drawer;
- store a `GlobalOptionsSnapshot` so future Run Options changes are visible in history;
- make advanced safety flags render in the record before the command family when they exist.

Command Preview:

- do not add a preview loop;
- build records from `JjCommandSpec` so preview can later create a pending record before execution
  or attach its preview text to the completed record.

Operation recovery:

- include `operation_id`;
- leave recovery actions disabled or absent while the id is `None`;
- future op-log recovery can add `o open operation` from history without changing record shape.

## Recording Scope

Start recording app-owned command paths that already have typed specs.

Read-only commands:

- initial log load and explicit log refresh;
- selected-change diff;
- selected-change show;
- root status;
- selected-change evolog;
- Workspaces list refresh;
- selected-workspace status;
- selected-workspace diff.

Local metadata mutation:

- selected-workspace `workspace update-stale`.

If an implementation path still constructs a command without a `JjCommandSpec`, either route it
through the spec first or explicitly leave it unrecorded with a TODO in the tests. Do not add
string-parsing history shims just to make coverage look complete.

### Behavior Preservation

Recording should not materially change existing behavior.

- Successful read-only loads should still open the same screens.
- Failed read-only loads should still show the same recoverable error views or status messages.
- `workspace update-stale` should still run directly as defined in `0013`, refresh on success, and
  keep the old list visible on failure.
- The only new visible behavior should be the history entry point and any concise "recorded
  command failed" details that replace previously lossy output summaries.

Avoid adding modal prompts, confirmation, or command preview as part of this slice.

## Recorder Boundary

Add a small recorder abstraction so command-running code does not need to know UI storage details.

Suggested shape:

```rust
pub struct CommandHistory {
    records: VecDeque<CommandRecord>,
    limit: usize,
}

pub struct PendingCommandRecord {
    id: CommandRecordId,
}

impl CommandHistory {
    pub fn start(&mut self, input: CommandRecordStart) -> PendingCommandRecord;
    pub fn finish(&mut self, pending: PendingCommandRecord, output: CommandRecordFinish);
    pub fn records(&self) -> impl Iterator<Item = &CommandRecord>;
}
```

The app may own `CommandHistory` inside `AppState`. The runner should receive enough mutable access
or callbacks to append records without making `jk-cli` depend on the TUI.

Keep the first seam simple. A callback like `record_command(spec, source, result)` is acceptable if
the existing synchronous runner makes pending records awkward. The durable requirement is that
timing, failures, and output summaries are captured at the runner boundary, not reconstructed from
rendered views afterward.

## History Details View

If the first slice adds a details child view, it should be read-only and text-based. A simple body
is enough:

```text
Command
  jj -R /repo status

Source
  Workspaces -> status

Result
  exit 0, 34 ms

Stdout
  ...

Stderr
  ...
```

Do not add copy or rerun actions yet. Show disabled or absent actions rather than advertising
behavior that does not exist.

## Tests

Add unit tests before relying on Betamax evidence.

### Model Tests

- starting and finishing a record stores one completed record;
- record ids are stable and monotonic within one app run;
- argv and preview come from the same command spec;
- global options render before command-family argv;
- source view/action are stored independently from command family;
- duration is present after completion;
- failed spawn records an error without an exit code;
- stdout and stderr summaries record byte count, line count, truncation, and snippets;
- secret-looking values are redacted in argv/config overlays/output summaries;
- the in-memory history limit drops oldest records first.

### Runner Tests

- read-only log/diff/show/status/evolog paths append records on success;
- command failures append records before returning existing error behavior;
- selected-workspace status and diff records include the selected workspace repository/root;
- `workspace update-stale` records `SafetyClass::LocalMetadata`;
- `workspace update-stale` failure records stderr and does not record a successful refresh;
- post-success workspace-list refresh creates its own record or is linked as a follow-up refresh,
  whichever the implementation chooses and documents.

### UI State Tests

- the history entry point opens a screen or overlay from log and Workspaces contexts;
- records render with title, status marker, duration, source action, and compact context;
- empty history renders a non-panicking empty state;
- selecting a record and pressing Enter opens details if details are implemented;
- `Backspace` or `Esc` returns to the previous view;
- `?` help and searchable discovery include the history entry point;
- copy, rerun, and operation-open actions are absent or disabled in this slice.

### Fixture Strategy

Reuse the existing fixture families instead of inventing command-history-only repositories:

- basic log/diff/show/status fixture for read-only records;
- selected-change evolog fixture when available;
- `multi-workspace-repo` fixture for Workspaces list, selected status/diff, and update-stale;
- a deterministic failing command seam in tests rather than a flaky real process failure.

The stale-workspace fixture from `0013` should be the main integration fixture for local-metadata
mutation history. Keep fixture setup isolated from the developer's real checkout.

## Betamax Evidence

Add Betamax evidence under `target/vibe-artifacts`. The goal is to prove visible behavior and local
artifact boundaries, not to commit generated output.

Suggested tape:

```text
tapes/validation/command-history-foundation.tape
```

Suggested journey:

1. open `jk` in a deterministic fixture;
1. open selected-change show or diff;
1. open Workspaces;
1. open selected-workspace status or diff;
1. run selected-workspace update-stale when the fixture supports it;
1. open Command History;
1. assert rows include command titles, source actions, success/failure status, and duration;
1. open one details view and assert bounded stdout/stderr summaries are visible.

Suggested local artifacts:

```text
target/vibe-artifacts/command-history/history-screen.txt
target/vibe-artifacts/command-history/history-details.txt
target/vibe-artifacts/command-history/records.json
target/vibe-artifacts/command-history/redaction.txt
```

The `records.json` artifact should contain redacted records only. If full output artifacts are
added for a test, store them under the same local artifact directory and do not reference them from
README, crates.io docs, or the website.

## Acceptance

This plan is ready for implementation when the next slice can be described as:

- `jk` records app-owned command executions through a single command-history model.
- Records include argv/spec preview, command family, title, source view/action, cwd/repository or
  global options, timing/duration, exit status, stdout/stderr summaries, retention policy, safety
  class, execution mode, refresh plan, and optional operation id.
- Recording covers current read-only specs and selected-workspace `workspace update-stale`.
- Output retention is bounded by default and full output is not persisted for normal app runs.
- Obvious secrets are redacted before records are displayed or written to local artifacts.
- A Command History screen or overlay is reachable from normal app contexts.
- History details expose compact command output without adding copy, rerun, or recovery actions.
- Existing command behavior stays materially unchanged.
- Tests cover the model, runner recording, UI entry point, redaction, retention, and workspace
  update-stale behavior.
- Betamax artifacts under `target/vibe-artifacts` prove the user-visible journey.

## Follow-Up Slices

### Command Mode

Add `:` jj command mode and `!` external command mode after the recorder can capture user-entered
argv, output summaries, failures, and source metadata.

### Command Preview

Add a preview loop that consumes `JjCommandSpec`, creates or annotates history records, and routes
confirmed commands through the same runner.

### Run Options

Expose advanced `GlobalOptions` fields through a drawer and make changed options visible in command
history.

### Operation Recovery

Fill `operation_id` reliably for mutating commands, then add history actions to open `jj op show`,
`jj op diff`, restore, integrate, undo, or redo where appropriate.

### Persistent History

Add cross-session storage under `$XDG_STATE_HOME/jk/command-log` with user-configurable retention,
schema migration, redaction guarantees, and export/debug tooling.

### Rerun And Copy

Add `y` copy command, `Y` copy command plus output, and confirmed `r` rerun only after command mode
and preview share the same argv and safety model.

### Broader Mutation Coverage

Record describe, new, commit, edit, rebase, abandon, squash, split, restore, diffedit, absorb,
bookmark, tag, fetch, and push once their command specs and safety flows exist.
