# Fragility Register

This register tracks places where `jk` depends on soft agreements with `jj`: rendered output shapes,
underspecified command behavior, inferred state, or duplicated logic. These assumptions are allowed
when they are narrow and useful, but they should be visible during planning and review.

## Current Code

  | Area                 | Current dependency                                             | Risk                                                               | Mitigation                                                                 |
  | -------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------------------- |
  | Render pipeline      | stdout -> ANSI parser -> styled spans -> Ratatui items         | Each conversion can lose semantic structure or style intent        | Keep conversion local; prefer shared rendering APIs if structure expands   |
  | Semantic inference   | Meaning reconstructed from rendered text after `jj` had it     | `jk` can misread or discard meaning that code paths could preserve | Prefer structured output, templates, `jj_cli`, `jj_lib`, or upstream APIs  |
  | View decisions       | Row composition, templates, colors, graph, and labels          | `jk` can diverge from jj defaults or user configuration            | Prefer APIs that expose semantic data and renderable view parts together   |
  | Log row expansion    | Compact row re-parsed before richer inline detail is attached  | Expanded rows can disagree with the compact row or lose identity   | Prefer a row contract with ids, graph info, styled spans, and detail hooks |
  | Action selection     | Mutation inputs inferred from selected rendered rows           | Actions can target the wrong revisions or lose multi-row meaning   | Carry exact revision ids and graph relationships through guided flows      |
  | Graph grouping       | Rendered `jj log` lines and graph glyph layout                 | Output shape changes can affect row selection or target extraction | Keep the parser narrow; test elided, malformed, and non-revision rows      |
  | Revision identity    | Template output for change and commit ids                      | Template behavior or fields may change                             | Keep templates minimal; move to typed API if identity extraction expands   |
  | Direct startup views | Local inference of `show` and `diff` targets from startup args | Option edge cases can be misread                                   | Preserve raw args; test supported direct-view cases only                   |
  | File headings        | Default and `--git` rendered diff/show headings                | Heading changes can break sticky file context                      | Preserve raw document lines; test default and git diff heading forms       |
  | Sticky projections   | Active file inferred from scroll position and headings         | Unusual output can produce no active file                          | Degrade to a plain scrollable document                                     |
  | Diff format state    | Detecting and adding `--git` format arguments                  | Other diff tools or future formats may not match assumptions       | Keep behavior explicit and local to diff/show command construction         |

## Planned Screens

Each planned screen should add entries here before implementation if it needs output parsing or
duplicated jj semantics.

Narrow machine-oriented templates are different from arbitrary user-facing rendered output. They are
acceptable when they expose explicit fields for `jk`; they should not become a second copy of jj's
template language or presentation model.

  | Screen or workflow | Expected dependency                                     | Preferred contract                                                                        |
  | ------------------ | ------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
  | Status             | Rendered `jj status` sections, working-copy file groups | Rendered output first; structured data if actions need precise file sets                  |
  | Operation log      | Rendered `jj op log` graph and operation ids            | Narrow parser for operation ids; typed API if recovery flows need transaction detail      |
  | Bookmarks          | Rendered bookmark lists and tracking markers            | Rendered output first; `jj_lib` or structured output if actions need exact tracking state |
  | File list          | File names from status, show, or diff output            | Structured output or narrow templates once file actions become mutation-capable           |
  | Resolve            | Conflict listing and file state                         | Typed API preferred before guided conflict resolution expands                             |
  | Rewrite flows      | Preview output plus command planning                    | Prefer `jj` command previews; avoid duplicating revset/fileset semantics                  |
  | Sync flows         | Fetch/push stderr/stdout and bookmark state             | Prefer command passthrough until structured state is needed                               |

## Duplication Watchlist

These areas are especially likely to drift if copied into `jk`:

- template parsing, rendering, and formatting;
- config interpretation, aliases, and style/color resolution;
- revset parsing and evaluation;
- fileset parsing and evaluation;
- graph layout, edge rendering, and glyph semantics;
- default row composition and user-configured view choices;
- semantic row fields paired with renderable styled segments;
- command planning, defaults, and safety checks;
- operation transaction behavior;
- conflict and merge state modeling;
- bookmark and remote-tracking semantics.

The adjacent `../jj` source currently exposes promising modules for several watchlist items:
`jj_cli::templater`, `jj_cli::commit_templater`, `jj_cli::formatter`, `jj_cli::graphlog`,
`jj_cli::revset_util`, and `jj_lib`'s repo/revset/graph/diff/transaction modules. Treat those as
preferred spike targets before copying behavior locally. Also treat them as provisional until the
spike proves the public surface is usable without depending on accidental internals.

If one of these areas becomes necessary, record the chosen path in the relevant screen or workflow
plan:

- use rendered output as-is;
- add a narrow parser;
- use a purpose-built template or structured output;
- use shared rendering, config, or template APIs;
- integrate through `jj_cli` or `jj_lib`;
- request or prototype an upstream API;
- accept temporary duplication with tests and an explicit removal path.

## Build-Break Preference

Prefer integration points that fail loudly when `jj` changes:

- compile errors from typed Rust APIs;
- schema or structured-output failures;
- snapshot diffs for rendered output that is intentionally parsed;
- focused parser tests around every soft agreement;
- explicit degraded behavior when parsing fails.

Avoid behavior that silently reconstructs a different repository model from the one `jj` would show
the user.
