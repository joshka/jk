# Rust Style Guidance For Agents

Load this document when a change touches Rust implementation structure, public or crate-local APIs,
naming, visibility, or refactoring.

## Maintenance Goal

Write `jk` code so a future maintainer can read a module from top to bottom and understand the local
concept without reconstructing hidden cross-module state.

Prefer code that is direct, explicit, and boring:

- small functions with meaningful names;
- narrow structs that own one coherent concept;
- local helper functions near the behavior they support;
- simple enums for command and state transitions;
- named locals when they make parsing, rendering, or side effects easier to audit.

Avoid clever generic layers, broad context objects, callback-heavy control flow, and wrappers that
exist only to hide one line of code.

## Module Shape

Put the main concept near the top after imports and module docs. Arrange code in the order a reader
naturally needs it:

- public or central type first;
- inherent methods near the type;
- rendering, command execution, and refresh helpers near their callers;
- parsing helpers near the parser they support;
- tests near the behavior they prove.

Caller-before-callee ordering is preferred when it makes the workflow clear. It is acceptable to put
small low-level helpers earlier when that improves local reading.

## Abstraction Standard

Add an abstraction only when it reduces real complexity:

- repeated behavior becomes easier to audit in one place;
- an invariant becomes impossible or harder to violate;
- a concept earns a clear name that readers already need;
- view code becomes more local without hiding side effects.

Do not add an abstraction merely because two blocks look textually similar. In TUI code, similar
rendering code can still represent different concepts. Keep the concept split when merging would
force readers to understand flags, generic parameters, or indirect callbacks before they can
understand the behavior.

## Visibility

Keep visibility narrow. Prefer private items until another module has a concrete need.

Use `pub` for crate-root or public library API only if the item is intentionally part of the
external surface. Use narrower visibility only when it simplifies a real module boundary; avoid
`pub(crate)`, `pub(super)`, and `pub(in ...)` as default tools.

When an item becomes visible outside its module, its name and docs should make the ownership
boundary obvious.

## Naming

Use names that expose ownership and side effects:

- `load_*` or `refresh` for operations that shell out or reload external state;
- `render_*` for drawing only;
- `parse_*` for narrow conversion from rendered text;
- `*_revset`, `*_change_id`, and `*_commit_id` when the distinction matters;
- `scroll_offset`, `line_index`, and `viewport_height` for navigation math.

Avoid vague names such as `data`, `manager`, `handler`, `processor`, `context`, or `state` unless
the surrounding type gives the term precise meaning.

## Error Handling

Use `color_eyre::Result` for app-level fallible paths, matching the existing crate style.

Errors from `jj` execution, output parsing, terminal I/O, and clipboard writes should either reach
the app status line or be handled explicitly near the call site. Do not swallow failures unless the
ignored failure is intentional and the code makes that choice obvious.

Keep partial-state updates conservative. If refresh fails, preserve the current usable view when
practical and report the error.

## Iterators And Loops

Use iterators for pure transformations over lines, spans, options, and parsed rows. Use loops when
the code has visible side effects, early stateful exits, or scroll/search logic that is clearer step
by step.

Do not compress navigation or parsing logic into dense iterator chains if named locals would make
the edge cases easier to audit.

## Ratatui Code

Keep layout and chrome centralized in `tui.rs`. View modules should build the widgets that represent
their content and let shared chrome handle the app frame.

Prefer stable dimensions and saturating arithmetic for scrollable UI. Any calculation involving
terminal height, line count, selected row, or scroll offset should handle empty content and very
small viewports.

When styling selected rows or search highlights, preserve the original rendered jj styling unless
there is a clear reason to override it.

## Dependency Posture

Do not add dependencies for small local helpers. A new dependency should have a clear job, a narrow
integration point, and a maintenance payoff.

For Rust dependency updates, keep maintenance-only bumps separate from behavior changes when
possible. Prefer widening only as much as the crate honestly supports, and let `Cargo.lock` carry
newer compatible patch releases.
