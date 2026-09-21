# jk-core

Revision, selection, command, and history models shared by `jk` crates.

Snapshots pair rendered `jj` output with metadata for navigation, selection recovery, and inline
details. Command specifications describe arguments, execution policies, and previews; command history
records execution results.

The rendered text remains opaque `jj` output. Structured fields exist only where the TUI needs
stable identity and navigation data that cannot be recovered safely from terminal text.

Selector resolution also lives here. Views keep ownership of cursor, marks, search, and refresh
state; workflows submit those choices to a pure resolver that reports resolved, ambiguous, invalid,
or cancelled outcomes without constructing jj commands. The crate does not launch processes or
render terminal widgets.
