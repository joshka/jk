# Command Inventory

This document maps the `jj` CLI surface to likely `jk` homes. The goal is not to make every command
a first-class screen immediately. The goal is to decide, deliberately, which commands deserve
persistent UI, which deserve guided flows, and which should remain command-mode passthrough until
proven important.

## Classification

- `native screen`: persistent read surface or central navigation home.
- `utility screen`: focused list/detail screen that supports a narrower task.
- `guided flow`: prompt and confirmation-driven flow where `jk` adds context or safety over raw CLI
  use.
- `passthrough`: command mode only for now.
- `defer`: intentionally out of near-term scope for `jk`.

These classifications are planning hypotheses. Promote a command from passthrough to native support
when there is evidence that `jk` can improve the workflow without hiding fragile parsing or
duplicating too much `jj` behavior.

## Prioritization Signals

The first-pass priority order here is informed by three inputs:

- the current `jk` product direction
- the old `prototype` branch and VHS artifacts
- common `jj` aliases collected in the Oh My Zsh `jj` plugin work, including `bookmark`, `commit`,
  `diff`, `git fetch`, `git push`, `log`, `new`, `rebase`, `restore`, `root`, `split`, `squash`, and
  `status`

That alias surface is not a perfect usage survey, but it is a useful signal for what experienced
users consider common enough to shorten aggressively.

## Core Read Surface

  | `jj` command | User goal                              | Likely `jk` home | Notes                                  |
  | ------------ | -------------------------------------- | ---------------- | -------------------------------------- |
  | `log`        | inspect history and stack shape        | `native screen`  | the home screen                        |
  | `show`       | inspect one change                     | `native screen`  | drill-down from graph                  |
  | `diff`       | inspect patch content                  | `native screen`  | drill-down from graph or show          |
  | `status`     | inspect working copy state             | `native screen`  | high-frequency triage surface          |
  | `evolog`     | inspect change evolution               | `utility screen` | likely later than op-log               |
  | `interdiff`  | compare patch deltas between revisions | `passthrough`    | can become a derived detail view later |
  | `version`    | inspect tool version                   | `passthrough`    | no dedicated screen needed             |

## Navigation And Working Copy

  | `jj` command | User goal                             | Likely `jk` home | Notes                                          |
  | ------------ | ------------------------------------- | ---------------- | ---------------------------------------------- |
  | `new`        | create a new empty change             | `guided flow`    | likely prompt plus confirmation-free execution |
  | `edit`       | make another change the working copy  | `guided flow`    | can be shortcut from graph                     |
  | `next`       | move to child revision                | `guided flow`    | high-value shortcut candidate                  |
  | `prev`       | move to parent revision               | `guided flow`    | high-value shortcut candidate                  |
  | `commit`     | finalize current change and advance   | `guided flow`    | likely prompt for description                  |
  | `describe`   | edit change metadata                  | `guided flow`    | should feel native                             |
  | `metaedit`   | edit metadata without content changes | `passthrough`    | similar goal, lower frequency                  |
  | `root`       | inspect workspace root                | `utility screen` | simple informational screen                    |
  | `workspace`  | manage workspaces                     | `utility screen` | list and focused subflows make sense           |
  | `sparse`     | manage sparse working copy            | `passthrough`    | likely too advanced for early UI               |

## Rewrite And Recovery

  | `jj` command          | User goal                             | Likely `jk` home | Notes                                  |
  | --------------------- | ------------------------------------- | ---------------- | -------------------------------------- |
  | `rebase`              | move revisions                        | `guided flow`    | safety and target preview are valuable |
  | `squash`              | move changes into another revision    | `guided flow`    | likely graph-driven                    |
  | `split`               | split one revision                    | `guided flow`    | deserves confirmation and preview      |
  | `abandon`             | remove a revision from active history | `guided flow`    | high-risk, preview-first               |
  | `duplicate`           | clone content into a new change       | `guided flow`    | not first-wave, but fits guided model  |
  | `parallelize`         | make revisions siblings               | `passthrough`    | niche until rewrite model matures      |
  | `simplify-parents`    | normalize parent edges                | `passthrough`    | niche advanced flow                    |
  | `absorb`              | move changes into mutable descendants | `passthrough`    | likely too subtle for early UI         |
  | `restore`             | restore paths from another revision   | `guided flow`    | useful once file flows exist           |
  | `revert`              | apply reverse changes                 | `guided flow`    | high-risk, preview-first               |
  | `undo`                | undo last operation                   | `guided flow`    | likely direct action from op-log       |
  | `redo`                | redo most recently undone operation   | `guided flow`    | likely direct action from op-log       |
  | `operation log`       | inspect op history                    | `native screen`  | central recovery surface               |
  | `operation show`      | inspect one operation                 | `utility screen` | drill-down from op-log                 |
  | `operation diff`      | inspect repo delta for an operation   | `utility screen` | drill-down from op-log                 |
  | `operation restore`   | restore repo to earlier op            | `guided flow`    | anchored in op-log                     |
  | `operation revert`    | revert an earlier op                  | `guided flow`    | anchored in op-log                     |
  | `operation integrate` | integrate non-integrated ops          | `passthrough`    | specialized                            |
  | `operation abandon`   | drop operation history                | `defer`          | too dangerous for early UI             |

## Bookmarks, Tags, And Related Refs

  | `jj` command       | User goal                        | Likely `jk` home | Notes                                       |
  | ------------------ | -------------------------------- | ---------------- | ------------------------------------------- |
  | `bookmark list`    | inspect bookmark state           | `utility screen` | strong fit from prototype ideas             |
  | `bookmark set`     | create or retarget bookmark      | `guided flow`    | should launch from bookmark screen or graph |
  | `bookmark create`  | create bookmark                  | `guided flow`    | may collapse into set                       |
  | `bookmark move`    | move bookmark                    | `guided flow`    | may collapse into set                       |
  | `bookmark rename`  | rename bookmark                  | `guided flow`    | utility-level                               |
  | `bookmark delete`  | delete bookmark                  | `guided flow`    | confirmation-worthy                         |
  | `bookmark forget`  | forget bookmark without deletion | `guided flow`    | advanced but screen-related                 |
  | `bookmark track`   | track remote bookmark            | `guided flow`    | belongs with bookmark screen                |
  | `bookmark untrack` | stop tracking remote bookmark    | `guided flow`    | belongs with bookmark screen                |
  | `bookmark advance` | advance closest bookmark         | `passthrough`    | probably not a first-class flow             |
  | `tag`              | manage tags                      | `utility screen` | similar to bookmarks, lower frequency       |

## Files And Resolve

  | `jj` command    | User goal                           | Likely `jk` home | Notes                                          |
  | --------------- | ----------------------------------- | ---------------- | ---------------------------------------------- |
  | `file list`     | inspect files in a revision         | `utility screen` | useful companion to show/diff                  |
  | `file show`     | inspect file contents in a revision | `utility screen` | likely drill-down from file list               |
  | `file search`   | search file contents in a revision  | `utility screen` | useful if scoped well                          |
  | `file annotate` | inspect line-level provenance       | `utility screen` | later read-surface                             |
  | `file track`    | start tracking paths                | `guided flow`    | probably from status/file list                 |
  | `file untrack`  | stop tracking paths                 | `guided flow`    | probably from status/file list                 |
  | `file chmod`    | flip executable bit                 | `guided flow`    | lower-frequency file action                    |
  | `resolve`       | resolve conflicts                   | `utility screen` | list-first view is more useful than a shortcut |

## Git And Remote Sync

  | `jj` command            | User goal               | Likely `jk` home | Notes                                |
  | ----------------------- | ----------------------- | ---------------- | ------------------------------------ |
  | `git fetch`             | update from remotes     | `guided flow`    | strong status-screen action          |
  | `git push`              | publish changes         | `guided flow`    | preview and confirmation matter      |
  | other `jj git` commands | lower-level Git interop | `passthrough`    | likely too broad for early native UI |

## Inspection But Probably Not Core

  | `jj` command | User goal                           | Likely `jk` home | Notes                                                                      |
  | ------------ | ----------------------------------- | ---------------- | -------------------------------------------------------------------------- |
  | `bisect`     | find bad revision by search         | `defer`          | large workflow, later if ever                                              |
  | `arrange`    | interactively arrange commit graph  | `defer`          | likely incompatible with the current single-view model without more design |
  | `diffedit`   | interactive patch editing           | `defer`          | editor-centric and high-complexity                                         |
  | `fix`        | apply formatting or automated fixes | `passthrough`    | can remain CLI-first                                                       |
  | `config`     | inspect or edit config              | `passthrough`    | not a product-center task                                                  |
  | `sign`       | sign revisions                      | `passthrough`    | advanced                                                                   |
  | `unsign`     | drop revision signature             | `passthrough`    | advanced                                                                   |
  | `gerrit`     | Gerrit integration                  | `defer`          | host-specific                                                              |
  | `util`       | infrequently used support commands  | `defer`          | explicitly not a first-class surface                                       |
  | `help`       | inspect CLI help                    | `passthrough`    | `jk` needs its own help, not a full CLI mirror                             |

## Current Planning Bias

Short version:

- First-wave native screens: `log`, `show`, `diff`, `status`, `operation log`.
- First-wave utility screens: bookmarks, files, resolve, tags, workspace root.
- First-wave guided/direct flows: `jj new trunk`, `new` from selected revision, `describe`,
  `commit`, `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`, `git fetch`, `git push`,
  bookmark set.
- Everything else stays passthrough or deferred until the core loop is strong.

When a command needs native structure, check [`integration-strategy.md`](integration-strategy.md)
before deciding whether to use rendered output, a narrow parser, structured output, `jj_cli`,
`jj_lib`, or an upstream API.
