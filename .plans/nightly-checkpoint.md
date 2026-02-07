# Nightly Checkpoint (2026-02-07)

## Completion status

- Gold command set is implemented in the TUI flow planner and guarded execution path.
- `jk` is the primary entrypoint and defaults to `jk log` behavior (no `jk tui` split).
- High-frequency flows are available from normal mode and command mode:
  - inspect: `log`, `status`, `show`, `diff`, `operation log`, `bookmark list`, `root`
  - mutate: `new`, `describe`, `commit`, `next`, `prev`, `edit`
  - rewrite/recover: `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`, `restore`,
    `revert`
  - remote/bookmark: `git fetch`, `git push`, bookmark create/set/move/track/untrack/etc.
- Tier C commands remain confirm-gated with previews/fallback preview behavior.

## Latest additions this pass

- Native wrapper parity:
  - `bookmark list`, `operation log`, `workspace root`, `workspace list`, `operation show`,
    `operation diff`, `resolve -l`, `file list`, `tag list`, `git fetch`, and `git push` render
    with native wrappers.
  - additional `file` read flows now render via native wrappers: `file show`, `file search`,
    `file annotate`.
  - additional `file` mutation flows now render via native wrappers: `file track`,
    `file untrack`, `file chmod`.
  - bookmark mutation flows now render via native wrappers:
    `bookmark create/set/move/track/untrack/delete/forget/rename`.
  - workspace mutation flows now render via native wrappers:
    `workspace add/forget/rename/update-stale`.
  - operation mutation flows now render via native wrappers:
    `operation restore` and `operation revert`.
  - top-level daily/rewrite mutation flows now render via native wrappers:
    `new`, `describe`, `commit`, `edit`, `next`, `prev`, `rebase`, `squash`, `split`, `abandon`,
    `undo`, `redo`, `restore`, `revert`.
- Shortcut coverage:
  - added `]`/`[`/`e` for `next`/`prev`/`edit`
  - added `o`/`L`/`w` for `operation log`/`bookmark list`/`root`
  - added `v` for `resolve -l`
  - added `f`/`t` for `file list`/`tag list`
- Discoverability:
  - added in-app alias catalog via `:aliases` and `:aliases <query>`
  - added keymap catalog via `:keys` and `:keys <query>` (also `jk keys` on startup)
  - added normal-mode `K` shortcut for quick keymap access
  - added normal-mode `A` shortcut for quick alias-catalog access
  - surfaced local discovery views (`aliases`, `keys`, `keymap`) in `:commands`
  - surfaced command-group default hints in unfiltered `:commands` output
  - defaulted `resolve` to `resolve -l` for list-first conflict inspection
  - defaulted `file` and `tag` groups to `list` views for faster command-mode inspection
  - switched `operation show` and `operation diff` to direct read execution without prompts
  - added guided prompts for `file track`, `file untrack`, and `file chmod`.
  - added guided prompts for `tag set` and `tag delete` while keeping list-first `tag` default.
- Alias behavior:
  - `rbm`/`rbt` now preserve explicit destination flags (`-d`/`--destination`/`--to`/`--into`)
    so explicit user intent wins over alias defaults.
  - alias catalog now includes the full installed OMZ plugin set (including `jjbc`, `jjbd`,
    `jjbf`, `jjbr`, `jjcmsg`, `jjdmsg`, `jjgcl`, `jjla`) with explicit coverage tests.
- Regression safety:
  - added OMZ gold-alias flow matrix test
  - added `insta` snapshots for bookmark and operation wrapper views
  - added `insta` snapshot for status wrapper output
  - added `insta` snapshots for `git fetch` and `git push` wrapper views
  - added `insta` snapshots for `file show`, `file search`, and `file annotate` wrappers
  - added `insta` snapshots for `file track`, `file untrack`, and `file chmod` wrappers
  - added `insta` snapshots for `bookmark set`, `workspace add`, and `operation restore` wrappers
  - added `insta` snapshots for top-level `commit` and `rebase` wrapper views
- Rendering polish:
  - `show`/`diff` wrappers now add section spacing between top-level file blocks
  - `status`/`operation log` wrappers now include compact summaries and clearer section spacing
  - `workspace list` and `operation show` wrappers now include compact summaries and tips
  - `operation diff` wrapper now includes changed-commit summary and operation-flow tip
  - `file list` and `tag list` wrappers now include compact summaries and empty-state hints
  - `resolve -l` wrapper now includes conflict-count and no-conflicts summaries
  - `git fetch` and `git push` wrappers now include compact summaries and follow-up tips
  - `file show`, `file search`, and `file annotate` wrappers now include concise summaries and
    direct follow-up tips
  - `file track`, `file untrack`, and `file chmod` wrappers now include mutation-focused summaries
    and follow-up tips
  - `bookmark`, `workspace`, and `operation restore/revert` mutation wrappers now include compact
    summaries and follow-up verification tips
  - top-level daily/rewrite mutation wrappers now include command-specific follow-up tips
  - command-safety mapping now marks read-only `file`/`tag` subcommands as Tier `A`
  - command-safety mapping now marks `resolve -l` as Tier `A`

## Recent commit stack

- `feat(view): wrap mutation outputs` (`change: tpqrulwsxpvr`)
- `feat(ux): expand read-mode wrappers and shortcuts` (`change: srzlwxtxtkll`)
- `fix(alias): preserve explicit destinations and parity` (`change: uxqqtlkqotxq`)
- `feat(flow): default list-first command groups` (`change: pmzoznlxuulu`)
- `feat(flow): default file and tag list views` (`change: qulmnqullnpn`)
- `feat(help): add local views to command registry` (`change: qqrxwkptvkns`)
- `test(view): snapshot status wrapper output` (`3edf25f61dc5`)
- `feat(view): refine status and operation summaries` (`544b9889c581`)
- `feat(tui): improve discoverability and scanability` (`b902e6a158f7`)
- `feat(view): unify workspace-root presentation` (`d3dc67c0af99`)
- `feat(ux): add quick read-mode shortcuts` (`69605ad1ece1`)
- `test(ux): expand alias and wrapper coverage` (`7c1746fee75e`)
- `feat(ux): add nav keys and list wrappers` (`8dd98891847f`)

## Validation checkpoint

- Latest full checkpoint passed on current working commit:
  - `markdownlint-cli2 README.md AGENTS.md .plans/*.md docs/**/*.md`
  - `cargo fmt --all`
  - `cargo check`
  - `cargo test` (151 passed)
  - `cargo clippy --all-targets --all-features -- -D warnings`

## Blockers

- No active blockers.
- Resolved blocker recorded in `.plans/blockers.md`:
  - mutation-prone shortcut tests can move working copy when they execute live `jj` commands.

## Suggested first task tomorrow

- Continue Phase 3 passthrough hardening:
  - tune and refine wrapper summaries for command-specific signal (for example, parsing rebase and
    bookmark mutation output into richer compact counts) without losing the pager-first style.
