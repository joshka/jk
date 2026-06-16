#!/usr/bin/env bash
set -euo pipefail

# Package the already-built jk binary for GitHub Releases, cargo-binstall, and
# Homebrew tap formula updates.
#
# Usage:
#
#   scripts/package-release-archive.sh <target-triple> <version> [dist-dir]
#
# Run `just build-release <target-triple>` first. This script intentionally does
# not build anything: CI chooses the runner and Rust target, then this script
# only verifies and packages the artifact that was actually built.
target="${1:?target triple is required}"
version="${2:?version is required}"
dist_dir="${3:-target/dist}"

binary="target/${target}/release/jk"
# The current release matrix does not build Windows assets, but this keeps the
# archive contract correct if a Windows target is added later.
if [[ "$target" == *"-windows-"* ]]; then
  binary="${binary}.exe"
fi

if [[ ! -x "$binary" ]]; then
  echo "missing release binary: $binary" >&2
  exit 1
fi

work_dir="${dist_dir}/jk-${version}-${target}"
archive="${dist_dir}/jk-${version}-${target}.tgz"
checksum="${archive}.sha256"

rm -rf "$work_dir" "$archive" "$checksum"
mkdir -p "$work_dir" "$dist_dir"

# cargo-binstall is configured with bin-dir = "{ bin }{ binary-ext }", so the
# binary must live at the archive root rather than under a versioned directory.
cp "$binary" "$work_dir/$(basename "$binary")"
tar -C "$work_dir" -czf "$archive" "$(basename "$binary")"

# Ubuntu runners provide sha256sum; macOS runners provide shasum. Keep both
# paths so the same script works locally and across the release matrix.
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$archive" > "$checksum"
else
  shasum -a 256 "$archive" > "$checksum"
fi

echo "$archive"
