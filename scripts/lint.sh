#!/usr/bin/env bash
# Runs fmt + clippy + machete for both the root workspace and desktop/src-tauri,
# plus eslint for the frontend.
#
# desktop/src-tauri is its own separate cargo workspace (see the comment in
# its Cargo.toml), so `cargo fmt`/`cargo clippy` run from the repo root never
# touch it -- it needs its own invocation, from within that directory.
# cargo-machete is the exception: it walks the filesystem for Cargo.toml
# files rather than following cargo's workspace graph, so one invocation
# from the root already covers desktop/src-tauri too.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

snapshot_rs_files() {
  find "$repo_root" -name '*.rs' -not -path '*/target/*' -exec sha256sum {} + | sort
}

before="$(snapshot_rs_files)"

echo "== root workspace =="
cd "$repo_root"
echo "-- checking fmt for root workspace --"
cargo +nightly fmt
echo "-- checking clippy for root workspace --"
cargo clippy -- -D warnings
echo "-- running cargo-machete for both root workspace and desktop/src-tauri --"
cargo machete

echo
echo "== desktop/src-tauri =="
cd "$repo_root/desktop/src-tauri"
echo "-- checking fmt for desktop/src-tauri --"
cargo +nightly fmt
echo "-- checking clippy for desktop/src-tauri --"
cargo clippy --all-targets -- -D warnings

echo
echo "== frontend =="
cd "$repo_root/web"
echo "-- checking eslint for web --"
pnpm lint

after="$(snapshot_rs_files)"

echo
if [ "$before" = "$after" ]; then
  echo "No files were reformatted."
else
  echo "Reformatted files:"
  diff <(echo "$before") <(echo "$after") | grep '^>' | awk '{print $3}' | sed "s|^$repo_root/||"
fi
