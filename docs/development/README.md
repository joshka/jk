# Development Guidance

This directory carries repo-local development guidance for agents and maintainers.

The generated rule files are copied from the `joshka/practice` repo. The canonical rendered
reference is [Software Practices](https://www.joshka.net/practice/). Update local validation
commands and repo-specific notes in the repo's `AGENTS.md`, but refresh copied shared guidance from
upstream instead of editing it by hand.

From a downstream repo that does not yet have this directory, install the copied guidance with:

```bash
temp_dir="$(mktemp -d)"
git -c commit.gpgsign=false clone --depth 1 https://github.com/joshka/practice.git \
  "$temp_dir/practice"
python3 "$temp_dir/practice/scripts/generate_downstream_template.py" \
  --output "$PWD" \
  --preserve-agents
```

From this downstream repo, refresh this copy from GitHub with:

```bash
python3 docs/development/update.py
```

Set `PRACTICE_GUIDANCE_DIR=/path/to/practice` only when testing against a local source checkout.

## Files

- `bootstrap-downstream.md`: instructions for an agent bootstrapping this guidance into a repo.
- `snippets/agents/rules.md`: generated single-file reviewed-rule pack.
- `rules/README.md`: generated index for the full compressed reviewed-rule pack.
- `rules/*.md`: generated domain files containing every reviewed rule.
- `update.py`: helper that refreshes this directory from the canonical source repository.

## Adoption Notes

Keep `AGENTS.md` short enough to scan. Put the full rule pack in generated domain files so agents
can load only the domains relevant to the task, while still ensuring every reviewed rule is
represented downstream.

This template intentionally copies compact agent-facing guidance rather than every source guide,
pattern, principle, and mechanism. When a compact rule needs more context, use the public reference
site or the canonical source repo.

Update local validation commands, source-control notes, and project-specific boundaries in
`AGENTS.md` or nearby local docs. If a generated rule is wrong for most projects, open a [guidance
feedback issue](https://github.com/joshka/practice/issues/new?template=guidance-feedback.yml),
update the canonical `joshka/practice` repo, and recopy the generated files.
