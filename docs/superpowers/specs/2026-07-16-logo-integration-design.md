# BSR Logo Website Integration Design

## Goal

Replace the temporary text-and-letter brand treatments in BSR Hub and BSR Runner with the approved BSR logo family across navigation, footer, and browser metadata.

## Approved Scope

- BSR Hub top navigation uses the icon-only SVG plus an accessible `BSR Hub` label.
- BSR Hub footer uses the horizontal Hub logo on a light presentation surface that remains readable against the dark footer.
- BSR Runner top navigation uses the icon-only SVG plus an accessible `BSR Runner` label.
- BSR Runner footer uses the horizontal Runner logo.
- Both applications use `bsr-icon.png` as their browser icon metadata.
- Mobile layouts reduce the rendered logo dimensions without hiding the product identity.

## Asset Strategy

The shared source assets remain under `apps/web/public/brand/`. Runner receives its own copied public assets under `apps/runner/public/brand/` because each Next.js static export has an independent public root. Components prefix asset paths with `NEXT_PUBLIC_BASE_PATH` so local builds and GitHub Pages subpaths both work.

## Component Changes

- Add a small reusable `BrandLogo` component in each application. It accepts `variant: "icon" | "horizontal"`, renders a normal `img` with fixed intrinsic dimensions, and supplies useful alternative text.
- Update Hub navigation and footer to use the Hub component.
- Update Runner navigation and footer to use the Runner component.
- Remove styling that creates the old `B`, `BSR`, or orange brand blocks; preserve unrelated navigation and footer styling.
- Add Next.js metadata icons in both root layouts.

## Responsive and Accessibility Rules

- Navigation logos remain buttons with the existing home action and an accessible home label.
- Hub navigation height remains 76 pixels; Runner topbar height remains 76 pixels.
- Icon height is 42 pixels on desktop and 36 pixels on screens below 650 pixels.
- Horizontal footer logos have a maximum width and preserve aspect ratio.
- Logos never shrink below legible dimensions or create horizontal page overflow.

## Verification

- Add pure component render checks only if the existing test environment supports React rendering without new dependencies; otherwise use TypeScript/build checks plus browser inspection.
- Run `npm run pages:check` to verify Hub, Runner, and static GitHub Pages output.
- Serve the Pages artifact using the `/bsr-hub/` directory shape and inspect Hub and Runner at desktop and mobile widths.
- Confirm icon and wordmark assets return successfully from both exported application paths.
