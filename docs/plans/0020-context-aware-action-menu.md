# Context-Aware Action Menu

## Goal

Ship one context-aware `a` action menu that is derived from the existing keymap and action
metadata, exposes only actions meaningful in the active view, and routes every selection through
the same existing action gateway and execution behavior.

The first shipped slice should cover the mutation and recovery workflows that already exist:

- describe;
- new;
- edit;
- abandon;
- undo;
- redo.

Make `m`, `n`, `e`, `u`, and `U` action-menu-only on the log. Change
`a` from direct abandon to opening the menu, with `a a` as the menu accelerator for abandon.

Do not implement rebase, squash, restore, refs, remote, or workspace workflows in this slice. The
registry shape should make those actions straightforward to add once their command specs and role
resolvers exist.

## Product Decisions

These decisions come from the current product plan and key conventions:

1. `a` opens a visible overlay, and action keys are active only while that overlay is open.
1. `a a` checks emptiness, then opens the existing `jj abandon` preview only for non-empty revisions.
1. `m`, `n`, `e`, `u`, and `U` are menu-only from the log.
1. The initial menu is meaningful only in the log context. Read-only inspection contexts must not
   expose revision mutations.
1. Rows are grouped as **Change actions** and **History and recovery**.
1. Each row explains whether it previews, runs immediately, or checks before a destructive action.
   The existing `JjCommandSpec` remains authoritative for execution mode and safety.
1. Selecting a row returns an existing semantic application action. It must not reconstruct jj
   arguments or duplicate mutation orchestration.
1. Help, discovery, hotbar text, and the action menu are projections of the same binding registry.

Primary evidence:

- `docs/product-plan.md`, sections 3.5, 3.7, 3.8, and 5.4;
- `docs/plans/cli-surface-addendum.md`, section 5.3;
- `crates/jk-tui/src/keymap.rs`, the existing contextual binding registry;
- `crates/jk/src/actions.rs`, the existing app-action dispatcher;
- `crates/jk/src/main.rs`, the existing mutation preview entry points;
- `crates/jk/src/menus.rs`, the existing selector and row-rendering primitives.

## Implementation State

The implementation is now complete for the shipped slice. The completed change:

- adds action-menu metadata and rows to `crates/jk-tui/src/keymap.rs`;
- exports the menu metadata through `jk_tui::command_discovery`;
- adds `InputMode::ActionMenu` and an action-bearing input result;
- maps `a` to `OpenActionMenu` and maps menu selections back to existing `AppKey` variants;
- adds menu input, selection, compact rendering, and initial tests;
- changes abandon's displayed source key to `a a`;
- updates README, usage, roadmap, product-plan text, and a focused Betamax tape.

The action menu remains intentionally limited to the six executable workflows above. Rebase,
squash, restore, workspace, refs, and remote workflows remain extension points rather than
placeholder rows.

## Implementation Plan

### 1. Audit And Reduce The Diff

1. Run `jj diff --stat`, `jj diff --summary`, and inspect the complete diff.
1. Remove accidental formatting or documentation churn unrelated to the shipped interaction.
1. Check that `ActionMenuAction`, `ActionMenuGroup`, `ActionMenuSafety`, and `ActionMenuRow` name
   durable concepts rather than mirroring transient rendering details.
1. Keep one authoritative action-menu inventory in the keymap registry. Do not add a second list in
   `jk` for labels, keys, grouping, or ranking.
1. Keep application effect routing in `jk`; `jk-tui` may expose semantic menu metadata but must not
   own command construction or repository mutation.

### 2. Finish Registry And Metadata Semantics

In `crates/jk-tui/src/keymap.rs`:

1. Verify that the top-level log binding says `a` opens actions.
1. Verify that the abandon discovery/help entry says `a a`, while its in-menu accelerator is `a`.
1. Keep the direct binding for `m`; advertise `n`, `e`, `u`, and `U` as `a n`, `a e`, `a u`, and
   `a U` in log help while retaining their in-menu accelerators.
1. Ensure menu rows are ranked independently of source declaration order.
1. Ensure menu groups use the existing user-facing vocabulary: **Change actions** and
   **History and recovery**.
1. Ensure safety labels are explanatory UI cues only. The command spec remains the safety source of
   truth.
1. Add a documented extension point that can later register rebase, squash, restore, workspace,
   refs, and remote actions without changing menu state or rendering code.

Do not add placeholder rows for unimplemented workflows. The menu must show only executable actions.

### 3. Finish State And Input Behavior

In `crates/jk/src/state.rs`, `crates/jk/src/main.rs`, and `crates/jk/src/key.rs`:

1. Keep action-menu selection as transient `InputMode` state owned by the terminal loop.
1. Open the menu only when `action_menu_rows(active_context)` is non-empty.
1. Support `j`, `k`, up, down, `Enter`, action accelerators, `Esc`, and backspace.
1. Decide intentionally whether `q` closes the menu or quits the application; match existing modal
   conventions and test the choice.
1. Ensure unknown characters are consumed by the menu and cannot leak to the background view.
1. Ensure selection remains valid if context filtering changes or the row list becomes empty.
1. Ensure opening, navigating, and closing the menu performs no process execution and creates no
   command-history record.

### 4. Prove Dispatch Reuse

In `crates/jk/src/actions.rs` and focused tests:

1. Route each selected `ActionMenuAction` to the existing `AppKey` or existing action gateway.
1. Verify the selected menu action reaches these existing owners:

   - `open_describe_message`;
   - the recorded New and Edit mutation runner;
   - the conditional Abandon probe and existing destructive preview;
   - the recorded recovery runner for undo and redo.

1. Do not create `DescribeQuery`, `NewQuery`, `EditQuery`, `AbandonQuery`, or recovery command specs
   inside menu code.
1. Verify non-empty `a a` produces the same `PendingCommandPreview`, `SourceAction`, safety class,
   history behavior, and refresh behavior as the former direct abandon path; verify empty `a a`
   executes through that same recorded and refreshed runner without opening a preview.
1. Verify menu Describe still produces the existing state transition; on the log,
   `m`, `n`, `e`, `u`, and `U` are consumed as action-menu accelerators while their existing context-specific
   meanings remain available in other views.

### 5. Finish Rendering And Narrow Layout

In `crates/jk/src/menus.rs` and `crates/jk/src/rendering.rs`:

1. Render task-oriented group headings, a single aligned key column, action labels, and safety cues.
1. Highlight only selectable action rows; headings and instructions are not selectable.
1. Keep the underlying log visible but clear or suppress duplicate hotbar controls beneath the
   overlay.
1. Size the overlay from its full content so selection changes do not resize it.
1. At narrow widths, shorten safety text before shortening action labels or keys.
1. Prevent row wrapping at representative widths. If the terminal is too narrow for the full row,
   switch to a tested compact layout.
1. Add scrolling only if the real shipped row count exceeds the available height. Do not add a
   speculative scroll state for six rows unless a small terminal requires it.

### 6. Complete Focused Tests

Add or finish tests for these contracts:

#### Registry And Metadata

- Log rows are exactly describe, new, edit, abandon, undo, and redo in intended rank order.
- Change and recovery groups are correct.
- Describe is labeled as a local rewrite whose prompt saves with `Enter`; New and Edit are labeled as
  immediate local rewrites; Abandon is labeled as destructive with an emptiness check; Undo and Redo
  are immediate local rewrites.
- Diff, inspection, workspaces, command history, and operation log do not expose revision actions in
  this slice.
- Help and discovery show `a` for the menu and `a a` for abandon.
- Hotbar text says `a actions`.
- Every menu row has help/discovery metadata and maps to one semantic application action.

#### State And Input

- `a` opens the menu on log and does not open it in contexts with no actions.
- Initial selection is the first ranked action.
- Up/down and `j`/`k` wrap consistently with existing menu primitives.
- `Esc` and backspace close without dispatch.
- `Enter` dispatches the selected row.
- Each action key dispatches its row directly.
- Unknown keys do nothing and do not close or leak through.

#### Existing Behavior Reuse

- Menu describe enters the existing describe-message mode.
- Menu Describe, New, Edit, Undo, and Redo execute through the existing recorded mutation runner.
  Empty-revision Abandon does the same after a read-only `self.empty()` probe; non-empty Abandon
  creates the existing command preview.
- `SourceAction`, safety class, source key, preview command, confirmation, refresh, and command-history
  behavior remain owned by the existing mutation gateways. Successful executions show a short-lived
  toast without replacing the normal footer controls.
- Menu `m`, `n`, `e`, `u`, and `U` select their rows without leaking into the underlying log; their
  existing context-specific meanings remain
  covered elsewhere.

#### Rendering

- The overlay shows title, groups, selected marker, action keys, and safety cues.
- The underlying hotbar does not compete with overlay instructions.
- Representative widths such as 40, 80, and 120 columns do not wrap or truncate keys.
- A short terminal does not panic or place content outside the buffer.

Prefer structural assertions over a broad full-buffer snapshot.

### 7. Add Visual Proof

Create `tapes/action-menu.tape` or another focused Betamax tape using the deterministic repository
fixture pattern from existing tapes.

The tape should:

1. hide fixture setup and build output;
1. open the log and press `a`;
1. capture a PNG showing the normal-width menu;
1. select abandon with the second `a` and prove the existing preview opens for a non-empty revision
   without executing it;
1. cancel the preview;
1. capture a narrow-width PNG showing the compact menu;
1. keep generated screenshots under `target/dogfood-artifacts/` and out of the tracked repository.

Use semantic `Wait+Screen` assertions before screenshots. A static PNG is sufficient; a GIF is not
needed unless accelerator-to-preview motion is important to the review.

### 8. Align Durable Documentation

Update only current-behavior documentation:

- `README.md`;
- `crates/jk/README.md`;
- `docs/usage.md`;
- `docs/roadmap.md` current implementation notes;
- `docs/product-plan.md` current implementation notes and the action map.

Describe:

- `a` opens the log action menu;
- `a a` checks emptiness and previews only non-empty abandon;
- `m`, `n`, `e`, `u`, and `U` are menu-only on the log;
- non-empty Abandon opens a preview before execution; New, Edit, Undo, Redo, and empty-revision
  Abandon execute immediately through the same recorded and refreshed mutation path;
- broader revision, workspace, refs, and remote action menus remain planned.

Do not rewrite completed historical plan records merely to make them read as if the menu existed at
the time. Check `/Users/joshka/local/jk-website` for copy or media drift after the code is stable,
but coordinate that as a separate repository change because this task is restricted to the action
menu workspace.

## Validation

Run focused checks while editing:

```sh
cargo test -p jk-tui keymap::tests --no-fail-fast
cargo test -p jk --bin jk action_menu --no-fail-fast
cargo test -p jk --bin jk
```

Then run the repository gates:

```sh
just fmt-check
just check
just clippy
just test
just lint-md
```

If the local `/opt/homebrew/bin/cargo` still rejects `cargo +nightly`, run formatting with the
nightly toolchain's cargo and rustfmt explicitly, then record the environment mismatch in the
handoff. Do not silently substitute stable rustfmt because this repository relies on nightly-only
formatting options.

Run the focused Betamax tape and inspect the generated PNG in Firefox, using a new filename if an
older capture may be cached.

Validation completed for this implementation:

- `cargo test --workspace --all-features --no-fail-fast` passed (464 tests across the workspace);
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed;
- `just check`, `just clippy`, `just test`, and `just lint-md` passed;
- nightly `cargo fmt --all -- --check` and Rust comment reflow checks passed directly;
- `betamax validate tapes/action-menu.tape` and `just betamax-action-menu` passed;
- normal-width, abandon-preview, direct-describe-cursor, and direct-describe-success PNGs were
  inspected under `target/dogfood-artifacts/betamax/`.

The repository `just fmt-check` recipe remains blocked by this checkout's `/opt/homebrew/bin/cargo`
wrapper, which rejects the `cargo +nightly` syntax; the direct nightly command above is equivalent
for this environment.

## Acceptance Criteria

The slice is complete when:

- `a` opens a context-aware action overlay from the log;
- the overlay contains only the six shipped workflows;
- `a a` checks emptiness, executing empty revisions immediately and previewing non-empty revisions;
- `m`, `n`, `e`, `u`, and `U` are action-menu accelerators; Describe saves on `Enter` without a
  second confirmation;
- all menu selections route through existing mutation and recovery behavior;
- grouping and safety cues are clear at normal and narrow widths;
- read-only contexts do not expose revision mutations;
- help, discovery, hotbar, usage docs, and roadmap agree;
- focused tests, repository checks, Markdown lint, and visual proof pass;
- `jj diff` contains no unrelated formatting churn;
- no rebase, squash, restore, workspace, refs, or remote workflow is implemented.

## Recommended Change Shape

Keep this as one atomic change because the keymap metadata, menu mode, dispatcher bridge, rendering,
tests, and current-behavior docs describe one user-visible interaction and should not land in a
state where `a` no longer reaches abandon.

If Luna needs to separate cleanup from behavior for review, use adjacent jj changes in this order:

1. behavior-preserving registry/API cleanup, only if required;
1. action-menu behavior, tests, and docs;
1. Betamax tape adjustments, squashed into the behavior change once verified.

Do not stack future rebase, squash, restore, workspace, refs, or remote implementations on this
change. Those should follow after this menu seam lands.
