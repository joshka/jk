# Safe Mutation Preview Foundation

Status: draft/spec

Owner: implementation spike

Scope: first mutation-preview implementation slice after Command History

## Decision

Add a reusable command-preview and confirmation boundary before implementing broad mutating
workflows. The first supported mutation family should be inline `jj describe` for one selected
revision:

```text
jj describe -m MESSAGE REV
```

This is the smallest useful local-history rewrite. It exercises the same preview, confirmation,
runner, command-history, refresh, and recovery hooks that later `new`, `commit`, `edit`, `rebase`,
`abandon`, `restore`, `squash`, and `split` flows need, while avoiding rebase role ambiguity and
external editor handoff.

The first slice should not add a general mutation menu or command mode. It should introduce the
shared preview model, route confirmed inline describe through the existing typed runner and command
history, and leave richer mutation families as follow-up work.

## Context

The roadmap orders the durable primitives this way:

1. `JjCommandSpec`, `GlobalOptions`, execution mode, safety class, and refresh plan;
1. inspection screens, View Options, searchable help, marks, and Workspaces;
1. selected-workspace `workspace update-stale` as the first local metadata mutation;
1. command-history recording for app-owned commands;
1. safe mutation previews and recovery.

The current code already has the main data boundary this slice should reuse:

- `JjCommandSpec` stores command-family argv, global options, cwd, stdin, title,
  `ExecutionMode`, `SafetyClass`, and `RefreshPlan`.
- `GlobalOptions` renders jj global argv before the command family, including repository,
  operation, working-copy, integration, immutability, output, debug, and config-overlay fields.
- `RecordingJjCommandRunner` records command-history entries around typed specs.
- `CommandRecord` already retains argv, display preview, source view/action, execution context,
  timing, output summaries, safety class, execution mode, refresh plan, and optional operation id.

`workspace update-stale` is intentionally lighter than this slice. It is a local metadata mutation
with `SafetyClass::LocalMetadata`; it should keep running as already specified unless a future
Run Options policy says all local metadata commands must preview too.

## First Mutation Family

Support inline selected-revision describe first:

```text
selected revision + m -> message prompt -> command preview -> Enter confirms
```

The resulting spec should be equivalent to:

```text
jj describe -m MESSAGE REV
```

Use `SafetyClass::LocalRewrite`, `ExecutionMode::ConfirmMutation`, and a refresh plan that reloads
the source view after success. The source action should identify this as an inline describe
mutation rather than a generic command-mode invocation.

Inline describe is the right first family because:

- it is a P0 roadmap mutation and part of the 0.6 Safe Mutation Core;
- it has one selected revision role and no source/destination ambiguity;
- it is recoverable through jj's operation log;
- it produces a clear before-run command preview;
- it does not require terminal restoration for an editor, diff editor, merge tool, or pager;
- it forces command history to represent a real local rewrite without remote side effects.

The message prompt is part of this slice only as far as needed to produce a valid spec. It does not
need multiline editing, templates, description diffing, or external editor support.

## Command Preview Model

Add a provider-neutral preview model near the command-spec or app-state boundary. Exact type names
can change, but the model should be able to answer:

- which typed `JjCommandSpec` will run;
- which source view, action, key, selected revision, and entered message produced it;
- which safety class and execution mode apply;
- which run/global options are active;
- what will refresh after success;
- what output or error should remain visible after execution;
- whether the command has run, is running, failed, succeeded, or was canceled.

Suggested conceptual shape:

```rust
struct CommandPreview {
    spec: JjCommandSpec,
    source: CommandSource,
    summary: MutationSummary,
    confirmation: ConfirmationPolicy,
    state: PreviewState,
}
```

Render the command from the same spec data that execution and history use. Do not reconstruct the
command from titles, status text, or shell strings.

The preview should show:

- primary command line, including user-relevant global options such as `--repository`;
- selected object summary: change id, commit id if known, current description, and immutable or
  conflict labels if already available in the source model;
- consequence summary: "updates the selected change description";
- run options summary, even if the first slice only shows defaults;
- confirmation hotbar: `Enter` run, `Esc` cancel, `?` help.

The full process argv, including output plumbing such as `--no-pager` and `--color`, may live in a
details row or command-history details. The visible preview must still preserve jj ordering:

```text
jj <global options> describe -m MESSAGE REV
```

## Confirmation Boundary

Opening the preview must not start a command-history record and must not spawn `jj`. A command is
recorded only after the user confirms with `Enter`.

Required behavior:

- `Esc` cancels the preview, returns to the previous view/mode, and leaves history unchanged.
- `Enter` starts a command-history record, runs the typed spec through the recording runner, and
  completes the record with stdout, stderr, exit status, duration, and operation id when available.
- successful execution refreshes according to the spec and shows a concise success status with a
  route to Command History;
- failed execution keeps the output visible or reachable from the preview result and still records
  the failed command;
- `Backspace` follows the existing view-stack rules only when no prompt or preview mode is active.

Strong confirmation is not required for normal inline describe. If the target is known immutable,
the preview may warn, but the foundation slice should not expose `--ignore-immutable`. Let jj fail
and show the recorded stderr until Run Options intentionally supports that override.

## Command History And Runner Use

Confirmed mutations must use the same recorder boundary as current app-owned commands. Do not add a
parallel mutation transcript.

The history record for inline describe should include:

- argv from `JjCommandSpec::process_argv()`, after redaction;
- the display preview/title from the same spec;
- `CommandFamily::Other("describe")` unless a typed describe family is added;
- source view/action for inline describe;
- repository/cwd/global-options snapshot;
- `SafetyClass::LocalRewrite`;
- `ExecutionMode::ConfirmMutation`;
- stdout/stderr summaries and failure details;
- optional operation id, when the runner can discover it without parsing fragile rendered output.

Operation id discovery can be best-effort in this slice. The preview and history model should not
block on a durable op-id parser, but it must leave the field and UI route ready for the operation
recovery slice.

## Global And Run Options

The preview consumes `GlobalOptions` from `JjCommandSpec`; it does not own separate global-flag
formatting.

Foundation behavior:

- preserve repository propagation from `jk -R` and workspace-scoped sources;
- keep global argv before `describe`;
- keep normal working-copy snapshot/update behavior for inline describe;
- show active global/run options in the preview summary;
- include redacted config overlays in history exactly as the command-history layer already does;
- do not silently enable `--no-integrate-operation` as a simulation mode.

Advanced Run Options are a follow-up UI, not a hidden default. In this foundation:

- `--at-operation` contexts should be inspect-only for mutation actions unless a later advanced
  confirmation design explicitly allows concurrent-operation mutation;
- `--ignore-working-copy` should not be exposed as a describe-flow toggle;
- `--ignore-immutable` should not be exposed;
- `--no-integrate-operation` should be reserved for a future explicit "simulate as operation"
  affordance and must never be applied automatically to remote/network commands.

The preview model should make adding a visible Run Options drawer straightforward: the drawer edits
`GlobalOptions`, regenerates the typed spec, and the preview/history/runner paths keep using that
same spec.

## Betamax Evidence

Add Betamax evidence under local artifacts, not committed generated media:

```text
tapes/validation/safe-mutation-preview-foundation.tape
target/jk-artifacts/safe-mutation-preview/preview-cancel.txt
target/jk-artifacts/safe-mutation-preview/preview-confirm.txt
target/jk-artifacts/safe-mutation-preview/history-after-confirm.txt
target/jk-artifacts/safe-mutation-preview/failure-output.txt
```

The tape should use an isolated fixture repository and prove this journey:

1. open `jk` on a deterministic fixture;
1. select a mutable revision;
1. start inline describe and enter a deterministic message;
1. verify the preview shows the command, selected object, consequence, run options, and
   `Enter`/`Esc` controls;
1. cancel once and verify command history has no new mutation record;
1. run the same preview again and confirm it;
1. verify the view refreshes and the updated description is visible;
1. open Command History and verify the describe mutation record includes command, source, success
   status, duration, safety class, and output summary;
1. run a deterministic failing describe path or fake runner case and verify stderr remains visible
   and recorded.

Prefer unit and state tests for most failure cases. Betamax should prove the user-visible journey,
not exhaust every runner branch.

## Non-Goals

- Do not implement rebase, abandon, restore, squash, split, commit, edit, undo, redo, push, or
  operation restore/revert.
- Do not add `:` command mode or `!` external command mode.
- Do not add external-editor describe in this slice.
- Do not add a full Run Options drawer.
- Do not add command editing, copy-to-clipboard, rerun, or persistent history.
- Do not add rebase ghost previews or native graph mutation previews.
- Do not make `workspace update-stale` use the new preview loop unless required by shared plumbing.
- Do not parse full operation ids from fragile human output.
- Do not regenerate README, crates.io, website, or screenshot assets.

## Acceptance Criteria

This implementation slice is ready when:

- inline selected-revision describe resolves to a typed `JjCommandSpec`;
- the mutation preview renders from that spec and shows command, target, consequence, active run
  options, safety class, and confirmation controls;
- canceling the preview runs no process and records no command-history entry;
- confirming the preview runs through the existing `RecordingJjCommandRunner`;
- command history records the confirmed mutation with argv, source, context, result, safety,
  execution mode, refresh plan, bounded output summaries, and optional operation id;
- successful describe refreshes the source view and preserves reasonable navigation context;
- failed describe leaves output visible or reachable and records the failure;
- repository/global options appear in jj order before `describe`;
- advanced unsafe options are absent unless already present in the typed spec from a future source;
- tests cover preview state, cancellation, confirmation, runner recording, refresh behavior,
  global-option ordering, and failure output;
- Betamax artifacts under `target/jk-artifacts` prove the cancel, confirm, refresh, and history
  journey.

## Dependency Order

1. Keep the existing command-spec, global-options, workspaces, and command-history foundations in
   place.
1. Add preview state and rendering tests without wiring a mutation key yet.
1. Add the inline describe resolver and message prompt for one selected revision.
1. Wire confirmation to the existing recording runner and source metadata.
1. Refresh the source view after success and keep failure output reachable.
1. Expose the resulting history record through the existing Command History surface.
1. Add Betamax evidence for cancel, confirm, refresh, and history.
1. Add the visible Run Options drawer before broader mutating families depend on advanced global
   flags.
1. Add operation-log recovery actions once mutation records can reliably carry or discover
   resulting operation ids.
1. Add rebase destination preview only after selector roles, Run Options, preview confirmation, and
   mutation history are all proven by inline describe.
