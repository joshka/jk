# Gold Command Details

This file is the command-level implementation contract for the overnight target set.
It is intentionally concrete and should be re-read during implementation checkpoints.

## Read and inspect

- `log`
  - Mode: native baseline (`jk` default view).
  - Safety: `A`.
  - Acceptance: opens on startup, keeps cursor selection stable across refreshes.
- `status`
  - Mode: execute in-app (`jj status`) from command mode or startup args.
  - Safety: `A`.
  - Acceptance: command output renders in the main pane with no shell escape.
- `show`
  - Mode: selection-aware execute (`Enter` or `:show`).
  - Safety: `A`.
  - Acceptance: selected revision opens details.
- `diff`
  - Mode: selection-aware execute (`d` or `:diff`).
  - Safety: `A`.
  - Acceptance: selected revision diff opens with `-r <selected>`.

## Daily mutation

- `new`
  - Mode: guided prompt (`new message`).
  - Safety: `B`.
- `describe`
  - Mode: guided prompt (`describe message for <selected>`).
  - Safety: `B`.
- `commit`
  - Mode: guided prompt (`commit message`).
  - Safety: `B`.
- `next`
  - Mode: direct execute.
  - Safety: `B`.
- `prev`
  - Mode: direct execute.
  - Safety: `B`.
- `edit`
  - Mode: selection-aware execute.
  - Safety: `B`.

## Rewrite and recovery

- `rebase`
  - Mode: guided prompt for destination.
  - Safety: `C` (explicit confirm required).
- `squash`
  - Mode: guided prompt for `--into`.
  - Safety: `C`.
- `split`
  - Mode: guided prompt for fileset (non-interactive).
  - Safety: `C`.
- `abandon`
  - Mode: selection-aware execute.
  - Safety: `C`.
- `undo`
  - Mode: direct execute.
  - Safety: `C`.
- `redo`
  - Mode: direct execute.
  - Safety: `C`.

## Bookmark workflows

- `bookmark list`
  - Mode: default when running `bookmark`.
  - Safety: `A`.
- `bookmark create`
  - Mode: guided prompt (name + selected revision target).
  - Safety: `B`.
- `bookmark set`
  - Mode: guided prompt (name + selected revision target).
  - Safety: `C`.
- `bookmark move`
  - Mode: guided prompt (name + selected revision target).
  - Safety: `C`.
- `bookmark track`
  - Mode: guided prompt (`<name> [remote]`).
  - Safety: `B`.
- `bookmark untrack`
  - Mode: guided prompt (`<name> [remote]`).
  - Safety: `B`.

## Remote workflows

- `git fetch`
  - Mode: guided prompt (optional remote).
  - Safety: `B`.
  - Aliases: `gf`, `jjgf`, `jjgfa`.
- `git push`
  - Mode: guided prompt (optional bookmark) plus preview.
  - Safety: `C`.
  - Aliases: `gp`, `jjgp`, `jjgpt`, `jjgpa`, `jjgpd`.
  - Acceptance: confirmation includes `--dry-run` preview when available.

## Alias defaults

- Core `jj` defaults:
  - `b` => `bookmark`
  - `ci` => `commit`
  - `desc` => `describe`
  - `op` => `operation`
  - `st` => `status`
- `rbm` => `rebase -d main`.
- `rbt` => `rebase -d trunk()`.
- OMZ compatibility includes `jjst`, `jjl`, `jjd`, `jjc`, `jjsp`, `jjsq`, `jja`, `jjrs`, `jjrt`.

## Required tests

- Alias normalization tests for native + OMZ gold aliases.
- Flow planner tests for core `jj` default alias behavior (`b`/`ci`/`desc`/`op`/`st`).
- Startup-action tests for core `jj` default aliases (`jk st/ci/desc/b/op`).
- Command-registry tests for default alias annotation/filtering in `:commands`.
- Flow planner tests for prompt-vs-execute behavior of every gold command.
- Safety tests proving all Tier `C` commands require confirmation.
- Snapshot smoke test for log-first default frame.
- Gold command wrapper matrix test validating native header rendering for core
  read/mutation/bookmark/remote flows.
