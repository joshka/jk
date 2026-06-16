# jk

`jk` is a log-first terminal UI for [Jujutsu](https://github.com/jj-vcs/jj). It is being rebuilt
from a clean root so the first release can be small, reviewed, tested, and useful.

The immediate goal is modest: keep a `jj` view open beside an editor, terminal, or coding agent,
then refresh in place when work changes. You should be able to look at the graph, open the relevant
diff, understand what changed, and get back to work without repeatedly quitting and rerunning
`jj log`.

Or, less formally: we shall know them by the diffs.

## Why This Exists

`jj` already has excellent command-line building blocks: `log`, `show`, `diff`, `op log`, revsets,
templates, operation history, and undo. The friction is in the loop between those commands. A common
workflow is:

1. Keep `jj log` open in one terminal pane.
1. Let an editor, shell, or coding agent make changes elsewhere.
1. Quit and rerun `jj log` to regain context.
1. Run `jj show` or `jj diff` to inspect what happened.
1. Repeat.

`jk` exists to tighten that loop. The first useful version should make it cheap to:

- keep a log-like view open;
- refresh with one key, and later refresh automatically when the repo changes;
- move through graph items rather than raw terminal lines;
- open `show` and `diff` for the selected change;
- preserve `jj` concepts, wording, templates, colors, and behavior wherever the library surface
  allows it.

This is especially useful when reviewing coding-agent work. Agents can produce a lot of plausible
text. The diffs are the artifact that matters.

## Design Lineage

This reset is based on four inputs:

- the previous `jk` prototypes archived as `joshka/prototype-*` bookmarks;
- the design discussion in
  [`jj-vcs/jj#9319`](https://github.com/jj-vcs/jj/pull/9319);
- the Discord discussion around built-in versus downstream TUI experiments;
- the release and crate hygiene patterns from nearby Rust TUI projects such as Betamax,
  `tui-widgets`, Ratatui, and `ratatui-toolbar`.

The important lesson from the prototypes is not that the broad app should be published. It should
not. The useful lessons are:

- log-first is still the right home surface;
- one active view is a better default than a pane-heavy dashboard;
- `show`, `diff`, refresh, search, and copy form the core inspection loop;
- mutation flows should wait until exact targets and confirmation semantics are boring;
- parsing rendered CLI output is the wrong long-term foundation for a serious jj-native TUI.

## Product Shape

The first release target is intentionally narrow.

`jk` should start with:

- a full-screen log view;
- manual refresh;
- selection movement;
- `show` and `diff` inspection for the selected change;
- enough help text to discover the current keys;
- tests for state transitions and rendered output.

Everything else is later:

- auto-refresh;
- operation log;
- status;
- bookmarks;
- file views;
- guided mutation actions;
- Homebrew formula maintenance;
- broader command coverage.

Those are valid directions, but they are not prerequisites for the first reviewable crate.

## Integration Principle

`jk` should not grow a shadow implementation of `jj`.

The preferred integration path is direct use of `jj-cli` and `jj-lib` so the TUI can reuse jj's own
concepts, templates, graph rendering, formatter behavior, revset handling, config, and command
semantics. Shelling out to `jj` and parsing stdout may be useful for comparison tests or temporary
spikes, but it should be treated as a fallback, not the architecture.

The first technical question is therefore:

> Can `jk` obtain log-like semantic records and CLI-equivalent renderable output
> through `jj-cli` / `jj-lib` without parsing `jj log` output?

If the answer is "not cleanly yet", that is useful evidence. `jk` should then make the missing
contract explicit instead of burying it under fragile text parsing.

## Interaction Bias

`jk` is view-centric rather than pane-centric.

The default screen should feel like an interactive `jj log`: compact, stable, and easy to refresh.
Preview layouts may become inline, split, or fullscreen, but panes are presentation choices, not the
core mental model.

The initial workflow should be:

```text
log -> show/diff -> back -> refresh
```

The app should work well inside another terminal split, tmux pane, or VS Code terminal. It should
not assume it owns the whole screen as a dashboard.

## Development

This repository uses `jj` for version control and `just` for local tasks.

```sh
just --list
just release-check
```

The current reset uses `color-eyre` for the binary harness. That is a pragmatic starting point, not
a deep product decision. `jj` itself uses its own `CommandError` shape and `thiserror` heavily; `jk`
should follow that style for domain errors as the code grows. `miette` remains worth considering if
user diagnostics need structured spans, source labels, or richer CLI reports.

## Release Posture

The crate name `jk` is reserved for a solid release, not a prototype dump.

Before publishing to crates.io, the release candidate should have:

- a reviewed and narrow product surface;
- tests for the core state machine and rendered output;
- `just release-check` passing;
- accurate README claims;
- no dependency on broad experimental prototype code.

Release-plz owns crates.io publishing, changelogs, tags, and GitHub Releases. The release workflow
also builds `jk` binary archives for macOS and Linux on `x86_64` and `aarch64`. Those archives are
named for cargo-binstall and include `.sha256` files so the Homebrew tap can install upstream
release assets and smoke-test `jk --version` / `jk --help`.

The workspace also carries placeholder crates for possible future boundaries:

- `jk-core`
- `jk-cli`
- `jk-tui`

They are intentionally empty reservation crates until there is a reviewed reason to make those
boundaries real.
