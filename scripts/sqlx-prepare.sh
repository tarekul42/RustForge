#!/usr/bin/env bash
set -euo pipefail

# Regenerate the sqlx offline cache after migration changes.
# Requires a running PostgreSQL instance with the skill_workshop database.
#
# Usage: bash scripts/sqlx-prepare.sh

DATABASE_URL="${DATABASE_URL:-postgres://workshop:workshop_secret@localhost:5432/skill_workshop}"

echo "Running migrations..."
cargo sqlx migrate run --database-url "$DATABASE_URL"

echo "Regenerating offline cache..."
cargo sqlx prepare --workspace --database-url "$DATABASE_URL"

echo "Done. Verify with: SQLX_OFFLINE=true cargo check --workspace"
