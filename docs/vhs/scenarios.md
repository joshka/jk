# VHS Scenario Plan

This file defines how `jk` tutorial captures are authored.

## Goals

1. Keep static docs crisp with screenshots instead of short flicker GIFs.
1. Use longer GIFs only where interaction transitions matter.
1. Keep captures aligned with common `jj` day-one workflows.
1. Maintain explicit 25-item tutorial coverage with one artifact per tutorial item.

## Theme and Rendering Defaults

- Theme: `Aardvark Blue`
- Width: `1280`
- Height: `760`
- Font size: `20`
- Padding: `24`
- Framerate: `30` for dynamic tapes, `24` for static-capture tape
- Typing speed: `45ms`

## Static Capture Scenario

- Tape: `docs/vhs/tutorial-static-screens.tape`
- Output GIF: `target/vhs/tutorial-static-screens.gif` (not primary artifact)
- Primary artifacts: `target/vhs/static-*.png` screenshots for key high-frequency views

## Full Tutorial Catalog Capture

- Tape: `docs/vhs/tutorial-full-catalog.tape`
- Output GIF: `target/vhs/tutorial-full-catalog.gif` (not primary artifact)
- Primary artifacts: `target/vhs/tutorial-01-*.png` ... `target/vhs/tutorial-25-*.png`
- Purpose: preserve one concrete visual artifact for every tutorial item.

## Dynamic Scenarios

1. Navigation and inspect:
   - Tape: `docs/vhs/tutorial-dynamic-navigation.tape`
   - GIF: `target/vhs/tutorial-dynamic-navigation.gif`
   - Shows item movement, show/diff entry, and return/back flow.
1. Command mode and history:
   - Tape: `docs/vhs/tutorial-dynamic-command-history.tape`
   - GIF: `target/vhs/tutorial-dynamic-command-history.gif`
   - Shows `:` command execution, help paging, and back/forward navigation.
1. Safety and mutation prompts:
   - Tape: `docs/vhs/tutorial-dynamic-safety.tape`
   - GIF: `target/vhs/tutorial-dynamic-safety.gif`
   - Shows prompt-mode and confirm-mode flows with explicit cancel/reject behavior.
1. Remote and operation flow:
   - Tape: `docs/vhs/tutorial-dynamic-remote-ops.tape`
   - GIF: `target/vhs/tutorial-dynamic-remote-ops.gif`
   - Shows fetch/push prompt paths and operation-log context switch.

## Authoring Rules

1. Prefer screenshots for static state demonstrations.
1. For dynamic tapes, include enough dwell time (`>= 1.3s`) after each meaningful transition.
1. Keep each dynamic tape focused on one narrative loop.
1. Do not include destructive accepts in tutorial tapes; use cancel/reject paths.
1. Update `docs/tutorial-vhs.md` when scenario scope changes.
1. Run captures with explicit color env:
   `env -u NO_COLOR CLICOLOR_FORCE=1 COLORTERM=truecolor TERM=xterm-256color`.
