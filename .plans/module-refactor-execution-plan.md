# Module Refactor Execution Plan (2026-02-07)

## Purpose

Refactor `jk` into small, command-aligned modules that are easy to navigate, test, and maintain.
This plan is the durable implementation contract for the current refactor.

## Scope and constraints

- Preserve existing behavior and keybindings while restructuring.
- Use command-first module organization with thin shared runtime layers.
- Keep modules under soft 300 LOC / hard 500 LOC (tests excluded).
- Keep tests and snapshots co-located with implementation domains.
- Ship in atomic `jj` commits with clear conventional commit messages and bodies.

## Target structure

```text
src/
  app/
    mod.rs
    runtime.rs
    history.rs
    input.rs
    selection.rs
    preview.rs
    view/
      mod.rs
      status_diff.rs
      root_resolve.rs
      file_tag.rs
      workspace_git_top.rs
      operation.rs
      common.rs
    tests.rs
  flow/
    mod.rs
    planner.rs
    prompt_kind.rs
    builders.rs
  commands/
    mod.rs
    spec.rs
    overview.rs
  alias/
    mod.rs
    normalize.rs
    catalog.rs
```

## Phase plan

1. App split (structural, no behavior changes)
   - Move `src/app.rs` to `src/app/mod.rs`.
   - Extract `tests` to `src/app/tests.rs`.
   - Extract runtime/input/history/selection/preview helpers into dedicated files.
   - Split `src/app/view.rs` into command-aligned submodules under `src/app/view/`.
   - Keep `App` public surface stable (`App::new`, `App::run`).

2. Flow split
   - Move `src/flows.rs` to `src/flow/planner.rs` + `prompt_kind.rs` + `builders.rs`.
   - Keep `plan_command`, `FlowAction`, `PromptKind`, `PromptRequest` API stable.

3. Commands split
   - Move registry/spec logic to `src/commands/spec.rs`.
   - Move command-overview rendering helpers to `src/commands/overview.rs`.
   - Re-export from `src/commands/mod.rs`.

4. Alias split
   - Move alias normalization to `src/alias/normalize.rs`.
   - Move alias catalog rendering to `src/alias/catalog.rs`.
   - Re-export from `src/alias/mod.rs`.

5. Documentation and rule alignment
   - Keep `.plans/implementation-status.md` updated per phase.
   - Update README/AGENTS if module paths or conventions change.

## Validation gates per phase

- `cargo fmt --all`
- `cargo check`
- targeted tests for touched modules
- at checkpoints: `cargo test`, strict clippy, markdown lint

## Commit strategy

- Commit each phase separately.
- Prefer structural-only commits before behavior changes.
- Include commit body with intent, constraints, and affected areas.
- After commit rewrites, print `jj --no-pager log -r @- -n 1` and
  `jj --no-pager show @- --stat`.

## Acceptance criteria

- No source module above hard 500 LOC (tests excluded).
- `app`, `flow`, `commands`, and `alias` are directory modules with clear boundaries.
- Existing user-visible behavior remains unchanged.
- All tests/lints/checks are green at final checkpoint.
