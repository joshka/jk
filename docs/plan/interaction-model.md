# Interaction Model

This document defines cross-screen interaction rules for `jk`. A TUI succeeds or fails on whether
its shortcuts, action safety, and view transitions feel predictable across screens.

## Common Manual Workflows

Two high-frequency manual workflows should shape the first native interactions:

- `jj git fetch`: update repo state from remotes, then refresh the current view.
- `jj new trunk`: start a new change from trunk, then keep working from the refreshed log/status
  context.

These are common enough to deserve low-friction flows. They should not be buried behind a generic
command launcher.

## Shared Shortcut Vocabulary

Shortcut behavior should be consistent across screens unless a screen has a clear reason to differ.

Core navigation:

- `j`/`k`, arrows: move selection or scroll.
- `g`/`G`: first/last item or top/bottom.
- `h` or `Esc`: go back or dismiss a transient surface.
- `Enter` or `l`: open the selected item when opening is the obvious action.

Core utility:

- `/`, `n`, `N`: search, next match, previous match.
- `r`: refresh current screen.
- `y`: copy menu for the current selection/context.
- `?`: help/keymap.

Inspection transitions:

- `s`: show-like detail when the selected item has a revision target.
- `d`: diff-like detail when the selected item has a revision target.
- `[`/`]`: previous/next file, section, or comparable local unit.

Action keys should reuse meanings across screens. For example, a key used for delete/abandon-like
behavior in one utility screen should not mean a harmless inspect action elsewhere.

## Mutation Safety Tiers

Not every mutation needs the same ceremony. Choose the interaction shape by risk, reversibility, and
how surprising the action is.

### Tier 0: Refresh-Like Or Harmless

These can be direct single-key actions with status feedback.

Examples:

- refresh current view;
- open a view;
- copy data;
- `jj git fetch`, once the remote and command shape are clear.

Fetch is technically state-changing, but it is usually safe, expected, and easy to repeat. It should
still show command output or failure clearly.

### Tier 1: Easy To Undo And Common

These can be direct or near-direct actions when the target is obvious, with post-action feedback and
an easy path to undo.

Examples:

- `jj new trunk`;
- `jj new` from a selected revision when the target is visible and exact;
- `jj edit` selected revision, if the target is explicit.

`jj new trunk` should be low-friction because it is a common starting point for OSS work. It should
run from an exact trunk target, refresh the log, and make the resulting working-copy change obvious.
If the user picked the wrong moment, normal `jj undo` should be the recovery path.

### Tier 2: Contextual But Reviewable

These should use a prompt, lightweight preview, or two-step flow.

Examples:

- `describe`;
- `commit`;
- bookmark set/move/create;
- file track/untrack/chmod;
- `git push` with a clear destination and preview.

The user should see what target is being acted on and what command shape will run.

### Tier 3: Risky Or Structurally Surprising

These require confirmation and, when possible, a preview.

Examples:

- `rebase`;
- `squash`;
- `split`;
- `abandon`;
- `operation restore`;
- `operation revert`;
- bookmark delete/forget;
- workspace forget.

These actions can change graph shape, discard visible state, or confuse recovery. They should never
infer their target from rendered text alone.

## Revset And View Modes

The default log view should stay close to built-in `jj` behavior: the current useful work from trunk
or main, using the user's configured default view where possible.

`jk` should also support broader view modes for orientation:

- Default work view: close to the built-in `jj` view; the normal home screen.
- Trunk work view: focused view around work from `trunk` or the configured main branch.
- Recent work view: changes worked on recently, useful across many OSS repositories.
- All/repo overview: broader history view for orientation and debugging.
- Custom revset view: user-provided revset, preserved in history where practical.

Switching view modes should preserve selection by semantic identity when possible. View modes should
be visible in the title/status chrome so the user knows whether they are looking at narrow current
work or a broader repository picture.

## Action Selection

Actions should carry semantic targets through the flow:

- selected revision ids for graph actions;
- selected paths for file actions;
- selected operation ids for recovery actions;
- selected bookmark/tag/workspace names for ref and workspace actions.

Multi-selection should preserve the order and relationships that matter to the command. For example,
a rebase flow may need source revisions and a destination revision; those roles should be explicit
rather than inferred from selection order if ambiguity would be risky.

## Feedback After Actions

After any action, `jk` should:

- show success or failure without hiding command output that matters;
- refresh the relevant screen;
- preserve selection or move to the newly relevant item where possible;
- provide a visible undo path for actions where `jj undo` is the expected recovery.

For common direct actions like `jj new trunk`, the post-action refresh is part of the interaction.
The user should immediately see the new working-copy change and know that undo is available.
