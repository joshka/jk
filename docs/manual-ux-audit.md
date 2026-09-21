# Manual workflow UX audit

Arbitrary commands entered while inspecting a sibling workspace still target the startup repository,
and the prompt does not identify that target. Commands can also leave the graph stale or require an
interactive tool that jk cannot run in the foreground. These are the largest workflow gaps found in
the [implemented command forms](manual-coverage.json).

The native previews make many mutations reviewable, but several show only commit hashes, discard
failed input, or leave users without a clear next step. This audit proposes changes; it does not
implement them.

## Evidence and review method

Each finding identifies source code and any reviewed recording. A recording archive contains the
PNG, terminal State JSON, executed tape, command log, binary hash/version, and repository
assertions. Reviewers inspect the image, read the text and styles, and check the resulting
repository state. The structured audit retains exact artifact hashes. Findings based only on source
say so.

The earlier coherence proof used binary SHA256
`88f18812c9142b11cab5e0d1f9937848e35160a2f368f8c673939af02f8b1919`. The later form-style proof uses
its separately recorded binary. The review matrix labels these older captures as baseline evidence.

Priority is an audit recommendation: **P1** can send a user to the wrong repository operation;
**P2** blocks or obscures a routine workflow; **P3** adds avoidable effort or visual ambiguity.
Priority does not assert measured user error or a production incident.

## Concern to outcome map

| User concern                  | Source-control operation                                      | jk interaction                                                                        | Observable result                                                          | Cancellation and recovery                                                                                 |
| ----------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Understand current work       | `log`, `show`, `status`, `evolog`                             | Move the log cursor; open a view; use templates or a revset                           | jj output plus selected object; inspect diff before acting                 | Back restores the previous view; clear a restrictive revset; inspect Command History on failure           |
| Compare the right content     | Revision or from/to `diff`; display formats                   | Cursor or ordered marks; open Diff; change format; supported file navigation          | Diff header and jj patch identify compared content                         | Return to log; clear/reorder marks; reset format; use a fileset through command mode                      |
| Start or resume a change      | `new`, multi-parent `new`, `edit`                             | Choose cursor/ordered marks, then action menu                                         | Working-copy node changes; new parents appear in graph                     | Immediate actions have no confirmation stage; inspect operation history, then Undo                        |
| Explain a change              | `describe`                                                    | Edit multiline message and save                                                       | Description changes without changing file content                          | Esc before saving; Undo after saving; editor handoff remains unavailable                                  |
| Move existing work            | Nine rebase source/placement combinations                     | Freeze source, choose role and destination, review, run                               | Changed parent relationships and descendant rewrites                       | Esc before run; cancel preserves selection; inspect operation and Undo after run                          |
| Combine related work          | Whole-change `squash`                                         | Mark sources, put cursor on destination, preview, run                                 | Source edits move into destination; destination description remains        | Esc preserves marks; after success inspect destination and Undo if needed                                 |
| Restore previous content      | All-path `restore --from … --into @`                          | Cursor chooses source; preview identifies working copy as destination                 | Working-copy files match source; unrelated working-copy edits are replaced | Esc before run; inspect status/diff after run; Undo the operation                                         |
| Remove an unwanted change     | Empty/nonempty `abandon`                                      | Empty change runs immediately; nonempty/probe failure opens contents and consequences | Change disappears; descendants reconnect                                   | Cancel defaults in nonempty dialog; Undo from action menu after execution                                 |
| Name and retrieve shared work | Bookmark create/move/delete; remote list/fetch                | Bookmark form, remote picker, command preview                                         | Exact local bookmark or fetched remote references change                   | Esc before command; inspect command output/history; local Undo does not reverse external publication      |
| Publish work                  | One-bookmark Git push dry-run                                 | Choose bookmark and remote, run dry-run                                               | Proposal is shown; no remote ref is published                              | Cancel before run; actual publication requires leaving the native flow                                    |
| Work in another directory     | Workspace list/log/status/diff/add/rename/forget/update-stale | Select workspace; inspect or choose lifecycle action                                  | Selected workspace content or metadata changes                             | Cancel before confirmation; retry failed form requires reentry; forgetting keeps files                    |
| Investigate a mistake         | `op log`, `op show`, `op diff`, Undo/Redo                     | Open operations or Command History, inspect effect, undo/redo                         | Operation identity and repository changes become inspectable               | Return without mutation; Redo reverses Undo; operation restore/integrate use command mode                 |
| Adjust one command            | Run Options                                                   | Open options from pending command; apply or cancel                                    | Exact preview shows working-copy, operation, immutable settings            | Esc discards draft options; apply returns to review before execution                                      |
| Use an unsupported form       | `:` jj command or `!` executable                              | Enter noninteractive arguments; run; inspect captured output                          | Exit status and output are recorded                                        | Esc cancels input; Backspace returns from output; in-flight commands cannot cancel; refresh stale views   |

The manual gives the current recovery steps: clear marks, leave jk for an interactive tool, retype
failed workspace input, or refresh after a command. The proposals below aim to remove those extra
steps where jk can preserve the user's intent.

## Findings

### UX-01 command-mode-tool-handoff

**P2 · confirmed support boundary · commit, split, resolve, describe, external commands.**

The native UI has no commit, split, absorb, or resolve flow. Command mode captures output with stdin
closed, so a noninteractive form can run, but there is no foreground-terminal handoff to an editor,
interactive diff selector, or merge tool. A separate-window GUI tool is a different execution case.
The input footer says `esc cancel` before execution; execution itself is synchronous, so a long
command also prevents normal UI input until it returns.

Evidence: [command mode](../crates/jk/src/command_mode.rs), [event loop](../crates/jk/src/main.rs),
and [process runner](../crates/jk-cli/src/command.rs). The existing external-command recording
proves captured success/failure; it does not prove foreground tool behavior.

**Proposal:** introduce one explicit foreground tool handoff, beginning with editor describe and
reusing the lifecycle for split, diffedit, and resolve. Before leaving the TUI, show the exact
repository and command. Restore terminal state and input on every return path, record exit status,
and refresh affected views. Captured commands should separately expose running state and a bounded
cancellation path.

Acceptance requires disposable-repository recordings for successful tool exit, tool failure,
cancellation, missing executable, and a tool that changes the repository before failing. After each,
the terminal must remain usable and the graph must agree with direct jj output. A split walkthrough
must show selected and remaining content, not only a successfully spawned process.

### UX-02 publication-dead-end

**P2 · confirmed support boundary · git-push-dry-run.**

The bookmark screen can choose an exact bookmark and remote and run a push dry-run, but cannot
complete publication. `:git push`, including `--dry-run`, is rejected. The user must reconstruct the
publication command in `!` or a shell, re-entering the bookmark and remote reviewed in the dry-run.

Evidence: [command-mode validation](../crates/jk/src/command_mode.rs), [bookmark
routes](../crates/jk/src/bookmark_routes.rs), and the refs/remotes push-preview checkpoint.

**Proposal:** retain bookmark, remote, and dry-run result through an explicit publish confirmation.
Refresh the proposal if local or remote state changes; show the exact update being sent. Publication
must have a distinct result from a successful dry-run.

Acceptance requires a disposable local remote: prove dry-run changes no remote refs, confirmation
publishes exactly the selected bookmark, cancellation publishes nothing, rejection retains useful
context, and successful publication can be verified against the remote. Do not label local Undo as
remote recovery; explain the new remote update required to reverse publication.

### UX-03 partial-content-selection

**P2 · confirmed support boundary · squash, restore, commit, split.**

Native squash moves whole changes and always keeps the destination description. Native restore
replaces all paths in the working copy. Diff file/hunk navigation does not select content for these
mutations. A user who came to remove one accidental edit must leave the current interaction or
perform a wider operation than intended.

Evidence: [squash scope](../crates/jk/src/squash.rs), [restore
scope](../crates/jk/src/mutations.rs), and the role/scope text in the squash and restore previews.
Their `whole changes` and `all paths` labels warn about scope, but offer no way to narrow it.

**Proposal:** one content-selection flow shared by commit/split, then squash/restore. Show source,
destination, selected content, and content left behind. Keep file navigation distinct from
selection; show counts and a reviewable patch before mutation. Offer message handling when combining
changes.

Acceptance requires mixed files and multiple hunks in a disposable repository, exact selected versus
remaining-content assertions, empty selection, conflict content, file rename/deletion, cancel, and
Undo. No unselected byte may change because a cursor happened to be on a file or hunk.

### UX-04 role-visibility

**P2 · source and baseline pixels/state confirmed · rebase, squash, restore, ordered marks.**

The final rebase and squash previews identify source and destination by full commit IDs. Human
descriptions and change IDs remain in the underlying graph, often partly covered. This makes the
final check depend on matching hexadecimal strings back to earlier context. In the narrow rebase
chooser, source-role controls become `[r/b/s]` and placement becomes `[o/A/B]`, removing the meaning
of alternatives at the moment the user chooses them.

Evidence: [rebase selector and preview](../crates/jk/src/rebase.rs), lines 104–112 and 257–263;
[squash preview](../crates/jk/src/squash.rs), lines 119–129; baseline `design-proof-normal-preview`,
`design-proof-light-preview`, and `form-style-narrow-rebase-selector` PNG/State pairs. The baseline
narrow chooser text reads `Scope: revision only [r/b/s]` and `Place: onto [o/A/B]`.

**Proposal:** show compact change identity and description alongside each role, with the exact
commit still available in the command. Keep the role explanation in a narrow layout using stacked
controls or a focused choice, and show the currently chosen consequence before continuing.
Explicitly label ordered marks when order changes meaning, such as new parents or a comparison
range.

Acceptance requires choosing among changes with similar descriptions, ordered marks, all nine rebase
forms, and 60-column output. Users must be able to identify source, destination, scope, and ordering
from the dialog. Long commands and revision identities must remain inspectable.

The multi-parent New recording shows a related risk after success: the cursor moves to the new
working copy while both parent marks remain. A subsequent New uses those marked parents again,
creating a sibling merge rather than a child of the cursor. The footer says `2 marked`, but the next
action still runs immediately. Review the lifetime of marks after mutations and make the operands of
that next command visible before it runs. `new-parents-merged` records this state; [New
completion](../crates/jk/src/mutations.rs), lines 37–53 and 297–305, reselects the new change
without clearing marks.

### UX-05 abandon-diff-fidelity

**P2 · source and baseline pixels/state confirmed · abandon-nonempty, abandon-probe-failure.**

The abandon dialog's View diff action forces `diff --git` and color `never`, then colors lines
beginning with `+` or `-` using jk's red/green choice. It also wraps patch lines to dialog width.
This diff differs from the user's configured jj format/colors and can split code lines, contrary to
the [TUI design contract](tui-design.md). The regular Diff view already preserves jj-rendered
content.

Evidence: [abandon detail query](../crates/jk-cli/src/abandon.rs), lines 222–237, and [abandon body
renderer](../crates/jk/src/abandon_confirmation.rs), lines 300–307; baseline
`abandon-confirmation-diff-v2-20260919` PNG/State pair.

**Proposal:** reuse the normal diff rendering and scrolling contract inside this inspection step, or
open a clearly scoped Diff view that returns to the same decision. Preserve the cancel default,
opaque dialog, contents summary, descendant warning, and destructive-action styling.

Acceptance requires custom diff colors and a non-Git default format, long code lines, narrow output,
light/dark surfaces, and returning to the same decision after scrolling. Compare both rendered text
and styled spans with jj output under the same configuration.

### UX-06 modal-focus

**P3 · source and baseline pixels/state confirmed · dialogs over the log.**

Most modal branches call the normal focused log renderer before painting the overlay. The underlying
graph therefore retains the active cursor treatment while the dialog also shows a focus marker. Log
template/view-options selectors already use an unfocused background path, so modal focus is
inconsistent: two elements look active although only the dialog receives input.

Evidence: [app rendering](../crates/jk/src/rendering.rs), lines 24–104, and [log
rendering](../crates/jk-tui/src/log_view.rs), lines 407–422 and 473–486; baseline normal/light
preview and narrow rebase-selector PNG/State pairs. Repository content is correctly retained around
the opaque surfaces.

**Proposal:** give every overlay an explicit inactive-background render path. Preserve cursor
position and marks as context, with only the active surface owning the focus style. Do not dim or
recolor jj-owned content wholesale.

Acceptance requires matching dark/light/narrow states for help, action menu, describe, abandon,
rebase, Run Options, and command preview. State assertions must distinguish inactive cursor from
active dialog focus, while pixels retain the real graph and a filled surface.

### UX-07 rebase-destination-discovery

**P2 · source confirmed · all native rebase forms.**

The chooser only searches revisions already loaded in the current log. A destination outside its
revset or limit cannot be named there; the flow can fail before opening with `No distinct visible
destination is available.` It accepts only one marked source. A user who discovers the missing
destination must cancel, widen the log, and reconstruct the action. Search Esc exits search before a
second Esc closes the chooser, but the compact footer does not explain that distinction.

Evidence: [rebase source resolution and candidates](../crates/jk/src/rebase.rs), lines 33–50,
277–285, 303–318, and 343–352.

**Proposal:** retain frozen source roles while allowing a destination revset or broader search with
explicit resolution. Distinguish no match, an invalid expression, and more than one result. Keep the
current small candidate list as a quick path. Decide multiple-source/destination support as an
explicit extension, rather than silently changing mark semantics.

Acceptance requires a destination hidden by revset, a destination beyond the log limit, no match,
ambiguous input, and cancellation during search. Failure must preserve the entered expression and
source selection. The active footer must state whether Esc leaves search or cancels the operation.

### UX-08 operation-error-visibility

**P2 · source confirmed; reproduction capture required · op-show, op-diff, history links.**

Opening an operation detail returns `AppTransition::Continue` on its error path. If the operation
lookup fails, the requested view does not open and the current view receives no explanation. Command
History links reuse this helper. A user cannot distinguish a failed load from a key that did
nothing.

Evidence: [open inspection](../crates/jk/src/refresh.rs), lines 186–195, and its operation-show/diff
call sites in [operation navigation](../crates/jk/src/main.rs), lines 2130–2151 and 2184–2204, and
[command history](../crates/jk/src/command_history.rs).

Acceptance requires a fixture command that fails operation lookup: retain the current selection,
show actionable failure text, preserve full stderr in history/details, and offer retry. A subsequent
successful retry must open the intended operation rather than changing selection silently.

Run Options exposes another path to this silent failure. After an explicit ignore-working-copy or
historical operation, the on-disk workspace can remain stale. Command History's operation-log
fallback uses the default query, which can fail on that stale state. Its error branch reports only
to an active Log view, so Command History stays open without a diagnostic. The failed options
recordings stalled at this navigation step. Preserve full failure text in the active view and allow
operation inspection without updating the stale working copy; see
[history fallback](../crates/jk/src/command_history.rs), lines 118–146.

### UX-09 command-result-refresh

**P2 · source confirmed; reproduction capture required · command-mode, external-command.**

Command mode deliberately uses `RefreshPlan::None`. A successful mutation such as `:describe -m
changed @`, followed by Back, returns to the old graph. Captured output offers edit/retry, while
refresh from command output reports a Command History refresh limitation. The manual's command
alternatives therefore lack the native flows' visible result verification.

Evidence: [command spec and output footer](../crates/jk/src/command_mode.rs), lines 10–18 and
250–251, and [command execution/back/refresh routing](../crates/jk/src/main.rs).

**Proposal:** returning from a command that may change the repository should refresh the relevant
view or mark it stale and provide one direct refresh action. Preserve command output and operation
links. External commands require an explicit conservative policy because their effects are not
inferable from the executable name.

Acceptance requires noninteractive describe, commit, duplicate, restore, and an external file edit.
After returning, descriptions, graph, status, and diff must match direct jj inspection; failures
that partially change state also require refresh. Read-only output must remain navigable.

### UX-10 workspace-retry-state

**P2 · runtime and source confirmed · workspace-add, workspace-rename.**

Workspace add/rename input is popped before execution. An existing destination or colliding name
therefore returns an error without the draft, forcing users to retype the full path/name. The
workspace confirmation also uses `y` to execute, whereas common command previews use `y` to copy. A
key learned as Copy can therefore execute a workspace mutation.

Evidence: [workspace mode handling](../crates/jk/src/main.rs), lines 517–525; [workspace failure
handling](../crates/jk/src/workspace_routes.rs), lines 91–110; and [workspace lifecycle
input](../crates/jk/src/workspace_lifecycle.rs).

Acceptance requires failed add and rename retaining editable input plus the diagnostic. Apply the
shared preview key contract or show a deliberate, consistent destructive-decision contract. Tests
must prove that a key used to copy in one preview cannot unexpectedly execute another mutation.

Recovering the full workspace error also takes extra navigation. Command History initially selects
the latest entry with an operation ID, which can be the earlier successful rename. After a failed
rename, `C`, Home, Enter reaches the newest failed command and its full diagnostic. Keep the failed
command selected when opening its details; see [history
selection](../crates/jk-tui/src/command_history_view.rs), lines 359–370.

A lower-priority validation detail appears in `manual-run-options-invalid-operation`: the same red
error is printed twice. Unlike workspace rename, this form retains its editable draft. Render one
field error and retain an extra summary only if it adds information; both copies come from [Run
Options rendering](../crates/jk/src/run_options.rs), lines 319–325.

### UX-11 bookmark-context-routing

**P2 · runtime and source confirmed · bookmark and remote workflows.**

Bookmarks maps to the Workspaces binding context, but its renderer has no Action Menu or Command
Discovery branch. `a` installs a workspace-context action mode without drawing its overlay. The
bookmark screen stays visible while that hidden mode intercepts keys. A workspace action selected
through that mode returns immediately because the route requires a Workspaces view. The second CI
run captures the unchanged bookmark screen after `a`. The native `?` help uses a separate route and
displays correctly; the first CI capture disproved the initial source-only hypothesis that help was
also hidden.

Evidence: [binding context](../crates/jk/src/main.rs), line 1655; [action routing and native-help
exception](../crates/jk/src/actions.rs), lines 54–76 and 156–157; [workspace action
guard](../crates/jk/src/workspace_routes.rs), lines 26–27; and [workspace
keymap](../crates/jk-tui/src/keymap.rs), and [bookmark mode
rendering](../crates/jk/src/rendering.rs), lines 107–166.

Acceptance requires opening Bookmarks, then `a`: the action menu must become visible. Every shown
action must apply to bookmark/remote work, route to an implemented action, and have accurate safety
cues. Preserve the working native bookmark help. Workspace actions and help must remain correct on
the actual Workspaces screen.

### UX-12 command-scope-visibility

**P1 · runtime and source confirmed · workspace inspection, `:`, `!`.**

After opening a sibling workspace's status, log, or diff, arbitrary commands still use the startup
repository or working directory. The command prompt shows the input but not that execution target.
For example, from a sibling Status view, `:describe -m changed @` targets the startup working copy.
Inspecting a workspace does not switch command scope. Without a label in the prompt, users must
remember that distinction before changing `@`.

Evidence: [startup and input dispatch](../crates/jk/src/main.rs), lines 209, 283, 531, and 537;
[command prompt rendering](../crates/jk/src/command_mode.rs), lines 33–52; and [scoped workspace
inspection](../crates/jk/src/workspace_routes.rs).

The corrected recording from CI run `35558741399` completes the operation and recovery. Seven
repository assertions prove that default receives the new description, scratch keeps its original
commit and description, and native Undo restores default's exact original commit and description.
All five PNG/State pairs and the sampled 31-second video passed review against the current tape.
The output clips the description at the right edge, which explains the earlier CI wait failure
(UX-15). The accepted CI artifact replaces the local reproduction as publication evidence.

**Proposal:** show the execution repository/workspace before any arbitrary command runs. Either
retain the startup target with an unmistakable label or provide an explicit scoped handoff. Do not
silently switch policy based on the view. Include the scope with captured command history/output.

Acceptance requires two disposable sibling workspaces with distinct `@` descriptions. From each
inspection view, run a noninteractive mutation and assert exactly which working copy changes. The
prompt, exact command, output, and history must agree. Test relative paths in `!` as well as `:`.

### UX-13 command-editing

**P3 · source confirmed · command-mode, external-command, edit/retry.**

Command input supports appending, Backspace, and Ctrl-u to clear the whole line. Left/Right,
Home/End, and normal word movement cannot move an editing cursor. Returning to edit a long failed
command therefore requires deleting its tail or retyping the entire line to repair an early operand.

Evidence: [jj/external command input handlers](../crates/jk/src/main.rs), lines 953–1077. The manual
history scenario uses Ctrl-u when changing a command, exposing the current editing cost.

Acceptance requires an editable cursor with character/word movement, deletion, Unicode input,
horizontal scrolling, and retry preserving the original command. Tests must change a middle operand
without changing the suffix, and verify the exact resulting argument vector. Reuse the established
text input model rather than teaching different editing rules in each form.

### UX-14 bookmark-result-feedback

**P3 · source confirmed; reproduction capture required · bookmark-create, bookmark-move.**

Successful bookmark mutation refreshes the list but does not display the prepared success message.
The refreshed view clears status unless the old selection disappears. For create or move, users must
infer success from a new row or changed target, which may be outside their current attention.

Evidence: [bookmark mutation completion](../crates/jk/src/mutations.rs), lines 106–127, and
[bookmark refresh](../crates/jk-tui/src/bookmark_view.rs), lines 212–233. This differs from the
factual completion status and recovery hints shown by several log mutations.

Acceptance requires a concise result identifying the bookmark and target, retaining useful
selection, and a direct Command History path. Verify creating into a long list, moving the current
bookmark, deletion, and refresh failure after a successful mutation. A failed refresh must not imply
the bookmark command itself failed.

### UX-15 command-output-visibility

**P2 · runtime and source confirmed · command mode and history details.**

Captured command output writes the full execution command on one line, including its repository path
before the command operands. The generic rendered view only scrolls vertically and does not wrap
that line. A long path can therefore hide the actual command and operands off the right edge. Long
stdout, stderr, and command-history detail lines have the same limitation. The recorded text is
intact, but the output view offers no way to scroll to the hidden operands.

Evidence: [captured command construction](../crates/jk/src/main.rs), line 1161; [output
metadata](../crates/jk/src/command_mode.rs), lines 230–236; and [rendered view
geometry](../crates/jk-tui/src/rendered_view.rs), lines 225–229. Command History can copy the
recorded command; editing the original command also retains its arguments.

Acceptance requires a long repository path and long quoted operands at 60 and 104 columns. Make the
whole execution command available on the output surface by wrapping metadata or scrolling it, while
preserving stdout/stderr/code whitespace. Provide horizontal navigation or another explicit
full-text inspection path for raw output. Copy must retain the exact arguments and exclude chrome.

### UX-16 inspection-back-navigation

**P2 · runtime and source confirmed · captured output, workspace and operation inspection.**

Escape dismisses prompts and previews but quits ordinary captured command output and rendered
workspace/operation inspection. The shared key mapping maps Escape to Quit, and these views
propagate it to application exit. Backspace is the actual one-level Back action. Recordings that
expected Escape to return stopped in the shell before their recovery steps. The user must reopen jk
and reconstruct the view stack after such an exit.

Evidence: [shared key mapping](../crates/jk/src/key.rs), lines 134–142; [rendered inspection
routing](../crates/jk/src/main.rs), lines 2253–2262 and 2322–2331; and [Back
implementation](../crates/jk/src/main.rs), lines 1874–1881.

**Proposal:** use one visible back/cancel contract across nested views. Keep an explicit Quit action
and distinguish it from Back in help and footers. Acceptance requires a failed command, output
inspection, return, edit/retry, and recovery without restarting the application. Exercise each
workspace/operation inspection and Command History details, preserving exact one-level navigation.

### UX-17 diff-format-navigation

**P2 · runtime and source confirmed · diff file/hunk navigation and folding.**

A real Git-format patch visibly contains files, but its file picker says `No files in this diff.`
The parser recognizes color-words file headers such as `Modified regular file …`, then looks for
`@@` hunk headers within those recognized sections. Git output has `@@` but no recognized file
headers; color-words output has recognized file headers but no `@@`. The hunk tests use a synthetic
combination that these real formats do not produce. The initial CI walkthrough claimed successful
hunk navigation/folding without a folded marker; it is rejected as tutorial evidence even though the
runner passed. Color-words file navigation/folding remains a usable partial recovery.

Evidence: [file and hunk parsing](../crates/jk-tui/src/diff_state.rs), lines 834–899;
`inspect-diff-navigate-files` and its paired State from CI run `35556462468`. The patched tutorial
must demonstrate only operations it visibly completes and keep this unsupported boundary explicit.

**Proposal:** construct navigation from actual format-aware file/hunk structure, or obtain semantic
anchors separately while preserving the configured patch. Until supported, disable or explain
inapplicable controls rather than claiming the populated patch has no files. Acceptance uses
unaltered real jj Git and color-words output with multiple files/hunks, renames, binary changes,
format switches, refresh, and visible folded/unfolded outcomes. Synthetic mixed headers cannot be
the only parser proof.

### UX-18 container-workspace-resolution

**P2 · runtime and source confirmed with jj 0.45.1 · container startup.**

Bare startup in a shared no-working-copy container fails to select the intended child workspace. The
tested jj emits `No working copy.` with a period; the resolver requires an exact line equal to `No
working copy`. The automatic resolution path is skipped. Launching `jk -R default` or entering the
intended child first restores working-copy context. The manual records this current boundary rather
than presenting a successful fallback that did not occur.

Evidence: [container resolver](../crates/jk/src/cli.rs), lines 175–194 and 237–250; the disposable
container reproduction under `startup-container`. The explicit repository option bypasses this
diagnostic check.

**Proposal:** prefer stable structured detection where available, or conservatively normalize the
specific supported diagnostic. Acceptance uses the actual current jj diagnostic and supported older
forms, verifies deterministic default/fallback child selection, keeps explicit `-R` authoritative,
and proves unrelated errors cannot silently select a different repository. A container-startup
mutation must assert the exact child working copy affected.

## Visual and text/style review matrix

The tables list the reviewed checkpoints. Other recordings remain pending until their exact images,
states, and videos are reviewed.

| Evidence pair                       | Size/appearance | Pixel observation                                                   | Text/style observation                                                          | Audit result                                           |
| ----------------------------------- | --------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------ |
| `design-proof-normal-preview`       | 104×41, dark    | Filled slate dialog; real graph above and left; clear teal controls | Explicit source/destination/whole-change/message text; jj-colored graph remains | Opaque/context contract passes; UX-04 and UX-06 remain |
| `design-proof-light-preview`        | 104×41, light   | Filled pale dialog separates from white canvas; controls readable   | Same role/command text; jj graph colors preserved                               | Light surface passes; UX-04 and UX-06 remain           |
| `form-style-narrow-rebase-selector` | 60×33, dark     | Filled dialog and visible graph; two cursor markers compete         | `[r/b/s]`, `[o/A/B]`, bare `Esc`; active accent `#65d4cf`, surface `#242c3a`    | UX-04 and UX-06 confirmed                              |

The initial local recordings use `inspection-reviewed-v2`, with jk 0.3.0, jj 0.45.1, Betamax 0.1.21,
and rustc
1.98.1 recorded correctly after the executable-shim provenance fix. The exact image/state hashes,
run paths, binary hash, and observations are in [the structured audit](manual-ux-audit.json).

| Evidence pair           | Pixel review                                                            | Text/style review                                                                        | Repository result          |
| ----------------------- | ----------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | -------------------------- |
| `inspect-orient-log`    | Readable graph, descriptions, bookmark context, and independent cursor  | 104×36; 14 styles retain jj identity/author/bookmark distinctions                        | Commit and graph unchanged |
| `inspect-orient-show`   | Reopened in Firefox; change identity, patch, and footer remain distinct | 11 styles; adjacent deleted `60` and added `120` have separate red/green underline spans | Commit and graph unchanged |
| `inspect-orient-status` | Working-copy, parent, and complete changed-file list are visible        | 8 styles; explicit working-copy and parent identities                                    | Commit and graph unchanged |

The CI review of run `35556462468` adds exact Linux artifacts for squash, restore, all three abandon
paths, operations, range diff, and workspace lifecycle. The structured audit records each inspected
PNG and State hash with individual observations. Squash and restore prove cancellation, exact
content movement/replacement, and Undo; abandon proves replacement, descendant rebasing, failed
emptiness probing, and cancellation. Workspace collision confirms UX-10's discarded draft and
incomplete diagnostic. Operation output confirms UX-15's right-edge clipping.

The dark and automatic theme captures have opaque surfaces, visible real repository content, and
readable controls. The first light capture has readable application controls but black video-caption
text against navy. Its replacement in CI run `35557653700` uses black captions on a light margin;
all three image/State pairs and the 11.8-second sampled video passed review. The replacement
log-template video also passed after captions were matched to the intermediate layouts. The first
diff-navigation video remains rejected because its narration claims actions the product did not
perform (UX-17). The second-run replacement correctly demonstrates color-words file navigation and
folding; it passed review without claiming working hunk navigation. Elision checkpoints from the
second run are diagnostic evidence only: Escape exits the application before the final return,
leaving the video incomplete (UX-16). The third-run replacement uses Backspace and passed review
of all three checkpoints and its 18.5-second sampled video.

Reviewers inspect unique PNG files inline alongside the matching terminal text and styles. Video
reviews compare duration, tape timing, and sampled beginning/intermediate/final frames; the review
records this sampling method rather than claiming continuous real-time playback. Publication
acceptance is separately bound to the exact current tape, images, states, video, and metadata in
`manual-evidence-index.json`. Changed media requires a new review, including recordings whose
automation passed.

## Proposed decision order

1. Repair scope labels, wrong-context routing, hidden operation errors, and lost retry input. These
   have bounded behavior and concrete acceptance tests.
1. Define a shared command completion lifecycle: visible running state, cancellation/tool return,
   captured diagnostics, result refresh, and operation recovery. This supports command alternatives
   without requiring users to rebuild context.
1. Review operand identity and narrow role selection together. Preserve jj meaning and exact commands
   while making the source/destination decision readable without memorizing hashes.
1. Add foreground tool handoff, partial-content operations, and publication as separately reviewable
   workflow designs. Their acceptance criteria include repository outcomes, not only new controls.

Keep the manual's native forms, command alternatives, and unsupported operations accurate as these
changes are implemented.
