# Observability And Failure

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `OBSERVABILITY-DISTINGUISH-FAILURE-STATES`: Distinguish partial, aborted, timed-out, denied,
  failed, and completed states because they need different recovery paths.
- `OBSERVABILITY-KEEP-DIAGNOSTICS-RETENTION-SAFE`: Keep diagnostics safe for their retention
  boundary, especially telemetry, CI artifacts, PR comments, and user-visible reports.
- `OBSERVABILITY-LOG-AT-OWNED-BOUNDARIES`: Log at owned boundaries because the best diagnostic point
  is usually where code still knows the operation, caller intent, input class, and external system
  boundary.
- `OBSERVABILITY-PRESERVE-OPERATION-CONTEXT-IN-ERRORS`: Preserve operation context in errors because
  an error such as "not found" or "permission denied" is rarely enough.
- `OBSERVABILITY-SURFACE-DURABLE-FAILURES`: Do not hide durable failures only in UI logs because a
  durable failure that only appears in an ephemeral UI log can disappear before a maintainer or user
  can act.
