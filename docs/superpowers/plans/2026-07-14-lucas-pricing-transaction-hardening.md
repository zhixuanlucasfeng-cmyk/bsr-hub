# Lucas Pricing and Transaction Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explainable PS5/workspace price recommendations, server-derived rental quantity, seller ±$5 control, fulfillment validation, and exact Stripe Checkout payment verification.

**Architecture:** Pure Rust domain modules calculate recommendations and billable units without HTTP or database dependencies. PostgreSQL stores an immutable ruleset snapshot per listing, thin Axum handlers authenticate owners and translate contracts, and the existing order repository remains the transactional boundary for quote/order/payment data.

**Tech Stack:** Rust 2024, Axum 0.8, SQLx/PostgreSQL, Serde, time, Stripe test mode, Supabase, TypeScript contracts.

## Global Constraints

- Use checked `i64` integer cents; never use floating-point money.
- Minimum billed duration is 30 minutes, even when actual use is shorter.
- Billing units are `thirty_minutes` and `day`.
- Seller adjustment is inclusive `-500..=500` cents per billing unit.
- Clients never submit authoritative units, recommendation, unit price, service fee, or total.
- Default service fee stays 600 basis points.
- Only `checkout.session.completed` with `payment_status=paid`, `currency=usd`, and exact amount may pay an unexpired order.
- Do not modify Yicheng's reserved migration names `20260714000200` through `20260714000400`.
- Follow red-green-refactor and commit each completed task.

---

## Planned File Structure

```text
supabase/migrations/20260714000500_listing_pricing.sql
services/core-api/src/domain/pricing.rs
services/core-api/src/domain/billing.rs
services/core-api/src/domain/quote.rs
services/core-api/src/ports/order_repository.rs
services/core-api/src/adapters/postgres_orders.rs
services/core-api/src/adapters/stripe.rs
services/core-api/src/http/pricing.rs
services/core-api/src/http/quotes.rs
services/core-api/src/http/orders.rs
services/core-api/src/http/stripe_webhook.rs
services/core-api/tests/pricing_api.rs
services/core-api/tests/ps5_api_flow.rs
services/core-api/tests/stripe_webhook.rs
packages/contracts/src/index.ts
docs/openapi/core-api.yaml
```

## Task 1: Deterministic Recommendation Domain

**Files:**
- Create: `services/core-api/src/domain/pricing.rs`
- Modify: `services/core-api/src/domain/mod.rs`
- Test: unit tests inside `pricing.rs`

**Interfaces:**
- Produces: `BillingUnit`, `PricingCategoryInput`, `Recommendation`, `PricingError`, `recommend()`.

- [ ] **Step 1: Write failing PS5 and workspace tests**

```rust
#[test]
fn ps5_pro_like_new_is_explainable() {
    let result = recommend(PricingCategoryInput::Ps5(Ps5Input {
        model: Ps5Model::Pro,
        age_months: 0,
        condition: Condition::LikeNew,
        cleanliness: 5,
        fully_operational: true,
        missing_nonessential_features: 0,
        controller_count: 2,
        billing_unit: BillingUnit::ThirtyMinutes,
    })).unwrap();
    assert_eq!(result.recommended_unit_price_cents, 1_000);
    assert_eq!(result.minimum_allowed_cents, 500);
    assert_eq!(result.maximum_allowed_cents, 1_500);
    assert!(result.reason_codes.contains(&"MODEL_PRO".to_owned()));
}

#[test]
fn five_hundred_square_foot_suburban_workspace_has_stable_price() {
    let result = recommend(PricingCategoryInput::Workspace(WorkspaceInput {
        square_feet: 500,
        location_tier: LocationTier::Suburban,
        cleanliness: 3,
        equipment_score: 0,
        amenity_count: 0,
        billing_unit: BillingUnit::ThirtyMinutes,
    })).unwrap();
    assert_eq!(result.recommended_unit_price_cents, 1_300);
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p core-api domain::pricing --offline`

Expected: FAIL because the pricing domain does not exist.

- [ ] **Step 3: Implement enums, validation, checked basis-point math, clamp, and 50-cent rounding**

Use `round_to_50(cents) = ((cents + 25) / 50) * 50` with checked addition. Reject non-operational PS5s, input ranges outside the design, negative results, and overflow. Return `rules-v1` and reason codes.

- [ ] **Step 4: Add boundary tests**

Cover PS5 models, every condition/cleanliness value, age cap, missing-feature cap, controller cap, workspace size bounds, location tiers, equipment/amenity caps, multiplier clamps, and overflow.

- [ ] **Step 5: Verify GREEN and commit**

```bash
cargo fmt --all --check
cargo test -p core-api domain::pricing --offline
git add services/core-api/src/domain
git commit -m "feat: add explainable pricing recommendations"
```

## Task 2: Server-Derived Billing Units

**Files:**
- Create: `services/core-api/src/domain/billing.rs`
- Modify: `services/core-api/src/domain/mod.rs`
- Modify: `services/core-api/src/domain/quote.rs`
- Test: unit tests inside `billing.rs` and `quote.rs`

**Interfaces:**
- Produces: `billable_units(start, end, BillingUnit) -> Result<i64, BillingError>`.
- Changes: `QuoteInput` contains derived `units` plus `FulfillmentMethod`; only application code constructs it.

- [ ] **Step 1: Write the duration matrix test**

```rust
#[test]
fn thirty_minute_billing_rounds_up() {
    for (minutes, units) in [(1,1), (20,1), (30,1), (31,2), (50,2), (60,2), (61,3)] {
        assert_eq!(units_for_minutes(minutes, BillingUnit::ThirtyMinutes).unwrap(), units);
    }
}

#[test]
fn daily_billing_rounds_up() {
    assert_eq!(units_for_seconds(1, BillingUnit::Day).unwrap(), 1);
    assert_eq!(units_for_seconds(86_400, BillingUnit::Day).unwrap(), 1);
    assert_eq!(units_for_seconds(86_401, BillingUnit::Day).unwrap(), 2);
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p core-api domain::billing --offline`

Expected: FAIL because billing functions are absent.

- [ ] **Step 3: Implement ceiling division**

Reject `end <= start`; compute seconds with checked subtraction; use `(seconds + divisor - 1) / divisor`; return at least one unit.

- [ ] **Step 4: Update quote output**

Add `unit_price_cents`, `billable_units`, and `billing_unit` to `QuoteBreakdown`. Replace `wants_delivery` with `fulfillment`; delivery fee is included only for `FulfillmentMethod::Delivery`.

- [ ] **Step 5: Verify and commit**

```bash
cargo test -p core-api domain::billing domain::quote --offline
git add services/core-api/src/domain
git commit -m "feat: derive rental billing from time"
```

## Task 3: Pricing Profile Migration and Repository

**Files:**
- Create: `supabase/migrations/20260714000500_listing_pricing.sql`
- Modify: `services/core-api/src/ports/order_repository.rs`
- Modify: `services/core-api/src/adapters/postgres_orders.rs`

**Interfaces:**
- Produces: `SavePricingProfile`, `StoredPricingProfile`, `save_pricing_profile()`, enriched `pricing()`.

- [ ] **Step 1: Define the migration**

```sql
create table public.listing_pricing_profiles (
  listing_id uuid primary key references public.listings(id) on delete cascade,
  category text not null check (category in ('ps5', 'workspace')),
  billing_unit text not null check (billing_unit in ('thirty_minutes', 'day')),
  attributes jsonb not null,
  ruleset_version text not null check (ruleset_version = 'rules-v1'),
  recommended_unit_price_cents bigint not null check (recommended_unit_price_cents >= 0),
  seller_adjustment_cents bigint not null check (seller_adjustment_cents between -500 and 500),
  final_unit_price_cents bigint generated always as
    (recommended_unit_price_cents + seller_adjustment_cents) stored,
  allowed_fulfillment_methods text[] not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (recommended_unit_price_cents + seller_adjustment_cents >= 0),
  check (cardinality(allowed_fulfillment_methods) > 0)
);

alter table public.listing_pricing_profiles enable row level security;
revoke insert, update, delete on public.listing_pricing_profiles from anon, authenticated;
```

- [ ] **Step 2: Write a repository contract compile test**

Update the API fake to implement saving and loading a profile, then run `cargo test -p core-api --test pricing_api --offline` and confirm it fails until the trait exists.

- [ ] **Step 3: Implement transactional owner-checked upsert**

Lock the active listing, compare `owner_id`, serialize validated attributes, and upsert every server-calculated field. Loading joins `listings` and `listing_pricing_profiles` and returns deposit, delivery, service fee, billing unit, and allowed fulfillment methods.

- [ ] **Step 4: Compile and commit**

```bash
cargo test -p core-api --no-run --offline
git add supabase/migrations/20260714000500_listing_pricing.sql services/core-api/src/ports services/core-api/src/adapters/postgres_orders.rs
git commit -m "feat: persist authoritative pricing profiles"
```

## Task 4: Owner Pricing API

**Files:**
- Create: `services/core-api/src/http/pricing.rs`
- Modify: `services/core-api/src/http/mod.rs`
- Modify: `services/core-api/src/lib.rs`
- Test: `services/core-api/tests/pricing_api.rs`

**Interfaces:**
- Produces: `PUT /v1/listings/{id}/pricing`.

- [ ] **Step 1: Write failing API tests**

Test missing auth `401`, non-owner `403`, valid PS5 recommendation `200`, adjustment `-501/501` as `422`, invalid workspace inputs `422`, and a response containing recommendation, min/max, final price, billing unit, ruleset, and reasons.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p core-api --test pricing_api --offline`

Expected: route returns `404`.

- [ ] **Step 3: Implement strict tagged requests and handler**

Use `#[serde(tag = "category", rename_all = "snake_case", deny_unknown_fields)]` category inputs. Authenticate, call `recommend`, apply the bounded seller adjustment, validate category-compatible fulfillment, and save through the repository.

- [ ] **Step 4: Verify and commit**

```bash
cargo test -p core-api --test pricing_api --offline
git add services/core-api/src/http services/core-api/src/lib.rs services/core-api/tests/pricing_api.rs
git commit -m "feat: expose owner pricing recommendations"
```

## Task 5: Harden Quote and Order Contracts

**Files:**
- Modify: `services/core-api/src/http/quotes.rs`
- Modify: `services/core-api/src/http/orders.rs`
- Modify: `services/core-api/tests/ps5_api_flow.rs`
- Modify: `packages/contracts/src/index.ts`

**Interfaces:**
- Consumes: stored pricing profile and `billable_units()`.
- Produces: quote/order requests without `units` or `wantsDelivery`.

- [ ] **Step 1: Rewrite the failing PS5 flow**

Submit `startAt`, `endAt`, and `fulfillment=delivery`; assert 20 minutes produces one unit, 31 minutes produces two, quote and order totals match, and unknown `units` causes `422` under strict deserialization.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p core-api --test ps5_api_flow --offline`

Expected: old handlers require client units and do not return billing details.

- [ ] **Step 3: Implement one shared application quote function**

Parse RFC3339 once, load the profile, reject disallowed fulfillment, derive units, and call the pure quote engine. Both quote and order handlers must call this same function.

- [ ] **Step 4: Update TypeScript contracts**

Remove `units` and `wantsDelivery` from requests. Add billing unit, billable units, unit price, recommendation fields, pricing category inputs, and stable new error codes.

- [ ] **Step 5: Verify and commit**

```bash
cargo test -p core-api --test ps5_api_flow --offline
git add services/core-api/src/http services/core-api/tests/ps5_api_flow.rs packages/contracts/src/index.ts
git commit -m "feat: make rental quotes time authoritative"
```

## Task 6: Exact Stripe Checkout Verification

**Files:**
- Modify: `services/core-api/src/adapters/stripe.rs`
- Modify: `services/core-api/src/ports/payment_gateway.rs`
- Modify: `services/core-api/src/ports/order_repository.rs`
- Modify: `services/core-api/src/adapters/postgres_orders.rs`
- Modify: `services/core-api/src/http/stripe_webhook.rs`
- Modify: `services/core-api/tests/stripe_webhook.rs`

**Interfaces:**
- Produces: `VerifiedCheckoutEvent { event_id, order_id, amount_total_cents, currency, paid }` and `PaymentEventOutcome`.

- [ ] **Step 1: Write the payment matrix tests**

Cover valid paid checkout, unrelated signed event, unpaid checkout, wrong currency, wrong amount, missing order, expired reservation, duplicate event, and correct event. Rename `payment_intent_id` to `checkout_session_id` in Checkout response tests.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p core-api --test stripe_webhook --offline`

Expected: parsed event lacks payment status, currency, and amount.

- [ ] **Step 3: Parse Checkout Session fields and harden the transaction**

After event-ID insert, lock the order and immutable amount row. Apply `paid` only when status is `pending_payment`, hold is unexpired, currency is USD, and amount matches. Record ignored/mismatch audit events and return HTTP 200 for signed but non-actionable events.

- [ ] **Step 4: Verify and commit**

```bash
cargo test -p core-api --test stripe_webhook --offline
cargo test -p core-api --offline
git add services/core-api/src services/core-api/tests/stripe_webhook.rs
git commit -m "fix: verify Stripe checkout before payment transition"
```

## Task 7: Contracts, Documentation, and Release Gate

**Files:**
- Modify: `docs/openapi/core-api.yaml`
- Modify: `docs/runbooks/core-api.md`
- Modify: `docs/runbooks/demo-smoke-test.md`
- Modify: `README.md`

- [ ] **Step 1: Update OpenAPI and demo examples**

Document pricing profile requests, billing fields, fulfillment errors, 20-minute minimum charge, seller ±$5 range, and exact Stripe acceptance rules.

- [ ] **Step 2: Run the final gate**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
git diff --check
```

Expected: every command exits `0`.

- [ ] **Step 3: Scan for secrets**

```bash
rg -n '(sk_live_|sk_test_[A-Za-z0-9]{20,}|whsec_[A-Za-z0-9]{20,}|postgres(ql)?://[^ ]+:[^ ]+@)' . -g '!.git/**' -g '!Cargo.lock'
```

Expected: no real credential matches.

- [ ] **Step 4: Commit**

```bash
git add README.md docs
git commit -m "docs: explain pricing and payment safeguards"
```

## Completion Gate

Lucas's pricing hardening is complete only when recommendation coefficients are fully tested, 20 minutes bills one 30-minute unit, the browser cannot control units or totals, seller adjustments outside ±$5 fail, fulfillment is validated, quote/order totals match, wrong Stripe events cannot pay orders, and the complete quality gate passes.
