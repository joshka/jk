# Per-Command Deep Plans

This file defines the implementation contract for `jj` command coverage in `jk`.
Each command has a mode, safety tier, and explicit test target.

## Execution modes

- `native`: purpose-built TUI flow.
- `guided`: TUI flow with prompts and previews.
- `passthrough`: run `jj` in-app and capture output.

## Safety tiers

- `A`: read-only or low-risk.
- `B`: mutable and generally reversible.
- `C`: history rewrite or remote-risking action; preview + explicit confirm required.

## Phase 1 Command Specs

### Read and inspect

- `log`
  - Mode/Tier: `native` / `A`
  - Flow: default screen, revset prompt, patch toggle, vim movement.
  - Tests: snapshot parity vs `jj log`, movement selection tests.
- `status`
  - Mode/Tier: `native` / `A`
  - Flow: inline status panel and `:` command entry (`jjst` alias supported).
  - Tests: status parser tests and snapshot tests.
- `show`
  - Mode/Tier: `native` / `A`
  - Flow: Enter on selected row opens revision details.
  - Tests: metadata and patch snapshots.
- `diff`
  - Mode/Tier: `native` / `A`
  - Flow: `d` key opens diff for selected revision, with format toggles.
  - Tests: command assembly for `-r`, `--from`, `--to`.

### Daily mutation

- `new`, `describe`, `commit`, `next`, `prev`, `edit`
  - Mode/Tier: `native` or `guided` / `B`
  - Flow: in-app prompt-driven actions anchored to selected revision context.
  - Tests: per-command arg assembly and happy-path command execution tests.

### Rewrite and recovery

- `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`
  - Mode/Tier: `guided` or `native` / `C`
  - Flow: mandatory preview with impacted revisions/bookmarks and explicit confirm.
  - Tests: guard tests proving no execution without confirmation.

### Bookmarks and remotes (core subset)

- `bookmark list`, `bookmark set`, `bookmark move`, `bookmark track`, `bookmark untrack`
  - Mode/Tier: `guided` / `B` or `C` depending on remote impact.
  - Flow: show local/remote/tracked/conflicted state before mutation.
  - Tests: command assembly and conflict-state behavior tests.
- `git fetch`, `git push`
  - Mode/Tier: `guided` / `B` for fetch, `C` for push.
  - Flow: include aliases `gf`, `gp`, `jjgf`, `jjgp`, and push preview defaults.
  - Tests: e2e alias tests and push safety guard tests.

## Phase 2 Command Specs

### Common advanced commands

- `restore`, `revert`, `absorb`, `duplicate`, `parallelize`, `interdiff`, `evolog`, `metaedit`
  - Mode/Tier: `guided` / mostly `B`, `restore` and `revert` are `C`.
  - Tests: flow assembly tests and safety tests where tier is `C`.

### Defer-lift focus (current implementation pass)

- `absorb`
  - Mode/Tier: `guided` / `C`
  - Default flow: selection-aware execute as `absorb --from <selected>` so startup and `:` command
    mode follow the selected revision model.
  - Presentation: render with native top-level mutation wrapper.
- `duplicate`
  - Mode/Tier: `guided` / `B`
  - Default flow: selection-aware execute as `duplicate <selected>`.
  - Presentation: render with native top-level mutation wrapper.
- `parallelize`
  - Mode/Tier: `guided` / `C`
  - Default flow: selection-aware execute as `parallelize <selected>`.
  - Presentation: render with native top-level mutation wrapper.
- `simplify-parents`
  - Mode/Tier: `guided` / `C`
  - Default flow: selection-aware execute as `simplify-parents <selected>`.
  - Presentation: render with native top-level mutation wrapper.
- `interdiff`
  - Mode/Tier: `guided` / `A`
  - Default flow: selection-aware execute as `interdiff --from @- --to <selected>`.
  - Presentation: render with a native diff-like wrapper for scanability.
- `evolog`
  - Mode/Tier: `guided` / `A`
  - Default flow: selection-aware execute as `evolog -r <selected>`.
  - Presentation: render with a native evolution-log wrapper and compact entry summary.
- `metaedit`
  - Mode/Tier: `guided` / `B`
  - Default flow: guided message prompt for selected revision, then
    `metaedit -m <message> <selected>`.
  - Presentation: render with native top-level mutation wrapper.

### Operation log workflows

- `operation log`, `operation show`, `operation diff`, `operation restore`, `operation revert`
  - Mode/Tier: `guided` / `A` for read, `C` for restore/revert.
  - Tests: operation renderer snapshots and guarded mutation tests.

### Additional bookmark and workspace workflows

- `bookmark create`, `bookmark rename`, `bookmark delete`, `bookmark forget`
  - Mode/Tier: `guided` / `C` when remote or history implications exist.
  - Tests: command assembly and conflict behavior tests.
- `workspace list`, `workspace add`, `workspace forget`, `workspace rename`, `workspace root`
  - Mode/Tier: `guided` / `B`
  - Tests: arg assembly and root-path display tests.

## Phase 3 Command Specs

### Passthrough-first commands

- `resolve`, `diffedit`, `fix`, `sparse *`, `file *`, `tag *`, `sign`, `unsign`
  - Mode/Tier: `passthrough` / mixed `A`, `B`, `C` by command risk.
- `git clone`, `git import`, `git export`, `git init`, `git remote *`, `git colocation`,
  `git root`
  - Mode/Tier: `passthrough` / mostly `B`, `C` for destructive remote operations.
- `config *`, `util *`, `gerrit upload`, `operation integrate`, `operation abandon`,
  `bisect run`
  - Mode/Tier: `passthrough` / mixed tiers.
- `version`
  - Mode/Tier: `passthrough` / `A`, with native wrapper presentation for consistency.
  - Tests: wrapper-render unit test + snapshot for version output formatting.

Passthrough-first means command invocation is in-app, output is in-app, and no shell escape is
required.

## Alias Compatibility Specs

### Native aliases

- `gf`, `gp`, `rbm`, `rbt`

### Oh My Zsh compatibility aliases

- `jjgf`, `jjgfa`, `jjgp`, `jjgpt`, `jjgpa`, `jjgpd`
- `jjrb`, `jjrbm`, `jjst`, `jjl`, `jjd`, `jjc`, `jjsp`, `jjsq`, `jjrs`, `jja`
- `jjrt` maps to in-app `root` action (no shell `cd`).

Normalization rule:

- Store raw alias and canonical command in telemetry and test canonical mapping directly.

## Automatic Validation Targets

- Targeted tests for changed modules first (`cargo test <name-pattern>`).
- Snapshot coverage for rendered screens and overlays with `insta`.
- Assembly tests for command arguments and alias normalization.
- Guard tests for all Tier `C` actions.
