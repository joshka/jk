# Open Questions

This document tracks unresolved planning decisions. Add questions here instead of letting them turn
into accidental implementation choices.

## Product Model

- How broad should command-mode support be before Phase 3 mutation flows land?
- Should `jk` expose a single compact help surface, or separate help and keymap screens?
- Should back/forward history remain purely screen-based, or should some flows preserve in-screen
  selection history too?
- What evidence would justify moving a feature from rendered-output parsing to `jj_cli`, `jj_lib`,
  structured output, RPC support, or an upstream API request?

## Screen Ownership

- Does `status` need a true selection model, or is it better as a scrollable inspection surface with
  targeted actions?
- Should `file list` open into `file show`, `show`, or `diff` by default?
- Does `bookmark list` need an always-visible distinction between local, tracking, and remote
  bookmarks, or can that remain rendered-`jj` output first?

## Mutation Design

- Which mutation flows deserve direct shortcuts in addition to command-mode or prompt entry?
- Which risky flows need previews, and what is the minimum viable preview for each?
- Should `next` and `prev` remain direct actions, guided flows, or both?

## Scope Control

- Which `jj` commands should remain passthrough unless strong evidence changes that?
- Is `evolog` important enough to become a near-term utility screen, or should it wait until after
  status/op-log/bookmarks/files?
- Is there any acceptable future for multi-pane presentation, or should all such ideas be treated as
  out of scope unless the single-view model clearly fails?
- Which parser assumptions are acceptable long term, and which should be treated as temporary
  evidence for stronger integration?

## Integration

- Which current output parsers should be converted to purpose-built templates or structured data
  first?
- Which planned mutation flows require single-operation transactions that are difficult to model
  through subprocess calls?
- Where would using `jj_cli` or `jj_lib` create a better failure mode than parsing rendered output?
- Which duplicated behaviors would be useful enough to extract upstream for other jj UI tools?

## Issue Promotion

Use these rules before creating GitHub issues from this plan:

1. The screen or workflow owner is already defined.
1. The phase placement is clear.
1. The acceptance criteria can be written without inventing new product scope.
1. The issue is implementation-shaped, not brainstorming-shaped.
