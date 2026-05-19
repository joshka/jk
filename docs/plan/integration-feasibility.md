# Integration Feasibility

This document records what the adjacent `../jj` checkout currently exposes for a Rust TUI. It is not
a full source audit. It exists to keep integration planning grounded in real `jj` code instead of
assumptions.

## Source Files Checked

Useful starting points in `../jj`:

- `cli/src/lib.rs`: public `jj_cli` modules.
- `lib/src/lib.rs`: public `jj_lib` modules.
- `cli/src/config/revsets.toml`: built-in revset defaults and aliases.
- `cli/src/config/templates.toml`: built-in templates and template aliases.
- `cli/src/commands/log.rs`: log command flow, revset evaluation, templates, graph rendering, and
  diff-in-log composition.
- `cli/src/graphlog.rs`: graph row rendering over `jj_lib::graph::GraphEdge`.
- `cli/src/formatter.rs`: formatter labels, color handling, and `FormatRecorder`.
- `cli/src/templater.rs` and `cli/src/commit_templater.rs`: template renderer and commit template
  language support.
- `cli/src/revset_util.rs` and `lib/src/revset.rs`: user revset parsing, resolution, and evaluation.

## Confirmed Defaults

These defaults should shape `jk` rather than being re-invented locally:

- Default log revset: `present(@) | ancestors(immutable_heads().., 2) | trunk()`. Planning
  implication: default mode should preserve the configured `jj log` view instead of replacing it.
- Graph priority: `present(@)`. Planning implication: log ordering around the working copy is a jj
  decision, not a jk decision.
- Trunk alias: configurable `trunk()` resolving common remote main/master/trunk names and `root()`
  fallback. Planning implication: direct `jj new trunk` must validate the resolved target before
  running.
- Current-stack style defaults: `reachable(@, mutable())` for arrange/fix/sign-like commands.
  Planning implication: useful evidence for a current-stack mode, but not automatically the default
  log mode.
- Log template: `templates.log = builtin_log_compact`. Planning implication: compact log should use
  configured jj rendering where possible.
- Show template: `templates.show = builtin_log_detailed`. Planning implication: expanded detail can
  start from existing show-style rendering.
- Operation log template: `templates.op_log = builtin_op_log_compact`. Planning implication:
  operation screen should preserve configured operation rendering.
- File list template: `format_path(path) ++ "\n"`. Planning implication: exact paths should come
  from structured/template data before file mutation actions.

## Candidate Code Paths

`jj_cli` currently exposes modules that look directly relevant to `jk`:

- `commit_templater`, `templater`, and `template_builder` for interpreting configured templates;
- `formatter` for labels, style decisions, `FormatterFactory`, and `FormatRecorder`;
- `graphlog` for graph row rendering over typed graph edges;
- `revset_util` for command-like revset parsing and evaluation;
- `ui`, `cli_util`, and `commands` for command behavior, config, and workspace setup.

`jj_lib` exposes lower-level repository semantics:

- `repo`, `workspace`, `view`, `transaction`, and `working_copy`;
- `revset`, `fileset`, `diff`, and `graph`;
- `operation`, `op_store`, `op_walk`, `refs`, and `git` feature APIs.

The most promising near-term path is a source integration spike that tries to render log rows using
the same revset, template, formatter, and graph code paths as `jj log`, while replacing terminal
stdout with a recorder that can produce Ratatui spans and semantic row metadata.

## Risks And Unknowns

Public Rust modules do not automatically mean stable external UI APIs. `jj_cli` exposes many useful
modules, but those modules may be public for internal crate organization, tests, or the `jj` binary
rather than as a long-term compatibility promise. Depending on them can produce the desired
build-break behavior when `jj` changes, but it may also make `jk` sensitive to normal upstream
refactoring.

Known risks:

- `jj_cli` APIs may change more often than `jj_lib` APIs.
- Command behavior in `jj_cli::commands` may be hard to reuse without adopting the full CLI command
  environment.
- The current graph renderer emits strings through `Write`; preserving graph semantics for row
  selection may need an adapter or upstream extraction.
- Template rendering can preserve labels and text through `FormatRecorder`, but `jk` still needs a
  clean conversion from formatter labels to Ratatui spans.
- `jj_lib` can provide strong semantic state, but using it alone risks duplicating CLI defaults,
  config, templates, graph presentation, and user-facing command behavior.

## Recommended Spike

Before expanding parser-dependent log actions, run a focused source integration spike:

1. Build a minimal internal adapter that can render a small log through `jj_cli` template and graph
   paths without spawning `jj`.
1. Capture both semantic commit identity and rendered styled segments for each row.
1. Compare the output to subprocess `jj log` for a small repository and at least one custom
   template/color setting.
1. Record which parts compile cleanly, which require copying or private assumptions, and which would
   benefit from upstream extraction.

Acceptance criteria:

- the spike can say whether `jj_cli` is viable for log rows now;
- the spike identifies the exact dependency surface it would add to `jk`;
- any remaining stdout/ANSI parsing fallback is narrower and documented;
- the result updates the fragility register and the log screen plan.

Recommendation: run this spike before making the log row parser support richer mutations. Do not
block fetch, refresh, basic view modes, or read-only rendered-output screens on the spike.
