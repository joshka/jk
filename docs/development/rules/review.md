# Private Context And Review Artifacts

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `REVIEW-ANSWER-QUESTIONS-BEFORE-CODE`: Answer reviewer questions before or with code updates so
  intent, tradeoffs, and remaining choices are visible instead of buried in a new patch.
- `REVIEW-CLASSIFY-PROTOTYPE-REUSE`: Classify prototype reuse as behavior, evidence, replaceable
  shape, or load-bearing boundary before changing the boundary.
- `REVIEW-DEFINE-SLICES-IN-ISSUES`: Define review-sized slices in issues because issues are often
  the first place a future PR gets shaped.
- `REVIEW-EXPLAIN-CONTROVERSIAL-CHOICES-INLINE`: Place short inline notes beside surprising review
  decisions so reviewers see the rationale at the line, file, generated artifact, or config boundary
  where the concern appears.
- `REVIEW-EXPLAIN-PR-PROBLEM-MODEL-AND-PROOF`: Explain the problem, mental model, tradeoffs,
  validation, and docs impact so reviewers do not reverse-engineer intent from the diff.
- `REVIEW-LABEL-SPECULATION-AS-INFERRED-OR-UNKNOWN`: Label speculation as inferred or unknown
  because review notes often mix facts, traces, guesses, and model-based conclusions.
- `REVIEW-LET-REVIEWERS-RESOLVE-THREADS`: Let reviewers resolve review threads unless resolution is
  unambiguous; thread resolution is a communication act, not just a UI cleanup.
- `REVIEW-MAKE-REVIEW-ARTIFACTS-STANDALONE`: Make issues, PRs, commit messages, and handoffs stand
  alone for readers who did not see the chat, notes, plan, or discarded attempts.
- `REVIEW-SEPARATE-DISCOVERY-SELECTION-IMPLEMENTATION`: Separate discovery, solution selection, and
  implementation review for unsettled scope or design; use issues, design notes, or ADRs before
  asking reviewers to judge a patch.
- `REVIEW-UPDATE-SOURCE-OF-TRUTH`: Update the owning issue, PR description, checklist, plan, or
  handoff for durable status changes; avoid comments that only nudge reviewers or restate current
  progress.
- `REVIEW-USE-ADRS-FOR-BOUNDARIES-AND-OWNERSHIP`: Use ADRs for decisions that outlive a PR, such as
  ownership, API boundaries, storage formats, runtime responsibility, and service or crate
  boundaries.
