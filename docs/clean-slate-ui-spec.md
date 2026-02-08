# jk Clean-Slate UI Spec

Status: Draft

Audience: maintainers and contributors building a new `jk` UI from first principles.

## Purpose

Define a clean-slate UX and architecture for `jk` optimized for real `jj` workflows, without
relying on heavy dashboard layouts or permanent multi-panel chrome.

This spec intentionally does not inherit implementation constraints from current `jk` or `jjui`.

## Product Thesis

`jk` should feel like the fastest possible interactive shell for `jj` workflows:

1. one focused surface at a time,
1. modeful keyboard interaction,
1. strong safety for mutations,
1. instant recovery and reversibility,
1. low visual noise and high semantic density.

## UX Principles

1. Single-surface first.
   Only one primary content canvas is active at a time.
1. Overlays, not dashboards.
   Secondary tasks use ephemeral overlays/sheets, then return to previous context.
1. Item semantics over line semantics.
   Selection movement always targets logical items where selection exists.
1. Predictable navigation.
   Back/forward history is explicit and consistent across screens.
1. Safety by default.
   Risky commands require a visible preview and explicit confirmation.
1. Progressive disclosure.
   Novice hints appear in context; power-user paths remain one keystroke away.

## Information Architecture

Top-level UI entities:

1. `Screen`: owns primary canvas content.
1. `Overlay`: temporary surface on top of a screen.
1. `Mode`: input grammar active at any moment.
1. `Selection`: typed selected entity for current screen.
1. `History`: back/forward stack of navigable screen states.

The app should never infer behavior from output text alone if a typed state exists.

## Screen Model

### 1. Timeline Home

Goal: inspect change graph and pick the next action.

Primary content:

1. revision graph rows,
1. selected revision metadata summary,
1. optional lightweight preview strip.

Primary actions:

1. inspect (`show`, `diff`, `evolog`),
1. mutate (`describe`, `rebase`, `squash`, `split`, `abandon`),
1. jump (`search`, `jump`, `top`, `bottom`).

### 2. Inspect Screen

Goal: deep inspection of selected revision content.

Primary content:

1. `show`/`diff` output,
1. scrollable body with preserved ANSI styling.

Primary actions:

1. switch inspect mode (`show` <-> `diff`),
1. navigate to previous/next revision,
1. return to timeline.

### 3. Action Composer

Goal: run multi-step rewrite actions with validation.

Primary content:

1. action form state,
1. computed command preview,
1. impact summary.

Primary actions:

1. edit parameters,
1. preview resulting command,
1. execute or cancel.

### 4. File Details Overlay

Goal: file-level operation loop for selected revision.

Primary content:

1. changed files list,
1. per-file state markers,
1. action hints.

Primary actions:

1. select files,
1. restore/split/diff file-level operations,
1. exit back to underlying screen.

### 5. Operation Log Screen

Goal: recovery and audit after mutation.

Primary content:

1. operation list,
1. selected operation summary,
1. recovery hints.

Primary actions:

1. inspect operation,
1. restore/revert operation,
1. jump back to timeline.

### 6. Revset Studio

Goal: build and apply revsets quickly and correctly.

Primary content:

1. revset input,
1. completion list,
1. history list.

Primary actions:

1. autocomplete/cycle candidates,
1. apply/cancel,
1. pin common revsets.

### 7. Command Palette

Goal: discover and launch commands by intent.

Primary content:

1. fuzzy-matched commands,
1. aliases and custom actions,
1. preview of selected command intent.

Primary actions:

1. run command,
1. open command docs/help,
1. save to recents.

### 8. Help and Tutorial Screen

Goal: make intent-to-action mapping obvious.

Primary content:

1. workflow-first command groups (`Inspect`, `Rewrite`, `Sync`, `Recover`),
1. keymap by context,
1. practical next-step prompts.

Primary actions:

1. scroll help,
1. jump between workflow sections,
1. open related screen directly.

## Mode Model

Input modes:

1. `Normal`: navigation and direct actions.
1. `Search`: in-view filtering and match navigation.
1. `Compose`: structured command/form editing.
1. `Prompt`: single-field or multi-field input capture.
1. `Confirm`: explicit accept/reject for risky operations.

Global mode invariants:

1. `Esc` cancels current mode and returns one level.
1. `Enter` submits the active intent in non-`Normal` modes.
1. mode transitions are visible in a single mode badge location.

## Navigation Contract

### Selection vs Scroll

1. selectable screens move by item (`j/k`, arrows, page keys, home/end).
1. non-selectable screens scroll only; they do not show a selected row marker.

### Back and Forward

1. every screen transition records a typed history entry.
1. back/forward restores:
   - prior screen,
   - prior cursor item,
   - prior viewport offset,
   - prior revset scope when applicable.

### Paging

1. `PageUp`/`PageDown` move by viewport span and land on valid item boundaries.
1. `Ctrl+u`/`Ctrl+d` perform half-page movement where meaningful.

## Visual Design Language

Layout:

1. Header bar: repository + mode badge + current scope.
1. Body: primary content canvas.
1. Footer bar: contextual shortcuts grouped by intent.

Style constraints:

1. avoid dense border/box UI patterns.
1. use spacing, typography weight, and color hierarchy before separators.
1. reserve accent colors for semantic states:
   - selected,
   - warning/risk,
   - success,
   - error.
1. preserve ANSI content colors from `jj` output in content regions.

## Safety and Trust Model

Mutation flow policy:

1. every dangerous mutation enters `Confirm` mode.
1. confirm screen shows:
   - exact command,
   - best-effort preview,
   - clear consequences.
1. cancel path must always be cheap and obvious.

Recovery policy:

1. operation log is always one gesture away.
1. undo/redo entries are visible in contextual hints after mutations.

## Power-User Features

Required power-user affordances:

1. fast command palette,
1. revset autocomplete with history,
1. quick search and jump,
1. batch selection for repeated actions,
1. optional custom command registry.

These features should never degrade novice navigation clarity.

## Runtime Architecture (Recommended)

State machine types:

1. `AppState`: root state container.
1. `ScreenState`: enum with per-screen structs.
1. `OverlayState`: optional overlay enum.
1. `ModeState`: input mode enum + mode-local state.
1. `SelectionState`: typed selection union.
1. `HistoryState`: back/forward stacks of serializable navigation entries.

Rendering contract:

1. each screen renders from typed state only,
1. each screen defines its own key handling map,
1. global shortcuts are resolved after focused mode/screen handlers decline.

## Delivery Plan (Vertical Slices)

1. Slice 1: state machine foundation (`Screen`, `Mode`, `History`, typed selection).
1. Slice 2: timeline screen with item navigation + paging guarantees.
1. Slice 3: inspect screen and clean back/forward restoration.
1. Slice 4: confirm mode with mutation preview and safety tests.
1. Slice 5: revset studio and command palette.
1. Slice 6: file details overlay + operation log recovery loop.

Each slice must include behavioral snapshots for transitions and key flows.

## Test Specification

Critical behavior tests:

1. item navigation monotonicity (`down/down/up` on 3+ items),
1. page movement lands on valid item boundaries,
1. scroll-only screens do not expose row selection,
1. back/forward restores cursor and viewport context,
1. confirm mode blocks execution until explicit accept,
1. cancel from any mode returns to previous stable mode.

## Non-Goals

1. full mouse-first UX parity,
1. permanent multi-panel dashboard layout,
1. broad plugin API in v1.

## Success Criteria

1. novice can complete inspect -> act -> verify -> recover in under 5 minutes without docs.
1. power user can navigate and execute common rewrite flows without leaving keyboard home row.
1. no navigation ambiguity between item selection and line scrolling.
1. no unsafe mutation can run without explicit confirmation.
