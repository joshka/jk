# README and UI Compaction Plan (2026-02-08)

## Goal

Make the app presentation calmer and easier to scan, and make README tell a clear user story
before technical details.

## UX/UI Changes

1. Unify header/footer chrome with muted colors (`dark gray` background, `white` foreground).
1. Remove repeated context in footer when header already shows active view/command.
1. Remove odd dark-gray heading/selection blocks by switching selection emphasis to
   marker + foreground style instead of full-row dark background.
1. Condense `:commands` help into a tighter two-column layout with less repeated labeling.
1. Condense `:keys` into a two-column layout with shorter action labels and reduced prefix noise.

## Documentation Changes

1. Rewrite README narrative to answer:
   - what problem `jk` solves,
   - why it helps `jj`/git users,
   - how daily workflows map to the interface.
1. Group user-facing capabilities by workflow instead of long flat feature lists.
1. Add narrative captions above tutorial media items.
1. Move implementation-heavy details to dedicated docs.
1. Keep contributor and architecture links visible but secondary.

## Extraction Targets

Move implementation-heavy sections into `docs/architecture.md`:

1. Architecture snapshot details.
1. Command entry internals.
1. Full implemented flow coverage matrix.
1. Alias coverage deep details.

## Validation and Artifacts

1. Add/refresh snapshots for condensed help/keymap output and log-selection behavior.
1. Run `cargo fmt --all`, `cargo check`, `cargo test`, strict clippy.
1. Run `markdownlint-cli2 "*.md" ".plans/*.md" "docs/**/*.md"`.
1. Regenerate VHS gifs and screenshots with updated pacing/theme.
