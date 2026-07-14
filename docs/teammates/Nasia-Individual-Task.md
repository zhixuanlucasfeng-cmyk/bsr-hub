# Nasia — Individual Task Instructions

## Your Role

You are responsible for **transaction interfaces, end-to-end quality assurance, and the final presentation**. You connect the visible marketplace pages to the Rust quote and order flow, then make sure the team can demonstrate the product clearly.

## Main Coding Responsibilities

### 1. Booking and checkout interface

Build the user interfaces for:

- selecting rental or workspace dates and times;
- choosing pickup, delivery, owner-location use, or on-site use;
- showing base price, deposit, delivery fee, service fee, and total;
- accepting a Rust-generated quote;
- opening Stripe test Checkout;
- returning to success, cancellation, or retry pages.

The browser must never calculate or override the authoritative total. Display only the quote returned by the Rust API.

### 2. Order dashboards

Build:

- buyer order list and order detail;
- seller order list and order detail;
- seller confirmation;
- delivery status updates;
- active/use status;
- return or fulfillment confirmation;
- completion and cancellation display;
- review form and review summary.

Only show actions allowed by the current order state and the signed-in user's relationship to the order.

### 3. Error states

Design and test clear interfaces for:

- dates becoming unavailable;
- payment cancellation or failure;
- expired 30-minute reservation;
- Rust API unavailable;
- user signed out;
- unauthorized action;
- missing listing;
- delivery unavailable for the selected listing.

## Quality Assurance Responsibilities

Maintain the shared end-to-end checklist. Test with two separate accounts: one owner and one customer.

The required demonstration path is:

```text
Sign in → Find PS5 → Choose dates and delivery → Receive Rust quote
→ Complete Stripe test checkout → Seller confirms → Rental becomes active
→ Return item → Complete order → Leave review
```

Test it on both desktop and mobile. Record the exact step and screenshot for every failure.

## Presentation Responsibilities

- Lead the two-minute team presentation.
- Assign and rehearse each person's speaking section.
- Prepare a short live-demo script with exact clicks and test credentials.
- Prepare backup screenshots and a screen recording.
- Explain the problem, product, team roles, UN Goal 8, and UN Goal 10.
- Keep the presentation within the teacher's time limit.

## Ten-Day Schedule

- **Day 1:** Confirm demo story, transaction wireframes, API contract, and presentation structure.
- **Day 2:** Prepare checkout and dashboard component skeletons.
- **Day 3:** Build date/time and fulfillment selection UI.
- **Day 4:** Connect quote display to the Rust endpoint and add error states.
- **Day 5:** Connect order creation and Stripe test Checkout.
- **Day 6:** Run the complete PS5 journey; fix only integration blockers.
- **Day 7:** Complete dashboards, delivery, return/fulfillment, and reviews.
- **Day 8:** Execute desktop/mobile QA and prepare backup assets.
- **Day 9:** No new features; rehearse, record the demo, and fix critical issues.
- **Day 10:** Run smoke tests, coordinate speakers, and deliver the presentation.

## Required Deliverables

- Booking, quote, checkout, success, cancellation, and retry pages.
- Buyer and seller order dashboards.
- Delivery, return, fulfillment, completion, and review interfaces.
- End-to-end test checklist with results.
- Final speaking script and live-demo script.
- Backup screenshots and screen recording.

## Working With Claude

```text
Work only on BSR Hub transaction and order UI inside apps/web.
Use the approved shared contracts. Never calculate the final price in the browser;
display the Rust quote. Include loading, expired, unavailable, payment-cancelled,
unauthorized, and API-offline states. Add relevant tests, run them, and summarize
every changed file. Do not modify migrations, Rust code, or environment secrets.
```

## File and Security Rules

- Do not include real payment cards, passwords, private addresses, or API keys in screenshots or recordings.
- Use Stripe's official test data only.
- Do not commit `.env`, generated build folders, or test credentials.
- Coordinate shared component edits with Anna.
- Send Lucian source changes, QA evidence, and presentation assets for the daily backup, excluding secrets and generated dependencies.

## Definition of Done

Nasia's work is complete when two test users can finish the entire PS5 transaction on mobile and desktop, every failure has a clear recovery path, and the team can deliver the presentation with a working live demo and backup recording.
