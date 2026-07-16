# BSR Hub Marketplace Luxury Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild the BSR Hub home and listing-detail experience with a responsive Tailwind design system, demo sign-in gate, icon-only left support launcher, and refined shared-economy marketplace UI without changing business rules.

**Architecture:** Presentational sections become focused React components driven by the existing listing, category, quote, order, and assistant adapters. A shared client demo instance preserves booking continuity during client navigation, while statically generated `/listings/[id]` routes keep GitHub Pages export compatibility. Tailwind owns the redesigned UI; the remaining legacy CSS only supports untouched order/create surfaces until they are expressed with component utility classes.

**Tech Stack:** Next.js 16, React 19, TypeScript 5.8, Tailwind CSS, Node test runner, static GitHub Pages export, existing Rust Axum API

## Global Constraints

- Preserve product rentals, creative-space bookings, second-hand sales, `Use more. Own less.`, current Rust contracts, pricing rules, and order transitions.
- Brand purple is `#7C3AED`, accent lime is `#CCFF00`, and page background is `#FAFAFA`.
- New containers use 16-pixel radii, paired ambient/contact shadows, 200ms interactions, and two-pixel rounded linear icons.
- Sign-in is explicitly fictional demo session selection and never collects or claims to verify real credentials.
- Workspaces only display fulfillment values present in their listing and never invent delivery.
- All asset and navigation paths remain valid under `NEXT_PUBLIC_BASE_PATH=/bsr-hub/hub`.
- Desktop, tablet, and mobile layouts must have no horizontal document overflow.

---

### Task 1: Tailwind foundation, marketplace view models, and demo-session rules

**Files:**
- Modify: `apps/web/package.json`
- Modify: `package-lock.json`
- Create: `apps/web/postcss.config.mjs`
- Modify: `apps/web/src/app/globals.css`
- Create: `apps/web/src/lib/marketplace.ts`
- Create: `apps/web/src/lib/marketplace.test.ts`
- Modify: `apps/web/src/lib/types.ts`
- Modify: `apps/web/src/lib/static-demo.ts`

**Interfaces:**
- Produces: `sellerFor(ownerId)`, `relatedListings(listing, listings, limit)`, `fulfillmentLabel`, `listingPriceLabel`, `demoSessionKey`, and `hubStaticDemo`.
- Consumes: existing `Listing`, `Fulfillment`, `categories`, and static catalog.

- [ ] **Step 1: Write failing marketplace unit tests**

Test that related listings rank same-category/type entries first, never return the source listing, stop at the requested limit, workspace fulfillment exposes only `on_site`, and sale/rental/workspace price labels use the correct unit.

- [ ] **Step 2: Run the focused test and confirm red state**

Run: `npm run test -w @bsr-hub/web -- src/lib/marketplace.test.ts`

Expected: FAIL because `marketplace.ts` does not exist.

- [ ] **Step 3: Install and configure Tailwind**

Run: `npm install -w @bsr-hub/web -D tailwindcss @tailwindcss/postcss`

Create `postcss.config.mjs` with the `@tailwindcss/postcss` plugin and add `@import "tailwindcss";` plus `@theme` tokens for the approved colors, radii, and shadows at the beginning of `globals.css`.

- [ ] **Step 4: Implement marketplace helpers and shared demo adapter**

Export a read-only `demoCatalog`, a `getDemoListing(id)` lookup, and one `hubStaticDemo = createHubStaticDemo()` instance. Extend `Listing` with optional `sellerName`, `sellerAvatar`, `distanceMiles`, `rating`, `reviewCount`, `specifications`, `usageNotes`, and `imageGallery`, with UI helpers supplying deterministic owner-based fallbacks.

- [ ] **Step 5: Run tests and typecheck**

Run:

```bash
npm run test -w @bsr-hub/web
npm run typecheck -w @bsr-hub/web
```

Expected: all marketplace and existing tests PASS.

- [ ] **Step 6: Commit the foundation**

```bash
git add apps/web/package.json package-lock.json apps/web/postcss.config.mjs apps/web/src/app/globals.css apps/web/src/lib
git commit -m "feat: add marketplace design foundation"
```

### Task 2: Global chrome, login gate, support launcher, and redesigned home

**Files:**
- Create: `apps/web/src/components/LinearIcon.tsx`
- Create: `apps/web/src/components/GlobalNav.tsx`
- Create: `apps/web/src/components/LoginModal.tsx`
- Create: `apps/web/src/components/HeroSection.tsx`
- Rewrite: `apps/web/src/components/CategoryBrowser.tsx`
- Create: `apps/web/src/components/MarketplaceListingCard.tsx`
- Create: `apps/web/src/components/FeaturedListings.tsx`
- Create: `apps/web/src/components/BusinessShowcase.tsx`
- Create: `apps/web/src/components/SiteFooter.tsx`
- Modify: `apps/web/src/components/ShopAssistant.tsx`
- Modify: `apps/web/src/app/page.tsx`
- Modify: `apps/web/src/app/globals.css`
- Modify: `scripts/check-brand-assets.mjs`

**Interfaces:**
- `GlobalNav` receives current persona, order count, active view, and navigation/list/avatar handlers.
- `LoginModal` receives `open`, `personas`, `onSelect`, `onGuest`, and `onClose` and writes no storage itself.
- `MarketplaceListingCard` receives `listing` and `href` and renders no pricing logic.
- `HeroSection`, `CategoryBrowser`, and `BusinessShowcase` report filter/navigation intents through callbacks.

- [ ] **Step 1: Extend the source contract check for redesigned components**

Require each component file, verify `ShopAssistant` uses an icon-only accessible launcher, verify page source imports the new home sections, and verify global CSS imports Tailwind.

- [ ] **Step 2: Run the contract check and confirm red state**

Run: `node scripts/check-brand-assets.mjs`

Expected: FAIL listing missing redesigned component contracts.

- [ ] **Step 3: Build global navigation and demo login gate**

Implement scroll-aware blur/shadow navigation. Read `bsr-demo-session` from `sessionStorage` after mount, open the modal on first visit, and route protected actions through one `requireSession(callback)` helper. Selecting a persona persists only its ID; guest browsing stores `guest` for the session.

- [ ] **Step 4: Build refined home sections**

Use Tailwind utilities for the textured hero and three photo cards, six-item icon category rail, four-column featured grid, and three real-photo business cards. Listing cards use base-path-safe images, seller metadata, distance, verified badge, and a hover reservation overlay linked to `/listings/{id}/`.

- [ ] **Step 5: Convert support to a left icon launcher**

Remove automatic panel opening. Render one 48-pixel circular button fixed at the lower-left with only a linear support icon; click opens the existing scripted conversation and worker-handoff panel. Preserve Escape close, accessible labels, safety copy, and callback behavior.

- [ ] **Step 6: Preserve orders/create/quote functionality inside the new shell**

Keep existing view state, API/static adapter selection, category/search filtering, quote calculation, order creation, and state actions. Replace only the marketplace composition and shared navigation/footer. Protected actions include orders, listing, reservation, and purchase.

- [ ] **Step 7: Run focused checks and build**

```bash
node scripts/check-brand-assets.mjs
npm run test -w @bsr-hub/web
npm run typecheck -w @bsr-hub/web
npm run build -w @bsr-hub/web
```

Expected: all commands PASS and the home route exports.

- [ ] **Step 8: Commit the home redesign**

```bash
git add apps/web/src apps/web/public scripts/check-brand-assets.mjs
git commit -m "feat: redesign BSR Hub marketplace home"
```

### Task 3: Static listing detail routes and adaptive booking experience

**Files:**
- Create: `apps/web/src/app/listings/[id]/page.tsx`
- Create: `apps/web/src/components/ListingDetailView.tsx`
- Create: `apps/web/src/components/ListingGallery.tsx`
- Create: `apps/web/src/components/SellerProfile.tsx`
- Create: `apps/web/src/components/BookingCard.tsx`
- Create: `apps/web/src/components/RelatedListings.tsx`
- Modify: `apps/web/src/app/globals.css`
- Modify: `scripts/check-brand-assets.mjs`

**Interfaces:**
- Static route exports `generateStaticParams()` from `demoCatalog` and renders `ListingDetailView` for `getDemoListing(id)`.
- `BookingCard` receives listing, persona/session callbacks, and quote/order adapters; it derives available fulfillment only from `listing.fulfillment`.
- `ListingGallery` receives one or more image descriptors and only displays thumbnails for multiple images.
- `RelatedListings` receives source listing and catalog, using `relatedListings` from Task 1.

- [ ] **Step 1: Extend contract tests for the detail route**

Require the route, gallery, booking, seller, and related-listings files; verify the route contains `generateStaticParams`, detail uses `BrandLogo`, and booking source references listing fulfillment.

- [ ] **Step 2: Run contract check and confirm red state**

Run: `node scripts/check-brand-assets.mjs`

Expected: FAIL because detail components do not exist.

- [ ] **Step 3: Build the static detail route and media gallery**

Generate all demo listing IDs at build time. Render a base-path-safe gallery with previous/next controls, thumbnail buttons only for multiple media items, pointer/touch swipe threshold of 45 pixels, and 300ms opacity transition.

- [ ] **Step 4: Build seller, description, specification, and protection sections**

Render seller fallback metadata, Verified badge, rating, expandable three-line description, structured specifications, usage notes, address-privacy messaging, protected payment, refundable deposit, and dispute-support statements.

- [ ] **Step 5: Build adaptive sticky transaction card**

Rental/workspace cards expose units and allowed fulfillment; sale cards display one-time price and quantity. Use the existing static/Rust quote endpoint contract, show the fee/deposit breakdown, and require a demo session before final Reserve/Buy/Book. Desktop card is sticky; mobile uses a bottom action bar with safe content padding.

- [ ] **Step 6: Add rule-ranked related listings**

Render up to six cards in a horizontal overflow row using the helper tested in Task 1.

- [ ] **Step 7: Build every static detail path**

Run:

```bash
npm run test -w @bsr-hub/web
npm run typecheck -w @bsr-hub/web
NEXT_PUBLIC_STATIC_DEMO=true NEXT_PUBLIC_BASE_PATH=/bsr-hub/hub npm run build -w @bsr-hub/web
```

Expected: every `/listings/{id}/` path appears in build output and checks PASS.

- [ ] **Step 8: Commit detail experience**

```bash
git add apps/web/src scripts/check-brand-assets.mjs
git commit -m "feat: add shared listing detail experience"
```

### Task 4: Full regression, responsive browser QA, and publishing

**Files:**
- Modify only if verification exposes a confirmed defect in files from Tasks 1–3.

**Interfaces:**
- Consumes: combined Hub/Runner/Rust repository and GitHub Pages workflow.
- Produces: verified `main` deployment at the existing public URLs.

- [ ] **Step 1: Run complete repository verification**

```bash
npm run check
node scripts/check-brand-assets.mjs
npm run pages:check
```

Expected: Rust, Hub, Runner, contract, and static-export checks all PASS.

- [ ] **Step 2: Serve the `/bsr-hub/` artifact and verify desktop/tablet/mobile**

Inspect home and one rental, workspace, and sale detail at 1440×900, 820×1180, and 390×844. Assert logo/media natural widths are positive, document overflow is false, navigation changes surface after scroll, login gate appears, left support icon opens one panel, categories filter, detail links load, and workspace delivery is absent.

- [ ] **Step 3: Verify protected demo flows**

Select Maya in the login modal, open PS5 detail, request a delivery quote, create a demo reservation, and verify the success state. Open the photo studio and confirm only on-site fulfillment. Open a second-hand listing and confirm one-time purchase copy.

- [ ] **Step 4: Merge and publish**

Fast-forward the verified feature branch into `main`, rerun `npm run pages:check` on merged `main`, push `main`, and wait for the Pages workflow conclusion `success`.

- [ ] **Step 5: Verify public deployment**

Confirm HTTP 200 and redesigned source markers for:

- `https://zhixuanlucasfeng-cmyk.github.io/bsr-hub/`
- `https://zhixuanlucasfeng-cmyk.github.io/bsr-hub/hub/`
- `https://zhixuanlucasfeng-cmyk.github.io/bsr-hub/hub/listings/ps5-slim/`
- `https://zhixuanlucasfeng-cmyk.github.io/bsr-hub/runner/`

## Self-Review

- Spec coverage: navigation, login, left support icon, hero, categories, featured listings, business cards, footer, Tailwind tokens, responsive detail layout, gallery, seller, adaptive booking, related listings, accessibility, existing logic, static export, and publishing are mapped to Tasks 1–4.
- Placeholder scan: no placeholder markers or unspecified implementation steps remain.
- Type consistency: `Listing`, `hubStaticDemo`, route IDs, fulfillment arrays, and helper names are consistent across all tasks.
- Scope: Runner and Rust remain regression-tested but visually unchanged, matching the approved BSR Hub scope.
