# Auto-Refresh Policy

## Why

Auto-refresh is part of the ideal first-use workflow, but it should not precede the manual refresh
model. The policy needs to avoid flicker and thrash while editors, shells, and coding agents are
writing repository state.

## Scope

Design and implement auto-refresh only after manual refresh is stable in the log view, and ideally
after the selected-change diff view has the same manual refresh shape.

## Policy Questions

- Which paths or events should trigger refresh: working copy files, `.jj/`, Git refs, or command
  output changes?
- What debounce window coalesces editor/agent writes without making the view feel stale?
- How should refresh behave while the user is scrolling or reading expanded details?
- Should auto-refresh be on by default, opt-in, or configurable per invocation?
- How should failures be surfaced without stealing focus?

## Done When

- Manual refresh remains available and predictable.
- Auto-refresh has tests for debounce/coalescing behavior.
- Selection, scroll, and expansion preservation follow the same rules as manual refresh.
- The status bar makes the refresh mode visible without adding noisy chrome.
