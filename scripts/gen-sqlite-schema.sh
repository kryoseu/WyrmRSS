#!/usr/bin/env bash
# Regenerates database/src/schema_sqlite.rs.
#
# `diesel migration run` has no --schema-key, so it always writes its schema
# dump to the [print_schema] `file` in diesel.toml -- database/src/schema.rs,
# the *postgres* one -- even when the migrations it just ran were sqlite's.
# That clobbers the postgres schema with one generated from a sqlite
# database. This script runs the two-step regen documented in diesel.toml,
# then restores schema.rs from git so only schema_sqlite.rs actually changes.
#
# Requires the diesel CLI built with the sqlite feature:
#   cargo install diesel_cli --no-default-features --features "postgres sqlite-bundled"
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

db="$(mktemp -u "${TMPDIR:-/tmp}/wyrm-gen-sqlite-schema-XXXXXX.db")"
cleanup() {
  rm -f "$db" "$db-wal" "$db-shm"
}
trap cleanup EXIT

diesel migration run \
  --database-url "$db" \
  --migration-dir migrations_sqlite

diesel print-schema \
  --schema-key sqlite \
  --database-url "$db"

# `migration run` above overwrote the postgres schema as a side effect;
# only database/src/schema_sqlite.rs was meant to change.
git checkout -- database/src/schema.rs

echo "Regenerated database/src/schema_sqlite.rs. Review the diff before committing:"
git --no-pager diff --stat -- database/src/schema_sqlite.rs
