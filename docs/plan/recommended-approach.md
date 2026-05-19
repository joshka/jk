# Recommended Approach

This document is the opinionated default plan. It is written so the maintainer can either approve
the direction as-is or override a specific decision with clear tradeoffs.

## Executive Recommendation

Build `jk` around a high-fidelity log-first core, then add the two common direct workflows:

1. harden the log row contract and selection model;
1. add log view modes for default, trunk-focused, recent, all/repo, and custom revset views;
1. add direct `jj git fetch`;
1. add direct `jj new trunk`;
1. generate help/keymap from command metadata;
1. add status and operation log;
1. add bookmarks and file/resolve utilities;
1. add risky graph mutations only after action selection and previews are solid.

The key idea is that the first implementation work should make the core interaction model reliable,
not broaden command coverage prematurely.

## Recommended Decisions

  | Decision           | Recommendation                                                     | Why                                                              | Revisit When                                            |
  | ------------------ | ------------------------------------------------------------------ | ---------------------------------------------------------------- | ------------------------------------------------------- |
  | Default screen     | Start in log/default work view                                     | Log is the navigation anchor and mirrors current manual `jj` use | Status becomes the dominant daily entry point           |
  | Default log source | Use the user-configured default `jj` view when possible            | Keeps jj fidelity and respects user templates/revsets            | The default view cannot provide stable row identity     |
  | Trunk target       | Use `trunk()` and require it to resolve exactly for direct actions | `trunk()` is jj's configurable built-in trunk abstraction        | User needs per-repo explicit target override            |
  | `jj new trunk`     | Direct low-friction action when trunk is exact                     | Common, easy to undo, and should not be buried                   | It commonly targets the wrong commit in practice        |
  | `jj git fetch`     | Direct action with command output and refresh                      | Common and low-risk despite being state-changing                 | Fetch options/remotes need frequent customization       |
  | `git push`         | Preview/confirmation flow, not direct                              | Publication is more consequential than fetch                     | A selected bookmark/remote makes the target unambiguous |
  | Risky rewrites     | Preview-first with confirmation                                    | Rebase/squash/split/abandon alter graph shape                    | A specific action proves safe enough for a lower tier   |
  | Multi-select       | Defer until first risky graph action needs it                      | Avoids making the core log UI heavier too early                  | Rebase/squash work begins                               |
  | Panes              | Inline expansion or detail screens first                           | Keeps the TUI focused and avoids dashboard layout                | Inline detail cannot preserve enough context            |
  | Semantic data      | Prefer structured/code contracts over parsed CLI output            | Avoids losing meaning and fragile output parsing                 | No API exists and the fallback stays narrow             |

## Keybinding Policy

Recommended policy:

- Lowercase keys run safe, common, or inspect-like actions.
- Uppercase keys open risky, destructive, publishing, or forceful flows.
- `?`, `/`, `n`, `N`, `r`, `y`, `j`, `k`, `g`, `G`, `h`, `l`, arrows, and `Esc` keep stable
  meanings.
- `s` means show-like detail for a selected revision.
- `d` means diff-like detail for a selected revision.
- Destructive delete/abandon-like behavior should not use lowercase `d`.
- Complex actions that need roles, prompts, or previews should go through an action menu or
  flow-specific prompt until the shortcut is proven obvious.

Recommended first bindings:

  | Key     | Meaning                                                 | Safety                         |
  | ------- | ------------------------------------------------------- | ------------------------------ |
  | `f`     | `jj git fetch`                                          | Direct with output and refresh |
  | `c`     | Create new change from trunk or exact selected context  | Direct when target is exact    |
  | `p`     | Open push preview flow                                  | Preview required               |
  | `w`     | Switch work/revset view mode                            | Direct view action             |
  | `x`     | Expand/collapse selected row                            | Direct view action             |
  | `Space` | Toggle multi-select, once multi-select exists           | Direct selection action        |
  | `A`     | Abandon flow                                            | Preview/confirmation required  |
  | `R`     | Rebase/restore-style flow, depending on screen          | Preview/confirmation required  |
  | `D`     | Delete/forget destructive flow in ref/workspace screens | Confirmation required          |

Tradeoff: `c` may later compete with commit-related flows. The recommendation is to reserve `c` for
"create a new change" because `jj new trunk` is one of the highest-frequency manual workflows.
Commit/describe can start in the action menu until their exact shortcut shape is earned.

## Revset And View Modes

Recommended modes:

  | Mode              | Recommended command shape                                    | Purpose                           |
  | ----------------- | ------------------------------------------------------------ | --------------------------------- | --------------------------------------------- | ----------------------------------------- |
  | Default work      | Run the default `jj`/`jj log` view without overriding revset | Match user-configured jj behavior |
  | Trunk work        | `jj log -r 'trunk()..                                        | trunk()'`                         | Show work not in trunk plus the trunk anchor  |
  | Current stack     | `jj log -r 'reachable(@, mutable())                          | trunk()'`                         | Show the active stack around the working copy |
  | Recent work       | `jj log -r 'latest(mutable(), 20)                            | @                                 | trunk()'`                                     | Find recent mutable work across OSS repos |
  | All/repo overview | `jj log -r 'all()'`                                          | Broad orientation/debugging view  |
  | Custom revset     | User-provided revset                                         | Power-user escape hatch           |

Recommendation: implement default, trunk work, recent work, all/repo, and custom first. Current
stack can be folded into trunk work or exposed later if it proves distinct enough.

Tradeoffs:

- Using `trunk()` respects jj config, but direct actions should validate it resolves exactly.
- `latest(mutable(), 20)` is a practical recent-work default, not a universal truth.
- Running default `jj` with no app-owned revset is the highest-fidelity default.
- Broader views can be slower in large repos; start with explicit modes rather than a permanent
  broad dashboard.

## Direct Action Policy

Direct actions are allowed when all of these are true:

1. The target is exact.
1. The command is common.
1. The result is easy to understand after refresh.
1. `jj undo` or repeating the action is a reasonable recovery path.

Recommended direct actions:

- refresh;
- copy;
- open show/diff;
- fetch;
- create new change from exact trunk;
- create new change from an exact selected revision, once selected-target UI is clear.

Recommended non-direct actions:

- push;
- abandon;
- rebase;
- squash;
- split;
- operation restore/revert;
- bookmark delete/forget;
- workspace forget.

## Help And Discoverability

Recommendation: generate keymap/help content from binding metadata as soon as possible. The help
screen should show:

- global keys;
- current screen keys;
- direct actions;
- preview/confirmation actions;
- view modes;
- the current command target when useful.

This is important because the TUI will only feel consistent if the shortcut model is visible.

## Integration Policy

Recommendation:

- Start with existing subprocess output only where it is already working.
- Introduce internal semantic models before adding more actions.
- Prefer exact code/structured contracts for action targets.
- Keep the fragility register current whenever a parser is required.
- Research `jj_cli`/`jj_lib` integration as a dedicated slice before parser-dependent log actions
  become richer mutation flows.

This lets implementation continue while producing concrete evidence about where jj needs stronger
UI-facing APIs.

## Source-Based Integration Recommendation

The adjacent `../jj` source shows enough reusable Rust surface to justify an early spike, but not
enough to assume the answer in advance. `jj_cli` publicly exposes template, formatter, graphlog, UI,
command, and revset utility modules. `jj_lib` exposes repository, revset, graph, diff, operation,
transaction, ref, and workspace semantics. The built-in config also confirms that `jk` should
preserve the configured default log revset and templates rather than replacing them with local
defaults.

Recommended answer: add a source integration spike before broadening log parsing. The spike should
try to produce log rows through `jj_cli` template/formatter/graph paths, carry semantic commit
identity alongside rendered spans, and compare the result against subprocess `jj log`. If this works
without copying large internals, it becomes the preferred path for high-fidelity log semantics. If
it does not, the exact failure becomes evidence for a smaller structured-output path or upstream
extraction.

Tradeoffs:

- using `jj_cli` gives a desirable build-break failure mode when `jj` changes, but may depend on
  APIs that are public without being stable as an external UI contract;
- using only `jj_lib` gives strong repository semantics, but risks duplicating CLI defaults,
  templates, graph rendering, style labels, and config interpretation;
- continuing with subprocess output preserves the installed `jj` behavior and backend support, but
  keeps semantic state on the fragile stdout/ANSI/parsing path.

The recommended compromise is to keep subprocess rendering for working read-only surfaces, run the
source spike early, and move semantic action targets toward code or structured contracts as soon as
they influence mutations.

## Decisions That Still Need Maintainer Approval

These recommendations are reasonable defaults. The maintainer only needs to override them if they
feel wrong.

  | Decision                                           | Recommended Answer | Alternative                                                   |
  | -------------------------------------------------- | ------------------ | ------------------------------------------------------------- |
  | Use `c` for `jj new trunk`                         | Yes                | Put new-change in action menu until keymap stabilizes         |
  | Make fetch direct                                  | Yes                | Put fetch behind a prompt if remote selection is often needed |
  | Make push preview-first                            | Yes                | Direct push only for exact selected bookmark/remote           |
  | Defer multi-select                                 | Yes                | Add multi-select before view modes if rebase comes first      |
  | Implement recent view with `latest(mutable(), 20)` | Yes                | Use configurable count or avoid recent view initially         |
  | Keep command mode secondary                        | Yes                | Make command mode a first-class launcher earlier              |

If these are acceptable, the implementation slices can be executed in order without further product
planning.
