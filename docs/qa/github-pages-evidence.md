# GitHub Pages public demo evidence

Verified on 2026-07-15 against commit `9f37c57` and GitHub Actions run [29402937939](https://github.com/zhixuanlucasfeng-cmyk/bsr-hub/actions/runs/29402937939).

## Deployment

- Repository quality gate: passed.
- Pages build job: passed.
- Pages deploy job: passed after enabling GitHub Actions as the Pages source and allowing `codex/bsr-runner` in the `github-pages` environment.
- Product selector: HTTP 200.
- BSR Hub: HTTP 200.
- BSR Runner: HTTP 200.

## Browser checks

All checks used the deployed GitHub Pages site with a 390 × 844 mobile viewport and fictional in-browser demo data.

### Product selector

- Hub and Runner relative links were present.
- One-column mobile layout rendered without horizontal overflow.
- Prototype disclosure was visible.
- No console errors were recorded.

### BSR Hub

- All 12 fictional listings rendered.
- PS5 Slim quote calculated two 30-minute units as $24.00, a 6% service fee as $1.44, and a refundable deposit as $100.00.
- A protected demo reservation was created in `pending_payment` state.
- No console errors or horizontal overflow were recorded.

### BSR Runner

- All four fictional task cards rendered.
- The first task exposed only public areas before acceptance.
- After Jordan accepted, only the assigned runner saw `1 Fictional Pickup` and `1 Demo Dropoff`.
- The task completed through accepted, picked-up, delivering, and completed states with classroom code `482731`.
- Completion released the $30.33 fictional runner payout.
- No console errors or horizontal overflow were recorded.

No real payments, addresses, identities, applications, or employment records were used.

## Image and persistent-cache verification — 2026-07-16

Verified the production-shaped `dist-pages` artifact at commit `03da49c` under the real `/bsr-hub/` base path.

- All 19 source photographs remained byte-for-byte unchanged; the optimizer generated 57 responsive WebP derivatives.
- The complete derivative set is 3,726,744 bytes: small cards use an 80-quality detail-preserving profile, while large cards and detail images use quality 84.
- Every image stayed inside the enforced budgets: 90 KiB for small cards, 180 KiB for large cards, and 270 KiB for detail images.
- Hub preloads only the first PS5 hero/card image; the remaining listing images stay lazy.
- No production CSS requests Google Fonts, removing a blocking third-party connection.
- The production artifact includes `/bsr-hub/sw.js`, which registered successfully in the local browser and precached `/bsr-hub/`, `/bsr-hub/hub/`, and `/bsr-hub/runner/`.
- A second controlled Hub reload requested no hashed `/_next/static/` JavaScript or CSS from the server, confirming cache-first reuse. Images used stale-while-revalidate and navigations used network-first with a cached fallback.
- Hub and Runner both rendered successfully from the production artifact with their expected titles, navigation, listings/tasks, and trust disclosures.
- Sensitive or dynamic paths (`/api/`, `/auth/`, `/checkout/`, `/payment/`, and `/payments/`) and requests carrying authorization headers are explicitly excluded from the cache.
