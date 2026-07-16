# BSR Hub Performance, Links, and Direct Routes Design

**Date:** 2026-07-16  
**Status:** Approved by the user (Approach A)

## Goal

Reduce first-load latency, display marketplace photography without square distortion, turn every footer item into a working destination, and make `/hub/orders/` and `/hub/create/` valid GitHub Pages entry routes.

## Root Causes

- The image library is 4.06 MiB of JPEG files, mostly 1,600 × 1,100 pixels.
- The home page eagerly loads three hero images plus the first four listing images.
- The global stylesheet imports Google Fonts at runtime, adding an external render dependency that can be slow or unavailable in China.
- Footer items are plain `<p>` elements and have no URLs.
- Orders and listing creation exist only as internal React state on `/hub/`; no static `/orders/` or `/create/` routes are exported.

## Image Pipeline

- Add a deterministic Node preprocessing script using `sharp`.
- Produce committed WebP variants from the current JPEG originals:
  - `card`: 720 × 540, 4:3 crop for listing and hero cards.
  - `detail`: maximum 1,440 pixels wide while preserving the source aspect ratio.
- Do not use 1:1 square presentation for primary images.
- Listing and hero cards use the 4:3 card asset with explicit width/height, `srcset`, `sizes`, and asynchronous decoding.
- Detail galleries use the natural 16:11-style source ratio with `object-contain` on a neutral background so the full subject remains visible.
- Only hero imagery and the first visible listing may load eagerly. All remaining marketplace and showcase images load lazily.
- Remove the Google Fonts network import and use a local system-font stack to remove the external font request.

## Footer Destinations

Every footer item becomes an anchor with hover and keyboard focus states.

### Marketplace

- Explore nearby → `/hub/#market`
- Rent products → `/hub/?type=rental#market`
- Book spaces → `/hub/?type=workspace#market`
- Second-hand → `/hub/?type=sale#market`

The home page reads the `type` query parameter so these links apply the matching filter.

### Trust & Help

A static `/hub/help/` page contains accessible sections for:

- Protected payment → `/hub/help/#protected-payment`
- Help center → `/hub/help/`
- Terms of service → `/hub/help/#terms`
- Privacy policy → `/hub/help/#privacy`

The terms and privacy copy are clearly labeled as classroom-demo information, not production legal advice.

### Impact

- UN SDG 8 & 10 → official United Nations Goal 8 and Goal 10 pages.
- Babson Summer Study → `https://www.babson.edu/summer-at-babson/high-school-learners/summer-study/`
- Contact the BSR team → the repository's GitHub Issues page.
- Social channels → the public BSR Hub GitHub repository.

External destinations open in a new tab with `rel="noreferrer"`.

## Direct Entry Routes

- `/hub/orders/` and `/hub/create/` are real statically exported Next.js routes.
- Each route renders a lightweight client redirect to `/?intent=orders` or `/?intent=create`, preserving the existing demo-session guard and avoiding duplicated marketplace state logic.
- Each route includes a visible fallback link for browsers that block JavaScript.
- GitHub Pages artifact tests assert that `orders/index.html` and `create/index.html` exist.

## Performance and Accessibility Checks

- Assert generated WebP files exist and remain below defined byte limits.
- Assert no Google Fonts URL remains in production CSS.
- Assert only one listing is marked eager on the home page.
- Assert all footer items are links with valid internal or HTTPS destinations.
- Verify desktop, tablet, and mobile layouts have no horizontal overflow.
- Verify direct route refreshes, filtered footer links, help anchors, and public GitHub Pages URLs.

## Non-goals

- No image CDN or paid service.
- No production authentication, real payment, or legal-policy system.
- No changes to Rust pricing or order-state rules.
