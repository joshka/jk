# jk-tui

Ratatui views and interaction state for `jk`.

This crate owns the current jj-native TUI surface: title and status chrome, movement and expansion
actions, rendered-log conversion, and selected-row highlighting.

The log body is intentionally borderless and remains visually based on `jj` output. `jk-tui` adds
only the interaction state needed for selection, refresh, and inline details.
