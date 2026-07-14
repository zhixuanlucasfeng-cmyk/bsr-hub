# Yicheng Rust Logic Review

- Integer cents and checked arithmetic are used for base, service, delivery, deposit, and total.
- Billing units are server-derived in the production quote path and explicit deterministic demo units are validated as positive.
- Seller recommendation adjustment is bounded to ±$5 per billing unit.
- The order state machine matches the approved matrix and rejects every unspecified transition.
- Stripe webhook verification checks signature age, event ID, payment status, order reference, amount, currency, and reservation expiration.
- Demo payment is visibly labeled and calls the same `OrderState::transition` rule; it never stores a card number.
- Remaining production requirement: execute pgTAP and true concurrent overlap tests against the team's Supabase project when credentials become available.
