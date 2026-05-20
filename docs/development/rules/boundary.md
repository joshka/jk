# Explicit Boundaries Preserve Correctness

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `BOUNDARY-AVOID-GLOBAL-MUTABLE-STATE`: Avoid global mutable state because it hides ownership,
  ordering, reset, and concurrency requirements.
- `BOUNDARY-CHOOSE-RESOURCE-IDENTITY-MODEL`: Choose the resource identity model up front because a
  system that mutates individual records behaves differently from one that mutates record sets,
  files, sessions, handles, or whole documents.
- `BOUNDARY-DEFINE-COMPACTION-INVARIANTS`: Define explicit budget and cut-point invariants before
  compaction deletes, summarizes, or moves information.
- `BOUNDARY-DEFINE-HOOK-FAILURE-POLICY`: Define hook failure policy because hooks can fail before,
  during, or after the main operation.
- `BOUNDARY-DISTINGUISH-INPUT-CLASSES`: Keep unknown, unsupported, denied, and preserved inputs
  distinct because each class needs different treatment.
- `BOUNDARY-EXPOSE-PARTIAL-STREAM-OUTPUT`: Expose partial provider-stream output without making it
  authoritative before the final result arrives.
- `BOUNDARY-GIVE-TOOLS-IDENTITY-POLICY-AND-LIMITS`: Give tool boundaries typed identity,
  authorization policy, cancellation, and output limits before crossing into filesystem, shell,
  network, provider, or user-visible effects.
- `BOUNDARY-GROUND-INTEGRATIONS-IN-PRIMARY-SOURCES`: Ground integration behavior in primary source
  documentation because provider and platform behavior changes.
- `BOUNDARY-IDENTIFY-ANEMIC-STATE-MACHINES`: Identify anemic state machines in auth flows, UI state,
  async routing, setup wizards, and lifecycle code.
- `BOUNDARY-KEEP-BACKEND-ADAPTERS-AT-EDGE`: Keep backend adapters at the edge because
  backend-specific APIs for terminals, storage, network providers, or runtimes spread quickly if
  they enter core logic.
- `BOUNDARY-MAKE-AMBIENT-INPUTS-EXPLICIT`: Make ambient inputs explicit because time, randomness,
  environment variables, current directories, locale, terminal size, network clients, and process
  state can change behavior without appearing in function signatures.
- `BOUNDARY-MAKE-DYNAMIC-CONFLICTS-DETERMINISTIC`: Make dynamic registration conflicts deterministic
  and explicit because dynamic registration from plugins, generated code, guests, or config can
  produce duplicate names, ordering conflicts, or shadowed handlers.
- `BOUNDARY-MAKE-EXEC-TOOLS-NONINTERACTIVE`: Make exec-like tools noninteractive by default because
  exec-like tools called by agents, CI, or background tasks cannot safely wait for prompts, editors,
  pagers, or credential UI.
- `BOUNDARY-MAKE-POLICY-BOUNDARIES-EXPLICIT`: Route policy-sensitive side effects through an
  explicit policy boundary before execution, and make allowed, denied, redacted, fallback,
  preserved, and unsupported outcomes visible to callers.
- `BOUNDARY-MODEL-REAL-UPSTREAM-SURFACE`: Model each integration as the real upstream surface it
  exposes because adapters should not pretend a provider supports a cleaner or broader API than it
  actually does.
- `BOUNDARY-NAME-LIFECYCLE-TRANSITIONS`: Treat lifecycle transitions as named operations because
  creation, activation, cancellation, teardown, reload, and promotion are different operations with
  different invariants.
- `BOUNDARY-PARSE-UNCERTAINTY-AT-EDGE`: Push uncertainty to the boundary, then pass trusted values
  inward from parsed strings, JSON, CLI args, provider responses, and user input.
- `BOUNDARY-READ-NORMALIZE-COMPARE-MUTATE`: Reconcile external state by reading, normalizing,
  comparing, then mutating so formatting, ordering, defaults, and outside actors do not hide drift.
- `BOUNDARY-REJECT-UNSUPPORTED-SHAPES`: Reject unsupported shapes early with clear errors because
  unsupported names, values, TTLs, targets, record families, protocols, or config modes should fail
  at the boundary with a clear error.
- `BOUNDARY-REPORT-PROVIDER-DIAGNOSTICS`: Report provider freshness, permissions, budget, load,
  cache, and degradation diagnostics so callers know how trustworthy the data is.
- `BOUNDARY-SEPARATE-PURE-CORE-FROM-EFFECTS`: Separate pure computation from I/O, rendering,
  mutation, and global state because that gives tests a stable behavior surface.
- `BOUNDARY-SEPARATE-UI-AND-APP-STATE`: Keep UI state separate from application-owned state because
  UI state such as selection, scroll offset, focus, expanded rows, or transient input mode changes
  at a different rate than application-owned data.
- `BOUNDARY-STAGE-GENERATED-BEHAVIOR`: Stage generated or reloadable behavior before promotion
  because generated, reloadable, or plugin-provided behavior can be malformed, stale, or
  incompatible with the current runtime.
- `BOUNDARY-TRACK-DYNAMIC-REGISTRATION-PROVENANCE`: Track provenance for extension, guest,
  generated-code, or config registrations so conflicts and failures identify their source.
- `BOUNDARY-TREAT-TERMINAL-UI-AS-PRODUCT-SURFACE`: Treat terminal UI as a product surface with
  platform-specific contracts because terminal UI is not just debug output.
- `BOUNDARY-USE-CONSERVATIVE-TERMINAL-DEFAULTS`: Use conservative terminal defaults because
  terminals vary in color support, width, input behavior, fonts, and accessibility settings.
