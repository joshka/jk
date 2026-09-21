#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/betamax-fixture-env.sh"

fixture_root=$(mktemp -d "${TMPDIR:-/tmp}/jk-workspaces.XXXXXX")
repo="$fixture_root/repo"
scratch="$fixture_root/scratch"

jj git init "$repo" >/dev/null
jj -R "$repo" describe --message "Workspace lifecycle demo" >/dev/null
jj -R "$repo" workspace add "$scratch" --name scratch >/dev/null
printf 'workspace files remain after metadata changes\n' >"$scratch/KEEP-ME.txt"

printf '%s\n' "$repo"
