#!/usr/bin/env bash
set -euo pipefail

BSR_DEMO_MODE=true PORT=8080 cargo run -p core-api &
api_pid=$!
cleanup() { kill "$api_pid" 2>/dev/null || true; }
trap cleanup EXIT INT TERM

for _ in {1..80}; do
  if curl -fsS http://127.0.0.1:8080/health >/dev/null 2>&1; then break; fi
  sleep 0.25
done

if ! curl -fsS http://127.0.0.1:8080/health >/dev/null; then
  echo "BSR Runner API did not become ready on port 8080" >&2
  exit 1
fi

NEXT_PUBLIC_RUNNER_API_URL=http://127.0.0.1:8080 npm run dev -w @bsr-hub/runner
