#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

mkdir -p target/vhs

cargo build --quiet

for tape in docs/vhs/*.tape; do
  vhs "$tape"
done
