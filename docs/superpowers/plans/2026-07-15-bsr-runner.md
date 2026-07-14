# BSR Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a separate BSR Runner website where adults can apply for flexible local work, customers can publish automatically priced errands, and both sides can complete protected demo transactions.

**Architecture:** Add `apps/runner` as an independent Next.js application and add a bounded `runner` domain plus `/v1/runner/demo/*` routes to the existing Rust/Axum service. The Rust backend owns quotes, roles, state transitions, completion codes, and simulated payouts; the web app renders customer, runner, and admin experiences from those contracts.

**Tech Stack:** Next.js 16.2.10, React 19, TypeScript 5.8, Rust stable, Axum, Tokio, Serde, PostgreSQL/Supabase SQL, Node's native test runner.

## Global Constraints

- The classroom demo must run without Stripe, Supabase, maps, identity documents, or external API keys.
- All money uses integer U.S. cents.
- Exact pickup and delivery addresses must not appear in public task responses.
- Applicants attest they are at least 18; real identity and background checks are out of scope.
- The Rust backend is authoritative for pricing and task transitions.
- Prohibited tasks include dangerous goods, weapons, controlled substances, cash transfers, medical emergencies, illegal requests, and work requiring entry into a private residence.
- The demo task sequence is `draft -> quoted -> funded -> available -> accepted -> picked_up -> delivering -> completed` with `cancelled`, `expired`, and `disputed` alternatives.
- BSR Runner must run as a separate website on port `3001`.

---

## File structure

- `services/core-api/src/domain/runner.rs`: pure quote inputs, price calculation, roles, task state, actions, and validation.
- `services/core-api/src/runner_demo.rs`: in-memory applications, tasks, earnings, completion-code verification, and HTTP handlers.
- `services/core-api/tests/runner_domain.rs`: quote, prohibited-task, role, and transition tests.
- `services/core-api/tests/runner_demo_api.rs`: end-to-end API tests for application, task, payout, and privacy behavior.
- `apps/runner/src/lib/contracts.ts`: web-facing API contracts, money formatting, and allowed-action labels.
- `apps/runner/src/lib/contracts.test.ts`: browser-independent contract tests.
- `apps/runner/src/app/page.tsx`: top-level screen state and API orchestration only.
- `apps/runner/src/components/*.tsx`: focused homepage, task form, market, application, tracking, earnings, and admin views.
- `apps/runner/src/app/globals.css`: standalone responsive visual system.
- `supabase/migrations/20260715001000_runner_marketplace.sql`: production-oriented schema and RLS.
- `supabase/tests/runner_marketplace.sql`: pgTAP security evidence.
- `scripts/runner-demo.sh`: start Rust demo API and Runner web app together.

### Task 1: Rust runner domain

**Files:**
- Create: `services/core-api/src/domain/runner.rs`
- Modify: `services/core-api/src/domain/mod.rs`
- Test: `services/core-api/tests/runner_domain.rs`

**Interfaces:**
- Produces: `RunnerQuoteInput`, `RunnerQuote`, `quote_runner_task(input)`, `RunnerTaskState`, `RunnerAction`, `RunnerRole`, `transition_for(role, action)`.
- Consumes: no HTTP or storage dependencies.

- [ ] **Step 1: Write failing quote and transition tests**

```rust
use core_api::domain::runner::{
    quote_runner_task, RunnerAction, RunnerQuoteInput, RunnerRole, RunnerTaskState,
    TaskCategory, Urgency, WeightBand,
};

#[test]
fn quote_is_explainable_and_uses_integer_cents() {
    let quote = quote_runner_task(RunnerQuoteInput {
        category: TaskCategory::PackagePickup,
        distance_tenths_mile: 32,
        estimated_minutes: 35,
        weight: WeightBand::Medium,
        urgency: Urgency::SameDay,
        waiting_minutes: 0,
    }).unwrap();
    assert_eq!(quote.currency, "usd");
    assert!(quote.runner_payout_cents > 0);
    assert_eq!(quote.total_cents, quote.runner_payout_cents + quote.service_fee_cents);
    assert!(!quote.explanation.is_empty());
}

#[test]
fn runner_cannot_accept_an_unfunded_task() {
    assert!(RunnerTaskState::Quoted
        .transition_for(RunnerRole::Runner, RunnerAction::Accept)
        .is_err());
}
```

- [ ] **Step 2: Run the tests and confirm they fail**

Run: `cargo test -p core-api --test runner_domain`

Expected: compilation failure because `domain::runner` does not exist.

- [ ] **Step 3: Implement the pure domain**

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerTaskState { Draft, Quoted, Funded, Available, Accepted, PickedUp, Delivering, Completed, Cancelled, Expired, Disputed }

pub fn quote_runner_task(input: RunnerQuoteInput) -> Result<RunnerQuote, RunnerError> {
    input.validate()?;
    let distance = i64::from(input.distance_tenths_mile) * 185;
    let time = i64::from(input.estimated_minutes) * 35;
    let payout = (650 + distance + time + input.weight.surcharge() + input.urgency.surcharge()
        + i64::from(input.waiting_minutes) * 30).max(1_200);
    let fee = (payout * 12 + 99) / 100;
    Ok(RunnerQuote::new(payout, fee, input))
}
```

Implement exhaustive `(state, role, action)` matching so invalid actions return `RunnerError::InvalidTransition`.

- [ ] **Step 4: Run domain quality checks**

Run: `cargo fmt --check && cargo clippy -p core-api --all-targets -- -D warnings && cargo test -p core-api --test runner_domain`

Expected: all new domain tests pass with no formatting or Clippy errors.

- [ ] **Step 5: Commit**

```bash
git add services/core-api/src/domain/mod.rs services/core-api/src/domain/runner.rs services/core-api/tests/runner_domain.rs
git commit -m "feat: add runner pricing and task domain"
```

### Task 2: Credential-free Rust runner API

**Files:**
- Create: `services/core-api/src/runner_demo.rs`
- Modify: `services/core-api/src/lib.rs`
- Modify: `services/core-api/src/demo.rs`
- Test: `services/core-api/tests/runner_demo_api.rs`

**Interfaces:**
- Consumes: `quote_runner_task`, `RunnerTaskState::transition_for` from Task 1.
- Produces: `runner_demo_router()` with applications, tasks, quote, actions, earnings, admin, and reset routes.

- [ ] **Step 1: Write failing API tests**

```rust
#[tokio::test]
async fn public_tasks_hide_exact_addresses() {
    let response = app().oneshot(Request::get("/v1/runner/demo/tasks").body(Body::empty()).unwrap()).await.unwrap();
    let body = json(response).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(body[0].get("pickup_address").is_none());
    assert!(body[0]["pickup_area"].is_string());
}

#[tokio::test]
async fn completion_code_releases_runner_payout() {
    // create -> quote -> fund -> accept -> pickup -> deliver -> complete
    // assert final task state and positive earnings balance
}
```

- [ ] **Step 2: Confirm the route tests fail**

Run: `cargo test -p core-api --test runner_demo_api`

Expected: failure because `/v1/runner/demo/tasks` is not registered.

- [ ] **Step 3: Implement in-memory state and routes**

Register:

```rust
Router::new()
    .route("/v1/runner/demo/applications", get(list_applications).post(apply))
    .route("/v1/runner/demo/applications/{id}/approve", post(approve_application))
    .route("/v1/runner/demo/quote", post(quote))
    .route("/v1/runner/demo/tasks", get(list_tasks).post(create_task))
    .route("/v1/runner/demo/tasks/{id}", get(task_detail))
    .route("/v1/runner/demo/tasks/{id}/actions", post(task_action))
    .route("/v1/runner/demo/earnings/{runner_id}", get(earnings))
    .route("/v1/runner/demo/admin", get(admin_summary))
    .route("/v1/runner/demo/reset", post(reset))
    .with_state(Arc::new(RunnerDemoState::seeded()))
```

Public task DTOs contain `pickup_area` and `dropoff_area`; private detail is returned only after a valid runner accepts the task. A completion action requires the seeded six-digit code and credits `runner_payout_cents` exactly once.

- [ ] **Step 4: Run API and full Rust tests**

Run: `cargo test -p core-api --test runner_demo_api && cargo test --workspace`

Expected: privacy, prohibited-task, conflict, completion-code, and payout tests pass; existing tests remain green.

- [ ] **Step 5: Commit**

```bash
git add services/core-api/src/lib.rs services/core-api/src/demo.rs services/core-api/src/runner_demo.rs services/core-api/tests/runner_demo_api.rs
git commit -m "feat: add runner demo API"
```

### Task 3: Runner web contracts and application shell

**Files:**
- Create: `apps/runner/package.json`
- Create: `apps/runner/tsconfig.json`
- Create: `apps/runner/next-env.d.ts`
- Create: `apps/runner/next.config.mjs`
- Create: `apps/runner/src/lib/contracts.ts`
- Create: `apps/runner/src/lib/contracts.test.ts`
- Create: `apps/runner/src/app/layout.tsx`
- Modify: `package.json`
- Modify: `package-lock.json`

**Interfaces:**
- Consumes: JSON shapes from Task 2.
- Produces: `RunnerTask`, `RunnerQuote`, `RunnerApplication`, `RunnerEarnings`, `RunnerPersona`, `money()`, `actionsFor()`.

- [ ] **Step 1: Write failing contract tests**

```ts
import assert from "node:assert/strict";
import test from "node:test";
import { actionsFor, money } from "./contracts.ts";

test("money formats integer cents", () => assert.equal(money(1845), "$18.45"));
test("only a runner accepts an available task", () => {
  assert.deepEqual(actionsFor("available", "runner"), ["accept"]);
  assert.deepEqual(actionsFor("available", "customer"), ["cancel"]);
});
```

- [ ] **Step 2: Confirm the web test fails**

Run: `node --experimental-strip-types --test apps/runner/src/lib/*.test.ts`

Expected: module-not-found failure for `contracts.ts`.

- [ ] **Step 3: Add the app package and typed contracts**

```ts
export type RunnerTaskState = "draft" | "quoted" | "funded" | "available" | "accepted" | "picked_up" | "delivering" | "completed" | "cancelled" | "expired" | "disputed";
export type RunnerPersona = "customer" | "runner" | "admin";
export const money = (cents: number) => new Intl.NumberFormat("en-US", { style: "currency", currency: "USD" }).format(cents / 100);
```

Add root scripts `runner:dev`, `runner:build`, `runner:test`, `runner:check`, and `runner:demo`.

- [ ] **Step 4: Install and verify contracts**

Run: `npm install && npm run runner:test && npm run typecheck -w @bsr-hub/runner`

Expected: two contract tests pass and TypeScript reports no errors.

- [ ] **Step 5: Commit**

```bash
git add package.json package-lock.json apps/runner
git commit -m "feat: scaffold runner web app"
```

### Task 4: Complete responsive Runner experience

**Files:**
- Create: `apps/runner/src/app/page.tsx`
- Create: `apps/runner/src/app/globals.css`
- Create: `apps/runner/src/components/HomeView.tsx`
- Create: `apps/runner/src/components/PostTaskView.tsx`
- Create: `apps/runner/src/components/TaskMarketView.tsx`
- Create: `apps/runner/src/components/ApplicationView.tsx`
- Create: `apps/runner/src/components/TaskTrackingView.tsx`
- Create: `apps/runner/src/components/EarningsView.tsx`
- Create: `apps/runner/src/components/AdminView.tsx`
- Create: `apps/runner/src/components/RunnerNav.tsx`

**Interfaces:**
- Consumes: contracts from Task 3 and `/v1/runner/demo/*` from Task 2.
- Produces: customer, runner, and admin browser flows with `data-testid` hooks for critical controls.

- [ ] **Step 1: Build the screen controller and API client**

`page.tsx` owns persona, current view, selected task, and toast state. API requests use `NEXT_PUBLIC_RUNNER_API_URL ?? "http://localhost:8080"`, preserve form values on failure, and show readable retry messages.

- [ ] **Step 2: Build customer task creation and quote UI**

The form includes category, public pickup/dropoff areas, distance, minutes, weight, urgency, waiting time, item description, and safety attestation. Quote cards show runner payout, service fee, total held, and explanation. Funding moves a created task to the public market.

- [ ] **Step 3: Build runner application, market, and tracking UI**

The application includes 18+ attestation, transportation, service radius, availability, safety agreement, and fictional-data notice. Market cards show only approximate public information. Tracking buttons render solely from `actionsFor(state, persona)`.

- [ ] **Step 4: Build earnings and admin UI**

Earnings summarize available balance, completed tasks, current-week income, and transaction history. Admin shows applications, active jobs, disputes, prohibited-task rejections, and platform totals.

- [ ] **Step 5: Add the visual system and responsive behavior**

Use a distinctive warm orange, ink, cream, and lime palette; large readable typography; high-contrast focus states; 44px touch targets; card layouts that collapse below 760px; and no horizontal overflow at 390px.

- [ ] **Step 6: Run the web quality gate**

Run: `npm run runner:check`

Expected: native tests, TypeScript, and the Next.js production build pass.

- [ ] **Step 7: Commit**

```bash
git add apps/runner
git commit -m "feat: build BSR Runner web experience"
```

### Task 5: Production-oriented database security

**Files:**
- Create: `supabase/migrations/20260715001000_runner_marketplace.sql`
- Create: `supabase/tests/runner_marketplace.sql`
- Create: `docs/database/runner-state-matrix.md`

**Interfaces:**
- Produces: runner application, task, assignment, evidence, payout, dispute, and review tables with RLS.
- Consumes: exact states and roles from Task 1.

- [ ] **Step 1: Write failing pgTAP assertions**

Assert RLS is enabled, public users cannot read exact addresses, runners cannot approve their own applications, only assigned runners can create task evidence, and payouts cannot be directly inserted by clients.

- [ ] **Step 2: Add schema, constraints, and policies**

Use enums/check constraints matching the Rust strings, `bigint` cents columns with non-negative checks, foreign keys, timestamps, unique active assignment, and policies based on `auth.uid()`.

- [ ] **Step 3: Document the state and actor matrix**

Record every source state, action, target state, permitted actor, and payout/address side effect. The matrix must match `RunnerTaskState::transition_for` exactly.

- [ ] **Step 4: Verify SQL statically**

Run: `rg -n "enable row level security|create policy|pickup_address|payout_cents" supabase/migrations/20260715001000_runner_marketplace.sql`

Expected: each sensitive table has RLS and private fields are referenced only in restricted policies.

- [ ] **Step 5: Commit**

```bash
git add supabase/migrations/20260715001000_runner_marketplace.sql supabase/tests/runner_marketplace.sql docs/database/runner-state-matrix.md
git commit -m "feat: secure runner marketplace schema"
```

### Task 6: Demo runner, CI, documentation, and browser acceptance

**Files:**
- Create: `scripts/runner-demo.sh`
- Modify: `.github/workflows/ci.yml`
- Modify: `README.md`
- Create: `docs/qa/runner-end-to-end-results.md`
- Create: `docs/presentation/runner-demo-script.md`

**Interfaces:**
- Consumes: all previous tasks.
- Produces: one-command demo, automated CI, presentation handoff, and browser evidence.

- [ ] **Step 1: Add the one-command runner**

```bash
#!/usr/bin/env bash
set -euo pipefail
export BSR_DEMO_MODE=true
cargo run -p core-api &
API_PID=$!
trap 'kill "$API_PID" 2>/dev/null || true' EXIT INT TERM
npm run dev -w @bsr-hub/runner -- --port 3001
```

- [ ] **Step 2: Extend CI and README**

CI runs `npm run check` and `npm run runner:check`. README explains requirements, `npm run runner:demo`, the demo personas, safety limitations, and the classroom completion code.

- [ ] **Step 3: Run fresh verification**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && npm audit --audit-level=moderate && npm run check && npm run runner:check && git diff --check`

Expected: exit code 0 for every command, zero Rust or web test failures, zero audit findings at moderate-or-higher severity, and no whitespace errors.

- [ ] **Step 4: Execute the browser walkthrough**

Start `npm run runner:demo`, open `http://localhost:3001`, then prove customer creation/funding, public-address privacy, runner application/approval, market acceptance, pickup, delivery, completion code, payout, prohibited-task rejection, and mobile layout. Record exact results in `docs/qa/runner-end-to-end-results.md`.

- [ ] **Step 5: Commit the release evidence**

```bash
git add scripts/runner-demo.sh .github/workflows/ci.yml README.md docs/qa/runner-end-to-end-results.md docs/presentation/runner-demo-script.md
git commit -m "chore: complete BSR Runner demo release"
```

- [ ] **Step 6: Push the branch**

Run: `git push -u origin codex/bsr-runner`

Expected: GitHub confirms the new branch and starts the quality workflow.
