# jk

`jk` is a jj-native terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

It keeps the parts of `jj` you already trust: the graph, colors, wording, revsets, templates, and
diff output still come from `jj`. `jk` adds an interactive review loop around that output so you can
keep context open while an editor, shell, or coding agent changes the repository.

![jk log view](https://www.joshka.net/jk-screenshots/assets/jk-log-v3.gif)

## Project Status

`jk` is early, useful for a narrow inspection loop, and moving quickly.

Today it is mostly a refreshable `jj log` / `jj diff` workflow: keep context open while an editor,
IDE, or coding agent changes the repository, refresh in place, and inspect selected changes without
losing your place.

The longer-term direction is broader. `jk` should accept the command shapes and flags you would
normally pass to `jj`, preserve your jj config, and add interaction where it helps: selected
objects, prompts, confirmation for risky changes, command output, refresh, and operation recovery.

This is not ready for broad adoption yet. Feedback is most useful when it is about product
direction, current inspection workflows, visual and accessibility problems, large graphs, multiple
workspaces, narrow terminal splits, or how close a TUI should stay to jj's command and config
model.

## Installation

Install with Homebrew:

```sh
brew trust --formula joshka/tap/jk
brew install joshka/tap/jk
```

The trust command scopes Homebrew's non-official tap trust to the `jk` formula.

Install prebuilt release assets with [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):

```sh
cargo binstall jk
```

Or build from the crates.io source package:

```sh
cargo install jk --locked
```

## What Works Today

`jk` currently focuses on the inspection loop:

```text
log -> details -> diff -> back -> refresh
```

The supported surface includes:

- a full-screen log view backed by `jj`;
- explicit `jk log` and default-command `jk` entry points;
- manual refresh without leaving the TUI;
- movement by change, page, and edge;
- inline expansion of the selected change description;
- selected-change diff inspection with `jk diff [REVISION]` or `d` from the log;
- return from diff to log without losing log selection;
- line, page, file, hunk, and horizontal movement in the diff view;
- file folding, fold-all/unfold-all, and hunk folding;
- sticky current-file context with diff stat suffix and `[file x/y]`;
- diff search with `/`, `n`, and `N`;
- retryable empty/error states for selected diffs;
- mode-specific help overlays with `?`;
- [Betamax](https://www.joshka.net/betamax/) visual tapes for the log and diff workflows.

The current implementation intentionally treats rendered `jj` output as the source of truth. The
TUI parses only enough structure to support navigation, search, sticky headers, and folding.

## Diff Review

The diff view preserves `jj diff` output while adding review controls around it.

![jk diff view](https://www.joshka.net/jk-screenshots/assets/jk-diff-v3.gif)

Useful bindings in the diff view:

| Key                           | Action                       |
| ----------------------------- | ---------------------------- |
| `j` / `k`                     | scroll one line              |
| `Space` / `b`                 | page down / page up          |
| `g` / `G`                     | jump to top / bottom         |
| `[` / `]`                     | previous / next file         |
| `{` / `}`                     | previous / next hunk         |
| `h` / `l`                     | fold / unfold current file   |
| `Ctrl-Left` / `Ctrl-Right`    | fold / unfold all files      |
| `-` / `+`                     | fold / unfold current hunk   |
| `<` / `>`                     | horizontal scroll            |
| `/`, `n`, `N`                 | search, next match, previous |
| `r`                           | refresh                      |
| `H` / `L`                     | return to the log            |
| `?`                           | show mode-specific help      |
| `q` / `Esc`                   | close help, then quit        |

## Commands

```sh
jk
jk log
jk diff
jk diff <revision>
jk -R /path/to/repo -n 20
jk log --repository /path/to/repo --limit 20
```

Bare `jk` follows `jj`'s configured `ui.default-command` when that command is log-like enough for
the semantic navigation pass. Use `jk log` when you want the explicit log command path.

## Current Focus

The detailed product and engineering plan lives in
[docs/product-plan.md](docs/product-plan.md). The shorter
[docs/roadmap.md](docs/roadmap.md) turns that plan into issue-sized milestones.

The active planning and dogfood work is concentrated around the 0.3 to 0.6 foundations:

- keep `jj` output and config fidelity at the center: revsets, templates, graph style, colors, and
  diff output should still matter;
- build the command-spec, global-options, view-stack, mode-stack, keymap, and task foundations that
  make later workflows reviewable;
- improve inspection around `log`, `diff`, `show`, `status`, `evolog`, View Options, and file
  navigation without turning the default screen into a dashboard;
- dogfood command history, `:` command mode, workspace views, operation recovery, and safe mutation
  previews before presenting them as stable user workflows;
- replace rough UI experiments, including selection and highlight behavior, with more readable and
  accessible defaults;
- make [Betamax](https://www.joshka.net/betamax/) tapes the source for regression tests,
  README/site media, and release demos.

Some of that work exists only in local dogfood spikes today. The README describes the released
surface first; the planning docs describe the direction.

## Development

This repository uses `jj` for version control and `just` for local tasks.

```sh
just --list
just check
just test
just clippy
just lint-md
```

Visual README media is generated with:

```sh
just readme-media
```

The generated screenshots and GIFs live in the separate
[`joshka/jk-screenshots`](https://github.com/joshka/jk-screenshots) repository and are served from
`https://www.joshka.net/jk-screenshots/`.

## Crates

The workspace is split into narrow crates:

- `jk`: binary crate and terminal lifecycle;
- `jk-cli`: temporary `jj` process integration;
- `jk-core`: shared records and small data types;
- `jk-tui`: Ratatui state, rendering, and input actions.

Those boundaries keep `jj` presentation decisions at the edge while the TUI owns interaction state.

## License

`jk` is dual-licensed under either [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE).
