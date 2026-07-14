# Yichen — Individual Task Instructions

## Your Role

You are responsible for **database design, permissions, booking logic, and logical test cases**. Your work ensures users cannot edit another person's data, two customers cannot rent the same item for overlapping dates, and orders follow valid state transitions.

Because you do not have a VPN, use **Qoder CN（原通义灵码）** as your coding agent. Work on local SQL, migration, Rust test, and documentation files. Lucas will create and operate the shared Supabase cloud project.

## Tool Setup

1. Install Qoder CN or its supported VS Code/JetBrains extension.
2. Open only the official BSR Hub project folder.
3. Ask Lucas for the current repository ZIP or Git access and the approved schema/API contracts.
4. Never paste API keys, `.env` contents, passwords, private addresses, or Stripe secrets into the agent.
5. Return source files and test output to Lucas through the agreed team channel.

## Main Responsibilities

### 1. PostgreSQL migrations

Create repeatable Supabase/PostgreSQL migrations for:

- profiles;
- listings;
- listing images;
- availability;
- orders;
- order amounts;
- payments;
- reviews;
- order events.

Include primary keys, foreign keys, required fields, timestamps, indexes, and check constraints. Store money as integer U.S. cents.

### 2. Row Level Security

Write and test policies so that:

- public users can read only active public listings and safe public profile fields;
- authenticated users can edit only their own profile and listings;
- buyers and sellers can read only orders in which they participate;
- users cannot change authoritative order prices or payment status directly;
- exact addresses are visible only to authorized order participants after confirmation;
- reviews require a completed order involving the reviewer.

### 3. Availability and conflict prevention

Design the constraint and transaction rules that prevent overlapping bookings for the same rentable listing or workspace. Pending-payment reservations last 30 minutes. Expired reservations must no longer block future bookings.

Prepare test cases for:

- identical dates;
- partially overlapping dates;
- one booking fully inside another;
- adjacent, non-overlapping dates;
- expired pending payment;
- completed, cancelled, and active orders;
- two simultaneous booking requests.

### 4. Order-state logic

Document and test the allowed state transitions:

```text
draft → pending_payment → paid → confirmed → active_or_fulfilled
      → returned_if_rental → completed
pending_payment → expired
```

Cancellation is permitted only from approved pre-completion states. Create a table showing every allowed and rejected transition for rental, sale, and workspace orders.

### 5. Rust logic review

Review Lucas's Rust implementation for:

- integer price calculations;
- deposit rules;
- delivery and service fees;
- authoritative server-side totals;
- transactional order creation;
- booking overlap checks;
- valid state transitions;
- idempotent Stripe webhook handling.

Report issues with a reproducible example and expected result. Do not rewrite Lucas's Rust files without coordinating first.

## Ten-Day Schedule

- **Day 1:** Finalize schema, constraints, state machine, and migration order.
- **Day 2:** Implement profiles, listings, images, availability, and initial RLS.
- **Day 3:** Implement orders, amounts, payments, reviews, events, and indexes.
- **Day 4:** Implement overlap protection and booking test matrix.
- **Day 5:** Review Rust quote/order logic and test Stripe-state assumptions.
- **Day 6:** Integration only; fix database or contract issues blocking the PS5 rental.
- **Day 7:** Finish review, delivery status, return, and review policies.
- **Day 8:** Run security, cross-user, concurrency, and migration-rebuild tests.
- **Day 9:** No new features; fix critical issues and prepare evidence.
- **Day 10:** Support the final smoke test and answer data/security questions.

## Required Deliverables

- Ordered migration files under `supabase/migrations/`.
- RLS policies and security tests.
- Booking-overlap and concurrency test cases.
- Order-state transition matrix.
- Rust logic review notes.
- A clean database rebuild procedure for Lucas.

## Recommended Qoder CN Request

```text
Work only on Supabase/PostgreSQL migrations, database tests, and logic documentation.
Do not change the Next.js frontend, Rust API implementation, or environment files.
Implement the approved BSR Hub schema and RLS policies with explicit tests proving
that cross-user writes and overlapping bookings are rejected. Store money as integer
cents. Summarize every changed file and provide the exact local verification commands.
```

## Definition of Done

Yichen's work is complete when Lucas can rebuild the database from migrations, two simultaneous users cannot book the same period, one user cannot modify another user's protected records, and every valid or invalid order transition has a documented test.
