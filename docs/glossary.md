# Glossary

This glossary keeps `jk` terms close to normal `jj` behavior.

## runs now

`jk` executes immediately from the current context without asking extra questions.

Examples: `log`, `status`, `show`, `diff`.

## opens prompt

`jk` asks for one or more inputs first, then runs the equivalent `jj` command.

Examples: `new`, `commit`, `describe`, `bookmark set`.

## asks confirmation

`jk` requires an explicit `y` before running a high-impact command.

Examples: `rebase`, `squash`, `split`, `git push`, `operation restore`.

## runs as jj

For some commands, `jk` runs the direct `jj` command path and shows the output in the app.
This is mostly used for long-tail commands that do not yet have a dedicated flow.
