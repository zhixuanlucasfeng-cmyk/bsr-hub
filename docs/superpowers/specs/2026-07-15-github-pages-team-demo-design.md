# GitHub Pages Team Demo Design

## Goal

Publish BSR Hub and BSR Runner from the public `bsr-hub` repository so teammates can open both products on any device without installing Node, Rust, Supabase, or Stripe.

## Chosen approach

GitHub Pages will host one static bundle with a small product selector at the root, BSR Hub under `/bsr-hub/hub/`, and BSR Runner under `/bsr-hub/runner/`. GitHub Actions will rebuild and deploy the bundle after relevant pushes.

The production-oriented Rust API remains the authoritative implementation in the repository. Pages cannot run Rust, so the public review build will use explicitly labeled, in-browser fictional demo stores. The static stores implement the same money shapes and allowed state transitions needed for the presentation, without real identities, addresses, payments, or network writes.

## UI and navigation

- Root: a polished BSR product-family page with two clear cards and links.
- Hub: existing marketplace UI, listings, quotes, reservations, persona switching, order progress, and listing preview.
- Runner: existing customer, runner, and admin views; task publishing, accepting, pickup, delivery, completion, earnings, and safety rejection.
- Every public page identifies itself as a fictional classroom demo.
- All asset and navigation URLs must work beneath the GitHub repository base path.

## Build architecture

- Both Next.js apps use `output: "export"`.
- `NEXT_PUBLIC_BASE_PATH` provides `/bsr-hub/hub` or `/bsr-hub/runner` during the Pages build and is empty during local development.
- `NEXT_PUBLIC_STATIC_DEMO=true` selects the in-browser demo adapter during the Pages build; local `npm run demo` continues to call the Rust API.
- The workflow combines both `out/` directories plus the root selector into one Pages artifact.
- Pages deployment uses GitHub’s official `configure-pages`, `upload-pages-artifact`, and `deploy-pages` actions with minimal permissions.

## Safety and scope

- Static mode never sends personal information or payment data.
- Exact addresses are fictional and remain hidden until the correct simulated assignment state.
- No authentication, payment, background check, or legal compliance is implied.
- This work publishes the classroom demo only; it does not replace the Rust API or production database.

## Verification

- Unit tests cover local static quotes and state transitions.
- Both apps pass type checks, unit tests, and static export builds.
- The combined artifact contains root, Hub, and Runner entry points.
- Browser checks cover both public URLs at desktop and 390-pixel mobile width.
- The GitHub Pages endpoint must return HTTP 200 before completion is reported.
