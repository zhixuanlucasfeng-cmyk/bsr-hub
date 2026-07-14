# BSR Hub

BSR Hub is a U.S. peer-to-peer marketplace for buying, selling, renting products, and booking immovable workspaces. The Rust core API owns money, reservation, order-state, and payment rules.

## Lucas core API

```bash
cargo test --workspace
cargo run -p core-api
```

Copy `.env.example` to `.env.local` and provide test/development credentials. Never use live Stripe keys for this MVP.

Key safeguards:

- all money is stored and calculated in integer U.S. cents;
- the service fee defaults to 600 basis points (6%);
- pending-payment inventory holds expire after 30 minutes;
- PostgreSQL prevents overlapping active bookings;
- Stripe webhook signatures are checked within a five-minute window;
- Stripe event IDs are stored before an order is marked paid;
- public listing queries do not need to expose private street addresses.

See [the core API runbook](docs/runbooks/core-api.md) and [the demo checklist](docs/runbooks/demo-smoke-test.md).

