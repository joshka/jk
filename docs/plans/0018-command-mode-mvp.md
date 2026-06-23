# Command Mode MVP

Scope: first `:` jj command mode slice after command history details and recovery affordances.

This slice gives dogfood users a direct escape hatch for jj-shaped commands that do not yet have a
native `jk` screen or action. It intentionally stays smaller than the full command-mode plan: run a
typed jj command, capture output, record history, and keep failures inspectable.

## Goals

- Let `:` open a prompt from normal app contexts.
- Accept commands with or without a leading `jj` prefix, such as `status` or `jj status`.
- Parse simple argv without invoking a shell.
- Run through the existing typed `JjCommandSpec` and `JjCommandRunner` path.
- Render stdout and stderr in a scrollable inspection view.
- Preserve stderr for failed commands.
- Record typed commands in Command History as user-entered command-mode actions.
- Keep command-mode help visible through `?` discovery without adding hotbar noise.

## Non-Goals

- Do not add `!` external command mode.
- Do not execute through a shell.
- Do not add persistent command history.
- Do not add retry/edit/rerun from the output view yet.
- Do not add a Run Options drawer or advanced safety policy for arbitrary typed commands yet.
- Do not update the website or public docs during the local current implementation spike.

## Interaction

From log, diff, inspection, workspaces, command history, or operation log:

1. Press `:`.
1. A centered `jj command` overlay opens above the current view.
1. Type a jj command after the implicit binary name. The command may include a leading `jj`.
1. Press Enter to run.
1. Press `Esc` to cancel.
1. Press Backspace to edit, or close the empty prompt.
1. Press `Ctrl-u` to clear the prompt.

The result opens a static rendered output view with the same navigation, paging, search, help, and
back behavior as other rendered inspection views.

## Command Parsing

The MVP parser is intentionally small and shell-free:

- whitespace separates arguments;
- single quotes group literal text;
- double quotes group text and allow backslash escaping;
- backslash outside quotes escapes the next character;
- unterminated quotes and dangling escapes stay in the prompt as validation errors;
- an optional first argument of `jj` is stripped before building the command spec.

This is enough for common revsets, filesets, templates, and message arguments without making the
terminal prompt a shell.

## Command Model

Command mode builds a `JjCommandSpec` with:

- `ExecutionMode::CommandMode`;
- `RefreshPlan::None`;
- process argv from the parsed command;
- repository global options inherited from startup `-R`, when provided;
- a display title prefixed with `:`.

The MVP uses the existing command runner directly and does not probe resulting operations. Broader
mutation support should move through command preview and Run Options before command mode becomes a
blind mutation surface.

## Command History

Command-mode records use:

- source view: `command mode`;
- source action: `command`;
- source key: `:`;
- command family: `UserJjCommand`, even when the parsed argv starts with `log`, `status`, or another
  modeled jj family.

This keeps history honest: a typed command is distinct from a built-in view loader.

## Output View

The command output view renders:

- exact redacted process command;
- status (`success`, `exit N`, or `spawn error`);
- stdout, with `<empty>` for no bytes;
- stderr, with `<empty>` for no bytes.

Failed commands must keep stderr visible. This is the main dogfood value of the MVP because it makes
bad revsets, unsupported flags, and missing commands diagnosable inside jk.

## Evidence

Unit coverage:

- parser behavior for whitespace, quotes, escapes, and validation errors;
- optional `jj` prefix handling;
- empty Enter keeps the prompt open with an error;
- command-mode execution records `UserJjCommand` history;
- failed output preserves stderr;
- `:` maps through the app key adapter;
- `:` appears in help/discovery across supported contexts.

Betamax coverage:

- `target/jk-artifacts/command-mode.tape`;
- `target/jk-artifacts/command-mode.gif`;
- prompt, success, and failure screenshots under `target/jk-artifacts/`.

## Follow-Ups

1. Add retry/edit from the command output view.
1. Define success refresh policy without re-running arbitrary commands unexpectedly.
1. Add richer prompt editing and history recall.
1. Add Run Options integration before exposing advanced operation-context and working-copy flags.
1. Add `!` external command mode as a separate shell-free argv surface.
