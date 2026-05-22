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

## Canonical Maintainability Guidance

Use the repo guidance in this order when maintainability, ownership, or structure matters:

- [`AGENTS.md`](AGENTS.md): entry-point rules, repo workflow, and which deeper docs to load.
- [`docs/agent/architecture.md`](docs/agent/architecture.md): canonical source of truth for current
  structure, ownership, feature roots, and shared-mechanics boundaries.
- [`docs/agent/rust-style.md`](docs/agent/rust-style.md): canonical Rust/module-shape guidance once
  the owner is known.
- [`docs/agent/workflow.md`](docs/agent/workflow.md): canonical implementation workflow,
  maintainability packet shape, completion bar, and the small current maintainability doctrine that
  future work should follow by default.
- [`docs/reference/README.md`](docs/reference/README.md): current product-facing reference entry
  point for screens, workflows, and the single-view model.

Rendered `jj` output is the default presentation source. Preserve user templates, colors, graph
symbols, diff style, wording, and command behavior wherever practical. Parse the minimum structure
needed for presentation-adjacent navigation, sticky file context, search, and copy actions, and keep
the rest opaque. Prefer code or structured contracts over parsed CLI output for semantic state.
Treat stdout -> ANSI -> styled spans -> Ratatui items as a fragile pipeline when the feature needs
semantics that could be represented directly in code. When work depends on underspecified output,
pipeline reconstruction, semantic inference from rendered text, or duplicated `jj` behavior, treat
that as evidence for stronger `jj`-side abstractions such as structured output, `jj_cli`, `jj_lib`,
shared rendering/config APIs, or future upstream UI-facing contracts. Record the soft agreement in
the owning docs, tests, or source comments instead of leaving it in chat context.

## Always-On Agent Guidance

Apply this section in every turn and in every subagent. It summarizes the repo-local shared guidance
copied into [`docs/development/`](docs/development/) so agents and GitHub readers do not depend on a
sibling checkout or generated scripts.

- Preserve unrelated human or agent work. Inspect the working copy before editing, avoid reverting
  unowned changes, and stop for direction if existing edits conflict with the task.
- Preserve intent over literal instructions. If the requested steps appear to miss the stated goal,
  choose the path that satisfies the goal and explain the tradeoff.
- Keep durable context on disk. Put repository-specific facts, assumptions, decisions, and follow-up
  notes in tracked docs or ignored `AGENTS.override.md` files rather than relying on chat context.
- Keep durable docs in this repo focused on current product, contributor, and maintenance guidance.
  Do not add packet logs, phase trackers, progress ledgers, or model/process experiments here.
- Prefer tools and checks over repeated prompting. If the same correction would be needed again,
  make it mechanical through code, tests, lint configuration, scripts, templates, or local guidance.
- Distill conventions from accepted artifacts before inventing style. Start from nearby code, tests,
  docs, snapshots, and existing architecture notes.
- Define the quality bar before judgment-heavy work such as naming, API shape, documentation
  structure, user-visible behavior, or rule wording.
- Identify the owning module before editing. Put behavior where the concept lives, not in the first
  caller, facade, helper, or test that mentions it.
- Keep changes minimal but complete. Use one purpose per change; treat `and` in a change description
  as a scope warning.
- Separate structure from behavior. Avoid mixing formatting, renames, dependency updates, rewrites,
  and semantic changes unless the coupling is necessary and documented.
- Prefer build-preserving edits. Make changes along natural paths and run focused validation early
  so failures stay close to their cause.
- Properly document and test implementation work. Record the behavior, constraints, and maintenance
  context needed to understand the change, and leave focused proof in the owning tests or docs
  instead of relying on chat context.
- Treat active guidance docs as part of the product boundary. When structure moves, update
  `AGENTS.md` and the relevant `docs/agent/*.md` files in the same change; historical ledgers may
  preserve old paths, but active guidance may not.
- Choose validation by risk. Match proof to the changed surface: parser samples for parsers,
  navigation boundaries for TUI movement, rendered output checks for presentation, and docs checks
  for documentation.
- Treat maintainability completion claims as a higher bar than green builds. Preserve both
  behavior-level proof for changed surfaces and durable ownership memory for what was intentionally
  left alone.
- Report proof in handoffs instead of confidence language. State what was run, what passed, what was
  not run, and any residual risk.
- Review output as a future maintainer. Check correctness, edge cases, API clarity, documentation
  truthfulness, ownership, test focus, validation evidence, and remaining risk before handoff.
- Keep secrets out of context, docs, logs, and tests. Redact or avoid exposing tokens, credentials,
  local private paths that are not needed, and sensitive command output.
- Make exec-like workflows noninteractive by default. Use `--no-pager`, avoid editor prompts, and
  keep commands suitable for agents, CI, and background tasks.
- Make ambient inputs explicit where they affect behavior: current directory, environment, terminal
  size, config, time, randomness, locale, network state, and process state.
- Push uncertainty to boundaries. Parse and validate external strings, CLI output, JSON, provider
  responses, and user input before passing trusted values inward.
- Treat terminal UI as a product surface. Preserve conservative terminal behavior, accessibility,
  platform variation, and user-configured `jj` presentation whenever practical.

## Shared Development Guidance

This repo carries a local copy of the reviewed `joshka/practice` development guidance in
[`docs/development/`](docs/development/). Use this repo's local rules first. When local guidance is
silent, use the copied shared guidance as the fallback. The public rendered reference is [Software
Practices](https://www.joshka.net/practice/), and the canonical source repository is
[`joshka/practice`](https://github.com/joshka/practice).

Load these files deliberately:

- [`docs/development/README.md`](docs/development/README.md): read when changing, refreshing, or
  explaining the copied shared guidance tree.
- [`docs/development/snippets/agents/rules.md`](docs/development/snippets/agents/rules.md): read at
  the start of broad, ambiguous, multi-domain, or high-risk agent work when a compact full rule
  surface is more useful than loading one domain.
- [`docs/development/rules/README.md`](docs/development/rules/README.md): read when choosing which
  domain rule files apply to a task.
- [`docs/development/bootstrap-downstream.md`](docs/development/bootstrap-downstream.md): read when
  installing, refreshing, or merging this guidance into another downstream repo.
- [`docs/development/update.py`](docs/development/update.py): inspect before changing the refresh
  workflow or running a guidance refresh from `joshka/practice`.

Read the domain files that match the task:

- [`docs/development/rules/agent-workflow.md`](docs/development/rules/agent-workflow.md): read for
  agent planning, delegation, context capture, handoffs, validation reporting, or repeated feedback
  that should become durable guidance.
- [`docs/development/rules/boundary.md`](docs/development/rules/boundary.md): read for parsing,
  validation, state ownership, lifecycle transitions, policy boundaries, provider/CLI integration,
  terminal behavior, or other effect boundaries.
- [`docs/development/rules/change-shape.md`](docs/development/rules/change-shape.md): read before
  shaping a change, splitting scope, touching generated artifacts, moving behavior between modules,
  changing dependencies, or mixing refactors with behavior changes.
- [`docs/development/rules/documentation.md`](docs/development/rules/documentation.md): read for
  README, Rustdoc, guides, examples, doc structure, claims about current behavior, or documentation
  review.
- [`docs/development/rules/observability.md`](docs/development/rules/observability.md): read when
  adding or changing diagnostics, errors, logs, degradation behavior, or failure visibility.
- [`docs/development/rules/performance.md`](docs/development/rules/performance.md): read when
  optimizing, changing hot paths, adding caches, making performance claims, or trading readability
  for speed.
- [`docs/development/rules/refactoring.md`](docs/development/rules/refactoring.md): read before
  refactoring, extracting abstractions, renaming concepts, moving code, or reducing duplication.
- [`docs/development/rules/review.md`](docs/development/rules/review.md): read when preparing a PR,
  responding to review, writing handoff notes, classifying prototype reuse, or deciding what belongs
  in issues, ADRs, or PR descriptions.
- [`docs/development/rules/rust.md`](docs/development/rules/rust.md): read for Rust API shape, trait
  design, generics, ownership, errors, features, dependencies, docs, examples, macros, unsafe, or
  public crate behavior.
- [`docs/development/rules/source.md`](docs/development/rules/source.md): read when gathering
  context, using generated/vendor files, relying on external sources, or deciding what evidence is
  authoritative.
- [`docs/development/rules/test-failures.md`](docs/development/rules/test-failures.md): read when
  writing assertions, improving failure messages, snapshotting output, or making tests explain the
  broken contract.
- [`docs/development/rules/testing.md`](docs/development/rules/testing.md): read when choosing
  validation, adding tests, changing parser samples, checking feature combinations, proving command
  construction, or updating CI expectations.
- [`docs/development/rules/vcs.md`](docs/development/rules/vcs.md): read before jj/git topology
  work, bookmarks, branching, rebasing, squashing, conflict recovery, publishing, or preserving
  unrelated working-copy edits.

Do not hand edit copied generated rule files unless intentionally forking this repo's guidance.
Prefer changing local project instructions in this file or the existing `docs/agent/` files. If a
shared rule causes friction or seems wrong for most projects, send feedback upstream instead of only
patching around it locally.

## Project Structure & Module Organization

Source lives in `src/`; tests are colocated in each module under `#[cfg(test)]`. The code is moving
toward feature roots plus shared infrastructure. Put each rule where a maintainer would look when
the user-visible concept changes: feature roots own view state, bindings, row interpretation,
selection/search/copy behavior, action availability, target resolution, and tests; shared modules
own only cross-cutting mechanics that two feature owners can use without understanding each other's
domain. Treat this as a feature-policy versus shared-mechanics split: feature roots answer what the
surface shows, selects, copies, recovers from, and offers; `tui`, `jj`, `actions`, `app`, and small
helpers answer boring cross-cutting questions after a feature has already chosen its policy.

Apply the project-structure rule from Ed Page's Rust style guidance when a module is split across
files: prefer a directory root (`foo/mod.rs`) over `foo.rs` plus `foo/`, and avoid `#[path]`. A
reader who sees `foo.rs` should be able to assume it is self-contained. A reader who sees `foo/`
should be able to treat the directory as the module unit.

For small or titular modules, `foo/mod.rs` may own the central type or function directly when that
is the clearest reader path. For larger modules, trend toward `mod.rs` as a table of contents with
definitions in named child files. Do not create `mod.rs` layers that hide feature policy or add
indirection without improving the reader path.

The code is organized by vertical slices where practical:

- `app/mod.rs` owns the terminal event loop, key dispatch, modal state, view stack, refresh, and
  cross-view transitions.
- `view_state/mod.rs` routes app-level view operations across the concrete feature views and keeps
  action-target delegation separate from feature-local policy.
- `command/mod.rs` owns key binding metadata and the command/effect vocabulary shared between app
  dispatch and individual views.
- `menus/mod.rs` owns shared menu vocabulary and prompt models. Feature roots own action
  availability and target selection.
- `jj/mod.rs` is the `jj` process and command-construction boundary. It owns `ViewSpec`, CLI process
  helpers, and `jj` syntax helpers while parsing only the minimal graph/revset metadata `jk` needs.
- `log/mod.rs` owns the default/log view, log-row selection, log search, and log-to-detail
  navigation.
- `show/mod.rs` and `diff/mod.rs` own their view policy and should stay distinct even when they
  share document mechanics.
- `documents/mod.rs` owns shared rendered document structure, show/diff/status/operation-detail
  document scrolling, file jumping, sticky heading projection, and document search.
- `rendered_rows/mod.rs` owns only shared rendered-row helpers; feature-specific row policy belongs
  in feature roots such as `log/rows.rs`, `bookmarks/rows/mod.rs`, and `operation_log/rows.rs`.
- `search/mod.rs`, `selection.rs`, and `clipboard.rs` own narrow support concepts and should not
  accumulate view policy. Copy-menu payload vocabulary now lives under `menus/model/copy.rs`.
- `terminal_process/mod.rs` owns inherited-stdio terminal suspension and restoration for interactive
  commands.
- `tui/mod.rs` owns shared chrome only: layout, status/header rendering, overlays, and modal
  presentation.

Avoid letting `jj`, `actions`, `rendered_rows`, `menus`, `tui`, or `view_state` become dumping
grounds for feature policy. Add a module only when it gives a real concept a local home. Avoid broad
reorganization unless it improves the reader path for a concrete change.

If a shared root still looks suspicious after review, do not split it by default. Record the no-move
decision near the source or in active guidance so future maintainers know why it stayed shared and
what policy does not belong there.

## Build, Test, And Development Commands

Use the repository `just` commands:

- `just check`: run nightly rustfmt, Panache Markdown checks, `cargo check`, `cargo test`,
  `cargo clippy -- -D warnings`, and the largest Rust source file report.
- `just packet-check`: run the clippy gate and largest Rust source file report.
- `just largest-rust-files`: print the top 20 Rust source files by line count.
- `just fmt`: run `rustup run nightly cargo fmt`.
- `just md-fmt`: run `panache format README.md AGENTS.md docs`.
- `just md-check`: run Panache format and lint checks for Markdown.
- `just test`: run `cargo test`.
- `just run`: run the TUI with `cargo run`.

Use `rustup run nightly cargo fmt` before finishing Rust changes. Markdown is formatted with
Panache, configured in [`panache.toml`](panache.toml) for GFM, 100-column reflow.

## Coding Style & Naming Conventions

Prefer feature-oriented modules over horizontal buckets. Use a plain `foo.rs` file only when `foo`
is self-contained. Once `foo` has child modules, prefer `foo/mod.rs` and decide whether the root
should hold the titular concept or only re-export named child modules based on reader locality.
Avoid `#[path]`, broad umbrella folders, and `mod.rs` layers that only reshuffle names.

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
- [`docs/reference/README.md`](docs/reference/README.md) for the durable current-state reference
  surface covering screens, workflows, and the view model.

## Testing Guidelines

Use Rust unit tests colocated with the module they describe. Prefer behavior-oriented test names,
for example `document_search_wraps_without_reselecting_current_line`.

Default to view-level tests for view behavior, especially when the contract includes both rendered
content and presentation. Prefer assertions and snapshots that show the user-visible result in a
maintainable way instead of only checking internal helpers.

Use inline insta snapshots for multi-line rendered/projection transitions. Run focused tests while
working and `just check` before handing off when practical.

Be aware that `NO_COLOR` or similar ambient environment can interfere with insta coverage or
ANSI-sensitive tests if it reaches `jj` calls. When a test depends on styled output, make the color
expectation explicit, verify both content and presentation at the appropriate level, and investigate
or correct color-handling leaks when snapshots stop exercising ANSI output.

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
prefer change ids from log rows; commit ids are exposed for copying.

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
