# Measure Before Optimizing

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `PERF-AVOID-SINGLE-RUN-CONCLUSIONS`: Do not decide performance from one short benchmark run
  because one short benchmark run can be dominated by warmup, scheduling, cache state, background
  load, or measurement noise.
- `PERF-JUSTIFY-COMPLEXITY-CHURN-AND-DEPENDENCIES`: Justify complexity, churn, and dependency cost
  because performance work adds branches, unsafe code, caching, data structure churn, or
  dependencies.
- `PERF-MEASURE-GOAL-CHANGE-COMPARE`: State the performance goal, measurement, change, and
  comparison so reviewers can evaluate the patch.
- `PERF-OPTIMIZE-MEASURED-HOTSPOTS`: Optimize measured hotspots, not interesting code that runs
  once, is off the critical path, or is invisible to users.
- `PERF-RECORD-BENCHMARK-PROVENANCE`: Record benchmark provenance so timing numbers remain
  comparable later.
- `PERF-RUN-CORRECTNESS-FIRST`: Run correctness before performance timing because fast wrong code is
  still wrong, and correctness failures can invalidate timing data.
- `PERF-RUN-TIMING-BENCHMARKS-SEQUENTIALLY`: Run timing benchmarks sequentially so CPU, cache,
  memory, disk, and thermal contention do not distort results.
