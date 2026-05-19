# Product Direction

This document captures the direction for `jk` as a product. It is intentionally separate from
implementation architecture notes: this says what kind of tool `jk` should become, not how every
module should be shaped.

## Product Shape

`jk` is a Rust TUI for `jj` that should feel like a native, navigable extension of the existing `jj`
CLI UI.

The core shape is:

- log-first;
- one active view at a time;
- vimish movement;
- fast navigation between jj concepts;
- refresh-in-place after external changes;
- rendered `jj` output remains the default presentation source.

The default experience should be a focused `jj` session, not a repository dashboard. Panes,
previews, overlays, and split layouts may be useful as local presentation choices, but they should
not become the main mental model.

## Principles

### Stay Close To `jj`

`jk` should preserve the user's configured `jj` as much as possible:

- templates;
- colors;
- graph symbols;
- wording;
- diff formats;
- revset behavior.

The goal is not to invent one correct TUI view of history or status. `jj` has defaults, and users
change those defaults to match what they consider useful. `jk` should respect that configurability
while adding interaction.

Rendered output should be treated as presentation first, not as the preferred source of semantic
data. When `jk` has to parse output, it should parse only enough structure for presentation-adjacent
behavior such as row selection, sticky context, search, and copy actions. For semantic information,
prefer code or structured contracts that preserve meaning before it is flattened into terminal text.
`jk` should degrade honestly when rendered output changes instead of pretending to own a complete
repository model.

### Treat Integration As Theory Testing

`jk` should make the external-tool versus upstream-integration tradeoff concrete through
implementation. The project should not assume in advance that a TUI must live inside `jj`, or that a
separate tool can always reproduce the parts of `jj` it needs.

The working hypothesis is:

- use rendered `jj` output as the default presentation source;
- keep non-essential structure opaque;
- identify every soft agreement, such as parsing underspecified output;
- distinguish data fragility from rendering-pipeline fragility;
- seek APIs that expose both semantic data and renderable view information;
- prefer code or structured APIs over parsing CLI output for semantic state;
- prefer harder contracts when a feature needs exact semantics;
- treat repeated duplication as evidence for better upstream APIs or extracted libraries.

This keeps the debate practical. If rendered output plus narrow parsing is enough for
presentation-adjacent flows, that is a useful result. If the tool repeatedly needs `jj_cli`,
`jj_lib`, a future RPC API, or code extracted from `jj`, that is also useful evidence.

### Prefer Depth Over Panes

The primary navigation model should be drill-down and return:

1. Start in the graph.
1. Select a change.
1. Open `show` or `diff`.
1. Move between files or matches.
1. Go back to the previous view.
1. Refresh when external work changes the repository.

This keeps the app easy to reason about in a terminal and avoids focus management as a product
feature.

### Keep The Home View Useful

The graph is the home surface. It should make common work cheap:

- scan the current stack;
- move by logical revision item, not visual noise;
- open the selected change quickly;
- copy change and commit identity;
- refresh without losing local context when possible.

### Treat Help As Part Of The App

`jk` should be discoverable without becoming command-sprawl. Help, keymaps, and copy menus should be
compact, current, and available in-app.

The goal is not to expose every possible `jj` command as a first-class button. The goal is to make
the important navigation and inspection loops obvious.

## Ideas To Preserve From The Old Main Branch

The old `main` branch was built with a broad, vibe-driven approach. Its code should not be treated
as the direction for this branch, but it contains useful product ideas and visual references.

Useful ideas to preserve:

- a log-first home screen;
- item-based revision navigation;
- low chrome around rendered `jj` output;
- fast back/forward screen history;
- status as a focused working-copy triage view;
- operation log as a recovery and audit view;
- file, bookmark, tag, resolve, and workspace utility views;
- compact help and keymap views;
- command prompt and confirmation flows for selected high-value actions;
- preview-before-confirm behavior for risky mutations;
- VHS screenshots and GIFs as design references and future regression assets.

Useful local artifacts include:

- `target/vhs/static-log.png`;
- `target/vhs/static-status.png`;
- `target/vhs/static-help.png`;
- `target/vhs/static-operation-log.png`;
- `target/vhs/static-file-list.png`;
- `target/vhs/static-bookmark-list.png`;
- `target/vhs/tutorial-dynamic-navigation.gif`;
- `target/vhs/tutorial-dynamic-safety.gif`;
- `target/vhs/tutorial-dynamic-command-history.gif`;
- `target/vhs/tutorial-dynamic-remote-ops.gif`.

These artifacts should be mined for product intent, interaction patterns, and visual density. They
should not be treated as implementation requirements.

## Ideas To Avoid Inheriting

The old branch should not steer implementation by inertia. Avoid inheriting:

- pane-first or dashboard-first layout;
- a command launcher as the center of the product;
- broad coverage of every `jj` command before the core navigation loop is excellent;
- generated tutorial scope as a feature roadmap;
- old module boundaries and abstractions;
- copied code from experiments unless it has been re-evaluated against the current architecture.

When an old idea is still valuable, reintroduce it deliberately in the current code shape with
focused tests.

## Near-Term Product Priorities

The healthiest near-term direction is to make the core loop excellent before expanding command
coverage:

1. Graph navigation.
1. `show` and `diff` drill-down.
1. Back/forward history.
1. Refresh-in-place.
1. Search and copy.
1. Sticky file context.
1. Compact help/keymap discovery.
1. Focused status and operation-log views.

Mutation workflows should come later and should start with high-signal, low-surprise flows. Risky
operations need explicit confirmation and a clear preview when possible.

## Decision Filter

Use this filter for new product ideas:

1. Does it preserve `jj` output and behavior rather than replacing it?
1. Does it make the log -> inspect -> return loop faster or clearer?
1. Can it work as one active view, a drill-down view, or a temporary overlay?
1. Does it avoid introducing a separate repository model?
1. Can it fail honestly when `jj` output or config differs?
1. Does it make fragile integration assumptions visible?

If the answer is mostly no, the idea probably belongs outside `jk` for now.
