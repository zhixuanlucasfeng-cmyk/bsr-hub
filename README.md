# BSR Hub

BSR Hub is a U.S. peer-to-peer marketplace for buying, selling, renting products, and booking immovable workspaces. The Rust core API owns money, reservation, order-state, and payment rules.

## Public team demos

- [Choose BSR Hub or BSR Runner](https://zhixuanlucasfeng-cmyk.github.io/bsr-hub/)
- [Open BSR Hub](https://zhixuanlucasfeng-cmyk.github.io/bsr-hub/hub/)
- [Open BSR Runner](https://zhixuanlucasfeng-cmyk.github.io/bsr-hub/runner/)

The public GitHub Pages versions use fictional in-browser demo data, so teammates can test the full journeys without installing Rust or running a server. The repository still contains the Rust/Axum implementation used for local development and verification. No real payment, address, identity, or employment data is collected.

### Real account demo

BSR Hub now has a real-account foundation for the public demo. Guest mode still uses fictional listings and browser-only state. Real-account mode uses Supabase Auth for sign up/login, the Rust API for token verification, and MongoDB for BSR Hub user profiles and marketplace records.

To turn on real accounts, set `NEXT_PUBLIC_SUPABASE_URL`, `NEXT_PUBLIC_SUPABASE_ANON_KEY`, `NEXT_PUBLIC_API_BASE_URL`, and `NEXT_PUBLIC_STATIC_DEMO=false` for the web app. The Rust API must also have `MONGODB_URI`, `MONGODB_DATABASE`, `SUPABASE_URL`, `SUPABASE_ANON_KEY`, `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `WEB_SUCCESS_URL`, `WEB_CANCEL_URL`, and `ALLOWED_ORIGIN`.

Important: the classroom demo must not collect real payment, identity, or private address data. Supabase stores passwords; MongoDB stores only BSR Hub profile and marketplace data.

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

To build the exact static artifact deployed to GitHub Pages:

```bash
npm run pages:check
```

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

### MongoDB persistent mode

Docker Desktop is required for the local persistent database. The setup uses a single-node replica set because reservations and payment transitions require transactions.

```bash
npm run mongo:up
cp .env.mongodb.example .env.local
set -a; source .env.local; set +a
npm run mongo:bootstrap
cargo run -p core-api
```

The bootstrap command is idempotent and loads only fictional classroom listings. MongoDB credentials remain in the Rust backend environment; they are never shipped to either frontend. Use `npm run mongo:down` to stop MongoDB while preserving data, or `npm run mongo:reset` to delete the local volume.

Run the complete persistence release gate—including two bootstrap passes, real replica-set transaction tests, Rust checks, and both frontend checks—with:

```bash
npm run mongo:check
```

Key safeguards:

- all money is stored and calculated in integer U.S. cents;
- PS5 and workspace prices are recommended by versioned Rust rules, with seller changes limited to plus or minus $5 per billing unit;
- sellers choose either 30-minute or daily billing; partial units are rounded up by the server (for example, 31 minutes is two units);
- the service fee defaults to 600 basis points (6%);
- pending-payment inventory holds expire after 30 minutes;
- MongoDB transactions and a unique half-hour slot index prevent overlapping active bookings;
- Stripe webhook signatures are checked within a five-minute window;
- Stripe event IDs are stored before an order is marked paid, and event type, paid status, order, amount, currency, and reservation expiry must all match;
- public listing queries do not need to expose private street addresses.

See [the core API runbook](docs/runbooks/core-api.md) and [the demo checklist](docs/runbooks/demo-smoke-test.md).

For the hosted Rust API and persistent Atlas database, use the [Render + MongoDB Atlas production runbook](docs/runbooks/render-mongodb-atlas.md). The checked-in Blueprint explicitly uses Render's free plan and never stores the MongoDB connection string in Git.

## BSR Runner: the local task marketplace

BSR Runner is the team's second website. Customers post lawful nearby errands or BSR Hub delivery requests; independent adult runners choose work and see their automatic payout before accepting. Exact addresses stay private until assignment, simulated payment stays protected until completion, and Rust controls every task transition.

Run it with the same Node.js and Rust requirements as BSR Hub:

```bash
npm install
npm run runner:demo
```

Open [http://localhost:3001](http://localhost:3001). No database, API key, real identity, real address, or real payment method is needed. The site uses fictional Massachusetts locations and resets from the avatar button.

Suggested full demo:

1. As **Maya · Customer**, post a lawful task and inspect the automatic Rust quote.
2. As **Jordan · Runner**, open the first package job, accept it, confirm pickup, and start delivery.
3. Switch to Maya, reopen that job, and complete it with classroom code `482731`.
4. Switch to Jordan to show the released earnings, then **Alex · Admin** for the safety desk.
5. Use **Prohibited item — safety test** in the post form to show the server rejecting unsafe work.

Important: this is a classroom MVP, not a production labor or payments service. A U.S. launch requires jurisdiction-specific review of worker classification, insurance, taxes, age/identity checks, background screening, emergency handling, accessibility, payments, and privacy.

Technical references:

- [BSR Runner design](docs/superpowers/specs/2026-07-15-bsr-runner-design.md)
- [Runner state and role matrix](docs/database/runner-state-matrix.md)
- [Two-minute presentation script](docs/presentation/bsr-runner-two-minute-demo.md)
