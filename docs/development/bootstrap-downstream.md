# Bootstrap Downstream Guidance

Use this document when an agent is asked to bring shared development guidance into a downstream
repository.

The goal is a useful local agent map, not a verbatim replacement of the downstream repo's existing
instructions. Preserve local project rules, validation commands, architecture notes, and workflow
constraints. Add links to the copied shared guidance so future agents can load more context when a
task needs it.

## Source

- Canonical source repository: `https://github.com/joshka/practice`
- Canonical rendered reference: [Software Practices](https://www.joshka.net/practice/)
- Local copied guidance root: `docs/development/`

## Bootstrap Steps

1. Inspect the downstream repo's existing `AGENTS.md` and nearby project docs.

1. Install the copied shared guidance from a temporary clone when this repo does not yet have
   `docs/development/update.py`:

   ```bash
   temp_dir="$(mktemp -d)"
   git -c commit.gpgsign=false clone --depth 1 https://github.com/joshka/practice.git \
     "$temp_dir/practice"
   python3 "$temp_dir/practice/scripts/generate_downstream_template.py" \
     --output "$PWD" \
     --preserve-agents
   ```

1. Refresh an existing copied guidance directory with:

   ```bash
   python3 docs/development/update.py
   ```

1. Merge the shared guidance into the downstream `AGENTS.md` instead of replacing local content.

1. Keep `AGENTS.md` short. It should route agents to deeper files rather than becoming the full rule
   book.

1. Add or keep local validation commands, source-control rules, ownership boundaries, and project
   conventions.

1. Run the downstream repo's normal formatting, linting, and test checks.

1. Report what changed, what was preserved, and what validation ran.

## Prompt For Another Agent

Use this prompt when asking a fresh Codex session to bootstrap another repo:

```text
Bootstrap this repo with the shared development guidance from
https://github.com/joshka/practice.

Use the downstream bootstrap template in that repo. Preserve this repo's existing AGENTS.md
instructions, validation commands, source-control rules, and project-specific conventions. Merge the
shared guidance into AGENTS.md instead of replacing local rules.

Install or refresh:

- docs/development/bootstrap-downstream.md
- docs/development/README.md
- docs/development/update.py
- docs/development/snippets/agents/rules.md
- docs/development/rules/*.md

Add a short "Shared Development Preferences" section to AGENTS.md that points agents to the copied
docs/development files and to https://www.joshka.net/practice/ for deeper context.

If a shared rule causes friction or seems wrong for most Rust or agent work, open a guidance
feedback issue upstream rather than only patching around it locally:
https://github.com/joshka/practice/issues/new?template=guidance-feedback.yml

Keep the change small and report exactly what was copied, what local instructions were preserved,
and what validation ran or was skipped.
```

## Recommended `AGENTS.md` Entry

Adapt this section to the downstream repo's voice:

```markdown
## Shared Development Preferences

This repo carries a local copy of shared development guidance in `docs/development/`.
Use this repo's local rules first. When local guidance is silent, use the shared guidance as a
fallback.

Entry points:

- `docs/development/snippets/agents/rules.md`: compact reviewed rule pack.
- `docs/development/rules/README.md`: rule domains for targeted loading.
- `docs/development/bootstrap-downstream.md`: how to refresh and merge the guidance.
- https://www.joshka.net/practice/: rendered reference with deeper guide, rule, pattern, principle,
  mechanism, and tag context.

If a shared rule causes friction or seems wrong for most Rust or agent work, open a guidance
feedback issue instead of only patching around it locally:
https://github.com/joshka/practice/issues/new?template=guidance-feedback.yml
```

## Merge Guidance

Prefer local specificity over shared defaults. For example, keep project-specific validation such as
`just check`, `cargo +nightly fmt --all`, fixture-update commands, or release gates.

Prefer shared guidance for general agent behavior, review handoffs, jj workflow, Rust
maintainability, documentation shape, and source-control hygiene when the downstream repo does not
already have a stronger local rule.

Do not copy every source guide into `AGENTS.md`. Link to the local compact rules and the public site
for deeper context.
