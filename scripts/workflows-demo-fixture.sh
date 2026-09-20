#!/usr/bin/env bash
# Create isolated repositories for the combined mutation-workflow tape.
set -euo pipefail
source "$(dirname "$0")/betamax-fixture-env.sh"

artifact_root=${1:?usage: workflows-demo-fixture.sh ARTIFACT_ROOT}
mkdir -p "$artifact_root"
fixture_root=$(mktemp -d "$artifact_root/workflows.XXXXXX")

init_repo() {
    local repo=$1
    mkdir -p "$repo"
    jj git init --colocate "$repo" >/dev/null 2>&1
    jj -R "$repo" config set --repo signing.behavior drop >/dev/null 2>&1
    jj -R "$repo" config set --repo user.name "jk workflow demo" >/dev/null 2>&1
    jj -R "$repo" config set --repo user.email "jk-demo@example.invalid" >/dev/null 2>&1
    jj -R "$repo" metaedit --update-author >/dev/null 2>&1
}

rebase_repo="$fixture_root/rebase"
init_repo "$rebase_repo"
printf 'base contents\n' >"$rebase_repo/base.txt"
jj -R "$rebase_repo" file track 'root:base.txt' >/dev/null 2>&1
jj -R "$rebase_repo" describe --message "Rebase base" >/dev/null 2>&1
jj -R "$rebase_repo" new --message "Rebase destination" >/dev/null 2>&1
printf 'destination contents\n' >"$rebase_repo/destination.txt"
jj -R "$rebase_repo" file track 'root:destination.txt' >/dev/null 2>&1
jj -R "$rebase_repo" new --message "Rebase source" @- >/dev/null 2>&1
printf 'source contents\n' >"$rebase_repo/source.txt"
jj -R "$rebase_repo" file track 'root:source.txt' >/dev/null 2>&1

squash_repo="$fixture_root/squash"
init_repo "$squash_repo"
printf 'destination contents\n' >"$squash_repo/destination.txt"
jj -R "$squash_repo" file track 'root:destination.txt' >/dev/null 2>&1
jj -R "$squash_repo" describe --message "Squash destination" >/dev/null 2>&1
jj -R "$squash_repo" new --message "Squash source" >/dev/null 2>&1
printf 'source contents\n' >"$squash_repo/source.txt"
jj -R "$squash_repo" file track 'root:source.txt' >/dev/null 2>&1

restore_repo="$fixture_root/restore"
init_repo "$restore_repo"
printf 'base content\n' >"$restore_repo/file.txt"
jj -R "$restore_repo" file track 'root:file.txt' >/dev/null 2>&1
jj -R "$restore_repo" commit --message "Restore base" >/dev/null 2>&1
printf 'working content\n' >"$restore_repo/file.txt"
jj -R "$restore_repo" describe --message "Restore working content" >/dev/null 2>&1

workspaces_repo="$fixture_root/workspaces/repo"
scratch_workspace="$fixture_root/workspaces/scratch"
mkdir -p "$fixture_root/workspaces/one" "$fixture_root/workspaces/two"
init_repo "$workspaces_repo"
jj -R "$workspaces_repo" describe --message "Workspace lifecycle demo" >/dev/null 2>&1
jj -R "$workspaces_repo" workspace add "$scratch_workspace" --name scratch >/dev/null 2>&1
printf 'workspace files remain after metadata changes\n' >"$scratch_workspace/KEEP-ME.txt"

printf '%s\n' "$fixture_root"
