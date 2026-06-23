# Release Stabilization Pass

Status: planned release-readiness pass after the dogfood spike

Scope: turn the current local implementation stack into an intentional, documented, releasable
milestone before starting rebase-specific workflows.

## Why This Exists

The local implementation has crossed from short spike into release-candidate territory. It now
contains enough user-facing behavior that the next best work is not another feature slice. The next
best work is to make the existing behavior read, test, document, and demo like a product milestone.

Rebase should wait until this pass is complete. Rebase needs selector and role-resolver clarity, and
adding it before stabilization would make the release harder to review and explain.

## Release Cutoff

Release the current dogfoodable workbench shape before rebase:

- jj-rendered log and diff views;
- show, status, evolog, workspaces, operation log, and command history;
- command mode for direct `jj` commands;
- safe command previews for describe, abandon, new, edit, undo, and redo;
- operation-id capture, command history links, copy affordances, and recovery footer;
- diff navigation, file list, folding, search, current-file status, and diff View Options;
- generated help, searchable command discovery, adaptive hotbar, ordered marks, and menu polish.

Do not include rebase in this release candidate unless it becomes necessary to repair an existing
workflow. Rebase belongs in the next feature milestone after selector and role behavior are clearer.

## Naming And Public Framing

Do not describe the release as "vibe coded" or as a "vibe spike" in public artifacts.

Acceptable framing:

- "dogfood milestone";
- "release stabilization pass";
- "jj-native workbench";
- "safe mutation preview loop";
- "operation recovery and command history";
- "Betamax-backed terminal evidence".

Local planning files may still mention the `vibe` workspace when they are explicitly about local
execution history. Public-facing docs, release notes, README copy, website copy, screenshots, GIF
captions, and commit descriptions should describe product behavior rather than the way it was built.

## Current Audit

Current stack evidence from `/Users/joshka/local/jk/vibe`:

- More than 40 changes sit on top of `main` after the product-plan merge.
- `crates/jk/src/main.rs` is over 6,000 lines and now owns many unrelated app-loop concepts.
- The repository README and crate README still describe the older inspection-loop surface.
- Product-plan and roadmap docs needed a first pass from local-spike wording toward
  implementation-status wording.
- Local Betamax evidence exists under `target/vibe-artifacts`, but public media has not been
  regenerated for the broader workbench surface.
- Website and screenshots repositories have not been updated for these new workflows.

This does not mean the work is unreleasable. It means the release pass needs to make the artifact
set honest and reviewable before publication.

## Pass 1: Stack And History Audit

Review the jj stack from `main` to the release candidate.

Goals:

- every change has a product-shaped description with a useful 72-column body;
- descriptions explain behavior and user value, not implementation trivia;
- no public-facing description depends on private chat context;
- duplicated or obsolete spec-only changes are squashed or clearly kept as durable decisions;
- local-spike wording is removed from changes that should become public review history;
- stack order still builds and validates at practical cut points.
- commit messages are durable implementation records, but release notes are not generated from
  commit messages.

Suggested grouping for eventual review:

1. foundation: command specs, view stack, help/hotbar, command metadata;
1. inspection: show, status, diff query formats, evolog, View Options;
1. workspaces: providers, models, list screen, stale update;
1. command history and command mode;
1. operation recovery and recovery previews;
1. safe mutation previews for describe, abandon, new, and edit;
1. diff navigation and file-list polish;
1. release docs, README, media, and website update.

History scrub checklist:

- remove local-workspace, model, or "vibe coding" framing from changes that should become public;
- make each description explain the user-visible behavior, safety policy, or durable decision;
- keep spec/planning changes only when they remain useful review context;
- squash follow-up fixes into the owning behavior change when the parent would be misleading
  without them;
- leave separate changes when they document a durable product or architecture decision.

## Pass 2: Code Shape Audit

Focus review on real ownership boundaries, not line count alone.

Known pressure points:

- `crates/jk/src/main.rs` owns terminal lifecycle, CLI args, app state, mode stack, command previews,
  command mode, mutation orchestration, workspace routing, and test fixtures.
- `crates/jk-tui/src/keymap.rs` is now a rich registry for help, discovery, and hotbar metadata.
- command history and mutation recording cross `jk`, `jk-cli`, and `jk-core`.

Potential extraction targets:

- app mode handlers and overlay movement helpers;
- command preview and confirmed mutation orchestration;
- command mode parsing/running;
- workspace view routing;
- test fixtures and fake command runners.

Do not extract merely because a file is long. Extract when the new owner names a stable concept,
reduces the context a reviewer must hold, and keeps side effects easier to audit.

Code-shape review criteria:

- reader locality: a maintainer can understand a flow without reconstructing unrelated modes;
- concept coherence: each module owns one recognizable idea rather than a helper bucket;
- explicit effects: command execution, mutation confirmation, refresh, operation capture, and
  terminal side effects are named at call sites;
- reversible structure: extraction happens after the behavior boundary is known, not as speculative
  architecture;
- test locality: fixtures and fake runners live where they prove the behavior without hiding the
  boundary under test.

Initial module-size pressure is evidence, not a rule. `main.rs` needs attention because it mixes
many responsibilities, not because it crosses a numeric threshold. Prefer extracting command mode,
preview/mutation orchestration, workspace routing, and app-mode handlers only when those moves make
the current release easier to review and maintain.

## Pass 3: User Documentation, Changelog, And Release Notes

Update user-facing docs to match the release candidate.

Required surfaces:

- repository `README.md`;
- `crates/jk/README.md`;
- release notes / changelog entry;
- local docs for command mode, safe mutation previews, command history, operation recovery,
  workspaces, View Options, and diff navigation;
- keymap/help documentation if the generated help surface becomes a documented contract.

Release notes and changelog policy:

- write release notes from a user-facing feature audit, not from commit messages or an automated
  commit summary;
- keep `CHANGELOG.md` useful for readers who want to know what changed, why it matters, what safety
  rules changed, and which limitations remain;
- group entries by workflow or user outcome, such as inspection, command history, recovery,
  workspaces, mutation previews, diff navigation, docs/media, and compatibility;
- include keymap changes and safety changes explicitly;
- include known limitations and deferred rebase work without making them sound like bugs;
- use the jj history as evidence, not as the structure of the release note.

The docs should be direct about limitations:

- rebase is intentionally deferred;
- command previews are the supported mutation shape;
- direct `a`, `n`, and `e` bindings are dogfood shortcuts until the action menu exists;
- command history is currently in-memory unless later persistence lands;
- generated media comes from Betamax and public assets live outside this source repository.

Documentation review criteria:

- README and crate README are entry points, not exhaustive manuals;
- user docs state current behavior, not aspiration;
- detailed workflows live in local docs or website pages where the reader can choose depth;
- Markdown stays lintable, wrapped, and readable in source form;
- examples and command lists are verified against the current binary before release.

## Pass 4: Betamax Evidence And Public Media

Keep two artifact classes separate:

- validation tapes prove behavior and should be suitable for CI or release smoke;
- media tapes produce polished GIFs/screenshots for README, crates.io, website, and release notes.

Candidate public media set:

1. workbench overview: log to show/diff/status and back;
1. command history and operation recovery after a confirmed mutation;
1. command mode success and failure output;
1. workspace list to workspace status/diff;
1. diff navigation with file list, folding, search, and View Options;
1. safe mutation preview cancel/copy/confirm.

Media requirements:

- use Betamax captions and keyboard overlay;
- dwell long enough for humans to read each state;
- avoid private local paths, user email, or unrelated repository noise;
- generate into the owning media repository, not the main `jk` source tree;
- verify published URLs return real media bytes, not Git LFS pointer files.

Public media may be prepared in the website or screenshot repositories, but it must not advertise
unreleased behavior as already available. The website is allowed to lag a release. It should not
lead one.

## Pass 5: Website Update

The website update is out of scope for the active local `vibe` goal, but it is required before a
polished public release or immediately after release as a tracked follow-up.

Website pass should happen in `/Users/joshka/local/jk-website` after the source release candidate is
stable enough to demo.

Update:

- homepage story from inspection helper to jj-native workbench;
- hero/demo media;
- feature sections for command previews, recovery, command history, workspaces, View Options, and
  diff navigation;
- install and release notes links if release assets or Homebrew behavior changed.

Website release policy:

- the website should track released behavior, even if it lags by a release;
- do not pre-publish demos or claims for unreleased behavior;
- media may be pushed to the website or screenshot repositories as part of the release handoff when
  the page that references it is gated to released behavior;
- website copy should link to the release notes and should not force users to infer feature status
  from commit history.

Run the website's normal build and Betamax media generation from that repository.

## Pass 6: Release Validation

Minimum source validation:

```sh
cargo test -p jk-core
cargo test -p jk-cli
cargo test -p jk
cargo test -p jk-tui
cargo fmt --check
cargo check
markdownlint-cli2 --config ~/.markdownlint-cli2.yaml README.md crates/jk/README.md docs/**/*.md
```

Minimum dogfood validation:

- `cargo run -- log`;
- `cargo run -- diff`;
- `cargo run -- status`;
- `cargo run -- workspaces`;
- command mode success and failure;
- one confirmed mutation in a disposable fixture, followed by command history and operation view;
- `cargo install --path crates/jk` for local dogfooding.

Minimum media validation:

- validation Betamax tapes pass for changed workflows;
- selected media tapes regenerate;
- published media URLs are checked after deploy.

Minimum release-channel validation:

- `just release-check` if available and still current;
- release-plz dry-run or release PR readback;
- cargo-binstall install smoke after release assets exist;
- Homebrew tap smoke after the release artifact path is updated.

## Done Criteria

This pass is complete when:

- the release candidate has no public "vibe coding" framing;
- the stack descriptions and docs explain shipped behavior and limitations;
- README, crate README, LLM-authored release notes, changelog, website, and media agree about the
  feature surface;
- public media has been regenerated and published through the owning repos;
- source, dogfood, media, and release-channel validation have passed or have explicit documented
  exceptions;
- rebase work has a clean next-slice entry point after this release.
