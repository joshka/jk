# Change Shape

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `CHANGE-AVOID-SPECULATIVE-PUBLIC-API`: Avoid speculative public API; add public names, types,
  extension points, and compatibility promises only for concrete current needs or explicitly
  accepted near-term requirements.
- `CHANGE-AVOID-UNNECESSARY-DEPENDENCY-CHURN`: Avoid dependency churn unless it is necessary for the
  task; dependency updates change lockfiles, feature graphs, minimum versions, build output, and
  downstream compatibility.
- `CHANGE-IDENTIFY-OWNING-MODULE-BEFORE-EDITING`: Identify the owning module before editing because
  editing the first file that mentions a behavior can put new logic in a caller, facade, test
  helper, or adapter that does not own the concept.
- `CHANGE-ISOLATE-CONTROVERSIAL-CHANGES`: Isolate controversial changes because formatting, renames,
  API breaks, dependency changes, unsafe code, large rewrites, and behavior changes all invite
  different review questions.
- `CHANGE-MINIMAL-BUT-COMPLETE`: Keep each change minimal but complete because a change that is too
  large hides risk, but a change that is too small can leave reviewers with an unexplained
  half-step.
- `CHANGE-PIN-BEHAVIOR-WITH-EARLY-TESTS`: Use early tests to pin current behavior before changing
  messy behavior so reviewers can separate existing behavior from the intended change.
- `CHANGE-PREFER-SMALL-FOLLOW-UPS`: Prefer small follow-ups for adjacent cleanup, docs drift, naming
  issues, or broader refactoring so the current diff keeps its purpose.
- `CHANGE-PRESERVE-UNOWNED-WORK`: Preserve unowned work because a working tree can contain edits
  from the user, another agent, generated state, or an earlier in-progress change.
- `CHANGE-RESPECT-GENERATED-ARTIFACT-OWNERSHIP`: Edit the source input, template, release metadata,
  or generator config for generated artifacts; hand edit generated output only for tool workflows
  that make curation durable.
- `CHANGE-SEPARATE-STRUCTURE-FROM-BEHAVIOR`: Separate structure and behavior changes because the
  combined diff makes reviewers prove both meaning preservation and new behavior.
- `CHANGE-SYNC-GENERATED-ARTIFACTS`: Update them with the source change because checked-in generated
  files, lockfiles, snapshots, API listings, or agent packs are review surfaces.
- `CHANGE-TREAT-AND-AS-SCOPE-WARNING`: Treat `and` in a change description as a scope warning
  because a change titled "fix parser and update docs and clean API" often contains multiple review
  units.
- `CHANGE-USE-ONE-PURPOSE-PER-CHANGE`: Use one purpose per change so reviewers can ask one main
  question: did this accomplish the stated goal?
