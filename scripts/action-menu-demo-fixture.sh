#!/usr/bin/env bash
# Build an isolated, disposable project history; print its path for the demo launcher.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p target/dogfood-artifacts/betamax
fixture=$(mktemp -d "$PWD/target/forecast-demo.XXXXXX")
jj --config 'user.name="Alex Developer"' --config 'user.email="alex@example.com"' git init "$fixture" >&2
jj -R "$fixture" config set --repo user.name 'Alex Developer'
jj -R "$fixture" config set --repo user.email 'alex@example.com'
jj -R "$fixture" config set --repo signing.behavior drop
mkdir -p "$fixture/src"
printf '# Forecast service\n\nA small service for local weather forecasts.\n' > "$fixture/README.md"
jj -R "$fixture" describe -m 'Document the forecast service' >&2
jj -R "$fixture" new -m 'Add the forecast endpoint' >&2
printf 'export function forecast(city) {\n  return { city, temperature: 21, condition: "sunny" };\n}\n' > "$fixture/src/forecast.js"
jj -R "$fixture" new -m 'Configure request timeouts' >&2
printf '{\n  "requestTimeoutMs": 3000,\n  "cacheTtlSeconds": 60\n}\n' > "$fixture/config.json"
jj -R "$fixture" new -m 'Experiment with forecast caching' >&2
printf 'const forecasts = new Map();\n\nexport function remember(city, forecast) {\n  forecasts.set(city, forecast);\n}\n' > "$fixture/src/cache.js"
jj -R "$fixture" status >&2
printf '%s\n' "$fixture"
