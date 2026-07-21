# BSR Hub Real Demo Accounts Design

Date: 2026-07-21
Owner: Lucas
Project: BSR Hub

## Goal

Add a real account and database foundation to BSR Hub so the public demo can support actual sign up, login, user profiles, and protected marketplace actions without pretending that browser-only demo data is production data.

The result should be clear enough for a Babson summer school demo: users can create an account, sign in, see their session persist, and use account-protected demo flows. The site must still warn visitors not to enter real payment, identity, or private address data because this is a class prototype.

## Current State

The project already has two modes:

- Static GitHub Pages demo: useful for public viewing, but account state is fictional and stored in the browser.
- Rust API demo: a local backend exists and already includes MongoDB support for listings, orders, booking slots, pricing profiles, and event records.

The missing part is a real public account layer. The public website does not currently let users create real accounts that persist online across devices.

## Recommended Architecture

Use Supabase Auth for identity, MongoDB Atlas for application data, and the existing Rust API as the trusted backend.

Supabase Auth will handle email/password login, password hashing, sessions, password reset, and JWT issuing. BSR Hub will not store user passwords in MongoDB.

MongoDB Atlas will store BSR Hub application records:

- user profiles
- listings
- orders and bookings
- booking slots
- pricing profiles
- audit events

The Rust API will verify Supabase JWTs before allowing protected actions. The frontend will never directly write protected marketplace records into MongoDB.

## Public Demo Modes

The project should keep two public behaviors:

- Guest demo mode: users can explore the marketplace with fictional listings and obvious demo labels.
- Real account demo mode: users can sign up or log in, then access profile, create listing, and order pages backed by the Rust API.

Guest demo mode is important because classmates and teachers should be able to view the product quickly without making an account.

Real account demo mode is important because the team can prove that BSR Hub has a real database and backend architecture.

## Account Flow

1. A visitor clicks the avatar or a protected action such as `+ List something`.
2. The frontend opens a sign in / create account modal.
3. Supabase Auth creates or verifies the account.
4. The frontend stores the Supabase session using the Supabase client.
5. The frontend sends the access token to the Rust API in the `Authorization: Bearer <token>` header.
6. The Rust API verifies the JWT against Supabase.
7. The Rust API creates or updates a MongoDB `user_profiles` document for that Supabase user.
8. Protected pages use that profile and user id for ownership checks.

## User Profile Data

MongoDB should store a `user_profiles` collection with these fields:

- `id`: internal profile id
- `auth_user_id`: Supabase user id, unique
- `display_name`
- `avatar_url`
- `role`: `buyer`, `seller`, `runner`, or `admin`
- `trust_level`: starter score for future trust and safety features
- `created_at`
- `updated_at`

The first version should collect only display name and optional avatar. It should not collect real ID images, exact home address, bank information, or private documents.

## Protected Marketplace Actions

The first account-backed version should protect these flows:

- account creation and login
- account menu and logout
- profile bootstrap from Supabase into MongoDB
- create listing page access
- my orders page access
- authenticated order or booking creation when the Rust API is available

Seller ownership checks must happen in Rust, not only in the frontend. A user should not be able to edit or cancel another seller's listing by changing browser data.

## Frontend Changes

The web app should add:

- a small Supabase client wrapper
- an auth provider or session helper
- sign in / sign up modal
- account menu state in the navbar
- protected route handling for create listing and orders pages
- clear loading, signed-out, signed-in, and API-unavailable states
- a demo warning banner on account and checkout flows

When the Rust API is not configured, the website should not silently fake a successful protected transaction. It should show a polished classroom-demo fallback explaining that the online backend is not connected in this environment.

## Rust API Changes

The Rust API should add or confirm:

- `POST /v1/profile/bootstrap`: verify token, create profile if missing, return profile
- `GET /v1/me`: return current authenticated user's profile
- `PATCH /v1/me`: update safe profile fields
- shared auth extraction middleware for protected routes
- MongoDB profile repository with indexes and schema validation

Existing reservation, booking, and order endpoints should continue using the current MongoDB repository patterns.

## Deployment Configuration

The public demo needs these environment variables:

Frontend:

- `NEXT_PUBLIC_SUPABASE_URL`
- `NEXT_PUBLIC_SUPABASE_ANON_KEY`
- `NEXT_PUBLIC_API_BASE_URL`
- `NEXT_PUBLIC_STATIC_DEMO`

Backend:

- `MONGODB_URI`
- `MONGODB_DATABASE`
- `SUPABASE_URL`
- `SUPABASE_ANON_KEY`
- `SUPABASE_JWT_SECRET` or verified JWKS configuration
- `ALLOWED_ORIGIN`
- existing Stripe test-mode variables for payment demo flows

Render or another backend host must include MongoDB variables. GitHub Pages can host the static frontend, but it cannot safely host backend secrets.

## Security Rules

The first version must follow these rules:

- Never store passwords in MongoDB.
- Never expose MongoDB credentials to the browser.
- Verify every protected Rust endpoint with a Supabase token.
- Restrict CORS to the BSR Hub public URLs and localhost development URLs.
- Keep payment flows in test/demo mode unless a real legal business setup exists.
- Do not collect sensitive identity documents in the class demo.
- Add visible Terms of Service and Privacy Policy links.

## Error Handling

The site should handle:

- invalid login
- expired session
- Rust API unavailable
- MongoDB unavailable
- duplicate profile creation
- user trying to open a protected page while signed out
- attempted self-rental or unauthorized seller action

Errors should be written in plain language and give the user a next action, such as sign in again or try the guest demo.

## Testing

Testing should cover:

- frontend auth modal renders correctly
- signed-out users are prompted before protected actions
- signed-in users can bootstrap a profile
- Rust profile endpoints reject missing or invalid tokens
- Rust profile endpoints accept valid Supabase test tokens or test-mode verified identities
- MongoDB profile repository creates one profile per Supabase user id
- public static demo still builds
- existing Rust tests still pass

## Success Criteria

The feature is successful when:

- a user can create an account through the website
- the user's account persists after refresh
- MongoDB stores the user's BSR Hub profile
- protected pages behave differently for signed-in and signed-out users
- the website still works as a guest demo
- the team can honestly say: "The public demo has a real account architecture using Supabase Auth, MongoDB, and a Rust API."

## Not In Scope For This First Version

The first version will not include:

- real money movement
- real identity document verification
- production insurance
- automatic police reports
- AI fraud investigation
- full runner payroll
- full admin moderation dashboard

Those are future trust and operations layers. The first priority is proving a clean account and database foundation.
