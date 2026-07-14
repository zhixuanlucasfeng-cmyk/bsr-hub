# BSR Hub Team Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and deliver a complete offline classroom demo for all five BSR Hub team roles.

**Architecture:** A Next.js App Router frontend calls the existing Rust/Axum API. Rust demo mode adds deterministic in-memory listings, orders, and protected-payment simulation while reusing authoritative pricing, quote, and state-machine code. Production-style Supabase migrations and team artifacts are delivered alongside the runnable demo.

**Tech Stack:** Rust 2021, Axum, Tokio, Serde, Next.js, React, TypeScript, CSS, Node test runner, PostgreSQL/Supabase SQL

## Global Constraints

- The demo must run without paid services, secret credentials, Docker, Supabase, Stripe, or internet access.
- Rust owns recommendation, quote, billing-unit, money, fulfillment, and transition rules.
- Money is represented as integer U.S. cents; service fee is 600 basis points.
- Rental duration is at least 30 minutes and seller price adjustment is limited to plus or minus $5 per billing unit.
- All people, listings, addresses, payments, and reviews are fictional.
- Public UI shows city and state only and never stores a card number.
- Existing user changes are preserved; the existing core migration is not edited.

---

### Task 1: Web Foundation and Demo Catalog

**Files:**
- Create: `apps/web/package.json`, `apps/web/tsconfig.json`, `apps/web/next.config.mjs`
- Create: `apps/web/src/app/layout.tsx`, `apps/web/src/app/page.tsx`, `apps/web/src/app/globals.css`
- Create: `apps/web/src/components/*`, `apps/web/src/lib/demo-data.ts`, `apps/web/src/lib/types.ts`
- Test: `apps/web/src/lib/demo-data.test.ts`

**Interfaces:**
- Produces: `Listing`, `DemoUser`, and catalog query helpers consumed by all pages.

- [ ] Write tests proving at least twelve fictional listings cover rentals, sales, workspaces, and delivery.
- [ ] Run the tests and confirm the missing catalog fails.
- [ ] Add the Next.js configuration, design system, navigation, responsive homepage, search/filter controls, and listing cards.
- [ ] Run unit tests, TypeScript check, lint, and production build.
- [ ] Commit the independently runnable catalog.

### Task 2: Listing Details, Persona Switcher, and Listing Form

**Files:**
- Create: `apps/web/src/app/listings/[id]/page.tsx`
- Create: `apps/web/src/app/listings/new/page.tsx`
- Create: `apps/web/src/components/persona-switcher.tsx`, `apps/web/src/components/listing-form.tsx`
- Test: `apps/web/src/lib/listing-form.test.ts`

**Interfaces:**
- Consumes: `Listing` and `DemoUser` from Task 1.
- Produces: validated `ListingDraft` with rental, sale, and workspace-specific fields.

- [ ] Write tests for listing-type validation, public location, delivery fee, deposit, and 30-minute minimum rules.
- [ ] Confirm the tests fail before implementation.
- [ ] Implement buyer, seller, and business persona switching plus detail and create-listing pages.
- [ ] Verify keyboard labels, mobile layout, validation, empty states, and no private-address fields.
- [ ] Run the full web quality gate and commit.

### Task 3: Rust Demo Catalog and Order Repository

**Files:**
- Create: `services/core-api/src/demo.rs`, `services/core-api/src/http/demo.rs`
- Modify: `services/core-api/src/config.rs`, `services/core-api/src/http/mod.rs`, `services/core-api/src/lib.rs`
- Test: `services/core-api/tests/demo_api.rs`

**Interfaces:**
- Produces: `GET /v1/demo/listings`, `GET /v1/demo/listings/{id}`, `GET /v1/demo/orders`, `POST /v1/demo/payments/{order_id}/complete`, and demo persona headers.
- Reuses: existing quote, order creation, fulfillment validation, and `OrderState::transition` logic.

- [ ] Write failing route tests for deterministic listings, persona access, protected payment, and invalid transitions.
- [ ] Run the focused Rust test and confirm the routes are absent.
- [ ] Implement `BSR_DEMO_MODE` configuration, in-memory fixtures, mock checkout, and demo routes without weakening production adapters.
- [ ] Run formatting, Clippy, all Rust tests, and commit.

### Task 4: Checkout, Dashboards, and Full Transaction UI

**Files:**
- Create: `apps/web/src/lib/api.ts`, `apps/web/src/lib/order-actions.ts`
- Create: `apps/web/src/app/checkout/[listingId]/page.tsx`
- Create: `apps/web/src/app/orders/page.tsx`, `apps/web/src/app/orders/[id]/page.tsx`
- Create: `apps/web/src/components/quote-panel.tsx`, `apps/web/src/components/order-timeline.tsx`
- Test: `apps/web/src/lib/order-actions.test.ts`, `apps/web/src/lib/api.test.ts`

**Interfaces:**
- Consumes: Rust quote/order/demo endpoints.
- Produces: action rendering derived from server state and persona relationship.

- [ ] Write failing tests for quote serialization, API-offline messages, and state/role-specific actions.
- [ ] Implement date/time, fulfillment, quote breakdown, create order, mock payment, seller confirmation, activation, return, completion, cancellation, and review UI.
- [ ] Prove that the browser never calculates the final total or invents transitions.
- [ ] Run all web and Rust gates and commit.

### Task 5: Database Security Deliverables

**Files:**
- Create: `supabase/migrations/20260714001000_marketplace_security.sql`
- Create: `supabase/tests/marketplace_security.sql`
- Create: `docs/database/booking-matrix.md`, `docs/database/order-transition-matrix.md`, `docs/reviews/yichen-rust-review.md`

**Interfaces:**
- Produces: listing images, availability, protected payments, completed-order reviews, and RLS evidence compatible with the existing schema.

- [ ] Write negative pgTAP cases for cross-user listing changes, browser payment writes, invalid reviews, and overlaps.
- [ ] Add the new migration without changing the existing core migration.
- [ ] Document identical, partial, contained, adjacent, expired, cancelled, completed, active, and concurrent booking expectations.
- [ ] Review Rust state and payment code against the matrices; record exact findings.
- [ ] Run available SQL checks, secret scan, and commit.

### Task 6: Research, Presentation, QA, and Handoff

**Files:**
- Create: `docs/research/competitor-summary.md`, `docs/research/demo-listings.json`, `docs/research/pricing-privacy-notes.md`
- Create: `docs/presentation/final-two-minute-script.md`, `docs/presentation/live-demo-script.md`
- Create: `docs/qa/accessibility-mobile-checklist.md`, `docs/qa/end-to-end-results.md`
- Create: `BSR-Hub-Team-Handoff/2026-07-14/INVENTORY.md`

**Interfaces:**
- Consumes: the final product routes, fictional listing catalog, and team role definitions.
- Produces: presentation-ready evidence and a safe source inventory.

- [ ] Add summarized competitor patterns without copying long website text.
- [ ] Add at least twelve fictional U.S. listings matching the web catalog.
- [ ] Write a timed five-speaker script covering problem, product, roles, UN Goals 8 and 10, and impact.
- [ ] Write exact live-demo clicks and recovery steps.
- [ ] Complete accessibility, mobile, error-state, and two-person journey results.
- [ ] Create the dated inventory and scan it for credentials, private data, and generated folders.
- [ ] Commit the documentation package.

### Task 7: Release, GitHub, and Browser Verification

**Files:**
- Modify: `README.md`, `.env.example`, `package.json`
- Create: `scripts/demo.sh`, `.github/workflows/ci.yml`

**Interfaces:**
- Produces: one-command local startup, CI gates, GitHub repository delivery, and verified browser demo.

- [ ] Add exact install, test, start, persona, and demo-flow instructions.
- [ ] Add CI for Rust format/Clippy/tests and web tests/typecheck/lint/build.
- [ ] Run the complete clean quality gate and record outputs in `docs/qa/end-to-end-results.md`.
- [ ] Start Rust and Next.js, open the site, and verify the PS5 journey in a browser.
- [ ] Commit all final files, push the current branch to the configured GitHub remote, and return the repository URL.
