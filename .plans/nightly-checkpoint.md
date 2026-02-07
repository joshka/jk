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
  - lifted rewrite-adjacent flows now also render via native wrappers:
    `absorb`, `duplicate`, and `parallelize`.
  - additional advanced command wrappers now render natively:
    `interdiff`, `evolog`, and `metaedit`.
  - rewrite flow `simplify-parents` now renders via native mutation wrapper.
  - rewrite flow `fix` now renders via native mutation wrapper.
  - resolve flow now has full wrapper parity for both `resolve -l` and direct resolve actions.
  - rewrite flow `diffedit` now renders via native mutation wrapper.
  - command output now preserves `jj` color styling in TUI views by forcing `jj --color=always`
    for interactive command execution.
  - render loop now resets terminal color after every row so truncated ANSI-styled lines do not
    leak style into following rows.
- Shortcut coverage:
  - added `l` as a dedicated home shortcut to jump to `log` from any normal-mode view
  - added `]`/`[`/`e` for `next`/`prev`/`edit`
  - added `o`/`L`/`w` for `operation log`/`bookmark list`/`root`
  - added `v` for `resolve -l`
  - added `f`/`t` for `file list`/`tag list`
- Discoverability:
  - `:commands` now annotates top-level `jj` default aliases
    (`bookmark (b)`, `commit (ci)`, `describe (desc)`, `operation (op)`, `status (st)`).
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
  - added native wrapper presentation for `version` output so utility command results remain
    scan-friendly inside the TUI.
- Alias behavior:
  - core `jj` default aliases `b`, `ci`, and `op` now canonicalize to `bookmark`, `commit`, and
    `operation` so native `jj` shorthand paths use the same guided/native wrappers.
  - in-app alias catalog now includes core `jj` defaults:
    `b`, `ci`, `desc`, `op`, and `st`.
  - `rbm`/`rbt` now preserve explicit destination flags (`-d`/`--destination`/`--to`/`--into`)
    so explicit user intent wins over alias defaults.
  - alias catalog now includes the full installed OMZ plugin set (including `jjbc`, `jjbd`,
    `jjbf`, `jjbr`, `jjcmsg`, `jjdmsg`, `jjgcl`, `jjla`) with explicit coverage tests.
- Regression safety:
  - added OMZ gold-alias flow matrix test
  - added gold-command wrapper matrix regression test for native header rendering
  - added `insta` snapshots for bookmark and operation wrapper views
  - added `insta` snapshot for status wrapper output
  - added `insta` snapshots for `git fetch` and `git push` wrapper views
  - added `insta` snapshots for `file show`, `file search`, and `file annotate` wrappers
  - added `insta` snapshots for `file track`, `file untrack`, and `file chmod` wrappers
  - added `insta` snapshots for `bookmark set`, `workspace add`, and `operation restore` wrappers
  - added `insta` snapshots for top-level `commit` and `rebase` wrapper views
  - added `insta` snapshots for additional mutation wrappers:
    bookmark `create/move/track/untrack`, workspace `forget/rename/update-stale`, and operation
    `revert`
  - added `insta` snapshots for remaining bookmark mutation wrappers:
    `bookmark delete`, `bookmark forget`, and `bookmark rename`
  - added `insta` snapshots for additional top-level mutation wrappers:
    `new`, `undo`, `abandon`, and `restore`
  - added `insta` snapshots for the remaining top-level mutation wrappers:
    `describe`, `edit`, `next`, `prev`, `squash`, `split`, `redo`, and `revert`
  - added broad top-level mutation wrapper regression coverage plus command-specific tip assertions
  - added wrapper-routing matrix tests for bookmark/workspace mutation subcommands and operation
    `revert`
  - added focused tests for core `jj` default alias flow planning and alias-catalog coverage
  - added command-registry tests for default-alias annotation and alias-based filtering
  - added startup-action regression test coverage for core `jj` default aliases
    (`jk st`, `jk ci`, `jk desc`, `jk b`, `jk op`)
  - added startup-action regression test coverage for high-frequency OMZ aliases
    (`jk jjst`, `jk jjl`, `jk jjd`, `jk jjgf`, `jk jjgp`, `jk jjrbm`, `jk jjc`, `jk jjds`)
  - added explicit summary-heuristic tests for bookmark/workspace/operation mutation wrappers
  - added explicit summary-heuristic tests for `git fetch`/`git push` wrapper summaries
  - added explicit wrapper render/decorator tests for `version` output
  - added flow-planner tests for selection-aware `absorb`/`duplicate`/`parallelize` defaults
  - added command-registry tests asserting `absorb`/`duplicate`/`parallelize` are `guided`
  - added flow-planner tests for selection-aware `interdiff`/`evolog` defaults and `metaedit`
    prompt routing
  - added command-registry tests asserting `interdiff`/`evolog`/`metaedit` are `guided`
  - added command-registry and flow-planner coverage for guided `simplify-parents` defaults
  - added command-registry and flow-planner coverage for guided `fix` defaults
  - added command-registry coverage for guided `resolve` mode
  - added command-registry and flow-planner coverage for guided `diffedit` defaults
  - updated top-level `commit` and `rebase` snapshots for signal-first summary rendering
  - added `insta` snapshots for top-level `absorb`, `duplicate`, and `parallelize` wrappers
  - added `insta` snapshots for `interdiff`, `evolog`, and top-level `metaedit` wrappers
  - added `insta` snapshot for top-level `simplify-parents` wrapper
  - added `insta` snapshot for top-level `fix` wrapper
  - added `insta` snapshot for resolve action wrapper output
  - added `insta` snapshot for top-level `diffedit` wrapper
  - added ANSI-specific regression tests for revision extraction and width trimming in colored
    output lines
  - refreshed `insta` snapshots for bookmark/workspace/operation mutation wrappers to lock in
    signal-first summary behavior
  - refreshed `insta` snapshot for `git push` wrapper signal-first summary behavior
  - added `insta` snapshot for `version` wrapper presentation
- Rendering polish:
  - `show`/`diff` wrappers now add section spacing between top-level file blocks
  - `interdiff` now uses a native diff-style wrapper with concise guidance for from/to comparison
  - `evolog` now uses a native evolution wrapper with compact entry-count summary
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
  - bookmark/workspace/operation mutation wrappers now prefer signal-first summary lines when
    command output includes a clear action line, and fall back to output-count summaries otherwise
  - `git fetch`/`git push` wrappers now also prefer signal-first action summaries when available
  - top-level daily/rewrite mutation wrappers now include command-specific follow-up tips
  - top-level mutation wrappers now prefer signal-first summary lines (for example rebase/undo
    action output) before falling back to generic output counts
  - top-level mutation wrappers now include rewrite-signal support for `simplify-parents`
    (`Rebased ...` output lines)
  - top-level mutation wrappers now include rewrite-signal support for `fix`
    (`Fixed ...` and `Rebased ...` output lines)
  - resolve output now renders through native wrappers for both list and action forms
  - top-level mutation wrappers now include rewrite-signal support for `diffedit`
    (`Rebased ...` output lines)
  - ANSI-aware parsing and width trimming now prevent color escape sequences from corrupting
    revision detection and line truncation
  - top-level mutation wrappers now include signal-first summary support for
    `absorb`/`duplicate`/`parallelize` when command output includes explicit action lines
  - command-safety mapping now marks read-only `file`/`tag` subcommands as Tier `A`
  - command-safety mapping now marks `resolve -l` as Tier `A`

## Recent commit stack

- `feat(view): add native version output wrapper` (`change: vtnvvxmvvzpk`)
- `feat(view): add signal summaries for remote wrappers` (`change: kkuqnwpqzmks`)
- `feat(view): prefer signal summaries in mutation wrappers` (`change: mmookzxkmqos`)
- `test(startup): cover core jj alias routing` (`change: ywvronpkrvxs`)
- `feat(commands): annotate default aliases in registry` (`change: ywqkqkkxuruv`)
- `feat(alias): surface core jj default shorthands` (`change: lnvzquozzqny`)
- `fix(alias): canonicalize core jj shorthands` (`change: rklkwyoyqlsp`)
- `test(view): add mutation wrapper routing matrix` (`change: xluyuvkxnqsn`)
- `test(view): snapshot remaining top-level mutations` (`change: nuvpwspkpwzx`)
- `test(view): snapshot top-level mutation variants` (`change: umvnxkynouzz`)
- `test(view): snapshot remaining bookmark mutations` (`change: xsovkmssrwnt`)
- `test(view): snapshot mutation wrapper variants` (`change: qkonnpzvwtor`)
- `test(view): add gold wrapper matrix` (`change: nxlpypntzumw`)
- `feat(view): add mutation summary heuristics` (`change: lvxysqknvsxt`)
- `test(view): broaden mutation wrapper coverage` (`change: pkvvwklrwnso`)
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
  - `cargo test` (218 passed)
  - `cargo clippy --all-targets --all-features -- -D warnings`

## Blockers

- No active blockers.
- Resolved blocker recorded in `.plans/blockers.md`:
  - mutation-prone shortcut tests can move working copy when they execute live `jj` commands.

## Suggested first task tomorrow

- Continue Phase 3 passthrough hardening:
  - tune and refine wrapper summaries for command-specific signal (for example, parsing rebase and
    bookmark mutation output into richer compact counts) without losing the pager-first style.
