# Agent Workflow Guidance

Load this document when planning non-trivial changes, reviewing code, preparing handoff notes, or
deciding how much validation is needed.

This is the canonical active guidance for implementation workflow, maintainability packet shape, and
completion criteria. Use [`architecture.md`](architecture.md) for the ownership map, and use
[`../reference/README.md`](../reference/README.md) for the current product-facing reference surface.

## Current Maintainability Doctrine

Future `jk` work should follow these rules by default:

- prefer feature roots over kind-of-code buckets;
- treat shared modules as boring mechanics after the feature has made the product decision;
- keep shared action families after target selection, and keep availability with the feature;
- move active guidance with structure, while letting historical or external records preserve old
  paths;
- record coherent no-move decisions when a shared root could otherwise look unreviewed;
- treat the runtime-path cleanup as done, and choose the next maintainability packets from measured
  reader pain in the current source tree instead of from old packet trackers;
- require both behavior-preservation proof and durable ownership memory before claiming a
  maintainability wave is complete.

## Guidance Hierarchy

For future `jk` work, use the active guidance in this order:

1. `AGENTS.md` for repo entry rules and load order.
1. `architecture.md` for current structure and ownership.
1. `rust-style.md` for Rust/module-shape decisions once the owner is known.
1. `workflow.md` for packet shape, validation posture, and completion criteria.
1. `../reference/README.md`, `../reference/screens.md`, `../reference/workflows.md`, and
   `../reference/view-model.md` for the durable current product reference surface.

Treat packet logs, phase trackers, and sibling planning-repo artifacts as history or working
material, not as the current ownership map.

## Start With The Existing Shape

Read the owning module before editing. Identify the concept that owns the change, the adjacent
modules it depends on, and the test style already used for that behavior.

Prefer the existing style when it is coherent. Improve local structure when the existing shape is
actively making the change harder to understand, but keep that improvement scoped to the same
concept.

## Scope Control

Keep changes atomic:

- one feature, bug fix, or documentation purpose per change;
- no unrelated refactors;
- no dependency churn unless it is necessary for the task;
- no broad module moves unless the reader path genuinely improves;
- no speculative public API for future features.

If a change reveals a separate cleanup, note it or make a separate `jj` change rather than folding
it into unrelated work.

## Maintainability Packets

For maintainability work, optimize for correct future change rather than for superficial tidiness.

- Start with the runtime path when the reader pain is in startup, dispatch, modal flow, action flow,
  or view routing.
- After the runtime path is healthy, switch to a measured reader-pain queue for dense feature or
  boundary owners instead of continuing to split by habit.
- Stop splitting when a module reads as one coherent owner. Record the no-move decision when the
  module could otherwise look like an unreviewed dumping ground.
- When structure moves, update active guidance in the same wave. Historical ledgers may preserve old
  paths elsewhere; active guidance in this repo may not.
- Treat maintainability completion as two claims: changed surfaces still behave correctly, and the
  repo now has durable ownership memory for what changed, what stayed, and what remains.

## Review Posture

Review from the perspective of a future maintainer who did not write the code. Prioritize:

- correctness and edge cases;
- jj CLI compatibility;
- public or crate-local API clarity;
- documentation truthfulness;
- rendering and terminal-state behavior;
- focused tests for the contract.

When code is hard to read, identify the reason precisely: concept mixing, poor ordering, hidden
state, vague names, too much abstraction, or missing tests.

## Implementation Checklist

Before editing:

- identify the owning module;
- identify whether rendered jj output should remain the presentation source;
- identify whether the change introduces a soft agreement that should be called out in the owning
  docs, tests, or source comments;
- check existing tests for the behavior;
- decide the narrowest useful validation command.

While editing:

- keep side effects visible;
- preserve current user behavior unless intentionally changing it;
- add or update tests alongside behavior;
- avoid widening visibility to work around module shape.

Before handoff:

- run focused tests for the touched behavior;
- for Rust changes, run `cargo clippy -- -D warnings` or the repository's documented equivalent such
  as `just check`, and list the exact blockers if clippy is not clean;
- run a `cargo run` smoke when it is practical for the change, and state whether that smoke was
  warning-free or why it was skipped;
- run `just check` when practical;
- if the local `just check` wrapper is broken, report the direct commands you ran instead;
- run Panache format and lint checks for Markdown-only or doc-heavy changes;
- report any validation gap directly.

## Handoff Notes

Final notes should be concise and concrete:

- what changed;
- where the important files are;
- what validation ran;
- what did not run, if anything;
- any residual risk or follow-up that matters.

Do not include long implementation diaries. The user shares the workspace and can inspect files
directly.

## Jujutsu Workflow

This repo uses `jj`. Prefer `jj --no-pager` for inspection and keep source of truth in jj, not Git.

For separable work, start a new working-copy change before editing:

```sh
jj --no-pager new
jj --no-pager desc --message "Document agent coding standards

Add focused guidance for agentic tooling so future changes follow the
project architecture, Rust style, documentation posture, testing habits,
and review workflow expected for jk."
```

Use imperative, concise descriptions. Keep unrelated changes separate, and do not rewrite user work.
Keep the title line at 50 characters or fewer, leave a blank line before the body, and wrap the body
at 72 columns.
