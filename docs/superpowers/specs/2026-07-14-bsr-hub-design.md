# BSR Hub MVP Design

**Date:** July 14, 2026  
**Audience:** Babson summer school project team  
**Delivery window:** Two weeks / ten working days  
**Team:** Lucas, Anna, Yichen, Lucian, and Nasia

## 1. Product Summary

BSR Hub is a responsive community marketplace for the Babson campus and nearby U.S. community. It gives people and small businesses affordable temporary access to products, equipment, and workspaces while allowing owners to earn income from underused resources.

One user account can participate on both sides of the marketplace. The same person can rent an item, rent out an item, buy or sell a second-hand product, book a workspace, and list a workspace.

The two-week deliverable is an English-language responsive website/PWA using U.S. dollars and U.S. address formats. It demonstrates payments and deposits through Stripe test mode; it does not process real money.

## 2. Problem and Outcomes

### Problem

- Students may want to use expensive products, such as a PS5, gaming PC, camera, or power tool, without buying them.
- Small businesses may need a studio, printer, workshop, or small factory for only a few hours.
- Owners often have useful products or spaces that remain unused.
- Existing access may be expensive, fragmented, or limited to one type of resource.

### Desired outcome

BSR Hub connects users who need affordable access with owners who have unused resources. The product supports UN Sustainable Development Goal 8, Decent Work and Economic Growth, and Goal 10, Reduced Inequalities.

## 3. Users and Account Model

There is one authenticated user type, not separate buyer and seller accounts. Any authenticated user can:

- publish, edit, or deactivate a listing;
- rent or buy another user's listing;
- rent out or sell their own property;
- list or book an on-site workspace;
- choose or offer pickup, delivery, owner-location use, or on-site use;
- manage orders and leave a review after completion.

The system also has an administrator capability for moderation, but its UI is a stretch goal. Administration can be performed through Supabase during the class demonstration if needed.

## 4. MVP Scope

### Required

1. Email sign-up, login, logout, and profile management.
2. Create, edit, view, and deactivate listings.
3. Three listing types:
   - rentable item;
   - second-hand item for sale;
   - bookable workspace or facility.
4. Listing images, category, condition, approximate location, description, pricing, deposit, and availability.
5. Search and category browsing.
6. Listing-specific fulfillment options:
   - customer pickup;
   - owner-arranged paid delivery;
   - use at the owner's location;
   - on-site space access.
7. Date or time selection for rentals and spaces.
8. Rust-generated quote containing base price, deposit, delivery fee, service fee, and total.
9. Rust booking transaction that prevents overlapping reservations.
10. Stripe test-mode checkout.
11. Buyer and seller order dashboards.
12. Confirmation, active/use, return/fulfillment, completion, and cancellation states.
13. Delivery status when delivery is selected.
14. Reviews after a completed order.
15. Responsive desktop and mobile layouts.

### Stretch goals

- favorites;
- simple buyer–seller messages;
- PWA installation prompt;
- administrator moderation screen;
- email booking notifications.

No stretch goal begins until the complete PS5 rental journey works on desktop and mobile.

### Explicitly outside the two-week MVP

- live payments, seller payouts, or real deposit holds;
- courier registration, job bidding, live tracking, route optimization, or courier payouts;
- identity or background verification;
- insurance and damage arbitration;
- native iOS and Android applications;
- multi-currency or multi-country operation.

## 5. Principal Customer Journeys

### Rent a movable product

1. User signs in and finds a PS5.
2. User selects dates and one supported method: pickup, delivery, or use at the owner's location.
3. Next.js sends the listing, dates, and fulfillment choice to the Rust quote endpoint.
4. Rust reads authoritative pricing and availability, then returns the itemized total.
5. User accepts the quote. Rust creates a `pending_payment` order and a 30-minute reservation in one database transaction.
6. Rust returns a Stripe test Checkout URL and the user completes checkout.
7. The verified Stripe webhook changes the order to `paid`; an unpaid reservation expires automatically.
8. Seller confirms the order.
9. The order moves through active, returned, and completed states.
10. Buyer and seller may review each other.

### Book a workspace

The user selects a date/time block and books on-site use. Pickup and delivery are not available for an immovable space.

### Buy a second-hand product

The user purchases one item for a fixed price and selects pickup or delivery. The order has no rental dates or refundable deposit.

## 6. Technical Architecture

### Repository

A single GitHub monorepo contains:

```text
apps/web/             Next.js PWA
services/core-api/    Rust/Axum API
packages/contracts/   Shared API schemas and generated TypeScript types
supabase/migrations/  PostgreSQL schema, constraints, and RLS policies
docs/                 Product, architecture, and team documentation
```

### Web application

- Next.js and TypeScript
- Tailwind CSS
- Supabase JavaScript client for authentication, profiles, public listings, images, and permitted dashboard reads
- Server-side calls to the Rust API for quotes and transactional order changes
- Stripe Checkout in test mode

### Rust core API

Rust with Axum and SQLx owns business rules where client-side mistakes would be harmful:

- price and deposit calculations;
- fulfillment validation;
- availability and overlap checks;
- transactional order creation;
- valid order-state transitions;
- Stripe webhook verification;
- audit-event creation.

The browser never supplies an authoritative total. Rust reloads listing prices and fees from PostgreSQL before creating a quote or order.

### Managed services

- Supabase: PostgreSQL, authentication, row-level security, and image storage
- Vercel: Next.js deployment
- Render: Rust API deployment
- Stripe: test checkout
- GitHub: source control and pull-request review

## 7. Service Interfaces

The minimum Rust API surface is:

- `GET /health` — deployment health check.
- `POST /v1/quotes` — validate dates and fulfillment, then calculate an itemized total.
- `POST /v1/orders` — create a `pending_payment` order and 30-minute reservation in a transaction, then return its Stripe test Checkout URL.
- `POST /v1/orders/{id}/transitions` — apply an allowed order-state change.
- `POST /v1/stripe/webhook` — verify Stripe events and update payment state idempotently.

Every response uses a stable JSON error shape:

```json
{
  "error": {
    "code": "LISTING_UNAVAILABLE",
    "message": "The selected dates are no longer available.",
    "request_id": "..."
  }
}
```

## 8. Data Model

### Core tables

- `profiles`: public profile data linked one-to-one to `auth.users`.
- `listings`: owner, listing type, details, pricing, deposit, approximate location, fulfillment capabilities, and status.
- `listing_images`: Supabase Storage paths and display order.
- `availability`: available or blocked time windows.
- `orders`: listing, buyer, seller, dates, fulfillment method, address reference, type, status, and pending-payment expiration.
- `order_amounts`: immutable itemized quote captured in integer U.S. cents.
- `payments`: Stripe test identifiers, payment state, and deposit state.
- `reviews`: order-linked rating and comment.
- `order_events`: append-only audit trail for order changes.

### Integrity rules

- PostgreSQL exclusion or equivalent transactional constraints prevent overlapping bookings for the same rental listing.
- Rust treats an expired `pending_payment` reservation as nonblocking and marks it `expired` during the next relevant booking transaction; the MVP does not require a background scheduler.
- Sale listings can complete only once.
- `buyer_id` cannot equal `seller_id`.
- Reviews require a completed order and are limited to one review per reviewer per order.
- Monetary amounts are stored as integers in cents.
- Order price snapshots are not recalculated after payment.

## 9. Order and Delivery States

The shared order state machine is:

```text
draft → pending_payment → paid → confirmed → active_or_fulfilled
      → returned_if_rental → completed
pending_payment → expired (payment window elapsed)
```

Cancellation is allowed only from documented pre-completion states. Invalid transitions return `409 INVALID_ORDER_TRANSITION`.

Delivery state is separate:

```text
not_required | requested | out_for_delivery | delivered
```

For the MVP, the owner arranges delivery and updates its status. A separate courier marketplace is not included.

## 10. Security and Privacy

- Supabase Row Level Security restricts profile, listing, and order writes.
- Public listing results show only city/area, never a private street address.
- Exact pickup or use addresses are revealed only to authorized order participants after confirmation.
- Next.js and Rust secrets are stored only as hosting environment variables.
- Stripe webhook signatures are verified in Rust.
- Rust verifies the Supabase access token and derives the acting user from it.
- The system never stores card numbers.
- File uploads enforce image MIME types, size limits, and owner-scoped paths.

## 11. Error Handling

- Expired session: redirect to sign-in and preserve the intended return page.
- Unavailable dates: show the conflict and do not create an order.
- Cancelled or failed test payment: preserve the pending-payment order until its 30-minute reservation expires and provide retry during that window.
- Image upload failure: preserve the form and retry only the image.
- Rust API unavailable: display a clear service error and never show false success.
- Unauthorized action: return `403`, log the request ID, and omit the action from normal UI.
- Repeated Stripe webhook: return success without duplicating the payment or transition.

## 12. Testing and Acceptance

### Automated tests

- Rust unit tests for rental units, pricing, fees, deposits, and every order transition.
- Rust/PostgreSQL integration test proving two simultaneous requests cannot reserve the same period.
- Supabase policy tests proving one user cannot edit another user's protected data.
- Frontend validation tests for listing and checkout forms.
- End-to-end test for login → PS5 search → booking → Stripe test payment → seller confirmation.

### Presentation-ready acceptance gate

The deployed product must complete the full PS5 rental journey twice with two separate test accounts on both desktop and mobile, without manual database edits. A backup recording and screenshots are prepared before the presentation.

## 13. Ten-Day Schedule and Ownership

### Ownership

- **Lucas — Coding and coordination:** repository, shared standards, Rust API, Stripe, deployment, integration, and release decisions.
- **Anna — Communication and frontend:** design system, navigation, authentication/profile UI, public browse/search/listing pages, listing forms, copy, accessibility, and team updates.
- **Yichen — Logic and data:** migrations, RLS, availability constraints, state-machine test cases, and Rust rule review.
- **Lucian — Research and file coordination:** user/competitor research, seed-listing content, pricing report, privacy notes, daily collection of teammate outputs, and preparation of a structured handoff package for Lucas. Lucian is not assigned AI-assisted coding work.
- **Nasia — Presentation and transaction UI:** checkout, dashboards, fulfillment UI, reviews, QA checklist, demo narrative, and presentation.

### Daily milestones

1. Repository, wireframes, database schema, API contracts, and development standards.
2. Authentication, profiles, migrations, design system, and seed-data research.
3. Listing creation/editing, image upload, categories, search, and mobile layout.
4. Availability UI, booking form, and Rust quote calculation.
5. Rust transactional booking, overlap checks, and Stripe test checkout.
6. Integration checkpoint: one complete PS5 rental; only integration fixes that day.
7. Dashboards, delivery status, returns/fulfillment, and reviews.
8. Security/RLS tests, error states, accessibility, mobile QA, and deployment.
9. Feature freeze: seed data, critical bug fixes, report, backup recording, and rehearsal.
10. Smoke test, final rehearsal, and live presentation.

## 14. Team Development Rules

- One short-lived branch per task.
- No two agents edit the same file concurrently.
- Database and API changes require Lucas and Yichen review.
- Each pull request includes a test result or screenshot and one human review.
- Merge at one scheduled time each day; Lucas owns integration conflict resolution.
- Shared contracts are agreed before dependent feature code begins.
- Lucian maintains a daily backup handoff folder organized by contributor. He does not merge source code; Lucas reviews and integrates collected work into the monorepo.

## 15. Cost

### Two-week class demonstration

- Vercel Hobby: $0.
- Supabase Free: $0.
- Render Free web service: $0.
- Stripe test mode: $0.
- Optional custom domain: approximately $10–20 per year depending on registrar and name.

Expected required platform cost: **$0**.

### Small real launch estimate

- Vercel Pro: $20/month.
- Supabase Pro: $25/month.
- Render Starter web service: $7/month.
- Domain: approximately $10–20/year.
- Stripe live U.S. domestic card processing: 2.9% + $0.30 per successful transaction.

Expected fixed platform cost: **approximately $52/month plus the domain and Stripe transaction fees**.

Pricing references checked July 2026:

- [Vercel pricing](https://vercel.com/pricing)
- [Supabase pricing](https://supabase.com/pricing)
- [Render pricing](https://render.com/pricing)
- [Stripe pricing](https://stripe.com/pricing)

## 16. Final Scope Decision

The team prioritizes a complete, credible marketplace transaction over breadth. The required PS5 rental journey is the integration spine. Item sales and workspace reservations reuse the same listings, orders, fulfillment, payment, and review foundations. Features outside the stated MVP do not enter development during the two-week program.
