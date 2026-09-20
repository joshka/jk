#!/usr/bin/env bash
# Create one disposable history and print only its absolute repository path.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/betamax-fixture-env.sh
mkdir -p target/dogfood-artifacts/fixtures
fixture=$(mktemp -d "$PWD/target/dogfood-artifacts/fixtures/${1:-inspection}.XXXXXX")
jj git init --colocate "$fixture" >&2
jj -R "$fixture" config set --repo signing.behavior drop
case "${1:-inspection}" in
    inspection)
        mkdir -p "$fixture/src"
        printf '# Terminal reports\n\nReadable reports for everyday project work.\n' > "$fixture/README.md"
        for index in $(seq 1 40); do
            printf 'pub const REPORT_COLUMN_%s: usize = %s;\n' "$index" "$index"
        done > "$fixture/src/report.rs"
        jj -R "$fixture" describe -m 'Start terminal reports' >&2
        jj -R "$fixture" new -m 'Add readable diff output' >&2
        python3 - "$fixture/src/report.rs" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
lines = path.read_text().splitlines()
lines[3] = 'pub const DIFF_CONTEXT: usize = 3; // Keep diff context readable.'
lines[24] = 'pub const DIFF_WIDTH: usize = 100; // Align diff columns.'
path.write_text('\n'.join(lines) + '\n')
PY
        printf '\nShow the diff beside the selected report.\n' >> "$fixture/README.md"
        jj -R "$fixture" new -m 'Show selected change details' >&2
        printf '\nUse the diff view to inspect each change.\n' >> "$fixture/README.md"
        printf '\npub const SELECTED_REPORT: &str = "diff details";\n' >> "$fixture/src/report.rs"
        ;;
    simple|action)
        printf 'one\n' > "$fixture/file.txt"
        jj -R "$fixture" describe -m 'Initial workspace' >&2
        jj -R "$fixture" new -m 'Second change' >&2
        printf 'two\n' >> "$fixture/file.txt"
        if [[ $1 == action ]]; then
            jj -R "$fixture" commit -m 'Second change with content' >&2
        fi
        ;;
    stability)
        jj -R "$fixture" describe -m 'A description with enough words to wrap across the narrow editor and finish here' >&2
        ;;
    squash)
        printf 'destination contents\n' > "$fixture/destination.txt"
        jj -R "$fixture" describe -m 'Destination change' >&2
        jj -R "$fixture" new -m 'Source change' >&2
        printf 'source contents\n' > "$fixture/source.txt"
        ;;
    *) printf 'Unknown fixture kind: %s\n' "$1" >&2; exit 2 ;;
esac
jj -R "$fixture" status >&2
printf '%s\n' "$fixture"
