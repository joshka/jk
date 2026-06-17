# JJ Integration Cleanup

## Why

The MVP uses a temporary stdout/template bridge to get both CLI-equivalent rendered output and
semantic records. That is acceptable for the first slice only if the boundary stays narrow and the
missing upstream contract remains explicit.

## Cleanup Targets

1. Keep rendered output handling isolated from semantic record parsing.
1. Preserve user templates, graph symbols, color behavior, and revset defaults.
1. Avoid teaching the TUI to reimplement `jj` display decisions.
1. Replace shell-out or template parsing when `jj-cli` / `jj-lib` exposes a stable better contract.

## Current Blocker

`jk` needs two things from the log path:

1. The exact bytes `jj` would render for the current config, template, colors, graph style,
   terminal width, revset defaults, and default command behavior.
1. Semantic records for navigation, expansion state, future diff/show targeting, and stable row
   identity.

The first item currently lives in `jj-cli`, not `jj-lib`. The `jj-cli` log implementation composes
workspace loading, config, aliases, default command resolution, revset parsing, template parsing,
graph rendering, diff rendering, formatter/color behavior, pager decisions, and warnings. Those
pieces are real product behavior, not incidental output formatting.

The reusable `jj-lib` layer owns the repository model and core query machinery, but not the CLI log
contract. Using `jj-lib` directly for `jk log` would avoid parsing process output, but it would also
force `jk` to duplicate a large part of `jj log` selection, template, graph, and presentation
behavior.

Using `jj-cli` as a library is closer, but today the useful log entry point is still command-shaped
and terminal-oriented. The log command writes through `Ui`, its args and helper path are not exposed
as a stable external "build log rows" API, and a full `CliRunner` run returns rendered terminal
output rather than semantic records.

## Integration Options

### Keep The Shell-Out Bridge

This is the current MVP shape. It delegates the full rendered surface to the installed `jj` binary
and uses a narrow semantic template pass for the records `jk` needs internally.

This is unattractive, but it is honest: it preserves jj's config/default-command/template/color
behavior without pretending that `jk` already owns that contract.

Risks to manage:

- The semantic template pass can drift from the rendered pass.
- Parser assumptions need realistic fixtures and clear failure messages.
- Process invocation means `jk` depends on the installed `jj` binary and its version.

### Build Directly On `jj-lib`

This would give `jk` typed access to repository objects, revsets, and graph inputs, but it would not
give `jk` the configured CLI log view. It is the wrong first replacement if the product requirement
is "look like `jj`".

This path only makes sense after the semantic needs exceed what the current bridge can support, or
after jj moves more reusable log machinery into `jj-lib`.

### Run `jj-cli` In Process

This could remove the external process dependency, but it does not automatically solve the main
shape problem. A command-runner integration still returns rendered bytes, still needs a controlled
output sink and terminal width, and still does not expose semantic rows.

This is worth a spike only if the goal is to measure whether `jj-cli` can replace process spawning
without changing product behavior.

### Extract Or Upstream A Log Core

The desirable end state is a jj-owned API that performs log selection and rendering setup once, then
lets callers consume both presentation and semantic row events. That could start in `jj-cli` and move
structured pieces into `jj-lib` later.

A useful API would need to own:

- default command and log revset resolution;
- workspace/config/template setup;
- graph row construction and synthetic elision nodes;
- formatter/color behavior with an injectable output sink and width;
- a callback, iterator, or model that exposes stable commit row identity alongside rendered bytes.

This is likely an upstream-shaped change because jj's roadmap already calls out better Rust APIs for
UIs and reducing duplicated CLI logic in external tools.

## Roadmap

1. Keep the current shell-out bridge through the log-first MVP and the 0005 hardening boundary.
   Treat it as the compatibility adapter for jj's current CLI behavior, not as the desired design.
1. Make the bridge safer before broadening scope:
   - keep rendered output opaque;
   - keep semantic parsing narrow;
   - add realistic fixtures for the rendered and semantic alignment assumptions;
   - document failures in terms of jj version/config drift.
1. Do a short `jj-cli` spike after the MVP boundary:
   - add a temporary local/path dependency to jj in a scratch change;
   - try to run the log command in process with controlled output, color, and width;
   - record which APIs are public, which are terminal-bound, and which require copying logic.
1. If the spike shows the only missing piece is output capture, consider a small upstream request for
   a testable/capturable `Ui` or formatter sink.
1. If the spike shows semantic row access requires copying `cmd_log`, stop and draft an upstream API
   proposal for a reusable log core instead of reimplementing jj log in `jk`.
1. Revisit replacing shell-out only when the replacement reduces duplicated jj behavior. A cleaner
   dependency graph is not enough if it makes `jk` responsible for matching jj's display contract.

## Questions To Answer

- What exact data does `jk` need for log navigation, expansion, and diff targeting?
- Which fields are semantic state, and which bytes are presentation that should stay opaque?
- Can one `jj` call provide both renderable output and semantic records without drift?
- Where should parser fixtures live so future `jj` output changes fail clearly?
- Can `jj-cli` expose a log core that emits semantic row events while still owning rendering?
- Which part belongs in `jj-cli` first, and which part should eventually move to `jj-lib`?

## Done When

- The current integration boundary is documented in code or tests, not only in plans.
- Parser assumptions are covered by realistic fixtures.
- Any move toward `jj-cli` / `jj-lib` reduces duplicated display behavior rather than hiding it.
- The post-MVP spike has either a narrow in-process path or a written upstream API proposal.
