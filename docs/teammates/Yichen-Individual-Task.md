# Yicheng — Data, Logic, and Security

## Your Assignment

You own the missing Supabase schema, Row Level Security, database tests, booking logic evidence, and Rust logic review for BSR Hub.

Read these files first:

1. `docs/superpowers/specs/2026-07-14-yichen-data-security-handoff-design.md`
2. `docs/superpowers/plans/2026-07-14-yichen-data-security.md`
3. `supabase/migrations/20260714000100_core_marketplace.sql`

The existing core migration is already merged and must not be edited. Create new timestamped migrations only.

## Tool

Because you do not have a VPN, use Qoder CN（原通义灵码）for local SQL, tests, and documentation. Never paste `.env`, API keys, database passwords, access tokens, private addresses, or Stripe secrets into any model.

## Required Work

- Add `listing_images` and `listing_availability`.
- Add backend-protected `payments` and completed-order `reviews`.
- Complete RLS for ownership, order participation, financial protection, and review eligibility.
- Prove identical, partial, contained, adjacent, expired, cancelled, completed, active, and concurrent booking behavior.
- Build the rental/sale/workspace state-transition matrix.
- Review Lucas's Rust quote, reservation, state, and Stripe logic without rewriting it.
- Return an organized `yicheng-handoff/` inventory with test evidence.

## Files You Must Not Change

```text
supabase/migrations/20260714000100_core_marketplace.sql
services/core-api/**
apps/web/**
.env
.env.local
render.yaml
```

If a Rust issue is discovered, record it in `docs/reviews/yichen-rust-review.md` and send it to Lucas.

## Qoder CN Prompt

```text
Work inside the BSR Hub repository and follow
docs/superpowers/plans/2026-07-14-yichen-data-security.md task by task.

Only create new Supabase/PostgreSQL migrations, pgTAP tests, database logic
documents, and the Yicheng handoff inventory. Do not modify the existing core
migration, Rust API, Next.js frontend, environment files, or deployment files.

Use red-green-refactor: write each pgTAP test, run it and confirm the expected
failure, implement the minimum migration or policy, then rerun it. Store money
as bigint integer cents. Prove that cross-user writes, browser payment writes,
invalid reviews, and overlapping bookings are rejected. Show every changed file,
exact verification command, output summary, and commit hash. Never include secrets.
```

## Delivery

Send Lucas the `yicheng-handoff/` folder plus the Git branch or ZIP containing the repository-ready files. Lucas will run the clean rebuild and merge review.

Your work is finished only when all database tests pass from a clean reset and the handoff contains no credentials or private user data.
