# BSR Hub Performance, Links, and Direct Routes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce BSR Hub image latency, preserve useful photo proportions, make footer destinations functional, and export working `/orders/` and `/create/` routes.

**Architecture:** A build-time Sharp script creates committed WebP variants; a focused `OptimizedImage` component selects them with responsive `srcset`. Footer destinations live in typed configuration and point to a new static help page or authoritative external URLs. Direct entry pages use a small client redirect component to enter the existing guarded flows without duplicating marketplace state.

**Tech Stack:** Next.js 16 static export, React 19, TypeScript, Tailwind CSS 4, Node test runner, Sharp, GitHub Pages.

## Global Constraints

- Primary marketplace and detail images must not use a 1:1 square layout.
- No paid image CDN or runtime image service.
- External Google Fonts requests must be removed.
- GitHub Pages base paths must remain `/bsr-hub/hub` and `/bsr-hub/runner`.
- Existing Rust pricing, payment, and order-state rules must not change.
- Every new route and link must work after a direct browser refresh.

---

### Task 1: Deterministic Image Preprocessing

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Create: `scripts/optimize-images.mjs`
- Create: `scripts/check-performance-links.mjs`
- Create: `apps/web/public/images/optimized/card-sm/*.webp`
- Create: `apps/web/public/images/optimized/card-lg/*.webp`
- Create: `apps/web/public/images/optimized/detail/*.webp`

**Interfaces:**
- Consumes: source JPEG files under `apps/web/public/images/listings` and `apps/web/public/images/categories`.
- Produces: `npm run images:optimize` and three WebP variants for every source image.

- [ ] **Step 1: Write the failing asset contract**

Create `scripts/check-performance-links.mjs` with checks that every source basename has `card-sm`, `card-lg`, and `detail` WebP files, that `card-sm` is no larger than 90 KiB, `card-lg` no larger than 180 KiB, and `detail` no larger than 260 KiB.

```js
const variants = ["card-sm", "card-lg", "detail"];
const limits = { "card-sm": 90 * 1024, "card-lg": 180 * 1024, detail: 260 * 1024 };
for (const source of sources) {
  for (const variant of variants) {
    const output = join(optimizedRoot, variant, `${basename(source, extname(source))}.webp`);
    const info = await stat(output).catch(() => null);
    if (!info) failures.push(`Missing ${output}`);
    else if (info.size > limits[variant]) failures.push(`${output} exceeds ${limits[variant]} bytes`);
  }
}
```

- [ ] **Step 2: Run the contract and verify it fails**

Run: `node scripts/check-performance-links.mjs`  
Expected: FAIL with missing optimized WebP paths.

- [ ] **Step 3: Add Sharp and the preprocessing command**

Add root dev dependency `sharp` and script:

```json
"images:optimize": "node scripts/optimize-images.mjs"
```

The preprocessing script must use:

```js
await sharp(input).rotate().resize(480, 360, { fit: "cover", position: "attention" }).webp({ quality: 76 }).toFile(cardSmall);
await sharp(input).rotate().resize(960, 720, { fit: "cover", position: "attention" }).webp({ quality: 80 }).toFile(cardLarge);
await sharp(input).rotate().resize({ width: 1440, withoutEnlargement: true }).webp({ quality: 82 }).toFile(detail);
```

- [ ] **Step 4: Generate images and verify the contract**

Run: `npm run images:optimize && node scripts/check-performance-links.mjs`  
Expected: PASS with a summary containing source count, generated variant count, and total optimized bytes.

- [ ] **Step 5: Commit**

```bash
git add package.json package-lock.json scripts/optimize-images.mjs scripts/check-performance-links.mjs apps/web/public/images/optimized
git commit -m "perf: preprocess marketplace images"
```

---

### Task 2: Responsive Image Delivery and Font Request Removal

**Files:**
- Create: `apps/web/src/lib/image-assets.ts`
- Create: `apps/web/src/lib/image-assets.test.ts`
- Create: `apps/web/src/components/OptimizedImage.tsx`
- Modify: `apps/web/src/components/HeroSection.tsx`
- Modify: `apps/web/src/components/MarketplaceListingCard.tsx`
- Modify: `apps/web/src/components/ListingGallery.tsx`
- Modify: `apps/web/src/components/BusinessShowcase.tsx`
- Modify: `apps/web/src/components/FeaturedListings.tsx`
- Modify: `apps/web/src/app/globals.css`

**Interfaces:**
- Produces: `optimizedImagePaths(source: string)` returning `{ small, large, detail }`.
- Produces: `OptimizedImage` props `{ source, alt, mode, eager?, className? }`.

- [ ] **Step 1: Write failing image-path tests**

```ts
test("maps a source image to every optimized variant", () => {
  assert.deepEqual(optimizedImagePaths("/images/listings/ps5-slim.jpg"), {
    small: "/images/optimized/card-sm/ps5-slim.webp",
    large: "/images/optimized/card-lg/ps5-slim.webp",
    detail: "/images/optimized/detail/ps5-slim.webp",
  });
});
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run: `node --experimental-strip-types --test apps/web/src/lib/image-assets.test.ts`  
Expected: FAIL because `optimizedImagePaths` is missing.

- [ ] **Step 3: Implement the path helper and optimized component**

`OptimizedImage` uses `<picture>` with small and large WebP candidates for card mode, the detail candidate for gallery mode, explicit dimensions, `decoding="async"`, `fetchPriority` only when eager, and the original JPEG as fallback.

```tsx
<picture>
  <source type="image/webp" srcSet={`${small} 480w, ${large} 960w`} sizes={sizes}/>
  <img src={fallback} width={960} height={720} loading={eager ? "eager" : "lazy"} decoding="async" fetchPriority={eager ? "high" : "auto"} alt={alt}/>
</picture>
```

- [ ] **Step 4: Replace runtime image usage**

- Hero: use optimized card images; only the main console card gets eager/high priority.
- Listing cards: only index `0` receives `eager`; all remaining cards are lazy.
- Business showcase: optimized card images, all lazy.
- Detail gallery: optimized detail image with a natural `aspect-[16/11]`, `object-contain`, and lazy inactive slides.
- Remove the `fonts.googleapis.com` import and replace Manrope/DM Sans with a system stack in the theme and body declarations.

- [ ] **Step 5: Run tests and source contract**

Run: `npm run test:web && node scripts/check-performance-links.mjs`  
Expected: all Hub tests pass and the performance contract passes.

- [ ] **Step 6: Commit**

```bash
git add apps/web/src/lib/image-assets.ts apps/web/src/lib/image-assets.test.ts apps/web/src/components apps/web/src/app/globals.css scripts/check-performance-links.mjs
git commit -m "perf: serve responsive marketplace imagery"
```

---

### Task 3: Working Footer Links and Help Page

**Files:**
- Create: `apps/web/src/lib/footer-links.ts`
- Create: `apps/web/src/lib/footer-links.test.ts`
- Modify: `apps/web/src/components/SiteFooter.tsx`
- Create: `apps/web/src/app/help/page.tsx`

**Interfaces:**
- Produces: `footerGroups`, an array of `{ title, links: { label, href, external? }[] }`.

- [ ] **Step 1: Write failing footer-link tests**

Assert every item has an `href`, internal links start with `/` or `#`, external links start with `https://`, and the Marketplace filters use `type=rental`, `type=workspace`, and `type=sale`.

- [ ] **Step 2: Run the focused test and verify it fails**

Run: `node --experimental-strip-types --test apps/web/src/lib/footer-links.test.ts`  
Expected: FAIL because `footerGroups` is missing.

- [ ] **Step 3: Implement typed footer destinations**

Use the approved internal routes, official United Nations and Babson URLs, GitHub Issues for contact, and the repository URL for social/project updates.

- [ ] **Step 4: Render real anchors and create the help page**

`SiteFooter` maps `footerGroups` to `Link` or `<a>` elements. External links use `target="_blank" rel="noreferrer"`. The help page contains sections with IDs `protected-payment`, `terms`, and `privacy`, plus visible back-navigation to the marketplace.

- [ ] **Step 5: Run footer tests and static contract**

Run: `npm run test:web && node scripts/check-performance-links.mjs`  
Expected: all tests pass; the script finds `href`, `/help/`, official Babson URL, GitHub Issues, and no footer-only `<p>` navigation items.

- [ ] **Step 6: Commit**

```bash
git add apps/web/src/lib/footer-links.ts apps/web/src/lib/footer-links.test.ts apps/web/src/components/SiteFooter.tsx apps/web/src/app/help/page.tsx scripts/check-performance-links.mjs
git commit -m "feat: add working footer destinations"
```

---

### Task 4: Filterable Footer URLs and Direct Orders/Create Routes

**Files:**
- Create: `apps/web/src/lib/entry-route.ts`
- Create: `apps/web/src/lib/entry-route.test.ts`
- Create: `apps/web/src/components/IntentRedirect.tsx`
- Create: `apps/web/src/app/orders/page.tsx`
- Create: `apps/web/src/app/create/page.tsx`
- Modify: `apps/web/src/app/page.tsx`
- Modify: `scripts/build-pages.sh`
- Modify: `scripts/check-performance-links.mjs`

**Interfaces:**
- Produces: `readMarketplaceEntry(search: string)` returning `{ intent, listingType }`.
- Produces: `<IntentRedirect intent="orders" | "create" />`.

- [ ] **Step 1: Write failing entry-route tests**

```ts
assert.deepEqual(readMarketplaceEntry("?type=rental"), { intent:null, listingType:"rental" });
assert.deepEqual(readMarketplaceEntry("?intent=orders"), { intent:"orders", listingType:"all" });
assert.deepEqual(readMarketplaceEntry("?type=unknown"), { intent:null, listingType:"all" });
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run: `node --experimental-strip-types --test apps/web/src/lib/entry-route.test.ts`  
Expected: FAIL because `readMarketplaceEntry` is missing.

- [ ] **Step 3: Implement query parsing and home-page hydration**

The home page reads `readMarketplaceEntry(window.location.search)`, applies a valid listing filter, and preserves the existing protected-session behavior for `orders` and `create` intents.

- [ ] **Step 4: Add direct route pages**

`IntentRedirect` calls `window.location.replace(`${basePath}/?intent=${intent}`)` after mount and renders a visible fallback `<a>` to the same destination. The static orders and create pages pass the corresponding intent.

- [ ] **Step 5: Enforce exported artifact paths**

Add to `scripts/build-pages.sh`:

```bash
test -f dist-pages/hub/orders/index.html
test -f dist-pages/hub/create/index.html
test -f dist-pages/hub/help/index.html
```

Add equivalent source markers and file existence checks to `scripts/check-performance-links.mjs`.

- [ ] **Step 6: Run tests and Pages build**

Run: `npm run test:web && npm run pages:build && node scripts/check-performance-links.mjs`  
Expected: all tests pass and all three new route artifacts exist.

- [ ] **Step 7: Commit**

```bash
git add apps/web/src/lib/entry-route.ts apps/web/src/lib/entry-route.test.ts apps/web/src/components/IntentRedirect.tsx apps/web/src/app/orders/page.tsx apps/web/src/app/create/page.tsx apps/web/src/app/page.tsx scripts/build-pages.sh scripts/check-performance-links.mjs
git commit -m "fix: export direct marketplace routes"
```

---

### Task 5: Full Verification, Browser QA, and Publication

**Files:**
- Modify only if QA reveals a reproducible defect.

**Interfaces:**
- Consumes: complete optimized and routed static export.
- Produces: verified GitHub Pages deployment.

- [ ] **Step 1: Run the complete local quality gates**

Run: `npm run check && node scripts/check-brand-assets.mjs && node scripts/check-performance-links.mjs && npm run pages:check`  
Expected: Rust, Hub, Runner, both production builds, image contract, and Pages artifact checks all pass.

- [ ] **Step 2: Compare asset size and request behavior**

Record original image total, optimized image total, and ensure only the hero/first listing are eager in page source. Confirm no `fonts.googleapis.com` request remains.

- [ ] **Step 3: Browser QA**

Verify at 1,440 × 900, 820 × 1,180, and 390 × 844:

- no horizontal overflow;
- images remain 4:3 on cards and natural ratio on details;
- footer links are clickable and keyboard-focusable;
- filtered Marketplace links select the right listing type;
- `/orders/` and `/create/` enter guarded flows after direct navigation;
- help anchors display the correct sections;
- console has no errors.

- [ ] **Step 4: Merge and publish**

Fast-forward the feature branch into `main`, rerun `npm run pages:check`, push `main`, and wait for the Pages workflow conclusion `success`.

- [ ] **Step 5: Verify public endpoints**

Require HTTP 200 and expected content at:

- `/bsr-hub/hub/`
- `/bsr-hub/hub/orders/`
- `/bsr-hub/hub/create/`
- `/bsr-hub/hub/help/`
- `/bsr-hub/hub/listings/ps5-slim/`

- [ ] **Step 6: Clean the feature worktree**

After successful deployment and public verification, remove the owned `.worktrees` entry and delete the merged feature branch.
