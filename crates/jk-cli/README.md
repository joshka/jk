# jk-cli

Command-line and `jj` process integration helpers for `jk`.

This crate owns the temporary shell-out boundary used by the log-first MVP. It runs `jj` once for
CLI-equivalent rendered output and once with a narrow JSON template for semantic records.

The long-term direction is direct `jj-cli` / `jj-lib` integration when that can preserve user
templates, colors, graph rendering, revsets, and command semantics without duplicating `jj` display
logic.
