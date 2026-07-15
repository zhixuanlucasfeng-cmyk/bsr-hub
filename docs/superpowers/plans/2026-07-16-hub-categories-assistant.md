# BSR Hub Categories and Hybrid Assistant Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real marketplace photography, category-based discovery, and a truthful hybrid shopping assistant to the public BSR Hub demo.

**Architecture:** Keep marketplace state in `Home`, move category and assistant rules into pure tested library modules, and render the new UI through three focused React components. Store licensed photographs as local static assets so GitHub Pages remains self-contained; persist only the one-session assistant greeting flag in `sessionStorage`.

**Tech Stack:** Next.js 16 static export, React 19, TypeScript 5.8, Node test runner, CSS, GitHub Pages.

## Global Constraints

- Scope is limited to `apps/web`; Rust pricing and order-state APIs do not change.
- The seven categories are Gaming, Computers, Cameras, Tools, Studios, Small Production Spaces, and Second-hand Goods.
- Every demo listing and category card uses a locally stored real photograph with useful alternative text.
- Photo provenance is recorded in `docs/PHOTO-SOURCES.md` and only reusable educational-demo assets are accepted.
- The assistant proactively opens at most once per browser session and remains dismissible.
- Automated messages are labeled `BSR Assistant`; the UI never invents a human response or fake online status.
- Without a configured public support destination, worker handoff offers a truthful copy-message fallback.
- The chat never requests payment details, identity documents, precise addresses, or other sensitive information.
- Desktop, mobile, keyboard, and screen-reader behavior must remain usable.

---

## File Structure

- Modify `apps/web/src/lib/types.ts`: add listing image fields.
- Modify `apps/web/src/lib/static-demo.ts`: attach local image metadata to all twelve listings.
- Create `apps/web/src/lib/categories.ts`: category definitions and combined filtering rules.
- Create `apps/web/src/lib/categories.test.ts`: pure category/filter tests.
- Create `apps/web/src/lib/assistant.ts`: quick-action IDs, responses, and handoff helpers.
- Create `apps/web/src/lib/assistant.test.ts`: pure assistant behavior tests.
- Create `apps/web/src/components/CategoryBrowser.tsx`: accessible category selection UI.
- Create `apps/web/src/components/ListingImage.tsx`: optimized photo rendering and fallback.
- Create `apps/web/src/components/ShopAssistant.tsx`: proactive greeting, quick actions, conversation, and handoff UI.
- Modify `apps/web/src/app/page.tsx`: integrate category state, photography, and assistant callbacks.
- Modify `apps/web/src/app/globals.css`: responsive category, image, and chat styling.
- Create `apps/web/public/images/categories/*`: seven category photographs.
- Create `apps/web/public/images/listings/*`: twelve listing photographs.
- Create `docs/PHOTO-SOURCES.md`: source URL, creator/source site, reuse note, and local filename.

### Task 1: Listing Photography Contract and Category Rules

**Files:**
- Modify: `apps/web/src/lib/types.ts`
- Modify: `apps/web/src/lib/static-demo.ts`
- Create: `apps/web/src/lib/categories.ts`
- Create: `apps/web/src/lib/categories.test.ts`

**Interfaces:**
- Produces: `Listing.imageSrc: string`, `Listing.imageAlt: string`.
- Produces: `CategoryId`, `CategoryDefinition`, `categories`, `matchesCategory(listing, categoryId)`, and `filterMarketplaceListings(listings, filters)`.
- Consumes: existing `Listing` and listing type values.

- [ ] **Step 1: Write failing category tests**

Create cases proving that Gaming matches both PS5 listings, Studios matches studio/maker/printing workspaces, Small Production Spaces matches maker/printing workspace entries, Second-hand Goods matches `sale`, and combined query/type/category filters all apply.

```ts
import assert from "node:assert/strict";
import test from "node:test";
import { categories, filterMarketplaceListings, matchesCategory } from "./categories.ts";
import { createHubStaticDemo } from "./static-demo.ts";

test("defines the seven approved categories", () => {
  assert.deepEqual(categories.map((item) => item.id), [
    "gaming", "computers", "cameras", "tools", "studios", "production", "second-hand",
  ]);
});

test("combines query, type, and category filters", () => {
  const listings = createHubStaticDemo().listings();
  const results = filterMarketplaceListings(listings, {
    query: "studio", listingType: "workspace", categoryId: "studios",
  });
  assert.deepEqual(results.map((item) => item.id), ["photo-studio"]);
  assert.equal(matchesCategory(listings.find((item) => item.id === "monitor-sale")!, "second-hand"), true);
});
```

- [ ] **Step 2: Run the tests and verify failure**

Run: `npm run test -w @bsr-hub/web`

Expected: FAIL because `categories.ts` and listing image fields do not exist.

- [ ] **Step 3: Add image fields and category/filter implementation**

Use the exact filter contract:

```ts
export type CategoryId = "all" | "gaming" | "computers" | "cameras" | "tools" | "studios" | "production" | "second-hand";
export type MarketplaceFilters = { query: string; listingType: string; categoryId: CategoryId };

export function filterMarketplaceListings(listings: Listing[], filters: MarketplaceFilters): Listing[] {
  const needle = filters.query.trim().toLowerCase();
  return listings.filter((listing) =>
    (filters.listingType === "all" || listing.listingType === filters.listingType) &&
    matchesCategory(listing, filters.categoryId) &&
    (!needle || `${listing.title} ${listing.category} ${listing.city}`.toLowerCase().includes(needle))
  );
}
```

Add a local `/images/listings/<id>.jpg` path and descriptive alt text to each catalog record.

- [ ] **Step 4: Run tests and type checking**

Run: `npm run test -w @bsr-hub/web && npm run typecheck -w @bsr-hub/web`

Expected: all tests PASS and TypeScript reports no errors.

- [ ] **Step 5: Commit the domain changes**

```bash
git add apps/web/src/lib/types.ts apps/web/src/lib/static-demo.ts apps/web/src/lib/categories.ts apps/web/src/lib/categories.test.ts
git commit -m "feat: add marketplace category rules"
```

### Task 2: Real Photo Assets and Provenance

**Files:**
- Create: `apps/web/public/images/categories/*.jpg`
- Create: `apps/web/public/images/listings/*.jpg`
- Create: `docs/PHOTO-SOURCES.md`

**Interfaces:**
- Consumes: image paths defined by Task 1 and category `imageSrc` values.
- Produces: optimized local JPEG files that exist at every referenced public path.

- [ ] **Step 1: Download reusable real photographs**

Select subject-accurate photographs from a source with clear public reuse terms. Save seven category files and twelve listing files using lowercase kebab-case names. Do not use screenshots, synthetic renders, logos, watermarked images, or hotlinks.

- [ ] **Step 2: Normalize image files**

Use an image optimizer to convert each asset to JPEG, remove metadata, limit the long edge to 1600 pixels, and target a quality near 82. Keep each file below 500 KB where possible without obvious artifacts.

- [ ] **Step 3: Record provenance**

For every file, add a row to `docs/PHOTO-SOURCES.md` containing local filename, subject/listing, original page URL, creator or source site, access date `2026-07-16`, and reuse/license note.

- [ ] **Step 4: Verify every referenced asset exists**

Run a Node test or shell check that extracts `/images/...` paths from category and listing data and verifies each maps to an existing file under `apps/web/public`.

Expected: zero missing files and zero files above the agreed size ceiling unless documented.

- [ ] **Step 5: Commit the photo set**

```bash
git add apps/web/public/images docs/PHOTO-SOURCES.md
git commit -m "feat: add real marketplace photography"
```

### Task 3: Category Browser and Listing Image Components

**Files:**
- Create: `apps/web/src/components/CategoryBrowser.tsx`
- Create: `apps/web/src/components/ListingImage.tsx`
- Modify: `apps/web/src/app/page.tsx`
- Modify: `apps/web/src/app/globals.css`

**Interfaces:**
- Consumes: `categories`, `CategoryId`, and `filterMarketplaceListings` from Task 1.
- Produces: `CategoryBrowser({ selected, onSelect })` and `ListingImage({ listing, priority?, className? })`.

- [ ] **Step 1: Add category selection to `Home`**

Create `categoryId` state with initial value `"all"`. Replace the inline `useMemo` filter with `filterMarketplaceListings(listings, { query, listingType: type, categoryId })`.

- [ ] **Step 2: Implement `CategoryBrowser`**

Render an `All categories` control and seven photograph cards as buttons. Apply `aria-pressed` to the selected card. On selection, call `onSelect(id)` and scroll `#market` into view from `Home`.

- [ ] **Step 3: Implement `ListingImage`**

Use Next.js `Image` with `fill`, explicit responsive `sizes`, `object-fit: cover`, and listing alt text. Track load error locally and render a branded `Photo unavailable` fallback without changing dimensions.

- [ ] **Step 4: Replace emoji artwork in cards and modal**

Use `ListingImage` for the listing grid and detail modal while retaining the listing-type badge and all price/condition/location information.

- [ ] **Step 5: Add responsive styling**

Create large-screen category grid, small-screen horizontal snap row, image overlays, hover/focus treatment, stable aspect ratios, and high-contrast labels. Preserve existing 1000 px and 650 px responsive behavior.

- [ ] **Step 6: Run web checks**

Run: `npm run check:web`

Expected: typecheck, library tests, and production build all PASS.

- [ ] **Step 7: Commit the category UI**

```bash
git add apps/web/src/components apps/web/src/app/page.tsx apps/web/src/app/globals.css
git commit -m "feat: add visual category discovery"
```

### Task 4: Hybrid Assistant Rules and Interface

**Files:**
- Create: `apps/web/src/lib/assistant.ts`
- Create: `apps/web/src/lib/assistant.test.ts`
- Create: `apps/web/src/components/ShopAssistant.tsx`
- Modify: `apps/web/src/app/page.tsx`
- Modify: `apps/web/src/app/globals.css`

**Interfaces:**
- Produces: `AssistantActionId`, `assistantActions`, `responseForAction(id)`, and `buildWorkerHandoff(message, supportDestination)`.
- Produces: `ShopAssistant({ onExplore, onList, onWorkspace, onDelivery })`.
- Consumes: navigation/filter callbacks supplied by `Home`.

- [ ] **Step 1: Write failing assistant tests**

Cover all six quick-action IDs, safe scripted responses, URL-encoded handoff content when a public support destination exists, and the copy-only result when it does not.

```ts
test("does not claim delivery without support configuration", () => {
  assert.deepEqual(buildWorkerHandoff("Please help", ""), {
    mode: "copy",
    message: "Please help",
  });
});
```

- [ ] **Step 2: Run tests and verify failure**

Run: `npm run test -w @bsr-hub/web`

Expected: FAIL because `assistant.ts` does not exist.

- [ ] **Step 3: Implement pure assistant rules**

Return concise answers for renting, listing, workspaces, delivery, payment/deposits, and worker handoff. State that payment is held until completion and warn users not to send sensitive information in chat.

- [ ] **Step 4: Implement the chat interface**

Build a launcher, dialog heading, close button, message list with `aria-live="polite"`, quick-action buttons, free-text handoff field, and copy/handoff action. Label every automated message `BSR Assistant`.

- [ ] **Step 5: Add one-session proactive greeting**

On client mount, read `sessionStorage.getItem("bsr-assistant-greeted")`. If absent, open after a short delay and set the flag. Wrap storage access in `try/catch`; closing cancels pending opening and never reopens during the same session.

- [ ] **Step 6: Connect assistant actions to marketplace state**

Renting opens the market and selects all rental products, listing opens the existing create view, workspaces filters to `workspace`, and delivery scrolls to marketplace results while explaining the fulfillment filter available on each listing.

- [ ] **Step 7: Add desktop/mobile/accessibility styling**

Render a fixed desktop panel and small-screen bottom sheet, visible focus rings, sufficient contrast, reduced-motion-safe transitions, and a launcher that does not cover footer or primary mobile controls.

- [ ] **Step 8: Run assistant and full web checks**

Run: `npm run test -w @bsr-hub/web && npm run check:web`

Expected: all assistant tests and existing web checks PASS.

- [ ] **Step 9: Commit the assistant**

```bash
git add apps/web/src/lib/assistant.ts apps/web/src/lib/assistant.test.ts apps/web/src/components/ShopAssistant.tsx apps/web/src/app/page.tsx apps/web/src/app/globals.css
git commit -m "feat: add hybrid marketplace assistant"
```

### Task 5: Static Pages Build, Browser Verification, and Publication

**Files:**
- Modify only if verification exposes a defect: files from Tasks 1–4.
- Generated and ignored: `dist-pages/`.

**Interfaces:**
- Consumes: completed web app.
- Produces: verified `/hub/` static export and updated GitHub Pages deployment.

- [ ] **Step 1: Run the complete repository quality gate**

Run: `npm run pages:check`

Expected: Hub typecheck/tests/build, Runner typecheck/tests/build, and static Pages assembly all PASS.

- [ ] **Step 2: Serve the static export locally**

Run the repository's existing static-preview command or a local HTTP server for `dist-pages`, then open `/hub/`.

- [ ] **Step 3: Verify desktop behavior**

Confirm all category and listing photos load, category/search/type filters combine correctly, modal photography remains sharp, assistant opens once, closes, reopens manually, and every quick action works.

- [ ] **Step 4: Verify mobile and keyboard behavior**

At a narrow mobile viewport, confirm category snapping, bottom-sheet chat, no covered controls, readable text, logical Tab order, Escape/close behavior, visible focus, and no horizontal page overflow.

- [ ] **Step 5: Verify worker fallback truthfulness**

With no support destination configured, confirm the UI says delivery is not connected, never shows a fake worker or `sent` message, and successfully copies the prepared text.

- [ ] **Step 6: Run final diff and regression review**

Run: `git diff --check`, `git status --short`, and inspect the complete change set for accidental generated files, secrets, private addresses, or unrelated edits.

- [ ] **Step 7: Commit any verification fixes**

```bash
git add apps/web docs/PHOTO-SOURCES.md
git commit -m "fix: polish marketplace photos and assistant"
```

- [ ] **Step 8: Push the current Pages branch**

Push the existing `codex/bsr-runner` branch only after every local quality gate passes. Verify the GitHub Pages workflow succeeds and load the public `/hub/` URL to confirm the deployed assets and assistant.

