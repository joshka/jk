# Prompt Approach (One-Shot Template)

Use this prompt template to reproduce this implementation style in one run:

## Prompt template

Build a `jj`-style TUI tool named `jk` with `jk` itself as the default command (equivalent to
`jk log`). Do not create a separate `jk tui` command.

Requirements:

1. Keep users in `jk` for normal workflows: navigate revisions, select context, act in-place.
2. Prefer pager-style interaction in alt-screen + raw mode, minimal chrome, no box-heavy UI.
3. Default keybindings are vim-like and loaded from TOML with good defaults and user overrides.
4. Implement flows for `log`, `status`, `show`, `diff`, `new`, `describe`, `commit`, `next`,
   `prev`, `edit`, `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`, bookmark actions,
   and `git fetch`/`git push`.
5. Include aliases: `gf`, `gp`, `rbm` (target `main`), `rbt`, plus OMZ compatibility aliases.
6. Use explicit preview + confirmation for risky rewrite/remote actions.
7. Keep docs and implementation in sync continuously (plans, status tracker, ADRs).
8. Run validation continuously: markdown lint after docs, `cargo fmt` + `cargo check` after code,
   targeted tests first, `clippy -D warnings` at checkpoints.
9. Keep commits atomic with conventional commit headers and informative bodies.
10. If blocked, log blocker, apply workaround, continue.

Output expectations:

- A runnable baseline TUI.
- Gold command set tracked in a file.
- Implementation status file updated with concrete coverage.
- Blockers file documenting issues and workarounds.
- Concise summary of completion level and remaining gaps.
