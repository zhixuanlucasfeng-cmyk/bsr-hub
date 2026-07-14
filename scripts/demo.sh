#!/usr/bin/env bash
set -euo pipefail

BSR_DEMO_MODE=true PORT=8080 cargo run -p core-api &
api_pid=$!
cleanup() { kill "$api_pid" 2>/dev/null || true; }
trap cleanup EXIT INT TERM

for _ in {1..60}; do
  if curl -fsS http://localhost:8080/health >/dev/null 2>&1; then break; fi
  sleep 0.25
done

NEXT_PUBLIC_API_URL=http://localhost:8080 npm run dev -w @bsr-hub/web
