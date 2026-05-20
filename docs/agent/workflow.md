# Agent Workflow Guidance

Load this document when planning non-trivial changes, reviewing code, preparing handoff notes, or
deciding how much validation is needed.

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
- identify whether the change introduces a soft agreement that belongs in the fragility register;
- check existing tests for the behavior;
- decide the narrowest useful validation command.

While editing:

- keep side effects visible;
- preserve current user behavior unless intentionally changing it;
- add or update tests alongside behavior;
- avoid widening visibility to work around module shape.

Before handoff:

- run focused tests for the touched behavior;
- for Rust changes, run `cargo clippy -- -D warnings` or the repository's documented equivalent, and
  list the exact blockers if clippy is not clean;
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
