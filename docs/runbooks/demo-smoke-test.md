# BSR Hub PS5 demo smoke test

Use two test accounts and Stripe test mode.

1. Seller lists a PS5 at $25 per unit, $100 deposit, and optional $15 delivery.
2. Buyer requests two units with delivery. Confirm the API returns $50 base, $3 service fee, $15 delivery, $100 deposit, and $168 total.
3. Buyer creates an order and receives a 30-minute reservation plus a Stripe test checkout URL.
4. Try the same booking in a second session and confirm it returns `409 LISTING_UNAVAILABLE`.
5. Complete Stripe test checkout and confirm one signed webhook changes the order to paid; replaying the same event must not apply it twice.
6. Seller confirms. Buyer/seller move the order through active or fulfilled, returned when applicable, and completed.
7. Repeat the quote and checkout start at a mobile viewport.

Do not capture names, tokens, private street addresses, or payment credentials in screenshots.

