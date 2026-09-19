# jk screen atlas

A design study of jk as a complete, jj-native terminal interface. This is a proposed destination,
not implemented application behavior. The gallery uses synthetic repository data and never runs a
command. Its screen selector and journey controls belong to the review gallery, not to the TUI.

The [design guidelines](../tui-design.md) define the contract. This atlas makes that contract
visible
across inspection, history editing, content movement, recovery, sharing, workspaces, and
maintenance.

## Fixed-cell terminal images

The terminal image version is the visual authority for this design. It uses one font size and
integer cell coordinates throughout, with flat colored surfaces and no decorative borders.
`terminal_mockups.py` emits ANSI truecolor cells into a real pseudo-terminal; Betamax renders those
cells into PNGs. The 1200x768 capture settings provide a 92-column, 38-row terminal.

The renderer uses the same 165 screen fixtures as the browser study, including a borderless file
picker, contextual help, and mutation previews. The abandon confirmation follows the user-supplied
terminal reference: affected files and counts, explicit consequences, a visible command, and colored
View diff / Cancel / Abandon action regions. Cancel starts focused. Every capture is a real terminal
rendering of a
mockup, not a screenshot of a working jk feature. Terminal capabilities are real; repository data
and workflows remain illustrative.

Generate every screen into an external image directory:

```sh
python3 docs/design/capture_terminal.py /path/outside/this/repository/terminal-mockups --run
```

Use `--theme light` for the alternate palette. The output includes all PNGs, an image-only gallery,
a reproducible Betamax tape, terminal dimensions, and selected terminal-state JSON captures. The
image gallery starts with the diff file picker. Its controls are only for browsing image files.

## Review the design

The self-contained [atlas fragment](atlas.html) contains the proposed screens. The inline Codex
version also includes an existing-capture toggle on the graph, diff, and graph-help screens.
Baseline PNGs remain in the separate screenshot repository, not embedded in this repository.

Start with these journeys:

1. Inspect a change: graph, inline details, show, diff, files, search, folding, help, and view
options.
1. Rebase a stack: action menu, source semantics, destination, preview, options, execution,
recovery.
1. Split and squash: filesets, selected/remaining content, command review, and external hunk
editing.
1. Share work: bookmarks, move, fetch, push dry-run, success, and failure.
1. Recover a mistake: command history, operation inspection, historical graph, and restoration.
1. Work across workspaces: scoped inspection, stale updates, creation, sparse patterns, and help.

Use the screen selector to inspect individual commands in priority order. Use command discovery to
search all visible command leaves. The design controls compare light/dark appearance, quiet row
selection versus marker-only selection, and compact versus roomier row spacing.

Navigation, journey steps, baseline comparison, command search, graph movement, marks, and selected
text inputs are interactive. They are illustrative transitions, not a repository simulation. Input
fields do not construct arbitrary executable commands; preview states use the recorded example
operands. Do not use this prototype to validate command execution or revision transformations.

## Baselines and what changes

Inspected source material:

- `tapes/readme-log.tape`: log, help, selection, inline expansion, and diff transitions.
- `tapes/readme-diff.tape`: diff help, search, file and hunk movement, and folding.
- `tapes/help-overlays.tape`: graph, diff, workspace, command-history, and operation-log help.
- Help-scroll tapes: compact, narrow/tall, standard, and wide layout checkpoints.
- `tapes/release-smoke.tape`: inspection, workspaces, command errors, describe preview, and
recovery.
- Published-source captures: `jk-log-v3.png`, `jk-diff-v3.png`, and `jk-log-help-v3.png` from the
  sibling `jk-screenshots/assets` directory. These are existing historical baselines, not newly
  recorded screenshots of the current working tree.
- The [roadmap](../roadmap.md), [product plan](../product-plan.md), and
  [CLI surface addendum](../plans/cli-surface-addendum.md).

The proposal keeps the recognizable graph, jj color roles, command header, full-content diff,
contextual help, and view-stack workflow. It changes selection from saturated paint to a quiet row
plus a separate gutter, reduces chrome contrast, and unifies overlays and command previews.

The fixture colors approximate the existing terminal palette. This is not a replacement for actual
jj template evaluation or ANSI rendering. A real implementation inherits jj configuration and
terminal colors. The compact graph explicitly represents a chosen compact template; it must not
silently replace the user's configured template when the terminal narrows.

## Scope and priority

The inventory is captured from **jj 0.45.1**. It contains **107 visible canonical command leaves**.
Aliases use the canonical command's design. Hidden/internal commands are outside this public CLI
inventory; command mode must continue to allow installed commands and user aliases verbatim.

There are **165 screen states** and **11 journeys**. Common workflows have dedicated screens;
less frequent commands have individual operand/effect/command reviews built from a shared control
family. A dedicated terminal screen for every CLI spelling would create unnecessary inconsistency.

The priority order follows the roadmap, with recently added commands assigned provisional places:

- P0: inspection, change creation/editing, rebase, content movement, operation recovery, bookmarks,
  fetch/push, and workspaces.
- P1: deeper inspection, file tooling, conflicts, tags/remotes, external tools, clone/init.
- P2: advanced revision surgery, convergence, revision-wide runs, sparse patterns, and config reads.
- P3: signing, bisection, Gerrit, config writes, utilities, and installed help.

The current CLI includes capabilities absent from the older 0.42 planning audit, including
`converge`,
`run`, remote tag tracking, and config maintenance. They have explicit routes in this atlas.
Priority assignments here do not silently change the release roadmap.

The complete command-to-screen mapping and all screen identifiers are in
[screen-index.json](screen-index.json). Existing behavior, future behavior, and unusual states share
one visual vocabulary; their presence in the atlas is not a release claim.

## Important design decisions

- One focus model applies everywhere. Working copy, cursor selection, and ordered marks remain
  separate. A full-width selection fill never flattens jj semantic foreground colors.
- Context remains visible behind bounded overlays. Long output is a full document view, not a
  growing confirmation dialog.
- Operand roles lead mutation previews. Source, destination, files, message, effects, and command
  appear in a predictable order. Help and Run Options remain secondary.
- Source-and-descendants, selected-revision, and whole-branch rebase are different choices. Onto,
  insert-before, and insert-after remain explicit placement choices.
- Split shows selected and remaining content. Squash shows source and destination. Restore shows
  content direction. Partial-hunk mutation delegates to the configured jj diff editor at this stage.
- A push dry-run is distinct from execution. Remote changes are not presented as recoverable through
  local operation undo.
- Workspace inspection does not silently switch mutation scope. Historical operation views keep
  operation scope visible.
- Text help is a readable reference. Search belongs in command discovery and operand selectors.
- Interactive jj tools, editors, signing providers, and credential prompts own the terminal during
  handoff. The atlas shows the before/after contract rather than inventing replacement tool UIs.

Shortcut labels illustrate the product plan's direction. They are not an approved final global
keymap: existing dogfood bindings and future prefix-menu bindings need a separate collision audit.
No native hunk editor or authoritative graph prediction is claimed by these mockups.

## Sources and regeneration

Files are intentionally simple:

- `screens.js`: hand-authored screen fixtures, command-specific reviews, and journeys.
- `atlas-shell.html`: shared terminal and gallery layout.
- `atlas-runtime.js`: local mockup interactions and optional design controls.
- `command-inventory.json`: captured installed help, including arguments and global options.
- `inventory.py`: read-only, sequential help inventory capture.
- `build.py`: assembles the fragment, optionally adding existing baselines to an external output.
- `validate.cjs`: checks all command routes, journey destinations, and unique screen identifiers.
- `screen-index.json` and `atlas.html`: generated review artifacts.

Run from the workspace root:

```sh
node docs/design/validate.cjs
python3 docs/design/build.py
markdownlint-cli2 docs/design/README.md docs/tui-design.md
```

Refresh `command-inventory.json` only when intentionally auditing a new installed jj version:

```sh
python3 docs/design/inventory.py
node docs/design/validate.cjs
python3 docs/design/build.py
```

The validator fails when a new command has no deliberate screen mapping. Review new command
semantics
before assigning a shared form. Do not manufacture a generic fallback and call coverage complete.

For an external review copy with the historical baseline PNGs embedded:

```sh
python3 docs/design/build.py \
  --baselines /path/to/jk-screenshots/assets \
  --output /path/outside/this/repository/jk-screen-atlas.html
```

The output is an HTML fragment for the Codex visualization renderer. A standalone preview needs the
visualization skill's `scripts/render.py` wrapper; opening the fragment directly also renders the
local design without host controls. No network resources are needed.

## Validation boundaries

The catalog validator checks complete visible-command coverage and navigation destinations. Syntax
checks cover JavaScript parsing. Static command review checks example option names against captured
help; it does not execute examples, validate arbitrary revsets, or prove their predicted effects.

Browser validation renders every screen in light and dark appearances at 1080, 720, and 390 pixels,
checks outer bounds and controls, and exercises journey navigation, baseline comparison, command
search, graph object movement, and marks. Representative PNGs cover graph, diff, help, rebase,
split,
and bookmarks. Browser widths are design checks, not Ratatui cell-layout tests.

Before implementation, translate chosen states into deterministic jj fixtures and Betamax journeys.
Retain their screen IDs in review notes so implementation proofs can be compared to these designs.
Test real terminal colors, Unicode widths, 80x24 and larger cell layouts, text input, output
capture,
and repository effects independently of this browser study.
