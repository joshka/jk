# Testing Guidance For Agents

Load this document when adding behavior, fixing bugs, changing parsers, touching navigation or
scroll state, updating rendering, or changing command construction.

## Testing Goal

Tests should prove user-visible contracts and local invariants. Avoid tests that only mirror
implementation details.

Good tests make the maintenance question obvious: if this behavior breaks, what would the user or
future maintainer notice?

## Where Tests Belong

Use colocated Rust unit tests under `#[cfg(test)]` for module behavior:

- graph row parsing and selection;
- command binding lookup;
- jj argument construction and label rendering;
- rendered line parsing;
- sticky file projection and scroll math;
- search wrapping and highlight behavior;
- copy option construction.

Use integration tests only when behavior crosses module or crate boundaries in a way that unit tests
would hide. This crate currently favors focused unit tests.

## Snapshot Tests

Use inline insta snapshots for multi-line rendered text, projection output, or UI-adjacent
structures where a plain assertion would be hard to read.

Snapshots should describe stable behavior, not incidental formatting churn. If a snapshot changes,
inspect whether the change reflects an intentional user visible result.

## Parser Tests

Parsers over rendered jj output should be tested with realistic samples. Cover both the recognized
shape and safe degradation:

- graph rows with change IDs and commit IDs;
- rows that should not be selectable;
- colorized or styled spans when style preservation matters;
- default diff file headings;
- git diff file headings;
- empty documents and documents with leading context;
- malformed or surprising lines that should be preserved without navigation.

Do not make parser tests broader than the parser's real contract. `jk` should recognize enough
structure to navigate and render, not claim to parse all jj output.

## Navigation And Scroll Tests

Navigation tests should cover boundary behavior:

- empty content;
- one item;
- first and last item;
- page movement with small viewports;
- search next and previous wrapping;
- refresh after content shrinks;
- sticky heading activation around blank separators;
- next-file and previous-file movement.

Use saturating expectations. Terminal UIs frequently run in small panes.

## Command Tests

Tests for `jj.rs` should prove command construction and display behavior:

- direct startup args are preserved;
- app labels use `jk` wording while command labels use `jj` wording;
- navigated detail views prefer change IDs;
- `--git` diff format is added, removed, and displayed intentionally;
- direct `show` revsets still work when no navigation target exists.

Avoid tests that require a real jj repository unless the behavior cannot be validated any other way.

## Validation Commands

Use focused checks while working:

```sh
cargo test <module_or_test_name>
```

Before handoff, run the project gate when practical:

```sh
just check
```

For Rust formatting-only validation:

```sh
just fmt
```

For Markdown formatting-only validation:

```sh
just md-fmt
```

For Markdown changes:

```sh
just md-check
```

If a check cannot be run, state that explicitly in the final note.

## Test Naming

Prefer behavior-oriented names:

```rust
document_search_wraps_without_reselecting_current_line
graph_navigation_uses_change_id_for_detail_views
refresh_clamps_selection_after_entries_shrink
```

Avoid names that only repeat the function under test:

```rust
test_next
test_parse
test_render
```
