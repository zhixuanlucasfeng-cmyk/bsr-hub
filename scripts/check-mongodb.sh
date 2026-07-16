#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root_dir"

cleanup() {
  docker compose -f compose.mongodb.yml down >/dev/null 2>&1 || true
}
trap 'status=$?; cleanup; exit "$status"' EXIT

command -v docker >/dev/null
docker compose version >/dev/null

npm run mongo:up

set -a
# shellcheck disable=SC1091
source .env.mongodb.example
set +a
export MONGODB_TEST_URI="$MONGODB_URI"

npm run mongo:bootstrap
npm run mongo:bootstrap

cargo test -p core-api \
  --test mongo_pricing_integration \
  --test mongo_reservation_integration \
  --test mongo_order_lifecycle_integration \
  -- --ignored

npm run check
git diff --check

echo "MongoDB persistence check passed"
