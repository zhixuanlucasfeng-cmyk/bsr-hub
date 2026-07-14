# Lucas Core Platform Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the reproducible BSR Hub monorepo foundation and a tested Rust/Axum core API for health, quotes, order reservations, state transitions, and Stripe test-mode integration.

**Architecture:** Next.js remains the web client, Supabase provides PostgreSQL/Auth/Storage, and Rust owns authoritative transaction rules. The Rust service uses domain modules that do not depend on HTTP, ports for persistence and payments, and thin Axum handlers. The first integration spine is a PS5 rental with a 30-minute pending-payment reservation and a server-calculated 6% configurable service fee.

**Tech Stack:** Rust stable, Cargo edition 2024, Axum 0.8, Tokio, SQLx/PostgreSQL, Serde, UUID, time, reqwest, HMAC-SHA256, Next.js/TypeScript contract package, Supabase, Stripe test mode, Render, Vercel.

## Global Constraints

- English UI, U.S. dollars, and U.S. address formats.
- Store all money as signed 64-bit integer cents; reject negative amounts.
- Configure the initial service fee as `600` basis points; clients cannot override it.
- A pending-payment reservation expires after 30 minutes.
- Public listing data never contains a private street address.
- The browser never calculates the authoritative total.
- Stripe is test mode only; refund, deposit, and delayed payout are simulated states in the MVP.
- Never commit `.env`, API keys, access tokens, private addresses, or Stripe secrets.
- Every implementation task follows red-green-refactor and ends with a focused commit.

---

## Planned File Structure

```text
Cargo.toml
package.json
.env.example
apps/web/README.md
packages/contracts/package.json
packages/contracts/src/index.ts
services/core-api/Cargo.toml
services/core-api/Dockerfile
services/core-api/src/main.rs
services/core-api/src/lib.rs
services/core-api/src/config.rs
services/core-api/src/error.rs
services/core-api/src/auth.rs
services/core-api/src/domain/mod.rs
services/core-api/src/domain/quote.rs
services/core-api/src/domain/order_state.rs
services/core-api/src/ports/mod.rs
services/core-api/src/ports/order_repository.rs
services/core-api/src/ports/payment_gateway.rs
services/core-api/src/adapters/mod.rs
services/core-api/src/adapters/postgres_orders.rs
services/core-api/src/adapters/stripe.rs
services/core-api/src/http/mod.rs
services/core-api/src/http/health.rs
services/core-api/src/http/quotes.rs
services/core-api/src/http/orders.rs
services/core-api/src/http/stripe_webhook.rs
services/core-api/tests/health_api.rs
services/core-api/tests/order_reservation.rs
services/core-api/tests/stripe_webhook.rs
render.yaml
```

## Task 1: Reproducible Workspace and Health Endpoint

**Files:**
- Create: `Cargo.toml`
- Create: `package.json`
- Create: `.env.example`
- Create: `apps/web/README.md`
- Create: `services/core-api/Cargo.toml`
- Create: `services/core-api/src/lib.rs`
- Create: `services/core-api/src/main.rs`
- Create: `services/core-api/src/http/mod.rs`
- Create: `services/core-api/src/http/health.rs`
- Test: `services/core-api/tests/health_api.rs`

**Interfaces:**
- Produces: `core_api::app() -> axum::Router`
- Produces: `GET /health -> 200 {"status":"ok"}`

- [ ] **Step 1: Create the workspace manifests**

```toml
# Cargo.toml
[workspace]
members = ["services/core-api"]
resolver = "3"
```

```json
{
  "name": "bsr-hub",
  "private": true,
  "workspaces": ["apps/*", "packages/*"],
  "scripts": {
    "check:rust": "cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace"
  }
}
```

```toml
# services/core-api/Cargo.toml
[package]
name = "core-api"
version = "0.1.0"
edition = "2024"

[dependencies]
async-trait = "0.1"
axum = "0.8"
hex = "0.4"
hmac = "0.12"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "time"] }
subtle = "2"
thiserror = "2"
time = { version = "0.3", features = ["serde", "parsing"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
uuid = { version = "1", features = ["serde", "v4"] }

[dev-dependencies]
http-body-util = "0.1"
tower = { version = "0.5", features = ["util"] }
wiremock = "0.6"
```

```dotenv
# .env.example
DATABASE_URL=
SUPABASE_URL=
SUPABASE_ANON_KEY=
SUPABASE_JWT_AUDIENCE=authenticated
STRIPE_SECRET_KEY=
STRIPE_WEBHOOK_SECRET=
WEB_SUCCESS_URL=http://localhost:3000/orders/success
WEB_CANCEL_URL=http://localhost:3000/orders/cancelled
SERVICE_FEE_BPS=600
RESERVATION_MINUTES=30
ALLOWED_ORIGIN=http://localhost:3000
PORT=8080
```

```markdown
<!-- apps/web/README.md -->
# BSR Hub Web

Anna and Nasia own the Next.js implementation. The web app consumes types from `packages/contracts` and treats Rust quote/order responses as authoritative.
```

- [ ] **Step 2: Write the failing health API test**

```rust
// services/core-api/tests/health_api.rs
use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok_json() {
    let response = core_api::app()
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], br#"{"status":"ok"}"#);
}
```

- [ ] **Step 3: Run the test and verify the expected failure**

Run: `cargo test -p core-api --test health_api`  
Expected: FAIL because `services/core-api/Cargo.toml` and `core_api::app` are not implemented.

- [ ] **Step 4: Implement the minimal Axum service**

```rust
// services/core-api/src/lib.rs
pub mod config;
pub mod http;

use axum::Router;

pub fn app() -> Router {
    Router::new().merge(http::routes())
}
```

```rust
// services/core-api/src/http/mod.rs
mod health;

use axum::Router;

pub fn routes() -> Router {
    Router::new().route("/health", axum::routing::get(health::get))
}
```

```rust
// services/core-api/src/http/health.rs
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

pub async fn get() -> Json<Health> {
    Json(Health { status: "ok" })
}
```

```rust
// services/core-api/src/main.rs
#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, core_api::app()).await.unwrap();
}
```

- [ ] **Step 5: Run the focused and workspace checks**

Run: `cargo test -p core-api --test health_api && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings`  
Expected: health test PASS; formatting and Clippy exit `0`.

- [ ] **Step 6: Commit the foundation**

```bash
git add Cargo.toml package.json .env.example apps/web services/core-api
git commit -m "feat: scaffold BSR Hub core API"
```

## Task 2: Integer Money and Authoritative Quote Engine

**Files:**
- Create: `services/core-api/src/domain/mod.rs`
- Create: `services/core-api/src/domain/quote.rs`
- Modify: `services/core-api/src/lib.rs`
- Create: `packages/contracts/package.json`
- Create: `packages/contracts/src/index.ts`

**Interfaces:**
- Produces: `calculate_quote(PricingSnapshot, QuoteInput) -> Result<QuoteBreakdown, QuoteError>`
- Produces TypeScript: `QuoteRequest`, `QuoteBreakdown`, and `ApiError`

- [ ] **Step 1: Write failing quote tests**

```rust
// inside services/core-api/src/domain/quote.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rental_quote_uses_integer_cents_and_six_percent_fee() {
        let pricing = PricingSnapshot {
            unit_price_cents: 2_500,
            deposit_cents: 10_000,
            delivery_fee_cents: 1_500,
            service_fee_bps: 600,
        };
        let quote = calculate_quote(pricing, QuoteInput { units: 2, wants_delivery: true }).unwrap();
        assert_eq!(quote.base_cents, 5_000);
        assert_eq!(quote.service_fee_cents, 300);
        assert_eq!(quote.delivery_fee_cents, 1_500);
        assert_eq!(quote.deposit_cents, 10_000);
        assert_eq!(quote.total_cents, 16_800);
    }

    #[test]
    fn zero_units_are_rejected() {
        let result = calculate_quote(
            PricingSnapshot { unit_price_cents: 2_500, deposit_cents: 0, delivery_fee_cents: 0, service_fee_bps: 600 },
            QuoteInput { units: 0, wants_delivery: false },
        );
        assert_eq!(result, Err(QuoteError::InvalidUnits));
    }
}
```

- [ ] **Step 2: Run the tests and verify failure**

Run: `cargo test -p core-api domain::quote::tests`  
Expected: FAIL because the domain types and `calculate_quote` do not exist.

- [ ] **Step 3: Implement the minimal quote domain**

```rust
// services/core-api/src/domain/quote.rs
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct PricingSnapshot {
    pub unit_price_cents: i64,
    pub deposit_cents: i64,
    pub delivery_fee_cents: i64,
    pub service_fee_bps: i64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct QuoteInput {
    pub units: i64,
    pub wants_delivery: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteBreakdown {
    pub base_cents: i64,
    pub service_fee_cents: i64,
    pub delivery_fee_cents: i64,
    pub deposit_cents: i64,
    pub total_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum QuoteError {
    #[error("units must be positive")]
    InvalidUnits,
    #[error("pricing values must be non-negative")]
    NegativeAmount,
    #[error("quote arithmetic overflow")]
    Overflow,
}

pub fn calculate_quote(pricing: PricingSnapshot, input: QuoteInput) -> Result<QuoteBreakdown, QuoteError> {
    if input.units <= 0 { return Err(QuoteError::InvalidUnits); }
    if [pricing.unit_price_cents, pricing.deposit_cents, pricing.delivery_fee_cents, pricing.service_fee_bps].iter().any(|v| *v < 0) {
        return Err(QuoteError::NegativeAmount);
    }
    let base = pricing.unit_price_cents.checked_mul(input.units).ok_or(QuoteError::Overflow)?;
    let fee = base.checked_mul(pricing.service_fee_bps).ok_or(QuoteError::Overflow)? / 10_000;
    let delivery = if input.wants_delivery { pricing.delivery_fee_cents } else { 0 };
    let total = base.checked_add(fee).and_then(|v| v.checked_add(delivery)).and_then(|v| v.checked_add(pricing.deposit_cents)).ok_or(QuoteError::Overflow)?;
    Ok(QuoteBreakdown { base_cents: base, service_fee_cents: fee, delivery_fee_cents: delivery, deposit_cents: pricing.deposit_cents, total_cents: total })
}
```

- [ ] **Step 4: Define the exact TypeScript contract**

```ts
// packages/contracts/src/index.ts
export type FulfillmentMethod = "pickup" | "delivery" | "owner_location" | "on_site";

export interface QuoteRequest {
  listingId: string;
  startAt: string | null;
  endAt: string | null;
  fulfillment: FulfillmentMethod;
}

export interface QuoteBreakdown {
  baseCents: number;
  serviceFeeCents: number;
  deliveryFeeCents: number;
  depositCents: number;
  totalCents: number;
  currency: "USD";
}

export interface ApiError {
  error: { code: string; message: string; requestId: string };
}
```

- [ ] **Step 5: Verify all domain tests and commit**

Run: `cargo test -p core-api domain::quote && cargo clippy -p core-api --all-targets -- -D warnings`  
Expected: all quote tests PASS and Clippy exits `0`.

```bash
git add services/core-api/src/domain packages/contracts services/core-api/src/lib.rs
git commit -m "feat: add authoritative quote calculation"
```

## Task 3: Order-State Machine

**Files:**
- Create: `services/core-api/src/domain/order_state.rs`
- Modify: `services/core-api/src/domain/mod.rs`

**Interfaces:**
- Produces: `OrderState` enum
- Produces: `OrderState::transition(self, OrderAction) -> Result<OrderState, TransitionError>`

- [ ] **Step 1: Write the transition matrix tests**

```rust
#[test]
fn paid_order_can_be_confirmed() {
    assert_eq!(OrderState::Paid.transition(OrderAction::Confirm), Ok(OrderState::Confirmed));
}

#[test]
fn completed_order_cannot_be_cancelled() {
    assert_eq!(OrderState::Completed.transition(OrderAction::Cancel), Err(TransitionError::Invalid));
}

#[test]
fn unpaid_order_can_expire() {
    assert_eq!(OrderState::PendingPayment.transition(OrderAction::Expire), Ok(OrderState::Expired));
}
```

- [ ] **Step 2: Run and verify failure**

Run: `cargo test -p core-api domain::order_state`  
Expected: FAIL because `OrderState`, `OrderAction`, and `transition` are absent.

- [ ] **Step 3: Implement the explicit state machine**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderState { PendingPayment, Paid, Confirmed, Active, Fulfilled, Returned, Completed, Cancelled, Expired }

#[derive(Debug, Clone, Copy)]
pub enum OrderAction { MarkPaid, Confirm, Activate, Fulfill, Return, Complete, Cancel, Expire }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionError { Invalid }

impl OrderState {
    pub fn transition(self, action: OrderAction) -> Result<Self, TransitionError> {
        use OrderAction::*;
        use OrderState::*;
        match (self, action) {
            (PendingPayment, MarkPaid) => Ok(Paid),
            (PendingPayment, Expire) => Ok(Expired),
            (PendingPayment | Paid | Confirmed, Cancel) => Ok(Cancelled),
            (Paid, Confirm) => Ok(Confirmed),
            (Confirmed, Activate) => Ok(Active),
            (Confirmed, Fulfill) => Ok(Fulfilled),
            (Active, Return) => Ok(Returned),
            (Returned | Fulfilled, Complete) => Ok(Completed),
            _ => Err(TransitionError::Invalid),
        }
    }
}
```

- [ ] **Step 4: Run the domain suite and commit**

Run: `cargo test -p core-api domain::order_state`  
Expected: all transition tests PASS.

```bash
git add services/core-api/src/domain
git commit -m "feat: enforce order state transitions"
```

## Task 4: Authenticated Order Reservation Port and PostgreSQL Adapter

**Files:**
- Create: `services/core-api/src/auth.rs`
- Create: `services/core-api/src/ports/mod.rs`
- Create: `services/core-api/src/ports/order_repository.rs`
- Create: `services/core-api/src/adapters/mod.rs`
- Create: `services/core-api/src/adapters/postgres_orders.rs`
- Create: `services/core-api/tests/support/mod.rs`
- Test: `services/core-api/tests/order_reservation.rs`

**Interfaces:**
- Consumes: Yichen's `listings`, `orders`, and `order_amounts` migrations.
- Produces: `AuthUser { user_id: Uuid }`
- Produces: `AuthVerifier::verify(&str) -> Result<AuthUser, AuthError>`
- Produces: `OrderRepository::reserve(CreateOrder) -> Result<ReservedOrder, ReserveError>`
- Produces: `PostgresOrderRepository::new(PgPool)`

- [ ] **Step 1: Write the failing concurrent reservation test**

```rust
#[sqlx::test(migrations = "../../supabase/migrations")]
async fn only_one_overlapping_reservation_succeeds(pool: sqlx::PgPool) {
    seed_ps5_fixture(&pool).await;
    let repo = std::sync::Arc::new(core_api::adapters::postgres_orders::PostgresOrderRepository::new(pool));
    let request = core_api::ports::order_repository::CreateOrder {
        listing_id: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        buyer_id: uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
        start_at: Some(time::OffsetDateTime::parse("2026-07-20T10:00:00Z", &time::format_description::well_known::Rfc3339).unwrap()),
        end_at: Some(time::OffsetDateTime::parse("2026-07-21T10:00:00Z", &time::format_description::well_known::Rfc3339).unwrap()),
        quote: core_api::domain::quote::QuoteBreakdown { base_cents: 5_000, service_fee_cents: 300, delivery_fee_cents: 1_500, deposit_cents: 10_000, total_cents: 16_800 },
    };
    let (a, b) = tokio::join!(repo.reserve(request.clone()), repo.reserve(request));
    assert_eq!([a.is_ok(), b.is_ok()].into_iter().filter(|ok| *ok).count(), 1);
}
```

Define the fixture using the schema contract that Yichen's migrations must preserve:

```rust
pub async fn seed_ps5_fixture(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let owner = uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    let buyer = uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
    for (id, name) in [(owner, "Owner"), (buyer, "Buyer")] {
        sqlx::query("INSERT INTO profiles (id, display_name, city, state) VALUES ($1, $2, 'Wellesley', 'MA')")
            .bind(id).bind(name).execute(pool).await?;
    }
    sqlx::query(
        "INSERT INTO listings (id, owner_id, listing_type, title, unit_price_cents, deposit_cents, delivery_fee_cents, status, city, state) VALUES ($1, $2, 'rental', 'PS5 Weekend Rental', 2500, 10000, 1500, 'active', 'Wellesley', 'MA')"
    )
    .bind(uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
    .bind(owner)
    .execute(pool)
    .await?;
    Ok(())
}
```

- [ ] **Step 2: Run and verify failure**

Run: `cargo test -p core-api --test order_reservation`  
Expected: FAIL until Yichen's migrations exist and the repository is implemented.

- [ ] **Step 3: Define the repository contract**

Define authentication before accepting an order. The production verifier calls Supabase Auth's `/auth/v1/user` with the incoming bearer token and configured anon key; a fake verifier supplies deterministic users in API tests.

```rust
#[derive(Debug, Clone, Copy)]
pub struct AuthUser { pub user_id: uuid::Uuid }

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("authentication required")]
    Missing,
    #[error("invalid or expired access token")]
    Invalid,
    #[error("authentication service unavailable")]
    Unavailable,
}

#[async_trait::async_trait]
pub trait AuthVerifier: Send + Sync {
    async fn verify(&self, bearer_token: &str) -> Result<AuthUser, AuthError>;
}
```

Test the verifier adapter with a local mock HTTP server: a `200 {"id":"bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"}` response returns that user; `401` maps to `Invalid`; `500` maps to `Unavailable`. The bearer token and anon key must never appear in logs or errors.

- [ ] **Step 4: Define the repository contract**

```rust
#[derive(Clone)]
pub struct CreateOrder {
    pub listing_id: uuid::Uuid,
    pub buyer_id: uuid::Uuid,
    pub start_at: Option<time::OffsetDateTime>,
    pub end_at: Option<time::OffsetDateTime>,
    pub quote: crate::domain::quote::QuoteBreakdown,
}

pub struct ReservedOrder {
    pub order_id: uuid::Uuid,
    pub expires_at: time::OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum ReserveError {
    #[error("listing unavailable")]
    Unavailable,
    #[error("buyer cannot purchase own listing")]
    SelfTransaction,
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

#[async_trait::async_trait]
pub trait OrderRepository: Send + Sync {
    async fn reserve(&self, order: CreateOrder) -> Result<ReservedOrder, ReserveError>;
}
```

- [ ] **Step 5: Implement one SQL transaction**

The adapter must begin a transaction, lock the listing row with `FOR UPDATE`, reject owner-as-buyer, mark expired pending orders, check overlap, insert `orders`, insert immutable `order_amounts`, append `order_events`, and commit. Map PostgreSQL exclusion violation `23P01` to `ReserveError::Unavailable`; propagate other SQL errors.

```sql
SELECT id, owner_id FROM listings WHERE id = $1 AND status = 'active' FOR UPDATE;
UPDATE orders SET status = 'expired' WHERE listing_id = $1 AND status = 'pending_payment' AND reservation_expires_at <= now();
```

- [ ] **Step 6: Run database and domain verification**

Run: `cargo test -p core-api --test order_reservation -- --nocapture && cargo test -p core-api`  
Expected: concurrent test reports exactly one success; all core-api tests PASS.

- [ ] **Step 7: Commit**

```bash
git add services/core-api/src/auth.rs services/core-api/src/ports services/core-api/src/adapters services/core-api/tests/order_reservation.rs
git commit -m "feat: reserve orders transactionally"
```

## Task 5: Stripe Test Checkout and Verified Webhook

**Files:**
- Create: `services/core-api/src/ports/payment_gateway.rs`
- Create: `services/core-api/src/adapters/stripe.rs`
- Create: `services/core-api/src/http/stripe_webhook.rs`
- Test: `services/core-api/tests/stripe_webhook.rs`

**Interfaces:**
- Produces: `PaymentGateway::create_checkout(CheckoutRequest) -> CheckoutSession`
- Produces: `verify_webhook(payload, signature, secret, now) -> Result<StripeEvent, WebhookError>`
- Produces: `POST /v1/stripe/webhook` with raw-body signature verification and idempotent event handling.

- [ ] **Step 1: Write failing signature tests using a fixed timestamp**

```rust
#[test]
fn valid_signature_is_accepted_once() {
    let payload = br#"{"id":"evt_test","type":"checkout.session.completed"}"#;
    let header = test_signature_header(1_700_000_000, payload, b"whsec_test");
    let event = verify_webhook(payload, &header, b"whsec_test", 1_700_000_100).unwrap();
    assert_eq!(event.id, "evt_test");
}

#[test]
fn old_signature_is_rejected() {
    let payload = br#"{"id":"evt_test","type":"checkout.session.completed"}"#;
    let header = test_signature_header(1_700_000_000, payload, b"whsec_test");
    assert_eq!(verify_webhook(payload, &header, b"whsec_test", 1_700_001_000), Err(WebhookError::Expired));
}
```

- [ ] **Step 2: Run and verify failure**

Run: `cargo test -p core-api --test stripe_webhook`  
Expected: FAIL because signature verification is not implemented.

- [ ] **Step 3: Implement signature verification and idempotency boundary**

Parse Stripe's `t=<timestamp>,v1=<hex>` header, reject timestamps older than 300 seconds, sign `<timestamp>.<raw payload>` with HMAC-SHA256, and compare signatures in constant time. Store each Stripe event ID with a unique constraint before changing payment/order state; a duplicate event returns HTTP `200` without applying a second transition.

- [ ] **Step 4: Implement test Checkout creation**

Create the session server-side with USD line items matching the immutable `order_amounts` snapshot. Put only `order_id` in Stripe metadata. Set success and cancellation URLs from configuration; never accept them from the browser.

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p core-api --test stripe_webhook && cargo test -p core-api`  
Expected: valid, expired, malformed, and duplicate-event tests PASS.

```bash
git add services/core-api/src/ports/payment_gateway.rs services/core-api/src/adapters/stripe.rs services/core-api/src/http/stripe_webhook.rs services/core-api/tests/stripe_webhook.rs
git commit -m "feat: add Stripe test checkout and webhook"
```

## Task 6: HTTP Routes, Stable Errors, and End-to-End API Test

**Files:**
- Create: `services/core-api/src/error.rs`
- Create: `services/core-api/src/http/quotes.rs`
- Create: `services/core-api/src/http/orders.rs`
- Modify: `services/core-api/src/http/mod.rs`
- Modify: `packages/contracts/src/index.ts`
- Test: `services/core-api/tests/ps5_api_flow.rs`

**Interfaces:**
- Consumes: quote engine, `OrderRepository`, `PaymentGateway`, and authenticated user.
- Produces: `AppState` containing `Arc<dyn OrderRepository>`, `Arc<dyn PaymentGateway>`, and `Arc<dyn AuthVerifier>`.
- Produces: `app_with_state(AppState) -> axum::Router`; Task 1's health test is updated to pass fake dependencies.
- Produces: `POST /v1/quotes`
- Produces: `POST /v1/orders`
- Produces: `POST /v1/orders/{id}/transitions`
- Produces stable error body `{ error: { code, message, request_id } }`

- [ ] **Step 1: Write the failing API flow test**

The test uses in-memory fake repository/payment ports and asserts:

1. Quote returns `base=5000`, `serviceFee=300`, `delivery=1500`, `deposit=10000`, `total=16800`.
2. Order creation returns `201`, an order UUID, expiration, and test Checkout URL.
3. A second overlapping order returns `409 LISTING_UNAVAILABLE`.
4. An unauthenticated order returns `401 AUTH_REQUIRED`.

Run: `cargo test -p core-api --test ps5_api_flow`  
Expected: FAIL because the routes and error mapping are absent.

- [ ] **Step 2: Implement the stable API error type**

```rust
#[derive(serde::Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(serde::Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
    pub request_id: uuid::Uuid,
}
```

Map authentication to `401`, missing listing to `404`, invalid input to `422`, overlap/state conflict to `409`, and unexpected dependency failure to `503` without exposing secrets or SQL text.

- [ ] **Step 3: Implement thin handlers and dependency state**

```rust
#[derive(Clone)]
pub struct AppState {
    pub orders: std::sync::Arc<dyn crate::ports::order_repository::OrderRepository>,
    pub payments: std::sync::Arc<dyn crate::ports::payment_gateway::PaymentGateway>,
    pub auth: std::sync::Arc<dyn crate::auth::AuthVerifier>,
}

pub fn app_with_state(state: AppState) -> axum::Router {
    crate::http::routes().with_state(state)
}
```

Handlers deserialize contracts, obtain `AuthUser`, call one domain/application service, and serialize the result. They must not duplicate quote arithmetic, SQL, or Stripe logic. Update `health_api.rs` to create `AppState` with deterministic fake ports before calling `app_with_state`.

- [ ] **Step 4: Run the full core API gate**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`  
Expected: formatting clean, zero Clippy warnings, every unit/integration/API test PASS.

- [ ] **Step 5: Commit**

```bash
git add services/core-api/src/error.rs services/core-api/src/http packages/contracts/src/index.ts services/core-api/tests/ps5_api_flow.rs
git commit -m "feat: expose BSR Hub transaction API"
```

## Task 7: Container, Render Deployment, and Release Evidence

**Files:**
- Create: `services/core-api/Dockerfile`
- Create: `render.yaml`
- Create: `services/core-api/src/config.rs`
- Modify: `services/core-api/src/main.rs`
- Modify: `.env.example`
- Create: `docs/runbooks/core-api.md`
- Create: `docs/runbooks/demo-smoke-test.md`

**Interfaces:**
- Produces: container listening on `$PORT` with `/health`.
- Produces: documented environment names only: `DATABASE_URL`, `SUPABASE_JWT_AUDIENCE`, `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `WEB_SUCCESS_URL`, `WEB_CANCEL_URL`, `SERVICE_FEE_BPS`, `RESERVATION_MINUTES`, `ALLOWED_ORIGIN`.

- [ ] **Step 1: Write the container health check before deployment**

```dockerfile
FROM rust:1-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release -p core-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/core-api /usr/local/bin/core-api
ENV PORT=8080
EXPOSE 8080
CMD ["core-api"]
```

- [ ] **Step 2: Make `main.rs` honor `$PORT` and validate configuration**

```rust
// services/core-api/src/config.rs
pub struct Config {
    pub port: u16,
    pub service_fee_bps: i64,
    pub reservation_minutes: i64,
    pub database_url: String,
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub web_success_url: String,
    pub web_cancel_url: String,
    pub allowed_origin: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        fn required(name: &str) -> Result<String, String> {
            std::env::var(name).map_err(|_| format!("{name} is required"))
        }
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into()).parse().map_err(|_| "PORT must be a u16")?;
        let service_fee_bps = std::env::var("SERVICE_FEE_BPS").unwrap_or_else(|_| "600".into()).parse().map_err(|_| "SERVICE_FEE_BPS must be an integer")?;
        let reservation_minutes = std::env::var("RESERVATION_MINUTES").unwrap_or_else(|_| "30".into()).parse().map_err(|_| "RESERVATION_MINUTES must be an integer")?;
        if !(0..=10_000).contains(&service_fee_bps) { return Err("SERVICE_FEE_BPS must be between 0 and 10000".into()); }
        if reservation_minutes <= 0 { return Err("RESERVATION_MINUTES must be positive".into()); }
        Ok(Self {
            port,
            service_fee_bps,
            reservation_minutes,
            database_url: required("DATABASE_URL")?,
            supabase_url: required("SUPABASE_URL")?,
            supabase_anon_key: required("SUPABASE_ANON_KEY")?,
            stripe_secret_key: required("STRIPE_SECRET_KEY")?,
            stripe_webhook_secret: required("STRIPE_WEBHOOK_SECRET")?,
            web_success_url: required("WEB_SUCCESS_URL")?,
            web_cancel_url: required("WEB_CANCEL_URL")?,
            allowed_origin: required("ALLOWED_ORIGIN")?,
        })
    }
}
```

Update `main.rs` to call `Config::from_env()`, bind `format!("0.0.0.0:{}", config.port)`, and exit before listening if validation fails.

Run: `PORT=8090 cargo run -p core-api`  
Expected: service listens on `0.0.0.0:8090`; `curl http://127.0.0.1:8090/health` returns `{"status":"ok"}`.

- [ ] **Step 3: Build and run the container locally**

Create the Render blueprint:

```yaml
# render.yaml
services:
  - type: web
    name: bsr-hub-core-api
    runtime: docker
    dockerfilePath: ./services/core-api/Dockerfile
    healthCheckPath: /health
    envVars:
      - key: DATABASE_URL
        sync: false
      - key: SUPABASE_URL
        sync: false
      - key: SUPABASE_ANON_KEY
        sync: false
      - key: STRIPE_SECRET_KEY
        sync: false
      - key: STRIPE_WEBHOOK_SECRET
        sync: false
      - key: WEB_SUCCESS_URL
        sync: false
      - key: WEB_CANCEL_URL
        sync: false
      - key: ALLOWED_ORIGIN
        sync: false
      - key: SERVICE_FEE_BPS
        value: "600"
      - key: RESERVATION_MINUTES
        value: "30"
```

Run: `docker build -f services/core-api/Dockerfile -t bsr-core-api .`  
Expected: image build exits `0`.

Run: `docker run --rm -p 8080:8080 --env-file .env.local bsr-core-api`  
Expected: `/health` returns HTTP `200`; no secret values appear in logs.

- [ ] **Step 4: Deploy to Render and run the release gate**

Set `CORE_API_URL` from the exact hostname returned by Render, then run:

```bash
curl -fsS "$CORE_API_URL/health"
```

Expected: `{"status":"ok"}`. Record the same `CORE_API_URL` value in the deployment runbook; do not commit credentials with it.

- [ ] **Step 5: Execute and record the smoke test**

Use two test accounts and record: quote response, Stripe test success, seller confirmation, return, completion, and review. Repeat once at a mobile viewport. Save screenshots without private data under `docs/demo-evidence/`.

- [ ] **Step 6: Run final verification and commit**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && git diff --check`  
Expected: every command exits `0`.

```bash
git add services/core-api/Dockerfile render.yaml .env.example docs/runbooks docs/demo-evidence
git commit -m "chore: deploy and verify BSR Hub core API"
```

## Plan Completion Gate

Lucas's portion is complete only when:

- a clean checkout can build and test the Rust workspace;
- `/health` works locally and on Render;
- quote totals are server-generated with configurable basis points;
- overlapping reservations produce exactly one winner;
- Stripe test Checkout and webhook tests pass;
- invalid transitions are rejected;
- the full PS5 smoke test succeeds across two accounts without manual database edits;
- no secret or private address appears in Git, logs, screenshots, or Lucian's handoff archive.
