#!/usr/bin/env bash
# Deterministic fault injection for disposable manual scenarios only.
set -euo pipefail
real_jj=${JK_REAL_JJ:?JK_REAL_JJ must name the real jj executable}
if [[ ${JK_MANUAL_FAULT:-} == abandon-probe ]]; then
    for argument in "$@"; do
        if [[ $argument == 'self.empty()' ]]; then
            printf 'Manual fixture: emptiness probe unavailable\n' >&2
            exit 1
        fi
    done
fi
exec "$real_jj" "$@"
