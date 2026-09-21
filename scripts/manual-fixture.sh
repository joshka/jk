#!/usr/bin/env bash
# Source from a hidden manual tape. All mutations stay in a fresh, isolated repository.
# shellcheck disable=SC2034 # Expose fixture variables to the tape and assertion helper.
set -euo pipefail
if [[ -n ${JK_MANUAL_PREPARED_FIXTURE:-} ]]; then
    case "$JK_MANUAL_PREPARED_FIXTURE" in
        "${JK_MANUAL_ARTIFACT_DIR:?}"/fixture-env.sh) ;;
        *) printf 'Prepared fixture must belong to this evidence directory\n' >&2; return 1 ;;
    esac
    # shellcheck source=/dev/null
    source "$JK_MANUAL_PREPARED_FIXTURE"
    [[ $manual_kind == "${1:?}" ]] || { printf 'Prepared fixture kind mismatch\n' >&2; return 1; }
    cd "$repo"
    printf 'JK_%s\n' MANUAL_FIXTURE_READY
    printf 'JK_%s\n' FIXTURE_READY
    return 0
fi
source_repo=${JK_SOURCE_REPO:-$PWD}
source_repo=$(cd "$source_repo" && pwd)
bin=${JK_BIN:?JK_BIN must point to the binary built from this checkout}
[[ $bin = /* && -x $bin ]] || { printf 'JK_BIN must be an absolute executable path\n' >&2; return 1; }
manual_kind=${1:?usage: source manual-fixture.sh KIND}
manual_real_jj=$(command -v jj)
mkdir -p "$source_repo/target/manual-fixtures"
manual_root=$(mktemp -d "$source_repo/target/manual-fixtures/$manual_kind.XXXXXX")
manual_home="$manual_root/home"
export XDG_CONFIG_HOME="$manual_home/.config" XDG_CACHE_HOME="$manual_home/.cache" XDG_DATA_HOME="$manual_home/.local/share"
mkdir -p "$manual_home" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME" "$XDG_DATA_HOME"
export JJ_CONFIG=/dev/null JJ_USER='Alex Developer' JJ_EMAIL='alex@example.invalid'
export JJ_TIMESTAMP='2026-01-01T12:00:00Z' JJ_OP_TIMESTAMP='2026-01-01T12:00:00Z'
unset NO_COLOR
cd "$source_repo"
case "$manual_kind" in
    changes|parents|empty|command-changes)
        repo=$(bash scripts/action-menu-demo-fixture.sh)
        case "$manual_kind" in
            parents)
                cache_change=$(jj -R "$repo" log --no-graph -r @ -T change_id)
                jj -R "$repo" new @- -m 'Document cache policy' >/dev/null 2>&1
                printf '# Cache policy\n\nExpire forecasts after two minutes.\n' > "$repo/cache-policy.md"
                jj -R "$repo" edit "$cache_change" >/dev/null 2>&1
                ;;
            empty) jj -R "$repo" new -m 'Empty checkpoint' >/dev/null 2>&1 ;;
            command-changes)
                python3 - "$repo/config.json" <<'PYFIX'
from pathlib import Path
import sys
path = Path(sys.argv[1])
path.write_text(path.read_text().replace('60', '120'))
PYFIX
                ;;
        esac
        ;;
    abandon) repo=$(bash scripts/abandon-dialog-fixture.sh) ;;
    selectors) repo=$(bash scripts/shared-selector-demo-fixture.sh) ;;
    container)
        container_repo="$manual_root/container"
        jj git init --colocate "$container_repo" >/dev/null 2>&1
        jj -R "$container_repo" config set --repo signing.behavior drop
        printf '# Container workspace\n' > "$container_repo/README.md"
        jj -R "$container_repo" describe -m 'Container workspace review' >/dev/null 2>&1
        jj -R "$container_repo" workspace rename container >/dev/null 2>&1
        jj -R "$container_repo" workspace add "$container_repo/default" --name default -r @ >/dev/null 2>&1
        jj -R "$container_repo" workspace forget container >/dev/null 2>&1
        repo="$container_repo/default"
        ;;
    inspection)
        repo="$manual_root/repo"
        jj git init --colocate "$repo" >/dev/null 2>&1
        jj -R "$repo" config set --repo signing.behavior drop
        mkdir -p "$repo/src"
        printf '# Forecast service\n\nReliable forecasts for every city.\n' > "$repo/README.md"
        printf '{\n  "requestTimeoutMs": 3000,\n  "cacheTtlSeconds": 60\n}\n' > "$repo/config.json"
        for index in $(seq 1 40); do printf 'export const cacheWindow%s = %s;\n' "$index" "$index"; done > "$repo/src/cache.js"
        jj -R "$repo" describe -m 'Document the forecast service' >/dev/null 2>&1
        jj -R "$repo" bookmark create main >/dev/null 2>&1
        jj -R "$repo" new -m 'Add the forecast endpoint' >/dev/null 2>&1
        printf 'export function forecast(city) { return { city, temperature: 21 }; }\n' > "$repo/src/forecast.js"
        jj -R "$repo" new -m 'Configure request timeouts' >/dev/null 2>&1
        printf '\nRequests time out after three seconds.\n' >> "$repo/README.md"
        jj -R "$repo" new -m 'Try a longer cache window' >/dev/null 2>&1
        python3 - "$repo" <<'PY'
from pathlib import Path
import sys
repo = Path(sys.argv[1])
path = repo / 'src/cache.js'
lines = path.read_text().splitlines()
lines[3] = 'export const cacheTtlSeconds = 120; // Keep forecasts fresh.'
lines[29] = 'export const cacheLimit = 256; // Bound memory usage.'
path.write_text('\n'.join(lines) + '\n')
path = repo / 'config.json'
path.write_text(path.read_text().replace('60', '120'))
with (repo / 'README.md').open('a') as out:
    out.write('\nCache forecasts for two minutes.\n')
PY
        jj -R "$repo" describe -m 'Experiment with forecast caching' >/dev/null 2>&1
        jj -R "$repo" bookmark create candidate >/dev/null 2>&1
        ;;
    refs|refs-tracking)
        repo=$(bash scripts/refs-remotes-demo-fixture.sh)
        remote_repo=$(dirname "$repo")/remote
        printf '# Forecast publication\n' > "$repo/README.md"
        jj -R "$repo" describe -m 'Prepare remote workflow' >/dev/null 2>&1
        jj -R "$repo" bookmark create review-keep stable-a stable-b stable-c >/dev/null 2>&1
        jj -R "$repo" new -m 'Review cache publication' >/dev/null 2>&1
        printf 'export const cacheTtlSeconds = 120;\n' > "$repo/cache.js"
        jj -R "$repo" bookmark move demo --to @ >/dev/null 2>&1
        printf '# Upstream release notes\n' > "$remote_repo/RELEASE.md"
        jj -R "$remote_repo" describe -m 'Publish upstream release notes' >/dev/null 2>&1
        jj -R "$remote_repo" bookmark create incoming >/dev/null 2>&1
        jj -R "$remote_repo" git export >/dev/null 2>&1
        if [[ $manual_kind == refs-tracking ]]; then
            jj -R "$repo" git fetch --remote fixture >/dev/null 2>&1
        fi
        manual_remote_graph=$(jj -R "$remote_repo" log --no-graph -r 'all()' -T 'commit_id ++ "\n"' | sort)
        ;;

    rebase|squash|squash-multiple|restore|workspaces|workspaces-stale|run-options|run-options-immutable)
        fixture_root=$(bash scripts/workflows-demo-fixture.sh "$manual_root")
        rebase_repo="$fixture_root/rebase"
        squash_repo="$fixture_root/squash"
        restore_repo="$fixture_root/restore"
        workspaces_repo="$fixture_root/workspaces/repo"
        scratch_workspace="$fixture_root/workspaces/scratch"
        if [[ $manual_kind == rebase ]]; then
            source_change=$(jj -R "$rebase_repo" log --no-graph -r @ -T change_id)
            destination_change=$(jj -R "$rebase_repo" log --no-graph -r 'description(substring:"Rebase destination")' -T change_id)
            jj -R "$rebase_repo" new "$destination_change" -m 'Verify destination' >/dev/null 2>&1
            printf 'destination checks\n' > "$rebase_repo/destination-checks.txt"
            jj -R "$rebase_repo" new "$source_change" -m 'Verify source' >/dev/null 2>&1
            printf 'source checks\n' > "$rebase_repo/source-checks.txt"
            jj -R "$rebase_repo" edit "$source_change" >/dev/null 2>&1
        fi
        if [[ $manual_kind == squash-multiple ]]; then
            jj -R "$squash_repo" new -m 'Squash second source' >/dev/null 2>&1
            printf 'second source contents\n' > "$squash_repo/second-source.txt"
        fi
        case "$manual_kind" in
            run-options|run-options-immutable) repo=$rebase_repo ;;
            squash-multiple) repo=$squash_repo ;;
            workspaces-stale) repo=$workspaces_repo ;;
            *) variable="${manual_kind}_repo"; repo=${!variable} ;;
        esac
        ;;
    *) printf 'Unknown manual fixture: %s\n' "$manual_kind" >&2; return 2 ;;
esac
fixture=$repo
fixture_root=${fixture_root:-$manual_root}
jj -R "$repo" config set --repo colors.change_id blue
jj -R "$repo" config set --repo colors.commit_id magenta
jj -R "$repo" status >/dev/null 2>&1
if [[ $manual_kind == run-options-immutable ]]; then
    immutable_source=$(jj -R "$repo" log --no-graph -r @ -T change_id)
    jj -R "$repo" config set --repo 'revset-aliases."immutable_heads()"' "$immutable_source"
fi
manual_initial_commit=$(jj -R "$repo" log --ignore-working-copy --no-graph -r @ -T commit_id)
manual_initial_change=$(jj -R "$repo" log --ignore-working-copy --no-graph -r @ -T change_id)
manual_initial_operation=$(jj -R "$repo" op log --no-graph --limit 1 -T id)
manual_initial_graph=$(jj -R "$repo" log --ignore-working-copy --no-graph -r 'all()' -T 'commit_id ++ "\n"' | sort)
manual_artifacts=${JK_MANUAL_ARTIFACT_DIR:-$manual_root/evidence}
mkdir -p "$manual_artifacts"
python3 - "$manual_artifacts/fixture.json" "$repo" "$manual_kind" "$manual_initial_commit" "$manual_initial_operation" "$bin" <<'PY'
import json
from pathlib import Path
import sys
output, repo, kind, commit, operation, binary = sys.argv[1:]
Path(output).write_text(json.dumps({'repository': repo, 'kind': kind, 'initial_commit': commit,
    'initial_operation': operation, 'binary': binary, 'isolated_jj_config': '/dev/null'}, indent=2) + '\n')
PY
export JK_DEMO_REPO=$repo JK_DEMO_BIN=$bin
cd "$repo"
if [[ $manual_kind == workspaces-stale ]]; then
    jj -R "$scratch_workspace" status >/dev/null 2>&1
    jj -R "$repo" describe -r 'scratch@' -m 'Refresh scratch description' >/dev/null 2>&1
    jj -R "$repo" restore --from 'root()' --into 'scratch@' README.md >/dev/null 2>&1
fi
# Save only explicit fixture values for the optional historical-operation tape preflight.
manual_exports=(source_repo bin manual_kind manual_real_jj manual_root manual_home
    XDG_CONFIG_HOME XDG_CACHE_HOME XDG_DATA_HOME JJ_CONFIG JJ_USER JJ_EMAIL JJ_TIMESTAMP
    JJ_OP_TIMESTAMP repo fixture fixture_root manual_initial_commit manual_initial_change
    manual_initial_operation manual_initial_graph manual_artifacts JK_DEMO_REPO JK_DEMO_BIN
    rebase_repo squash_repo restore_repo workspaces_repo scratch_workspace remote_repo
    manual_remote_graph container_repo)
: > "$manual_artifacts/fixture-env.sh"
for manual_variable in "${manual_exports[@]}"; do
    if declare -p "$manual_variable" >/dev/null 2>&1; then
        declare -p "$manual_variable" >> "$manual_artifacts/fixture-env.sh"
    fi
done
printf 'JK_%s\n' MANUAL_FIXTURE_READY
printf 'JK_%s\n' FIXTURE_READY
