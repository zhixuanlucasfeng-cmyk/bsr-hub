# Lucas — Individual Task Instructions

## Your Role

You are the **project coordinator, integration owner, and primary backend coder**. You control architecture changes, the Rust core API, Stripe integration, deployment, daily merges, and release readiness.

Your most important responsibility is not writing the most code. It is keeping five people's work compatible and protecting the complete PS5 rental journey.

## Main Technical Responsibilities

### 1. Monorepo and shared standards

Create and maintain:

```text
apps/web/
services/core-api/
packages/contracts/
supabase/migrations/
docs/
```

Configure formatting, linting, tests, environment templates, CI commands, and contribution rules. Publish API contracts before dependent work begins.

### 2. Rust/Axum core API

Implement:

- `GET /health`;
- `POST /v1/quotes`;
- `POST /v1/orders`;
- `POST /v1/orders/{id}/transitions`;
- `POST /v1/stripe/webhook`.

Use SQLx with explicit transactions. Rust must reload authoritative listing prices, validate fulfillment, prevent overlap, create 30-minute pending-payment reservations, calculate totals in integer cents, enforce order transitions, and append audit events.

### 3. Stripe test integration

- Create test Checkout sessions from the Rust API.
- Verify webhook signatures.
- Make webhook handling idempotent.
- Store Stripe identifiers and payment states without storing card numbers.
- Support successful, cancelled, repeated, and expired flows.

### 4. Integration and deployment

- Configure Supabase project access and provide Yichen with migration contracts and local files.
- Deploy Next.js to Vercel.
- Deploy Rust/Axum to Render.
- Configure safe environment variables and CORS.
- Keep test and production-style environments clearly separated.
- Maintain a health check and final smoke-test procedure.

## Coordination Responsibilities

- Run a brief daily stand-up and integration window.
- Assign one owner per file or feature.
- Review API/database changes with Yichen.
- Coordinate shared frontend changes between Anna and Nasia.
- Receive Lucian's daily inventory and backup package.
- Reject scope additions after Day 8.
- Maintain the blocker list and decide what is cut when time is limited.

## Ten-Day Schedule

- **Day 1:** Create monorepo, contracts, CI, environment templates, branches, and task board.
- **Day 2:** Implement Rust service skeleton, auth-token verification, health check, and SQLx connection.
- **Day 3:** Integrate profiles/listings data and stabilize shared TypeScript contracts.
- **Day 4:** Implement and test quote calculation and availability validation.
- **Day 5:** Implement transactional order creation, overlap protection, Stripe Checkout, and webhook.
- **Day 6:** Merge core work and complete one PS5 rental across two accounts; integration fixes only.
- **Day 7:** Implement/review transitions, delivery, return, completion, and reviews.
- **Day 8:** Deploy, run security/concurrency tests, mobile smoke test, and resolve critical defects.
- **Day 9:** Feature freeze; rehearse recovery, create backup release, and fix only critical bugs.
- **Day 10:** Final smoke test, protect the demo environment, and coordinate presentation support.

## Required Deliverables

- Working monorepo and documented setup.
- Rust API endpoints and automated tests.
- Stripe test Checkout and verified webhook.
- Deployed web, API, and Supabase services.
- Shared contracts and environment template without secrets.
- Daily integration report and blocker list.
- Release checklist, smoke-test evidence, and rollback/backup instructions.

## Coding-Agent Request Template

```text
Follow the approved BSR Hub design and work only on the specified files.
Use test-driven development. Do not change public API contracts or database migrations
without explicit approval. Implement one bounded task, run the complete relevant tests,
show every changed file, and report unresolved risks. Never read, print, or commit secrets.
```

## Integration Rules

- Never merge code based only on an agent's statement that it works.
- Require a test result or screenshot and one human review.
- Run migration rebuild, Rust tests, frontend tests, and the PS5 smoke test after contract changes.
- Do not manually combine teammates' source files from Lucian's backup. Use the official branches or reviewed patches; the backup is for recovery and inventory.
- Keep `main` deployable after the Day 6 integration checkpoint.

## Definition of Done

Lucas's work is complete when the repository is reproducible, all required services are deployed, two separate users can complete the PS5 rental without manual database edits, the critical test suite passes, and the team has a reliable backup demonstration.
