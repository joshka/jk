#!/usr/bin/env bash
# Build this checkout, then retain each recording and its text checkpoints in a unique run directory.
set -euo pipefail
cd "$(dirname "$0")/.."
export JK_SOURCE_REPO=$PWD
export BETAMAX_WORKING_DIRECTORY=$PWD
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-3}
if [[ ${JK_BETAMAX_SKIP_BUILD:-0} != 1 ]]; then
    cargo build --locked -p jk
fi
if [[ -z ${JK_BIN:-} ]]; then
    target_dir=$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
    export JK_BIN="$target_dir/debug/jk"
fi
[[ $JK_BIN = /* ]] || export JK_BIN="$PWD/$JK_BIN"
[[ -x $JK_BIN ]] || { printf 'Missing jk executable: %s\n' "$JK_BIN" >&2; exit 1; }
media_dir=
if [[ ${1:-} == --readme-media ]]; then
    shift
    media_repo=${JK_SCREENSHOTS_REPO:-}
    search_parent=$PWD
    while [[ -z $media_repo && $search_parent != / ]]; do
        search_parent=$(dirname "$search_parent")
        if [[ -d $search_parent/jk-screenshots/assets ]]; then
            media_repo="$search_parent/jk-screenshots"
        fi
    done
    [[ -n $media_repo && -d $media_repo ]] || {
        printf 'README media requires the sibling jk-screenshots repo, or JK_SCREENSHOTS_REPO.\n' >&2
        exit 1
    }
    mkdir -p "$media_repo/assets"
    media_dir=$(cd "$media_repo/assets" && pwd)
fi
tape=${1:?usage: run-betamax.sh [--readme-media] TAPE}
mkdir -p target/dogfood-artifacts/betamax/runs
run_dir=$(mktemp -d "$PWD/target/dogfood-artifacts/betamax/runs/$(basename "$tape" .tape).XXXXXX")
python3 - "$tape" "$run_dir" "$media_dir" <<'PY'
from pathlib import Path
import shlex
import sys
source, run = Path(sys.argv[1]), Path(sys.argv[2])
media = Path(sys.argv[3]) if sys.argv[3] else None
lines = []
for line in source.read_text().splitlines():
    if line.startswith(('Output ', 'Screenshot ', 'State ')):
        command, path = shlex.split(line)
        destination = run
        if media and command in ('Output', 'Screenshot') and 'readme-media/' in path:
            destination = media
        line = f'{command} "{destination / Path(path).name}"'
    lines.append(line)
(run / 'capture.tape').write_text('\n'.join(lines) + '\n')
PY
printf 'Betamax review artifacts: %s\n' "$run_dir"
if [[ -n $media_dir ]]; then
    printf 'README media assets: %s\n' "$media_dir"
fi
"${BETAMAX:-betamax}" run "$run_dir/capture.tape"
