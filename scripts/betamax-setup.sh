#!/usr/bin/env bash
# Source from a hidden tape setup to select the built binary and a fresh fixture.
set -euo pipefail
source_repo=${JK_SOURCE_REPO:-$PWD}
cd "$source_repo"
source_repo=$PWD
source scripts/betamax-fixture-env.sh
bin=${JK_BIN:-$source_repo/target/debug/jk}
[[ $bin = /* ]] || bin="$source_repo/$bin"
if [[ ! -x $bin ]]; then
    printf 'Build jk first, or run the tape through scripts/run-betamax.sh: %s\n' "$bin" >&2
    return 1
fi
mkdir -p target/dogfood-artifacts/betamax target/dogfood-artifacts/readme-media
case "${1:-inspection}" in
    refs) fixture=$(bash scripts/refs-remotes-demo-fixture.sh) ;;
    selectors) fixture=$(bash scripts/shared-selector-demo-fixture.sh) ;;
    forecast) fixture=$(bash scripts/action-menu-demo-fixture.sh) ;;
    abandon) fixture=$(bash scripts/abandon-dialog-fixture.sh "${2:-0}") ;;
    workflows)
        fixture_root=$(bash scripts/workflows-demo-fixture.sh "$source_repo/target/dogfood-artifacts")
        fixture=$fixture_root/workspaces/repo
        rebase_repo=$fixture_root/rebase
        squash_repo=$fixture_root/squash
        restore_repo=$fixture_root/restore
        workspaces_repo=$fixture_root/workspaces/repo
        ;;
    *) fixture=$(bash scripts/betamax-fixture.sh "${1:-inspection}") ;;
esac
# Exercise foreground fidelity with explicit jj colors in both light and dark design captures.
if [[ ${JK_DEMO_JJ_COLORS:-0} == 1 ]]; then
    jj -R "$fixture" config set --repo colors.change_id blue
    jj -R "$fixture" config set --repo colors.commit_id magenta
fi
repo=$fixture
export JK_DEMO_REPO=$fixture
export JK_DEMO_BIN=$bin
cd "$fixture"
printf 'JK_%s\n' FIXTURE_READY
