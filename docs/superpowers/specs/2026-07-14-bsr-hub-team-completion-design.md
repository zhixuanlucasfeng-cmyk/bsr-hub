# BSR Hub Team Completion Design

**Status:** Approved by Lucas's instruction to complete the full team scope without further confirmation

## Goal

Deliver a classroom-ready BSR Hub MVP that runs without paid services or secret credentials, demonstrates meaningful Rust ownership of pricing and transaction rules, and includes the frontend, data, research, QA, and presentation work assigned to all five teammates.

## Scope and Success Criteria

The demo must let a presenter switch between a buyer and seller persona and complete this journey:

`Browse PS5 -> select time and fulfillment -> request Rust quote -> create order -> simulate protected payment -> seller confirms -> activate -> return -> complete -> review`

The product must also visibly support second-hand sales, user-created rental listings, workspaces, search/filtering, local delivery, deposits, and BSR-held payment. The repository must contain reproducible commands, no credentials, at least twelve fictional U.S. listings, competitor research, a two-minute presentation script, a live-demo checklist, and a dated handoff inventory.

## Chosen Architecture

Use a hybrid offline demo architecture:

- A Next.js App Router client in `apps/web` provides the complete responsive marketplace and persona-based demo authentication.
- The existing Rust/Axum service remains authoritative for price recommendations, quotes, billing units, money calculations, fulfillment validation, and order transitions.
- A Rust in-memory demo repository and mock checkout adapter provide deterministic local data without PostgreSQL, Supabase, Stripe, Docker, or network access.
- The browser stores only non-authoritative UI preferences and the selected demo persona. Transaction totals and allowed state transitions come from Rust.
- PostgreSQL/Supabase migrations remain production-oriented evidence and are expanded separately with listing images, availability, protected payments, reviews, and RLS.

## Components

### Web application

The web app contains a home page, marketplace/search view, listing detail, create-listing flow, checkout, buyer and seller dashboards, order detail, and review flow. Shared components cover navigation, cards, badges, form controls, empty/error/loading states, and responsive layout. Three demo personas are provided: buyer, seller, and small-business owner.

### Rust demo API

Demo mode is enabled only by `BSR_DEMO_MODE=true`. It exposes fictional listings and orders through explicit `/v1/demo/*` routes while reusing the existing pricing, quote, and order-state domain code. Mock payment changes only `pending_payment` to `paid`; it cannot skip state-machine rules. No live Stripe key is accepted.

### Data and security evidence

New timestamped migrations add listing images, availability, payments, reviews, and restrictive RLS without editing the existing core migration. SQL documents include the booking-overlap and state-transition matrices. Demo fixtures contain no real address, phone number, identity, or payment information.

### Research and presentation

Lucian's output is repository-ready Markdown/JSON: competitor patterns, pricing/privacy notes, twelve-plus fictional listings, and a dated team inventory. Nasia's output includes a timed two-minute script, exact demo clicks, failure recovery steps, and desktop/mobile QA evidence. Anna's output includes copy and accessibility review notes.

## Error Handling

The UI has explicit states for API offline, unavailable dates, expired reservation, failed or cancelled mock payment, unauthorized transition, missing listing, and unavailable delivery. Errors use stable Rust error codes where available. The demo never displays a private street address or asks for real payment credentials.

## Testing

- Rust: existing workspace tests plus demo-route and state-transition integration tests.
- Web: TypeScript check, lint, production build, and focused unit tests for API/data helpers and allowed-action rendering.
- End-to-end: scripted buyer/seller PS5 journey against local Rust and Next.js servers.
- Data: migration syntax/rebuild instructions and negative RLS cases documented for a future Supabase environment.
- Release: secret scan, mobile viewport smoke test, broken-link check, and reproducible start command.

## Delivery

The final branch is committed and pushed to the configured GitHub remote. The local website is started and opened in the browser. If the repository already has a deployable GitHub Pages or hosting configuration, publish it; otherwise the handoff clearly distinguishes the GitHub repository URL from the local demo URL.
