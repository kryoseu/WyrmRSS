#!/usr/bin/env bash
# Bumps the release version in both places it lives: the root [workspace.package]
# version (inherited by every crate in the main workspace) and
# desktop/src-tauri/Cargo.toml, which carries its own hardcoded version because
# it's deliberately excluded from the root workspace (see the comment in
# Cargo.toml) and so inherits nothing from it.
#
# Usage: scripts/bump-version.sh <new-version>
#   e.g. scripts/bump-version.sh 0.26.0
#
# Takes a plain semver (no leading 'v', no prerelease suffix) -- the release
# workflow adds the 'v' and any '-rcN' suffix itself at tag time.
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <new-version>" >&2
  exit 1
fi

new_version="$1"
if ! [[ "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Version must be a plain semver like 0.26.0 (no 'v' prefix, no prerelease suffix)." >&2
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

for manifest in Cargo.toml desktop/src-tauri/Cargo.toml; do
  if ! grep -q '^version = "' "$manifest"; then
    echo "Could not find a 'version = \"...\"' line in $manifest" >&2
    exit 1
  fi
  sed -i.bak "s/^version = \"[^\"]*\"/version = \"${new_version}\"/" "$manifest"
  rm -f "$manifest.bak"
  echo "Bumped $manifest to ${new_version}"
done

echo
echo "Refreshing root Cargo.lock..."
cargo check --workspace --quiet

echo "Refreshing desktop/src-tauri/Cargo.lock..."
if ! (cd desktop/src-tauri && cargo check --quiet); then
  echo "warning: could not refresh desktop/src-tauri/Cargo.lock (missing platform build deps?)." >&2
  echo "Run 'cargo check' inside desktop/src-tauri yourself before committing." >&2
fi

echo
echo "Review the diff, then commit:"
git --no-pager diff --stat -- Cargo.toml Cargo.lock desktop/src-tauri/Cargo.toml desktop/src-tauri/Cargo.lock
