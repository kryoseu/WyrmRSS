#!/usr/bin/env bash
# Regenerates database/src/schema_sqlite.rs.
#
# Runs against diesel-sqlite.toml (a separate config from the root
# diesel.toml -- see the note there) so `migration run` writes and patches
# database/src/schema_sqlite.rs directly; nothing here touches the postgres
# schema or its config.
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
  --config-file diesel-sqlite.toml

echo "Regenerated database/src/schema_sqlite.rs. Review the diff before committing:"
git --no-pager diff --stat -- database/src/schema_sqlite.rs
