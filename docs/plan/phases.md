# Phases

This document defines implementation cut lines. The point is to avoid the old prototype failure mode
where many useful ideas arrived without a durable order.

## Phase 0: Planning Baseline

Goal: make implementation order explicit before broadening command coverage.

Deliverables:

- command inventory
- screen specs
- workflow matrix
- phase plan
- open questions log

Exit criteria:

- every major `jj` command family has a first-pass classification
- core screens are defined
- “screen vs flow vs passthrough” is answered for most high-frequency tasks

## Phase 1: Core Navigation

Goal: make the log -> inspect -> return loop excellent.

Scope:

- log
- show
- diff
- back/forward history
- search
- copy actions
- refresh-in-place
- sticky file context
- compact help/keymap

Exit criteria:

- graph selection is stable and item-based
- show/diff drill-down is cheap and predictable
- refresh and search do not feel bolted on
- help explains the app without requiring the CLI

## Phase 2: Inspection Breadth

Goal: add the highest-value read surfaces around the core loop.

Scope:

- status
- operation log
- bookmark list
- file list/show
- resolve list
- tag list
- workspace root/list

Exit criteria:

- common read tasks stay inside `jk`
- utility screens feel like siblings of the core screens, not special cases

## Phase 3: Safe Mutation Flows

Goal: add the most important write actions through guided flows.

Scope:

- `new`
- `describe`
- `commit`
- `edit`
- `next`
- `prev`
- `rebase`
- `squash`
- `split`
- `abandon`
- `undo`
- `redo`
- `git fetch`
- `git push`
- bookmark set-related actions

Exit criteria:

- high-value mutations have previews and confirmations where appropriate
- the graph and op-log remain the mental anchors for mutation outcomes
- risky actions feel safer than raw shell use

## Phase 4: Utility And Long-Tail Coverage

Goal: decide what deserves promotion from passthrough to native support.

Scope:

- file annotate/search refinements
- additional bookmark/tag/workspace actions
- selected advanced rewrite flows
- selected passthrough commands if real usage proves value

Exit criteria:

- native support is added because it improves workflows, not because a CLI subcommand exists
- promotions include an explicit integration choice: rendered output, narrow parser, structured
  output, `jj_cli`, `jj_lib`, future RPC, or upstream extraction

## Explicit Non-Goal

Do not aim for “every `jj` command gets a dedicated screen and shortcut” as a phase goal. That is a
useful coverage test, but it is not a healthy product cutline for `jk`.
