# Repository Guidelines

## Project Structure & Module Organization

- Core runtime lives in `src/`.
- Keep modules focused by responsibility:
  `src/app/` (runtime/input/history + rendering), `src/flow/` (command planning + prompts),
  `src/alias/` (alias normalization + catalog), and `src/commands/` (registry + safety/overview).
- Keep rendering command-aligned under `src/app/view/` (for example status/diff, file/tag,
  workspace/git/top-level mutation, and operation views).
- Prefer command-aligned files and co-located tests; avoid catch-all files.
- Follow size targets for maintainability: soft 300 LOC, hard 500 LOC (tests excluded).
- Default keybindings are in `config/keybinds.default.toml`.
- Planning and delivery context belongs in `.plans/`.
- ADRs and architecture notes belong in `docs/adr/`.

## Build, Test, and Development Commands

- `cargo run`: launch `jk` locally.
- `cargo fmt --all`: format Rust code.
- `cargo check`: fast compile validation.
- `cargo test`: run test suite.
- `cargo clippy --all-targets --all-features -- -D warnings`: strict lint gate.
- `markdownlint-cli2 "*.md" ".plans/*.md" "docs/**/*.md"`: lint Markdown with 100-char wrapping.

## Coding Style & Naming Conventions

- Follow Rust 2024 idioms and keep changes `rustfmt` clean.
- Use `snake_case` for functions/modules/files and `UpperCamelCase` for types.
- Prefer small, readable modules and obvious control flow over clever abstractions.
- Maintain a pager-first TUI design: minimal chrome, no box-heavy dashboard patterns.
- Treat maintainability and readability as primary design constraints.

## Testing Guidelines

- Put focused unit tests near implementation (`#[cfg(test)]`).
- Use `insta` snapshots for visual and wrapper rendering regressions.
- Prefer simple WET tests with clear names over complex test abstractions.
- If a test becomes hard to understand, refactor production code to simplify behavior.

## Commit & Pull Request Guidelines

- Use `jj` for version control (`jj --no-pager ...` for output commands).
- Use Conventional Commit headers (for example: `feat(flow): default list-first views`).
- Commit messages should be imperative, concise in the title, and include a body by default.
- In the body, explain why the change exists and what behavior it affects.
- Keep commits atomic; split oversized work into logical chunks.
- After any rewrite (`describe`, `squash`, `rebase`), print updated commit details for validation.

## Documentation & Execution Discipline

- This project is documentation-heavy: update docs and plans as you implement.
- Before implementation starts from a Plan Mode proposal, write the full execution plan to a
  dedicated file under `.plans/` and record a handoff entry in
  `.plans/implementation-status.md`.
- Lint Markdown immediately after writing docs.
- After code changes, run `cargo fmt --all` and `cargo check`.
- Run targeted tests for changed behavior first, then full `cargo test` + strict clippy at
  checkpoints.
