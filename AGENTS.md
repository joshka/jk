# Repository Guidelines

## Project Structure & Module Organization

- `src/main.rs` is the current application entry point.
- Add new runtime code under `src/` with one concern per module file.
- Keep user-facing behavior and design notes in `README.md`.
- Put integration tests in `tests/` when features stabilize.
- Treat `target/` as generated output; never edit or commit it.

## Build, Test, and Development Commands

- `cargo build`: Compile the project in debug mode.
- `cargo run`: Build and launch the app locally.
- `cargo test`: Run unit and integration tests.
- `cargo fmt --all`: Format Rust code using `rustfmt`.
- `cargo clippy --all-targets --all-features -- -D warnings`: Run strict lint checks.
- `markdownlint-cli2 "*.md"`: Lint Markdown files (100-character line limit).

## Coding Style & Naming Conventions

- Follow Rust 2024 idioms and keep code `rustfmt` clean.
- Use `snake_case` for modules, files, functions, and variables.
- Use `UpperCamelCase` for structs, enums, and traits.
- Use `SCREAMING_SNAKE_CASE` for constants and static values.
- Prefer small, composable functions over large stateful blocks.
- Keep modules ordered and sized for maintainability; prefer readable files over large multi-purpose
  modules.
- Prioritize readability over cleverness and optimize for obvious, reviewable code paths.
- Write comments for intent and constraints, not obvious mechanics.

## Testing Guidelines

- Place unit tests near implementation with `#[cfg(test)]`.
- Use integration tests in `tests/` for command and workflow behavior.
- Use `insta` for TUI visual/snapshot tests to lock rendering and interaction output.
- Name tests by behavior, for example `opens_default_log_view`.
- Keep tests simple and obvious; if tests become complex, refactor production code to simplify.
- Add regression tests for parser, revset handling, and rendering fixes.
- Run `cargo test` before submitting changes.

## Documentation Expectations

- Treat this project as documentation-heavy: keep plans, command docs, and ADRs current as code
  evolves.
- Document architecture and tradeoffs in ADRs when introducing non-obvious decisions.
- Prefer high-quality documented code (module docs and focused comments) over implicit behavior.
- Write documentation as work progresses, not only at the end, to preserve context and rationale.

## Commit & Pull Request Guidelines

- Use `jj` commands for local workflow (for example `jj --no-pager status`).
- Keep changes focused and commit in small atomic chunks.
- Squash or amend only when it improves history clarity for reviewers.
- Use Conventional Commits for all commit titles (for example
  `feat(tui): add log navigation`).
- Keep the summary in imperative mood and concise.
- Make commit bodies the default, not the exception.
- In the body, explain what changed and why it matters so humans and agents can
  orient quickly from history.
- After any commit rewrite (amend, squash, rebase, describe), print the updated
  commit details in the result so it can be validated.
- For pull requests, include problem statement, approach, and verification commands.
- Add screenshots or terminal captures for TUI behavior changes.

## Execution Discipline

- After writing Markdown docs, run `markdownlint-cli2` before moving on.
- After writing Rust code, run `cargo fmt` and `cargo check` before further edits.
- Run targeted tests for newly implemented behavior first; expand to broader tests at checkpoints.
- Run `cargo clippy --all-targets --all-features -- -D warnings` at the end of each major
  checkpoint.
- Keep validation mostly automatic and prefer simple, explicit tests over complex abstractions.
- Re-read active plan files periodically and update implementation status as progress changes.

## Configuration & Safety Notes

- Keep default behavior aligned with `jj` config and `jj log` expectations.
- Avoid hardcoding machine-specific paths, usernames, or terminal assumptions.
