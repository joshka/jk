# Documentation Guidance For Agents

Load this document when adding or changing Rustdoc, module comments, README-like material,
architecture notes, command docs, or user-facing explanations.

## Documentation Role

Documentation in `jk` should explain durable intent and local ownership. It should help a maintainer
understand why the code is shaped around jj CLI output, view-local behavior, sticky file context,
and terminal UI state.

Do not document trivia. A comment that restates a function name or a single obvious assignment makes
the file harder to scan.

## Module Documentation

Each important module should answer the questions a non-author maintainer has when landing there:

- What concept does this module own?
- What adjacent concept does it intentionally not own?
- Which jj, terminal, rendering, or navigation assumption shapes the code?
- Where should new behavior in this area be added?

Good module docs are short. Two or three concrete sentences are often enough.

## Rustdoc For Types And Functions

Add Rustdoc where an item is public, central to a module, or enforces an invariant that is not
obvious from the fields.

Rustdoc should cover the caller-facing contract:

- what the type or function represents;
- which IDs, coordinates, rows, offsets, or command args it accepts;
- what side effects occur, especially jj execution and clipboard writes;
- how errors are surfaced;
- what state is preserved or clamped after refresh;
- why an output-shape assumption is valid enough for this parser.

Use `# Errors` for fallible public APIs when the failure is meaningful to a caller. For
crate-internal functions, prefer a short direct sentence when a full Rustdoc section would be noise.

## Comments

Use comments for non-obvious policy and ordering:

- rendered jj output is preserved instead of regenerated;
- change IDs are preferred over commit IDs for navigation;
- blank-line behavior keeps sticky file headers visually aligned with jj;
- scroll math avoids dead key presses or invalid offsets;
- a parser intentionally recognizes only a conservative subset.

Avoid comments that say what the next line of code already says.

## Examples

Examples should show practical use, not just construction. If adding examples later, prefer examples
that demonstrate a real workflow:

- running the TUI with jj args;
- opening a graph row into show or diff;
- copying a change ID or file label;
- searching rendered output;
- preserving rendered styles from jj output.

Keep examples small and deterministic. Avoid examples that depend on a specific local repository
unless they are clearly marked as illustrative.

## Truthfulness

Do not document planned behavior as if it exists. If a feature is partial, experimental,
platform-specific, or dependent on jj output shape, say so plainly.

When docs describe compatibility, be specific about what is preserved:

- user jj config;
- templates;
- colorized terminal output;
- graph symbols;
- diff format;
- command-line arguments.

Do not imply that `jk` understands the full jj repository model unless the code actually does.

## Markdown Style

Follow the repository markdown conventions:

- wrap prose at 100 characters;
- put blank lines after headings and around lists and code blocks;
- use fenced code blocks with a language;
- use `1.` markers for numbered lists;
- run `panache format --check` and `panache lint --check` on changed Markdown when practical.

Keep agent-facing docs organized by task. If a topic is only relevant while testing, put it in
`docs/agent/testing.md`; if it is about architecture, put it in `docs/agent/architecture.md`.
