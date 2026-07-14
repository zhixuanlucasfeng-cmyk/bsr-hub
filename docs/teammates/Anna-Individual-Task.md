# Anna — Individual Task Instructions

## Your Role

You are responsible for **team communication and the main public frontend experience** of BSR Hub. Your job is to make the product easy to understand and use on both phones and computers.

You may use Claude to help write code, but you must review every change, run the relevant checks, and coordinate file ownership with Lucas before editing shared files.

## Main Coding Responsibilities

### 1. Shared design system

Build reusable components for:

- buttons;
- text inputs, text areas, selects, and validation messages;
- listing cards;
- category filters;
- price and status badges;
- loading, empty, success, and error states;
- responsive navigation and page layout.

Use consistent colors, spacing, typography, and interaction patterns. Components must work with keyboard navigation and mobile screens.

### 2. Authentication and profiles

Build the interfaces for:

- sign up;
- sign in;
- sign out;
- forgotten password link or placeholder flow;
- profile view and edit;
- avatar, display name, city, and state.

Lucas or Yichen will provide the Supabase authentication setup and data contracts. Do not create a second authentication system.

### 3. Public marketplace pages

Build:

- homepage;
- category browsing;
- search-results page;
- listing-detail page;
- responsive listing cards.

The homepage should clearly present three choices:

1. Rent products.
2. Book workspaces.
3. Buy second-hand products.

Search and category pages must support listing type, category, price, and approximate location. Public pages show only city and state, never a private street address.

### 4. Listing creation and editing

Build forms that allow any authenticated user to:

- choose rental, sale, or workspace;
- add title, category, description, condition, and images;
- set rental or sale price;
- set a deposit for rentals;
- choose pricing unit such as hour or day;
- provide city and state;
- enable pickup, delivery, owner-location use, or on-site use;
- set a delivery fee when delivery is enabled;
- save, edit, deactivate, and preview a listing.

The form must change based on listing type. For example, a second-hand sale does not ask for rental dates or a deposit.

## Communication Responsibilities

- Post one short team update at the end of each workday.
- Record decisions that affect page names, wording, or shared components.
- Tell Lucas immediately when another person's API or database work blocks the frontend.
- Confirm that Nasia understands the final user journey and presentation wording.
- Keep product language simple, consistent, and in English.

Use this daily update format:

```markdown
## Anna — Day X Update

- Completed:
- Screenshots or test results:
- Waiting for:
- Blocked by:
- Next task:
```

## Ten-Day Schedule

### Day 1

- Confirm page map, shared API types, and files you own.
- Create wireframes for mobile and desktop.
- Agree on colors, typography, spacing, and component names.

### Day 2

- Build navigation, layout, buttons, inputs, and status components.
- Build sign-up, sign-in, and profile interfaces.

### Day 3

- Build homepage, listing cards, category browsing, search results, and listing detail.
- Build create-listing and edit-listing forms.

### Day 4

- Connect pages to the agreed Supabase queries and types.
- Add loading, empty, validation, and failure states.

### Day 5

- Finish image upload and listing preview.
- Support Lucas and Nasia with shared components needed for checkout.

### Day 6 — Integration Day

- Do not add new pages.
- Fix only issues preventing the complete PS5 rental journey.
- Verify search and listing pages using two different accounts.

### Day 7

- Improve responsive layout and shared order-status components.
- Review all English labels and instructions.

### Day 8

- Complete keyboard, mobile, contrast, form-label, and error-message checks.
- Fix high-priority UI issues and help verify deployment.

### Day 9 — Feature Freeze

- Add no new features.
- Fix critical bugs, capture screenshots, and help prepare the final report.

### Day 10

- Run the smoke-test checklist.
- Support Nasia during rehearsal and the live demonstration.

## Required Deliverables

- Responsive shared design system.
- Authentication and profile interfaces.
- Homepage, search, category, and listing-detail pages.
- Create-listing and edit-listing forms.
- Loading, empty, validation, and error states.
- Mobile and accessibility checklist.
- Daily communication updates.
- Screenshots of the major pages for the presentation backup.

## Working With Claude

Before asking Claude to edit code, provide:

- the exact task;
- the files Anna owns;
- the approved API or database contract;
- acceptance criteria;
- commands that must pass.

Example request:

```text
Work only inside apps/web and do not change database migrations or the Rust API.
Implement the responsive BSR Hub listing card using the existing shared types.
Include loading and unavailable states, keyboard-accessible controls, and tests.
Run the relevant checks and summarize every changed file.
```

Do not ask Claude to redesign the architecture, modify Rust, change migrations, or replace shared contracts without Lucas and Yichen's approval.

## File and Security Rules

- Work in the official Git repository and use one short-lived branch per task.
- Do not allow two agents to edit the same file at the same time.
- Do not commit `.env`, passwords, API keys, or private addresses.
- Do not send `node_modules`, `.next`, `dist`, or other generated folders to Lucian.
- Give Lucian only the source files, screenshots, and update notes needed for the daily backup.
- Open a pull request with a screenshot or test result and obtain one human review.

## Definition of Done

Anna's work is complete when a new user can sign up, understand the three marketplace choices, find a PS5, view its details, and publish a new listing on both mobile and desktop without seeing a private address or unclear error message.
