# TUI design guidelines

Status: design contract for new and revised UI, not a claim that every rule is implemented today.

This guide defines the visual and interaction language for all jk screens, items, controls, help,
menus, dialogs, and views. It elaborates the [product plan](product-plan.md), especially sections
3.2 through 3.8. That plan owns command semantics and key assignments; this guide owns their
presentation. Changes to either must preserve the other, or update both with an explicit rationale.

## 1. Design intent

jk should feel like an interactive form of the user's jj CLI: familiar output, precise controls,
clear consequences, and stable context. Its visual character is restrained and deliberate.
Readability, alignment, spacing, and consistent emphasis provide polish. Decoration must not compete
with repository content. A muted interface still needs obvious focus and readable text.

Every surface must answer:

- Where am I, and which repository or workspace does this affect?
- What object or text am I inspecting?
- Where will keyboard input go?
- What can I do next, and what will that action affect?
- How do I return, cancel, or recover?

## 2. Presentation ownership

### jj owns repository meaning

Preserve configured jj templates, aliases, graph and node styles, labels, colors, descriptions,
revsets, filesets, diff formats, and diagnostic wording. Do not replace these with a jk house style.
Use the installed jj output under the same effective configuration as the comparison reference.

- Preserve graph topology, prefixes, Unicode symbols, and relationships between continuation lines.
- Preserve styled spans within output; do not flatten a row to one foreground color for selection.
- Preserve whitespace and signs in diffs, code, and command output.
- Do not rename bookmarks as branches or introduce staging language for jj operations.
- Do not infer object identity or action eligibility from rendered colors or decorative symbols.
- If a custom template prevents semantic navigation, keep the output readable and explain which
  interaction is unavailable. Never silently select a guessed revision.
- Respect explicit view options. Clearly expose an active override and a way to return to the
  configured default; opening jk must not silently substitute a preferred template or diff format.

Presentation fidelity does not require identical terminal coordinates. jk may add chrome, reserve
an interaction gutter, fold content explicitly, and provide previews. Width-dependent jj rendering
must use the actual content width after those reservations. Never reflow a graph or diff as prose.

### jk owns interaction

jk owns focus, selection, marks, input fields, menus, help, dialogs, status, and navigation. These
must use one consistent visual language across views. Keep jk annotations outside jj-owned content
where possible. A synthetic sticky header or folded-content summary must be distinguishable from
original output and must not be copied as if jj emitted it.

Keep the text of commands and diagnostics available without truncation, even when a compact summary
is initially shown. Copy operations must exclude chrome, selection markers, and fold decorations.

## 3. Visual hierarchy and appearance

### Emphasis budget

Use this hierarchy in every screen:

1. Active input or consequential decision.
1. Selected object and its action scope.
1. Repository content and section titles.
1. Supporting metadata, hints, and separators.

One surface owns keyboard focus at a time. Avoid several equally bright bars, borders, and headings.
Use bold for short titles or key facts, not whole panes. Preserve jj's own emphasis within content.
Use one restrained configurable accent for jk focus and active controls. Semantic warning and error
styles remain distinct. Do not use gradients, pulsing highlights, decorative shadows, or animated
selection transitions by default.

### Semantic styles

Define shared roles rather than choosing colors independently in each renderer:

| Role            | Treatment                                                           |
| --------------- | ------------------------------------------------------------------- |
| Canvas          | Terminal-default foreground and background.                         |
| Surface         | A coherent overlay background with readable foreground.             |
| Primary text    | Normal readable text; no arbitrary recoloring of jj output.         |
| Supporting text | Lower emphasis, still readable; never essential text in faint gray. |
| Title           | Short bold label, normally without a filled badge.                  |
| Separator       | Quiet line or spacing; no competing emphasis.                       |
| Focus           | Accent marker and an explicit active-control indication.            |
| Selection       | Quiet flat background when compatible, plus a persistent marker.    |
| Mark            | Explicit persistent symbol; optional accent.                        |
| Search match    | Local span emphasis, distinct from selection and focus.             |
| Warning / error | Explicit wording or symbol plus semantic color.                     |
| Success         | Brief factual result; subdued unless user attention is needed.      |

Selection, match, and diagnostic styles must have documented composition rules. Preserve semantic
foregrounds by default. If a background would obscure configured jj colors, use marker-only
selection instead. Search emphasis must not erase diff additions/deletions or conflict information.
A cursor marker must remain visible when the selected object also contains a match or a warning.

### Terminal and user preferences

The default appearance inherits terminal and jj colors. Do not hard-code a black canvas, white text,
or a dark selection surface across all terminals. Do not assume ANSI color names correspond to
particular RGB values in the user's palette.

A coordinated light or dark appearance may style jk-owned surfaces, but must not overwrite jj
content colors. Provide semantic overrides and selection styles rather than per-screen palettes.
Appearance settings must not alter key behavior, action scope, or information ordering.

Support marker-only and high-contrast interaction treatments. In monochrome or reduced-color output,
focus, marks, warnings, and selected options must remain distinguishable by symbols and wording.
Terminal capability detection must have a safe fallback and an explicit override; uncertainty must
not block startup. Opaque dialog cells must clear underlying text, including on a default or
transparent terminal background.

## 4. Shared screen structure

Use a focused view stack, not a permanent pane grid. The main content is borderless unless a border
communicates a real independent interaction region.

- Header: view name and enough repository/workspace/filter context to establish scope.
- Body: the task's primary content, using most of the available area.
- Footer: a small contextual action set and, when needed, concise result or progress information.

Avoid repeating the same title, target, status, or key instructions in several locations. Keep
workspace scope visible during mutations, including when the underlying screen is covered.
Reserve persistent chrome space so a transient status does not make the content jump.

Use a one-cell horizontal inset for jk-owned prose and controls where space permits. Use one blank
line between logical groups, not between every item. Do not indent or pad jj output inconsistently.
Align keys, labels, values, and status columns across the entire surface. Prefer whitespace to
nested boxes. Use one simple border around a floating overlay when it helps distinguish ownership.

Optional preview panes must earn their space. Show them only when both content regions remain
readable. Label their target and focus state. On constrained terminals use a full view with a stable
return path, rather than squeezing both panes into unreadable columns.

## 5. Items, selection, marks, and focus

Selection identifies the current object; marking collects explicit action operands; focus identifies
where input goes. Scrolling changes the viewport. Never make these interchangeable.

- Use a dedicated gutter marker for the selected item; never replace jj graph nodes with it.
- For multiline objects, keep the anchor obvious and indicate the selected extent in the gutter.
  A subtle anchor-row background is optional; a saturated block covering all lines is not.
- Marked items retain a separate symbol when the cursor moves away. Show a count and, when operand
  order matters, an order indicator. Do not imply that hidden marks have been cleared.
- Preserve jj's working-copy node independently of cursor selection.
- An unfocused list may retain its selected object but must lose active-focus emphasis.
- Object navigation reveals the selected object. Viewport-only scrolling must not change selection
  or snap back to the cursor on the next draw. Explain off-screen action scope before mutations.
- Preserve selection by identity across refreshes. If the object disappears, select a sensible nearby
  object and report the change. Do not run a pending action against an automatic replacement.

Long object names may use a compact representation in browsing lists only when an inspection path
reveals the full value. Identifiers must remain unambiguous. Mutation targets and selected control
values must be fully inspectable before execution; an ellipsis is not sufficient confirmation.

## 6. Controls and input

All controls need a label, current value or state, focus treatment, activation behavior, and an
unavailable-state explanation where relevant. Hover must never be required.

| Control          | Required behavior                                                     |
| ---------------- | --------------------------------------------------------------------- |
| Action           | Verb and object; activation performs the named step.                  |
| Toggle           | Visible on/off state; activation changes it without executing a task. |
| Single choice    | Distinguish applied value from the cursor's candidate.                |
| Multiple choice  | Separate marks from cursor; expose count and any ordering.            |
| Text input       | Visible label/caret; preserve typed value on validation failure.      |
| Completion list  | Separate candidate navigation, acceptance, and command execution.     |
| Unavailable item | Explain why; never rely on dim color alone.                           |

Use readable textual symbols with an ASCII fallback. Do not require a patched font or emoji widths
for essential controls. Use arrows and established key names consistently: `Enter`, `Esc`, `Space`,
`Tab`, `Shift-Tab`, and `Ctrl-x`. Preserve uppercase/lowercase distinctions in bindings.

Text entry owns printable keys, including `?`, `q`, and navigation letters. Do not intercept valid
revset, fileset, description, or command text as global shortcuts. Provide discoverable help access
outside that input binding conflict. `Esc` dismisses completion before leaving the input surface;
cancellation must not execute partially entered text.

Validation errors belong next to the relevant field and must state how to fix the input. Preserve
the original value and cursor. Do not announce incomplete input as a command failure on every key.
Show loading separately from no matching completions.

Keyboard is the complete interaction path. If mouse support exists, clicking focuses/selects and
activates only the indicated control; scrolling must affect the region under the pointer without
executing actions. Never require a double-click or pointer gesture for an essential workflow.

## 7. Menus, pickers, and view options

Menus offer actions; pickers choose operands; view options change presentation. Give each a concrete
title and keep their consequences distinct.

- Group entries by user task, not implementation enum or dispatch order.
- Align shortcut, action, current value, and status columns globally.
- Use explicit labels such as `Rebase` or `Diff format`; avoid redundant command paraphrases.
- Show submenu continuation consistently. Opening a submenu is not execution.
- Keep an unavailable action visible with a reason when it helps explain the current state; omit
  actions that are irrelevant to the surface. Do not show unfinished implementation placeholders.
- List pickers use object movement. Navigation clamps at the ends by default; any wrapping behavior
  must be deliberate and consistent within that control family.
- Applying a view option preserves the selected object where possible and exposes the applied value.
- `Esc` returns one level and restores the originating focus and selection.

Search belongs in large operand pickers and command discovery where it serves the task. It must
not turn ordinary contextual help into a search-first interface.

## 8. Help and hotbars

Help is a contextual reference to implemented actions in the active scope. Generate help and
hotbars from the same binding/action information used for dispatch. Never display a shortcut that
is unavailable, remapped, or intercepted by the current mode without explaining that state.

Order groups for the reader: Open and inspect, Change actions, History and recovery, view options,
Move and find, and Session, omitting empty groups. Adapt the order when a specific task warrants it.
Help is document content: scroll by rendered line, with no fake selectable command cursor.

Use one aligned key column and one action column as the baseline. Add object or command detail only
when it clarifies behavior. Split bindings that perform different actions into separate rows.
Multiple columns are allowed only when they reduce scrolling, remain readable, and preserve an
obvious top-to-bottom reading order. Do not spread short entries across the entire terminal.

Size help from the full document, bounded by the terminal, and keep its rectangle stable while
scrolling. Show scroll indicators only when content is hidden. Keep close/navigation instructions
available without duplicating the entire underlying hotbar. Closing help restores exact context.

The hotbar prioritizes the next useful actions and a route to help. Drop lower-priority hints before
wrapping or truncating key labels. It is not a catalog of every binding. Error/progress text must
not permanently displace the only visible cancel or recovery instruction.

## 9. Dialogs and mutation previews

Use dialogs for bounded decisions or input. Use full views for long documents and detailed diffs.
Size dialogs to rendered content with minimal padding and terminal bounds; do not use arbitrary
large rectangles for a two-line decision. Keep titles and primary controls stable when errors appear.

A mutation preview uses this reading order:

1. Action and repository/workspace scope.
1. Explicit source, destination, and other operand roles, including marked-object scope.
1. Consequences and relevant conflicts, immutable constraints, or remote effects.
1. Exact jj command or sequence, with a full inspection/copy path.
1. Execute, edit where supported, copy, and cancel controls.

Use concrete control wording such as `Run rebase`, not `OK`. Preserve the established preview key
contract from the product plan. Enter may run only the currently reviewed, valid preview; opening a
menu or selecting a target must not also execute it. Changes to operands, edited commands, or
repository state that invalidate the preview require a refreshed preview before execution.

Do not claim a predicted graph or effect is authoritative unless it is validated. Clearly label
estimates and unavailable previews. Warnings must identify the actual condition; avoid generic
warning banners on every action. Cancellation leaves the repository unchanged before execution.
After execution starts, distinguish canceling a running process from undoing completed operations.
Never promise rollback merely because the dialog can be closed.

On failure, preserve input and the command, show jj's diagnostic, and offer relevant next steps.
On success, return to a meaningful context with a short result and history/recovery access. Do not
require dismissing a success dialog for routine operations.

## 10. View-specific contracts

| Surface                | Additional rules                                                          |
| ---------------------- | ------------------------------------------------------------------------- |
| Log and evolog         | Preserve jj graph/templates; reveal selected change without rewriting it. |
| Show, status, output   | Preserve jj sections and diagnostics; scroll as a document.               |
| Diff                   | Preserve format, signs, whitespace, colors, and file/hunk boundaries.     |
| Files and hunks        | Clearly distinguish navigation selection from marked mutation operands.   |
| Operation log          | Preserve jj operation presentation; distinguish inspection from restore.  |
| Command history        | Show scope, command, outcome, and inspectable output; replay is explicit. |
| Workspaces             | Distinguish inspected workspace, active scope, and stale state.           |
| Bookmarks/tags/remotes | Preserve jj names and state; label local/remote relationships explicitly. |
| Command mode/discovery | Separate typing, suggestions, preview, and execution.                     |
| Recovery               | Identify the operation and affected state before offering restoration.    |

Diff folds need a visible fold marker and hidden extent. Sticky headers must remain distinguishable
from original lines. File/hunk navigation must not silently mark content. Do not change diff format
merely to accommodate a layout; use an explicit view option.

Long output gets a document view, not a growing dialog. Reading a command history entry must never
rerun it. Opening another workspace for inspection must not silently retarget the next mutation.

## 11. Responsive layout and text

Measure terminal cells, not bytes or character counts. Handle wide glyphs, combining marks, ANSI
styles, tabs, and clipped content without corrupting adjacent columns. Prefer actual rendered
content measurements to fixed percentages.

When space decreases, adapt in this order:

1. Remove optional metadata and redundant spacing.
1. Use shorter unambiguous labels and fewer hotbar hints.
1. Collapse discretionary columns or preview panes.
1. Scroll the appropriate content region and show hidden-content indicators.
1. If a decision cannot be presented safely, show a compact explanation and a back/resize path.

Keep controls and action scope readable. Wrap prose at word boundaries; do not wrap key tokens or
control labels into fragments. Preserve the configured wrapping semantics of jj content. Long
commands, paths, and identifiers need explicit horizontal scrolling or a detail surface when they
cannot fit; never silently remove meaningful command arguments.

Resize must preserve input, focus, marks, and selected identity. Hidden focused controls must move
into view or transfer focus predictably. Never let a zero-height body or tiny terminal panic or
leave an invisible action active. Avoid layout oscillation as selection or scroll position changes.

## 12. Feedback, refresh, and continuity

Distinguish initial loading, refreshing existing data, empty results, filtering with no matches,
command failure, and unavailable capabilities. State what happened and the useful next action.
Keep previous readable data during refresh when valid, and label it stale when that matters.

Navigation feedback is immediate. Use progress indicators only for real pending work, without
claiming an invented percentage. Background refresh must not steal focus, reopen overlays, reset
scroll, or overwrite text input. Stop or supersede obsolete previews so an older result cannot
replace the current target's preview.

Return from details, help, dialogs, an editor, or a shell to the same logical context. Restore terminal
state and refresh after external tools. Report failures with persistent inspectable diagnostics;
short success messages may expire. Respect reduced-motion preferences and avoid decorative motion.

## 13. Implementation and review contract

Use shared semantic styles and shared control behavior. Avoid per-view color constants or string
heuristics that decide whether a line is a heading, warning, or selected control. Keep semantic
state separate from rendered text. This is a design boundary, not a requirement for a large widget
framework or a new renderer.

For each UI change, reviewers must be able to identify:

- Which content is jj-owned and which annotation is jk-owned.
- The selected object, focused control, marks, working copy, and action scope independently.
- How the user discovers, executes, cancels, and recovers from the action.
- Behavior with a custom jj template, colors, graph style, and relevant diff format.
- Behavior in light, dark, low-color, and monochrome environments.
- Narrow/short layout behavior and full access to long names, commands, and diagnostics.

Validate representative screens at 80x24, 120x40, and 160x50, plus a constrained 60x16 fallback.
Include multiline descriptions, long paths/bookmarks, Unicode, conflicts, empty results, loading,
errors, off-screen marks, and refresh while inspecting. These sizes are review fixtures, not fixed
breakpoints. Compare jj-owned content with output from the same installed jj/config and content
width; normalize only known variable fields.

Use layout assertions for bounds, alignment, scroll indicators, focus, and selection preservation.
Use PNG proof for visual hierarchy and readability, with deterministic fixtures and a fresh filename
for each revision. Review graph, diff, help, a picker, and a mutation preview together when changing
shared styling. A screenshot cannot substitute for interaction or config-fidelity checks.

## 14. Adoption

Apply this guide to new surfaces immediately. Bring existing surfaces into compliance in small,
reviewable changes rather than combining a behavior rewrite with a palette change.

Start with shared styles and the selection/focus/mark contract, then chrome and help, then menus,
inputs, and mutation previews. Current bright graph selection, fixed black/white chrome, and
per-surface style choices are migration targets, not approved exceptions. Preserve jj output and
existing documented key semantics throughout.

A visual-only change does not require new product claims. Once visible behavior or appearance ships,
review the README, crate README, website, and media for consistency under the repository's normal
publication rules. This document itself adds no new implemented capabilities.
