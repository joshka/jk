#!/usr/bin/env bash
set -euo pipefail

state_dir=${JK_FAKE_JJ_STATE:?JK_FAKE_JJ_STATE must name the fixture state directory}
real_jj=${JK_REAL_JJ:?JK_REAL_JJ must name the real jj binary}
mkdir -p "$state_dir"

while ! mkdir "$state_dir/count.lock" 2>/dev/null; do
    :
done
count=0
if [[ -f "$state_dir/count" ]]; then
    read -r count < "$state_dir/count"
fi
count=$((count + 1))
printf '%s\n' "$count" > "$state_dir/count"
rmdir "$state_dir/count.lock"

case "$count" in
    3)
        end=$((SECONDS + 5))
        while ((SECONDS < end)); do
            :
        done
        ;;
    4)
        end=$((SECONDS + 1))
        while ((SECONDS < end)); do
            :
        done
        ;;
    6)
        printf '%s\n' "fixture refresh failed" >&2
        exit 1
        ;;
esac

exec "$real_jj" "$@"
