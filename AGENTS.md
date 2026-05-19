# Repository Guidelines

## Current State

This is the cleaned-up `main` line for `jk`, a Rust 2024 binary crate for a ratatui-based TUI over
`jj`. The old broad prototype line is preserved as the `prototype` branch locally and on GitHub;
mine it for product ideas and visual references, not for code architecture.

The current product direction is log-first, single-active-view, vimish, and `jj`-shaped. Load
[`docs/product-direction.md`](docs/product-direction.md) when work affects user-visible scope,
navigation model, command coverage, or visual direction. Load
[`docs/agent/architecture.md`](docs/agent/architecture.md) when work touches command execution, view
behavior, rendering, navigation, search, copying, or terminal lifecycle.

Rendered `jj` output is the default presentation source. Preserve user templates, colors, graph
symbols, diff style, wording, and command behavior wherever practical. Parse the minimum structure
needed for presentation-adjacent navigation, sticky file context, search, and copy actions, and keep
the rest opaque. Prefer code or structured contracts over parsed CLI output for semantic state.
Treat stdout -> ANSI -> styled spans -> Ratatui items as a fragile pipeline when the feature needs
semantics that could be represented directly in code. When work depends on underspecified output,
pipeline reconstruction, semantic inference from rendered text, or duplicated `jj` behavior, load
[`docs/plan/integration-strategy.md`](docs/plan/integration-strategy.md) and record assumptions in
[`docs/plan/fragility-register.md`](docs/plan/fragility-register.md).

## Project Structure & Module Organization

Source lives in `src/`; tests are colocated in each module under `#[cfg(test)]`. The code is
organized by vertical slices where practical:

- `app.rs` owns the terminal event loop, key dispatch, modal state, view stack, refresh, and
  cross-view transitions.
- `view_state.rs` routes app-level view operations to graph, show, and diff view implementations.
- `command.rs` owns key binding metadata and the command/effect vocabulary shared between app
  dispatch and individual views.
- `jj.rs` builds `jj` commands, owns `ViewSpec`, loads rendered CLI output, and parses only the
  minimal graph/revset metadata `jk` needs.
- `graph.rs` owns the default/log graph view, graph-row selection, graph search, and graph-to-detail
  navigation.
- `show.rs` and `diff.rs` own their view policy and should stay distinct even when they share
  document mechanics.
- `sticky_file_view.rs` owns shared show/diff document scrolling, file jumping, sticky heading
  projection, and document search.
- `rendered_jj.rs` owns lightweight structure over rendered jj lines, including file heading
  detection and sticky projection inputs.
- `search.rs`, `selection.rs`, `copy.rs`, and `clipboard.rs` own narrow support concepts and should
  not accumulate view policy.
- `tui.rs` owns shared chrome only: layout, status/header rendering, overlays, and modal
  presentation.

Add a module only when it gives a real concept a local home. Avoid broad reorganization unless it
improves the reader path for a concrete change.

## Build, Test, And Development Commands

Use the repository `just` commands:

- `just check`: run nightly rustfmt, Panache Markdown checks, `cargo check`, and `cargo test`.
- `just fmt`: run `cargo +nightly fmt`.
- `just md-fmt`: run `panache format README.md AGENTS.md docs`.
- `just md-check`: run Panache format and lint checks for Markdown.
- `just test`: run `cargo test`.
- `just run`: run the TUI with `cargo run`.

Use `cargo +nightly fmt` before finishing Rust changes. Markdown is formatted with Panache,
configured in [`panache.toml`](panache.toml) for GFM, 100-column reflow.

## Coding Style & Naming Conventions

Prefer feature-oriented modules over horizontal buckets. Use named files such as `graph.rs` and
`show.rs`; avoid `mod.rs` re-export layers unless there is a strong reason.

Write Rustdoc/module comments for durable intent: jj CLI compatibility, navigation policy, sticky
scroll behavior, and other non-obvious constraints. Avoid comments that restate simple code. Keep
visibility narrow. Do not introduce `pub(crate)`, `pub(super)`, or `pub(in ...)` unless there is a
concrete need and no cleaner local structure.

When writing Rust, prefer idiomatic readability:

- inline `format!` arguments when possible;
- collapse nested `if` statements when that improves clarity;
- use method references over redundant closures when practical;
- avoid boolean or ambiguous `Option` parameters that make call sites opaque;
- prefer exhaustive `match` statements where the domain is known;
- avoid helper functions or abstractions that are used once and do not name a real concept.

Use the deeper agent guidance when the change touches the relevant area:

- [`docs/agent/architecture.md`](docs/agent/architecture.md) for app shape, view ownership, jj
  command boundaries, and rendering assumptions.
- [`docs/agent/rust-style.md`](docs/agent/rust-style.md) for local Rust style, API shape,
  visibility, naming, and abstraction choices.
- [`docs/agent/documentation.md`](docs/agent/documentation.md) for Rustdoc, comments, README-style
  prose, and truthfulness about current behavior.
- [`docs/agent/testing.md`](docs/agent/testing.md) for unit tests, snapshots, command parsing tests,
  and validation expectations.
- [`docs/agent/workflow.md`](docs/agent/workflow.md) for agent workflow, review posture, and handoff
  notes.

## Testing Guidelines

Use Rust unit tests colocated with the module they describe. Prefer behavior-oriented test names,
for example `document_search_wraps_without_reselecting_current_line`.

Use inline insta snapshots for multi-line rendered/projection transitions. Run focused tests while
working and `just check` before handing off when practical.

For Markdown-only changes, run `just md-check`. For Rust formatting-only validation, run `just fmt`.

## Commit, Branch, And Pull Request Guidelines

This repository uses jujutsu. Prefer `jj --no-pager` commands for version-control inspection. Do not
use Git for normal source-control workflows in this repo unless the operation is transport-level and
jj does not cover it.

Current branch topology:

- `main` is the cleaned-up implementation line and GitHub default branch.
- `prototype` preserves the old broad prototype branch for context mining.

For separable work, start from a fresh jj working-copy change and describe it early:

```sh
jj --no-pager new
jj --no-pager desc --message "Update agent repository guidance

Refresh the repo-local AGENTS guidance so future work starts from the
current product direction, tooling, branch topology, and module ownership."
```

Commit descriptions should be imperative and concise. Pull requests should summarize user-visible
behavior, note jj command/config assumptions, and list the validation run. Include terminal
screenshots only for meaningful TUI rendering changes.

## Product And Architecture Notes

`jk` intentionally starts from shelling out to `jj` and presenting rendered jj output. This
preserves user config, templates, colors, graph symbols, and jj CLI behavior. Navigation should
prefer change ids from graph rows; commit ids are exposed for copying.

Treat integration choices as theory testing. Rendered output is preferred first for presentation,
but semantic state should prefer structured or code contracts. Fragile parser assumptions, repeated
duplication, or workflows that need exact transaction semantics are evidence for stronger contracts
such as structured output, `jj_cli`, `jj_lib`, future RPC APIs, upstream extraction, or in-tree
work.

Prefer APIs that expose both semantic fields and renderable view information when possible. `jk`
should preserve jj-like defaults and user-configured templates/colors without reparsing terminal
output or duplicating jj's display decisions.

Keep the product focused on the core loop before expanding command coverage:

1. Graph navigation.
1. `show` and `diff` drill-down.
1. Back/forward history.
1. Refresh-in-place.
1. Search and copy.
1. Sticky file context.
1. Compact help/keymap discovery.
1. Focused status and operation-log views.

When mining the `prototype` branch or `target/vhs` artifacts, preserve useful interaction ideas such
as item-based navigation, low chrome, safety prompts, and compact keymap/help views. Avoid
inheriting pane-first layout, command launcher scope, generated tutorial scope as roadmap, or old
module boundaries.
