# BSR Hub Categories, Real Photos, and Hybrid Assistant Design

**Date:** 2026-07-16  
**Status:** Approved design, awaiting written-spec review  
**Scope:** `apps/web` public BSR Hub demo only

## Objective

Make the BSR Hub marketplace easier to understand and more credible by adding visual category discovery, replacing illustrative emoji artwork with real product and workspace photography, and providing an accessible hybrid shopping assistant that can answer common questions and prepare a handoff to a human worker.

## User Experience

### Category discovery

Add a `Browse by category` section between the marketplace promise section and the listing grid. It contains seven selectable category cards:

1. Gaming and consoles
2. Computers and electronics
3. Cameras and creator equipment
4. Tools and maker equipment
5. Studios and workspaces
6. Small production spaces
7. Second-hand goods

Each card uses a real photograph, a readable label, and a short example such as `PS5 and controllers` or `Photo and podcast studios`. Selecting a category filters the existing listing grid and scrolls the visitor to the results. An `All categories` control clears the filter.

### Real listing photography

Replace the emoji/gradient listing artwork with real photographs for every demo listing. Images are stored locally under `apps/web/public/images/` so the public demo is not dependent on third-party hotlinks. Every image has concise alternative text based on the listing title.

The listing card continues to show listing type, category, condition, location, price, and billing unit. The detail modal displays a larger version of the same photograph. Images use consistent aspect ratios with `object-fit: cover`, responsive sizes, and a neutral fallback when an asset cannot load.

Photo provenance is recorded in `docs/PHOTO-SOURCES.md`. Only photographs with reuse terms suitable for this public educational demo may be added.

### Hybrid shopping assistant

Add a floating `BSR Assistant` launcher in the bottom-right corner. On the visitor's first marketplace view in a browser session, the panel opens automatically after a short delay and displays:

> Hi! Welcome to BSR Hub. Can I help you find something to rent, sell an item, book a workspace, or arrange delivery?

The automatic greeting happens at most once per browser session. Closing the panel keeps it closed until the visitor deliberately reopens it.

The first version offers six quick actions:

- Find something to rent
- List my item
- Book a workspace
- Arrange delivery
- Payment and deposit help
- Talk to a worker

The first five actions return short, accurate answers and may trigger an existing navigation or filter action. Automated messages are labeled `BSR Assistant`.

`Talk to a worker` switches to a handoff view where the visitor can write a message. The static site prepares a handoff through a configurable support destination. The code must not claim that a worker is online or that a message was delivered unless a real support destination is configured. Without configuration, it offers a transparent `Copy message` fallback and states that live worker delivery is not yet connected.

No payment details, identity documents, precise addresses, or other sensitive information should be requested in the chat.

## Component Design

### Data model

Extend each static listing with:

- `imageSrc`: local public-image path
- `imageAlt`: descriptive alternative text

Add a small category configuration array containing category ID, label, example text, image path, and the listing categories/types it matches.

### Components

Keep `Home` responsible for marketplace state while extracting the new UI into focused components:

- `CategoryBrowser`: renders category cards and reports the selected category.
- `ListingImage`: renders optimized listing photography and a fallback state.
- `ShopAssistant`: owns the visible chat panel, scripted messages, quick actions, and worker-handoff state.

The assistant receives callbacks for marketplace navigation instead of directly changing unrelated page state.

### State and persistence

Marketplace category selection remains React state. Search, listing type, and category filters are combined when calculating visible listings.

Assistant conversation state remains local to the page. A `sessionStorage` flag records that the proactive greeting has already been shown. No chat transcript is uploaded or persisted by the static demo.

The human handoff target is read from a public build-time configuration value. Because the site is static, this value must contain only a public support destination and never a secret.

## Responsive and Accessible Behavior

- Category cards form a horizontal snap row on small screens and a responsive grid on larger screens.
- The assistant becomes a nearly full-width bottom sheet on small screens and a fixed panel on desktop.
- The launcher and close button have accessible names.
- The panel uses dialog semantics, a visible heading, logical focus order, keyboard-operable controls, and an `aria-live` region for new assistant messages.
- Photography has useful alternative text; decorative overlays are hidden from assistive technology.
- Text overlays maintain sufficient contrast over photographs.
- The chat never blocks marketplace navigation after it is dismissed.

## Failure and Edge Cases

- Missing image: render a neutral branded fallback without breaking card dimensions.
- Empty category: show the existing empty-results treatment and allow clearing the category.
- Storage unavailable: show the greeting normally without failing the page.
- Missing support destination: do not display `sent`, `online`, or a fake human response; provide the copy fallback.
- Repeated entry during one session: do not reopen the assistant automatically.

## Verification

Automated checks will cover:

- category-to-listing matching;
- combined search, type, and category filtering;
- assistant quick-action responses;
- proactive greeting session behavior where practical;
- worker-handoff behavior with and without a configured destination;
- existing quote, order, and static-demo tests remaining green;
- TypeScript type checking and production build.

Browser verification will cover desktop and mobile layouts, real-image rendering, category filtering, opening and dismissing the assistant, keyboard access, and the truthful unconfigured-worker fallback.

## Out of Scope

- A staffed real-time support dashboard
- WebSocket chat infrastructure
- AI-generated free-form customer-support answers
- Collecting private customer data in the static demo
- Changes to the Rust pricing or order-state APIs

