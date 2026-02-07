# Gold Command Set (Morning Target)

This is the non-negotiable command set for the overnight implementation pass.
All items are equally important and should reach useful TUI-supported behavior.

## Core read and navigation

- `log` (default `jk` entry)
- `status`
- `show`
- `diff`

## Core change workflows

- `new`
- `describe` (`desc`)
- `commit`
- `next`
- `prev`
- `edit`

## Core rewrite and recovery workflows

- `rebase`
- `squash`
- `split`
- `abandon`
- `undo`
- `redo`

## Bookmark workflows

- `bookmark list`
- `bookmark create`
- `bookmark set`
- `bookmark move`
- `bookmark track`
- `bookmark untrack`

## Remote workflows

- `git fetch`
- `git push`

## Alias coverage

Native aliases:

- `gf`, `gp`, `rbm`, `rbt`

Oh My Zsh compatibility aliases (high-frequency focus):

- `jjgf`, `jjgfa`, `jjgp`, `jjgpt`, `jjgpa`, `jjgpd`
- `jjrb`, `jjrbm`, `jjst`, `jjl`, `jjd`, `jjc`, `jjds`, `jje`, `jjn`, `jjnt`
- `jjsp`, `jjsq`, `jjb`, `jjbl`, `jjbs`, `jjbm`, `jjbt`, `jjbu`, `jjrs`, `jja`, `jjrt`

## Read-before-coding checkpoint

When implementation drifts or context gets compacted, re-read these files:

1. `.plans/gold-command-set.md`
2. `.plans/gold-command-details.md`
3. `.plans/jk-tui-command-plan.md`
4. `.plans/command-deep-plans.md`
5. `.plans/implementation-status.md`
6. `.plans/blockers.md`
