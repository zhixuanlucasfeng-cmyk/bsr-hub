# Lucas Pricing and Transaction Hardening Design

**Date:** 2026-07-14  
**Owner:** Lucas  
**Status:** Approved direction; coefficients frozen for MVP review

## Objective

Build a transparent Rust pricing engine for PS5 rentals and workspaces, allow sellers to move the recommendation by at most five U.S. dollars per billing unit, derive rental quantity from server-validated timestamps, and prevent an unrelated or incorrect Stripe event from marking an order paid.

The MVP uses deterministic rules. A later image model may suggest condition and cleanliness inputs, but Rust remains authoritative and applies the same bounds regardless of where the attributes came from.

## User Rules

- A booking may be shorter than 30 minutes, including 20 minutes, but the minimum charge is 30 minutes.
- Sellers choose either `thirty_minutes` or `day` as the listing billing unit.
- Thirty-minute listings charge `ceil(duration_seconds / 1800)` units.
- Daily listings charge `ceil(duration_seconds / 86400)` units.
- Every valid booking charges at least one unit.
- The client never submits authoritative quantity or total.
- The seller adjustment is an integer from `-500` through `500` cents per billing unit.
- The final unit price is `recommended_unit_price_cents + seller_adjustment_cents` and may not be negative.
- The service fee remains server-controlled at 600 basis points by default.
- All calculations use checked signed 64-bit integer cents; no floating-point money is permitted.

## Supported Recommendation Categories

Version 1 recommends prices for `ps5` and `workspace`. Other rentals and second-hand sales continue to use their existing listing price until a category-specific ruleset is approved. The API returns `PRICING_CATEGORY_UNSUPPORTED` instead of guessing for an unsupported recommendation category.

## PS5 Recommendation

### Inputs

- `model`: `original`, `slim`, or `pro`.
- `age_months`: integer from 0 through 120.
- `condition`: `like_new`, `good`, `fair`, or `worn`.
- `cleanliness`: integer from 1 through 5.
- `fully_operational`: must be `true`; unsafe or non-operational consoles are rejected.
- `missing_nonessential_features`: integer from 0 through 3.
- `controller_count`: integer from 1 through 4.
- `billing_unit`: `thirty_minutes` or `day`.

### Base prices

| Model | Per 30 minutes | Per day |
|---|---:|---:|
| Original | $5.00 | $25.00 |
| Slim | $6.00 | $30.00 |
| Pro | $8.00 | $40.00 |

These are demo coefficients, not a claim of live U.S. market value. They are versioned as `rules-v1` and may be updated after Lucian supplies comparable-market evidence.

### Adjustments

- Age: subtract 500 basis points for each complete 12 months, capped at 2,000 basis points.
- Condition: `like_new +1000`, `good +0`, `fair -1500`, `worn -3000` basis points.
- Cleanliness: score 1 `-1000`, 2 `-500`, 3 `0`, 4 `+300`, 5 `+500` basis points.
- Missing nonessential features: subtract 1,500 basis points each, capped at 4,500.
- Extra controllers beyond one: add 100 cents per 30-minute unit or 500 cents per daily unit, capped at two extra controllers.
- Clamp the combined percentage multiplier between 4,000 and 13,000 basis points.
- Apply the multiplier to the base, add the controller amount, and round to the nearest 50 cents using integer arithmetic.

The response includes the recommendation, the `$-5/$+5` range, ruleset version, and reason codes such as `MODEL_PRO`, `AGE_24_MONTHS`, `CONDITION_FAIR`, and `EXTRA_CONTROLLER`.

## Workspace Recommendation

### Inputs

- `square_feet`: integer from 50 through 20,000.
- `location_tier`: `residential`, `suburban`, `urban`, or `premium`.
- `cleanliness`: integer from 1 through 5.
- `equipment_score`: integer from 0 through 5.
- `amenity_count`: integer from 0 through 10.
- `billing_unit`: `thirty_minutes` or `day`.

### Formula

1. Calculate the 30-minute base as `300 + square_feet * 2` cents.
2. For daily billing, multiply that base by 12, representing a six-hour discounted day.
3. Apply location: residential `-1000`, suburban `0`, urban `+1500`, premium `+2500` basis points.
4. Apply the same cleanliness table used for PS5s.
5. Add 300 basis points per equipment score point, capped at 1,500.
6. Add 200 basis points per amenity, capped at 1,000.
7. Clamp the combined multiplier between 5,000 and 15,000 basis points.
8. Apply it with checked integer arithmetic and round to the nearest 50 cents.

The response provides reason codes for size, location, cleanliness, equipment, and amenities so the seller understands the recommendation.

## Persistence

Create `supabase/migrations/20260714000500_listing_pricing.sql`. The higher timestamp avoids colliding with Yicheng's reserved `00200` through `00400` migrations.

Add `listing_pricing_profiles`:

- `listing_id` primary key referencing `listings` with cascade deletion;
- `category` restricted to `ps5` or `workspace`;
- `billing_unit` restricted to `thirty_minutes` or `day`;
- `attributes` JSONB containing the validated category inputs;
- `ruleset_version` fixed to `rules-v1` for this release;
- `recommended_unit_price_cents` non-negative bigint;
- `seller_adjustment_cents` bigint checked from `-500` through `500`;
- `final_unit_price_cents` generated from recommendation plus adjustment and checked non-negative;
- `allowed_fulfillment_methods` text array restricted by the Rust input enum;
- timestamps.

Anonymous and authenticated browser roles receive no insert, update, or delete policy. The Rust service authenticates the listing owner, recalculates the recommendation, and performs the upsert through its trusted database connection.

## API Changes

### Save or update pricing

`PUT /v1/listings/{listing_id}/pricing`

Requires a Supabase bearer token and listing ownership. The request contains category, billing unit, category attributes, seller adjustment, and allowed fulfillment methods. It never accepts the recommendation itself. Rust recalculates and persists it.

The response contains:

- `recommendedUnitPriceCents`;
- `sellerAdjustmentCents`;
- `finalUnitPriceCents`;
- `minimumAllowedCents`;
- `maximumAllowedCents`;
- `billingUnit`;
- `rulesetVersion`;
- `reasonCodes`.

### Quote

`POST /v1/quotes` removes client-controlled `units` and `wantsDelivery`. It accepts `listingId`, `startAt`, `endAt`, and `fulfillment`.

Rust loads the stored pricing profile, verifies that fulfillment is permitted, calculates billable units from the timestamps, and returns quantity, billing unit, unit price, base, service fee, delivery, deposit, total, and `USD`.

### Order creation

`POST /v1/orders` accepts the same time and fulfillment fields as quote. It recalculates the quote from the stored profile inside the server flow. A client-provided recommendation, unit price, units, fee, total, success URL, or cancellation URL is ignored by contract and rejected when supplied under strict request deserialization.

## Fulfillment

Allowed values remain `pickup`, `delivery`, `owner_location`, and `on_site`.

- PS5 listings may allow pickup, delivery, or owner-location use.
- Workspace listings must allow on-site use and may not use delivery.
- A delivery fee is charged only when `delivery` is selected.
- Selecting a method absent from the stored pricing profile returns `FULFILLMENT_NOT_ALLOWED` with HTTP 422.

## Stripe Payment Hardening

The current raw-body signature verification and event-ID idempotency remain. Before changing an order to paid, Rust must also verify:

- event type is exactly `checkout.session.completed`;
- Checkout Session `payment_status` is `paid`;
- metadata contains a valid `order_id`;
- currency is `usd`;
- `amount_total` exactly equals immutable `order_amounts.total_cents`;
- the order is still `pending_payment` and its reservation has not expired.

Unrelated signed events return HTTP 200 as ignored so Stripe does not retry forever. A completed event with the wrong currency, amount, order, or expired reservation is recorded for idempotency but does not transition the order; it produces a safe audit event without exposing financial data. Replayed event IDs return HTTP 200 without a second state change.

Rename the misleading `payment_intent_id` field in the Checkout response to `checkout_session_id`, because Stripe returns a Checkout Session ID beginning with `cs_`.

## Error Handling

New stable codes are `PRICING_CATEGORY_UNSUPPORTED`, `INVALID_PRICING_ATTRIBUTES`, `SELLER_ADJUSTMENT_OUT_OF_RANGE`, `PRICING_PROFILE_NOT_FOUND`, `FULFILLMENT_NOT_ALLOWED`, `INVALID_RENTAL_WINDOW`, and `PAYMENT_MISMATCH`. Errors retain the existing envelope and request ID. No attributes, tokens, SQL, Stripe payloads, or private addresses are logged in user-facing messages.

## Testing

- Unit-test every PS5 coefficient, workspace coefficient, clamp, rounding boundary, and arithmetic overflow.
- Test 1, 20, 30, 31, 50, 60, and 61 minutes for 30-minute billing.
- Test sub-day, exact-day, and over-day daily billing.
- Test seller adjustments at `-501`, `-500`, `0`, `500`, and `501` cents.
- API-test that client quantity is no longer accepted and that quote/order produce identical totals.
- Repository-test ownership, profile persistence, and immutable server recommendations.
- Webhook-test unrelated event types, unpaid sessions, wrong order, wrong currency, wrong amount, expired order, correct payment, and duplicate delivery.
- Run formatting, Clippy with warnings denied, the complete Rust suite, migration rebuild, and database tests before merge.

## Scope Boundary

This release does not call a paid AI service, inspect images, scrape live prices, predict demand, perform real seller payouts, or recommend prices for categories beyond PS5 and workspace. The domain types and `ruleset_version` preserve a clean future path for AI-assisted attribute extraction and new category rules.

## Completion Criteria

The work is complete when Rust—not the browser—derives billable quantity and price; 20-minute use bills one 30-minute unit; daily time rounds upward; seller adjustment cannot exceed five dollars; PS5 and workspace recommendations return explainable reasons; quote and order totals match; only a correct paid Checkout Session with an exact amount can mark an unexpired order paid; and all automated quality gates pass.
