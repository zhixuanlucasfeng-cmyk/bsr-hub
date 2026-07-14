# BSR Hub — Teammate Task Sheet

**Deadline:** Ten working days  
**MVP integration target:** Complete PS5 rental by the end of Day 6  
**Feature freeze:** End of Day 8

## Shared Rules for Everyone

1. Pull the latest `main` before starting a task.
2. Use one short-lived branch per task; do not let two AI agents edit the same file.
3. Read the agreed API contract and database migration before generating dependent code.
4. Open a pull request with a test result or screenshot.
5. Obtain one human teammate review before merge.
6. Join the daily integration window; report blockers before starting unrelated work.

## Lucas — Coding and Project Coordination

### Primary ownership

- Set up the monorepo, linting, formatting, tests, and environment templates.
- Build the Rust/Axum API and SQLx data layer.
- Implement quote calculation, booking transaction, conflict prevention, state transitions, and Stripe webhook.
- Configure Vercel, Render, Supabase, and Stripe test environments.
- Coordinate daily priorities, integrate pull requests, and decide release readiness.

### Required deliverables

- `GET /health`, `POST /v1/quotes`, `POST /v1/orders`, order-transition, and Stripe webhook endpoints.
- Rust unit and database integration tests.
- Working deployed environments and documented setup commands.

## Anna — Communication and Frontend

### Primary ownership

- Build the design system, responsive navigation, and shared form components.
- Build sign-up, login, profile, create-listing, and edit-listing interfaces.
- Write consistent English product copy, empty states, and error messages.
- Maintain the daily team update and identify cross-person communication gaps.
- Review mobile accessibility, labels, keyboard use, and color contrast.

### Required deliverables

- Responsive auth, profile, and listing forms connected to Supabase.
- Reusable visual components and documented usage.
- Mobile and accessibility review checklist.

## Yichen — Logic and Data

### Primary ownership

- Design and implement Supabase migrations for profiles, listings, availability, orders, amounts, payments, reviews, and events.
- Implement Row Level Security policies and ownership rules.
- Implement or review the no-overlap booking constraint.
- Define valid order transitions and adversarial test cases.
- Review Lucas's Rust pricing and booking logic against the written rules.

### Required deliverables

- Repeatable migrations and seed-compatible schema.
- RLS tests showing cross-user writes are rejected.
- Booking and order-state test matrix.

## Lucian — Research and Marketplace

### Primary ownership

- Research comparable rental, resale, and workspace services and summarize useful patterns.
- Build the home, category, search-results, and listing-detail pages.
- Create realistic seed data for PS5s, computers, cameras, tools, studios, printing facilities, and small factories.
- Document pricing assumptions, privacy expectations, and future marketplace risks.
- Verify all public pages reveal only approximate locations.

### Required deliverables

- Responsive browse, search, and listing-detail experience.
- Seed-data script or migration with presentation-ready content.
- Short research and cost summary for the final report.

## Nasia — Presentation and Transaction UI

### Primary ownership

- Build date/time selection, fulfillment choice, quote display, and Stripe test checkout UI.
- Build buyer and seller dashboards, order detail, delivery state, return/fulfillment, and review UI.
- Maintain the end-to-end QA checklist and coordinate testing across two accounts.
- Lead the final presentation, demo script, backup recording, and rehearsal.

### Required deliverables

- Complete transaction UI connected to the Rust API.
- Buyer and seller order-management views.
- Tested two-minute team statement, live-demo script, and backup demo assets.

## Daily Integration Checklist

- Day 1: Contracts, migrations, routes, and ownership agreed.
- Day 2: Authentication and profiles work.
- Day 3: Listings can be published and found.
- Day 4: Dates and Rust quotes work.
- Day 5: Booking and Stripe test checkout work.
- Day 6: Full PS5 rental works across two accounts.
- Day 7: Dashboards, fulfillment, return, and reviews work.
- Day 8: Security, mobile, accessibility, and deployment pass.
- Day 9: No new features; fix critical issues and rehearse.
- Day 10: Smoke test and present.
