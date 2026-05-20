# Source And Context Hygiene

Generated from the canonical `joshka/practice` rule catalog. Do not edit copied rule text by hand;
update the source repo and recopy this file.

## Instructions

- `SOURCE-GENERALIZE-PROJECT-SPECIFIC-RULES`: Generalize project-specific rules before promotion
  because local mining often starts from one repository, tool, provider, or incident.
- `SOURCE-KEEP-BINARIES-OUT-OF-SOURCE-CONTROL`: Keep binary artifacts out of Git history; use Git
  LFS, release assets, PR uploads, CI artifacts, or external storage instead.
- `SOURCE-MAKE-SHARED-ARTIFACTS-STANDALONE`: Make shared docs, templates, generated packs, and
  reusable guidance stand alone because they leave the development session.
- `SOURCE-PREFER-PRIMARY-STABLE-SOURCES`: Use primary or stable sources because a reader needs to
  verify, compare, or challenge durable guidance.
