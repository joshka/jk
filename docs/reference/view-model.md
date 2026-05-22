# View Model

This document describes how `jk` should present information across screens. It exists to keep the
app from drifting into a fixed-pane dashboard while still allowing richer, more informative views
than a plain one-line list.

## Core Principle

`jk` should feel like a terminal-native extension of `jj`, not like a GUI dashboard squeezed into
boxes.

The preferred order is:

1. single active surface
1. inline expansion inside that surface
1. temporary overlay
1. optional split for a specific screen

The preferred model is not:

1. fixed left nav pane
1. fixed right detail pane
1. fixed bottom action pane
1. permanent boxed layout everywhere

## Hybrid Views

Hybrid views are allowed and encouraged when they make a screen more informative without creating a
dashboard.

Examples:

- log rows that can expand inline to show body text, file headings, commit id, bookmarks, or other
  secondary metadata
- status sections that expand to show file details or suggested next actions
- bookmark or file list entries that reveal more context inline before opening a separate detail
  screen

This is a better fit for `jk` than forcing every inspection step into either:

- a one-line list with no context, or
- a fixed sibling pane that is always present whether it is useful or not

## Splits

Splits are not banned. They are a tool, not the default product model.

An under-split or side-split is acceptable when all of the following are true:

- the user benefits from keeping selection context visible while inspecting richer detail
- the detail is tightly coupled to the selected item
- the split is local to that screen, not a global frame around the whole app
- the screen still reads as one task, not as multiple dashboard widgets

Examples that may justify a split later:

- log with a transient inline-or-under detail preview mode
- bookmark list with a lower detail preview for the selected bookmark
- file list with under-preview of file contents

Examples that do not justify a split:

- making every screen inherit a permanent left list plus right detail layout
- adding fixed panes merely because other terminal apps do it

## Boxes And Chrome

Low chrome remains the bias. The app should not create heavy visual framing around everything.

Good uses of framing:

- temporary overlays
- confirmations
- prompts
- help surfaces
- a rare detail split when it clarifies one screen

Bad uses of framing:

- every view rendered as a separate persistent boxed region
- decorative panelization that consumes space without improving comprehension

## Detail Hierarchy

When deciding how much information a screen should show, prefer this order:

1. essential summary always visible
1. expanded inline detail on demand
1. dedicated detail screen for deep inspection
1. optional local split when inline expansion is not enough

This keeps the app terminal-native while still allowing richer context than the raw CLI often shows
at first glance.

## View Scope

The default log scope should stay close to what `jj` already shows by default: current useful work
from trunk or main, shaped by user configuration. `jk` should also make it easy to switch to broader
scopes when the user needs orientation across many repositories or branches.

Useful scopes include:

- default work view;
- trunk-focused work view;
- recent work view;
- all/repo overview;
- custom revset view.

View scope should be visible in status/title chrome. Switching scope should preserve selection by
semantic identity when possible.

## Defense Of The Model

This framing is not just preference. It has product reasons:

- terminal space is scarce, and fixed panes spend it continuously
- fixed panes create focus-management work for the user
- `jj` workflows are often sequential: inspect, drill down, return, mutate, verify
- a single-surface or inline-expanded model matches that sequential flow better than a dashboard
- `jk` should earn complexity only where it clearly beats shell ping-pong

## Design Filter

Before adding a split or a second visible region to a screen, ask:

1. Could this be solved with inline expansion?
1. Could this be solved with a richer dedicated screen?
1. Does the split improve comprehension enough to justify its permanent space cost?
1. Is the split local to one screen, or is it trying to become the whole app model?

If the answer to the last question is yes, it is probably the wrong direction.
