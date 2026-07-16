# BSR Hub PS5 demo smoke test

Use two test accounts and Stripe test mode.

Before a persistent-backend demo, run `npm run mongo:check`. It proves the seed is idempotent and exercises actual replica-set transactions; a normal GitHub Pages classroom demo continues to use fictional in-browser data.

1. Seller enters PS5 model, age, condition, cleanliness, operational status, missing features, controllers, and selects 30-minute billing. Confirm Rust returns a `rules-v1` recommended unit price and only permits a seller adjustment between -$5 and +$5.
2. Buyer requests a 31-minute rental with delivery. Confirm the API calculates two billable 30-minute units itself and returns the unit price, base, 6% service fee, delivery, deposit, and total. Confirm adding a client-controlled `units` field returns `422`.
3. Buyer creates an order and receives a 30-minute reservation plus a Stripe test checkout URL.
4. Try the same booking in a second session and confirm it returns `409 LISTING_UNAVAILABLE`.
5. Complete Stripe test checkout and confirm one signed `checkout.session.completed` webhook changes the order to paid. Replaying the event must not apply it twice; an unpaid, wrong-amount, wrong-currency, unknown-order, or late event must not mark an order paid.
6. Seller confirms. Buyer/seller move the order through active or fulfilled, returned when applicable, and completed.
7. Repeat the quote and checkout start at a mobile viewport.

Do not capture names, tokens, private street addresses, or payment credentials in screenshots.
