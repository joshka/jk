#!/usr/bin/env bash
# Extend the isolated forecast demo with a mixed change and two descendants.
set -euo pipefail
cd "$(dirname "$0")/.."
fixture=$(bash scripts/action-menu-demo-fixture.sh)
printf '{\n  "requestTimeoutMs": 3000,\n  "cacheTtlSeconds": 120\n}\n' > "$fixture/config.json"
rm "$fixture/README.md"
for ((i=0; i<${1:-0}; i++)); do
    printf 'export const cacheLimit = %s;\n' "$i" > "$fixture/src/cache-policy-$i.js"
done
jj -R "$fixture" describe -m 'Tune forecast caching' >&2
jj -R "$fixture" new -m 'Add cache metrics' >&2
printf 'export const cacheHits = 0;\n' > "$fixture/src/metrics.js"
jj -R "$fixture" new -m 'Document cache metrics' >&2
printf '# Cache metrics\n\nCount hits before changing the policy.\n' > "$fixture/metrics.md"
jj -R "$fixture" status >&2
printf '%s\n' "$fixture"
