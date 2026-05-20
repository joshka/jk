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

## Slice 0 Result

The spike result is mixed:

- `jj_cli` is viable today for the low-level pieces `jk` needs to preserve view fidelity:
  `formatter::FormatRecorder`, `graphlog::{get_graphlog, GraphStyle}`, template renderers, and
  `revset_util` are all public and compile from an external crate.
- `jj_cli` is not yet a clean end-to-end external log-row renderer for `jk`. The actual `jj log`
  flow lives in `commands/log.rs`, but `LogArgs` and `cmd_log()` are `pub(crate)`. The workspace and
  settings path that makes template parsing and revset resolution convenient is routed through
  `cli_util::{CommandHelper, WorkspaceCommandHelper}`, whose public methods are useful but whose
  constructors are not public.
- `jj_lib` remains the reliable semantic source for repo, revset, graph, and commit identity, but
  using it alone would make `jk` reassemble more CLI behavior locally than this slice wants.

### Probe Evidence

This slice used two kinds of checks:

1. A temporary scratch crate with path dependencies on the adjacent `../jj/cli` and `../jj/lib`
   checkouts compiled and ran a minimal program using `jj_cli::formatter::FormatRecorder`,
   `jj_cli::graphlog::get_graphlog`, and `jj_lib::graph::GraphEdge`. The probe rendered a curved
   graph row as `@  change summary`, which matches the row prefix shape expected from `jj log`.
1. Subprocess comparisons in this repo confirmed that:
   - default `jj log` output keeps the compact configured row shape;
   - `--config='ui.graph.style="ascii"'` switches graph glyphs as expected;
   - a custom `-T 'change_id.shortest() ++ " " ++ description.first_line() ++ "\\n"'` template
     materially changes the row text shape that `jk` would need to preserve.

### Required Dependency Surface

The smallest promising source-backed surface for a future adapter is:

- `jj_cli::formatter::{FormatRecorder, Formatter}`;
- `jj_cli::graphlog::{get_graphlog, GraphStyle}`;
- `jj_cli::templater::TemplateRenderer`;
- `jj_cli::commit_templater` and `jj_cli::template_builder` via public template parsing helpers;
- `jj_cli::revset_util::RevsetExpressionEvaluator`;
- `jj_cli::cli_util::WorkspaceCommandHelper` methods for `settings()`, `repo()`,
  `commit_template_language()`, `parse_template()`, and `parse_revset()`;
- `jj_lib::repo`, `jj_lib::revset`, `jj_lib::graph`, `jj_lib::commit`, and `jj_lib::workspace`.

### Blocking Or Awkward Pieces

- `jj_cli::commands::log::{LogArgs, cmd_log}` are crate-private, so `jk` cannot reuse the whole
  command implementation directly.
- `CommandHelper` and `WorkspaceCommandHelper` expose helpful methods, but the setup path that
  produces them is not exposed as a simple external constructor.
- `FormatRecorder` is public, but its recorded label operations are private. Replaying into Ratatui
  spans would still need either a custom `Formatter` implementation in `jk` or a higher-level
  style/spans adapter from upstream.
- A `jk` adapter would still need to mirror some of `cmd_log()`'s orchestration: loading the
  workspace, resolving the configured default revset, parsing the template, prioritizing graph rows,
  and pairing rendered content with semantic commit identity.

### Recommended Next Step

Treat `jj_cli` as a promising rendering and formatting dependency, not yet as a drop-in replacement
for the current subprocess path.

For Slice 1, keep the narrowed subprocess-plus-template metadata approach and harden the row
contract there. Revisit the code-native path only after one of these becomes true:

- `jk` successfully compiles a small in-repo adapter that opens a workspace and drives the public
  `jj_cli` helpers without copying `cmd_log()` wholesale; or
- upstream exposes a higher-level UI-facing helper for workspace setup and log rendering.
