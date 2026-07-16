# BSR Hub MongoDB Primary Database Design

**Date:** 2026-07-16  
**Status:** Approved  
**Scope:** Replace the PostgreSQL business-data adapter with a MongoDB primary data layer for the classroom MVP while preserving the Rust API, domain rules, and Supabase token verification.

## 1. Goals

- Persist BSR Hub users, listings, pricing profiles, orders, bookings, runner tasks, conversations, and recommendation events in MongoDB.
- Keep all database credentials and direct database access on the Rust backend.
- Preserve the existing pricing algorithm, integer-cent money model, order state machine, payment verification rules, and repository boundary.
- Support local development with Docker and production deployment with MongoDB Atlas through the same `MONGODB_URI` configuration.
- Prevent duplicate reservations, duplicate payment application, overselling, and unauthorized order transitions.
- Make each layer independently testable: HTTP controllers, application services/domain rules, repository ports, and MongoDB adapters.

## 2. Non-goals

- Real-money production processing, legal escrow, identity verification, or storage of real payment-card data.
- Kubernetes, Kafka, Redis, Elasticsearch, sharding, or microservice decomposition during the two-week MVP.
- Rebuilding authentication. Supabase remains the bearer-token verifier; MongoDB stores the corresponding BSR user profile and roles.
- Direct MongoDB access from the statically hosted frontend.

## 3. Architecture

```text
GitHub Pages web clients
        |
        | HTTPS JSON API
        v
Rust Axum HTTP layer
        |
        v
Application and domain rules
        |
        v
Repository ports
        |
        v
MongoDB adapters
        |
        +-- local single-node replica set (Docker)
        +-- MongoDB Atlas replica set (production-ready configuration)
```

Dependencies remain one-way. HTTP handlers depend on application/domain behavior and repository interfaces. MongoDB-specific BSON types, queries, sessions, and index definitions remain inside the adapter and bootstrap modules.

## 4. Runtime Modes

### Local database mode

- Docker Compose starts MongoDB as a single-node replica set so transactions work locally.
- The Rust service reads `MONGODB_URI=mongodb://...` and `MONGODB_DATABASE=bsr_hub`.
- An idempotent bootstrap command creates validators and indexes, then optionally loads fictional demo data.

### Atlas mode

- The same service connects through `MONGODB_URI` and TLS.
- Credentials are injected as deployment secrets and never committed.
- The frontend receives only the public Rust API base URL.

### Demo-only fallback

The existing in-memory demo application remains available through `BSR_DEMO_MODE=true` for isolated UI demonstrations and fast tests. Database mode is the default path for persistent behavior.

## 5. Identifier and Data Conventions

- Existing UUIDs are stored as canonical lowercase strings to minimize API and domain migration risk.
- All monetary values use signed 64-bit integer cents and an explicit ISO currency code.
- Timestamps use UTC BSON dates.
- Order and runner states use the existing closed enum values; arbitrary strings are rejected.
- Documents include `schema_version`, `created_at`, and `updated_at` where applicable.
- Sensitive delivery details are returned only to authorized participants and never written to public catalog responses.

## 6. Collections

### `users`

Stores the BSR profile associated with a Supabase-authenticated user.

Key fields: `_id` UUID string, display name, avatar URL, roles, verification flags, credit score, coarse location, encrypted/private address references, timestamps, and schema version.

### `listings`

Stores rentals, second-hand products, and workspaces in one polymorphic catalog.

Key fields: `_id`, `owner_id`, `listing_type`, category, title, description, condition, status, images, inventory, GeoJSON location, delivery options, deposit cents, and timestamps. Type-specific attributes are stored in a validated `attributes` object.

### `pricing_profiles`

Stores the algorithm input, recommendation, seller adjustment, final price, billing unit, reason codes, ruleset version, and permitted fulfillment methods. The seller adjustment remains limited to plus or minus 500 cents by domain rules.

### `orders`

Stores buyer, seller, listing, fulfillment, reservation window, current state, immutable quote snapshot, payment summary, deposit state, reservation expiration, and timestamps. Historical orders are never deleted by TTL.

### `booking_slots`

Serializes reservable half-hour slots. Each document identifies one listing and one UTC slot boundary. A unique compound index on `(listing_id, slot_start)` prevents overlapping rentals or workspace bookings. Temporary holds contain `expires_at`; paid or active slots remove that field so TTL no longer applies.

### `order_events`

Append-only audit events for reservations, payment results, state transitions, cancellations, fulfillment, and deposit actions. Duplicate Stripe events are rejected through a unique provider-event index.

### `runner_tasks`

Stores the delivery task, related order, customer, assigned runner, fee quote, public/coarse location, protected exact pickup/drop-off details, task state, completion code hash, payout state, and timestamps.

### `conversations` and `messages`

Conversations store participant UUIDs and the related listing/order. Messages store conversation ID, sender ID, safe text content, moderation state, read state, and creation time. The classroom MVP does not store arbitrary executable content or payment credentials in chat.

### `recommendation_events`

Stores bounded analytics events such as impression, view, search, category selection, favorite, and reservation. It does not store secrets or exact private addresses. These events support later recommendation ranking without coupling analytics logic to order documents.

## 7. Indexes and Validators

- `listings`: `(status, listing_type, category, created_at)`, `(owner_id, status)`, and `location: 2dsphere`.
- `pricing_profiles`: unique `listing_id`.
- `orders`: `(buyer_id, created_at)`, `(seller_id, created_at)`, `(listing_id, status)`, and `(status, reservation_expires_at)`.
- `booking_slots`: unique `(listing_id, slot_start)` and TTL `expires_at`.
- `order_events`: `(order_id, created_at)` and unique sparse `(provider, provider_event_id)`.
- `runner_tasks`: `(status, created_at)`, `(runner_id, status)`, and `public_location: 2dsphere`.
- `conversations`: `(participant_ids, updated_at)` and related listing/order indexes.
- `messages`: `(conversation_id, created_at)`.
- `recommendation_events`: `(user_id, created_at)` and `(listing_id, event_type, created_at)`.

MongoDB JSON Schema validators enforce required identifiers, allowed enum values, integer-cent money, coordinate shape, and minimum schema versions. Domain validation remains authoritative for cross-document business rules.

## 8. Critical Data Flows

### Listing creation

1. HTTP layer validates the request shape and authenticated user.
2. Service computes/validates the price recommendation and seller adjustment.
3. Repository transaction writes the listing and pricing profile.
4. The API returns the public listing view without private seller data.

### Rental or workspace reservation

1. Service validates the time range and converts it into 30-minute slot boundaries.
2. A MongoDB transaction inserts all required `booking_slots`, creates the pending order, writes the quote snapshot, and appends the reservation event.
3. The unique slot index returns a conflict when any interval overlaps.
4. Pending slots carry an expiration timestamp. Successful payment removes expiration; cancellation deletes/releases the slots.

### Second-hand purchase

1. Repository atomically decrements inventory only when `inventory > 0` and listing status is active.
2. The same transaction creates the order and audit event.
3. A failed conditional update returns `409 Conflict` rather than creating an invalid order.

### Payment event

1. Stripe signature verification remains outside the repository.
2. The repository transaction inserts the provider event with a unique ID.
3. It validates amount, currency, order state, and reservation expiration against stored data.
4. A valid event changes the order to paid and promotes booking slots; duplicates become idempotent no-ops.

### Runner acceptance

The repository performs a conditional update from `available` to `accepted` only when no runner is assigned. Exactly one concurrent runner can claim a task.

## 9. Error Handling

- Invalid input or unsupported state: `400 Bad Request`.
- Missing or invalid authentication: `401 Unauthorized`.
- Participant/owner mismatch: `403 Forbidden`.
- Missing resource: `404 Not Found`.
- Slot conflict, sold item, duplicate claim, or invalid concurrent transition: `409 Conflict`.
- Temporary database unavailability: `503 Service Unavailable` with a stable error code; credentials and driver internals are never returned.

MongoDB duplicate-key and transaction errors are mapped at the adapter boundary into existing repository/domain errors. Transient transaction errors use a small bounded retry policy; permanent validation errors are never retried.

## 10. Testing Strategy

- Domain unit tests continue without MongoDB.
- HTTP tests mock repository ports and verify status/error contracts.
- MongoDB integration tests run against a replica-set test container and verify CRUD, validators, indexes, transactions, and error mapping.
- Concurrency tests submit many reservations for one slot and assert exactly one succeeds.
- Inventory tests submit concurrent purchases and assert stock never becomes negative.
- Payment tests verify duplicate events apply once and amount/currency mismatches never change order state.
- Bootstrap tests prove repeated index/validator setup is idempotent.
- End-to-end tests run the Rust API against MongoDB and exercise the web catalog, listing creation, orders, and runner flows.
- Existing Rust formatting, Clippy, unit tests, frontend tests, builds, and responsive browser QA remain release gates.

## 11. Migration and Compatibility

- No real production data currently needs migration; fictional catalog data becomes an idempotent MongoDB seed.
- PostgreSQL migrations remain in the repository as historical/reference material but are not executed by MongoDB runtime mode.
- The `OrderRepository` port remains stable where practical. New catalog, user, runner, chat, and recommendation ports are added by bounded domain area rather than one oversized database interface.
- Shared API contracts remain backward compatible while the frontend switches from local static stores to the Rust JSON API.

## 12. Deployment Boundary

GitHub Pages can host only static frontend files; it cannot run MongoDB or the Rust service. Production persistence therefore requires:

1. MongoDB Atlas or another reachable MongoDB replica set.
2. A deployed Rust API service with `MONGODB_URI`, `MONGODB_DATABASE`, auth, Stripe test-mode, and CORS secrets.
3. A frontend build configured with the public API base URL.

Until those external resources are configured, the repository will include a fully working local Docker path and deployment-ready environment templates without committing credentials.
