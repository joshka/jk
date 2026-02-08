# Glossary

## guided

A command flow where `jk` asks for extra input before execution (for example via prompt mode), then
builds final `jj` tokens.

Examples:

- `rebase` asks for a destination revset.
- `bookmark set` asks for a bookmark name tied to the selected revision.

## native

A command flow where `jk` executes directly and renders output with an in-app wrapper view
(header/summary/tips) instead of showing raw passthrough output.

Examples:

- `status`, `show`, `diff`
- wrapper-rendered read views such as `operation log` and `file list`

## tier A

Read-only or low-risk commands that do not require confirmation in normal operation.

Examples:

- `log`, `show`, `status`
- `operation log`, `resolve -l`

## tier B

Commands that can mutate state but are generally not considered high-risk destructive operations.

Examples:

- `new`, `commit`, `bookmark track`, `git fetch`

## tier C

High-risk mutation commands that require explicit confirmation in `jk` before execution. `jk` shows
best-effort preview output when possible before the user confirms.

Examples:

- `git push`
- `rebase`, `squash`, `split`
- `operation restore`, `operation revert`
