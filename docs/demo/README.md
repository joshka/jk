# Demo Capture Specs

This directory tracks the VHS specs and setup helper for `jk` demo captures. The specs are the
source of truth; generated media stays out of the repository.

## Capture Rules

- Use `target/demo-repos/` for disposable jj repositories created by the setup helper.
- Render media into `target/vhs/`.
- Keep setup noise hidden, use a dark theme, stay at or below 1200 px width, and leave enough dwell
  time for the core workflow to be readable.
- Prefer one focused workflow per capture.
- Host or attach the rendered media externally instead of committing GIFs or screenshots.

## Current Specs

- `static-log.tape` captures the default graph view plus shipped `j`, `s`, `h`, and `q` navigation.
- `operation-recovery.tape` captures operation-log startup plus shipped `u` undo handling as the
  safety/recovery example.

## Workflow

1. Prepare the disposable demo repositories.

   ```sh
   just demo-setup
   ```

2. Validate the tracked tapes without rendering media.

   ```sh
   vhs validate docs/demo/*.tape
   ```

3. Render the captures.

   ```sh
   just demo-static-log
   just demo-operation-recovery
   ```

The helper recipes keep generated repo state under `target/demo-repos/` and rendered media under
`target/vhs/`.
