# jk-core

Shared log records and small text helpers for `jk`.

This crate keeps the first shared contract narrow: a rendered `jj` log body plus semantic records
for navigation, selection preservation, and inline expansion.

The rendered text remains opaque `jj` output. Structured fields exist only where the TUI needs
stable state that cannot be recovered safely from terminal text.
