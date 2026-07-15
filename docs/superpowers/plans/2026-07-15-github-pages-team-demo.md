# GitHub Pages Team Demo Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish fully navigable static demos of BSR Hub and BSR Runner at stable GitHub Pages paths.

**Architecture:** Each Next.js app keeps its existing Rust-backed local mode and gains a typed in-browser adapter selected only by `NEXT_PUBLIC_STATIC_DEMO=true`. GitHub Actions exports both apps with repository-aware base paths and assembles them behind a small product-family landing page.

**Tech Stack:** Next.js 16 static export, TypeScript 5.8, Node test runner, GitHub Actions, GitHub Pages.

## Global Constraints

- The Rust API and local demo behavior must remain intact.
- Public data is fictional and browser-local; no real network writes occur.
- Hub path is `/bsr-hub/hub/`; Runner path is `/bsr-hub/runner/`.
- Mobile width 390 px must have no horizontal overflow.
- No new runtime npm dependency is permitted.

---

### Task 1: Static export configuration and public selector

**Files:**
- Modify: `apps/web/next.config.mjs`
- Modify: `apps/runner/next.config.mjs`
- Create: `deploy/pages/index.html`
- Create: `deploy/pages/styles.css`
- Test: `scripts/build-pages.sh`

**Interfaces:**
- Consumes: `NEXT_PUBLIC_BASE_PATH` and each app’s existing `build` script.
- Produces: `dist-pages/index.html`, `dist-pages/hub/index.html`, and `dist-pages/runner/index.html`.

- [ ] Configure both apps with `output: "export"`, `images.unoptimized: true`, and environment-derived `basePath`/`assetPrefix`.
- [ ] Create an accessible BSR product selector containing exact relative links `./hub/` and `./runner/`.
- [ ] Add `scripts/build-pages.sh` that cleans only generated output, builds both apps in static mode, copies their exports, and asserts all three entry files exist.
- [ ] Run `bash -n scripts/build-pages.sh`; expect exit 0.

### Task 2: BSR Hub in-browser demo adapter

**Files:**
- Create: `apps/web/src/lib/static-demo.ts`
- Create: `apps/web/src/lib/static-demo.test.ts`
- Modify: `apps/web/src/app/page.tsx`

**Interfaces:**
- Produces: `hubStaticDemo.listings`, `hubStaticDemo.ordersFor(persona)`, `hubStaticDemo.quote(...)`, `hubStaticDemo.createOrder(...)`, and `hubStaticDemo.act(...)`.
- Consumes: existing `Listing`, `Quote`, `DemoOrder`, and `Fulfillment` contracts.

- [ ] Write tests proving catalog availability, integer-cent quotes, order creation, role-specific transitions, and reset isolation.
- [ ] Run `npm run test -w @bsr-hub/web`; expect the new tests to fail because the adapter does not exist.
- [ ] Implement the adapter with fictional seed data and the existing state/action rules.
- [ ] Route page operations to the adapter only when `NEXT_PUBLIC_STATIC_DEMO === "true"`.
- [ ] Run web typecheck and tests; expect all pass.

### Task 3: BSR Runner in-browser demo adapter

**Files:**
- Create: `apps/runner/src/lib/static-demo.ts`
- Create: `apps/runner/src/lib/static-demo.test.ts`
- Modify: `apps/runner/src/app/page.tsx`

**Interfaces:**
- Produces: `runnerStaticRequest<T>(path, init?)`, matching the JSON response surface used by the current `request<T>` helper.
- Consumes: existing runner contract types and current demo endpoint paths.

- [ ] Write tests for four seeded tasks, prohibited quote rejection, create/fund/publish, assignment privacy, completion-code payout idempotency, applications, and admin summary.
- [ ] Run runner tests; expect failure because `runnerStaticRequest` does not exist.
- [ ] Implement the browser-local route adapter and fictional store.
- [ ] Select it only when `NEXT_PUBLIC_STATIC_DEMO === "true"`; preserve the Rust fetch path otherwise.
- [ ] Run runner typecheck and tests; expect all pass.

### Task 4: GitHub Pages workflow

**Files:**
- Create: `.github/workflows/pages.yml`
- Modify: `package.json`
- Modify: `README.md`

**Interfaces:**
- Consumes: `scripts/build-pages.sh` and GitHub’s Pages artifact actions.
- Produces: a Pages deployment for `github-pages` environment.

- [ ] Add `pages:build` and `pages:check` commands.
- [ ] Add a workflow with `contents: read`, `pages: write`, and `id-token: write`; build on pushes to `codex/bsr-runner` and manual dispatch.
- [ ] Document the public Hub and Runner URL shapes plus the distinction between static review mode and the Rust local demo.
- [ ] Run `npm run pages:check`; expect all entry assertions and app checks to pass.

### Task 5: Publish and verify

**Files:**
- Create: `docs/qa/github-pages-evidence.md`

**Interfaces:**
- Consumes: pushed workflow and public Pages URLs.
- Produces: exact HTTP and browser evidence for teammate handoff.

- [ ] Run `npm run check` and `npm run pages:check`; require exit 0.
- [ ] Commit and push `codex/bsr-runner`.
- [ ] Monitor the Pages workflow through the public GitHub API until success or a concrete actionable failure.
- [ ] Verify root, Hub, and Runner return HTTP 200.
- [ ] Test both product URLs in a browser at desktop and 390 px; record zero-overflow and console results.
- [ ] Write the evidence file, commit it, push it, and re-check the final URLs.
