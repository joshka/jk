#!/usr/bin/env bash
# Create local client and remote stores; no network remote or development graph is used.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/betamax-fixture-env.sh
mkdir -p target/dogfood-artifacts/fixtures
fixture_root=$(mktemp -d "$PWD/target/dogfood-artifacts/fixtures/refs.XXXXXX")
jj git init --no-colocate "$fixture_root/client" >&2
jj git init --no-colocate "$fixture_root/remote" >&2
jj -R "$fixture_root/client" config set --repo signing.behavior drop
jj -R "$fixture_root/remote" config set --repo signing.behavior drop
jj -R "$fixture_root/client" describe -m 'Prepare remote workflow' >&2
jj -R "$fixture_root/client" bookmark create demo >&2
jj -R "$fixture_root/client" git remote add fixture "$fixture_root/remote/.jj/repo/store/git" >&2
printf '%s\n' "$fixture_root/client"
