# Workspace integration

The `integration-all` jj workspace combines the unfinished workflows reviewed on 2026-09-20. The
source workspaces and recovery bookmarks remain available. The local review bookmark is
`joshka/workflow-coherence`; publication and merge status are tracked on its GitHub pull request.

## Product direction

The [TUI design guide](tui-design.md) records the direction established in
[Design a coherent TUI UI strategy](codex://threads/01a0b680-89a8-7ee2-8840-db4d9533e8d9)
and refined during this integration:

- Make jk feel like the interactive jj CLI, including configured graph, templates, and colors.
- Build visual hierarchy from alignment, spacing, clear labels, and restrained color regions.
- Keep normal views borderless and preserve one dominant task rather than competing panels.
- Give dialogs a distinct opaque fill and retain real repository content around them in screenshots.
- Use the original menu's slate, teal, blue-gray, and burgundy roles across dialog controls.
- Distinguish cursor position, marks, working copy, focus, and mutation operands.
- Show the objects, consequences, exact command, and controls before executing a mutation.
- Cover ordinary jj workflows coherently; document partial forms and advanced-command escape paths.

[Command coverage](command-coverage.md) lists implemented workflows and remaining forms. The
[screen atlas](design/README.md) contains proposals, while
[community reports](plans/0022-community-workflow-signals.md) supply additional validation cases.

## Recovered work

| Source workspace             | Recovered input | Treatment                                                       |
| ---------------------------- | --------------- | --------------------------------------------------------------- |
| `work/workflows-integration` | `330a0b57`      | Rebase, squash, restore, and workspace lifecycle foundation     |
| `tui-design-guidelines`      | `ac1db262`      | Integrate guide and distinguish atlas proposals from behavior   |
| `work/refs-remotes`          | `843810c7`      | Review bookmarks, explicit fetch, and push dry-run              |
| `work/external-command-mode` | `f82c856e`      | Integrate shell-free captured external commands                 |
| `work/cancellable-refresh`   | `ceae5e9c`      | Integrate cancellable log refresh and stale-result protection   |
| `work/shared-selectors`      | `d890ed38`      | Review operand identity and adopt existing selector consumers   |
| `work/run-options`           | Empty change    | Add per-command working-copy, operation, and immutable policies |
| `work/discord-requirements`  | Empty change    | Record public-archive workflow reports and validation cases     |

The action-menu, contextual help, layout-guideline, Betamax PR preview, graph-prefix, and release
workspaces contain earlier work already represented on main. The old `vibe`, prototype, and recovery
branches remain historical references. The recovered workflow integration supersedes the conflicted
rebase/action-menu variants.

## Repository and visual checks

Changes to the shared jj graph run sequentially. Sibling jj workspaces preserve the source changes;
the integration joins their histories and resolves shared dispatch, rendering, history, and refresh
behavior at the combined tip.

Behavior tests and recordings run against fresh disposable jj repositories with isolated config
and identity. The source workspace supplies the binary; it is never the test repository. Local
file-backed remotes cover fetch/push behavior without using the project's network remotes.

Every meaningful UI checkpoint pairs a PNG with Betamax terminal-state JSON after a semantic wait.
Inspect pixels for hierarchy and spacing, and `viewport_text` and styles for content and color
fidelity. Narrow layouts, cancellation, failures, and context-preserving returns are part of the
review. Generated media stays in ignored output directories or the separate media repositories.

## Verified behavior

The combined source passes 675 Rust tests, strict all-target Clippy, and nightly formatting.
Fresh-repository refs regressions verify literal wildcard names, deleted tracked bookmarks, and
modify/delete conflicts using actual jj output. Process tests cover cancellation, helper cleanup,
nonzero exits, and concurrent pipe draining. Layout tests cover preserved foregrounds, constrained
viewports, distinct marks, and confirmation controls.

Review fixes include exact new-parent operands, nullable/conflicted bookmark target parsing,
fetch operation links, preserved input and picker selection, cancellation before mutations,
terminal-default canvas styling, coordinated opaque dialogs, and child-process output width matched
to the content viewport. Run Options applies to one pending local command and requires the updated
preview to be confirmed.

Normal log loads and explicit refreshes follow jj's working-copy snapshot behavior. Only the
immediate refresh after an ignore-working-copy or historical mutation suppresses snapshots, so that
refresh cannot undo the command's chosen policy. Cancellation remains observable while helper
processes hold output pipes after their parent exits. Confirmation controls ignore key releases.

## Release compatibility

The new variants in the public exhaustive `jk_tui::command_discovery::CommandFamily` enum break
downstream exhaustive matches. The action-menu enums also gain variants relative to main. The
workspace is prepared for 0.3.0, with matching internal dependency requirements. The release
workflow must refresh its pending patch-release proposal to use this minor version. Other reviewed
public constructors, accessors, and modules are additive.

Release notes should mention that normal log refresh now snapshots working-copy edits like jj,
and captured commands receive null stdin. Interactive editor/merge-tool handoff remains planned.

Update the companion website's rebase, squash, restore, refs/remotes, and media claims with the
release that lands these changes. It currently describes released features.

## Remaining presentation limits

The background graph keeps its cursor marker while a modal owns input. The modal hides the
background hotbar; a separate unfocused graph treatment remains to be designed.

Abandon's embedded patch preview still uses a fixed Git-format diff with local addition/deletion
colors. It does not yet inherit configured jj diff formatting and colors. The normal diff view
preserves jj rendering. A return path from that view to abandon confirmation would remove this
separate patch renderer.

## Betamax evidence

Fifteen Betamax workflow and layout scenarios pass on fresh fixtures. Two targeted recordings verify
the final keyboard-hint styles in normal and narrow forms, bringing the total to 17 recordings and
91 paired PNG/terminal-state checkpoints. The index distinguishes the behavioral baseline from the
later changes to form-key colors. The layout matrix covers dark and light terminals, 80x24, and
60x16. Dialogs retain an opaque fill and visible repository context; compact previews keep their
controls visible while the complete command scrolls.

Repository assertions verify rebase cancellation and the confirmed destination, squashed source
content, restored file contents, and files retained after forgetting a workspace. Other scenarios
cover local remote fetch, captured command failures, responsive refresh cancellation, retained
failure context, selector handoffs, Run Options draft/apply behavior, and abandon confirmations.

Local review artifacts are indexed in `target/dogfood-artifacts/coherence-proof-index.md`. The index
links each PNG to its terminal-state JSON, executed tape, binary hash, and recording. Synthetic
screens in the design atlas are excluded from this implementation evidence.
