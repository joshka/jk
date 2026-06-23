# Dogfood Backlog Order

This note records the current `vibe` spike ordering for dogfoodable progress. It is intentionally
more execution-oriented than the product plan: finish the tight recovery and command surfaces first,
then broaden into mutation flows, selectors, run options, and generated help.

## Next Dogfood Sprint

1. Done: finish mutation-to-operation linking. Confirmed mutations in Command History now focus the
   newest recorded operation row when Command History opens, so `o` opens the exact resulting
   `jj op show` instead of landing on a later refresh row and falling back to `jj op log`.
1. Done: add `u` undo preview. Pressing `u` shows `jj undo` in command preview, runs only on
   confirm, refreshes, records history, and exposes the recovery footer after success.
1. Done: add `U` redo preview. Pressing `U` shows `jj redo` in the same confirmation flow, so undo
   and redo use one recovery interaction model.
1. Done: add post-mutation recovery footer. After a mutation succeeds, the log footer makes
   `u undo`, `U redo`, `o operation`, and `C history` visible.
1. Done: add copy command from preview/history. Users can copy the exact redacted `jj ...` command
   that will run from preview or that already ran from Command History.
1. Done: add Command History output/details view. Enter on a history row shows full argv,
   stdout/stderr summary, status, duration, and operation id.
1. Done: add the `:` command mode MVP. It runs jj commands from an overlay, captures output in a
   rendered view, records history, and preserves failed-command stderr.
1. Partly done: add command-mode follow-ups. Command output can now reopen the `:` prompt with the
   previous command for edit/retry; success refresh policy, richer prompt editing, and history recall
   remain follow-up work.
1. Add `!` external command MVP: run non-shell external commands safely, with explicit argv handling
   and output capture.
1. Done: add diff View Options quick wins. `V` from diff can reload the active diff as patch,
   summary, stat, types, name-only, git, or color-words output, matching the supported local
   `jj diff` flags.
1. Partly done: make rich diff navigation solid. Diff already supports `[ ]` file movement, `{ }`
   hunk movement, search, horizontal scroll, sticky headers, a current-file status line, and a file
   list jump overlay; remaining polish includes broader view options and any dogfood issues found
   in multi-file diffs.
1. Done: add diff file list overlay. Pressing `f` from a diff shows files in the current diff,
   preserves the active file as the selected row, and jumps to the chosen file on Enter.

## First Useful Mutation Sprint

1. Done: add `jj abandon` preview flow. Pressing `a` on the selected log revision opens a
   destructive command preview for `jj abandon REV`; Enter runs it, refreshes the log, records
   history with the resulting operation id, and exposes recovery actions.
1. Add `jj new` preview flow: create a new change from selected context with clear preview and
   recovery.
1. Add `jj edit` preview flow: move working copy to selected change, with warning when the action
   may surprise.
1. Add inline describe polish: multiline/editor describe, better prompt editing, and clearer
   before/after description display.
1. Add minimal rebase preview: resolve source/destination from marks plus cursor and show exact
   `jj rebase` command before running.
1. Add rebase destination search: search/filter the log while choosing a destination.
1. Add rebase role resolver UI: make source, branch, destination, insert-before, and insert-after
   roles explicit.
1. Add `jj squash` preview flow: use selected/marked revisions and show the exact squash command.
1. Add `jj split` entry point: likely external/editor-backed first, with preview and recovery rather
   than native hunk UI.
1. Add `jj restore` preview flow: start with file-level restore before hunk-level restore.

## Foundation Catch-Up

1. Add shared revision selector: reusable selector output for selected revision, marked revisions,
   revsets, and role picking.
1. Add shared fileset selector: reusable file/fileset picker for diff, status, restore, split, and
   squash workflows.
1. Add shared operation selector: reuse operation choices for op show/diff, undo context, restore,
   revert, and history links.
1. Add Run Options drawer MVP: expose repository, working-copy policy, operation context, operation
   integration, immutable override, and config overlays.
1. Add generated jj help manifest: ingest `jj help` / `jj util markdown-help` so supported flags and
   command families stay grounded.
1. Add cancellable command runner: slow diff/log/preview work should not freeze navigation or apply
   stale results.
1. Define refresh policy in code: preserve selection, scroll, marks, expansion, and diff state
   consistently across refreshes.
1. Add auto-refresh opt-in: only after manual refresh preservation is reliable.

## Later Broadening

1. Expand workspace actions: add workspace add, forget, rename if supported, and better stale/update
   flows.
1. Add workspace-scoped log/status/diff: make selected workspace context explicit and reusable.
1. Add bookmark list screen: list bookmarks, show target commits, and expose safe actions.
1. Add bookmark create/move/delete previews: command-preview first, no blind mutation.
1. Add `git fetch` flow: first read/network-safe fetch path with visible output and history.
1. Add push dry-run flow: `git push --dry-run` first, then confirmed push only after preview.
1. Add tag/remotes screens: after bookmarks/fetch/push establish the refs/remotes pattern.
1. Add op restore preview: strong confirmation, clear warning, and recovery docs.
1. Add op revert preview: safer-than-restore messaging, still treated as a mutation.
1. Add command rerun from history: probably after command details/copy are stable.
1. Add persistent command history: durable local history once in-memory history semantics are good.
1. Add Betamax validation taxonomy: organize tapes by global options, selectors, view options, run
   options, workspaces, recovery, and mutations.
1. Add user docs for dogfood flows: local docs for command mode, safe mutation loop, operation
   recovery, diff navigation, and workspaces.
1. Add config-fidelity fixtures: aliases, templates, colors, graph styles, and wrapping.
1. Add hybrid graph previews: only after config-fidelity and rebase preview behavior are well
   tested.

## Steering Notes

- Treat items 1-12 as the next dogfood sprint.
- Treat items 13-22 as the first useful mutation sprint.
- Treat items 23-30 as the point where roadmap foundation needs to catch up before broadening
  further.
- For the near-term `vibe` spike, favor direct implementation in the orchestration thread for small
  app wiring, crash fixes, and live `cargo run` feedback.
- Use subagents only for independent, file-scoped chunks large enough to amortize context setup.
- Reserve Betamax GIF work for milestone user-visible flows rather than every incremental patch.
