#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root_dir"

rm -rf dist-pages apps/web/out apps/runner/out

NEXT_PUBLIC_STATIC_DEMO=true NEXT_PUBLIC_BASE_PATH=/bsr-hub/hub npm run build -w @bsr-hub/web
NEXT_PUBLIC_STATIC_DEMO=true NEXT_PUBLIC_BASE_PATH=/bsr-hub/runner npm run build -w @bsr-hub/runner

mkdir -p dist-pages/hub dist-pages/runner
cp -R deploy/pages/. dist-pages/
cp -R apps/web/out/. dist-pages/hub/
cp -R apps/runner/out/. dist-pages/runner/

test -f dist-pages/index.html
test -f dist-pages/hub/index.html
test -f dist-pages/hub/orders/index.html
test -f dist-pages/hub/create/index.html
test -f dist-pages/hub/help/index.html
test -f dist-pages/runner/index.html

echo "GitHub Pages artifact ready: dist-pages"
