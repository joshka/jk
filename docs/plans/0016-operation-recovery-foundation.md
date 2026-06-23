# Operation Recovery Foundation

Status: draft/spec

Owner: `vibe` workspace spike

Scope: bounded operation-recovery foundation after the first safe mutation flow

## Decision

Add operation recovery as the next safety layer after the first confirmed safe mutation flow. The
first slice should make operation history visible, connect command-history records to operation
ids, and expose read-only operation inspection before adding recovery mutations.

The first reviewable implementation should:

- add a focused Operation Log screen backed by `jj op log`;
- add read-only Operation Show and Operation Diff views backed by `jj op show` and `jj op diff`;
- let successful mutation results point to command history and operation recovery;
- define confirmation boundaries for `jj undo`, `jj redo`, `jj op restore`, and `jj op revert`;
- keep all command rendering and recording on the existing `JjCommandSpec` and command-history
  path;
- avoid broad command mode, persistent history, full Run Options, or new graph mutation flows.

This is a foundation slice. It should make recovery impossible to miss after mutation, but it should
not try to make every recovery verb one-key executable in the first PR.

## Context

The current plan sequence is:

1. command specs, global options, view stack, keymap data, and inspection screens;
1. workspaces and the selected-workspace `workspace update-stale` local metadata mutation;
1. command history for app-owned commands;
1. inline describe as the first safe mutation preview flow;
1. operation recovery, undo/redo, and op-log entry points.

The existing code already has the important seams this slice should reuse:

- `JjCommandSpec` owns command argv, `GlobalOptions`, execution mode, safety class, and refresh
  plan.
- `GlobalOptions` can render `--at-operation`, `--ignore-working-copy`,
  `--no-integrate-operation`, `--ignore-immutable`, output policy, repository, and config overlays.
- `CommandPreview` warns for local rewrites, destructive local operations, network writes,
  operation time travel, non-integrated operations, ignored working copy, and ignored immutability.
- `CommandRecord` stores source view/action, argv, context, result summaries, safety class,
  execution mode, refresh plan, and optional `operation_id`.
- Command History is currently a focused, read-only screen. It does not yet open output details,
  operation details, rerun, or copy commands.

The foundation should extend these seams rather than introduce operation-specific command strings or
a parallel recovery transcript.

## Goals

- Make operation history a first-class object screen.
- Make successful mutations visibly recoverable from the success footer and Command History.
- Keep `jj op log`, `jj op show`, and `jj op diff` read-only in the first screen flow.
- Define exact confirmation policies before enabling `undo`, `redo`, `restore`, or `revert`.
- Reuse command-history records as the audit trail for recovery commands.
- Preserve jj global-option ordering and operation-time-travel behavior.
- Establish the selector roles that later mutation and content workflows can share.
- Add Betamax evidence for the visible recovery journey without committing generated media.

## First Operation Log Screen

The first Operation Log screen is an object-list screen, like Workspaces and Command History.

Entry points:

- `o` from a post-mutation success footer opens Operation Log.
- `o` from Command History opens the resulting operation when the selected record has an
  `operation_id`; otherwise it opens the Operation Log focused near the current operation.
- `o` from the graph/log screen may open the Operation Log once keymap scope is available.

First-screen scope:

- load recent operations with `jj op log`;
- show operation id, description, age or timestamp, user if present in jj output, and current marker
  when this can be obtained from rendered output without fragile parsing;
- keep the selected row stable across refresh by operation id when possible;
- support `j/k`, arrows, paging, `g/G`, `r` refresh, `?` help, `Esc`/`Backspace` back, and `q`
  following existing object-screen behavior;
- support `Enter` for Operation Show;
- support `d` for Operation Diff;
- support `S` for a stat or summary operation diff only after View Options can express that format;
- support `l` and `s` for read-only time-travel inspection only after the target command specs can
  carry `--at-operation`;
- show disabled recovery actions when confirmation is not implemented yet, instead of hiding the
  recovery model.

The first screen should not parse the entire operation DAG into a native model. It can start from
jj-rendered output, but selected-operation actions must retain a stable operation id instead of
searching visible text at action time.

## Read-Only Operation Commands

All read-only operation commands must use typed specs and the recording runner.

Operation Log:

```text
jj op log
```

Operation Show:

```text
jj op show OPERATION
```

Operation Diff:

```text
jj op diff OPERATION
```

Requirements:

- use `SafetyClass::ReadOnly` and `ExecutionMode::RenderReadOnly`;
- record `CommandFamily::JjOperation` and typed source actions rather than `Other("op")` where
  practical;
- render process argv from `JjCommandSpec::process_argv()`;
- preserve repository, output policy, config overlays, and operation time-travel flags in jj global
  option order;
- keep output bounded in command history using the current retention rules;
- show failures as read-only output views or recoverable status messages.

`jj op show` and `jj op diff` should be child views in the view stack. They should not replace the
Operation Log selection state.

## Recovery Mutation Boundaries

Recovery commands are local repository mutations and require preview before execution.

The first foundation may implement these as disabled actions plus specs/tests, or implement one
confirmed command if the preview path is already stable enough. It must not run them directly from a
single keypress.

### Undo And Redo

Global keys:

- `u`: preview `jj undo`;
- `U`: preview `jj redo`.

Default safety:

- `SafetyClass::LocalRewrite`;
- `ExecutionMode::ConfirmMutation`;
- refresh the active source view after success;
- record the resulting operation id when discoverable.

Confirmation policy:

- normal confirmation is enough for undo/redo from the latest operation context;
- the preview must show the current operation context and the command that will run;
- if the app is currently viewing an older operation with `--at-operation`, `u/U` should be disabled
  or require an advanced confirmation that explicitly says it will create a new concurrent
  operation from an older context.

### Operation Restore

Operation Log key:

- `r`: preview `jj op restore OPERATION`.

Default safety:

- `SafetyClass::DestructiveLocal`;
- `ExecutionMode::ConfirmMutation`;
- strong confirmation.

Strong confirmation means the preview requires one additional deliberate action beyond `Enter`.
Acceptable first designs include typing the short operation id, pressing a second confirmation key,
or choosing a clearly labeled "restore repository to this operation" row. The exact UI can be
chosen during implementation, but a single accidental `r Enter` path is not acceptable.

The preview must say that restore moves the repository to the selected operation state.

### Operation Revert

Operation Log key:

- `v`: preview `jj op revert OPERATION`.

Default safety:

- `SafetyClass::LocalRewrite`;
- `ExecutionMode::ConfirmMutation`;
- strong confirmation.

The preview must explain that revert applies the inverse of the selected operation as a new
operation. This is safer than restore in some workflows, but it is still a repository mutation and
must not be treated as read-only.

### Restore Versus Revert Naming

Use jj's exact operation language in UI labels:

- "restore operation" for `jj op restore`;
- "revert operation" for `jj op revert`;
- "undo" and "redo" only for `jj undo` and `jj redo`.

Do not reuse "restore" for file content recovery in this screen. File/revision restore belongs to
later content workflows and should route through separate selector roles.

## Command History And Operation Ids

Command History is the bridge between a mutation and recovery.

Required behavior after a confirmed mutation:

- create no history record until the user confirms the preview;
- record the confirmed mutation through the existing command-history recorder;
- keep `operation_id = None` when no reliable id is available;
- fill `operation_id` when the runner can obtain it from a stable source;
- show the post-mutation footer with Command History and Operation Log entry points;
- prefer exact operation ids internally, while allowing short ids for display and typed
  confirmation.

Allowed operation-id sources:

- a future structured runner result;
- stable jj output that already reports the operation id;
- a bounded follow-up query only when the implementation can prove it is race-aware and does not
  invent an id for an unrelated external operation.

Disallowed sources:

- parsing unrelated visible graph text;
- assuming the newest `jj op log` row belongs to the mutation without comparing pre-run and
  post-run context;
- running unbounded recovery probes after every read-only command.

Command History actions:

- `o` opens the recorded resulting operation when `operation_id` is present;
- `o` opens Operation Log when no specific operation id is known;
- `Enter` remains output/details for the command record;
- rerun and copy remain out of scope unless the Command History plan has already added them.

Recovery commands themselves must be recorded in Command History with their own operation ids when
available. The original command record should not be mutated after the fact except to attach an id
that belongs to that command.

## `--at-operation` And Run Options

Operation time travel is a read-only inspection feature by default.

When a user opens graph, status, show, diff, or operation views at a selected operation:

- build the target command with `GlobalOptions::operation = AtOperation(OPERATION)`;
- let `--at-operation` imply ignored working-copy behavior in argv rendering;
- label the screen with the selected operation context;
- show "working copy ignored" or equivalent status when the screen is time-traveled;
- keep the selected operation id in the view source metadata so refresh and history can explain the
  context.

Run Options interaction:

- the full Run Options drawer is not required in this slice;
- this slice must not silently enable `--no-integrate-operation`;
- this slice must not expose `--ignore-immutable`;
- this slice must not let ordinary recovery keys mutate an older operation context;
- future Run Options may explicitly allow advanced concurrent-operation mutation, but only through
  a preview that states the loaded operation and integration policy.

`--no-integrate-operation` remains a future simulation/sandbox option for local commands. It is not
a recovery default and must never be applied automatically to network commands.

## Safety Classes

Use the current safety classes consistently:

- `ReadOnly`: `jj op log`, `jj op show`, `jj op diff`, and views loaded with
  `--at-operation`.
- `LocalMetadata`: not expected in this slice except existing workspace operations.
- `LocalRewrite`: `jj undo`, `jj redo`, `jj op revert`, and normal history-rewrite recovery.
- `DestructiveLocal`: `jj op restore` and any future operation action that discards current local
  state to match an older operation.
- `NetworkRead`: not part of operation recovery.
- `NetworkWrite`: not part of operation recovery.
- `ExternalCommand`: not part of operation recovery.

If an implementation cannot confidently classify a command, it should not wire the action yet.

## Key Surface

Global or post-mutation footer:

- `u`: undo preview;
- `U`: redo preview;
- `o`: Operation Log or resulting operation;
- `C`: Command History, following the existing command-history surface;
- `?`: generated contextual help.

Operation Log:

- `Enter`: Operation Show;
- `d`: Operation Diff;
- `S`: stat or summary operation diff after View Options supports it;
- `V`: View Options after the operation views participate in the shared overlay;
- `l`: log at selected operation;
- `s`: status at selected operation;
- `r`: restore selected operation, strong confirmation;
- `v`: revert selected operation, strong confirmation;
- `u`: undo from current operation context, confirmed when enabled;
- `U`: redo from current operation context, confirmed when enabled;
- `y`: copy operation id or command only after copy support exists for the adjacent surfaces.

Keys should be generated from the same keymap registry as the current help and hotbar. Do not
hard-code help text inside the operation view.

## Selector Role Model

This slice should define operation selectors as one member of the shared selector model, not as a
bespoke op-log-only prompt.

Operation selector output:

- selected operation id;
- short display id;
- operation description or title;
- parent operation ids when available;
- whether it is current, root, or otherwise special when jj output exposes that safely;
- source view and key/action that selected it.

Roles:

- `OperationTarget`: the operation to show, diff, restore, revert, or inspect at;
- `OperationBase`: optional future base operation for operation diff ranges if jj supports or
  exposes that shape;
- `OperationContext`: the operation loaded by `--at-operation`;
- `ResultingOperation`: an operation id attached to a command-history record.

The same selector concepts should later compose with revision, fileset, bookmark, tag, remote, and
workspace selectors. Recovery should not invent a separate selection stack.

## Betamax Evidence Matrix

Betamax evidence should live under local ignored artifacts:

```text
tapes/validation/operation-recovery-foundation.tape
target/vibe-artifacts/operation-recovery/
```

- Open operation log: `op-log.txt` proves `o` opens a focused Operation Log screen from a fixture.
- Inspect operation: `op-show.txt` proves `Enter` opens `jj op show OPERATION` as a child view.
- Diff operation: `op-diff.txt` proves `d` opens `jj op diff OPERATION` as a child view.
- Mutation footer: `post-mutation-footer.txt` proves confirmed describe shows `u`, `U`, `o`, and
  the history route.
- History to operation: `history-op-link.txt` proves a history row with an operation id opens that
  operation.
- Read-only time travel: `log-at-operation.txt` proves `l` opens log with `--at-operation` and a
  working-copy note.
- Undo preview: `undo-preview.txt` proves `u` shows a preview and does not run until confirmed.
- Restore guard: `restore-guard.txt` proves `r` requires strong confirmation before restore can
  run.

If operation-id discovery is not reliable in the first implementation, split the evidence:

- one fake-runner or unit/state test proves history rows with operation ids route correctly;
- one Betamax journey proves the fallback path opens Operation Log when the id is absent.

Betamax should prove visible user journeys. Use unit and state tests for parser edge cases,
selection preservation, disabled keys, and command-spec ordering.

## Non-Goals

- Do not implement `:` command mode or `!` external command mode.
- Do not add persistent cross-session command history.
- Do not add command rerun, command editing, or copy-to-clipboard unless already implemented by
  adjacent work.
- Do not add full Run Options UI.
- Do not expose `--ignore-immutable` or `--no-integrate-operation` as recovery toggles.
- Do not allow ordinary mutation from an older `--at-operation` context.
- Do not implement file/revision restore, squash, split, absorb, rebase, abandon, commit, edit, or
  bookmark recovery in this slice.
- Do not parse native operation DAG data unless a stable jj output source exists.
- Do not add operation abandon or integrate commands.
- Do not regenerate README, crates.io, website, screenshots, GIFs, or public media.

## Acceptance Criteria

This slice is ready when:

- Operation Log opens as a focused screen backed by a typed `jj op log` spec;
- Operation Show and Operation Diff open from a selected operation with typed read-only specs;
- operation command records use command family, source view/action, safety, execution mode, refresh
  plan, context, and bounded output summaries from the shared command-history model;
- successful mutations have a visible route to Command History and Operation Log;
- command-history records with `operation_id` can route to that operation;
- command-history records without `operation_id` degrade to the Operation Log fallback;
- `u/U`, restore, and revert have documented preview and confirmation policies before any of them
  can run;
- `jj op restore` requires strong confirmation if implemented;
- `--at-operation` inspection screens are clearly labeled and do not silently enable mutation;
- Run Options compatibility is preserved without adding the full drawer;
- safety classes are assigned consistently for every wired operation action;
- operation selectors are modeled as selector roles rather than one-off visible-text parsing;
- generated help and hotbar text include only actions that are actually enabled;
- tests cover command-spec ordering, safety classification, selector routing, disabled recovery
  keys, command-history operation routing, and operation time-travel labels;
- Betamax artifacts under `target/vibe-artifacts/operation-recovery/` prove the read-only op-log
  journey and any enabled confirmation journey.

## Dependency Order

1. Keep the existing command-spec, global-options, command-history, and safe-mutation preview
   foundations in place.
1. Add operation source view/action and command-family coverage where current enums need typed
   variants.
1. Add the operation selector role model and selected-operation state.
1. Add read-only Operation Log backed by `jj op log`.
1. Add read-only Operation Show and Operation Diff child views.
1. Connect post-mutation success footer and Command History to Operation Log.
1. Connect history rows with known operation ids to Operation Show or focused Operation Log.
1. Add `--at-operation` read-only inspection flows for log/status only after labels and argv
   ordering are tested.
1. Add undo/redo preview only after the confirmation loop can express operation context.
1. Add restore/revert preview and strong confirmation after operation selectors are stable.
1. Add broader Run Options integration before advanced operation-context mutations.
1. Add content recovery and history-editing workflows only after this operation foundation is
   proven by tests and Betamax evidence.
