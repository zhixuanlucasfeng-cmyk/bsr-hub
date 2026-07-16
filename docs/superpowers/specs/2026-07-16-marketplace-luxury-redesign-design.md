# BSR Hub Marketplace Luxury Redesign

## Goal

Rebuild BSR Hub's public marketplace and listing experience in a refined Airbnb/Fat Llama-inspired shared-economy style while preserving the existing rental, workspace, second-hand, pricing, ordering, category, listing, and support flows.

## Product Boundaries

- Preserve the three core businesses: product rentals, creative-space bookings, and second-hand sales.
- Preserve the brand slogan `Use more. Own less.` and the approved BSR logo family.
- Preserve the Rust API contracts, static-demo adapter, pricing rules, order state machine, protected-payment messaging, category filtering, persona switching, listing creation demo, and GitHub Pages static export.
- This phase does not create production authentication. The sign-in gate stores only a local demo session and clearly labels demo identities.
- Recommendation logic remains explainable and rule-based in the public demo.
- Immovable workspaces never offer delivery.

## Design System

- Tailwind CSS becomes the primary styling system for new and rebuilt marketplace UI.
- Brand purple: `#7C3AED`; accent lime: `#CCFF00`; background: `#FAFAFA`.
- Neutral tokens use four levels for primary text, secondary text, borders, and muted surfaces.
- Containers use 16-pixel rounded corners and paired ambient/contact shadows.
- Typography uses 700 for display headings, 500 for subheadings, and 400 for body copy with 1.5–1.6 line height.
- Icons use a consistent two-pixel rounded linear style implemented as reusable inline SVG components.
- Interactive elements use 200ms ease transitions; cards rise two pixels, media scales subtly, and primary gradients shift on hover.
- Reduced-motion preferences disable nonessential transforms and reveal animation.

## Global Navigation and Authentication

The 76-pixel global navigation remains fixed. Before scrolling it uses a warm near-white surface; after scrolling it switches to a translucent blurred surface with a soft bottom shadow. It contains the Hub logo, Explore/Categories/My orders navigation, a gradient `+ List something` action, and the current persona avatar.

A first-entry sign-in modal asks the visitor to choose a fictional demo identity. Browsing remains available through an explicit guest/demo path, but reserving, buying, listing, or opening orders triggers the gate when no demo identity has been selected. The selected identity is stored in `sessionStorage`; no real credentials or private identity data are collected.

## Home Page

### Hero

The hero retains the slogan, supporting copy, primary and secondary actions, and the protected-payment, verified-community, and local-delivery trust claims. A faint dotted radial texture replaces the flat background. Three real-photo cards represent console rental, tool rental, and studio booking with price pills, soft shadows, staggered positioning, and restrained hover motion.

### Category Rail

A horizontally scrollable rail immediately follows the hero. It exposes Gaming consoles, Camera gear, Power tools, Office equipment, Creative spaces, and Second-hand tech. Each entry combines a linear icon and label; the active category uses a purple fill and filters the marketplace.

### Featured Listings

`Popular rentals` displays a four-column desktop grid, two-column tablet grid, and one-column narrow-mobile grid. Cards contain real media, type badge, title, price/unit, seller avatar/name, verification, and distance. Hover enlarges the image and reveals a reservation action. Reveal-on-scroll is staggered with `IntersectionObserver` and respects reduced motion.

### Business Showcase

Three full-width image cards highlight Rent products, Book creative spaces, and Buy second-hand. Each uses a real scene image, dark gradient overlay, left-aligned copy, and an Explore action that applies the matching marketplace filter.

### Footer

The dark footer contains the BSR introduction, marketplace links, terms, privacy, help, contact, social placeholders, UN goals, and the truthful demo/payment disclaimer.

## Support Assistant

The current large lower-right launcher becomes a single circular icon fixed at the lower-left edge. Clicking it opens the existing support conversation panel; closing it returns to the icon. The assistant retains rent, listing, workspace, delivery, safety, and worker-handoff actions. The expanded panel remains accessible by keyboard and never overlaps the mobile booking bar.

## Listing Detail Experience

Each listing opens a route-shaped full detail experience using `/listings/[id]` semantics where static export permits; a query-backed client route is acceptable only if required to preserve GitHub Pages and booking continuity. The preferred implementation uses static listing paths generated from the demo catalog.

On desktop the detail layout is 70/30: media and information on the left, sticky transaction card on the right. Mobile becomes one column with a fixed bottom booking/buy action.

The media gallery supports previous/next controls, swipe gestures, fade transition, and thumbnail selection when multiple images exist. A one-image listing shows no fake thumbnails.

The information column includes title, seller avatar/name, verified badge, positive rating, expandable three-line description, structured specifications, rental/use instructions, and protection policy. The transaction card adapts by listing type:

- Rental: price per day or 30-minute unit, date/rental units, deposit, fulfillment, and Reserve.
- Second-hand: one-time price, quantity, fulfillment, and Buy.
- Workspace: 30-minute units, date/time selection, on-site rules, and Book.

The card continues to call the existing quote and order adapters. Fulfillment options come from each listing, so immovable spaces cannot display delivery. Below the detail, four to six rule-ranked related listings use category, listing type, location, and price proximity.

## Component Boundaries

- `GlobalNav`: scroll surface, navigation, persona, protected-action gate.
- `LoginModal`: fictional demo identity/session selection.
- `LinearIcon`: shared two-pixel icon system.
- `HeroSection`: slogan, trust, actions, visual scene cards.
- `CategoryRail`: category selection and horizontal navigation.
- `FeaturedListings` and `MarketplaceListingCard`: listing presentation and actions.
- `BusinessShowcase`: three business filters.
- `ListingDetailView`, `ListingGallery`, `SellerProfile`, `BookingCard`, `RelatedListings`: detail journey.
- `SupportAssistant`: icon launcher and existing scripted/handoff panel.
- `SiteFooter`: legal/help/impact links and disclaimers.

## State and Data Flow

The existing `Home` component remains the source of marketplace view, category, persona, quote, and order state until those units are moved into focused hooks. Protected actions call one `requireSession(action)` boundary. Listing cards pass listing IDs into the detail surface. Booking uses the same static-demo or Rust request adapter already selected by environment variables. No pricing formula is duplicated in presentational components.

## Responsive and Accessibility Requirements

- Desktop, tablet, and mobile widths receive dedicated layout checks.
- Navigation, login modal, support panel, gallery, cards, and booking controls are keyboard reachable and visibly focused.
- Dialogs use correct labels, modal semantics, escape/close behavior, and focus restoration.
- Images have meaningful alternative text and fixed aspect ratios to prevent layout shift.
- Sticky elements do not overlap each other; mobile content includes safe bottom padding.
- No viewport has horizontal document overflow.

## Verification

- Unit tests cover sign-in session rules, related-listing ranking, fulfillment adaptation, and existing marketplace behavior.
- TypeScript, Hub tests, Rust checks where relevant, Runner checks, and both Next.js builds pass.
- `npm run pages:check` produces the combined static artifact.
- Browser verification covers home and detail at desktop, tablet, and mobile widths, plus login gate, customer-service expansion, category filtering, listing navigation, quote creation, and no-overflow checks.
- Public GitHub Pages URLs and logo/media assets return successfully after deployment.
