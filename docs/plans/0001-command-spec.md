# Command Spec Foundation

Status: draft  
Owner: current dogfood workspace pass  
Scope: first foundation implementation chunk

## Problem

`jk` currently builds `jj` commands directly inside `jk-cli` loaders. That is enough for the
current log and diff views, but it makes later command preview, command history, mutation safety,
and command-mode work harder to add without scattering command formatting rules.

The first foundation slice should introduce a typed command description that can render an argv for
display and build a process command for read-only loaders. It should not change the current user
interface.

## Goals

- Add a small command model in `jk-core`.
- Keep the first implementation read-only.
- Route current log and diff command construction through the command model where practical.
- Preserve current CLI behavior, titles, color forcing, repository handling, and tests.
- Keep the change small enough to review as a foundation chunk.

## Non-Goals

- Do not add mutation previews.
- Do not add command history storage.
- Do not add command mode.
- Do not add async execution or cancellation.
- Do not add a provider crate or new dependency.

## Proposed API

Add `crates/jk-core/src/command.rs` and re-export the central types from `jk-core`.

Core types:

```rust
pub struct JjCommandSpec {
    argv: Vec<OsString>,
    cwd: Option<PathBuf>,
    repository: Option<PathBuf>,
    stdin: Option<String>,
    title: String,
    execution_mode: ExecutionMode,
    safety: SafetyClass,
    refresh_plan: RefreshPlan,
}

pub enum ExecutionMode {
    RenderReadOnly,
    ConfirmMutation,
    ConfirmExternalTool,
    DryRunThenConfirm,
    CommandMode,
}

pub enum SafetyClass {
    ReadOnly,
    LocalMetadata,
    LocalRewrite,
    DestructiveLocal,
    NetworkRead,
    NetworkWrite,
    ExternalCommand,
}

pub enum RefreshPlan {
    None,
    ReRunSpec,
}
```

Default `render_read_only` values:

- `execution_mode = RenderReadOnly`;
- `safety = ReadOnly`;
- `refresh_plan = ReRunSpec`;
- no `cwd`;
- no `repository`;
- no `stdin`.

`ConfiguredDefault` remains safety-ambiguous because bare `jj` follows `ui.default-command`. The
first implementation records the read-only assumption that already exists today; it does not try to
prove or enforce it.

Required methods:

- `JjCommandSpec::render_read_only(args)`;
- `with_title(title)`;
- `with_cwd(cwd)`;
- `with_repository(repository)`;
- `with_stdin(stdin)`;
- `with_mode(mode)`;
- `with_safety(safety)`;
- `with_refresh_plan(refresh_plan)`;
- `argv()`;
- `cwd()`;
- `repository()`;
- `stdin()`;
- `title()`;
- `preview()`;
- `mode()`;
- `safety()`;
- `refresh_plan()`;

`preview()` should return a shell-readable display string without claiming shell execution. The
first implementation can quote conservatively for whitespace, backticks, and single quotes.
Execution must use `argv` directly.

## First Implementation Chunks

### Chunk 1: command model only

Files:

- `crates/jk-core/src/command.rs`
- `crates/jk-core/src/lib.rs`

Acceptance:

- The command model compiles.
- Unit tests prove display rendering for simple args, whitespace args, and single quotes.
- No other crates change.

### Chunk 2: route read-only jj loaders through specs

Files:

- `crates/jk-cli/src/command.rs`
- `crates/jk-cli/src/lib.rs`
- `crates/jk-cli/src/log.rs`
- `crates/jk-cli/src/diff.rs`

Acceptance:

- A private `jk-cli` adapter converts `JjCommandSpec` into `std::process::Command`.
- The adapter applies `--no-pager`, `--color`, repository metadata, cwd metadata, stdin metadata
  for future use, and the existing color-suppression environment removals.
- `JjLog` builds rendered and semantic commands from `JjCommandSpec`.
- `JjDiff` builds diff and stat commands from `JjCommandSpec`.
- Existing behavior and command tests still pass.
- Titles come from the spec instead of separate local title builders where possible.

### Chunk 3: expose preview-shaped metadata to callers

Files:

- `crates/jk-cli/src/log.rs`
- `crates/jk-cli/src/diff.rs`
- `crates/jk/src/main.rs`

Acceptance:

- Current log and diff loaders can report the spec title used for the active view.
- Direct error-state diff titles use the same display string as the spec path.
- No visible UI behavior changes beyond any corrected title consistency.

## Tests

Add or keep focused tests for:

- command display with plain arguments;
- command display with whitespace;
- command display with backticks and single quotes;
- `render_read_only(["diff", "-r", "abc123"])` sets `RenderReadOnly`, `ReadOnly`, and `ReRunSpec`;
- `with_repository`, `with_cwd`, and `with_title` preserve argv;
- diff command argv still includes `--color always`, `diff -r REV`, and `--stat`;
- log process command still includes `--no-pager`, `--color always`, env removals, repository flag
  ordering, and `-T LOG_TEMPLATE` only for the semantic pass;
- log command title remains `jj` for configured default and `jj log` for explicit log.

## Risks

- Over-modeling too early would make the foundation chunk harder to review. Keep the first spec
  read-only and add fields only when real call sites need them.
- Display strings must not be treated as shell commands. The spec stores argv first and renders
  display text second.
- Repository path handling should stay in `jk-cli` until command preview needs it in the shared
  model.
