# BSR Hub End-to-End Verification

**Date:** 2026-07-15  
**Environment:** Next.js 16.2.10 at `http://localhost:3000`, Rust demo API at `http://localhost:8080`  
**Result:** Passed

## Marketplace walkthrough

The complete PS5 rental journey was exercised in Chrome against the running application:

1. Maya opened **PS5 Slim + Two Controllers**.
2. She selected two 30-minute units and local delivery.
3. The Rust pricing endpoint returned:
   - Rental price: `$24.00`
   - BSR service fee (6%): `$1.44`
   - Delivery: `$8.00`
   - Refundable deposit: `$100.00`
   - Total held by BSR: `$133.44`
4. Maya created the reservation and completed the protected demo payment.
5. Jordan confirmed the order and started the rental.
6. Maya returned the item.
7. Jordan completed the order.
8. The completed order exposed the review action, and the five-star demo review was saved.

Observed authoritative state sequence:

`pending_payment -> paid -> confirmed -> active -> returned -> completed`

Each transition was accepted by the Rust state machine and immediately reflected in the web dashboard.

## Seller walkthrough

Jordan successfully opened the listing form and saved a private-preview rental listing with a title, description, rate, deposit, and public city. The interface confirmed that the listing was saved and that exact addresses remain private.

## Coverage confirmed

- Rental products
- Workspaces and studios
- Second-hand sales
- Pickup, owner-location use, and local delivery choices
- Rust-calculated price breakdowns and 6% service fee
- Deposits and protected-payment language
- Buyer and seller dashboards
- Role-specific valid order actions
- Reviews
- User-created listing form
- Public city only; exact-address privacy copy
- UN Sustainable Development Goals 8 and 10 messaging

This test uses fictional demo data and no real payment information.
