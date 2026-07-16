# BSR Hub Image Cache Performance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve original marketplace photos, serve high-quality responsive derivatives, and add a safe root-scoped browser cache that improves repeat visits across BSR Hub, BSR Runner, and the GitHub Pages entry page.

**Architecture:** Keep source JPEG/PNG files immutable and generate bounded WebP variants with Sharp. Install one service worker at `/bsr-hub/sw.js`; cache hashed assets first, refresh images stale-while-revalidate, and keep navigations network-first. Register the worker from the two Next.js apps and static entry page without coupling it to marketplace state.

**Tech Stack:** Next.js 16 static export, React 19, TypeScript 5.8, Sharp 0.35, browser Service Worker and Cache Storage APIs, GitHub Pages

## Global Constraints

- Preserve every file under `apps/web/public/images/listings` and `apps/web/public/images/categories` unchanged.
- Keep the deployed routes `/bsr-hub/`, `/bsr-hub/hub/`, and `/bsr-hub/runner/` unchanged.
- Generated budgets are 90 KiB for `card-sm`, 180 KiB for `card-lg`, and 270 KiB for `detail`.
- Only the primary Hub hero image is eager, high priority, and preloaded.
- Cache only successful same-origin `GET` responses under the service-worker scope.
- Never cache authorization-bearing requests, API routes, payment routes, checkout routes, or cross-origin traffic.
- Do not change MongoDB, Rust API, marketplace domain, payment, authentication, or routing behavior in this plan.
- Do not add runtime CDN or Google Fonts dependencies.

---

### Task 1: Extend the Performance Contract First

**Files:**
- Modify: `scripts/check-performance-links.mjs`
- Test: `scripts/check-performance-links.mjs`

**Interfaces:**
- Consumes: repository source files and generated image files.
- Produces: one non-zero/zero process exit contract used by every later task.

- [ ] **Step 1: Add failing service-worker, registration, font, and preload assertions**

After the existing image checks, read the new files and assert exact safety markers:

```js
const serviceWorker = await readFile("deploy/pages/sw.js", "utf8").catch(() => "");
for (const marker of [
  'const CACHE_PREFIX = "bsr-static-"',
  'request.method !== "GET"',
  'request.headers.has("authorization")',
  'url.origin !== self.location.origin',
  'request.mode === "navigate"',
  'url.pathname.includes("/_next/static/")',
  'event.waitUntil',
]) {
  if (!serviceWorker.includes(marker)) failures.push(`Service worker is missing ${marker}`);
}

for (const file of [
  "apps/web/src/components/PerformanceBoot.tsx",
  "apps/runner/src/components/PerformanceBoot.tsx",
]) {
  const source = await readFile(file, "utf8").catch(() => "");
  for (const marker of ["serviceWorker", "register", "rootPath", "NODE_ENV"]) {
    if (!source.includes(marker)) failures.push(`${file} is missing ${marker}`);
  }
}

for (const [file, marker] of [
  ["apps/web/src/app/layout.tsx", "<PerformanceBoot/>"],
  ["apps/runner/src/app/layout.tsx", "<PerformanceBoot/>"],
  ["deploy/pages/index.html", "navigator.serviceWorker.register"],
  ["scripts/build-pages.sh", "dist-pages/sw.js"],
  ["apps/web/src/app/layout.tsx", "rel=\"preload\""],
  ["apps/web/src/app/layout.tsx", "ps5-slim.webp"],
]) {
  const source = await readFile(file, "utf8").catch(() => "");
  if (!source.includes(marker)) failures.push(`${file} is missing ${marker}`);
}

for (const file of [
  "deploy/pages/styles.css",
  "apps/web/src/app/globals.css",
  "apps/runner/src/app/globals.css",
]) {
  const source = await readFile(file, "utf8");
  if (source.includes("fonts.googleapis.com")) failures.push(`${file} still loads Google Fonts`);
}
```

Change the detail budget to the approved value:

```js
const limits = { "card-sm": 90 * 1024, "card-lg": 180 * 1024, detail: 270 * 1024 };
```

- [ ] **Step 2: Run the contract and verify RED**

Run:

```bash
node scripts/check-performance-links.mjs
```

Expected: FAIL because `deploy/pages/sw.js`, both `PerformanceBoot.tsx` files, registration markers, preload, and font removals do not exist yet.

- [ ] **Step 3: Commit the failing contract**

```bash
git add scripts/check-performance-links.mjs
git commit -m "test: define persistent cache performance contract"
```

---

### Task 2: Regenerate Higher-Quality Responsive Images

**Files:**
- Modify: `scripts/optimize-images.mjs`
- Regenerate: `apps/web/public/images/optimized/card-sm/*.webp`
- Regenerate: `apps/web/public/images/optimized/card-lg/*.webp`
- Regenerate: `apps/web/public/images/optimized/detail/*.webp`
- Test: `scripts/check-performance-links.mjs`

**Interfaces:**
- Consumes: immutable JPEG/PNG sources from listing and category directories.
- Produces: the existing `small`, `large`, and `detail` paths returned by `optimizedImagePaths(source)`.

- [ ] **Step 1: Record source-image hashes before generation**

Run:

```bash
find apps/web/public/images/listings apps/web/public/images/categories \
  -type f \( -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' \) \
  -exec shasum -a 256 {} \; | sort > /tmp/bsr-source-images.before
```

Expected: one hash per source file and no command error.

- [ ] **Step 2: Implement deterministic quality-preserving settings**

Replace the three Sharp writes with:

```js
await sharp(input)
  .rotate()
  .resize(480, 360, { fit: "cover", position: "attention" })
  .webp({ quality: 80, effort: 6, smartSubsample: true, preset: "photo" })
  .toFile(cardSmall);

await sharp(input)
  .rotate()
  .resize(960, 720, { fit: "cover", position: "attention" })
  .webp({ quality: 84, effort: 6, smartSubsample: true, preset: "photo" })
  .toFile(cardLarge);

await sharp(input)
  .rotate()
  .resize({ width: 1440, withoutEnlargement: true })
  .webp({ quality: 84, effort: 6, smartSubsample: true, preset: "photo" })
  .toFile(detail);
```

Add per-variant totals to the existing report:

```js
const variantBytes = { cardSmall: 0, cardLarge: 0, detail: 0 };
// After writing each source:
variantBytes.cardSmall += (await stat(cardSmall)).size;
variantBytes.cardLarge += (await stat(cardLarge)).size;
variantBytes.detail += (await stat(detail)).size;
// After the existing summary:
console.log(`card-sm total: ${(variantBytes.cardSmall / 1048576).toFixed(2)} MiB.`);
console.log(`card-lg total: ${(variantBytes.cardLarge / 1048576).toFixed(2)} MiB.`);
console.log(`detail total: ${(variantBytes.detail / 1048576).toFixed(2)} MiB.`);
```

- [ ] **Step 3: Regenerate derivatives**

Run:

```bash
npm run images:optimize
```

Expected: 19 sources and 57 WebP outputs, with separate variant totals.

- [ ] **Step 4: Prove source images were unchanged**

Run:

```bash
find apps/web/public/images/listings apps/web/public/images/categories \
  -type f \( -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' \) \
  -exec shasum -a 256 {} \; | sort > /tmp/bsr-source-images.after
cmp /tmp/bsr-source-images.before /tmp/bsr-source-images.after
```

Expected: exit 0 with no output.

- [ ] **Step 5: Run the image subset of the contract**

Run:

```bash
node scripts/check-performance-links.mjs 2>&1 | tee /tmp/bsr-performance-red.log
! rg "Missing apps/web/public/images/optimized|exceeds [0-9]+ bytes" /tmp/bsr-performance-red.log
```

Expected: the overall contract remains RED only for worker/registration/font/preload requirements; no image is missing or over budget.

- [ ] **Step 6: Commit the image pipeline and derivatives**

```bash
git add scripts/optimize-images.mjs apps/web/public/images/optimized
git commit -m "perf: preserve detail in responsive images"
```

---

### Task 3: Implement the Root-Scoped Service Worker

**Files:**
- Create: `deploy/pages/sw.js`
- Modify: `scripts/build-pages.sh`
- Test: `scripts/check-performance-links.mjs`

**Interfaces:**
- Consumes: scope assigned by `navigator.serviceWorker.register(..., { scope })`.
- Produces: cache-first hashed assets, stale-while-revalidate images, and network-first navigations.

- [ ] **Step 1: Create the worker with explicit exclusions**

Create `deploy/pages/sw.js`:

```js
const CACHE_PREFIX = "bsr-static-";
const CACHE_VERSION = "2026-07-16-v1";
const ASSET_CACHE = `${CACHE_PREFIX}assets-${CACHE_VERSION}`;
const PAGE_CACHE = `${CACHE_PREFIX}pages-${CACHE_VERSION}`;

const appRoot = new URL(self.registration.scope).pathname;
const shellUrls = [appRoot, `${appRoot}hub/`, `${appRoot}runner/`];

function isSensitive(request, url) {
  return request.headers.has("authorization") || [
    "/api/", "/auth/", "/checkout/", "/payment/", "/payments/",
  ].some((segment) => url.pathname.includes(segment));
}

async function cacheFirst(request) {
  const cache = await caches.open(ASSET_CACHE);
  const cached = await cache.match(request);
  if (cached) return cached;
  const response = await fetch(request);
  if (response.ok) await cache.put(request, response.clone());
  return response;
}

async function staleWhileRevalidate(event) {
  const cache = await caches.open(ASSET_CACHE);
  const cached = await cache.match(event.request);
  const network = fetch(event.request).then(async (response) => {
    if (response.ok) await cache.put(event.request, response.clone());
    return response;
  });
  if (cached) {
    event.waitUntil(network.catch(() => undefined));
    return cached;
  }
  return network;
}

async function networkFirst(request) {
  const cache = await caches.open(PAGE_CACHE);
  try {
    const response = await fetch(request);
    if (response.ok) await cache.put(request, response.clone());
    return response;
  } catch (error) {
    return (await cache.match(request, { ignoreSearch: true }))
      ?? (await cache.match(`${appRoot}hub/`))
      ?? Response.error();
  }
}

self.addEventListener("install", (event) => {
  event.waitUntil(caches.open(PAGE_CACHE).then((cache) => cache.addAll(shellUrls)));
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil((async () => {
    const names = await caches.keys();
    await Promise.all(names
      .filter((name) => name.startsWith(CACHE_PREFIX) && ![ASSET_CACHE, PAGE_CACHE].includes(name))
      .map((name) => caches.delete(name)));
    await self.clients.claim();
  })());
});

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET") return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin || !url.pathname.startsWith(appRoot)) return;
  if (isSensitive(request, url)) return;

  if (request.mode === "navigate") {
    event.respondWith(networkFirst(request));
    return;
  }
  if (url.pathname.includes("/_next/static/")) {
    event.respondWith(cacheFirst(request));
    return;
  }
  if (request.destination === "image" || url.pathname.includes("/brand/")) {
    event.respondWith(staleWhileRevalidate(event));
  }
});
```

- [ ] **Step 2: Require the worker in the Pages artifact**

Add to `scripts/build-pages.sh` after the existing route assertions:

```bash
test -f dist-pages/sw.js
```

- [ ] **Step 3: Run the contract and verify the worker assertions turn GREEN**

Run:

```bash
node scripts/check-performance-links.mjs 2>&1 | tee /tmp/bsr-performance-worker.log
! rg "Service worker is missing|scripts/build-pages.sh is missing dist-pages/sw.js" /tmp/bsr-performance-worker.log
```

Expected: no worker/build assertion appears; the overall command remains RED for registration, preload, and fonts.

- [ ] **Step 4: Commit the worker**

```bash
git add deploy/pages/sw.js scripts/build-pages.sh
git commit -m "perf: add safe persistent asset cache"
```

---

### Task 4: Register One Worker Across Entry, Hub, and Runner

**Files:**
- Create: `apps/web/src/components/PerformanceBoot.tsx`
- Create: `apps/runner/src/components/PerformanceBoot.tsx`
- Modify: `apps/web/src/app/layout.tsx`
- Modify: `apps/runner/src/app/layout.tsx`
- Modify: `deploy/pages/index.html`
- Test: `scripts/check-performance-links.mjs`

**Interfaces:**
- Consumes: `NEXT_PUBLIC_BASE_PATH` values `/bsr-hub/hub` and `/bsr-hub/runner`.
- Produces: registration of `${rootPath}/sw.js` with scope `${rootPath}/` after page load.

- [ ] **Step 1: Create the Hub registration component**

Create `apps/web/src/components/PerformanceBoot.tsx`:

```tsx
"use client";

import { useEffect } from "react";

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";
const rootPath = basePath.replace(/\/(?:hub|runner)\/?$/, "");

export function PerformanceBoot() {
  useEffect(() => {
    if (process.env.NODE_ENV !== "production" || !("serviceWorker" in navigator)) return;
    const register = () => {
      void navigator.serviceWorker.register(`${rootPath}/sw.js`, {
        scope: `${rootPath || ""}/`,
      }).catch(() => undefined);
    };
    window.addEventListener("load", register, { once: true });
    return () => window.removeEventListener("load", register);
  }, []);
  return null;
}
```

- [ ] **Step 2: Create the identical isolated Runner registration component**

Create `apps/runner/src/components/PerformanceBoot.tsx`:

```tsx
"use client";

import { useEffect } from "react";

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";
const rootPath = basePath.replace(/\/(?:hub|runner)\/?$/, "");

export function PerformanceBoot() {
  useEffect(() => {
    if (process.env.NODE_ENV !== "production" || !("serviceWorker" in navigator)) return;
    const register = () => {
      void navigator.serviceWorker.register(`${rootPath}/sw.js`, {
        scope: `${rootPath || ""}/`,
      }).catch(() => undefined);
    };
    window.addEventListener("load", register, { once: true });
    return () => window.removeEventListener("load", register);
  }, []);
  return null;
}
```

Keeping a local copy avoids coupling the two independently built Next.js applications.

- [ ] **Step 3: Install the components in both layouts**

In each layout, import the local component:

```tsx
import { PerformanceBoot } from "../components/PerformanceBoot";
```

Render it as the first body child:

```tsx
<body><PerformanceBoot/>{children}</body>
```

- [ ] **Step 4: Register from the static entry page**

Add before `</body>` in `deploy/pages/index.html`:

```html
<script>
  if ("serviceWorker" in navigator && location.protocol !== "file:") {
    addEventListener("load", () => {
      navigator.serviceWorker.register("./sw.js", { scope: "./" }).catch(() => undefined);
    }, { once: true });
  }
</script>
```

- [ ] **Step 5: Run contract, type checks, and component tests**

Run:

```bash
node scripts/check-performance-links.mjs 2>&1 | tee /tmp/bsr-performance-registration.log
! rg "PerformanceBoot.tsx is missing|layout.tsx is missing <PerformanceBoot/>|index.html is missing navigator.serviceWorker.register" /tmp/bsr-performance-registration.log
npm run typecheck -w @bsr-hub/web
npm run typecheck -w @bsr-hub/runner
npm run test -w @bsr-hub/web
npm run test -w @bsr-hub/runner
```

Expected: registration assertions absent; both type checks and test suites pass. The overall performance contract remains RED only for fonts/preload.

- [ ] **Step 6: Commit registration**

```bash
git add apps/web/src/components/PerformanceBoot.tsx apps/runner/src/components/PerformanceBoot.tsx \
  apps/web/src/app/layout.tsx apps/runner/src/app/layout.tsx deploy/pages/index.html
git commit -m "perf: register shared pages cache"
```

---

### Task 5: Remove Runtime Fonts and Preload the LCP Image

**Files:**
- Modify: `deploy/pages/styles.css`
- Modify: `apps/runner/src/app/globals.css`
- Modify: `apps/web/src/app/layout.tsx`
- Test: `scripts/check-performance-links.mjs`

**Interfaces:**
- Consumes: `basePath` already defined in the Hub layout.
- Produces: one `<link rel="preload">` for the 480-pixel WebP hero asset.

- [ ] **Step 1: Remove both Google Fonts imports**

Delete the `@import url('https://fonts.googleapis.com/...');` first line from:

```text
deploy/pages/styles.css
apps/runner/src/app/globals.css
```

Retain their existing fallback font-family declarations so the browser uses local system fonts immediately.

- [ ] **Step 2: Add the Hub hero preload**

Change the Hub root layout to include an explicit head:

```tsx
export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="en">
    <head>
      <link
        rel="preload"
        as="image"
        type="image/webp"
        href={`${basePath}/images/optimized/card-sm/ps5-slim.webp`}
        fetchPriority="high"
      />
    </head>
    <body><PerformanceBoot/>{children}</body>
  </html>;
}
```

- [ ] **Step 3: Run the complete performance contract and verify GREEN**

Run:

```bash
node scripts/check-performance-links.mjs
```

Expected: exit 0 and `Performance and links contract passed`.

- [ ] **Step 4: Build Hub and Runner production exports**

Run:

```bash
NEXT_PUBLIC_STATIC_DEMO=true NEXT_PUBLIC_BASE_PATH=/bsr-hub/hub npm run build -w @bsr-hub/web
NEXT_PUBLIC_STATIC_DEMO=true NEXT_PUBLIC_BASE_PATH=/bsr-hub/runner npm run build -w @bsr-hub/runner
```

Expected: both static exports complete with exit 0.

- [ ] **Step 5: Verify generated HTML has one preload and no Google Fonts**

Run:

```bash
test "$(rg -o 'rel="preload"' apps/web/out/index.html | wc -l | tr -d ' ')" = "1"
! rg "fonts.googleapis.com" apps/web/out apps/runner/out deploy/pages
```

Expected: exit 0.

- [ ] **Step 6: Commit fonts and preload**

```bash
git add deploy/pages/styles.css apps/runner/src/app/globals.css apps/web/src/app/layout.tsx
git commit -m "perf: remove font latency and preload hero"
```

---

### Task 6: Build and Browser-Verify the Combined Pages Artifact

**Files:**
- Verify: `dist-pages/index.html`
- Verify: `dist-pages/hub/index.html`
- Verify: `dist-pages/runner/index.html`
- Verify: `dist-pages/sw.js`
- Modify: `docs/qa/github-pages-evidence.md`

**Interfaces:**
- Consumes: all outputs from Tasks 1 through 5.
- Produces: a deployable `dist-pages` artifact and recorded browser evidence.

- [ ] **Step 1: Run all static checks**

Run:

```bash
npm run check:web
npm run runner:check
npm run pages:build
node scripts/check-performance-links.mjs
```

Expected: all commands exit 0.

- [ ] **Step 2: Verify combined artifact contents**

Run:

```bash
test -f dist-pages/index.html
test -f dist-pages/hub/index.html
test -f dist-pages/hub/orders/index.html
test -f dist-pages/hub/create/index.html
test -f dist-pages/runner/index.html
test -f dist-pages/sw.js
! rg "fonts.googleapis.com" dist-pages
```

Expected: exit 0.

- [ ] **Step 3: Serve the artifact under the production path**

Run from the repository parent directory:

```bash
mkdir -p /tmp/bsr-pages-host/bsr-hub
cp -R dist-pages/. /tmp/bsr-pages-host/bsr-hub/
python3 -m http.server 4173 --directory /tmp/bsr-pages-host
```

Expected: `http://127.0.0.1:4173/bsr-hub/` serves the entry page.

- [ ] **Step 4: Verify in a browser**

Open `http://127.0.0.1:4173/bsr-hub/hub/` and verify:

```text
- The hero and listing images render without distortion.
- Only the first hero image is eager/high priority.
- navigator.serviceWorker.controller becomes non-null after one reload.
- Cache Storage contains current bsr-static-assets and bsr-static-pages caches.
- Reloading serves hashed Next.js assets and previously seen images from the worker.
- Hub, orders, create, help, and Runner links still open.
- No console error is produced by registration or fetch handling.
```

- [ ] **Step 5: Record concise QA evidence**

Append the date, browser URL, worker scope, cache names, route results, and test commands to `docs/qa/github-pages-evidence.md`.

- [ ] **Step 6: Run final source and secret checks**

Run:

```bash
git diff --exit-code -- apps/web/public/images/listings apps/web/public/images/categories
if git grep -nE 'mongodb(\+srv)?://[^[:space:]]+:[^[:space:]]+@' -- ':!*.example' ':!docs/**'; then exit 1; fi
git status --short
```

Expected: source-image diff is empty; no MongoDB credential match; status contains only intentional performance/QA changes and unrelated pre-existing untracked paths.

- [ ] **Step 7: Commit QA evidence if changed**

```bash
git add docs/qa/github-pages-evidence.md
git commit -m "test: verify cached pages experience"
```

---

## Post-Performance Handoff

After Task 6 passes, resume MongoDB in the existing isolated worktree:

```text
/Users/lucasfeng/Documents/babson/.worktrees/mongodb-core-persistence
```

Continue from Task 2 of `docs/superpowers/plans/2026-07-16-mongodb-core-persistence.md`. Do not merge partial MongoDB configuration into the performance branch before the performance verification is complete.
