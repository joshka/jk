# JJ Topology And Source Control

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `VCS-ASK-BEFORE-REPAIRING-JJ-ALIASES`: Ask before repairing jj aliases that encode remote or
  bookmark assumptions, especially `trunk()` and publish helpers.
- `VCS-AVOID-INTERACTIVE-JJ-IN-AGENT-WORK`: Avoid interactive-by-default jj commands in unattended
  agent work because interactive jj commands can open editors, prompts, merge tools, or pagers that
  unattended agents cannot handle reliably.
- `VCS-CONFIGURE-JJ-PAGER`: Configure `JJ_PAGER` for agent tooling so paged output does not block or
  truncate command results.
- `VCS-CONFIRM-BROAD-JJ-OPERATIONS`: Treat broad jj operations as confirmation-worthy because
  commands that abandon, rebase, squash, split, restore, publish, or affect many revisions can
  rewrite a large part of the graph.
- `VCS-CONFIRM-GITHUB-REMOTE-TOPOLOGY`: Confirm GitHub `origin` and `upstream` topology before
  publication because forks and GitHub defaults can make `origin` mean the user fork while
  `upstream` means the canonical repo, or vice versa in owned repos.
- `VCS-CREATE-OPERATION-LOG-POINT-BEFORE-RESHAPING`: Before risky jj stack reshaping, run harmless
  inspection so there is a recent operation-log point for recovery.
- `VCS-DO-NOT-FALL-BACK-TO-GIT-FOR-JJ-ISSUES`: Stay in jj for transient jj lock or sandbox issues;
  Git does not represent the full jj change graph.
- `VCS-DRY-RUN-SURPRISING-PUBLICATION`: Use dry-run for surprising jj publication: ambiguous remote,
  new bookmark, force-like update, fork topology, or unclear PR base.
- `VCS-DUPLICATE-FOR-ALTERNATIVE-CANDIDATES`: Use `jj duplicate` for alternative fixes or refactor
  shapes so the original candidate stays available.
- `VCS-INSPECT-SPARSE-STATE`: Inspect sparse state before treating a missing path as missing
  history; sparse patterns can hide files that still exist.
- `VCS-INSPECT-STATE-BEFORE-MUTATING`: Before creating, squashing, rebasing, publishing, or editing
  files, the agent needs to know the current working copy, parent, bookmarks, conflicts, and unowned
  changes, inspect working-copy and stack state before mutating.
- `VCS-JJ-AS-SOURCE-OF-TRUTH`: Use `jj` as the source of truth in `.jj` repositories because a `.jj`
  repo has jj changes, operation log, working-copy state, and bookmarks layered over Git storage.
- `VCS-JJ-NEW-FOR-REVIEW-LANES`: Use `jj new` for separate review lanes because a new task needs a
  separate review lane before unrelated edits accumulate.
- `VCS-MAKE-GITHUB-HANDOFF-EXPLICIT`: Make GitHub handoff explicit after jj state is coherent
  because jj state and GitHub state are related but not identical.
- `VCS-MATCH-JJ-TOPOLOGY-TO-REPO-ROLE`: Match jj remote topology to the repository role because
  owned repos, maintainer-access repos, and fork-only contributor repos need different remote and
  bookmark topology.
- `VCS-NAME-EXACT-JJ-MUTATION-TARGETS`: Name exact revisions, filesets, bookmarks, and destinations
  for mutating jj commands instead of relying on stack-sensitive defaults.
- `VCS-QUOTE-REVSETS-AND-SHELL-SYNTAX`: Quote revsets and shell-sensitive syntax because revsets and
  bookmark syntax often contain characters such as `@`, `|`, `&`, `~`, parentheses, or spaces that
  shells can interpret.
- `VCS-RECOVER-WITH-OPERATION-LOG`: Use operation-log recovery instead of destructive cleanup
  because jj records repository operations so many mistakes are recoverable without destructive Git
  reset or stash habits.
- `VCS-REPAIR-REMOTE-TOPOLOGY-COHERENTLY`: Repair remote topology coherently because remote topology
  has several coupled pieces: fetch remote, push remote, tracked bookmark, trunk alias, PR base, and
  PR head.
- `VCS-RUN-JJ-MUTATIONS-SEQUENTIALLY`: Run jj mutations sequentially because jj mutating commands
  update working-copy and operation state.
- `VCS-SCOPE-JJ-FILE-TRACKING`: Scope jj file track and untrack commands to intended paths because
  `jj file track` and `jj file untrack` can affect more files than intended if paths are omitted or
  globbed too broadly.
- `VCS-STOP-REPEATED-JJ-RETRIES-AND-LOCALIZE-STATE`: Stop repeated jj retries and localize state
  because repeating a failing jj command without new information usually compounds confusion.
- `VCS-TRACK-REMOTES-EXPLICITLY`: Track remotes explicitly for bookmark names that exist on multiple
  remotes so source and publication targets are clear.
- `VCS-TREAT-BOOKMARK-REMOTE-SYNTAX-AS-VERSION-SENSITIVE`: Treat `bookmark@remote` command syntax as
  version-sensitive because jj command syntax around `bookmark@remote` and remote bookmark handling
  can vary by version and command.
- `VCS-USE-EVOLOG-AND-OPERATION-LOG`: Use `jj evolog` for one change's evolution and `jj op log` for
  repository operations because `jj evolog` answers how one change evolved; `jj op log` answers how
  the repository state changed.
- `VCS-USE-GIT-FORMATTED-DIFFS-FOR-AGENTS`: Use Git-formatted diffs for agents and review tools that
  understand patch format better than native jj summaries.
- `VCS-USE-IGNORE-WORKING-COPY-CAREFULLY`: Use `--ignore-working-copy` only for lock-safe inspection
  or intended metadata work because it may skip current file snapshots.
- `VCS-WORKSPACE-ADD-FOR-SECOND-CHECKOUTS`: Use `jj workspace add` only for a second filesystem
  checkout; use `jj new` for another change in the same checkout.
