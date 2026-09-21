#!/usr/bin/env bash
# Build an isolated repository with sibling revisions, multiple files, and operation history.
set -euo pipefail
source "$(dirname "$0")/betamax-fixture-env.sh"
cd "$(dirname "$0")/.."
mkdir -p target/dogfood-artifacts/betamax
fixture=$(mktemp -d "$PWD/target/shared-selector-demo.XXXXXX")
jj --config 'user.name="Alex Developer"' --config 'user.email="alex@example.com"' \
  git init "$fixture" >&2
jj -R "$fixture" config set --repo user.name 'Alex Developer'
jj -R "$fixture" config set --repo user.email 'alex@example.com'
jj -R "$fixture" config set --repo signing.behavior drop

mkdir -p "$fixture/src"
printf '# Forecast service\n' > "$fixture/README.md"
jj -R "$fixture" describe -m 'Document the forecast service' >&2
jj -R "$fixture" new -m 'Add the forecast endpoint' >&2
printf 'export function forecast(city) { return { city, temperature: 21 }; }\n' \
  > "$fixture/src/forecast.js"
jj -R "$fixture" status >&2
base_change=$(jj -R "$fixture" log --no-graph -r @ -T 'change_id.short()')

jj -R "$fixture" new "$base_change" -m 'Add forecast caching' >&2
printf 'export const forecasts = new Map();\n' > "$fixture/src/cache.js"
jj -R "$fixture" status >&2

jj -R "$fixture" new "$base_change" -m 'Configure request timeouts' >&2
printf 'export const requestTimeoutMs = 3000;\n' > "$fixture/src/timeout.js"
printf 'export const localizedMessage = "Ready";\n' \
  > "$fixture/src/界界界界界界界界界界界界界界界界界界界界界界界界界界.js"
printf '\nRequests time out after three seconds.\n' >> "$fixture/README.md"
jj -R "$fixture" status >&2

printf '%s\n' "$fixture"
