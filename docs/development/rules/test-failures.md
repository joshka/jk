# Tests Should Explain Failures

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `TEST-AVOID-OPAQUE-BOOLEAN-ASSERTIONS`: Avoid boolean assertions for values with multiple failure
  causes because an assertion like `assert!(items.contains(x))` or `assert!(result.is_ok())` can
  fail for many reasons while showing little useful state.
- `TEST-OPTIMIZE-FAILURE-OUTPUT`: Optimize tests for useful failure output because a passing test is
  useful, but a failing test is where maintainers and agents spend repair time.
- `TEST-SPLIT-UNRELATED-ASSERTIONS`: Split unrelated assertions because one failing check would hide
  the real scope or cause of a regression.
