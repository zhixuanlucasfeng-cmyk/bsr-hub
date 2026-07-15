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
