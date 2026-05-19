# Integration Strategy

This document describes how `jk` should choose between rendered `jj` output, subprocess calls,
structured command output, Rust library APIs, future RPC APIs, and upstream extraction. It is meant
to turn the upstream-versus-external TUI question into a testable engineering question rather than a
fixed assumption.

## Theory To Test

The open question is not whether `jk` must be external or must be in-tree. The question is what kind
of integration lets a TUI feel native to `jj` while remaining maintainable.

Current plausible answers include:

- an external tool that shells out to `jj`;
- an external Rust tool that uses `jj_cli`, `jj_lib`, or both;
- an external tool backed by a future stable UI/RPC API;
- upstream extraction of reusable UI-facing APIs from `jj`;
- an in-tree command, if some behaviors cannot be made robust out of tree.

`jk` should treat these as hypotheses. Each screen and workflow should make the tradeoff visible:
what is reused from `jj`, what is inferred from output, what is duplicated, and what would become
more reliable with a stronger API.

## Fidelity Model

The ideal `jk` view should look like the corresponding `jj` command would have looked, while still
being interactive. `jj` already makes reasonable default choices about what information to show, and
users override those choices based on their own mental model. Some users want more author and date
context. Some want less. Some want different templates, colors, graph symbols, or metadata. There is
no one true view that `jk` should hard-code.

That means fidelity has two parts:

1. **Semantic fidelity.** `jk` should know the change ids, commit ids, paths, graph relationships,
   operation ids, conflict state, and other meanings that drive interaction.
1. **View fidelity.** `jk` should preserve the same configured template, color, graph, formatting,
   and default display choices that make the output feel like the user's `jj`.

The best contract would expose both semantic information and view information. For example, a log
row should not force `jk` to choose between a styled string and raw repository objects. It should be
possible to know the row's change id, commit id, graph role, labels, and renderable styled segments
without reconstructing those meanings from terminal output.

That same row also needs to support TUI behavior that the plain CLI does not need. `jk` may need to
copy the change id or commit id, highlight matching text during search, keep graph selection stable,
select one or more rows as inputs to an action, or replace one compact row with an expanded inline
row showing a long description, file list, commit id, or other detail. Action flows such as `jj new`
or `jj rebase` need stable revision identities and relationships, not text that merely looks like a
revision. Those interactions should consume semantic fields and renderable segments from the same
contract where possible, instead of re-parsing the compact row and then running separate commands to
rediscover the missing meaning.

When that contract does not exist, `jk` has to repeat decisions `jj` already made: which data
belongs in a row, how config changes that row, which pieces are interactive, and how graph and
template output should be styled. Avoid repeating those decisions where possible. If they must be
repeated, record the duplication and treat it as evidence for a shared API or upstream extraction.

## Integration Ladder

Use this ladder when adding a capability. Prefer the earliest rung that can deliver the behavior
without creating misleading state, losing semantic meaning, or excessive duplication.

1. **Rendered output as-is.** Use this when the screen can display `jj` output directly and does not
   need semantic state beyond presentation. This preserves user templates, colors, graph symbols,
   wording, and diff style.
1. **Rendered output plus narrow parsing.** Use this only when the parsed structure is
   presentation-adjacent, such as selecting a rendered row or finding a file heading. Treat this as
   a fallback for semantic data. Record the assumption in
   [`fragility-register.md`](fragility-register.md).
1. **Structured output or purpose-built templates.** Use this when `jj` can expose the exact data
   without requiring `jk` to reimplement command semantics. Purpose-built templates should be narrow
   and machine-oriented, not arbitrary user-facing rendered output treated as a data model.
1. **Shared semantic and rendering APIs.** Use this when `jk` needs both semantic fields and the
   same configured template, color, graph, or formatting behavior as `jj`, but wants to avoid
   converting command output through stdout, ANSI parsing, and Ratatui reconstruction.
1. **Rust APIs.** Use `jj_cli` or `jj_lib` when a feature needs command behavior, repository state,
   template behavior, revset/fileset parsing, transaction behavior, or semantics that would be risky
   to infer from output.
1. **Future UI/RPC API.** Prefer this direction when external tools need stronger contracts while
   still supporting arbitrary user-installed `jj` binaries and backends.
1. **Upstream extraction or in-tree implementation.** Use this as evidence when `jk` repeatedly
   needs logic that only exists inside `jj` internals or cannot be made reliable as an external
   integration.

## Why Rendered Output Still Comes First

Rendered `jj` output is the closest thing to the user's actual CLI experience. Starting there keeps
`jk` aligned with:

- user templates and aliases;
- color and graph configuration;
- diff format choices;
- command wording and layout;
- whatever a future `jj` version chooses to display.

The cost is that rendered output is usually a soft agreement, not a typed contract. It is good for
display and sometimes enough for presentation-adjacent navigation. It is the wrong default for
semantic state, command semantics, or mutation plans because parsing it loses meaning that `jj`
already had before rendering.

## Semantic Data Preference

`jk` should strongly prefer code or structured contracts for semantic information. Examples include
revision identity, operation identity, bookmark tracking state, conflict state, exact paths,
revset/fileset meaning, command defaults, and transaction planning.

Rendered output may still be the best presentation source, but it should not become the primary data
model for those semantics. If a feature needs meaning that `jj` already knows internally, prefer one
of these paths:

1. structured command output;
1. a purpose-built template whose fields are narrow and explicit;
1. shared `jj` semantic-data, rendering, config, template, graph, or style APIs;
1. `jj_cli` or `jj_lib`;
1. a future RPC or UI API;
1. upstream extraction when the needed code is not available.

Parsing rendered CLI output for semantic state is acceptable only as a temporary or narrow fallback,
and should produce a fragility-register entry with a planned stronger contract when the behavior
becomes important.

## Interface Fragility

Some integration choices are fragile because of the number of transformations between `jj` semantics
and what `jk` renders, even when the original `jj` output is useful.

The current subprocess path is roughly:

1. `jj` interprets config, templates, revsets, graph settings, and command options.
1. `jj` renders styled terminal output.
1. `jk` reads stdout or stderr.
1. `jk` parses ANSI styling into intermediate spans.
1. `jk` infers structure from rendered text.
1. `jk` converts the result into Ratatui styled items.

Each step can lose information or turn a `jj` behavior change into a parser problem. More
importantly, the semantic meaning existed before rendering and has to be reconstructed afterward. A
code-native path that interprets the same config and templates and produces Ratatui spans directly
would remove several of those transformations. That can be less fragile than stdout -> ANSI ->
parsed spans -> Ratatui reconstruction.

The tradeoff is duplication. If `jk` implements template parsing, config interpretation, graph
rendering, or style translation independently, it may become more internally reliable while drifting
away from `jj`. That path is reasonable only when the shared behavior is exposed by `jj_cli`,
`jj_lib`, structured semantic/rendering APIs, or a future extracted UI-facing library. Otherwise,
local duplication should be treated as temporary evidence for upstream extraction.

## Soft Agreements

A soft agreement is any dependency on behavior that is not a documented, typed, or structured API.
Soft agreements are not forbidden, but they must be named and tested.

Examples include:

- stdout, stderr, or ANSI rendering pipelines that `jk` must reconstruct;
- graph glyph shape and row layout;
- default file heading wording;
- `--git` diff heading layout;
- stderr wording used for user guidance;
- command output ordering;
- template output that `jk` assumes can be parsed;
- inferred relationships between rendered lines and repository objects.

When a soft agreement is introduced, add or update an entry in the fragility register. The entry
should describe the assumption, failure mode, and preferred mitigation.

## Harder Contracts

Harder contracts are integration points where changes are more likely to produce compile failures,
schema failures, or focused test failures instead of silent UI drift.

Examples include:

- typed Rust APIs in `jj_lib` or `jj_cli`;
- structured command output;
- shared semantic, config, template, color, graph, or rendering APIs;
- documented template fields with narrow templates owned by `jk`;
- operation and transaction APIs;
- a future RPC response schema;
- snapshot tests around rendered output that `jk` intentionally parses.

The goal is not to avoid all rendered-output parsing. The goal is to move important or brittle
behavior toward contracts that fail loudly when `jj` changes.

## Duplication Test

Before duplicating `jj` behavior in `jk`, answer these questions in the relevant screen or workflow
plan:

1. Can the behavior be shown by using rendered `jj` output as-is?
1. Does the interaction need semantic information, or only presentation?
1. Does the view need user-configured display decisions that `jj` already knows how to make?
1. Is the behavior already available through `jj_cli`, `jj_lib`, structured output, or a narrow
   template?
1. Is the fragile part the data contract, the rendering pipeline, or both?
1. Would duplication make `jk` disagree with user configuration, templates, revsets, filesets, or
   command semantics?
1. Would extracting a reusable API from `jj` help other UI tools too?
1. Is this duplication evidence that the external-tool path needs a stronger upstream contract?

High-risk duplication includes:

- template parsing, formatting, and rendering;
- config interpretation and style/color resolution;
- revset and fileset parsing or evaluation;
- graph layout and glyph semantics;
- default row composition and user-configured view choices;
- command planning, defaults, and safety checks;
- operation transaction behavior;
- conflict and merge state modeling;
- bookmark and remote-tracking semantics.

## Upstream Feedback Loop

`jk` should feed useful evidence back into the `jj` ecosystem. The evidence should be specific:
which behavior was hard to implement, which output assumption was fragile, which code would need to
be copied, and which API would make the screen more reliable.

Possible conclusions should stay open:

- If rendered output plus narrow parsing works well for presentation-adjacent flows, that supports
  `jk` as an external tool.
- If `jk` repeatedly needs `jj_cli` or `jj_lib`, that supports clearer public Rust APIs.
- If backend compatibility and user-installed `jj` behavior matter most, that supports a future RPC
  or structured UI API.
- If important flows require single-operation transactions that cannot be expressed externally, that
  becomes evidence for stronger upstream integration.

The aim is to build the best possible `jj` TUI and learn, through implementation, which integration
model is actually correct.
