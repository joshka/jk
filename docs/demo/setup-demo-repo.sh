#!/usr/bin/env sh

set -eu

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <static-log|operation-recovery>" >&2
  exit 1
fi

scenario="$1"
repo_root="target/demo-repos"

jj_demo() {
  jj --no-pager --quiet \
    --config user.name="Demo User" \
    --config user.email="demo@localhost" \
    --config signing.behavior=drop \
    --config signing.backend=none \
    "$@"
}

case "$scenario" in
  static-log)
    repo_dir="$repo_root/static-log"
    ;;
  operation-recovery)
    repo_dir="$repo_root/operation-recovery"
    ;;
  *)
    echo "unknown demo scenario: $scenario" >&2
    exit 1
    ;;
esac

rm -rf "$repo_dir"
mkdir -p "$repo_root"

jj --no-pager --quiet git init "$repo_dir"
cd "$repo_dir"

case "$scenario" in
  static-log)
    cat > journal.txt <<'EOF'
base
first follow-up
EOF
    jj_demo commit -m "demo: base graph"

    printf '%s\n' "second follow-up" >> journal.txt
    jj_demo commit -m "demo: extend the log"

    printf '%s\n' "third follow-up" >> journal.txt
    jj_demo commit -m "demo: finish the graph"
    ;;
  operation-recovery)
    cat > notes.txt <<'EOF'
base
EOF
    jj_demo commit -m "demo: base history"

    printf '%s\n' "recovery anchor" >> notes.txt
    jj_demo commit -m "demo: stage recovery"

    jj_demo describe -m "demo: ready to undo"
    ;;
esac
