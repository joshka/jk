# Screen Guide

This document is the canonical reference for every `jk` screen and interaction mode.
It is organized for day-one users first, then power-user patterns.

## Design Contract

- Navigation is modal and pager-first: low chrome, no dashboard boxes.
- `j/k` and arrow keys move by logical item in log-like views.
- `Left/Right` (or `Ctrl+o/i`) traverses visited screens.
- `:` always provides an exact-command escape hatch.

## VHS Previews

The GIFs below are generated locally into `target/vhs/` (gitignored).
For full tutorial coverage, see `docs/tutorial-vhs.md`.

![Log flow](../target/vhs/log-flow.gif)
![Status flow](../target/vhs/status-flow.gif)
![Help flow](../target/vhs/help-flow.gif)
![History flow](../target/vhs/history-flow.gif)

Regenerate with:

```bash
docs/vhs/render.sh
```

## Screen Index

| Screen | Enter via | Purpose | Primary actions |
| --- | --- | --- | --- |
| `Log` | `jk`, `l`, `:log` | Inspect history and revisions | `Enter` show, `d` diff, `p` patch |
| `Status` | `s`, `:status` | Inspect working copy state | `l` return home, `F` fetch, `P` push |
| `Show` | `Enter`, `:show <rev>` | Inspect selected revision details | `d` diff, `Left` back |
| `Diff` | `d`, `:diff -r <rev>` | Inspect selected patch | `Enter` show selected, `Left` back |
| `Operation Log` | `o`, `:operation log` | Audit operations and recover | `u` undo, `U` redo |
| `Operation Show` | `:operation show` | Inspect one operation record | `Left` back, `o` op log |
| `Operation Diff` | `:operation diff` | Inspect operation-level patch | `Left` back, `o` op log |
| `Bookmark List` | `L`, `:bookmark list` | Inspect bookmark state | `b` set bookmark, `Left` back |
| `Resolve List` | `v`, `:resolve -l` | Inspect merge conflicts | `Left` back, `:resolve` mutate |
| `File List` | `f`, `:file list` | Inspect tracked paths | `:file show`, `:file search` |
| `File Show` | `:file show <path>` | Inspect file at revision | `Left` back |
| `File Search` | `:file search <pattern>` | Search files in revision | `Left` back |
| `File Annotate` | `:file annotate <path>` | Line-level blame/annotate view | `Left` back |
| `Tag List` | `t`, `:tag list` | Inspect tags | `:tag set`, `:tag delete` |
| `Workspace Root` | `w`, `:root` | Confirm current workspace path | `l` log, `s` status |
| `Command Registry` | `?`, `:commands` | Discover commands and run behavior | `PgUp/PgDn`, query |
| `Keymap` | `K`, `:keys` | Discover active key bindings | filter by action/key |
| `Alias Catalog` | `A`, `:aliases` | Discover alias mappings | filter by alias/expansion |
| `Prompt Mode` | prompt-based actions | Collect command input | `Enter` submit, `Esc` cancel |
| `Confirm Mode` | dangerous flows | Confirm risky mutation | `y` accept, `n`/`Esc` reject |

## Per-Screen Behavior

### Log

- Role: home screen for day-to-day work.
- Selection: moves by revision item, not by individual wrapped text row.
- Primary intent paths:
  - inspect: `Enter` -> `Show`, `d` -> `Diff`
  - mutate selected: `D`, `b`, `a`, `B`, `S`, `X`, `O`, `R`
  - switch context: `s` status, `o` operation log

### Status

- Role: working-copy triage and next-action decision point.
- Primary intent paths:
  - return to history: `l`
  - sync remote: `F` fetch, `P` push
  - mutation follow-up: choose target revision in log, then mutate

### Show and Diff

- Role: detailed revision inspection surfaces.
- Expectation: selected revision in log should map directly to these views.
- Backtracking: use `Left`/`Ctrl+o` to return to the previous screen.

### Operation Screens

- `Operation Log`: operation history for undo/redo and audit.
- `Operation Show`: per-operation metadata.
- `Operation Diff`: per-operation patch impact.

### Bookmark, Resolve, File, Tag, Root

- These are focused utility screens for high-frequency maintenance.
- They intentionally preserve low visual chrome and command-driven follow-through.
- They are screen-history aware, so moving back to the prior context is one key.

### Command Registry, Keymap, Alias

- `:commands` / `?`: intent-first help with day-one flows first.
- `:help inspect|rewrite|sync|recover`: workflow-scoped help presets.
- `:keys`: exact, runtime-resolved bindings for normal/command/confirm modes.
- `:aliases`: discover normalization behavior from short aliases to canonical commands.

### Prompt and Confirm Modes

- Prompt mode captures required input with minimal interruption.
- Confirm mode is explicit and previews risky mutations where possible.
- Both modes advertise submit/cancel semantics directly in the footer.

## Power-User Flow Patterns

1. History-first loop:
   `l` -> select item -> `Enter`/`d` -> `Left` back -> next item.
1. Status triage loop:
   `s` -> scan -> `l` -> mutate selected revision -> `s` verify.
1. Rewrite safety loop:
   select -> mutate (`B`/`S`/`X`) -> confirm -> `o` inspect op log -> `u` if needed.
1. Remote sync loop:
   `s` -> `F` -> `P` -> `l` or `s` verify.

## UX Acceptance Checklist

Use `docs/navigation-behavior-checklist.md` for regression checks.
