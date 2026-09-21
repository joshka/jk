# jk screen atlas

Proposed jk workflows for inspection, history editing, content movement, recovery, sharing, and
workspaces. The mockups use synthetic repository data and never run commands. Consult the
[coverage audit](../command-coverage.md) for implemented behavior and the
[design guidelines](../tui-design.md) for the visual and interaction contract.

The gallery's screen selector and journey controls are review tools, not proposed TUI controls.

## Fixed-cell terminal images

Judge terminal fit using the fixed-cell images: one font size, integer cell coordinates, flat
colored surfaces, and no decorative borders.
`terminal_mockups.py` emits ANSI truecolor cells into a real pseudo-terminal; Betamax renders those
cells into PNGs. The 1200x768 capture settings provide a 92-column, 38-row terminal.

The renderer uses the same 165 screen fixtures as the browser study, including a borderless file
picker, contextual help, and mutation previews. The abandon confirmation follows the user-supplied
terminal reference: affected files and counts, explicit consequences, a visible command, and colored
View diff / Cancel / Abandon action regions. Cancel starts focused. Betamax renders the mockup's
terminal output; these images do not demonstrate working repository operations.

Generate every screen into an external image directory:

```sh
python3 docs/design/capture_terminal.py /path/outside/this/repository/terminal-mockups --run
```

Use `--theme light` for the alternate palette. The output includes all PNGs, an image-only gallery,
a reproducible Betamax tape, terminal dimensions, and selected terminal-state JSON captures.

## Review the design

Open the [atlas](atlas.html) to inspect the proposed screens. The inline Codex version can also
compare historical captures of the graph, diff, and graph-help screens.
Baseline PNGs remain in the separate screenshot repository, not embedded in this repository.

Start with these journeys:

1. Inspect a change: graph, inline details, show, diff, files, search, folding, help, and view options.
1. Rebase a stack: action menu, source semantics, destination, preview, options, execution, recovery.
1. Split and squash: filesets, selected/remaining content, command review, and external hunk editing.
1. Share work: bookmarks, move, fetch, push dry-run, success, and failure.
1. Recover a mistake: command history, operation inspection, historical graph, and restoration.
1. Work across workspaces: scoped inspection, stale updates, creation, sparse patterns, and help.

Use the screen selector to inspect individual commands in priority order. Use command discovery to
search all visible command leaves. The design controls compare light/dark appearance, quiet row
selection versus marker-only selection, and compact versus roomier row spacing.

Navigation, command search, graph movement, marks, and selected text inputs respond to interaction.
Preview states use fixed example operands; input fields do not construct arbitrary commands. Validate
execution and revision transformations separately on fresh fixture repositories.

## Historical baselines

- `tapes/readme-log.tape`: log, help, selection, inline expansion, and diff transitions.
- `tapes/readme-diff.tape`: diff help, search, file and hunk movement, and folding.
- `tapes/help-overlays.tape`: graph, diff, workspace, command-history, and operation-log help.
- Help-scroll tapes: compact, narrow/tall, standard, and wide layout checkpoints.
- `tapes/release-smoke.tape`: inspection, workspaces, command errors, describe preview, and recovery.
- Published-source captures: `jk-log-v3.png`, `jk-diff-v3.png`, and `jk-log-help-v3.png` from the
  sibling `jk-screenshots/assets` directory. These predate the integration under review.
- The [roadmap](../roadmap.md), [product plan](../product-plan.md), and
  [CLI surface addendum](../plans/cli-surface-addendum.md).

The proposed appearance preserves the graph, jj colors, command header, full-content diff, contextual
help, and view stack. It replaces saturated selection with a separate gutter and an optional quiet
row background. Dialogs follow the original menus' slate surfaces, teal keys and focus, blue-gray
secondary actions, and burgundy destructive actions, with readable light equivalents. Keyboard hints
remain the usual controls; filled buttons are useful for bounded choices such as Cancel and Abandon.

Use opaque dialog fills while keeping the real repository view visible around them. Implementation
proof must show both surfaces together; an isolated mockup cannot establish that separation.

Fixture colors approximate a terminal palette. The implementation must inherit jj configuration and
terminal colors. The compact graph represents an explicitly chosen template; shrinking a terminal
must not silently change the user's configured template.

## Scope and priority

The inventory is captured from **jj 0.45.1**. It contains **107 visible canonical command leaves**.
Aliases use the canonical command's design. Hidden/internal commands are outside this public CLI
inventory. Command mode's implemented restrictions are documented in the coverage audit.

There are **165 screen states** and **11 journeys**. Common workflows have dedicated screens;
less frequent commands reuse operand, effect, and command controls.

The priority order follows the roadmap, with recently added commands assigned provisional places:

- P0: inspection, change creation/editing, rebase, content movement, operation recovery, bookmarks,
  fetch/push, and workspaces.
- P1: deeper inspection, file tooling, conflicts, tags/remotes, external tools, clone/init.
- P2: advanced revision surgery, convergence, revision-wide runs, sparse patterns, and config reads.
- P3: signing, bisection, Gerrit, config writes, utilities, and installed help.

The jj 0.45.1 inventory includes `converge`, `run`, remote tag tracking, and config maintenance,
which were absent from the 0.42 planning audit. Their priority assignments remain provisional.

See [screen-index.json](screen-index.json) for command mappings and screen identifiers.

## Workflow decisions

- One focus model applies everywhere. Working copy, cursor selection, and ordered marks remain
  separate. A full-width selection fill never flattens jj semantic foreground colors.
- Bounded overlays preserve repository context. Long output uses a full document view.
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
- Foreground jj tools, editors, signing providers, and credential prompts must own terminal input
  during handoff. The mockups show the proposed return path; foreground handoff is still planned.

Shortcut labels illustrate the product plan's direction. They are not an approved final global
keymap: existing dogfood bindings and future prefix-menu bindings need a separate collision audit.
No native hunk editor or authoritative graph prediction is claimed by these mockups.

## Sources and regeneration

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

The validator fails when a new command lacks a screen mapping. Review its operands and effects
before assigning a shared form; a generic fallback does not establish command coverage.

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

The validator checks command mappings and navigation destinations. Syntax checks cover JavaScript
parsing. Static command review compares example options with captured help; it does not execute
examples, validate arbitrary revsets, or prove predicted effects.

Browser validation renders every screen in light and dark appearances at 1080, 720, and 390 pixels,
checks outer bounds and controls, and exercises journey navigation, baseline comparison, command
search, graph object movement, and marks. Representative PNGs cover graph, diff, help, rebase, split,
and bookmarks. Browser widths do not validate Ratatui cell layouts.

Before implementation, translate chosen states into deterministic jj fixtures and Betamax journeys.
Retain their screen IDs in review notes so implementation proofs can be compared to these designs.
Test real terminal colors, Unicode widths, constrained and larger cell layouts, text input, output
capture, and repository effects. Pair PNGs with terminal-state JSON after semantic waits, and follow
the size and fixture matrix in the design guidelines.
