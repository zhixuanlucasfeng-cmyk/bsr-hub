#!/usr/bin/env bash
set -euo pipefail

compose=(docker compose -f compose.mongodb.yml)

for _ in $(seq 1 30); do
  if "${compose[@]}" exec -T mongodb mongosh --quiet \
    --username bsr --password bsr-local-only --authenticationDatabase admin \
    --eval 'db.adminCommand({ ping: 1 }).ok' | grep -q 1; then
    break
  fi
  sleep 2
done

"${compose[@]}" exec -T mongodb mongosh --quiet \
  --username bsr --password bsr-local-only --authenticationDatabase admin \
  --eval 'try { rs.status().ok } catch (_) { rs.initiate({_id:"rs0",members:[{_id:0,host:"localhost:27017"}]}); }'

for _ in $(seq 1 30); do
  if "${compose[@]}" exec -T mongodb mongosh --quiet \
    --username bsr --password bsr-local-only --authenticationDatabase admin \
    --eval 'db.hello().isWritablePrimary' | grep -q true; then
    exit 0
  fi
  sleep 2
done

echo "MongoDB replica set did not become primary" >&2
exit 1
