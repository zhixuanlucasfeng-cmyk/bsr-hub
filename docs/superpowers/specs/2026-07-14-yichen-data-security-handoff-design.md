# Yicheng Data and Security Handoff Design

**Date:** 2026-07-14  
**Owner:** Yicheng  
**Integration owner:** Lucas

## Objective

Yicheng will complete the database features and security evidence that are still missing after Lucas's Rust core-platform merge. His work must extend the current schema without rewriting the migration or Rust implementation that already passes the project test suite.

## Existing Baseline

The following file is immutable after being applied and must not be edited:

`supabase/migrations/20260714000100_core_marketplace.sql`

It already defines profiles, listings, private listing locations, orders, immutable order amounts, order events, Stripe event idempotency, basic Row Level Security, and the PostgreSQL exclusion constraint for overlapping bookings.

Lucas's Rust service already owns authoritative quotes, the configurable 600-basis-point service fee, 30-minute reservations, order-state transitions, Supabase token verification, PostgreSQL transactions, Stripe test Checkout, and signed webhook processing.

## Scope

Yicheng owns four bounded work areas:

1. New migrations for `listing_images`, `availability`, `payments`, and `reviews`.
2. Additional RLS policies for ownership, order participation, private locations, protected financial fields, and completed-order reviews.
3. SQL security and booking test cases, including concurrent overlap attempts.
4. A read-only review of Lucas's Rust transaction rules with reproducible issue reports.

He must not modify the Next.js frontend, environment files, deployment configuration, or Lucas's Rust source without coordination. Any database change must be introduced by a new timestamped migration after `20260714000100`.

## Schema Additions

### Listing images

Each image belongs to one listing, stores a storage-object path rather than image bytes, has a stable display order, and is deleted with its listing. Only the listing owner can create, reorder, or delete images. Public users can read images only for active listings.

### Availability

Availability rows define owner-approved rentable windows for rental and workspace listings. A row contains a start time and end time with `start_at < end_at`. Availability does not replace the existing order exclusion constraint; the database must require a requested rental window to fit an allowed availability window and still reject overlaps with active orders.

### Payments

Payment rows contain the BSR order ID, provider name, provider reference, integer amount in U.S. cents, currency fixed to `USD`, and a controlled status. Browser roles may read payment status only for their own order but may not insert, update, or delete payment records. Only the trusted backend database role may mutate them.

### Reviews

A review belongs to one completed order and records reviewer, reviewee, rating from 1 through 5, optional short text, and creation time. Only an order participant may review the other participant. Each participant can submit at most one review per order, and reviews cannot be created before the order reaches `completed`.

## Security Rules

- Anonymous users may read active listings, their public images, and safe public profile fields only.
- Authenticated users may update only their own profile and listings.
- Exact street addresses remain in `listing_private_locations`; listing owners may manage them. A confirmed transaction participant may receive the required address through a controlled backend path, not unrestricted public table access.
- Buyers and listing owners may read only orders in which they participate.
- Browser users cannot alter authoritative order amounts, payment status, Stripe event IDs, or protected state fields.
- Review creation must be enforced by database policy and constraints, not only by UI checks.
- Service-role tests must be separated from anonymous and authenticated-user tests so elevated credentials do not make an insecure policy appear to pass.

## Booking Test Matrix

The evidence must cover identical periods, partial overlaps, contained periods, containing periods, adjacent non-overlapping periods, expired pending-payment orders, cancelled orders, completed orders, active orders, and two concurrent insert attempts. All overlap shapes must yield one winner at most. Adjacent periods, expired holds, cancelled orders, and completed orders must not block a valid new booking.

## Rust Review Boundary

Yicheng will review integer-cent calculations, deposits, delivery fees, the 600-basis-point service fee, server-authoritative totals, self-transaction prevention, transactional reservations, overlap handling, state transitions, and Stripe webhook idempotency. Each finding must include the exact request or database state, actual result, expected result, severity, and affected file. He will not directly rewrite Rust code without Lucas assigning the change.

## Delivery Structure

Yicheng will return one folder:

```text
yicheng-handoff/
├── migrations/
├── database-tests/
├── rls-tests/
├── booking-test-matrix.md
├── order-transition-matrix.md
├── rust-review.md
└── README.md
```

The repository-ready migration and tests must also retain their normal project paths. The handoff copy is for team review and archival, not a second source of truth.

## Verification

Yicheng must provide exact commands and captured results for a clean migration rebuild, anonymous access tests, two-user cross-write tests, protected financial-record tests, review eligibility tests, every booking-overlap case, and the concurrent booking case. No real credentials, access tokens, private addresses, or Stripe secrets may appear in the handoff.

## Completion Criteria

Yicheng's portion is complete when a clean Supabase/PostgreSQL database can be rebuilt from ordered migrations; cross-user writes and protected financial mutations are rejected; private addresses are not publicly readable; reviews require a completed order; conflicting concurrent reservations cannot both succeed; and Lucas receives the organized handoff with reproducible test evidence.
