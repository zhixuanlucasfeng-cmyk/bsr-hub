# Yicheng Data Security Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the existing BSR Hub Supabase schema with listing media, availability, protected payments, completed-order reviews, comprehensive RLS, and reproducible database security evidence.

**Architecture:** New timestamped migrations extend the immutable core migration; no applied migration is rewritten. PostgreSQL constraints, triggers, and RLS remain the authoritative security boundary, while pgTAP tests exercise anonymous, authenticated, and concurrent booking behavior independently of the frontend.

**Tech Stack:** PostgreSQL 15+, Supabase CLI, Supabase Auth/RLS, pgTAP, SQL migrations, Markdown review evidence, Qoder CN.

## Global Constraints

- Do not modify `supabase/migrations/20260714000100_core_marketplace.sql`.
- Do not modify Rust, Next.js, `.env`, deployment, or Stripe configuration files.
- Create one new timestamped migration per independently reviewable schema/security change.
- Store all money as signed 64-bit integer cents and fix currency to `USD`.
- Store image storage paths, never image bytes or expiring signed URLs.
- Browser roles may not mutate order amounts, payments, or Stripe event records.
- Exact street addresses may not be exposed by public listing reads.
- Rental and workspace orders require a valid availability window and may not overlap active reservations.
- Never place credentials, JWTs, private addresses, or payment details in fixtures or captured output.

---

## Planned File Structure

```text
supabase/migrations/20260714000200_listing_media_availability.sql
supabase/migrations/20260714000300_payments_reviews.sql
supabase/migrations/20260714000400_complete_rls.sql
supabase/tests/001_schema_constraints.sql
supabase/tests/002_rls_security.sql
supabase/tests/003_booking_conflicts.sql
docs/database/booking-test-matrix.md
docs/database/order-transition-matrix.md
docs/reviews/yichen-rust-review.md
docs/runbooks/database-rebuild.md
yicheng-handoff/README.md
```

## Task 1: Listing Images and Availability Windows

**Files:**
- Create: `supabase/migrations/20260714000200_listing_media_availability.sql`
- Test: `supabase/tests/001_schema_constraints.sql`

**Interfaces:**
- Consumes: `listings(id, owner_id, listing_type, status)` from the core migration.
- Produces: `listing_images`, `listing_availability`, and `enforce_order_availability()`.

- [ ] **Step 1: Write the failing pgTAP schema test**

Create `supabase/tests/001_schema_constraints.sql`:

```sql
begin;
select plan(8);

select has_table('public', 'listing_images', 'listing_images exists');
select has_table('public', 'listing_availability', 'listing_availability exists');
select col_type_is('public', 'listing_images', 'storage_path', 'text', 'image path is text');
select col_is_unique('public', 'listing_images', array['listing_id', 'display_order'], 'image order is unique per listing');
select col_not_null('public', 'listing_availability', 'start_at', 'availability start is required');
select col_not_null('public', 'listing_availability', 'end_at', 'availability end is required');
select has_function('public', 'enforce_order_availability', array[]::text[], 'availability trigger function exists');
select has_trigger('public', 'orders', 'orders_require_availability', 'orders enforce availability');

select * from finish();
rollback;
```

- [ ] **Step 2: Run the test and verify RED**

Run: `supabase test db supabase/tests/001_schema_constraints.sql`

Expected: FAIL because `listing_images`, `listing_availability`, and the trigger do not exist.

- [ ] **Step 3: Implement the migration**

Create `supabase/migrations/20260714000200_listing_media_availability.sql`:

```sql
create table public.listing_images (
  id uuid primary key default gen_random_uuid(),
  listing_id uuid not null references public.listings(id) on delete cascade,
  storage_path text not null check (storage_path <> '' and storage_path !~ '^(https?://)'),
  alt_text text not null default '' check (char_length(alt_text) <= 160),
  display_order integer not null default 0 check (display_order >= 0),
  created_at timestamptz not null default now(),
  unique (listing_id, display_order),
  unique (listing_id, storage_path)
);

create table public.listing_availability (
  id uuid primary key default gen_random_uuid(),
  listing_id uuid not null references public.listings(id) on delete cascade,
  start_at timestamptz not null,
  end_at timestamptz not null,
  created_at timestamptz not null default now(),
  check (start_at < end_at),
  exclude using gist (
    listing_id with =,
    tstzrange(start_at, end_at, '[)') with &&
  )
);

create index listing_images_listing_idx
  on public.listing_images(listing_id, display_order);
create index listing_availability_listing_idx
  on public.listing_availability(listing_id, start_at, end_at);

create or replace function public.enforce_order_availability()
returns trigger
language plpgsql
security definer
set search_path = public
as $$
declare
  kind text;
begin
  select listing_type into kind from public.listings where id = new.listing_id;
  if kind in ('rental', 'workspace') then
    if new.start_at is null or new.end_at is null or new.start_at >= new.end_at then
      raise exception using errcode = '23514', message = 'rental window is required';
    end if;
    if not exists (
      select 1 from public.listing_availability a
      where a.listing_id = new.listing_id
        and tstzrange(a.start_at, a.end_at, '[)') @>
            tstzrange(new.start_at, new.end_at, '[)')
    ) then
      raise exception using errcode = '23514', message = 'order is outside listing availability';
    end if;
  elsif new.start_at is not null or new.end_at is not null then
    raise exception using errcode = '23514', message = 'sale orders cannot contain a rental window';
  end if;
  return new;
end;
$$;

create trigger orders_require_availability
before insert or update of listing_id, start_at, end_at on public.orders
for each row execute function public.enforce_order_availability();
```

- [ ] **Step 4: Rebuild and verify GREEN**

Run: `supabase db reset && supabase test db supabase/tests/001_schema_constraints.sql`

Expected: database rebuild exits `0`; all 8 pgTAP assertions pass.

- [ ] **Step 5: Commit**

```bash
git add supabase/migrations/20260714000200_listing_media_availability.sql supabase/tests/001_schema_constraints.sql
git commit -m "feat: add listing media and availability schema"
```

## Task 2: Protected Payments and Completed-Order Reviews

**Files:**
- Create: `supabase/migrations/20260714000300_payments_reviews.sql`
- Modify: `supabase/tests/001_schema_constraints.sql`

**Interfaces:**
- Consumes: `orders`, `order_amounts`, `profiles`, and `listings`.
- Produces: `payments`, `reviews`, and `validate_review_participants()`.

- [ ] **Step 1: Extend the failing schema test**

Add these assertions before `finish()` and change the plan from `8` to `16`:

```sql
select has_table('public', 'payments', 'payments exists');
select has_table('public', 'reviews', 'reviews exists');
select col_type_is('public', 'payments', 'amount_cents', 'bigint', 'payment money uses bigint cents');
select col_has_check('public', 'payments', 'amount_cents', 'payment amount is checked');
select col_is_unique('public', 'payments', array['provider', 'provider_reference'], 'provider reference is idempotent');
select col_has_check('public', 'reviews', 'rating', 'review rating is checked');
select col_is_unique('public', 'reviews', array['order_id', 'reviewer_id'], 'one review per participant and order');
select has_trigger('public', 'reviews', 'reviews_validate_participants', 'review participants are validated');
```

- [ ] **Step 2: Verify RED**

Run: `supabase test db supabase/tests/001_schema_constraints.sql`

Expected: the eight new assertions fail because the new migration is absent.

- [ ] **Step 3: Implement payment and review tables**

Create `supabase/migrations/20260714000300_payments_reviews.sql`:

```sql
create table public.payments (
  id uuid primary key default gen_random_uuid(),
  order_id uuid not null references public.orders(id) on delete restrict,
  provider text not null check (provider in ('stripe_test')),
  provider_reference text not null,
  amount_cents bigint not null check (amount_cents >= 0),
  currency text not null default 'USD' check (currency = 'USD'),
  status text not null check (status in ('created', 'succeeded', 'failed', 'refunded_simulated')),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  unique (provider, provider_reference)
);

create table public.reviews (
  id uuid primary key default gen_random_uuid(),
  order_id uuid not null references public.orders(id) on delete restrict,
  reviewer_id uuid not null references public.profiles(id),
  reviewee_id uuid not null references public.profiles(id),
  rating smallint not null check (rating between 1 and 5),
  body text not null default '' check (char_length(body) <= 1000),
  created_at timestamptz not null default now(),
  check (reviewer_id <> reviewee_id),
  unique (order_id, reviewer_id)
);

create index payments_order_idx on public.payments(order_id);
create index reviews_reviewee_idx on public.reviews(reviewee_id, created_at desc);

create or replace function public.validate_review_participants()
returns trigger
language plpgsql
security definer
set search_path = public
as $$
declare
  order_buyer uuid;
  order_owner uuid;
  order_status text;
begin
  select o.buyer_id, l.owner_id, o.status
    into order_buyer, order_owner, order_status
  from public.orders o
  join public.listings l on l.id = o.listing_id
  where o.id = new.order_id;

  if order_status is distinct from 'completed' then
    raise exception using errcode = '23514', message = 'reviews require a completed order';
  end if;
  if not (
    (new.reviewer_id = order_buyer and new.reviewee_id = order_owner) or
    (new.reviewer_id = order_owner and new.reviewee_id = order_buyer)
  ) then
    raise exception using errcode = '23514', message = 'review participants do not match the order';
  end if;
  return new;
end;
$$;

create trigger reviews_validate_participants
before insert or update on public.reviews
for each row execute function public.validate_review_participants();
```

- [ ] **Step 4: Verify GREEN**

Run: `supabase db reset && supabase test db supabase/tests/001_schema_constraints.sql`

Expected: all 16 assertions pass.

- [ ] **Step 5: Commit**

```bash
git add supabase/migrations/20260714000300_payments_reviews.sql supabase/tests/001_schema_constraints.sql
git commit -m "feat: add protected payments and verified reviews"
```

## Task 3: Complete Row Level Security Policies

**Files:**
- Create: `supabase/migrations/20260714000400_complete_rls.sql`
- Create: `supabase/tests/002_rls_security.sql`

**Interfaces:**
- Consumes: all marketplace tables from Tasks 1 and 2.
- Produces: browser-role policies with no client write path to authoritative financial tables.

- [ ] **Step 1: Write the failing RLS test**

Create `supabase/tests/002_rls_security.sql`:

```sql
begin;
select plan(11);

insert into public.profiles(id, display_name, city, state) values
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Owner', 'Wellesley', 'MA'),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Buyer', 'Wellesley', 'MA');

insert into public.listings(
  id, owner_id, listing_type, title, unit_price_cents,
  deposit_cents, delivery_fee_cents, status, city, state
) values (
  'cccccccc-cccc-cccc-cccc-cccccccccccc',
  'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
  'rental', 'Protected PS5', 2500, 10000, 1500, 'active', 'Wellesley', 'MA'
);

insert into public.listing_private_locations(listing_id, street_address, postal_code)
values ('cccccccc-cccc-cccc-cccc-cccccccccccc', '1 Test Only Road', '02481');

insert into public.listing_availability(listing_id, start_at, end_at)
values ('cccccccc-cccc-cccc-cccc-cccccccccccc', '2026-07-20', '2026-07-31');

insert into public.orders(
  id, listing_id, buyer_id, status, start_at, end_at, reservation_expires_at
) values (
  'dddddddd-dddd-dddd-dddd-dddddddddddd',
  'cccccccc-cccc-cccc-cccc-cccccccccccc',
  'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
  'completed', '2026-07-20', '2026-07-22', '2026-07-20'
);

insert into public.order_amounts(
  order_id, base_cents, service_fee_cents,
  delivery_fee_cents, deposit_cents, total_cents, currency
) values (
  'dddddddd-dddd-dddd-dddd-dddddddddddd',
  5000, 300, 1500, 10000, 16800, 'USD'
);

select policies_are('public', 'payments', array['participants read payments'], 'payments expose one read policy');
select policies_are('public', 'order_amounts', array['participants read order amounts'], 'amounts expose one read policy');
select policies_are('public', 'reviews', array['public reads reviews', 'participants create reviews'], 'review policies are exact');
select policies_are('public', 'listing_images', array['public reads active listing images', 'owners manage listing images'], 'image policies are exact');
select policies_are('public', 'listing_availability', array['public reads active availability', 'owners manage availability'], 'availability policies are exact');

select lives_ok($$ set local role authenticated $$, 'authenticated role can be selected');
select lives_ok($$ select set_config('request.jwt.claim.sub', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', true) $$, 'buyer identity is set');
select results_eq(
  $$ update public.listings set title = 'stolen' where owner_id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa' returning id $$,
  array[]::uuid[],
  'buyer cannot modify seller listing'
);
select throws_ok($$ insert into public.payments(order_id, provider, provider_reference, amount_cents, status) values ('dddddddd-dddd-dddd-dddd-dddddddddddd', 'stripe_test', 'pi_forbidden', 100, 'succeeded') $$, '42501', null, 'browser cannot create payment');
select throws_ok($$ update public.order_amounts set total_cents = 1 where order_id = 'dddddddd-dddd-dddd-dddd-dddddddddddd' $$, '42501', null, 'browser cannot change authoritative total');
select results_eq(
  $$ select street_address from public.listing_private_locations $$,
  array[]::text[],
  'buyer cannot read private location directly'
);

select * from finish();
rollback;
```

- [ ] **Step 2: Verify RED**

Run: `supabase test db supabase/tests/002_rls_security.sql`

Expected: policy-list assertions fail because the complete policies do not exist.

- [ ] **Step 3: Implement the exact policies**

```sql
alter table public.listing_images enable row level security;
alter table public.listing_availability enable row level security;
alter table public.payments enable row level security;
alter table public.reviews enable row level security;

create policy "public reads active listing images"
on public.listing_images for select
using (exists (
  select 1 from public.listings l
  where l.id = listing_id and l.status = 'active'
));

create policy "owners manage listing images"
on public.listing_images for all to authenticated
using (exists (
  select 1 from public.listings l
  where l.id = listing_id and l.owner_id = auth.uid()
))
with check (exists (
  select 1 from public.listings l
  where l.id = listing_id and l.owner_id = auth.uid()
));

create policy "public reads active availability"
on public.listing_availability for select
using (exists (
  select 1 from public.listings l
  where l.id = listing_id and l.status = 'active'
));

create policy "owners manage availability"
on public.listing_availability for all to authenticated
using (exists (
  select 1 from public.listings l
  where l.id = listing_id and l.owner_id = auth.uid()
))
with check (exists (
  select 1 from public.listings l
  where l.id = listing_id and l.owner_id = auth.uid()
));

drop policy if exists "participants read order amounts" on public.order_amounts;
create policy "participants read order amounts"
on public.order_amounts for select to authenticated
using (exists (
  select 1
  from public.orders o
  join public.listings l on l.id = o.listing_id
  where o.id = order_id
    and (o.buyer_id = auth.uid() or l.owner_id = auth.uid())
));

create policy "participants read payments"
on public.payments for select to authenticated
using (exists (
  select 1
  from public.orders o
  join public.listings l on l.id = o.listing_id
  where o.id = order_id
    and (o.buyer_id = auth.uid() or l.owner_id = auth.uid())
));

create policy "public reads reviews"
on public.reviews for select
using (true);

create policy "participants create reviews"
on public.reviews for insert to authenticated
with check (reviewer_id = auth.uid() and exists (
  select 1
  from public.orders o
  join public.listings l on l.id = o.listing_id
  where o.id = order_id
    and o.status = 'completed'
    and (o.buyer_id = auth.uid() or l.owner_id = auth.uid())
));

revoke insert, update, delete on public.order_amounts from anon, authenticated;
revoke insert, update, delete on public.payments from anon, authenticated;
revoke insert, update, delete on public.stripe_events from anon, authenticated;
```

Do not add any browser-role write policy to `order_amounts`, `payments`, or `stripe_events`. Policy names must exactly match the arrays asserted in the test.

- [ ] **Step 4: Verify GREEN and negative access**

Run: `supabase db reset && supabase test db supabase/tests/002_rls_security.sql`

Expected: all 11 assertions pass; protected table writes return SQLSTATE `42501`, and owner-only rows are filtered by RLS.

- [ ] **Step 5: Commit**

```bash
git add supabase/migrations/20260714000400_complete_rls.sql supabase/tests/002_rls_security.sql
git commit -m "feat: complete marketplace row level security"
```

## Task 4: Booking Conflict and Expiration Evidence

**Files:**
- Create: `supabase/tests/003_booking_conflicts.sql`
- Create: `docs/database/booking-test-matrix.md`

**Interfaces:**
- Consumes: `listing_availability`, `orders_no_overlapping_active_booking`, and 30-minute reservation semantics.
- Produces: executable overlap evidence and a presentation-ready matrix.

- [ ] **Step 1: Seed one rental fixture and write nine pgTAP cases**

Create `supabase/tests/003_booking_conflicts.sql` with the complete fixture and helper below, then add the nine assertions after it:

```sql
begin;
select plan(9);

insert into public.profiles(id, display_name, city, state) values
  ('11111111-1111-1111-1111-111111111111', 'Owner', 'Wellesley', 'MA'),
  ('22222222-2222-2222-2222-222222222222', 'Buyer One', 'Wellesley', 'MA'),
  ('33333333-3333-3333-3333-333333333333', 'Buyer Two', 'Wellesley', 'MA');

insert into public.listings(
  id, owner_id, listing_type, title, unit_price_cents,
  deposit_cents, delivery_fee_cents, status, city, state
) values (
  'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
  '11111111-1111-1111-1111-111111111111',
  'rental', 'PS5 Test Rental', 2500, 10000, 1500, 'active', 'Wellesley', 'MA'
);

insert into public.listing_availability(listing_id, start_at, end_at)
values (
  'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
  '2026-07-20T00:00:00Z', '2026-07-31T00:00:00Z'
);

create function pg_temp.seed_existing(
  state text,
  starts timestamptz,
  ends timestamptz,
  expires timestamptz default '2026-08-01T00:00:00Z'
) returns void language plpgsql as $$
begin
  delete from public.orders
  where listing_id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
  insert into public.orders(
    id, listing_id, buyer_id, status, start_at, end_at, reservation_expires_at
  ) values (
    gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    '22222222-2222-2222-2222-222222222222', state,
    starts, ends, expires
  );
end;
$$;

create function pg_temp.attempt(starts timestamptz, ends timestamptz)
returns void language sql as $$
  insert into public.orders(
    id, listing_id, buyer_id, status, start_at, end_at, reservation_expires_at
  ) values (
    gen_random_uuid(), 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    '33333333-3333-3333-3333-333333333333', 'pending_payment',
    starts, ends, '2026-08-01T00:00:00Z'
  );
$$;

select pg_temp.seed_existing('pending_payment', '2026-07-20', '2026-07-22');
select throws_ok(
  $$ select pg_temp.attempt('2026-07-20', '2026-07-22') $$,
  '23P01', null, 'identical period is rejected'
);

select pg_temp.seed_existing('pending_payment', '2026-07-20', '2026-07-22');
select throws_ok(
  $$ select pg_temp.attempt('2026-07-21', '2026-07-23') $$,
  '23P01', null, 'partial overlap is rejected'
);

select pg_temp.seed_existing('pending_payment', '2026-07-20', '2026-07-24');
select throws_ok(
  $$ select pg_temp.attempt('2026-07-21', '2026-07-22') $$,
  '23P01', null, 'contained period is rejected'
);

select pg_temp.seed_existing('pending_payment', '2026-07-21', '2026-07-22');
select throws_ok(
  $$ select pg_temp.attempt('2026-07-20', '2026-07-24') $$,
  '23P01', null, 'containing period is rejected'
);

select pg_temp.seed_existing('pending_payment', '2026-07-20', '2026-07-22');
select lives_ok(
  $$ select pg_temp.attempt('2026-07-22', '2026-07-24') $$,
  'adjacent period is allowed'
);

select pg_temp.seed_existing(
  'pending_payment', '2026-07-20', '2026-07-22', '2026-07-19'
);
update public.orders set status = 'expired'
where reservation_expires_at < '2026-07-20';
select lives_ok(
  $$ select pg_temp.attempt('2026-07-20', '2026-07-22') $$,
  'expired hold no longer blocks'
);

select pg_temp.seed_existing('cancelled', '2026-07-20', '2026-07-22');
select lives_ok(
  $$ select pg_temp.attempt('2026-07-20', '2026-07-22') $$,
  'cancelled order does not block'
);

select pg_temp.seed_existing('completed', '2026-07-20', '2026-07-22');
select lives_ok(
  $$ select pg_temp.attempt('2026-07-20', '2026-07-22') $$,
  'completed order does not block'
);

select pg_temp.seed_existing('active', '2026-07-20', '2026-07-22');
select throws_ok(
  $$ select pg_temp.attempt('2026-07-20', '2026-07-22') $$,
  '23P01', null, 'active order blocks overlap'
);

select * from finish();
rollback;
```

The test implements this matrix:

| Case | Existing | Attempt | Expected |
|---|---|---|---|
| Identical | Jul 20–22 | Jul 20–22 | reject `23P01` |
| Partial right | Jul 20–22 | Jul 21–23 | reject `23P01` |
| Contained | Jul 20–24 | Jul 21–22 | reject `23P01` |
| Contains | Jul 21–22 | Jul 20–24 | reject `23P01` |
| Adjacent | Jul 20–22 | Jul 22–24 | allow |
| Expired pending | expired Jul 20–22 | Jul 20–22 | allow after expiry update |
| Cancelled | cancelled Jul 20–22 | Jul 20–22 | allow |
| Completed | completed Jul 20–22 | Jul 20–22 | allow |
| Active | active Jul 20–22 | Jul 20–22 | reject `23P01` |

Use `throws_ok(..., '23P01', ...)` for rejected inserts and `lives_ok(...)` for allowed inserts. Reset fixture orders between cases inside the surrounding rollback transaction.

- [ ] **Step 2: Verify the booking suite**

Run: `supabase test db supabase/tests/003_booking_conflicts.sql`

Expected: all nine cases pass.

- [ ] **Step 3: Test real concurrency**

Open two `psql` sessions against the local Supabase database. In both sessions begin a transaction and attempt the same active order window. Commit session A, then commit session B.

Expected: session A commits; session B fails with exclusion violation `23P01`. Record only the SQLSTATE and outcome, not connection credentials.

- [ ] **Step 4: Document the matrix**

Create `docs/database/booking-test-matrix.md` with the nine rows above plus the concurrency result, exact test command, date, PostgreSQL version, and `PASS`/`FAIL` outcome for each row.

- [ ] **Step 5: Commit**

```bash
git add supabase/tests/003_booking_conflicts.sql docs/database/booking-test-matrix.md
git commit -m "test: prove booking conflict protection"
```

## Task 5: Order Matrix, Rust Review, and Rebuild Runbook

**Files:**
- Create: `docs/database/order-transition-matrix.md`
- Create: `docs/reviews/yichen-rust-review.md`
- Create: `docs/runbooks/database-rebuild.md`

**Interfaces:**
- Consumes: Rust state machine, repository adapter, quote engine, Stripe adapter, and all database test results.
- Produces: human review evidence without changing Lucas's Rust files.

- [ ] **Step 1: Document every state transition**

Create a table covering `pending_payment`, `paid`, `confirmed`, `active`, `fulfilled`, `returned`, `completed`, `cancelled`, and `expired`. For each state/action pair record rental, sale, or workspace applicability; next state or `REJECT`; and actor (`buyer`, `seller`, or `system`). Match `services/core-api/src/domain/order_state.rs` exactly.

- [ ] **Step 2: Review the Rust implementation**

Inspect:

```text
services/core-api/src/domain/quote.rs
services/core-api/src/domain/order_state.rs
services/core-api/src/adapters/postgres_orders.rs
services/core-api/src/adapters/stripe.rs
services/core-api/src/http/orders.rs
services/core-api/src/http/stripe_webhook.rs
```

For each issue write severity, file, reproducible input/state, actual result, expected result, and suggested ownership. If no issue is found for a category, write `PASS` with the test or source evidence; do not leave blank sections.

- [ ] **Step 3: Write the clean rebuild runbook**

Document these commands and expected results:

```bash
supabase start
supabase db reset
supabase migration list
supabase test db
cargo test --workspace
```

Expected: all migrations show as applied in timestamp order; all pgTAP and Rust tests exit `0`.

- [ ] **Step 4: Run the combined gate**

Run: `supabase db reset && supabase test db && cargo test --workspace && git diff --check`

Expected: every command exits `0` and no migration drift is reported.

- [ ] **Step 5: Commit**

```bash
git add docs/database docs/reviews/yichen-rust-review.md docs/runbooks/database-rebuild.md
git commit -m "docs: record database logic and security review"
```

## Task 6: Create the Handoff Package

**Files:**
- Create: `yicheng-handoff/README.md`
- Copy for handoff: new migrations, database tests, and the three evidence documents.

**Interfaces:**
- Consumes: all completed Yicheng files and final test output.
- Produces: one reviewable folder for Lucas without becoming a second source of truth.

- [ ] **Step 1: Create the handoff inventory**

List every source path, commit hash, purpose, verification command, result, and whether Lucas must merge or only review it. State that repository paths are canonical and handoff copies are archival.

- [ ] **Step 2: Scan for sensitive data**

Run:

```bash
rg -n '(eyJ[A-Za-z0-9_-]{20,}|sk_(test|live)_[A-Za-z0-9]{16,}|whsec_[A-Za-z0-9]{16,}|postgres(ql)?://[^ ]+:[^ ]+@)' yicheng-handoff supabase docs
```

Expected: no output.

- [ ] **Step 3: Run final verification**

Run: `supabase db reset && supabase test db && cargo test --workspace && git status --short`

Expected: all tests pass; status lists only the intended handoff files before commit.

- [ ] **Step 4: Commit the inventory**

```bash
git add yicheng-handoff/README.md
git commit -m "docs: add Yicheng handoff inventory"
```

## Completion Gate

Yicheng's work is complete only when all new migrations rebuild from an empty database, all pgTAP tests pass, concurrent overlaps yield one winner, browser roles cannot mutate protected financial data, completed-order review rules are enforced, the Rust review has evidence for every category, and the handoff contains no sensitive data.
