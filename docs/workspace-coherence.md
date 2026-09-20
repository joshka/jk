# Workspace coherence pass

This pass consolidates unfinished work as of 2026-09-20. The source workspaces and recovery
bookmarks remain available. The integration is review work, not a claim that every planned command
has shipped.

## Product direction

The [TUI design guide](tui-design.md) is the visual and interaction contract. The
[screen atlas](design/README.md) illustrates a proposed destination; it is not feature coverage.
The [command coverage audit](command-coverage.md) separates native workflows from partial and
deferred forms. [Community signals](plans/0022-community-workflow-signals.md) contribute dated
observations and acceptance criteria without replacing the maintainer's direction.
The user confirmed this direction in
[Design a coherent TUI UI strategy](codex://threads/01a0b680-89a8-7ee2-8840-db4d9533e8d9):

- Make jk feel like the interactive jj CLI, including configured graph, templates, and colors.
- Build visual hierarchy from alignment, spacing, clear labels, and restrained color regions.
- Keep normal views borderless and preserve one dominant task rather than competing panels.
- Distinguish cursor position, marks, working copy, focus, and mutation operands.
- Show the objects, consequences, exact command, and controls before executing a mutation.
- Cover ordinary jj workflows coherently; document partial forms and advanced-command escape paths.

## Recovered work

| Source workspace             | Recovered input | Treatment                                                     |
| ---------------------------- | --------------- | ------------------------------------------------------------- |
| `work/workflows-integration` | `330a0b57`      | Rebase, squash, restore, and workspace lifecycle foundation   |
| `tui-design-guidelines`      | `ac1db262`      | Integrate guide and distinguish atlas proposals from behavior |
| `work/refs-remotes`          | `843810c7`      | Review bookmarks, explicit fetch, and push dry-run            |
| `work/external-command-mode` | `f82c856e`      | Integrate shell-free captured external commands               |
| `work/cancellable-refresh`   | `ceae5e9c`      | Integrate cancellable log refresh and stale-result protection |
| `work/shared-selectors`      | `d890ed38`      | Review operand identity and adopt existing selector consumers |
| `work/run-options`           | Empty change    | Complete a bounded command-scoped Run Options slice           |
| `work/discord-requirements`  | Empty change    | Complete the public-archive requirements note                 |

The action-menu, contextual help, layout-guideline, Betamax PR preview, graph-prefix, and release
workspaces contain earlier work already represented on main. The old `vibe`, prototype, and recovery
branches are historical reference, not competing implementations to merge wholesale. In particular,
the conflicted rebase/action-menu variants are superseded by the recovered workflow integration.

## Integration discipline

All changes to the shared jj graph are coordinated sequentially. Review work uses sibling jj
workspaces; it does not rewrite source changes or insert new revisions before unrelated workspaces.
A multi-parent integration retains source provenance while resolving shared command dispatch,
rendering, history, and refresh seams together.

Behavior tests and recordings run against fresh disposable jj repositories with isolated config
and identity. The source workspace supplies the binary; it is never the test repository. Local
file-backed remotes cover fetch/push behavior without using the project's network remotes.

Every meaningful UI checkpoint pairs a PNG with Betamax terminal-state JSON after a semantic wait.
Inspect pixels for hierarchy and spacing, and `viewport_text` and styles for content and color
fidelity. Narrow layouts, cancellation, failures, and context-preserving returns are part of the
review. Generated media stays in ignored output directories or the separate media repositories.

## Completion criteria

1. Resolve all shared dispatch and model conflicts without dropping any completed workflow.
1. Apply the design contract to shared selection, chrome, selectors, and confirmation surfaces.
1. Fix source-identity, cancellation, history, and operation-link defects found during review.
1. Validate each integrated command family on fresh fixtures and inspect text plus image evidence.
1. Align README, crate README, usage, roadmap, and command coverage with actual behavior.
1. Record the review boundary, validation results, and remaining limits before publication.

The companion website currently describes the released surface. Its rebase, squash, restore,
refs/remotes, and media claims need updating with the release that lands these changes. A local
integration build is not sufficient evidence to advertise them as released.
