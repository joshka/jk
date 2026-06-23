# Selected Workspace Update-Stale

Status: draft/spec

Owner: current dogfood workspace pass

Scope: second workspace implementation slice

## Decision

Add a narrow `u` action on the Workspaces screen that runs `jj workspace update-stale` for the
selected workspace, then refreshes the workspace list.

This is the second workspace chunk after the read-only Workspaces screen. It should build on the
existing `W` screen, selected-row model, workspace provider, `JjCommandSpec` command construction,
generated help, and rendered status/diff child views. It should not become a broad workspace
mutation framework.

The first reviewable implementation should:

- Bind `u` only in the Workspaces screen.
- Require a selected workspace row with a known root path.
- Build the command through `JjCommandSpec`.
- Execute the selected-workspace command as:

  ```text
  jj -R WORKSPACE_ROOT workspace update-stale
  ```

- Show the command being run in help, status, and error output.
- Refresh the workspace list after success.
- Preserve the selected workspace by name when it still exists.
- Keep the previous list visible and show the command output on failure.
- Add focused unit tests, a stale-workspace fixture path, and Betamax evidence.

## Context

[0012 Workspace Scope](0012-workspace-scope.md) deliberately kept
`jj workspace update-stale` out of the first chunk because it mutates workspace metadata. The first
chunk established the `W` screen, current-workspace marker, selected-workspace `s` status, selected
workspace `d` diff, manual `r` refresh, generated workspace help, and view-stack integration.

The roadmap keeps workspaces in the 0.5 core scope and names update-stale in the acceptance
criteria. The product plan also treats workspace stale handling as early work, but it says mutating
actions should be command-shaped, visible, and followed by state refresh and recovery affordances.

Current code behavior relevant to this slice:

- `crates/jk-cli/src/workspaces.rs` owns `JjWorkspaces`, workspace list parsing, and selected
  workspace status/diff specs.
- `crates/jk-tui/src/workspaces_view.rs` owns provider-neutral workspace rows, selection
  preservation, error status text, and `WorkspacesActionResult` effects.
- `crates/jk-tui/src/keymap.rs` generates Workspaces help, hotbar, and discovery rows.
- `crates/jk/src/key.rs` maps terminal keys to reusable log-style actions and special app keys.
- `crates/jk/src/main.rs` maps Workspaces actions to provider calls, view-stack effects, and
  refresh behavior.
- `u` is not currently bound for the workspace list.

## Goals

- Let the user update a stale selected workspace without leaving `jk`.
- Keep the command scoped to the selected workspace root, not the process current directory.
- Preserve jj command fidelity by constructing and executing a typed `JjCommandSpec`.
- Keep this mutation visible even before the full Command Preview mode exists.
- Refresh the workspace list after success so stale/current state and target IDs are not left stale.
- Preserve selection, scroll, help, and view-stack behavior already proven by the read-only slice.
- Capture command output well enough to debug a failed or no-op stale update.
- Leave the implementation shape compatible with future Run Options and Command Preview.

## Non-Goals

- Do not implement `jj workspace add`.
- Do not implement `jj workspace forget`.
- Do not implement `jj workspace rename`.
- Do not implement workspace switching or implicit process-directory changes.
- Do not add a broad workspace mutation runner.
- Do not add a reusable Command Preview mode in this slice.
- Do not add a reusable Run Options drawer in this slice.
- Do not add command history storage unless the command-history primitive already exists.
- Do not implement background refresh, auto-refresh, or cancellable task orchestration.
- Do not add sparse-pattern controls or workspace creation options.
- Do not rewrite workspace list rendering beyond the minimum needed to show update-stale state.
- Do not regenerate website media or public README media for this slice.

## Command Shape

The provider should expose one new command spec for selected-workspace stale updates:

```rust
pub fn update_stale_spec(&self, query: &WorkspaceInspectionQuery) -> JjCommandSpec
```

The spec should use the selected workspace root as the jj repository:

```text
jj --repository WORKSPACE_ROOT workspace update-stale
```

The display title can use the existing command-title convention:

```text
jj -R WORKSPACE_ROOT workspace update-stale
```

If the implementation standardizes on `--repository` in titles before this lands, use that current
convention instead. The important contract is that the visible title comes from the same
`JjCommandSpec` used for execution.

The command should not use `--ignore-working-copy`, `--at-operation`, `--no-integrate-operation`,
`--ignore-immutable`, `--config`, or `--config-file` in this slice. Those belong to future Run
Options.

### Safety Class

Classify the spec as local metadata mutation:

```rust
SafetyClass::LocalMetadata
```

Use a mutating execution mode if one already exists by implementation time. If the only available
execution mode is still `RenderReadOnly`, add the smallest command-core extension needed to
represent a foreground local mutation without pretending it is read-only.

The spec refresh plan should be list refresh, not re-run rendered output:

```rust
RefreshPlan::RefreshWorkspaceList
```

If `RefreshPlan` does not yet have workspace-specific variants, keep the execution path explicit in
the Workspaces app handler and leave a test naming the desired refresh contract.

## UI And Key Behavior

### Workspaces Screen

Add `u` in the Workspaces binding context:

- Label: `u update-stale`.
- Help text: `update selected stale workspace`.
- Command family: `jj workspace`.
- Discovery aliases: `update`, `stale`, `workspace`, `selected`, `refresh`.

The hotbar should include `u update-stale` only when it fits with the existing adaptive behavior.
It must always appear in `?` help and searchable discovery even if the hotbar omits it.

Pressing `u` should:

1. Do nothing if there is no selected row, except optionally show `No workspace selected`.
1. Show an inline error if the selected row has no root path.
1. Run update-stale for the selected row's root path.
1. Refresh the workspace list after success.
1. Preserve the Workspaces screen as the active view.
1. Leave status/diff child views unchanged unless the user explicitly opens or refreshes them.

### Key Translation

The implementation should avoid overloading existing global meanings outside Workspaces. One small
way to do that is:

- add a reusable semantic action such as `LogAction::UpdateStale`, or
- add a workspace-specific app key if the app key layer grows screen-aware dispatch first.

The durable behavior matters more than the exact enum name:

- `u` updates stale only while `AppView::Workspaces` is active.
- `u` remains unbound or reserved elsewhere until undo/redo and command preview policy are defined.
- `?` still opens command discovery for Workspaces rather than directly toggling ad hoc help.
- `Esc`, `Backspace`, `H`, and `L` continue to return to the previous view.

### Status Text

While the command is synchronous, the status line only needs stable before/after messages:

- Before or during run: `running jj -R WORKSPACE_ROOT workspace update-stale`.
- Success with output: `updated WORKSPACE_NAME: OUTPUT_SUMMARY`.
- Success without output: `updated WORKSPACE_NAME`.
- Failure: `jj -R WORKSPACE_ROOT workspace update-stale failed: STDERR_SUMMARY`.

Do not add a spinner or background job state until the cancellable task model exists.

## Safety And Confirmation

This slice is intentionally lighter than future rewrite/destructive mutations:

- Direct `u` execution is acceptable for this first update-stale slice.
- The command must be visible in help/status/error output.
- No extra confirmation is required for a selected workspace with a known root.
- No strong confirmation is required.
- Do not expose `e` edit-command or `y` copy-command here unless Command Preview exists.

Rationale: `jj workspace update-stale` updates local workspace metadata so a stale workspace can be
used again. It is not a history rewrite, destructive workspace removal, or network operation. It
still mutates repository metadata, so the command must not be hidden behind a generic refresh.

When Command Preview lands, this action should route through the normal preview loop unless the user
has configured local metadata actions as no-confirm. Until then, the Workspaces screen itself is the
minimal command-visible confirmation surface.

## State Refresh

After a successful update-stale run:

1. Reload `jj workspace list` through the same provider path used by `r`.
1. Replace rows only if the reload succeeds.
1. Preserve selection by workspace name when the selected workspace still exists.
1. Fall back to the current workspace row, then clamp to the nearest available row.
1. Clamp scroll so the selected row remains visible.
1. Clear stale error text and show a short success message.

If the update-stale command succeeds but list refresh fails:

- Keep the previous list visible.
- Show a status message that distinguishes command success from refresh failure.
- Include enough detail to make `r` retry obvious.

Example:

```text
updated dogfood; refresh failed: jj workspace list failed: ...
```

If stale-state display is available by implementation time, the refreshed row should no longer show
stale. If stale-state display is not available yet, this slice can still land as long as the command
runs, the list refreshes, and Betamax evidence shows the underlying jj behavior.

## Error Handling

The Workspaces screen should keep the old list visible on every failure path.

Handle these cases explicitly:

- No selected row: no command runs.
- Selected row has no root: show `workspace NAME has no root`.
- `jj` process cannot start: show the I/O error.
- `jj workspace update-stale` exits unsuccessfully: show stderr and keep the old list.
- `jj workspace update-stale` exits successfully but emits warnings: show a compact output summary.
- Post-success `jj workspace list` fails: keep the old list and show the refresh failure.

Failure output should prefer `stderr`, then non-empty `stdout`, then the exit status. Avoid
replacing the Workspaces view with a rendered output child view in this slice; keep the failure on
the list so the user can retry or inspect status/diff.

If command output is too long for the status line, store the full output in the view state and show
a one-line summary. A future command-output/history screen can expose the full command transcript.

## Betamax Evidence

Add Betamax evidence under `target/dogfood-artifacts` for the full Workspaces stale-update journey.
The evidence should prove the user-visible behavior, not just unit-level command construction.

Suggested tape:

```text
tapes/validation/workspace-update-stale.tape
```

The tape should use a deterministic fixture with:

- a primary workspace;
- a sibling workspace that becomes stale after an operation in the primary workspace;
- `jk` opening the Workspaces screen;
- selection moving to the stale workspace;
- `u` running update-stale;
- the Workspaces screen refreshing and preserving selection;
- status text showing the command or result;
- `s` or `d` still working for the selected workspace after update.

Capture at least:

- a screenshot before pressing `u`;
- a screenshot after the list refresh;
- a state JSON file if the current Betamax helper supports it;
- the raw command transcript or fixture setup log needed to reproduce stale state.

If the installed jj version changes the exact output text, assertions should key on stable visible
contracts such as the workspace name, `workspace update-stale`, refreshed row presence, and selected
row preservation.

## Tests

### Command And Provider Tests

Add focused tests in the provider layer:

- `update_stale_spec` renders repository before `workspace update-stale`.
- the display title includes the selected workspace root.
- the spec has the local metadata safety class.
- successful update-stale output is captured for status reporting.
- failed update-stale stderr is surfaced without running list refresh.

If command execution is hard to fake with the current adapter, introduce the smallest test seam
needed for `JjWorkspaces` command execution rather than broad dependency injection.

### TUI State Tests

Add provider-neutral `WorkspacesView` tests:

- `u` maps to a new `WorkspacesActionResult::UpdateStale`.
- no selected row ignores `u`.
- missing root keeps the view and shows an error.
- success status survives the post-success refresh instead of being cleared accidentally.
- selection preservation after update-stale follows the same rules as manual refresh.
- help lines include `u update selected stale workspace`.
- discovery can find the action by `workspace stale`.

### App Wiring Tests

Add binary-layer tests where current seams allow:

- pressing `u` in Workspaces requests the selected-workspace update path.
- pressing `u` outside Workspaces does not run workspace update-stale.
- update failure does not push a child view.
- update success refreshes the existing Workspaces view rather than stacking another Workspaces
  view.

### Fixture Tests

Add or extend a multi-workspace fixture so future work can share it:

```text
tapes/fixtures/multi-workspace-repo.sh
```

The fixture should be deterministic, isolated, and able to create one stale workspace without using
the developer's real checkout. It should be suitable for the later forget/add workspace slices too,
but this slice should only assert update-stale behavior.

## Interaction With Future Run Options

This slice should not add a Run Options drawer, but it should avoid blocking one.

Implementation constraints:

- Keep repository/root, working-copy policy, output policy, and config overlays inside
  `JjCommandSpec` and `GlobalOptions`.
- Do not hand-build argv in the app event handler.
- Do not store a bare shell string as the source of truth.
- Do not bake `--ignore-working-copy` or operation-time-travel flags into the action.
- Keep the selected workspace root as a resolver output that Run Options can display later.

When Run Options exists, this action should inherit the shared global-option surface. The default
should remain the current safe direct command:

```text
jj -R WORKSPACE_ROOT workspace update-stale
```

Advanced users can then opt into command-wide flags through Run Options without this screen needing
special per-flag UI.

## Interaction With Future Command Preview

This slice should make command preview adoption mechanical:

1. The Workspaces screen resolves the selected workspace row.
1. The provider builds a `JjCommandSpec`.
1. The app currently runs the spec directly because local metadata direct-run is allowed.
1. Future Command Preview can intercept the same spec and show it before execution.
1. On success, the same refresh path runs.
1. On failure, the same output-preservation path displays the transcript.

Do not introduce a one-off preview overlay in this slice. It would duplicate the future preview
mode and make later command-history integration harder. The durable contract is that all data needed
by the preview already exists in the spec and selected workspace row.

## Acceptance

This plan is ready for implementation when the next slice can be described as:

- `W` opens the existing Workspaces screen.
- `j/k` selects a workspace.
- `u` runs `jj -R SELECTED_ROOT workspace update-stale`.
- success refreshes the list and preserves selected workspace identity where possible.
- failure keeps the list visible and shows the failed command output.
- `s` and `d` continue to inspect the selected workspace after the update.
- generated help, hotbar, and searchable discovery include the `u` action.
- tests cover command shape, view state, app wiring, and stale-workspace fixture behavior.
- Betamax artifacts prove the stale update journey under `target/dogfood-artifacts`.

## Deferred Follow-Up

After this slice, the workspace sequence should remain:

1. Forget confirmation.
1. Add workspace.
1. Rename workspace.
1. Operation log relevant to selected workspace.
1. Sparse options and workspace creation sparse-pattern controls.
1. Full Command Preview and command history integration.
1. Run Options for advanced global flags.

Do not pull any of those into the update-stale implementation unless a test seam requires a tiny
shared helper.
