# MongoDB Core Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Rust core API's PostgreSQL order adapter with a MongoDB-backed implementation that persists listings, pricing, reservations, payments, and order transitions without breaking existing HTTP or domain behavior.

**Architecture:** Keep Axum handlers and domain rules database-agnostic behind `OrderRepository`. Add a focused MongoDB adapter with BSON persistence models, idempotent index/bootstrap logic, and transaction-based reservation/payment operations. Local development uses a single-node replica set through Docker Compose; Atlas uses the same `MONGODB_URI` contract.

**Tech Stack:** Rust 2024, Axum 0.8, official MongoDB Rust driver 3.3, Tokio, Serde, UUID, Docker Compose, MongoDB replica-set transactions.

## Global Constraints

- The frontend never receives `MONGODB_URI` and never connects directly to MongoDB.
- All money remains signed 64-bit integer U.S. cents; no floating-point money is introduced.
- Existing UUIDs remain canonical lowercase UUID strings in BSON.
- Existing pricing, quote, payment verification, and order transition domain functions remain authoritative.
- Supabase remains the bearer-token verifier; this plan does not rebuild authentication.
- MongoDB-specific errors and BSON types do not cross the adapter boundary.
- Production credentials are environment variables and are never committed.
- The existing `BSR_DEMO_MODE=true` in-memory classroom path continues to work without MongoDB.
- Real payments, legal escrow, real identity verification, and real private addresses remain outside the classroom MVP.

## Scope Boundary

This first implementation plan delivers the working database foundation and the transactional core marketplace path. Runner persistence, chat persistence, and recommendation-event ingestion are independent subsystems and will use separate follow-up plans after the core MongoDB adapter is stable. Their collection names and indexes are reserved by the approved design, but this plan does not expose unfinished HTTP APIs for them.

---

### Task 1: MongoDB Configuration and Database-Neutral Errors

**Files:**
- Modify: `services/core-api/Cargo.toml`
- Modify: `services/core-api/src/config.rs`
- Modify: `services/core-api/src/ports/order_repository.rs`
- Modify: `services/core-api/src/adapters/postgres_orders.rs`
- Modify: `services/core-api/tests/config_rules.rs`
- Create: `.env.mongodb.example`

**Interfaces:**
- Produces: `Config.mongodb_uri: String`, `Config.mongodb_database: String`.
- Produces: `ReserveError::Database(String)` and `ReserveError::database(error)`.
- Consumes: existing `SERVICE_FEE_BPS`, `RESERVATION_MINUTES`, Stripe test-mode, Supabase, and CORS configuration.

- [x] **Step 1: Write failing MongoDB configuration tests**

Add cases to `services/core-api/tests/config_rules.rs` proving `MONGODB_URI` and `MONGODB_DATABASE` are required and that the database name rejects an empty value or names containing `/`, `\\`, `.`, space, `"`, `$`, or a null byte.

```rust
#[test]
fn mongodb_configuration_is_required() {
    let mut values = required();
    values.retain(|(key, _)| key != "MONGODB_URI");
    assert_eq!(
        Config::from_values(values).unwrap_err(),
        "MONGODB_URI is required"
    );
}

#[test]
fn mongodb_database_name_is_validated() {
    let mut values = required();
    values.push(("MONGODB_DATABASE", "bad/name"));
    assert_eq!(
        Config::from_values(values).unwrap_err(),
        "MONGODB_DATABASE contains unsupported characters"
    );
}
```

- [x] **Step 2: Run the focused tests and verify RED**

Run: `cargo test -p core-api --test config_rules`

Expected: compilation or assertion failure because `Config` still reads `DATABASE_URL` and has no MongoDB fields.

- [x] **Step 3: Add the driver and configuration contract**

In `services/core-api/Cargo.toml`, add the MongoDB dependencies while temporarily retaining SQLx until the PostgreSQL adapter is removed in Task 6:

```toml
futures-util = "0.3"
mongodb = "3.3"
```

Add `"macros"` to the existing `time` crate features so focused timestamp tests can use `time::macros::datetime`.

Keep SQLx out of runtime source after the PostgreSQL adapter is retired. Change `Config` to:

```rust
pub struct Config {
    pub port: u16,
    pub service_fee_bps: i64,
    pub reservation_minutes: i64,
    pub database_url: String,
    pub mongodb_uri: String,
    pub mongodb_database: String,
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub web_success_url: String,
    pub web_cancel_url: String,
    pub allowed_origin: String,
}
```

`database_url` remains as a transitional field while `PostgresOrderRepository` is still compiled; Task 6 removes it together with the PostgreSQL runtime adapter.

Validate the database name before constructing `Config`:

```rust
let mongodb_database = required("MONGODB_DATABASE")?;
if mongodb_database
    .chars()
    .any(|character| matches!(character, '/' | '\\' | '.' | ' ' | '"' | '$' | '\0'))
{
    return Err("MONGODB_DATABASE contains unsupported characters".into());
}
```

Change the repository error variant to avoid leaking SQLx through the domain port:

```rust
#[error("database error")]
Database(String),

impl ReserveError {
    pub fn database(error: impl std::fmt::Display) -> Self {
        Self::Database(error.to_string())
    }
}
```

Update `PostgresOrderRepository::map_sql` and direct SQLx error mappings to call `ReserveError::database(error)` so the existing adapter continues to compile during the incremental migration.

Create `.env.mongodb.example` with non-secret local values and explicit test placeholders:

```dotenv
MONGODB_URI=mongodb://bsr:bsr-local-only@localhost:27017/?replicaSet=rs0&authSource=admin
MONGODB_DATABASE=bsr_hub
PORT=8080
SERVICE_FEE_BPS=600
RESERVATION_MINUTES=30
SUPABASE_URL=https://example.supabase.co
SUPABASE_ANON_KEY=replace-with-development-anon-key
STRIPE_SECRET_KEY=sk_test_replace_me
STRIPE_WEBHOOK_SECRET=whsec_replace_me
WEB_SUCCESS_URL=http://localhost:3000/?payment=success
WEB_CANCEL_URL=http://localhost:3000/?payment=cancelled
ALLOWED_ORIGIN=http://localhost:3000
```

- [x] **Step 4: Run tests and quality checks**

Run: `cargo test -p core-api --test config_rules && cargo fmt --check && cargo clippy -p core-api --all-targets -- -D warnings`

Expected: configuration tests pass; format and Clippy exit 0.

- [x] **Step 5: Commit**

```bash
git add services/core-api/Cargo.toml services/core-api/src/config.rs services/core-api/src/ports/order_repository.rs services/core-api/src/adapters/postgres_orders.rs services/core-api/tests/config_rules.rs .env.mongodb.example Cargo.lock
git commit -m "feat: configure MongoDB persistence"
```

---

### Task 2: Local Replica Set, BSON Models, and Idempotent Bootstrap

**Files:**
- Create: `compose.mongodb.yml`
- Create: `scripts/init-mongo-replica.sh`
- Create: `services/core-api/src/adapters/mongo/mod.rs`
- Create: `services/core-api/src/adapters/mongo/models.rs`
- Create: `services/core-api/src/adapters/mongo/bootstrap.rs`
- Create: `services/core-api/src/adapters/mongo/slots.rs`
- Create: `services/core-api/tests/mongo_slots.rs`
- Modify: `services/core-api/src/adapters/mod.rs`
- Modify: `package.json`

**Interfaces:**
- Produces: `MongoCollections::new(Database)`, `bootstrap(&Database) -> Result<(), MongoAdapterError>`.
- Produces: `slot_boundaries(start, end) -> Result<Vec<OffsetDateTime>, SlotError>`.
- Produces persistence structs `ListingDocument`, `PricingProfileDocument`, `OrderDocument`, `BookingSlotDocument`, and `OrderEventDocument`.

- [x] **Step 1: Write slot-boundary tests**

Create `services/core-api/tests/mongo_slots.rs`:

```rust
use core_api::adapters::mongo::slots::slot_boundaries;
use time::macros::datetime;

#[test]
fn half_hour_window_creates_one_slot() {
    let slots = slot_boundaries(
        datetime!(2026-07-16 10:00 UTC),
        datetime!(2026-07-16 10:30 UTC),
    )
    .unwrap();
    assert_eq!(slots, vec![datetime!(2026-07-16 10:00 UTC)]);
}

#[test]
fn unaligned_or_backwards_windows_are_rejected() {
    assert!(slot_boundaries(
        datetime!(2026-07-16 10:10 UTC),
        datetime!(2026-07-16 10:40 UTC),
    )
    .is_err());
    assert!(slot_boundaries(
        datetime!(2026-07-16 11:00 UTC),
        datetime!(2026-07-16 10:30 UTC),
    )
    .is_err());
}
```

- [x] **Step 2: Run the test and verify RED**

Run: `cargo test -p core-api --test mongo_slots`

Expected: compilation failure because the MongoDB adapter and slot helper do not exist.

- [x] **Step 3: Implement slot generation and BSON documents**

`slot_boundaries` must accept only UTC boundaries aligned to minute `00` or `30`, require `end > start`, cap a single reservation at 2,880 slots, and emit every half-hour start in `[start, end)`.

Use explicit persistence structs rather than serializing domain objects directly. For example:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingSlotDocument {
    pub listing_id: String,
    pub slot_start: mongodb::bson::DateTime,
    pub order_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<mongodb::bson::DateTime>,
    pub schema_version: i32,
}
```

Store enum strings with explicit conversions from existing domain enums. Store the complete quote snapshot inside `OrderDocument`, including base, service fee, delivery fee, deposit, total, and currency.

- [x] **Step 4: Add Docker replica-set development commands**

`compose.mongodb.yml` runs MongoDB with `--replSet rs0 --bind_ip_all`, a named data volume, health check, and local-only credentials. `scripts/init-mongo-replica.sh` calls `rs.initiate()` idempotently and waits until the node becomes primary.

Add root scripts:

```json
{
  "mongo:up": "docker compose -f compose.mongodb.yml up -d --wait && bash scripts/init-mongo-replica.sh",
  "mongo:down": "docker compose -f compose.mongodb.yml down",
  "mongo:reset": "docker compose -f compose.mongodb.yml down -v"
}
```

- [x] **Step 5: Implement indexes and validators**

`bootstrap()` creates collections when missing and applies idempotent indexes with stable names:

```rust
IndexModel::builder()
    .keys(doc! { "listing_id": 1, "slot_start": 1 })
    .options(
        IndexOptions::builder()
            .name("booking_slot_unique".into())
            .unique(true)
            .build(),
    )
    .build()
```

Create `booking_slot_expiry` with `expire_after: Duration::ZERO` on `expires_at`, `pricing_listing_unique` on `pricing_profiles.listing_id`, and `provider_event_unique` as a unique partial index on provider event IDs. Apply `$jsonSchema` validators using `collMod` for required identifier, money, state, timestamp, and schema-version fields.

- [x] **Step 6: Run focused and static checks**

Run: `cargo test -p core-api --test mongo_slots && cargo fmt --check && cargo clippy -p core-api --all-targets -- -D warnings`

Expected: slot tests pass; format and Clippy exit 0.

- [x] **Step 7: Commit**

```bash
git add compose.mongodb.yml scripts/init-mongo-replica.sh package.json services/core-api/src/adapters services/core-api/tests/mongo_slots.rs
git commit -m "feat: bootstrap MongoDB collections"
```

---

### Task 3: Fictional Seed and Pricing Persistence

**Files:**
- Create: `services/core-api/src/adapters/mongo/seed.rs`
- Create: `services/core-api/src/bin/mongo_bootstrap.rs`
- Create: `services/core-api/tests/mongo_pricing_integration.rs`
- Modify: `services/core-api/src/adapters/mongo/mod.rs`
- Modify: `package.json`
- Modify: `README.md`

**Interfaces:**
- Produces: `MongoOrderRepository::connect(uri, database, service_fee_bps, reservation_minutes)`.
- Produces: `seed_fictional_catalog(&Database) -> Result<SeedReport, MongoAdapterError>`.
- Implements: `OrderRepository::pricing` and `OrderRepository::save_pricing_profile`.

- [x] **Step 1: Write ignored integration tests for pricing round trips**

The test reads `MONGODB_TEST_URI`; when absent it reports a clear skip through `#[ignore = "requires MONGODB_TEST_URI replica set"]`. When enabled, it creates a unique database name, bootstraps it, seeds one active listing, saves a PS5 pricing profile, reads it back through `pricing()`, and drops the test database.

Key assertion:

```rust
assert_eq!(snapshot.unit_price_cents, stored.final_unit_price_cents);
assert_eq!(snapshot.service_fee_bps, 600);
assert_eq!(snapshot.deposit_cents, 10_000);
assert!(snapshot
    .allowed_fulfillment_methods
    .contains(&FulfillmentMethod::Pickup));
```

- [x] **Step 2: Run the integration test and verify RED**

Run: `cargo test -p core-api --test mongo_pricing_integration -- --ignored --nocapture`

Expected: compilation failure because `MongoOrderRepository` and seed functions are not implemented.

- [x] **Step 3: Implement connection, seed, and pricing methods**

`connect()` parses `ClientOptions`, assigns app name `bsr-hub-core-api`, creates one shared `Client`, selects the configured database, and invokes bootstrap before returning.

The seed uses stable fictional UUIDs and `replace_one(...).upsert(true)` so repeated execution is safe. It must include at least one rental, workspace, and second-hand listing without real names, addresses, or payment information.

`pricing()` queries the active listing and its unique pricing profile, converts BSON values to `PricingSnapshot`, and maps malformed persistence values to `ReserveError::InvalidPricing`.

`save_pricing_profile()` verifies active listing ownership, then upserts the profile. The Rust domain recommendation and ±500-cent seller limit remain authoritative; the adapter only persists the already-validated values.

- [x] **Step 4: Add bootstrap command**

`mongo_bootstrap` reads `Config`, connects, bootstraps, seeds, prints only collection counts, and never prints the URI. Add:

```json
"mongo:bootstrap": "cargo run -p core-api --bin mongo_bootstrap"
```

Document `npm run mongo:up`, copying `.env.mongodb.example`, loading environment variables, and `npm run mongo:bootstrap` in `README.md`.

- [x] **Step 5: Run local integration verification**

Run:

```bash
npm run mongo:up
MONGODB_TEST_URI='mongodb://bsr:bsr-local-only@localhost:27017/?replicaSet=rs0&authSource=admin' cargo test -p core-api --test mongo_pricing_integration -- --ignored --nocapture
```

Expected: one ignored integration test runs and passes; database cleanup succeeds.

- [x] **Step 6: Commit**

```bash
git add services/core-api/src/adapters/mongo services/core-api/src/bin/mongo_bootstrap.rs services/core-api/tests/mongo_pricing_integration.rs package.json README.md
git commit -m "feat: persist MongoDB listing prices"
```

---

### Task 4: Transactional Reservation and Booking Slots

**Files:**
- Modify: `services/core-api/src/adapters/mongo/mod.rs`
- Modify: `services/core-api/src/adapters/mongo/models.rs`
- Modify: `services/core-api/src/ports/order_repository.rs`
- Modify: `services/core-api/src/http/orders.rs`
- Create: `services/core-api/tests/mongo_reservation_integration.rs`

**Interfaces:**
- Implements: `OrderRepository::reserve(CreateOrder) -> Result<ReservedOrder, ReserveError>`.
- Consumes: `slot_boundaries`, `BookingSlotDocument`, active `ListingDocument`, `FulfillmentMethod`, and immutable `QuoteBreakdown`.

- [x] **Step 1: Write reservation concurrency tests**

Create tests that seed one rental and issue 20 simultaneous reservations for the same 30-minute interval. Assert exactly one `Ok(ReservedOrder)` and 19 `ReserveError::Unavailable` results. Also assert a buyer cannot reserve their own listing and a non-overlapping slot succeeds.

```rust
let successes = results.iter().filter(|result| result.is_ok()).count();
let conflicts = results
    .iter()
    .filter(|result| matches!(result, Err(ReserveError::Unavailable)))
    .count();
assert_eq!((successes, conflicts), (1, 19));
```

- [x] **Step 2: Run and verify RED**

Run: `cargo test -p core-api --test mongo_reservation_integration -- --ignored --nocapture`

Expected: failure because MongoDB `reserve()` is not implemented.

- [x] **Step 3: Implement one transaction per reservation**

First add `fulfillment: FulfillmentMethod` to `CreateOrder`. In the HTTP handler, copy `request.fulfillment` before passing the request into `prepare_quote`, then include it in `CreateOrder` so persistence never has to infer the chosen method from delivery fees.

Within a `ClientSession` transaction:

1. Read the active listing and reject self-transactions.
2. Generate all slot documents for rental/workspace orders.
3. Insert the slots with the session; map duplicate key `11000` to `ReserveError::Unavailable`.
4. For second-hand stock, conditionally update with `inventory: { $gt: 0 }` and `$inc: { inventory: -1 }`.
5. Insert the pending order containing seller ID, quote snapshot, window, fulfillment, and `reservation_expires_at`.
6. Append a `reservation_created` event.
7. Commit and return the UUID and expiration.

Use the MongoDB 3.x session action API (`collection.insert_many(documents).session(&mut session).await`) and never run parallel operations inside one session.

- [x] **Step 4: Implement bounded transient retry**

Retry the whole transaction at most three times only when the driver marks the error with `TransientTransactionError` or `UnknownTransactionCommitResult`. Duplicate keys and validation errors return immediately.

- [x] **Step 5: Run reservation and existing API tests**

Run:

```bash
MONGODB_TEST_URI='mongodb://bsr:bsr-local-only@localhost:27017/?replicaSet=rs0&authSource=admin' cargo test -p core-api --test mongo_reservation_integration -- --ignored --nocapture
cargo test -p core-api --test ps5_api_flow
```

Expected: concurrency test proves one winner; existing HTTP flow remains green.

- [x] **Step 6: Commit**

```bash
git add services/core-api/src/adapters/mongo services/core-api/src/ports/order_repository.rs services/core-api/src/http/orders.rs services/core-api/tests/mongo_reservation_integration.rs
git commit -m "feat: reserve MongoDB booking slots atomically"
```

---

### Task 5: Idempotent Payment Events and Order Transitions

**Files:**
- Modify: `services/core-api/src/adapters/mongo/mod.rs`
- Modify: `services/core-api/src/adapters/mongo/models.rs`
- Create: `services/core-api/tests/mongo_order_lifecycle_integration.rs`

**Interfaces:**
- Implements: `OrderRepository::apply_payment_event`.
- Implements: `OrderRepository::transition`.
- Consumes: existing `validate_payment` and `OrderState::transition` domain functions.

- [x] **Step 1: Write lifecycle integration tests**

Cover:

- the same provider event applied twice returns `Applied`, then `Duplicate`;
- wrong amount and currency return the existing rejected outcomes without marking the order paid;
- an expired reservation becomes expired and releases its booking slots;
- only the seller can confirm/fulfill and only the buyer can activate/return;
- invalid transitions return `ReserveError::InvalidTransition`;
- concurrent runner-independent order transitions cannot create two next states.

- [x] **Step 2: Run and verify RED**

Run: `cargo test -p core-api --test mongo_order_lifecycle_integration -- --ignored --nocapture`

Expected: failure because payment and transition methods are not implemented by MongoDB.

- [x] **Step 3: Implement idempotent payment transaction**

Insert `order_events` with `(provider="stripe", provider_event_id)` first inside the transaction. Duplicate key returns `PaymentEventOutcome::Duplicate`. Read the order, build `StoredOrderPayment`, call `validate_payment`, and only for `Accepted` update `pending_payment -> paid` while unsetting `booking_slots.expires_at`. For expired/rejected outcomes append a bounded event type and release slots only when the reservation is expired.

- [x] **Step 4: Implement authorized compare-and-set transitions**

Read order participant IDs and current state, validate actor authorization and the domain transition, then update with a filter containing both `_id` and the previously read state. A zero-match update is a concurrent conflict and maps to `InvalidTransition`. Append the state event in the same transaction.

- [x] **Step 5: Run lifecycle and domain verification**

Run:

```bash
MONGODB_TEST_URI='mongodb://bsr:bsr-local-only@localhost:27017/?replicaSet=rs0&authSource=admin' cargo test -p core-api --test mongo_order_lifecycle_integration -- --ignored --nocapture
cargo test -p core-api --test payment_verification --test domain_rules --test stripe_webhook
```

Expected: integration and existing domain tests pass.

- [x] **Step 6: Commit**

```bash
git add services/core-api/src/adapters/mongo services/core-api/tests/mongo_order_lifecycle_integration.rs
git commit -m "feat: persist MongoDB order lifecycle"
```

---

### Task 6: Runtime Wiring and Health Evidence

**Files:**
- Modify: `services/core-api/src/main.rs`
- Modify: `services/core-api/src/adapters/mod.rs`
- Delete: `services/core-api/src/adapters/postgres_orders.rs`
- Modify: `services/core-api/src/http/health.rs`
- Modify: `services/core-api/tests/health_api.rs`
- Modify: `services/core-api/Dockerfile`
- Modify: `docs/runbooks/core-api.md`

**Interfaces:**
- Consumes: `MongoOrderRepository::connect` and `Config.mongodb_*`.
- Produces: production `AppState.orders: Arc<MongoOrderRepository>`.
- Produces: `OrderRepository::readiness() -> Result<(), ReserveError>` with a safe default for test fakes and a MongoDB ping override.

- [x] **Step 1: Write health/readiness contract tests**

Keep `/health` as a process liveness endpoint. Add `OrderRepository::readiness()` with a default `Ok(())`, then add `/ready`: ready returns `200 {"status":"ready"}` and unavailable returns `503 {"code":"database_unavailable"}`. Existing fake repositories therefore require no extra test setup.

- [x] **Step 2: Run and verify RED**

Run: `cargo test -p core-api --test health_api`

Expected: failure because `/ready` and its readiness port do not exist.

- [x] **Step 3: Wire MongoDB into non-demo startup**

Replace `PgPoolOptions` and `PostgresOrderRepository` with:

```rust
let orders = MongoOrderRepository::connect(
    &config.mongodb_uri,
    &config.mongodb_database,
    config.service_fee_bps,
    config.reservation_minutes,
)
.await
.expect("connect to MongoDB and bootstrap collections");
```

Share the repository through `Arc`. Preserve the Stripe and Supabase adapters and existing CORS behavior. Remove the PostgreSQL adapter module and runtime SQLx dependency.

- [x] **Step 4: Add readiness and deployment documentation**

Readiness uses a MongoDB `ping` command with a short server-selection timeout and maps failures to `503` without returning the URI. Update the Dockerfile and runbook with the two runtime modes, secret names, Atlas network allow-list requirement, and the fact that GitHub Pages cannot host the Rust process.

- [x] **Step 5: Run the core quality gate**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

Expected: all Rust formatting, lint, unit, and HTTP tests pass.

- [x] **Step 6: Commit**

```bash
git add services/core-api/src services/core-api/tests services/core-api/Cargo.toml services/core-api/Dockerfile docs/runbooks/core-api.md Cargo.lock
git commit -m "feat: run core API on MongoDB"
```

---

### Task 7: End-to-End Verification and Handoff

**Files:**
- Create: `scripts/check-mongodb.sh`
- Modify: `package.json`
- Modify: `README.md`
- Modify: `docs/runbooks/demo-smoke-test.md`

**Interfaces:**
- Produces: `npm run mongo:check` for repeatable local database verification.
- Consumes: Docker Compose MongoDB, bootstrap binary, Rust API, and existing test commands.

- [x] **Step 1: Write the verification script contract**

`scripts/check-mongodb.sh` must:

1. require Docker and Compose;
2. start the replica set;
3. export `MONGODB_TEST_URI` without printing it;
4. run bootstrap twice to prove idempotency;
5. run all `mongo_*_integration` ignored tests explicitly;
6. run the complete repository quality gate;
7. stop containers on exit while preserving the volume unless `MONGO_RESET=true`.

- [x] **Step 2: Add the root command and handoff documentation**

Add:

```json
"mongo:check": "bash scripts/check-mongodb.sh"
```

Document the exact local start, bootstrap, API start, health/readiness URLs, fictional-data limitation, Atlas secret boundary, and cleanup commands.

- [x] **Step 3: Run full MongoDB verification**

Run: `npm run mongo:check`

Expected: bootstrap succeeds twice, all MongoDB integration tests pass, all existing Rust/Hub/Runner tests pass, and static production builds succeed.

- [x] **Step 4: Confirm no secret or PostgreSQL runtime references remain**

Run:

```bash
git grep -nE 'mongodb(\+srv)?://[^[:space:]]+:[^[:space:]]+@' -- ':!*.example' ':!docs/**'
git grep -n 'PostgresOrderRepository\|PgPoolOptions\|DATABASE_URL' -- services/core-api ':!docs/**'
git diff --check
```

Expected: both secret/runtime searches print nothing; `git diff --check` exits 0.

- [x] **Step 5: Commit**

```bash
git add scripts/check-mongodb.sh package.json README.md docs/runbooks/demo-smoke-test.md
git commit -m "test: verify MongoDB core persistence"
```

---

## Follow-up Plans After Core Persistence

1. **MongoDB catalog and web synchronization:** public listing reads, authenticated listing creation, user profiles, and the GitHub Pages frontend's runtime API client with static fallback.
2. **MongoDB Runner persistence:** persistent runner applications, task creation, conditional acceptance, completion codes, payout states, and protected locations.
3. **MongoDB conversations and recommendations:** participant-authorized conversations, messages, moderation metadata, analytics events, and explainable recommendation ranking.

Each follow-up keeps its own repository port, HTTP contract, integration tests, and release gate rather than expanding `OrderRepository` into a database-wide god interface.
