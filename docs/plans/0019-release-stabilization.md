# Release Stabilization Pass

Status: planned release-readiness pass after the dogfood milestone

Scope: turn the current local implementation stack into an intentional, documented, releasable
milestone before starting rebase-specific workflows.

## Why This Exists

The local implementation has crossed from early exploration into release-candidate territory. It
now contains enough user-facing behavior that the next best work is not another feature slice. The
next best work is to make the existing behavior read, test, document, and demo like a product
milestone.

Rebase should wait until this work is complete. Rebase needs selector and role-resolver clarity, and
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

Do not describe the release by its implementation process in public artifacts.

Acceptable framing:

- "dogfood milestone";
- "release stabilization pass";
- "jj-native workbench";
- "safe mutation preview loop";
- "operation recovery and command history";
- "Betamax-backed terminal evidence".

Local planning files may still mention dogfood context when they are explicitly about local
execution history. Public-facing docs, release notes, README copy, website copy, screenshots, GIF
captions, and commit descriptions should describe product behavior rather than the way it was built.

## Current Audit

Current stack evidence from the local release workspace:

- More than 40 changes sit on top of `main` after the product-plan merge.
- `crates/jk/src/main.rs` started this work over 6,000 lines and owned many unrelated app-loop
  concepts; first extractions have moved stable support concepts into focused modules, but the app
  loop still needs more release-readiness review.
- The repository README and crate README still describe the older inspection-loop surface.
- Product-plan and roadmap docs needed a first pass from local dogfood wording toward
  implementation-status wording.
- Local Betamax evidence exists under `target/dogfood-artifacts`, but public media has not been
  regenerated for the broader workbench surface.
- Website copy has been updated locally for the release snapshot. Screenshot publication is staged
  for the owning media repository, but final public media should wait for the Betamax layout update
  if release assets should include captions plus on-screen keys.

This does not mean the work is unreleasable. It means the release pass needs to make the artifact
set honest and reviewable before publication.

## Pass 1: Stack And History Audit

Review the jj stack from `main` to the release candidate.

Goals:

- every change has a product-shaped description with a useful 72-column body;
- descriptions explain behavior and user value, not implementation trivia;
- no public-facing description depends on private chat context;
- duplicated or obsolete spec-only changes are squashed or clearly kept as durable decisions;
- local-dogfood wording is removed from changes that should become public review history;
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

- remove local-workspace, model, or "dogfood implementation" framing from changes that should
  become public;
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

- startup CLI parsing, query conversion, and `jj` source construction;
- operation-log snapshot parsing;
- command mode parsing, command spec construction, and command output rendering;
- workspace snapshot mapping, stale-update status formatting, and workspace action mapping;
- popup/menu view models, wrapping policy, and selector lines;
- mutation preview metadata, prompt text, selected-parent policy, and failure text;
- refresh/load helpers across log, diff, rendered inspections, operation views, and workspaces;
- stateful workspace routing for list, selected status/diff, and update-stale flows;
- app mode handlers and overlay rendering;
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

Mechanical split plan:

1. Keep `state.rs` as the private app-state owner: `AppView`, `AppState`, `ViewStack`, `ModeStack`,
   `InputMode`, and `InputModeResult`.
1. Keep pure view-model modules separate from side effects: `menus.rs`, `operation_log.rs`,
   `mutation_preview.rs`, `workspaces.rs`, and `command_mode.rs` own formatting, parsing, selector
   policy, and command previews.
1. Extract rendering next only if it can own the `AppView` + `InputMode` draw matrix without taking
   command execution, refresh, or input dispatch with it.
1. Keep confirmed mutation execution separate from preview metadata: `mutations.rs` owns command
   confirmation, recovery preview entry, post-mutation log refresh, and the recovery footer.
1. Extract action dispatch only after rendering is separate enough that handlers can be grouped by
   user intent: open/push actions, rendered-view actions, workspace actions, and operation actions.
1. Keep reload mechanics separate from input dispatch: `refresh.rs` owns command-runner history
   recording, error fallback, and reload transitions across log, diff, rendered inspections,
   operation views, and workspaces.
1. Keep workspace routing separate from pure workspace view models: `workspace_routes.rs` owns the
   stateful list, selected status/diff, repository status, and update-stale flows while
   `workspaces.rs` keeps snapshot mapping and pure action policy.
1. Keep shared binary test fixtures in `test_support.rs` so behavior tests can read as assertions
   about product flows rather than fixture construction.

Stop conditions for each extraction:

- no behavior change is intended;
- `cargo fmt --check`, focused tests where available, `cargo test -p jk`, and `cargo check` pass;
- the new module has a single recognizable owner and is not a helper bucket;
- `main.rs` loses live context rather than gaining indirection through generic wrappers.

First extraction status:

- `crates/jk/src/cli.rs` now owns startup argument parsing, root-command query conversion, and
  repository-aware construction of `jj` sources.
- `crates/jk/src/operation_log.rs` now owns conversion from rendered `jj op log` text into
  operation-log snapshot rows, with local parser tests for row titles and ANSI stripping.
- `crates/jk/src/command_mode.rs` now owns command-mode argument parsing, command spec
  construction, prompt lines, and command-output rendering, with local parser and output tests.
- `crates/jk/src/workspaces.rs` now owns workspace-list snapshot mapping, stale-update status
  formatting, and pure key-action mapping for workspace list and workspace inspection views.
- `crates/jk/src/mutation_preview.rs` now owns safe mutation preview metadata, describe prompt
  text, selected-parent policy for `jj new`, and confirmed-command failure messages.
- `crates/jk/src/menus.rs` now owns popup/menu selector policy, view-option row models,
  diff-file-list rows, and template selector lines.
- `crates/jk/src/state.rs` now owns the app view enum, view stack, mode stack, and transient input
  mode model.
- `crates/jk/src/rendering.rs` now owns the `AppView` + `InputMode` draw matrix and shared overlay
  rendering without taking command execution, refresh, or input dispatch with it.
- `crates/jk/src/actions.rs` now owns normal-mode key dispatch and search-mode selection while
  leaving command execution, refresh, and mutation helpers in their current owners.
- `crates/jk/src/mutations.rs` now owns confirmed command execution, recovery preview entry,
  post-mutation log refresh, and recovery-footer status text.
- `crates/jk/src/refresh.rs` now owns command-runner history recording, error fallback, and reload
  transitions across log, diff, rendered inspections, operation views, and workspaces.
- `crates/jk/src/workspace_routes.rs` now owns stateful workspace list, selected status/diff,
  repository status, and update-stale flows while `workspaces.rs` keeps pure view mapping and
  action policy.
- `crates/jk/src/test_support.rs` now owns shared binary test fixtures: synthetic log/diff/workspace
  views, command-history appenders, process-output builders, buffer helpers, and the sequenced fake
  `jj` runner.
- `crates/jk/src/root_views.rs` now owns initial root view construction for `jk`, `jk log`,
  `jk diff`, `jk show`, and `jk status`, including error fallback for initial rendered views.
- `crates/jk/src/runner.rs` now owns the small system-runner recording adapter used by loaders.
- `crates/jk/src/clipboard.rs` now owns OSC52 command-copy support and its encoding tests.
- `crates/jk/src/command_history.rs` now owns command-history list/details routing,
  command-history operation links, operation-log opening, and command-history action translation.
- `main.rs` is smaller and more reviewable, but still owns app-loop behavior and many behavior
  tests. Further extraction should be driven by a user-visible change or a clearer owner, not by
  line count alone.

Second extraction status:

- Started from a mechanical module map instead of opportunistic helper moves.
- Kept the second pass behavior-preserving: no key bindings, command construction, mutation policy,
  or rendered output semantics intentionally changed.
- Reduced `main.rs` to 3,693 lines while keeping app startup, terminal lifecycle, input-mode
  handling, active-view action application, and the existing broad behavior tests in one place.
- Left the next likely extraction as input-mode handling only if it can be split by coherent mode
  ownership rather than by line count.

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

Pass 3 status:

- `README.md` and `crates/jk/README.md` describe the current dogfoodable workbench surface at entry
  point depth.
- `CHANGELOG.md` has a curated `Unreleased` entry grouped by user workflow rather than commit
  order.
- `docs/workbench.md` now documents current command surface, log inspection, diff review, safe
  command previews, command mode, command history, operation recovery, workspaces, generated help,
  and known limitations.
- A progressive-disclosure pass keeps README files at entry-point depth, moves task flows into
  `docs/workbench.md`, and leaves exhaustive key reference to in-app generated help.
- The documented root command list has been checked against `cargo run -p jk -- --help` and the
  `log`, `diff`, `show`, and `status` subcommand help output.
- Public README, crates.io, and website media still need the separate media/release handoff before
  publication.

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

- use Betamax keyboard overlays where they clarify the action being demonstrated;
- use captions only when Betamax can place them outside the terminal content or otherwise avoid
  covering the TUI hotbar, footer, prompts, and status text;
- keep captions and keyboard overlays in separate visual space so they do not compete for the
  viewer's attention or obscure each other;
- dwell long enough for humans to read each state;
- avoid private local paths, user email, or unrelated repository noise;
- generate into the owning media repository, not the main `jk` source tree;
- verify published URLs return real media bytes, not Git LFS pointer files.

Public media may be prepared in the website or screenshot repositories, but it must not advertise
unreleased behavior as already available. The website is allowed to lag a release. It should not
lead one.

Media ownership rule:

- keep generated screenshots, GIFs, and videos out of the main `jk` source repository so jj does
  not have to carry Git LFS-heavy media churn;
- use `jk-screenshots` for README, crates.io, release-note, and other reusable public assets;
- use `jk-website` for site-specific assets when the website is the only consumer;
- keep local dogfood validation artifacts under `target/dogfood-artifacts/` until a release
  handoff intentionally publishes them.

Local tape status:

- `just betamax` runs the existing log and diff visual smoke tapes with the installed `betamax`
  binary by default.
- `just betamax-release-smoke` runs the broader dogfood fixture tape for root status, workspaces,
  command mode, mutation preview, command history, and operation-detail routing.
- The tapes write local artifacts under `target/dogfood-artifacts/betamax/`, including
  `jk-log.gif`, `jk-diff.gif`, screenshots, and terminal state JSON.
- `just readme-media` is local-only during this work and writes to
  `target/dogfood-artifacts/readme-media/`; publishing to the screenshots repository is a later
  release handoff.
- The README log hero tape intentionally keeps overlays off so the first product image reads as
  the application, not a tutorial.
- The deeper README diff tape and documentary dogfood tapes use short captions with
  `Set KeyboardOverlay Input` where typed intent is part of the release narrative.
- The installed Betamax build supports semantic waits, screenshots, state JSON, GIF/video output,
  readable dwell, captions, and keyboard overlays with the caption and key chips outside the
  terminal canvas.
- `release-smoke.gif` is validation evidence, not final public media. It intentionally keeps enough
  setup context to prove the disposable-repository fixture, while public website/release-note clips
  should start on the TUI surface after setup has completed.

Installed Betamax overlay behavior:

- The local installed Betamax build includes the presentation-overlay fix from
  `joshka/betamax#94`.
- Betamax reserves a bottom presentation row before deriving the terminal grid whenever `Caption`
  or `KeyboardOverlay` is active. Captions sit below the terminal frame on the left, keyboard chips
  sit below the frame on the right, and long captions truncate instead of wrapping into the key
  chips.
- Render `jk` release tapes with `/Users/joshka/.cargo/bin/betamax` or by setting
  `BETAMAX=/Users/joshka/.cargo/bin/betamax` for `just` recipes.
- Because the reserved row changes the derived terminal grid, captioned/keyed public media should
  avoid oversized margin or internal padding. The current release tapes use `Set Padding 0` and
  `Set Margin 20` so the terminal remains prominent while the presentation row stays outside the
  TUI canvas.

Release-media overlay plan:

- Keep the first README/crates.io hero GIF clean: no caption and no keyboard overlay. It should
  show `jk` as the product without documentary chrome competing with the TUI.
- Keep compact README final-frame screenshots clean unless the screenshot is explicitly teaching a
  key-driven state. Static entry-point images should prioritize terminal legibility.
- Use captions plus `Set KeyboardOverlay Keys` for documentary website and release-note clips that
  teach key-driven navigation, such as log to show/status/evolog, diff file and hunk navigation,
  View Options, operation log, and Command History routing.
- Use captions plus `Set KeyboardOverlay Input` only when typed input is part of the story, such as
  command mode success/failure or describe-message editing. Avoid `Input` for the first product
  GIF because typed command chips can make the demo feel like a tutorial rather than the product.
- Avoid `Set KeyboardOverlay All` for public `jk` media. Long setup commands should stay hidden,
  and public overlays should show user intent rather than implementation setup.
- Place `Caption` commands before the semantic action they explain, then use `Wait+Screen` or
  `State` for the behavior proof. Add `Sleep` after a caption-only transition only when the
  animation needs readable dwell without terminal changes.
- Keep captions short enough to share the row with key chips. Prefer action-oriented captions such
  as "Open the selected diff", "Preview before mutating", and "Follow the recorded operation".
- Clear captions with `Caption ""` before cleanup, shell exit, or any frame that should return to a
  pure terminal presentation.

Planned public media set after Betamax release:

1. `jk-log-v3.gif`: README/crates.io hero, clean log-to-diff overview, no captions or key overlay.
1. `jk-diff-v3.gif`: README diff review with captions and input overlay only for guided search and
   folding moments where typed intent improves the reader's understanding.
1. Website workbench overview: captioned, key-driven navigation from log to show/diff/status and
   back.
1. Website command preview/recovery: captioned, `KeyboardOverlay Input` for the describe prompt,
   confirmation, Command History, and operation show route.
1. Website command mode: captioned, `KeyboardOverlay Input` for `:status` and one failing command
   so stderr capture is visible.
1. Website workspaces: captioned, `KeyboardOverlay Keys` for workspace list, selected status, and
   selected diff.
1. Release-note stills: checkpoint PNGs from the same documentary tapes, with captions only where
   the frame needs context outside the terminal text.

## Pass 5: Website Update

The active release goal includes the website. Website copy should track the release snapshot before
publication. Final media can use the installed Betamax presentation-overlay behavior, but website
media should still be published only when the source release candidate is stable enough to demo.

Website pass happens in `/Users/joshka/local/jk-website` after the source release candidate is
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

Pass 5 status:

- `/Users/joshka/local/jk-website` now describes the current release snapshot rather than the
  earlier log/diff-only milestone.
- The homepage includes sections for inspection, diff review, command previews, command mode,
  command history, operation recovery, workspaces, View Options, release boundaries, and docs.
- `pnpm build` passes.
- Playwright visual QA against `http://127.0.0.1:4322/jk` showed no horizontal overflow on desktop
  or mobile, and header/footer navigation uses the release labels.
- Final homepage/README media publication is pending the Betamax caption/key-overlay layout update.

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

Pass 6 source-validation status:

- `just fmt-check` initially found Rust comment wrapping drift; `just fmt` corrected it and the
  follow-up formatting check passed.
- `just check` passed after the app-support extraction.
- `just test` passed across the workspace after the clippy and documentation cleanup.
- `just clippy` passed after fixing release-readiness warnings in the command model, command
  history, TUI views, and app support modules.
- `just doc` passed cleanly after fixing the command-runner intra-doc link.
- `just package`, `just install-smoke`, and `just udeps` passed.
- CLI help smoke checks passed for `jk --help`, `jk log --help`, `jk diff --help`,
  `jk show --help`, `jk status --help`, and `jk --version`.
- `betamax validate 'tapes/*.tape'` passed for the five repository tapes before the final
  captioned media pass.
- `just betamax` passed for the current log and diff visual smoke tapes and wrote artifacts under
  `target/dogfood-artifacts/betamax/`.
- `just readme-media` passed locally before the final captioned media pass and wrote artifacts
  under `target/dogfood-artifacts/readme-media/`.
- `tapes/release-smoke.tape` passed with a disposable jj repository, covering root status,
  workspaces, command-mode success/failure, confirmed describe mutation, Command History, and the
  captured operation route.
- An early captioned README diff preview rendered with `/Users/joshka/.cargo/bin/betamax`; visual
  inspection confirmed the `jk` hotbar stayed inside the terminal while the caption and Enter chip
  shared the reserved presentation row below it.
- `BETAMAX=/Users/joshka/.cargo/bin/betamax just readme-media` passed after switching the README
  log hero to a clean overlay-free frame and the README diff media to captioned input-overlay
  guidance with `Set Padding 0` and `Set Margin 20`.
- `BETAMAX=/Users/joshka/.cargo/bin/betamax just betamax` passed for the captioned log and diff
  smoke tapes using the same frame settings.
- `JK_SOURCE_REPO=/path/to/jk BETAMAX=/Users/joshka/.cargo/bin/betamax just
  betamax-release-smoke` passed for the disposable-repository release smoke tape using the same
  frame settings.
- Visual inspection of `target/dogfood-artifacts/readme-media/jk-log-v3.png`,
  `target/dogfood-artifacts/readme-media/jk-diff-search-v3.png`, and
  `target/dogfood-artifacts/betamax/jk-log-expanded.png` confirmed the clean hero, captioned
  search frame, and captioned key-driven frame keep the `jk` hotbar visible while captions and key
  chips stay outside the terminal canvas.
- Versioned README/crates.io media was copied into `/Users/joshka/local/jk-screenshots/assets/`
  for publication under `https://www.joshka.net/jk-screenshots/assets/`. The screenshot repository
  has Git LFS attributes for PNG and GIF assets.
- Website media was copied into `/Users/joshka/local/jk-website/public/assets/` under the
  unversioned filenames used by the project page.
- The website homepage tape was updated for the current workbench behavior where `Enter` opens
  `jk jj show`, then regenerated with
  `JK_SOURCE_REPO=/path/to/jk JK_WEBSITE_REPO=/Users/joshka/local/jk-website
  /Users/joshka/.cargo/bin/betamax run tapes/homepage.tape`.
- `pnpm build` passed in `/Users/joshka/local/jk-website`.
- Browser validation of the local website preview passed for the desktop hero, media grid, image
  asset loading, and a 390 px mobile viewport with no horizontal overflow.
- A tracked-text scrub removed local workspace naming from README, docs, release planning,
  Betamax tapes, and source fixtures; local media now writes under `target/dogfood-artifacts/`.
- `just release-check` passed on the current release-candidate workspace after the history and
  public-wording scrub. This re-ran formatting, check, tests, clippy, udeps, docs, package,
  install-smoke, and Markdown lint.
- Broader dogfood fixture runs, final public-media publication, and post-published install-channel
  checks remain open release work.

## Done Criteria

This pass is complete when:

- the release candidate has no public "dogfood implementation" framing;
- the stack descriptions and docs explain shipped behavior and limitations;
- README, crate README, LLM-authored release notes, changelog, website, and media agree about the
  feature surface;
- public media has been regenerated and published through the owning repos;
- source, dogfood, media, and release-channel validation have passed or have explicit documented
  exceptions;
- rebase work has a clean next-slice entry point after this release.
