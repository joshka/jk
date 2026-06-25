# Repository Guidelines

## Project Structure & Module Organization

This is a Rust 2024 workspace for `jk`, a jj-native terminal UI for
Jujutsu. The root `Cargo.toml` is workspace-only. The publishable
crates live under `crates/`:

- `crates/jk`: binary crate and current default workspace member.
- `crates/jk-core`: shared log records.
- `crates/jk-cli`: temporary `jj` process integration.
- `crates/jk-tui`: Ratatui state and rendering.

Release and CI automation lives in `.github/`, `release-plz.toml`,
`cliff.toml`, `deny.toml`, and `scripts/package-release-archive.sh`.
Short-term planning notes live in `.plans/`.

## Build, Test, and Development Commands

Use `just` for local tasks:

```sh
just --list
just check
just test
just clippy
just udeps
just lint-md
just release-check
```

`just release-check` is the broad local gate: formatting, check,
tests, clippy, unused dependency checks, docs, packaging, install
smoke, and Markdown lint. `just build-release <target>` and
`just package-release-archive <target> <version>` support release asset
testing.

## Coding Style & Naming Conventions

Rust formatting uses `rustfmt.toml` with unstable nightly rustfmt
options for comment wrapping, doc-comment formatting, import grouping,
and macro matcher formatting. Run the repo task before finishing Rust
changes:

```sh
just fmt
```

Use `just fmt-check` when checking formatting without editing. Both
tasks intentionally call `cargo +nightly fmt --all` so local formatting
matches CI and the full workspace is covered.

Rust comments and Rustdoc should be reflowed to the configured
100-character comment width. Do not hand-wrap Rust comments at 72 or 80
characters unless a narrower shape is needed for readability.

Keep Rust module names lowercase with underscores. Prefer clear, small
modules over broad utility buckets, stacked in abstraction layers where
each module owns one coherent concept. Avoid `unsafe`; the workspace
forbids it. Markdown uses `markdownlint-cli2`, 100-character prose, and
aligned tables.

Document Rust modules, items, and meaningful helpers by intent. Public
Rustdoc should describe the caller-facing contract. Private helper docs or
comments should explain ownership, invariants, side effects, or why the helper
exists, not restate the implementation. Binary crate roots should still have
crate-level `//!` docs when the package publishes docs.rs metadata.

## Testing Guidelines

Use Rust unit tests close to the module they describe. Name tests by
behavior, such as `refresh_keeps_selected_change_when_still_visible`.
Run focused tests while editing and `just release-check` before
release-oriented changes.

## TUI App Layout Guidelines

Default to user comprehension over implementation structure. Screens, overlays,
hotbars, menus, previews, and other discretionary layout should read like
user-facing product surfaces, not debug views of internal enums or dispatch
order.

- Group actions and information by user task, not implementation type. Prefer
  broad groups such as Open and inspect, Move and find, Change actions, History
  and recovery, and Session.
- Keep labels concrete without repeating the same idea twice. Use a command
  name when the command is the clearest label, or use an action label when the
  action is clearer; avoid command-plus-parenthetical forms that restate each
  other.
- Align columns globally within a surface. Key, action, object, status, and
  command columns should share column stops rather than jittering per section.
- Avoid accidental horizontal spread. Multi-column layouts are useful only when
  related columns remain visually connected by a deliberate gap.
- Size overlays and modals to rendered content with minimal padding. Avoid fixed
  large dialog dimensions when the content is smaller. Scrollable document
  overlays should size from the full rendered document, not from the current
  scroll slice, so the box does not resize while scrolling.
- Use available space before scrolling when it improves scanning. Show all
  relevant content when the terminal can fit it cleanly, but width-constrained
  document overlays may keep a stable viewport instead of expanding into a tall
  sheet. Show scroll indicators only when content is hidden.
- Match scrolling to the surface. Document-like content should scroll by
  rendered line; selection-style movement belongs to actual selectable rows.
- Do not repeat controls between body, hotbar, footer, and chrome. Chrome should
  add only information not already carried by the active surface, except when
  the visible chrome belongs to the underlying screen and remains useful for
  mode safety.
- Prefer readable key notation. Use symbols where they reduce noise, such as
  `↑/↓` and `←/→`, and split overloaded bindings when they represent different
  actions.
- Protect keys, action text, object labels, and status text from wrapping and
  truncation. At narrow widths, fall back to shorter labels or fewer columns
  before allowing awkward wrapping.
- Choose column count from content shape, not width alone. A two-column layout
  should activate only when both columns fit, the gap is readable, vertical
  scrolling is materially reduced, and the result still scans top-to-bottom.
- Test layout as data. Unit tests should cover representative sizes and assert
  width bounds, scroll indicators, column switching, readable compact layouts,
  and consistent alignment.
- Use PNG or terminal visual proof for polish and gestalt. Mechanical failures
  such as overflow, stale scroll indicators, bad column switching, and unwanted
  truncation belong in tests.

## Commit & Pull Request Guidelines

Use plain imperative commit summaries, not conventional commits. Keep
summary lines short, then wrap every commit-message body line at 72
columns, including multiline `jj desc` descriptions created during local
work. Pull requests should explain user-visible behavior, list
validation run, and call out release or crates.io impact. Link related
issues when they exist. Include terminal screenshots only for meaningful
TUI rendering changes.

## Security & Release Notes

Do not add long-lived crates.io tokens to workflows. The crates are
bootstrapped; future publishing should use crates.io trusted publishing
through the `release-plz.yml` workflow and `release` environment.
