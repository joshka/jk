#!/usr/bin/env bash
# Build a small disposable history without modifying the checkout's jj state.
set -euo pipefail

cd "$(dirname "$0")/.."
mkdir -p target/betamax-preview
fixture=$(mktemp -d "$PWD/target/betamax-preview/repo.XXXXXX")
printf '%s\n' "$fixture" > target/betamax-preview/repository

# Keep developer identity, signing, aliases, and templates out of the fixture.
export JJ_CONFIG=/dev/null
export JJ_USER='Preview Author'
export JJ_EMAIL='preview@example.com'
export JJ_TIMESTAMP='2026-01-01T12:00:00Z'
export JJ_OP_TIMESTAMP="$JJ_TIMESTAMP"
jj git init "$fixture"
cd "$fixture"
jj config set --repo signing.behavior drop
printf '# Preview project\n\nA small project for terminal previews.\n' > README.md
jj describe -m 'Start preview project'
jj new -m 'Add greeting'
printf 'Hello from jk!\n' > greeting.txt
jj new -m 'Improve greeting'
printf 'Hello from jk!\nReview changes from your terminal.\n' > greeting.txt
