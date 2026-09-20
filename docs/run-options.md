# Run options

From a rebase, squash, or restore command preview, press `o` to edit execution options for that
command. The repository stays fixed to the workspace that supplied its selected operands. The
drawer shows the repository, affected policy, and complete command; Page Up/Down scroll long text.

1. Move with Up/Down or `j`/`k`; press Enter to edit the selected option.
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
destination would otherwise refer to a different historical working-copy change. Success returns
to a refreshed latest graph; the refresh does not snapshot working-copy files. Command History
retains the executed options and diagnostic output.

This first slice does not expose repository switching, detached operations
(`--no-integrate-operation`), config/config-file editing, or output-policy editing. It is available
only for the supported local mutation previews, not workspace, network, external-tool, or arbitrary
command-mode execution. Detached operations need their own result and recovery flow before a
control can accurately promise that behavior.

## Validation

The pure option tests cover exact global-option ordering, preserved operands/config/output,
historical and immutable warnings, and excluded command scopes. State tests cover cancel/apply,
history remaining untouched before execution, fresh confirmation, invalid operation IDs, and
restore's fixed destination context. Terminal-buffer tests exercise 40x14, 80x24, and 120x40 layouts,
scrolling, and unavailable controls when the terminal is too small.

Betamax proof should use a new disposable repository: open a rebase preview, press `o`, change the
working-copy policy, apply, and capture both terminal state and a PNG of the updated preview. Also
capture cancellation and a narrow drawer. Never use the development graph as scenario data.
