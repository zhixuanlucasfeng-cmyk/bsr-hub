# BSR Hub Image and Cache Performance Design

**Date:** 2026-07-16  
**Status:** Approved approach B  
**Scope:** BSR Hub, BSR Runner, and the GitHub Pages product entry page

## Objective

Reduce repeat-visit latency and unnecessary image transfer without visibly lowering image quality or breaking the current static GitHub Pages deployment. Preserve every source image as the archival original while serving smaller responsive derivatives to browsers.

MongoDB persistence is a separate implementation stream. This performance work must not change API contracts or database behavior. After this work is verified, development resumes from `docs/superpowers/plans/2026-07-16-mongodb-core-persistence.md` and its approved architecture in `docs/superpowers/specs/2026-07-16-mongodb-primary-database-design.md`.

## Evidence and Constraints

- The deployed entry page returned a first-byte time of about 2.3 seconds during the audit.
- The deployed Hub and a sample image returned first-byte times of about 3.7 to 4.0 seconds during separate cold connections.
- GitHub Pages returned `Cache-Control: max-age=600`, which provides only a ten-minute browser cache lifetime.
- The repository already contains responsive WebP derivatives, but they are encoded at quality 76 to 82 and there is no application-managed persistent cache.
- A representative 960 x 720 image was 143,698 bytes with the current WebP settings, 631,924 bytes with near-lossless WebP, and 820,178 bytes with mathematically lossless WebP. True lossless WebP would increase transfer size and worsen latency.
- GitHub Pages does not allow this project to define custom long-lived response headers.
- The static deployment must continue to work under `/bsr-hub/`, `/bsr-hub/hub/`, and `/bsr-hub/runner/`.

## Chosen Strategy

Use a quality-preserving responsive image pipeline plus a scoped service worker.

1. Keep original JPEG and PNG files unchanged as archival and fallback sources.
2. Generate responsive WebP derivatives for actual browser delivery. Use explicit dimensions and conservative quality settings that preserve visual detail while enforcing byte budgets.
3. Continue using `<picture>` and `srcset` so browsers receive only the size required by the current viewport.
4. Add a service worker that creates persistent Cache Storage for the product shells and assets.
5. Remove runtime Google Fonts requests from the entry page and Runner so the first render does not require another origin and TLS connection.
6. Preload only the primary Hub hero image. All non-critical images remain lazy and asynchronously decoded.

The phrase “lossless compression” is implemented as lossless source preservation plus visually lossless delivery derivatives. Mathematically lossless WebP is rejected because the measured output is about 5.7 times larger than the current derivative.

## Image Pipeline

### Source preservation

Files under `apps/web/public/images/listings` and `apps/web/public/images/categories` remain unchanged. The optimizer never overwrites them.

### Generated variants

For every supported source, the optimizer produces:

- `card-sm`: 480 x 360 WebP for phones and compact cards.
- `card-lg`: 960 x 720 WebP for desktop cards and hero scenes.
- `detail`: up to 1440 pixels wide WebP for listing galleries.

Generation is deterministic: auto-rotation is applied, card crops use the existing attention focal strategy, metadata is excluded, and the same input/settings produce the same pixels. The optimizer prints source, generated, and per-variant totals.

### Budgets

- `card-sm`: at most 90 KiB.
- `card-lg`: at most 180 KiB.
- `detail`: at most 270 KiB.
- The first hero image should be below 40 KiB at its small variant.

The performance contract fails when an output is missing or exceeds its budget. This prevents future uploads from silently making the site slow.

### Rendering behavior

- The first hero image is eager, high priority, and preloaded.
- Other hero images and all below-the-fold cards are lazy.
- Width, height, `sizes`, and `srcset` remain explicit to avoid layout shift and oversized downloads.
- The original image remains the fallback for browsers without WebP support.

## Cache Architecture

### Service-worker scope

Use one service worker at the GitHub Pages root, `/bsr-hub/sw.js`, with scope `/bsr-hub/`. It controls the entry page, Hub, and Runner without creating competing workers.

The static build copies a versioned worker into `dist-pages/sw.js`. Hub, Runner, and the entry page register it only in production-capable browsers. Registration failure is non-fatal and never blocks rendering.

### Cache policies

- **Hashed Next.js assets:** cache first. Their filenames are content-addressed, so cached responses are safe to reuse.
- **Images and brand assets:** stale while revalidate. Return the cached response immediately, then refresh it in the background so unchanged URLs can still receive new content.
- **HTML navigations:** network first with a short fallback to the last cached shell. This avoids trapping users on an outdated page while still providing a fast/offline fallback.
- **Third-party requests and API requests:** never cached by this worker.

Only successful same-origin `GET` responses under `/bsr-hub/` are cached. Requests containing authentication headers, non-GET requests, and API-like paths are excluded.

### Versioning and cleanup

Cache names include a schema version. On activation, the worker deletes only old BSR cache names and leaves unrelated browser caches untouched. Updating the cache schema requires changing the version constant.

## Font and Connection Changes

- Remove `fonts.googleapis.com` imports from `deploy/pages/styles.css` and `apps/runner/src/app/globals.css`.
- Use existing system-font stacks so text renders immediately.
- Do not add external CDN dependencies for runtime assets.
- Keep all critical Hub images on the same GitHub Pages origin to reuse the existing connection.

## Components and Files

Expected implementation boundaries:

- `scripts/optimize-images.mjs`: deterministic responsive derivative generation and reporting.
- `scripts/check-performance-links.mjs`: image budget, service-worker, registration, font, and preload contracts.
- `apps/web/src/lib/image-assets.ts`: stable mapping from source image to generated variants.
- `apps/web/src/components/OptimizedImage.tsx`: responsive image behavior and priority attributes.
- `apps/web/src/components/PerformanceBoot.tsx`: client-only service-worker registration for Hub.
- `apps/runner/src/components/PerformanceBoot.tsx`: client-only service-worker registration for Runner.
- `apps/web/src/app/layout.tsx` and `apps/runner/src/app/layout.tsx`: install the registration component.
- `deploy/pages/index.html`: register the root worker for the entry page.
- `deploy/pages/styles.css` and `apps/runner/src/app/globals.css`: remove runtime font imports.
- `deploy/pages/sw.js`: source service worker copied to the deployment root.
- `scripts/build-pages.sh`: verify that the worker is present in the artifact.

The registration components contain no marketplace state. They only register the worker after the page becomes interactive.

## Error Handling and Safety

- A browser without service-worker support continues normally.
- Registration errors are reported only in development diagnostics and do not show user-facing errors.
- Failed image requests continue to use existing fallbacks.
- The service worker does not cache payment callbacks, authentication data, API responses, or cross-origin resources.
- Source images are never deleted or rewritten.
- The current routes and static-export structure remain unchanged.

## Testing and Verification

Testing follows red-green-refactor.

1. Extend the performance contract first so it fails because the worker, registrations, font removal, and preload are absent.
2. Implement the smallest changes that satisfy each failure.
3. Regenerate images and verify every derivative and byte budget.
4. Run Hub tests, type checking, and production export.
5. Run Runner tests, type checking, and production export.
6. Build the combined GitHub Pages artifact and verify the three public entry points plus `dist-pages/sw.js`.
7. Serve `dist-pages` locally and verify service-worker registration, cache population, repeat navigation, responsive images, and no console errors in a browser.
8. Re-measure deployed headers and timing after publishing. The service worker is expected to improve repeat visits; it cannot remove GitHub Pages’ first-connection regional latency.

## Acceptance Criteria

- Source images are unchanged.
- Every referenced listing/category image has all required WebP variants.
- Generated variants remain within the budgets.
- Only the primary hero image is eager/high priority and it is preloaded.
- No runtime Google Fonts request remains in the entry page, Hub, or Runner.
- A single root-scoped service worker controls all three static experiences.
- Repeat visits can serve hashed assets and images from Cache Storage.
- HTML remains network-first and API/auth/payment traffic is excluded.
- Hub and Runner tests, type checks, builds, and the combined Pages build pass.
- Existing marketplace routes and MongoDB work are not regressed.

