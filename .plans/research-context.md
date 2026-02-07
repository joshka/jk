# Research Context

## Snapshot

- Date: 2026-02-07
- Repository: `/Users/joshka/local/jk`
- Intent: build a log-first `jk` TUI that keeps users inside `jk` for daily workflows.

## Local command-surface audit

Sources were collected from `jj --no-pager --help` and per-command `--help` output.

- Top-level `jj` commands discovered: 44
- Subcommands discovered across command groups: 59
- Command tree source file: `/tmp/jj_command_tree.tsv`
- Full help dump source file: `/tmp/jj_top_level_help.txt`

Representative observations from command help and output:

1. `jj log` is graph-first and already configurable via templates/revsets.
2. `jj status` reports working copy, parent, and conflicted bookmarks.
3. `jj git push` has explicit safety checks similar to force-with-lease semantics.
4. `jj rebase` has rich source/target modes and high rewrite risk.
5. `jj undo` and `jj op log` make operation history a first-class recovery mechanism.

Sample outputs captured in `/tmp/jj_research_snippets.txt`:

- `jj --no-pager status`
- `jj --no-pager log -n 6`
- `jj --no-pager log --help`
- `jj --no-pager git push --help`
- `jj --no-pager rebase --help`

## Oh My Zsh `jj` plugin audit (local install)

Source: `~/.oh-my-zsh/plugins/jj/jj.plugin.zsh`

Aliases confirmed and relevant to `jk` planning:

- Navigation/read: `jjl`, `jjla`, `jjst`, `jjd`
- Mutation: `jjc`, `jjds`, `jje`, `jjsp`, `jjsq`, `jja`, `jjrb`
- Remote: `jjgf`, `jjgfa`, `jjgp`, `jjgpt`, `jjgpa`, `jjgpd`
- Bookmarks: `jjb`, `jjbl`, `jjbs`, `jjbm`, `jjbt`, `jjbu`, etc.
- Shell-only: `jjrt` (`cd` to `jj root`), which needs an in-app equivalent in `jk`.

## External references and takeaways

### Jujutsu docs (official)

- Site: `https://docs.jj-vcs.dev/latest/`
- Tutorial: `https://docs.jj-vcs.dev/latest/tutorial/`
- Operation log doc:
  `https://raw.githubusercontent.com/jj-vcs/jj/main/docs/operation-log.md`
- Bookmarks doc:
  `https://raw.githubusercontent.com/jj-vcs/jj/main/docs/bookmarks.md`

Takeaways:

- Operation log enables robust undo/recovery and concurrent operations.
- Bookmarks are central to Git interoperability and remote safety behavior.
- Working copy is commit-based, so interactive tooling should surface that model clearly.

### Steve Klabnik tutorial

- Site: `https://steveklabnik.github.io/jujutsu-tutorial/`
- Source markdown examples used:
  - `.../src/introduction/what-is-jj-and-why-should-i-care.md`
  - `.../src/hello-world/viewing-contents.md`
  - `.../src/real-world-workflows/the-squash-workflow.md`

Takeaways:

- Emphasizes conceptual simplicity: fewer concepts, orthogonal powerful commands.
- Highlights `jj log` readability, stable change IDs, and revset discoverability.
- Demonstrates practical interactive `squash -i` workflows that map naturally to TUI flows.

## jj crate baseline reference

Cloned source used for dependency parity review:

- `git -c commit.gpgsign=false clone --depth 1 https://github.com/jj-vcs/jj /tmp/jj-upstream.2dGMe8`

Key crates observed in workspace/CLI:

- Core UX/runtime: `clap`, `crossterm`, `sapling-renderdag`, `sapling-streampager`,
  `scm-record`
- Config/data: `serde`, `toml`, `toml_edit`, `indexmap`
- Utility/errors: `itertools`, `regex`, `bstr`, `thiserror`, `tracing`
- Testing: `insta`, `assert_cmd`, `test-case`, `proptest`
