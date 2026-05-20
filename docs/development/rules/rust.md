# Rust API And Crate Shape

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `RUST-ADD-BENCHMARKS-FOR-PERFORMANCE-CLAIMS`: Benchmark Rust hot paths or allocation-sensitive
  changes before making or preserving performance claims.
- `RUST-ALIGN-RELEASE-SUPPORT-CLAIMS`: Keep release claims aligned across `Cargo.toml`, README,
  Rustdoc, changelog, docs.rs metadata, examples, and CI support matrices.
- `RUST-AVOID-BROAD-CONTEXT-AND-CALLBACKS`: Avoid broad context objects and callback-heavy control
  flow because broad context objects and callback-heavy flows hide which inputs, effects, and
  ordering a function needs.
- `RUST-AVOID-EMPTY-WRAPPER-TYPES`: Avoid wrapper types that add no invariant, behavior, or
  ownership clarity; a wrapper should earn its name.
- `RUST-AVOID-GIANT-CRATE-ROOTS`: Avoid giant crate roots because a giant `lib.rs` or `main.rs`
  makes the crate root carry every concept, helper, import, and re-export.
- `RUST-AVOID-GLOB-REEXPORTS`: Use explicit public re-exports in Rust facades so the exported API is
  reviewable at the facade.
- `RUST-AVOID-INLINE-MODULES`: Avoid inline modules except for tests, preludes, and generated code
  because named files improve navigation, search, and ownership.
- `RUST-AVOID-MOD-RS-BY-DEFAULT`: Prefer named Rust module root files over `mod.rs` for module
  concepts that benefit from a filename.
- `RUST-AVOID-OVERCOMMENTING-TRIVIAL-CODE`: Remove Rust comments that merely restate obvious code;
  keep comments for invariants, tradeoffs, contracts, and safety.
- `RUST-AVOID-PATH-ATTRIBUTE`: Avoid `#[path]` in Rust modules unless an unusual generated or
  platform layout is genuinely clearer.
- `RUST-AVOID-PUBLIC-DEPENDENCY-COUPLING`: Avoid leaking dependency types in public APIs unless
  integration is the point; exposed dependency types make downstream users care about that
  dependency's version, features, and semantics.
- `RUST-AVOID-TINY-MODULE-MAZES`: Keep Rust helpers near their use unless splitting a file gives a
  real concept its own home.
- `RUST-AVOID-VAGUE-DOCS-AND-GENERIC-EXAMPLES`: Replace vague Rustdoc and generic examples with
  contract details and examples that prove realistic use.
- `RUST-CENTRAL-ITEM-FIRST`: Put the main Rust type, trait, enum, or function before helpers so
  readers find the module concept first.
- `RUST-CHOOSE-GENERICS-AND-TRAIT-OBJECTS-DELIBERATELY`: Choose generics, stored type parameters,
  and trait objects deliberately because they trade off monomorphization, object safety, compile
  time, and ergonomics.
- `RUST-COMPARE-CRATES-BY-FIT-AND-TRADEOFF`: Compare Rust crates accurately and charitably by fit,
  scope, and tradeoff, not universal superiority.
- `RUST-CONFIGURE-DOCS-RS`: Configure docs.rs metadata intentionally because docs.rs is often the
  rendered documentation users see first.
- `RUST-CONSIDER-DOWNSTREAM-API-IMPACT`: Consider downstream impact before changing public API
  because changing a public Rust API can break external imports, trait impls, type inference, docs,
  examples, and semver expectations.
- `RUST-CONTAIN-UNSAFE`: Keep unsafe small, wrapped, documented, and tested through the safe API
  because unsafe concentrates obligations the compiler cannot check.
- `RUST-DENY-ACCIDENTAL-UNSAFE`: Deny accidental unsafe code in crates that do not need unsafe so
  unsafe blocks fail loudly.
- `RUST-DO-NOT-DEFAULT-PUB-CRATE`: Do not default to `pub(crate)` because crate-wide visibility
  still expands the internal surface and lets modules reach into each other.
- `RUST-DO-NOT-PIN-PATCH-VERSIONS`: Avoid patch-pinned `Cargo.toml` requirements unless the patch
  supplies an API, behavior, or fix the crate actually needs.
- `RUST-DOCUMENT-CURRENT-IMPLEMENTED-BEHAVIOR`: Write Rust public docs in present tense for
  implemented behavior; label planned behavior as future work or scope.
- `RUST-DOCUMENT-FEATURE-CONTRACTS`: Document Rust feature flags by naming the public API, runtime
  behavior, dependencies, platform support, or docs coverage they enable.
- `RUST-DOCUMENT-LIFECYCLE-SIDE-EFFECTS`: Document startup, shutdown, cleanup, cancellation,
  ordering, and coexistence for Rust APIs with host, process, runtime, terminal, network, or UI side
  effects.
- `RUST-DOCUMENT-PERFORMANCE-CONTRACTS`: Document blocking behavior, allocation expectations,
  buffering, clone cost, and runtime constraints that callers may reasonably depend on.
- `RUST-DOCUMENT-PUBLIC-PANIC-CONTRACTS`: Document public panic contracts as precondition violations
  because a public panic is a contract boundary: the caller violated a precondition or the crate has
  a bug.
- `RUST-DOCUMENT-SCHEDULING-FOR-LONG-ASYNC`: Document scheduling expectations for async work that
  can starve executors, ignore cancellation, hold locks, or rely on runtime assumptions.
- `RUST-DOCUMENT-VISIBILITY-OWNERSHIP`: Update names and docs while widening Rust visibility so the
  owning concept and intended callers are clear.
- `RUST-ENCODE-DURABLE-RULES-IN-LINTS`: Use lint configuration for durable project policy, not
  transient taste or migration states that need frequent exceptions.
- `RUST-EXPOSE-PRIMARY-PATH-FROM-CRATE-ROOT`: Make `lib.rs` teach and expose the primary crate path
  while pointing readers to deeper modules.
- `RUST-FORMAT-DOCS-AND-COMMENTS-CONSISTENTLY`: Format Rust doc-comment code, doc attributes,
  grouped imports, and prose comments with the project's formatter conventions.
- `RUST-GROUP-MODULE-IMPORTS`: Prefer grouped module imports over one-import-per-line style because
  grouped module imports keep related names together and make dependencies easier to scan.
- `RUST-GROUP-PRIVATE-IMPORTS-BEFORE-PUBLIC-RE-EXPORTS`: Group private imports before public
  re-exports because private imports and public re-exports answer different questions.
- `RUST-HIDE-TEST-ONLY-HELPERS`: Keep test-only helpers out of the normal public API because
  test-only helpers should not become production API or crate-wide concepts by accident.
- `RUST-IMPLEMENT-DEBUG-FOR-PUBLIC-TYPES`: Implement `Debug` for public types unless that is unsafe
  or misleading; `Debug` is the baseline diagnostic trait for Rust values.
- `RUST-IMPLEMENT-STANDARD-TRAITS-FOR-PUBLIC-ERRORS`: Implement `Debug`, `Display`, and
  `std::error::Error` for public errors that cross into callers, logs, tests, and user messages.
- `RUST-INJECT-HOST-INTERACTIONS-AT-BOUNDARIES`: Inject Rust host dependencies at boundaries for
  tests or alternate environments that need deterministic control.
- `RUST-KEEP-CI-HIGH-SIGNAL`: Keep Rust PR CI focused on fast deterministic gates, and move
  expensive checks to scheduled, manual, or release workflows.
- `RUST-KEEP-COMPATIBLE-UPDATES-IN-LOCKFILE`: Keep compatible dependency updates in the lockfile,
  not the manifest requirement, unless the crate actually needs the newer version.
- `RUST-KEEP-CONCEPTS-COHERENT`: Keep Rust modules, types, and helpers centered on one recognizable
  concept so readers can find the owner of behavior.
- `RUST-KEEP-CRATE-BOUNDARIES-NARROW`: Put Rust code and tests in the owning crate or module, and
  expose shared helpers only for intentional shared concepts.
- `RUST-KEEP-DEPENDENCY-UPDATES-INTENTIONAL`: Group routine Rust dependency updates, separate
  behavior-affecting updates, and use Cargo-aware commands to preserve manifest consistency.
- `RUST-KEEP-EDITS-SCOPED-TO-OWNING-CONCEPT`: Before editing Rust code, identify the owning module
  or crate and keep unrelated cleanup out of the change.
- `RUST-KEEP-LINTS-ACTIONABLE`: Enable Rust lints only for durable policy, and keep suppressions
  narrow with a reason.
- `RUST-KEEP-MARKDOWN-OUTSIDE-RUSTDOC-PURPOSEFUL`: Keep Rust API contracts in Rustdoc, README entry
  points in README, and use Markdown guides for long-form workflow, architecture, or process
  material.
- `RUST-KEEP-PRE-RELEASE-COMPATIBILITY-INTENTIONAL`: Clean up accidental pre-release Rust API
  compatibility after the intended API becomes clearer and the crate has not promised the old shape.
- `RUST-KEEP-PRELUDES-REEXPORT-ONLY`: Keep Rust prelude modules as import surfaces that re-export
  owned items from their real modules.
- `RUST-KEEP-PUBLIC-API-SHAPE-INTENTIONAL`: Keep public API shape intentional because every public
  item becomes something users can import, name, document, and depend on.
- `RUST-KEEP-RUSTDOC-AND-README-EXAMPLES-ALIGNED`: Update README, crate Rustdoc, doctests, and
  example projects together for Rust public example changes that teach the same contract.
- `RUST-MAKE-FEATURE-FLAGS-ADDITIVE-WHERE-POSSIBLE`: Make feature flags additive where possible
  because Rust feature unification means enabling a feature in one dependency path can affect the
  whole build.
- `RUST-MAKE-PUBLIC-API-BROWSEABLE-FROM-LAYOUT`: Align Rust public modules, files, and re-exports so
  readers can browse from public API to owning source without translation.
- `RUST-MAKE-SIDE-EFFECTS-EXPLICIT`: Expose Rust side effects in names, call sites, and Rustdoc for
  calls that mutate state, perform I/O, register globally, or start background work.
- `RUST-NAME-AUDITABLE-INTERMEDIATES`: Name intermediate Rust values because they expose ownership,
  parsing, validation, rendering, or side-effect policy decisions.
- `RUST-NAME-TESTS-BY-BEHAVIOR`: Name Rust tests for the behavior, boundary, or regression they
  protect, not just the function under test.
- `RUST-NON-EXHAUSTIVE-PUBLIC-ERRORS`: Use `#[non_exhaustive]` for public error enums unless
  exhaustive matching is intentional; integrations, validation, and provider behavior often add
  variants over time.
- `RUST-ORDER-CODE-FOR-READING`: Arrange Rust modules so public or central items appear before
  helpers and reading order follows execution or conceptual dependency.
- `RUST-ORDER-ITEMS-FOR-API-READING`: Order Rust imports, public items, inherent impls, trait impls,
  and helpers to make the file's API story easy to scan.
- `RUST-PREFER-BORING-DIRECT-CODE`: Prefer boring direct Rust over clever framework-shaped code
  because boring Rust makes ownership, error handling, and control flow visible.
- `RUST-PREFER-CONCEPT-OWNED-MODULES-AND-NAMED-FILES`: Prefer concept-owned modules and named files
  because modules should be owned by concepts, not by miscellaneous implementation layers.
- `RUST-PREFER-CONSTRUCTORS-AND-CONVERSION-TRAITS`: Prefer constructors or conversion traits that
  show whether callers are building, validating, converting, or borrowing values.
- `RUST-PREFER-EXPECT-FOR-LINT-SUPPRESSIONS`: Use `#[expect]` for lint suppressions that should
  disappear once the warning is fixed.
- `RUST-PREFER-SMALL-CLEAR-SHAPES`: Prefer small functions, narrow structs, and simple enums to
  reduce live fields, branches, lifetimes, and invariants.
- `RUST-PRESERVE-ERROR-CONTEXT`: Preserve Rust error source, operation, and recoverable context
  while wrapping or mapping failures.
- `RUST-PRESERVE-VALID-STATE-ON-FAILURE`: Stage fallible Rust refresh, parse, I/O, or render work
  before mutating usable state.
- `RUST-REEXPORT-FOR-DISCOVERY`: Use re-exports for discovery, not ownership hiding, so users find
  APIs without losing where concepts live.
- `RUST-RELEASE-ONLY-AFTER-ARTIFACT-VALIDATION`: Before publishing or tagging a Rust release,
  dry-run and inspect the package artifact, metadata, docs, examples, license files, and generated
  content.
- `RUST-REVIEW-AS-FUTURE-MAINTAINER`: Review Rust changes for the future maintainer: name
  readability causes, public API risks, docs truth, and validation gaps concretely.
- `RUST-RUN-FEATURE-GATED-VALIDATION`: Validate the feature combinations affected by a Rust change,
  including docs for features that change public items.
- `RUST-SHAPE-EXPRESSIONS-FOR-AUDITABILITY`: Use named locals, visible branches, whitespace
  paragraphs, and loops for side-effectful Rust logic that dense expressions would hide.
- `RUST-TEACH-CRATE-FROM-CRATE-ROOT`: Teach the crate from the crate root because the crate root is
  the first Rustdoc page and often the first source file a reader opens.
- `RUST-TIE-OPTIONAL-DEPENDENCIES-TO-NAMED-FEATURES`: Keep optional dependencies tied to clearly
  named features because they become part of the feature contract.
- `RUST-USE-BUILDERS-FOR-OPTIONAL-OR-VALIDATED-FIELDS`: Use builders for many optional fields or
  cross-field validation because constructors with many optional arguments or cross-field validation
  become hard to call correctly and hard to extend compatibly.
- `RUST-USE-DEBUG-ASSERT-FOR-INTERNAL-INVARIANTS`: Use `debug_assert!` for internal Rust invariants,
  not for public validation or safety requirements.
- `RUST-USE-DIRECTORY-MODULES-AS-TABLES-OF-CONTENTS`: Use directory-root modules as tables of
  contents because a directory-root module should orient readers to the submodules it owns.
- `RUST-USE-DOC-INLINE-FOR-CANONICAL-REEXPORTS`: Use `#[doc(inline)]` only for Rust re-exports whose
  facade path is the canonical public landing point.
- `RUST-USE-FIELD-INIT-SHORTHAND`: Use Rust field init shorthand for same-name fields unless
  explicit mapping clarifies conversion or meaning.
- `RUST-USE-FUNCTIONS-FOR-INCIDENTAL-TYPES`: Prefer regular functions because a type name is
  incidental and does not own the operation or invariant.
- `RUST-USE-HONEST-MINIMUM-DEPENDENCIES`: Use the lowest honest compatible dependency requirement
  because the manifest should state the lowest compatible dependency versions the crate honestly
  supports.
- `RUST-USE-MEANINGFUL-STANDARD-TYPES`: Use standard library types that carry meaning because
  standard library types such as `PathBuf`, `NonZeroUsize`, `Duration`, `Cow`, `Arc`, and `Result`
  carry familiar ownership and invariant signals.
- `RUST-USE-SEND-STATIC-ACROSS-TASKS`: Use `Send + static` bounds for values, futures, errors, and
  handles that cross task or thread boundaries.
- `RUST-VALIDATE-BUILDERS-ON-BUILD`: Validate Rust builder cross-field invariants in `build` and
  return an error for fallible construction.
- `RUST-VALIDATE-PACKAGE-CONTENTS-BEFORE-RELEASE`: Validate package contents before release because
  the crate package is what users receive, not the working tree.
- `RUST-VALIDATE-RUST-DOCS-AS-CODE`: After Rust documentation changes, run the relevant docs build,
  doctests, feature-gated checks, and Markdown lint for the changed surface.
- `RUST-VALIDATE-SEMVER-BREAKS-AGAINST-EXTERNAL-USE`: Validate semver-breaking changes against real
  external use because semver tools can detect many API breaks, but real downstream code shows how
  the public surface is actually used.
- `RUST-VALIDATE-UNSAFE-THROUGH-SAFE-API`: Validate Rust unsafe code through its safe API wrapper,
  with internal tests only as supporting evidence.
- `RUST-WORKING-RUST-CODE-NOT-ENOUGH`: Working Rust code is not enough because rust code can compile
  while still being hard to read, poorly documented, wrongly public, feature-fragile, or painful for
  downstream users.
- `RUST-WRITE-ACTIONABLE-ERROR-DISPLAY`: Write human-oriented and actionable error `Display` output
  because `Display` is often what users, CLIs, logs, and support messages show.
- `RUST-WRITE-PUBLIC-DOCS-FOR-CALLER-TASKS`: Write Rustdoc for caller tasks: begin with a concise
  behavior sentence, use prose for arguments, and cross-link only to improve understanding or
  auditability.
- `RUST-WRITE-RUSTDOC-AS-API-CONTRACT`: Write Rustdoc as caller-facing contract text, not decoration
  or generic prose.
