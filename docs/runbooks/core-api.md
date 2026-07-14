# Core API runbook

## Local setup

1. Create a Supabase project and apply every SQL file in `supabase/migrations` in filename order through `20260714000500_listing_pricing.sql`.
2. Copy `.env.example` to `.env.local` and fill only test/development values.
3. Export the variables in your shell, then run `cargo run -p core-api`.
4. Check `curl http://127.0.0.1:8080/health`.

The service refuses to start when required configuration is absent. Secret values are never included in API errors.

## Required environment variables

`DATABASE_URL`, `SUPABASE_URL`, `SUPABASE_ANON_KEY`, `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `WEB_SUCCESS_URL`, `WEB_CANCEL_URL`, and `ALLOWED_ORIGIN` are required. `SERVICE_FEE_BPS`, `RESERVATION_MINUTES`, and `PORT` default to `600`, `30`, and `8080`.

Use only Stripe test keys beginning with `sk_test_`. Register `/v1/stripe/webhook` as the test webhook endpoint and subscribe to `checkout.session.completed`. BSR Hub ignores other signed event types and unpaid Checkout Sessions. A payment is accepted only when its order ID, total amount, USD currency, and unexpired reservation all match the stored order.

## Pricing and billing contract

The listing owner saves pricing through `PUT /v1/listings/{id}/pricing`. The Rust service calculates the recommendation from PS5 or workspace attributes; the seller may submit `sellerAdjustmentCents` only from `-500` through `500`. PS5 listings may use pickup, delivery, or owner location. Workspaces are immovable and must use `on_site`.

Quote and order requests send `listingId`, RFC 3339 `startAt`/`endAt`, and `fulfillment`. They never send price or billable units. The server rounds any partial 30-minute or daily unit upward and returns the authoritative breakdown.

## Release gate

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

Deploy from `render.yaml`, configure secrets in Render rather than Git, then verify the returned hostname at `/health`.
