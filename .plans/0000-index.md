# Plan Index

Short, numbered follow-up notes for the reset line. These are ordered by what should be made solid
before expanding the TUI surface.

| File                                                                           | Topic                        | Outcome                                                                        |
| ------------------------------------------------------------------------------ | ---------------------------- | ------------------------------------------------------------------------------ |
| [`0001-trusted-publishing.md`](0001-trusted-publishing.md)                     | crates.io trusted publishing | Delete the long-lived crates.io API token after CI proves OIDC publishing.     |
| [`0002-branch-protection.md`](0002-branch-protection.md)                       | required CI and automerge    | Make `Check`, `Markdown`, and `MSRV` required once GitHub settings allow it.   |
| [`0003-release-assets.md`](0003-release-assets.md)                             | binary release assets        | Prove cargo-binstall and Homebrew can consume release archives and checksums.  |
| [`0004-log-first-mvp.md`](0004-log-first-mvp.md)                               | first product slice          | Build the smallest log, movement, refresh, and inline expansion loop.          |
| [`0005-log-mvp-review-hardening.md`](0005-log-mvp-review-hardening.md)         | MVP review hardening         | Review the log MVP shape and tighten edge cases before expanding scope.        |
| [`0006-selected-change-diff.md`](0006-selected-change-diff.md)                 | selected-change diff         | Add manual-refresh diff inspection with collapsible file or hunk sections.     |
| [`0007-auto-refresh-policy.md`](0007-auto-refresh-policy.md)                   | auto-refresh policy          | Design debounce and preservation rules before enabling automatic refresh.      |
| [`0008-jj-integration-cleanup.md`](0008-jj-integration-cleanup.md)             | jj integration cleanup       | Narrow the stdout/template bridge and document parser assumptions.             |

Keep these files current by replacing stale notes with decisions. Avoid turning `.plans` into a
progress ledger; durable product or architecture decisions belong in tracked docs once accepted.
