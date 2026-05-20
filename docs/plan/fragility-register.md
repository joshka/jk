# Fragility Register

This register tracks places where `jk` depends on soft agreements with `jj`: rendered output shapes,
underspecified command behavior, inferred state, or duplicated logic. These assumptions are allowed
when they are narrow and useful, but they should be visible during planning and review.

## Current Code

  | Area                 | Current dependency                                               | Risk                                                                            | Mitigation                                                                                       |
  | -------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
  | Render pipeline      | stdout -> ANSI parser -> styled spans -> Ratatui items           | Each conversion can lose semantic structure or style intent                     | Keep conversion local; prefer shared rendering APIs if structure expands                         |
  | Semantic inference   | Meaning reconstructed from rendered text after `jj` had it       | `jk` can misread or discard meaning that code paths could preserve              | Prefer structured output, templates, `jj_cli`, `jj_lib`, or upstream APIs                        |
  | View decisions       | Row composition, templates, colors, graph, and labels            | `jk` can diverge from jj defaults or user configuration                         | Prefer APIs that expose semantic data and renderable view parts together                         |
  | Log row expansion    | Compact row re-parsed before richer inline detail is attached    | Expanded rows can disagree with the compact row or lose identity                | Prefer a row contract with ids, graph info, styled spans, and detail hooks                       |
  | Action selection     | Mutation inputs inferred from selected rendered rows             | Actions can target the wrong revisions or lose multi-row meaning                | Carry exact revision ids and graph relationships through guided flows                            |
  | Log selection state  | Exact change-id multiselect state kept separately from row index | Reordered rows and filtered views can still invalidate cached selected rows     | Store selected items by exact id, reconcile against current view on refresh/mode changes         |
  | Graph grouping       | Rendered `jj log` lines and graph glyph layout                   | Output shape changes can affect row selection or target extraction              | Keep the parser narrow; test elided, malformed, and non-revision rows                            |
  | Revision identity    | Template output for change and commit ids                        | Template behavior or fields may change                                          | Keep templates minimal; move to typed API if identity extraction expands                         |
  | Direct startup views | Local inference of `show` and `diff` targets from startup args   | Option edge cases can be misread                                                | Preserve raw args; test supported direct-view cases only                                         |
  | File headings        | Default and `--git` rendered diff/show headings                  | Heading changes can break sticky file context                                   | Preserve raw document lines; test default and git diff heading forms                             |
  | Sticky projections   | Active file inferred from scroll position and headings           | Unusual output can produce no active file                                       | Degrade to a plain scrollable document                                                           |
  | Diff format state    | Detecting and adding `--git` format arguments                    | Other diff tools or future formats may not match assumptions                    | Keep behavior explicit and local to diff/show command construction                               |
  | Operation-log rows   | Rendered `jj operation log` item starts and graph-only lines     | Output shape changes can split or merge visible operations                      | Keep parsing narrow; test multi-line rows, ANSI spans, and graph-only rows                       |
  | Operation-log ids    | Separate `self.id()` template output paired by row order         | Rendered rows and metadata rows can drift or differ under new args              | Keep both calls on `--at-op=@ operation log`; degrade to non-copyable rows                       |
  | Bookmark metadata    | Separate bookmark template output paired to rendered local rows  | Rendered rows and metadata rows can drift; remote rows have no ids              | Pair only non-indented local rows; keep remote/tracking state non-semantic                       |
  | File inspection      | Rendered `jj file list` rows and `jj file show` document output  | Label drift or heading shape changes can break copy, refresh, or sticky anchors | Keep exact paths separate from labels; preserve raw document lines; test path-preserving refresh |

## Planned Screens

Each planned screen should add entries here before implementation if it needs output parsing or
duplicated jj semantics.

Narrow machine-oriented templates are different from arbitrary user-facing rendered output. They are
acceptable when they expose explicit fields for `jk`; they should not become a second copy of jj's
template language or presentation model.

  | Screen or workflow | Expected dependency                                            | Preferred contract                                                                                                                   |
  | ------------------ | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
  | Status             | Rendered `jj status` sections, working-copy file groups        | Rendered output first; structured data if actions need precise file sets                                                             |
  | Operation log      | Rendered `jj op log` graph and operation ids                   | Narrow parser for operation ids; use `--at-op=@` to avoid view-time snapshots; typed API if recovery flows need transaction detail   |
  | Bookmarks          | Rendered bookmark lists and local/remote tracking markers      | Rendered output first; narrow local bookmark metadata now, and `jj_lib` or structured output if actions need exact tracking state    |
  | File list / show   | File list rows and file document output                        | Structured output or narrow templates once file actions need mutation-capable exact-path semantics                                   |
  | Resolve            | Conflict listing and file state                                | Typed API preferred before guided conflict resolution expands                                                                        |
  | Rewrite flows      | Preview output, action menu role prompts, and command planning | `jj` command previews can drift from direct shell inference; avoid executing rewrites without explicit preview and role confirmation |
  | Sync flows         | Fetch/push stderr/stdout and bookmark state                    | Prefer command passthrough until structured state is needed                                                                          |

## Source Integration Spike

Slice 0 showed that `jj_cli` is promising but not yet a complete external log-rendering contract.

  | Area                     | Current dependency or gap                                                                        | Risk                                                                       | Mitigation                                                                                           |
  | ------------------------ | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
  | Shared log rendering     | Public low-level `jj_cli` graph, formatter, and template pieces                                  | `jk` could still copy `cmd_log()` orchestration or depend on awkward setup | Keep Slice 1 on the narrowed subprocess path; only switch after a compiled in-repo adapter           |
  | Workspace/template setup | `CommandHelper` and `WorkspaceCommandHelper` methods without a simple external construction path | External integration may depend on public-but-not-convenient APIs          | Treat this as upstream API evidence; avoid production dependency until setup is cleaner              |
  | Style-to-span capture    | `FormatRecorder` exposes replay but not its recorded label ops                                   | `jk` may preserve text while still redoing style mapping locally           | Prefer a custom `Formatter` adapter over private-op inspection; request higher-level spans if needed |
  | Default log semantics    | Configured revset, graph priority, and template wiring live in CLI command flow                  | Local recreation can drift from `jj` defaults and user config              | Mirror only the current narrow metadata template; revisit shared rendering later                     |

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
