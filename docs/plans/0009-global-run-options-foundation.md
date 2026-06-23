# Global Run Options Foundation

Status: first foundation chunk implemented in the current implementation spike

Owner: implementation spike

Scope: smallest foundation slice after `0008-view-options-overlay.md`

## Decision

Add a reusable `GlobalOptions` model to the command-spec layer before adding evolog, workspace
screens, command mode, command history, or a visible Run Options drawer.

This slice should move today's scattered repository and output plumbing into data that every
`JjCommandSpec` carries. It should also define the future fields needed by the Run Options drawer,
but it should not expose the drawer, mutation previews, operation recovery, command history, or
new `jj` workflows yet.

The first implementation should be boring: current `log`, `diff`, `show`, and `status` behavior
stays the same, command argv ordering becomes explicit and test-backed, and future advanced global
flags have one owned place to land.

## Context

The current code already has the first command-spec layer:

1. `crates/jk-core/src/command.rs` defines `JjCommandSpec` with argv after the `jj` binary, cwd
   metadata, repository metadata, stdin, title, execution mode, safety class, and refresh plan.
1. `crates/jk-cli/src/command.rs` turns a spec into `std::process::Command`.
1. The adapter currently prepends `--no-pager --color <policy>`, then optional `--repository
   <path>`, then `spec.argv()`.
1. `JjLog`, `JjDiff`, `JjShow`, and `JjStatus` each store their own `Option<PathBuf>` repository
   field and call `spec.with_repository(repository)` when constructing a command spec.
1. Command preview and titles currently render only `jj` plus `spec.argv()`. They do not show
   adapter-owned flags such as `--no-pager`, `--color`, or `--repository`.

That split is acceptable for the current read-only app, but it will drift once command preview,
command history, command mode, mutation confirmation, Run Options, workspaces, and operation
time-travel all need to render the same command.

The installed `jj` compatibility oracle for this plan is `jj 0.42.0`. `jj --help` and `jj util
markdown-help --help` both expose these global options:

- `-R`, `--repository`;
- `--ignore-working-copy`;
- `--no-integrate-operation`;
- `--ignore-immutable`;
- `--at-operation`, alias `--at-op`;
- `--debug`;
- `--color`;
- `--quiet`;
- `--no-pager`;
- `--config`;
- `--config-file`.

## Goals

- Introduce `GlobalOptions` in `jk-core` and make `JjCommandSpec` carry it.
- Preserve current user-visible behavior for bare `jk`, `jk log`, `jk diff`, `jk show`, and
  `jk status`.
- Make global flag ordering explicit: `jj` binary, global flags, command family, command args.
- Preserve today's `-R`/`--repository` propagation across all current read-only sources.
- Keep command previews and future command history from inventing separate global-flag formatting.
- Define the future Run Options fields now, so later UI work appends behavior instead of moving
  data between crates.
- Keep generated `jj` help and `jj util markdown-help` as compatibility inputs, not runtime UI
  dependencies.
- Add tests that fail if a command places global options after the command family.

## Non-Goals

- Do not implement the visible Run Options drawer.
- Do not add new CLI flags to `jk` beyond existing `-R`/`--repository`, `-n`, `log -T`, and current
  inspection flags.
- Do not expose `--ignore-working-copy`, `--at-operation`, `--no-integrate-operation`,
  `--ignore-immutable`, `--config`, or `--config-file` in the UI yet.
- Do not add command history, mutation preview, command mode, operation log, operation recovery, or
  workspace screens.
- Do not change View Options behavior or add diff/show/status display toggles.
- Do not parse the full generated `jj` help manifest in this slice.
- Do not switch from spawning `jj` to `jj-lib` or `jj-cli` internals.

## Current Command Model

`JjCommandSpec` is argv-first. The spec stores command-family argv such as:

```text
[]
["log"]
["diff", "-r", "abc123"]
["diff", "--from", "main", "--to", "@"]
["show", "abc123"]
["status", "src"]
```

`build_jj_command` currently transforms a spec into process argv like:

```text
jj --no-pager --color always --repository /repo diff -r abc123
```

That ordering is correct for `jj`: global options must come before the command family. The problem
is ownership, not behavior. `--no-pager`, `--color`, and `--repository` are adapter-owned today,
while `JjCommandSpec::preview()` only knows about command-family argv.

The next slice should make the command model own the whole `jj` command except the binary name and
process-specific execution details.

## Desired `GlobalOptions`

Add a `GlobalOptions` type in `jk-core`, near `JjCommandSpec`:

```rust
pub struct GlobalOptions {
    pub repository: Option<PathBuf>,
    pub working_copy: WorkingCopyPolicy,
    pub operation: OperationLoadPolicy,
    pub operation_integration: OperationIntegrationPolicy,
    pub immutability: ImmutabilityPolicy,
    pub output: OutputPolicy,
    pub debug: bool,
    pub config_overlays: Vec<ConfigOverlay>,
}
```

The exact visibility can be builder-based instead of public fields. The important contract is that
`GlobalOptions` renders global argv in jj syntax and stays independent from command-family argv.

Start with these supporting types:

```rust
pub enum WorkingCopyPolicy {
    SnapshotAndUpdate,
    Ignore,
}

pub enum OperationLoadPolicy {
    Latest,
    AtOperation(String),
}

pub enum OperationIntegrationPolicy {
    Integrate,
    DoNotIntegrate,
}

pub enum ImmutabilityPolicy {
    Enforce,
    Ignore,
}

pub struct OutputPolicy {
    pub color: ColorPolicy,
    pub pager: PagerPolicy,
    pub quiet: bool,
}

pub enum ColorPolicy {
    Always,
    Never,
    Debug,
    Auto,
}

pub enum PagerPolicy {
    Disable,
    Inherit,
}

pub enum ConfigOverlay {
    Inline { name_value: String },
    File(PathBuf),
}
```

Defaults should match current behavior for app-owned read-only commands:

- no repository;
- snapshot/update working copy;
- latest operation;
- integrate operations;
- enforce immutability;
- `--color always`;
- `--no-pager`;
- not quiet;
- not debug;
- no config overlays.

If keeping output policy outside `GlobalOptions` is temporarily easier, the first implementation may
keep `run_jj_spec(spec, color)` as a compatibility shim. The durable destination is for color and
pager policy to live in `spec.global()`, because command previews and command history need the same
data as process execution.

## Ordering Contract

Render argv in this order:

```text
jj <global options> <command family> <command args>
```

Examples:

```text
jj --no-pager --color always log
jj --no-pager --color always -R /repo diff -r @
jj --no-pager --color always --at-operation abc123 status
jj --no-pager --color always --ignore-immutable rebase -s a -d b
```

Use one canonical spelling for generated commands:

- prefer `-R` for short previews only if the existing code already uses short global flags in
  preview strings;
- otherwise prefer long flags such as `--repository` for clearer previews and tests;
- accept user-entered aliases such as `--at-op` later in command mode, but generate
  `--at-operation` from typed specs.

The first implementation should choose one repository spelling and test it. The current adapter
uses `--repository`, so keeping that spelling minimizes churn.

## Repository Propagation

Today `Args.repository` is copied into each source constructor:

```text
Args.repository
  -> JjLog::with_repository
  -> JjDiff::with_repository
  -> JjShow::with_repository
  -> JjStatus::with_repository
  -> JjCommandSpec::with_repository
  -> build_jj_command
```

The foundation slice should preserve that path while replacing `with_repository` internals:

```text
Args.repository
  -> source GlobalOptions or source.with_repository
  -> JjCommandSpec.global.repository
  -> build_jj_command
```

Do not introduce a repository picker, workspace repository switcher, or command-mode `:global`
command yet. The only externally visible requirement is that `jk -R /repo log`, `jk -R /repo diff`,
selected-change diff, `show`, and `status` still pass the same repository path to `jj`.

Command previews may continue hiding `--no-pager` and `--color always` if that is clearer for title
bars, but they should not hide user-selected global context forever. Decide this explicitly:

- **Process argv preview:** includes all global flags for command history and mutation preview.
- **Title label:** may omit plumbing flags and show a concise title such as `jj diff -r @`.

If the slice cannot add both forms cleanly, add `process_argv()` first and leave title behavior
unchanged.

## Later Run Options Fields

The future Run Options drawer should edit `GlobalOptions`, but this slice only defines the model and
test-backed argv rendering.

### Working-Copy Policy

`--ignore-working-copy` belongs to `WorkingCopyPolicy::Ignore`.

Later UI rule:

- expose it as an advanced Run Options toggle;
- label screens loaded with stale state;
- do not let auto-refresh silently change the policy.

### Operation Time Travel

`--at-operation <OPERATION>` belongs to `OperationLoadPolicy::AtOperation`.

Later UI rule:

- operation-log inspection can set this for read-only screens;
- screens loaded at an operation should show the operation ID and that the working copy is ignored;
- mutating at an old operation requires an advanced confirmation path.

Because jj implies `--ignore-working-copy` for `--at-operation`, the model can either render only
`--at-operation` or store the implied policy for UI labels. It should not render duplicate flags
unless a test proves jj output requires it.

### Operation Integration

`--no-integrate-operation` belongs to `OperationIntegrationPolicy::DoNotIntegrate`.

Later UI rule:

- expose it only as an advanced local-operation simulation path;
- never apply it automatically to commands with remote side effects such as `jj git push` or
  Gerrit upload;
- after execution, show the resulting operation ID and routes to inspect, restore, or integrate it.

It is not a dry run. The spec and future drawer copy should say that clearly.

### Immutability

`--ignore-immutable` belongs to `ImmutabilityPolicy::Ignore`.

Later UI rule:

- do not make it a one-key action;
- mutation previews must say that immutable commits may be rewritten;
- keep it in Run Options or an advanced action confirmation, not in ordinary role pickers.

### Config And Output

`--config` and `--config-file` belong to `ConfigOverlay`.

Later UI rule:

- command mode must preserve raw values without over-parsing TOML;
- native rendering must account for command- and workspace-scoped conditional config;
- tests should use config overlays for deterministic fixtures once generated manifests exist.

`--quiet`, `--debug`, `--color`, and `--no-pager` belong to `OutputPolicy` plus `debug`.

Later UI rule:

- app-owned rendered views should keep explicit color and pager policy;
- command output panes may choose a different color policy;
- quiet should not be applied globally without an explicit user choice because hints and warnings
  are useful.

## Generated Help Compatibility

Do not make runtime UI depend on `jj util markdown-help` in this slice. Use generated help as a
compatibility input for tests and future manifests.

Near-term strategy:

1. Keep using installed `jj --help` and `jj util markdown-help --help` as the manual compatibility
   oracle while writing the `GlobalOptions` tests.
1. Add a small fixture or helper only when command specs start validating flags against a manifest.
1. Prefer `jj util markdown-help` for full command and flag snapshots because it emits all
   subcommand help in one stable Markdown stream.
1. Treat generated help drift as scheduled compatibility work, not a reason for ordinary feature PRs
   to fail unless the supported jj version range changes.

Manifest compatibility should eventually check that every generated global flag still exists in the
supported jj help snapshot. This slice only needs unit tests for the flags the typed model renders.

## Betamax Matrix

Betamax should validate flag families as shared behavior, not one tape per command forever. The
foundation slice should plan these tapes even if it only adds unit tests first:

- `validation/global-repository.tape`: start `jk -R <fixture> log`, open selected diff, show, and
  status; assert each view still uses the selected repository.
- `validation/global-working-copy-ignore.tape`: later, show read-only stale-state labeling when the
  Run Options drawer exposes `--ignore-working-copy`.
- `validation/global-at-operation.tape`: later, load log/status at an operation and assert the title
  or status line shows the operation context and working-copy policy.
- `validation/global-no-integrate-local-rebase.tape`: later, run a local mutation preview path and
  assert the resulting operation ID is visible; never use this for push.
- `validation/global-ignore-immutable.tape`: later, assert immutable override requires an advanced
  confirmation and appears before the command family.
- `validation/global-config-overlay.tape`: later, prove a config overlay affects rendered output in
  a deterministic fixture.

The immediate implementation does not need to add all tapes. It should make the argv model testable
enough that these tapes can assert visible behavior later without reworking command construction.

## Tests And Validation

Add focused unit tests before UI or Betamax work:

- `GlobalOptions::default()` renders the current app-owned globals: `--no-pager --color always`.
- repository renders before the command family;
- `--ignore-working-copy` renders before the command family;
- `--at-operation` renders before the command family and uses canonical long spelling;
- `--no-integrate-operation` renders before the command family;
- `--ignore-immutable` renders before the command family;
- repeated `--config` and `--config-file` preserve order within `GlobalOptions`;
- `JjCommandSpec::preview()` or the new full-preview method quotes global and command args with the
  same rules as command-family args;
- existing `JjLog`, `JjDiff`, `JjShow`, and `JjStatus` spec tests still pass;
- adapter tests assert the exact argv order for at least one repository command.

Suggested validation for the implementation chunk:

```sh
cargo test -p jk-core command
cargo test -p jk-cli command
cargo test -p jk-cli log
cargo test -p jk-cli diff
cargo test -p jk-cli show
cargo test -p jk-cli status
markdownlint-cli2 docs/plans/0009-global-run-options-foundation.md
```

If public signatures in `jk-core` or `jk-cli` change more broadly than expected, also run:

```sh
cargo test -p jk-core
cargo test -p jk-cli
cargo test -p jk
```

## Dependency Ordering

This slice should land before these features:

- standalone `v` evolog, because evolog will need command specs that preserve global context and
  can later render `--at-operation` correctly from operation screens;
- `W` workspaces, because workspace-specific status/diff/log actions must not invent repository or
  output handling separately;
- command mode and command history, because history needs process argv and display preview from the
  same data model;
- mutation preview and rebase workflows, because safety flags and global options must appear before
  the command family in previews;
- the visible Run Options drawer, because the drawer should edit `GlobalOptions` instead of owning a
  parallel settings struct.

The one exception is small View Options follow-up work. Display-format rows can continue under `V`
as long as they reuse existing command specs and do not introduce advanced global flags.

## Implementation Chunks

### Chunk 1: `GlobalOptions` Model

Files:

- `crates/jk-core/src/command.rs`
- `crates/jk-core/src/lib.rs`

Acceptance:

- `JjCommandSpec` stores `global: GlobalOptions` instead of a direct repository field;
- existing `with_repository` remains as compatibility sugar and sets `global.repository`;
- tests cover global argv rendering and ordering;
- no source outside `jk-core` needs to know how globals become argv.

### Chunk 2: Adapter Uses Spec Globals

Files:

- `crates/jk-cli/src/command.rs`
- `crates/jk-cli/src/log.rs`
- `crates/jk-cli/src/diff.rs`
- `crates/jk-cli/src/show.rs`
- `crates/jk-cli/src/status.rs`

Acceptance:

- `build_jj_command` asks the spec for global argv instead of hard-coding repository separately;
- the color parameter is either moved into `GlobalOptions` or preserved as a temporary shim with a
  clear test;
- current command output and title behavior stay unchanged;
- repository propagation tests pass for log, diff, show, and status.

### Chunk 3: Preview Split

Files:

- `crates/jk-core/src/command.rs`
- affected tests in `jk-cli`

Acceptance:

- add a full process preview or argv accessor that includes global options before command argv;
- keep concise titles stable for existing views;
- document which method future command history and mutation preview should use.

Stop there. The next PR can add the visible Run Options drawer after command history or mutation
preview has a real screen to consume it.

## Risks

- Moving repository handling can accidentally drop `-R` from pushed views. Keep one test per current
  source family.
- Showing all app-owned plumbing flags in title bars would make current UI noisier. Keep title and
  full preview separate.
- Adding every global flag as UI state now would overbuild the slice. Render typed defaults and
  advanced fields in tests, but do not expose controls.
- Treating `--no-integrate-operation` as a dry run would be wrong and dangerous for remote commands.
  Keep that warning in the type docs and future drawer copy.
- Generated help fixtures can become busywork if they land before there is a parser or manifest
  consumer. Defer the fixture until a command-spec compatibility check uses it.

## Acceptance Criteria

- `docs/plans/0009-global-run-options-foundation.md` records the narrow next slice.
- `GlobalOptions` has an agreed field set and default policy.
- The plan states that global flags render before command-family argv.
- The plan preserves current `-R` propagation and names the existing propagation path.
- The plan explains how `--ignore-working-copy`, `--at-operation`, `--no-integrate-operation`,
  `--ignore-immutable`, config overlays, and output policy fit later.
- The plan defines generated-help compatibility as a test input, not a runtime dependency.
- The plan defines a Betamax matrix by global flag family.
- The plan puts this work before evolog, workspaces, command history, mutation preview, and the
  visible Run Options drawer.
