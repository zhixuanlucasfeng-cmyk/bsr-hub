# BSR Runner Product Design

**Status:** Approved  
**Date:** 2026-07-15  
**Product:** A separate local-task marketplace connected to the BSR ecosystem

## Purpose

BSR Runner helps adults who need flexible income find nearby delivery and errand work. Customers can publish ordinary local tasks, while BSR Hub can send rental and second-hand delivery jobs into the same task market. Runners choose when and where they work; they are not assigned fixed shifts.

The classroom demo must be usable without real identity documents, payments, maps, or production credentials. It must still demonstrate the real business rules clearly enough that the team can explain how a production system would work.

## Users

### Customers

Customers create tasks, receive an automatic quote, fund a protected demo payment, track progress, confirm delivery with a completion code, and review the runner.

### Runners

Applicants confirm that they are at least 18, provide fictional demo verification data, choose a transportation method and service area, and accept platform rules. Approved demo runners browse available jobs, accept one, record pickup evidence, complete delivery, and view earnings.

### Administrators

Administrators review runner applications, see active jobs, respond to reports, and suspend unsafe tasks or accounts. The classroom version simulates these actions without collecting sensitive identity data.

## Scope

### Task types

- BSR Hub rental delivery
- BSR Hub second-hand delivery
- Package pickup
- Grocery or store pickup
- Document delivery
- Small-item local delivery
- Other lawful local errands approved by platform rules

The platform rejects illegal requests, dangerous goods, weapons, controlled substances, cash transfer requests, medical emergencies, and tasks that require entering a private residence.

### Automatic pricing

The Rust pricing engine calculates a quote from:

- estimated distance;
- estimated duration;
- item weight band;
- task category;
- urgency;
- transportation method;
- optional waiting time;
- platform service fee.

Money is represented in integer U.S. cents. Customers cannot freely underprice a task. The response includes a readable price explanation.

### Protected payment

The customer funds the task before it becomes available. BSR Runner holds the payment until the customer enters or shares the completion code after delivery. The demo simulates this escrow flow; it never charges a real card.

## Job state machine

The Rust backend is authoritative for valid actions.

`draft -> quoted -> funded -> available -> accepted -> picked_up -> delivering -> completed`

Alternative terminal states are `cancelled`, `expired`, and `disputed`. Only state-appropriate actors can perform each transition. Completion releases the demo payout to the runner's earnings balance.

## Safety and privacy

- Applicants must attest that they are at least 18.
- Identity and background review are simulated in the classroom demo.
- Exact pickup and delivery addresses remain hidden until a runner accepts a funded task.
- Public cards show only city, neighborhood, approximate distance, category, payout, and time estimate.
- Pickup evidence and completion codes reduce false completion claims.
- Customers and runners can report unsafe behavior or open a dispute.
- Emergency language tells users to contact emergency services rather than publish urgent safety or medical tasks.
- Production launch requires legal review of worker classification, insurance, taxes, background screening, accessibility, privacy, and local delivery regulations.

## User experience

### Public homepage

The homepage explains flexible local earnings and trustworthy neighborhood help. It provides clear entry points for **Post a task** and **Become a runner**, plus sample tasks, safety promises, earnings examples, and the relationship to BSR Hub.

### Customer flow

1. Select a task type.
2. Enter public areas, item details, urgency, distance, and time estimate.
3. Receive a Rust-generated quote.
4. Fund the protected demo payment.
5. Track runner acceptance, pickup, delivery, and completion.
6. Confirm with a completion code and leave a review.

### Runner flow

1. Submit an 18+ demo application.
2. Receive simulated approval.
3. Browse jobs using category, distance, payout, and transport filters.
4. Accept one available job.
5. Follow the Rust-controlled pickup and delivery actions.
6. Complete the job and see the payout in the earnings dashboard.

### Admin flow

The admin view summarizes application status, task state, disputes, unsafe-content flags, and platform volume. Demo actions are clearly marked and reversible by resetting the server.

## Architecture

### Web application

`apps/runner` is a separate Next.js and TypeScript application with its own visual identity and development port. It reuses shared conventions from BSR Hub but does not appear as a page inside the existing marketplace.

### Rust service

The existing Axum workspace gains a bounded runner module for quotes, applications, tasks, actions, earnings, and demo reset. Domain functions remain separate from HTTP handlers so pricing and state transitions can be unit tested.

### Database

Production-oriented Supabase migrations define runner profiles, applications, tasks, assignments, evidence metadata, payouts, disputes, and reviews. Row-level security separates public task discovery from private addresses and identity-review fields.

### Demo mode

`BSR_DEMO_MODE=true` serves fictional runner data from memory. Restarting or calling the reset endpoint restores the known classroom scenario. No Stripe, Supabase, mapping service, or API key is required.

## Error handling

- Invalid quotes return field-specific validation messages.
- Unsafe or prohibited task categories are rejected before funding.
- Conflicting task acceptance returns a clear already-taken response.
- Invalid role or state actions return the current state and allowed actions.
- Network errors preserve form input and offer retry.
- Demo reset restores a predictable initial state.

## Testing and acceptance

The quality gate includes Rust formatting, Clippy, domain tests, API tests, TypeScript checks, web unit tests, a production build, and a browser walkthrough.

The final browser walkthrough must prove:

1. a customer creates and funds an automatically priced errand;
2. an approved runner accepts it;
3. exact addresses are not visible on the public job card;
4. pickup and delivery transitions follow the Rust state machine;
5. the completion code releases the simulated payout;
6. the runner sees updated earnings;
7. an applicant can complete the demo approval flow;
8. prohibited tasks are rejected;
9. the site remains usable at mobile width.

## Out of scope for the classroom release

- Real payments or payouts
- Real identity documents or background checks
- Live GPS tracking
- Turn-by-turn navigation
- Production insurance and tax processing
- Employee scheduling
- Multi-state legal compliance claims

These boundaries keep the two-week build credible while preserving a clear path to production.
