# Docs Are Contracts

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `DOCS-ALIGN-README-AND-CRATE-RUSTDOC`: Keep crate README and crate-level Rustdoc aligned because
  crate users often meet the README on GitHub and the crate-level Rustdoc on docs.rs.
- `DOCS-AVOID-GENERATED-PROSE-TELLS`: Avoid generated-prose tells, including component-centered
  navigation copy, that replace concrete behavior, evidence, tradeoffs, or direct labels.
- `DOCS-AVOID-UNEARNED-PRAISE`: Avoid unearned ranking and vague praise because words such as
  "simple," "powerful," "best," and "easy" are often unearned unless the doc states the comparison
  or tradeoff.
- `DOCS-BUILD-DOCS-LIKE-USERS-READ-THEM`: Build Rust docs the way users will read them because rust
  docs are consumed through rendered Rustdoc, docs.rs feature configuration, intra-doc links,
  search, and examples.
- `DOCS-CHOOSE-DOCUMENT-TYPE`: Choose the document type before editing because a page that mixes
  tutorial, reference, explanation, decision record, and changelog work makes every reader pay for
  every mode.
- `DOCS-COMPARE-LIBRARIES-ACCURATELY`: Compare nearby libraries accurately and charitably;
  inaccurate comparisons undermine trust.
- `DOCS-DISTINGUISH-EXAMPLE-ROLES`: Name the example's primary role before expanding it; keep
  focused, canonical, survey, integration, and showcase examples from collapsing into one generic
  snippet.
- `DOCS-DOCUMENT-LIFECYCLE-AND-SIDE-EFFECTS`: Document lifecycle, ownership, side effects, feature
  flags, platform assumptions, and compatibility for APIs that create caller obligations.
- `DOCS-EXPOSE-MOVE-RISK-AND-EXAMPLE-IN-PATTERNS`: For pattern-style guidance, include the
  recognizable situation, preferred move, risk, example, and agent instruction.
- `DOCS-FRONT-LOAD-USEFUL-POINT`: Front-load the useful point because readers scan docs for the
  decision, command, invariant, or warning that matters.
- `DOCS-GROUP-RELATED-LIST-ITEMS`: Group long lists into named clusters for distinct rule families,
  reader tasks, or decision surfaces; keep short homogeneous lists flat.
- `DOCS-HIDE-CATALOG-MECHANICS`: Lead with reader-facing work areas and artifacts; mention rule IDs,
  prefixes, domains, generated indexes, source layout, or UI containers only for citation,
  automation, or contribution workflow.
- `DOCS-KEEP-MARKDOWN-LINTABLE`: Keep Markdown lintable because formatting drift adds review noise
  and makes generated or agent-edited docs harder to maintain.
- `DOCS-MAKE-GUIDANCE-REVIEW-STATE-VISIBLE`: Keep guidance status visible on reusable rules,
  patterns, principles, and mechanisms; route drafts to review queues before copying them into
  execution packs.
- `DOCS-MAKE-REVIEW-EASY-TO-INSPECT`: Make documentation review easy to inspect because docs are
  often reviewed as Markdown diffs even though users read rendered pages, generated Rustdoc,
  examples, screenshots, or command output.
- `DOCS-MARK-NONCOMPILING-EXAMPLES-HONESTLY`: Prefer compiling Rust examples, and mark noncompiling
  examples honestly because users and doctests often copy them directly.
- `DOCS-MATCH-PAGE-SHAPE-TO-READER-TASK`: For documentation sites that render Markdown, choose a
  page shape for the reader task before exposing the content. Use catalogs for choosing, prose pages
  for explanation, rule layouts for instructions, mechanism layouts for runnable checks, and
  reference layouts for source catalogs.
- `DOCS-NAME-DESTINATION-NOT-DIRECTION`: Write navigation and index copy as destination, decision,
  artifact, or work-area labels; avoid directive phrases such as "start here," "use this guide," and
  "open this guide" on reference surfaces.
- `DOCS-ONE-DOMINANT-MODE-PER-PAGE`: Pick one dominant documentation mode per page because a page
  with competing modes forces readers to switch mental models.
- `DOCS-PROSE-FOR-RELATIONSHIPS-LISTS-FOR-ENUMERATION`: Use prose for relationships and lists for
  enumeration because lists are good for fields, steps, options, and checks, but weak for explaining
  causality.
- `DOCS-PROVE-REAL-USE-WITH-EXAMPLES`: Prove real use with examples because examples that only
  construct a type or call the happy-path function do not prove that the API works in the way users
  need.
- `DOCS-PUT-UNCERTAINTY-IN-TRACKED-PLACES`: Put uncertainty in issues, ADRs, or roadmaps rather than
  user docs that should describe current truth.
- `DOCS-README-AS-ENTRY-POINT`: Keep README files as entry points because a README is usually the
  first page for humans and agents.
- `DOCS-REVIEW-CORRECTNESS-AND-RISK-FIRST`: Review docs for correctness, contract ambiguity, risk,
  drift, and operability before style polish; use severity labels to separate blocking
  misunderstandings from wording cleanup.
- `DOCS-SHOW-SIDE-EFFECTS-IN-LIVE-EXAMPLES`: Show side effects and cleanup in examples that create
  files, hit networks, write records, open terminals, spawn tasks, or mutate services.
- `DOCS-STATE-CURRENT-BEHAVIOR-NOT-ASPIRATION`: State current behavior, not aspiration, because
  aspirational docs become false contracts.
- `DOCS-TREAT-DOCS-AS-CONTRACTS`: Treat docs as contracts because humans and agents use them to
  infer supported behavior.
- `DOCS-USE-CONCRETE-DETAILS`: Use concrete nouns, real paths, defaults, commands, examples, and
  named work areas so readers do not infer the actual object.
- `DOCS-USE-DESCRIPTIVE-HEADINGS`: Use descriptive headings for reference and landing pages so the
  heading names the destination, content, or decision area, not a slogan-like instruction or
  next-step direction.
- `DOCS-USE-SOURCE-LINKS-AS-SUPPORT`: Use source links as support, not wording supply, so references
  help readers verify or compare judgment.
- `DOCS-VERIFY-COMMANDS-PATHS-AND-LINKS`: Verify example commands, file paths, and linked references
  because they act like executable instructions.
- `DOCS-WRITE-FOR-NON-LINEAR-READERS`: Write docs for non-linear readers because many readers do not
  read documentation front to back.
- `DOCS-WRITE-TECHNICAL-PROSE`: Write technical docs, not marketing, coaching, or chat, so readers
  can make correct decisions.
