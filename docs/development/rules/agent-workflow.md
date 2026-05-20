# Agent Workflow

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `AGENT-BUDGET-FOR-FEEDBACK-LOOPS`: Budget tokens and time for reading, editing, checks, failure
  inspection, and proof reporting.
- `AGENT-DEFINE-GOOD-BEFORE-JUDGMENT-HEAVY-WORK`: Before naming, grouping, documentation voice, API
  shape, or rule IDs, define the quality bar so the agent has concrete goalposts.
- `AGENT-DISTILL-FROM-BLESSED-ARTIFACTS`: Distill conventions, principles, and review expectations
  from accepted artifacts before inventing style; adapt the accepted pattern to the current task.
- `AGENT-ENCODE-NONFUNCTIONAL-REQUIREMENTS`: Encode nonfunctional requirements that may not appear
  in the diff, such as latency, accessibility, security, privacy, determinism, and compatibility.
- `AGENT-GIVE-OBJECTIVES-WITH-BOUNDARIES`: Give agents objectives with boundaries so they can adapt
  to real repo structure without following brittle step lists.
- `AGENT-GRANT-SCOPED-CAPABILITIES`: Grant scoped agent capabilities because agents with broad
  authority can accidentally mutate external systems, publish state, delete files, or read secrets
  unrelated to the task.
- `AGENT-ISOLATE-WORKSPACES-BY-TASK`: Isolate workspaces for parallel agent tasks so diffs,
  validation, and ownership stay unambiguous.
- `AGENT-KEEP-DURABLE-CONTEXT-ON-DISK`: Keep durable context on disk because prompt context
  disappears, compacts, or becomes invisible to future sessions.
- `AGENT-KEEP-SECRETS-OUT-OF-CONTEXT`: Keep secrets out of context because secrets pasted into
  prompts, docs, logs, or test output can be retained, repeated, or committed accidentally.
- `AGENT-MAKE-BAD-OUTPUT-HARD`: Make bad output mechanically hard because repeated prompt reminders
  are weaker than a repo that rejects bad output mechanically.
- `AGENT-PREFER-BUILD-PRESERVING-EDITS`: Prefer build-preserving edits on natural paths so failures
  stay close to the edit that caused them.
- `AGENT-PREFER-IN-DISTRIBUTION-TOOLS`: Prefer in-distribution tools for agent-facing work because
  trained and tested tool paths tend to be more reliable.
- `AGENT-PREFER-TOOLS-OVER-PROMPTS`: Prefer tools and checks over repeated prompting; put repeated
  instructions in a tool, check, template, or guide.
- `AGENT-PRESENT-CONCRETE-NEXT-OPTIONS`: After a validated chunk, name the next concrete chunk and
  why to choose it so the maintainer controls scope cheaply.
- `AGENT-PRESERVE-HUMAN-WORK`: Preserve unrelated human work because agents share a working tree
  with human edits and sometimes other agents.
- `AGENT-PRESERVE-INTENT`: Preserve intent over literalism because literal execution can satisfy the
  words while missing the goal.
- `AGENT-PRODUCE-REVIEW-PACKETS`: Produce review packets for agent output because agent output often
  spans code, docs, generated artifacts, and validation logs.
- `AGENT-PROVE-SECURITY-IMPACT`: Prove security impact separately from hypotheses because security
  claims are easy to overstate.
- `AGENT-REPORT-PROOF-IN-HANDOFFS`: Report proof in handoffs instead of confidence language, because
  confidence is not evidence.
- `AGENT-REVIEW-OUTPUT-AS-FUTURE-MAINTAINER`: Review agent output as a future maintainer: check
  correctness, edge cases, API clarity, documentation truthfulness, readable ownership, focused
  tests, validation proof, and residual risk.
- `AGENT-SEPARATE-NOTES-FROM-CORRECTIONS`: Separate note capture from correction during fast review
  so clustered feedback becomes cleaner edits or durable guidance.
- `AGENT-SPEND-HUMAN-ATTENTION-ON-AMBIGUITY`: Spend human attention on ambiguity because agents can
  spend a lot of effort executing through an unresolved decision.
- `AGENT-SUGGEST-LOCAL-OVERRIDE-FILES`: Suggest ignored override files for checkout-only facts such
  as local jj topology, plan directories, machine paths, or temporary repo notes.
- `AGENT-TURN-FEEDBACK-INTO-GUIDANCE`: Turn repeated feedback into durable guidance because repeated
  corrections such as "show why," "name the next thing," or "do not use abstract rule names" are
  process bugs.
- `AGENT-USE-AGENTS-MD-AS-MAP`: Use `AGENTS.md` to route agents to deeper guides because a full rule
  set would make the file hard to scan.
- `AGENT-VERIFY-RISKY-CHANGES-WITH-CANARIES`: Use canaries for changes that can pass local tests but
  fail under real traffic, docs rendering, provider state, or users.
