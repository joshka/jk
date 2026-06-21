# jk

`jk` is a log-first terminal UI for [Jujutsu](https://github.com/jj-vcs/jj).

It keeps a `jj` log-like view open, lets you refresh in place, and adds interactive navigation for
reviewing change descriptions and selected-change diffs.

![jk log view](https://www.joshka.net/jk-screenshots/assets/jk-log.gif)

## Current Status

The supported surface includes:

- log view backed by `jj`;
- manual refresh;
- movement by change, page, and edge;
- inline expansion of the selected change description;
- selected-change diff inspection from the log or with `jk diff [REVISION]`;
- diff search, file/hunk navigation, horizontal scrolling, and folding;
- mode-specific help overlays;
- retryable empty/error states for selected diffs.

## Commands

```sh
jk
jk log
jk diff
jk diff <revision>
jk -R /path/to/repo -n 20
```

Bare `jk` follows `jj`'s configured `ui.default-command` when that command is log-like enough for
navigation. Use `jk log` for the explicit log path.

## Roadmap

Near-term diff-review improvements include a file jump overlay, search highlighting, clearer
current-hunk affordances, richer fold indicators, wrapping mode, and better presentation for
binary, rename, conflict, mode-change, and permission-change output.

Broader directions include automatic refresh, selected-change `show`, operation log inspection,
repository status and bookmark views, copy/export helpers, and carefully scoped mutation actions.

See the repository README for the full status and development workflow.
