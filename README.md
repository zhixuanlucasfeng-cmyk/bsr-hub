# BSR Hub

BSR Hub is a U.S. peer-to-peer marketplace for buying, selling, renting products, and booking immovable workspaces. The Rust core API owns money, reservation, order-state, and payment rules.

## Run the complete classroom demo

Requirements: Node.js 22+, npm, and stable Rust. No database, Stripe key, Supabase account, Docker, or real payment information is required.

```bash
npm install
npm run demo
```

Open [http://localhost:3000](http://localhost:3000). Search for the PS5 Slim, request a Rust quote, reserve it, then use the **Demo as** selector to switch between Maya (buyer) and Jordan (seller). Complete protected payment, seller confirmation, activation, return, completion, and review.

The demo API is served at `http://localhost:8080`. Its data is fictional and resets when the Rust process restarts.

## Quality gate

```bash
npm run check
```

This checks Rust formatting, Clippy, all Rust tests, web types, web unit tests, and the Next.js production build.

## Project map

- `apps/web`: responsive Next.js marketplace and complete transaction UI.
- `services/core-api`: Rust/Axum pricing, quote, order, payment, and demo APIs.
- `supabase`: production-oriented PostgreSQL schema, RLS, and pgTAP evidence.
- `docs/research`: Lucian's competitor, pricing, privacy, and listing research.
- `docs/presentation`: Nasia's timed statement and live-demo script.
- `docs/qa`: Anna/Nasia accessibility, mobile, and end-to-end evidence.

## Lucas core API

```bash
cargo test --workspace
cargo run -p core-api
```

Copy `.env.example` to `.env.local` and provide test/development credentials. Never use live Stripe keys for this MVP.

Key safeguards:

- all money is stored and calculated in integer U.S. cents;
- PS5 and workspace prices are recommended by versioned Rust rules, with seller changes limited to plus or minus $5 per billing unit;
- sellers choose either 30-minute or daily billing; partial units are rounded up by the server (for example, 31 minutes is two units);
- the service fee defaults to 600 basis points (6%);
- pending-payment inventory holds expire after 30 minutes;
- PostgreSQL prevents overlapping active bookings;
- Stripe webhook signatures are checked within a five-minute window;
- Stripe event IDs are stored before an order is marked paid, and event type, paid status, order, amount, currency, and reservation expiry must all match;
- public listing queries do not need to expose private street addresses.

See [the core API runbook](docs/runbooks/core-api.md) and [the demo checklist](docs/runbooks/demo-smoke-test.md).
