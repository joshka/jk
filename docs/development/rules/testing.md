# Testing And Verification

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `TEST-CHECK-IMPORTANT-FEATURE-COMBINATIONS`: Test all features and important feature combinations
  because rust feature flags can change public API, optional dependencies, cfg-gated docs, and
  compile paths.
- `TEST-CHECK-MAINTAINER-COMMANDS-IN-CI`: If the README or maintainer guide names local validation
  commands, CI should exercise the same commands or an intentionally stronger equivalent.
- `TEST-CHECK-MSRV-AND-PLATFORMS`: Run MSRV and platform checks for crates that publish those
  compatibility claims.
- `TEST-CHOOSE-VALIDATION-BY-RISK`: Choose validation by risk because different changes need
  different proof.
- `TEST-COVER-ASYNC-ROUTING-EDGE-CASES`: Cover async routing cases for unrelated input, late
  replies, timeouts, unmatched responses, and wrong-request matches.
- `TEST-COVER-LOCAL-LOGIC-WITH-UNIT-TESTS`: Cover local logic with unit tests because small pure
  logic is cheapest to test close to where it lives.
- `TEST-COVER-NAVIGATION-BOUNDARIES`: Cover navigation and scroll boundaries in tests because
  navigation and scrolling bugs usually happen at the edges: empty lists, first item, last item,
  small viewport, oversized content, saturating offsets, and repeated key presses.
- `TEST-COVER-POLICY-OUTCOMES`: Cover allowed, denied, redacted, fallback, preserved, and
  unsupported outcomes in policy tests.
- `TEST-COVER-PUBLIC-BOUNDARIES-WITH-INTEGRATION-TESTS`: Use integration tests for public behavior
  that can break across modules, crates, features, or adapters despite passing unit tests.
- `TEST-COVER-PUBLIC-EXAMPLES-WITH-DOCTESTS`: Use doctests for public examples that teach humans and
  agents how to call the API and can compile without fragile assumptions.
- `TEST-FUZZ-PARSERS-FORMATTERS-AND-STATE-MACHINES`: Use fuzzing or property tests for parsers,
  formatters, decoders, state machines, and untrusted input with large edge-case spaces.
- `TEST-KEEP-DRIFT-CLAIMS-ALIGNED`: Use drift tests to keep support claims, fixtures, docs,
  examples, and public API paths aligned.
- `TEST-KEEP-SLOW-CHECKS-OUT-OF-PR-CI`: Keep slow fuzzing, long benchmarks, and exhaustive
  compatibility checks outside required PR CI unless they are fast and deterministic.
- `TEST-MATCH-EVIDENCE-TO-SURFACE`: Match validation evidence to the changed surface because a
  change to rendered docs, terminal layout, parser output, public API, or performance needs evidence
  from that surface.
- `TEST-PREFER-DETERMINISTIC-TESTS`: Prefer deterministic tests over timing or external-state tests
  because tests that depend on timing, network state, random ordering, real clocks, or external
  services fail for reasons unrelated to the code under review.
- `TEST-PROVE-COMMAND-CONSTRUCTION-AND-DISPLAY`: Prove command construction and display behavior in
  tests because command-building code can be wrong in quoting, argument order, display redaction,
  environment handling, or platform formatting while still invoking a happy path locally.
- `TEST-PROVE-CONTRACTS-NOT-TRIVIA`: Prove contracts with tests, not implementation trivia that
  makes refactoring expensive without proving user-visible behavior.
- `TEST-RUN-DOCS-AS-FIRST-CLASS-GATE`: Run docs as a first-class validation job because docs contain
  commands, examples, feature claims, public API paths, and Rustdoc links.
- `TEST-RUN-FAST-FORMAT-AND-LINT-GATES-EARLY`: Run formatting and clippy early because those
  failures are cheap to find and noisy to review.
- `TEST-USE-REALISTIC-PARSER-SAMPLES`: Use realistic samples and safe degradation cases in parser
  tests because parser tests built only from idealized examples miss real whitespace, ordering,
  partial data, unknown fields, legacy formats, invalid input, and safe degradation behavior.
- `TEST-VALIDATE-DECLARED-MINIMUM-DEPENDENCY-VERSIONS`: Validate declared minimum dependency
  versions because cargo manifests communicate the minimum compatible versions a downstream project
  may resolve.
- `TEST-WRITE-REGRESSION-TESTS-FOR-BUG-FIXES`: Write regression tests for bug fixes that could
  silently revert, especially edge cases, integration paths, and user-reported behavior.
