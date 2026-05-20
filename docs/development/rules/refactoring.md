# Local Reasoning And Refactoring

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `REFACTORING-ALIGN-SEAMS-WITH-REAL-VARIATION`: Align seams with observed variation such as
  backends, policies, protocols, test doubles, or ownership boundaries, not hypothetical variation.
- `REFACTORING-DO-NOT-OVER-APPLY-DRY`: Do not over-apply DRY because two blocks that look similar
  may change for different reasons.
- `REFACTORING-EXTRACT-CONCEPT-HELPERS`: Extract helpers only for real concept boundaries, not just
  to hide a few lines from the reader.
- `REFACTORING-KEEP-LINEAR-STORY-VISIBLE`: Keep linear work visible because the clearest story is
  read input, validate, transform, then emit result.
- `REFACTORING-KEEP-WEAK-ABSTRACTIONS-CLOSE-TO-THEIR-USE`: Keep weak abstractions close to their use
  because new abstractions are often tentative.
- `REFACTORING-MAKE-EDGE-CASES-EXPLICIT`: Make edge-case behavior visible near correctness-sensitive
  branches, calculations, or returns; prefer stronger types for reusable invariants.
- `REFACTORING-PREFER-LOCAL-REASONING`: Prefer designs where readers can see relevant state,
  invariants, and effects nearby instead of reconstructing distant context.
- `REFACTORING-PREFER-LOOPS-FOR-SIDE-EFFECTS`: Prefer loops over combinators for business-logic side
  effects because iterator chains are compact, but business-logic side effects often need named
  steps, early exits, logging, error handling, or comments.
- `REFACTORING-USE-WHITESPACE-AS-FUNCTION-PARAGRAPHS`: Use whitespace as function paragraphs because
  blank lines can show that a function has phases: gather inputs, validate, calculate, perform
  effects, return.
