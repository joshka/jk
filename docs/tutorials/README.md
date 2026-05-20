# Tutorials

These pages cover the shipped day-to-day loops in `jk`. They stay on current behavior, use the
tracked demo setup where that keeps the examples concrete, and avoid generated media in the repo.

## Walkthroughs

- [Daily loop](daily-loop.md): inspect log, show, diff, status, fetch, push, and create new work.
- [Rewrite and recovery](rewrite-and-recovery.md): describe, commit, abandon, squash, rebase,
  absorb, restore, revert, and operation recovery.
- [Bookmarks and conflicts](bookmarks-and-conflicts.md): bookmark management and the read-only
  resolve screen.

## Demo Repos

The deterministic demo setup lives in [docs/demo/README.md](../demo/README.md). It creates
disposable repositories under `target/demo-repos/`, and any rendered media belongs under ignored
`target/vhs/` or outside the repository.
