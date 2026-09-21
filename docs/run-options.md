# Run options

From a rebase, squash, or restore command preview, press `o` to edit execution options for that
command. The repository stays fixed to the workspace where you selected its revisions. The drawer
shows that repository, the execution settings, and the complete command. Use Page Up/Down to scroll
long text.

1. Move with Up/Down or `j`/`k`; press Enter to change the selected option.
1. Choose whether jj snapshots and updates working-copy files, or ignores them.
1. For rebase or squash, optionally enter a specific operation ID with at least 12 hexadecimal
   digits. Clear the field with Ctrl-u to return to latest. Press Enter to finish editing it.
1. Leave immutable commits protected, or explicitly allow rewriting them for this command.
1. Press `a` to apply to the command preview. Review its updated command and warnings, then press
   Enter to execute. Applying options does not run the command.

Escape discards the drawer's draft, including while editing the operation ID. Applying changes only
the pending preview. Cancelling the preview discards all its options; the next command starts with
its own defaults. Existing configuration overlays and output policy are preserved.

Historical execution ignores working-copy files and can create concurrent history. Rebase and
squash retain the exact commit operands chosen before opening Run options; jj reports an error if
they are unavailable at that operation. Restore keeps the latest operation because its `@`
destination would otherwise refer to a different historical working-copy change.

After success, jk refreshes the latest graph. For historical execution or ignored working-copy
files, that refresh also skips snapshotting; later ordinary refreshes use normal jj behavior.
Command History retains the executed options and diagnostic output.

Run options is available only for rebase, squash, and restore previews. It does not change workspace
commands, network commands, external tools, or command-mode execution. Repository switching,
config/config-file editing, and output-policy editing are not available. Normal operation
integration stays enabled; detached operations (`--no-integrate-operation`) need a separate result
and recovery workflow before jk can offer them here.

## Validation

Tests in [`run_options.rs`](../crates/jk/src/run_options.rs) cover global-option ordering, preserved
operands/config/output, historical and immutable warnings, and excluded commands. They also check
cancel/apply behavior, history remaining untouched before execution, a new confirmation after
applying, invalid operation IDs, and restore's fixed destination. Terminal-buffer tests exercise
40x14, 80x24, and 120x40 layouts, scrolling, and disabled controls when the terminal is too small.

The [Run options tape](../tapes/run-options.tape) and
[narrow tape](../tapes/run-options-narrow.tape) create disposable repositories, open a rebase preview,
change working-copy policy, and check cancel/apply behavior. Each checkpoint captures terminal-state
JSON and a PNG; the [proof checker](../scripts/check-run-options-proof.py) compares commands before
and after. Run these scenarios against fresh repositories, never the development graph.
