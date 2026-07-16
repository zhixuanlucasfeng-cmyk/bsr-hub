# BSR Logo Website Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the temporary Hub and Runner brand treatments with the approved BSR logo family in navigation, footers, and browser metadata, then publish the verified result through GitHub Pages.

**Architecture:** Each Next.js app owns a small `BrandLogo` component and its own public assets because Hub and Runner are exported independently. Asset URLs are prefixed with `NEXT_PUBLIC_BASE_PATH`, while page layouts expose the PNG icon through Next metadata. Existing navigation actions and footer content remain unchanged.

**Tech Stack:** Next.js 14, React 18, TypeScript, CSS, static export, GitHub Actions/Pages

## Global Constraints

- Hub navigation remains 76 pixels high; Runner topbar remains 76 pixels high on desktop.
- Navigation logos remain clickable home buttons with accessible labels.
- Rendered logo height is 42 pixels on desktop and 36 pixels below 650 pixels.
- Every static asset URL must include `NEXT_PUBLIC_BASE_PATH` when present.
- Do not introduce a new dependency or alter marketplace behavior.
- Preserve the old `codex/bsr-runner` deploy trigger and add `main` so future merged work is published.

---

### Task 1: Add reusable logo rendering and app-owned assets

**Files:**
- Create: `apps/web/src/components/BrandLogo.tsx`
- Create: `apps/runner/src/components/BrandLogo.tsx`
- Copy: `apps/web/public/brand/bsr-icon.svg` to `apps/runner/public/brand/bsr-icon.svg`
- Copy: `apps/web/public/brand/bsr-icon.png` to `apps/runner/public/brand/bsr-icon.png`
- Copy: `apps/web/public/brand/bsr-runner-logo.svg` to `apps/runner/public/brand/bsr-runner-logo.svg`
- Modify: `apps/web/src/app/layout.tsx`
- Modify: `apps/runner/src/app/layout.tsx`

**Interfaces:**
- Consumes: `process.env.NEXT_PUBLIC_BASE_PATH`, existing SVG/PNG files in `apps/web/public/brand/`.
- Produces: `BrandLogo({ variant, className })`, where `variant` is `"icon" | "horizontal"` and the rendered image has correct intrinsic dimensions and alt text.

- [ ] **Step 1: Add a source-level contract test**

Create `scripts/check-brand-assets.mjs` that verifies both component sources use `NEXT_PUBLIC_BASE_PATH`, both layouts mention `bsr-icon.png`, and every required Hub/Runner public asset exists and is non-empty.

- [ ] **Step 2: Run the contract test and verify it fails**

Run: `node scripts/check-brand-assets.mjs`

Expected: FAIL because the components, Runner asset copies, and metadata icons do not exist yet.

- [ ] **Step 3: Implement both logo components and metadata icons**

The Hub component maps `icon` to `bsr-icon.svg` with `BSR Hub` alt text and maps `horizontal` to `bsr-hub-logo.svg`. The Runner component maps `icon` to `bsr-icon.svg` with `BSR Runner` alt text and maps `horizontal` to `bsr-runner-logo.svg`. Both components render fixed `width` and `height` attributes and prefix `src` with `NEXT_PUBLIC_BASE_PATH`. Add `${basePath}/brand/bsr-icon.png` to each layout's `metadata.icons.icon`.

- [ ] **Step 4: Copy Runner-owned assets and run the contract test**

Run: `node scripts/check-brand-assets.mjs`

Expected: PASS with `Brand asset contract passed.`

- [ ] **Step 5: Commit the reusable asset layer**

Run:

```bash
git add apps/web/src/components/BrandLogo.tsx apps/runner/src/components/BrandLogo.tsx apps/web/src/app/layout.tsx apps/runner/src/app/layout.tsx apps/runner/public/brand scripts/check-brand-assets.mjs
git commit -m "feat: add reusable BSR brand assets"
```

### Task 2: Integrate logo components into Hub and Runner chrome

**Files:**
- Modify: `apps/web/src/app/page.tsx`
- Modify: `apps/web/src/app/globals.css`
- Modify: `apps/runner/src/components/RunnerNav.tsx`
- Modify: `apps/runner/src/app/page.tsx`
- Modify: `apps/runner/src/app/globals.css`

**Interfaces:**
- Consumes: `BrandLogo` from each app's component directory.
- Produces: clickable horizontal navigation logo, readable horizontal footer logo, responsive `.brand-logo` and `.footer-logo` styles.

- [ ] **Step 1: Extend the contract test for integration points**

Update `scripts/check-brand-assets.mjs` so it asserts Hub page, Runner nav, and Runner page import and render `BrandLogo`, and that both stylesheets contain `.brand-logo` plus `.footer-logo` rules.

- [ ] **Step 2: Run the contract test and verify it fails**

Run: `node scripts/check-brand-assets.mjs`

Expected: FAIL because the pages still render the old text blocks.

- [ ] **Step 3: Replace temporary navigation and footer branding**

Use `<BrandLogo variant="horizontal" className="brand-logo" />` inside both existing home buttons. Use `<BrandLogo variant="horizontal" className="footer-logo" />` in both footers. Keep `aria-label="BSR Hub home"` and `aria-label="BSR Runner home"` on navigation buttons, and leave all click handlers and footer copy intact.

- [ ] **Step 4: Add responsive and footer presentation styles**

Remove the old `.brand span` and `.brand-mark` visual blocks. Set navigation logo height to `42px`, width to `auto`, and reduce it to `36px` below `650px`. Give footer logos a white rounded presentation surface, maximum width, and preserved aspect ratio. Ensure buttons keep a visible focus outline and do not shrink.

- [ ] **Step 5: Run source contract and app checks**

Run:

```bash
node scripts/check-brand-assets.mjs
npm run check
```

Expected: both PASS.

- [ ] **Step 6: Commit the visible integration**

Run:

```bash
git add apps/web/src/app/page.tsx apps/web/src/app/globals.css apps/runner/src/components/RunnerNav.tsx apps/runner/src/app/page.tsx apps/runner/src/app/globals.css scripts/check-brand-assets.mjs
git commit -m "feat: integrate BSR logos across both sites"
```

### Task 3: Verify the static deployment and publish it

**Files:**
- Modify: `.github/workflows/pages.yml`

**Interfaces:**
- Consumes: both static Next.js exports and existing GitHub Pages workflow.
- Produces: a Pages workflow that deploys pushes to `main` as well as the legacy deployment branch.

- [ ] **Step 1: Add `main` to the Pages push branches**

Set the workflow branch list to `[main, codex/bsr-runner]` without changing build or deployment jobs.

- [ ] **Step 2: Run the complete static Pages quality gate**

Run: `npm run pages:check`

Expected: Hub and Runner checks/builds pass and `dist-pages/hub/index.html`, `dist-pages/runner/index.html`, and `dist-pages/index.html` exist.

- [ ] **Step 3: Inspect the built site at deployed-path shape**

Serve `dist-pages` beneath `/bsr-hub/`, then inspect `/bsr-hub/hub/` and `/bsr-hub/runner/` at desktop and mobile widths. Verify both navigation/footer logos have a positive natural width, there is no horizontal overflow, and both pages expose a `rel="icon"` link whose URL loads successfully.

- [ ] **Step 4: Commit the deployment trigger**

Run:

```bash
git add .github/workflows/pages.yml
git commit -m "ci: deploy main to GitHub Pages"
```

- [ ] **Step 5: Merge, push, and verify the public URLs**

Merge the implementation branch into local `main`, push `main`, wait for the Pages workflow, then verify:

- `https://zhixuanlucasfeng-cmyk.github.io/bsr-hub/`
- `https://zhixuanlucasfeng-cmyk.github.io/bsr-hub/hub/`
- `https://zhixuanlucasfeng-cmyk.github.io/bsr-hub/runner/`

Expected: all return successfully and display the approved BSR logos.

## Self-Review

- Spec coverage: Hub/Runner navigation, footer, browser icon, base path, asset ownership, responsive sizing, accessibility, export verification, and public publishing are mapped to Tasks 1–3.
- Placeholder scan: no TBD, TODO, or unspecified implementation steps remain.
- Type consistency: both apps expose the same `BrandLogo` prop names and variant union; every use matches that interface.
