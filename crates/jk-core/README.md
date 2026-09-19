# jk-core

Shared state and provider-neutral contracts for `jk`.

This crate keeps the first shared contract narrow: a rendered `jj` log body plus semantic records
for navigation, selection preservation, and inline expansion.

The rendered text remains opaque `jj` output. Structured fields exist only where the TUI needs
stable state that cannot be recovered safely from terminal text.

Selector resolution also lives here. Views keep ownership of cursor, marks, search, and refresh
state; workflows submit those choices to a pure resolver that reports resolved, ambiguous, invalid,
or cancelled outcomes without constructing jj commands.
