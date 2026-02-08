# jk vs jjui Deep Screen + Mode Analysis (2026-02-08)

## Goal

Provide an implementation-level comparison focused on:

1. which screens/modes `jjui` actually implements (from code, not just README),
1. why those interactions feel strong for daily `jj` use,
1. what `jk` should adopt first.

## Evidence Used

`jjui` internals (local clone):

1. `/tmp/jjui/internal/ui/ui.go`
1. `/tmp/jjui/internal/ui/revisions/revisions.go`
1. `/tmp/jjui/internal/ui/revisions/displaycontext_renderer.go`
1. `/tmp/jjui/internal/ui/render/list.go`
1. `/tmp/jjui/internal/ui/revset/revset.go`
1. `/tmp/jjui/internal/ui/oplog/operation_log.go`
1. `/tmp/jjui/internal/ui/preview/preview.go`
1. `/tmp/jjui/internal/ui/split.go`
1. `/tmp/jjui/internal/ui/status/status.go`
1. `/tmp/jjui/internal/ui/operations/details/details.go`
1. `/tmp/jjui/internal/ui/operations/evolog/evolog_operation.go`
1. `/tmp/jjui/internal/ui/git/git.go`
1. `/tmp/jjui/internal/ui/bookmarks/bookmarks.go`
1. `/tmp/jjui/internal/ui/custom_commands/custom_commands.go`
1. `/tmp/jjui/internal/config/default/config.toml`

`jk` internals:

1. `src/app/runtime.rs`
1. `src/app/input/mod.rs`
1. `src/app/input/normal.rs`
1. `config/keybinds.default.toml`
1. `README.md`

## jjui Implementation Rundown (What Exists Today)

### 1) Root Screen Orchestrator

`jjui` has one root UI model that composes all major sub-models and routes input by focus
precedence.

- Main surfaces: revisions, op log, revset line, preview pane, status line.
- Overlay stack: git/bookmarks/custom commands/choose/input/undo/redo/password/diff.
- Route guardrails: keys first hit focused editors/overlays before global commands.

Files:

1. `/tmp/jjui/internal/ui/ui.go`

### 2) Revision-Centric Main Screen With Operation Modes

The revisions screen is not just a static list. It has a default mode plus operation-specific
modes (rebase/squash/revert/duplicate/details/evolog/describe/ace jump/set-parents/etc.).

- Cursor is item-based while rendering supports variable-height rows.
- Supports checked-item selection for batch operations.
- Supports quick-search cycle and multiple navigation targets (parent/child/working copy).

Files:

1. `/tmp/jjui/internal/ui/revisions/revisions.go`
1. `/tmp/jjui/internal/ui/operations/default_operation.go`
1. `/tmp/jjui/internal/ui/operations/operation.go`

### 3) Viewport + List Engine That Separates Item Selection From Scroll Offset

`ListRenderer` tracks `StartLine` (viewport) separately from `cursor` (item index), which avoids
mixing line-scrolling with item selection semantics.

- Page movement translates to item-step by visible span.
- Mouse click/scroll are attached as interactions to rendered item rectangles.

Files:

1. `/tmp/jjui/internal/ui/render/list.go`
1. `/tmp/jjui/internal/ui/revisions/displaycontext_renderer.go`

### 4) First-Class Revset Editing Mode

Revset editing is its own mode with autocomplete provider, history, cycling, and preview-before-
commit behavior.

- Tab/Shift-Tab and Up/Down cycle completion candidates.
- Enter applies, Esc cancels, history is persisted.

Files:

1. `/tmp/jjui/internal/ui/revset/revset.go`

### 5) Persistent Preview Pane (Split + Scroll + Resize)

Preview is a durable pane, not a one-shot command replacement.

- Toggle visibility.
- Auto side/bottom positioning.
- Independent vertical/horizontal scroll.
- Resize controls and mouse-draggable split separator.

Files:

1. `/tmp/jjui/internal/ui/preview/preview.go`
1. `/tmp/jjui/internal/ui/split.go`
1. `/tmp/jjui/internal/ui/ui.go`

### 6) Dedicated Op Log Screen Model

Operation log is a proper screen model with navigation, restore/revert actions, and diff inspection.

Files:

1. `/tmp/jjui/internal/ui/oplog/operation_log.go`

### 7) Status Line as Interaction Surface

Status is not passive text. It owns:

- command running state + spinner/success/failure,
- expanded help overlay,
- quick-search input,
- exec-jj/exec-shell input modes,
- file fuzzy-search entry.

Files:

1. `/tmp/jjui/internal/ui/status/status.go`

### 8) High-Value Overlay Screens

`jjui` ships several focused overlays that preserve main-context selection:

1. details overlay with file-level actions and confirm dialogs,
1. bookmarks menu with remote-aware filtering,
1. git menu with remote cycling and filtering,
1. custom commands menu with key shortcuts and sequences.

Files:

1. `/tmp/jjui/internal/ui/operations/details/details.go`
1. `/tmp/jjui/internal/ui/bookmarks/bookmarks.go`
1. `/tmp/jjui/internal/ui/git/git.go`
1. `/tmp/jjui/internal/ui/custom_commands/custom_commands.go`

### 9) Evolog as a Two-Mode State Machine

Evolog has explicit select mode and restore mode. This keeps risky actions explicit and makes the
UI state obvious.

File:

1. `/tmp/jjui/internal/ui/operations/evolog/evolog_operation.go`

## Where jk Is Currently Simpler

`jk` has strong foundations but flatter mode structure:

1. global app mode is `Normal|Command|Confirm|Prompt`,
1. command output generally replaces the body view,
1. navigation and rendering are robust but mostly single-surface,
1. current power-user features are mostly command-driven rather than pane/mode-driven.

Files:

1. `src/app/mod.rs`
1. `src/app/runtime.rs`
1. `src/app/input/mod.rs`

## Practical Opportunities for jk (Mode Changes First)

### Tier A: Highest ROI

1. Add explicit `Screen` + `Overlay` state machine.
   - Keep `Mode` for input, add `Screen` for content ownership.
   - This avoids feature coupling and clarifies back/forward policy.
1. Introduce a dedicated `LogScreen` model with item cursor + viewport offset.
   - Keep cursor on revision item boundaries only.
   - Keep scroll offset independent from selected item index.
1. Add revset mode (`L`) as first-class, not command-line only.
   - History + autocomplete + apply/cancel parity.
1. Add persistent preview pane (`p`) with split resizing.
   - Start simple: toggle + scroll + width/height steps.

### Tier B: Strong UX Lift

1. Add details overlay (`l`/right) for file-level actions on selected revision.
1. Add quick search mode (`/`) with next/prev match cycle.
1. Add checked-item multi-select for batch rewrite commands.
1. Add op-log screen model that keeps operation selection typed and actionable.

### Tier C: Power-User Expansion

1. Add custom command palette/registry (declarative first, scripting later).
1. Add ace-jump for long log views.

## Concrete Vertical Slices for jk

1. Slice 1: `Screen` enum + routing + back/forward invariants.
   - Keep behavior same, just move state ownership.
1. Slice 2: `LogScreen` with strict item navigation.
   - Snapshot-test j/k/page/home/end transitions.
1. Slice 3: revset mode.
   - Snapshot-test mode entry/edit/apply/cancel.
1. Slice 4: preview pane split.
   - Snapshot-test toggle, resize, scroll interactions.
1. Slice 5: details overlay.
   - Snapshot-test open/close/file cursor/action prompts.

## Snapshot Coverage Additions (Critical)

Add transition snapshots that assert:

1. down/up always moves item-to-item on log screen (never line drift),
1. page-up/page-down land on valid item boundaries,
1. help/keymap use scroll-only semantics (no item selection),
1. back/forward returns to prior screen + cursor context,
1. preview toggle does not lose main-screen selection,
1. entering/exiting overlays restores prior focus target.

## Status

1. `considered` - deep implementation inventory complete.
1. `considered` - mode/screen adoption plan for `jk` drafted.
1. `todo` - choose the first vertical slice and implement.
