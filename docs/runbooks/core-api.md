# Core API runbook

## Local setup

1. Start Docker Desktop and run `npm run mongo:up` for the local MongoDB replica set.
2. Copy `.env.mongodb.example` to `.env.local` and fill only test/development values.
3. Export the variables, run `npm run mongo:bootstrap`, then run `cargo run -p core-api`.
4. Check liveness at `curl http://127.0.0.1:8080/health` and database readiness at `curl http://127.0.0.1:8080/ready`.

The service refuses to start when required configuration is absent. Secret values are never included in API errors.

## Required environment variables

`MONGODB_URI`, `MONGODB_DATABASE`, `SUPABASE_URL`, `SUPABASE_ANON_KEY`, `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `WEB_SUCCESS_URL`, `WEB_CANCEL_URL`, and `ALLOWED_ORIGIN` are required. `SERVICE_FEE_BPS`, `RESERVATION_MINUTES`, and `PORT` default to `600`, `30`, and `8080`.

For MongoDB Atlas, store `MONGODB_URI` only in the backend host's secret manager, require TLS, and add only the backend's outbound address to the Atlas network access list. GitHub Pages cannot run the Rust service and must call a separately hosted HTTPS API. Never place the URI in `NEXT_PUBLIC_*` variables.

`BSR_DEMO_MODE=true` keeps the fictional in-memory classroom flow available without MongoDB. Persistent mode is for backend development and uses only fictional seed data until the project has completed legal, privacy, payment, and identity work.

Use only Stripe test keys beginning with `sk_test_`. Register `/v1/stripe/webhook` as the test webhook endpoint and subscribe to `checkout.session.completed`. BSR Hub ignores other signed event types and unpaid Checkout Sessions. A payment is accepted only when its order ID, total amount, USD currency, and unexpired reservation all match the stored order.

## Pricing and billing contract

The listing owner saves pricing through `PUT /v1/listings/{id}/pricing`. The Rust service calculates the recommendation from PS5 or workspace attributes; the seller may submit `sellerAdjustmentCents` only from `-500` through `500`. PS5 listings may use pickup, delivery, or owner location. Workspaces are immovable and must use `on_site`.

Quote and order requests send `listingId`, RFC 3339 `startAt`/`endAt`, and `fulfillment`. They never send price or billable units. The server rounds any partial 30-minute or daily unit upward and returns the authoritative breakdown.

## Release gate

For the complete MongoDB and repository gate, start Docker Desktop and run:

```bash
npm run mongo:check
```

This starts a local replica set, bootstraps it twice, runs the real pricing, concurrent-reservation, payment-idempotency, and state-transition integration tests, then runs all Rust and frontend checks. The database volume is preserved after the container stops.

The individual non-Docker checks are:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

Deploy the container to a Rust-capable host, configure secrets there rather than in Git, then verify `/health` and `/ready`. A failed `/ready` returns a stable `DATABASE_UNAVAILABLE` response without exposing the URI or driver details.
