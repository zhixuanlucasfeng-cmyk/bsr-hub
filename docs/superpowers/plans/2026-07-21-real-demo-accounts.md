# Real Demo Accounts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real public demo accounts for BSR Hub with Supabase Auth, MongoDB-backed user profiles, protected frontend routes, and clear classroom-demo fallbacks.

**Architecture:** The browser uses Supabase Auth for email/password sessions and sends the Supabase access token to the existing Rust API. The Rust API verifies the token, stores BSR Hub profile data in MongoDB, and exposes profile endpoints that the Next.js app uses for signed-in state and protected pages.

**Tech Stack:** Next.js 16, React 19, TypeScript, Supabase JS client, Rust 2024, Axum 0.8, MongoDB 3.3 Rust driver, existing GitHub Pages static export, Render-compatible Rust backend configuration.

## Global Constraints

- Passwords are handled only by Supabase Auth and are never stored in MongoDB.
- MongoDB credentials and backend secrets are never exposed to browser code.
- The public demo must still work for guests using fictional demo content.
- Protected transactions must not silently fake success when the Rust API is unavailable.
- The first version does not collect real ID documents, exact home addresses, bank information, or real payment information.
- Keep the existing marketplace, runner, and static demo routes working.
- Use existing codebase patterns instead of replacing the app architecture.

---

## File Structure

- Modify `services/core-api/src/lib.rs`: add profile repository to `AppState`.
- Modify `services/core-api/src/http/mod.rs`: register `/v1/me` and `/v1/profile/bootstrap`.
- Create `services/core-api/src/http/profile.rs`: authenticated profile handlers.
- Create `services/core-api/src/ports/profile_repository.rs`: backend profile interface.
- Modify `services/core-api/src/ports/mod.rs`: export the profile repository port.
- Create `services/core-api/src/adapters/mongo/profiles.rs`: MongoDB implementation of the profile repository.
- Modify `services/core-api/src/adapters/mongo/mod.rs`: expose profile repository and include profile collection.
- Modify `services/core-api/src/adapters/mongo/models.rs`: add `UserProfileDocument`.
- Modify `services/core-api/src/adapters/mongo/bootstrap.rs`: create `user_profiles`, unique index, and schema validator.
- Modify `services/core-api/src/main.rs`: construct and inject the Mongo profile repository.
- Add backend tests inside `services/core-api/src/http/profile.rs` and Mongo repository tests inside `services/core-api/src/adapters/mongo/profiles.rs`.
- Modify `apps/web/package.json`: add `@supabase/supabase-js`.
- Create `apps/web/src/lib/auth-config.ts`: read and validate public auth configuration.
- Create `apps/web/src/lib/supabase-browser.ts`: browser Supabase client factory.
- Create `apps/web/src/lib/api-client.ts`: authenticated Rust API fetch helper.
- Create `apps/web/src/lib/auth-state.ts`: pure helpers for signed-in, signed-out, and API-unavailable states.
- Add tests in `apps/web/src/lib/auth-state.test.ts` and `apps/web/src/lib/api-client.test.ts`.
- Create `apps/web/src/components/AuthProvider.tsx`: React auth context.
- Create `apps/web/src/components/AuthModal.tsx`: sign in and sign up modal.
- Modify `apps/web/src/components/GlobalNav.tsx`: account menu, auth modal trigger, protected create route behavior.
- Modify `apps/web/src/app/layout.tsx`: wrap app with `AuthProvider`.
- Modify `apps/web/src/app/create/page.tsx`: protect listing creation UI.
- Modify `apps/web/src/app/orders/page.tsx`: protect orders UI.
- Modify `apps/web/src/components/BookingCard.tsx`: use authenticated API token for order creation.
- Modify `.env.example`, `.env.local.example`, `.env.mongodb.example`, and `render.yaml`: document required Supabase and Mongo variables.
- Modify `README.md`: explain static guest demo vs real account demo.

---

### Task 1: Backend Profile Port And Auth Helper

**Files:**
- Create: `services/core-api/src/ports/profile_repository.rs`
- Modify: `services/core-api/src/ports/mod.rs`
- Create: `services/core-api/src/http/auth_extract.rs`
- Modify: `services/core-api/src/http/mod.rs`

**Interfaces:**
- Consumes: `crate::auth::{AuthUser, AuthVerifier}` and `axum::http::HeaderMap`.
- Produces: `ProfileRepository`, `UserProfile`, `ProfileError`, `ProfilePatch`, and `require_auth`.

- [ ] **Step 1: Create the profile port**

Create `services/core-api/src/ports/profile_repository.rs` with:

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: Uuid,
    pub auth_user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: ProfileRole,
    pub trust_level: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileRole {
    Buyer,
    Seller,
    Runner,
    Admin,
}

impl ProfileRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buyer => "buyer",
            Self::Seller => "seller",
            Self::Runner => "runner",
            Self::Admin => "admin",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProfilePatch {
    pub display_name: Option<String>,
    pub avatar_url: Option<Option<String>>,
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("profile was not found")]
    NotFound,
    #[error("profile data is invalid")]
    Invalid,
    #[error("profile persistence failed")]
    Unavailable,
}

#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn bootstrap(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError>;
    async fn get_by_auth_user_id(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError>;
    async fn update(
        &self,
        auth_user_id: Uuid,
        patch: ProfilePatch,
    ) -> Result<UserProfile, ProfileError>;
}
```

- [ ] **Step 2: Export the port**

Modify `services/core-api/src/ports/mod.rs`:

```rust
pub mod order_repository;
pub mod payment_gateway;
pub mod profile_repository;
```

- [ ] **Step 3: Extract shared bearer-token auth**

Create `services/core-api/src/http/auth_extract.rs`:

```rust
use axum::http::{HeaderMap, header::AUTHORIZATION};

use crate::{AppState, auth::AuthUser, error::ApiError};

pub async fn require_auth(state: &AppState, headers: &HeaderMap) -> Result<AuthUser, ApiError> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
        .ok_or_else(ApiError::auth_required)?;

    state
        .auth
        .verify(token)
        .await
        .map_err(|_| ApiError::auth_required())
}
```

- [ ] **Step 4: Register the helper module**

Modify `services/core-api/src/http/mod.rs`:

```rust
mod auth_extract;
mod health;
mod orders;
mod pricing;
mod profile;
mod quotes;
mod stripe_webhook;
```

- [ ] **Step 5: Verify compile errors are limited to missing profile state**

Run: `cargo check -p core-api`
Expected: FAIL because `AppState` does not yet include `profiles` and `profile` route code is not implemented.

---

### Task 2: MongoDB User Profile Repository

**Files:**
- Modify: `services/core-api/src/lib.rs`
- Modify: `services/core-api/src/adapters/mongo/models.rs`
- Modify: `services/core-api/src/adapters/mongo/bootstrap.rs`
- Create: `services/core-api/src/adapters/mongo/profiles.rs`
- Modify: `services/core-api/src/adapters/mongo/mod.rs`
- Modify: `services/core-api/src/main.rs`

**Interfaces:**
- Consumes: `ProfileRepository`, `ProfilePatch`, `UserProfile`, `ProfileRole`.
- Produces: `MongoProfileRepository`.

- [ ] **Step 1: Add profile state**

Modify `services/core-api/src/lib.rs`:

```rust
use ports::{
    order_repository::OrderRepository, payment_gateway::PaymentGateway,
    profile_repository::ProfileRepository,
};

#[derive(Clone)]
pub struct AppState {
    pub orders: Arc<dyn OrderRepository>,
    pub profiles: Arc<dyn ProfileRepository>,
    pub payments: Arc<dyn PaymentGateway>,
    pub auth: Arc<dyn AuthVerifier>,
    pub stripe_webhook_secret: Arc<str>,
}
```

- [ ] **Step 2: Add Mongo document model**

Append to `services/core-api/src/adapters/mongo/models.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub auth_user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub trust_level: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub schema_version: i32,
}
```

- [ ] **Step 3: Add profile collection bootstrap**

Modify `COLLECTIONS` in `services/core-api/src/adapters/mongo/bootstrap.rs` to include `"user_profiles"`.

Add this index in `bootstrap` after existing indexes:

```rust
database
    .collection::<Document>("user_profiles")
    .create_index(index(
        doc! { "auth_user_id": 1 },
        "user_profile_auth_user_unique",
        true,
    ))
    .await?;
```

Add this validator entry in `apply_validators`:

```rust
(
    "user_profiles",
    json_schema(
        &[
            "_id",
            "auth_user_id",
            "display_name",
            "role",
            "trust_level",
            "created_at",
            "updated_at",
            "schema_version",
        ],
        doc! {
            "_id": { "bsonType": "string" },
            "auth_user_id": { "bsonType": "string" },
            "display_name": { "bsonType": "string", "minLength": 1, "maxLength": 80 },
            "avatar_url": { "bsonType": ["string", "null"] },
            "role": { "enum": ["buyer", "seller", "runner", "admin"] },
            "trust_level": { "bsonType": "int", "minimum": 0 },
            "created_at": { "bsonType": "date" },
            "updated_at": { "bsonType": "date" },
            "schema_version": { "bsonType": "int", "minimum": 1 },
        },
    ),
),
```

- [ ] **Step 4: Implement repository**

Create `services/core-api/src/adapters/mongo/profiles.rs` with:

```rust
use async_trait::async_trait;
use mongodb::{
    Collection,
    bson::{doc, oid::ObjectId},
    options::ReturnDocument,
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    adapters::mongo::{MongoAdapterError, models::UserProfileDocument},
    ports::profile_repository::{ProfileError, ProfilePatch, ProfileRepository, ProfileRole, UserProfile},
};

#[derive(Clone)]
pub struct MongoProfileRepository {
    profiles: Collection<UserProfileDocument>,
}

impl MongoProfileRepository {
    pub fn new(profiles: Collection<UserProfileDocument>) -> Self {
        Self { profiles }
    }

    fn default_display_name(auth_user_id: Uuid) -> String {
        format!("BSR member {}", &auth_user_id.to_string()[..8])
    }
}

#[async_trait]
impl ProfileRepository for MongoProfileRepository {
    async fn bootstrap(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError> {
        let now = OffsetDateTime::now_utc();
        let update = doc! {
            "$setOnInsert": {
                "_id": ObjectId::new().to_hex(),
                "auth_user_id": auth_user_id.to_string(),
                "display_name": Self::default_display_name(auth_user_id),
                "avatar_url": null,
                "role": "buyer",
                "trust_level": 1_i32,
                "created_at": super::bson_datetime(now),
                "schema_version": 1_i32,
            },
            "$set": {
                "updated_at": super::bson_datetime(now),
            }
        };
        let document = self
            .profiles
            .find_one_and_update(doc! { "auth_user_id": auth_user_id.to_string() }, update)
            .upsert(true)
            .return_document(ReturnDocument::After)
            .await
            .map_err(|_| ProfileError::Unavailable)?
            .ok_or(ProfileError::Unavailable)?;
        document.try_into()
    }

    async fn get_by_auth_user_id(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError> {
        let document = self
            .profiles
            .find_one(doc! { "auth_user_id": auth_user_id.to_string() })
            .await
            .map_err(|_| ProfileError::Unavailable)?
            .ok_or(ProfileError::NotFound)?;
        document.try_into()
    }

    async fn update(&self, auth_user_id: Uuid, patch: ProfilePatch) -> Result<UserProfile, ProfileError> {
        let mut set = doc! { "updated_at": super::bson_datetime(OffsetDateTime::now_utc()) };
        if let Some(display_name) = patch.display_name {
            let trimmed = display_name.trim();
            if trimmed.is_empty() || trimmed.len() > 80 {
                return Err(ProfileError::Invalid);
            }
            set.insert("display_name", trimmed);
        }
        if let Some(avatar_url) = patch.avatar_url {
            set.insert("avatar_url", avatar_url);
        }
        let document = self
            .profiles
            .find_one_and_update(doc! { "auth_user_id": auth_user_id.to_string() }, doc! { "$set": set })
            .return_document(ReturnDocument::After)
            .await
            .map_err(|_| ProfileError::Unavailable)?
            .ok_or(ProfileError::NotFound)?;
        document.try_into()
    }
}
```

- [ ] **Step 5: Add conversions**

In the same file, add:

```rust
impl TryFrom<UserProfileDocument> for UserProfile {
    type Error = ProfileError;

    fn try_from(value: UserProfileDocument) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&value.id).unwrap_or_else(|_| Uuid::new_v4());
        let auth_user_id = Uuid::parse_str(&value.auth_user_id).map_err(|_| ProfileError::Invalid)?;
        let role = match value.role.as_str() {
            "buyer" => ProfileRole::Buyer,
            "seller" => ProfileRole::Seller,
            "runner" => ProfileRole::Runner,
            "admin" => ProfileRole::Admin,
            _ => return Err(ProfileError::Invalid),
        };
        Ok(UserProfile {
            id,
            auth_user_id,
            display_name: value.display_name,
            avatar_url: value.avatar_url,
            role,
            trust_level: value.trust_level,
            created_at: super::time_from_bson(value.created_at),
            updated_at: super::time_from_bson(value.updated_at),
        })
    }
}
```

If `time_from_bson` is private or missing, expose a small conversion helper from `services/core-api/src/adapters/mongo/mod.rs` using the same `time` conversion pattern already used by order documents.

- [ ] **Step 6: Wire repository into Mongo module**

Modify `services/core-api/src/adapters/mongo/mod.rs`:

```rust
pub mod bootstrap;
pub mod models;
pub mod profiles;
pub mod seed;
pub mod slots;
```

Add to `MongoCollections`:

```rust
pub user_profiles: Collection<UserProfileDocument>,
```

Add to `MongoCollections::new`:

```rust
user_profiles: database.collection("user_profiles"),
```

- [ ] **Step 7: Wire repository into app startup**

Modify `services/core-api/src/main.rs` so `MongoProfileRepository::new(order_repository.collections().user_profiles.clone())` or an equivalent database collection constructor is passed into `AppState`.

If `MongoCollections` is private to the order repository, add a method to `MongoOrderRepository`:

```rust
pub fn profile_repository(&self) -> profiles::MongoProfileRepository {
    profiles::MongoProfileRepository::new(self.collections.user_profiles.clone())
}
```

- [ ] **Step 8: Run backend compile**

Run: `cargo check -p core-api`
Expected: PASS.

---

### Task 3: Backend Profile HTTP Routes

**Files:**
- Create: `services/core-api/src/http/profile.rs`
- Modify: `services/core-api/src/http/mod.rs`
- Test: `services/core-api/src/http/profile.rs`

**Interfaces:**
- Consumes: `require_auth`, `AppState.profiles`.
- Produces: `POST /v1/profile/bootstrap`, `GET /v1/me`, `PATCH /v1/me`.

- [ ] **Step 1: Write profile handlers**

Create `services/core-api/src/http/profile.rs`:

```rust
use axum::{Json, extract::State, http::{HeaderMap, StatusCode}};
use serde::Deserialize;

use crate::{
    AppState,
    error::ApiError,
    ports::profile_repository::{ProfileError, ProfilePatch, UserProfile},
};

use super::auth_extract::require_auth;

pub async fn bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<UserProfile>), ApiError> {
    let user = require_auth(&state, &headers).await?;
    let profile = state
        .profiles
        .bootstrap(user.user_id)
        .await
        .map_err(map_profile_error)?;
    Ok((StatusCode::OK, Json(profile)))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserProfile>, ApiError> {
    let user = require_auth(&state, &headers).await?;
    let profile = state
        .profiles
        .get_by_auth_user_id(user.user_id)
        .await
        .map_err(map_profile_error)?;
    Ok(Json(profile))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<Option<String>>,
}

pub async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, ApiError> {
    let user = require_auth(&state, &headers).await?;
    let profile = state
        .profiles
        .update(
            user.user_id,
            ProfilePatch {
                display_name: request.display_name,
                avatar_url: request.avatar_url,
            },
        )
        .await
        .map_err(map_profile_error)?;
    Ok(Json(profile))
}

fn map_profile_error(error: ProfileError) -> ApiError {
    match error {
        ProfileError::NotFound => ApiError::new(StatusCode::NOT_FOUND, "PROFILE_NOT_FOUND", "Profile was not found"),
        ProfileError::Invalid => ApiError::new(StatusCode::BAD_REQUEST, "INVALID_PROFILE", "Profile details are invalid"),
        ProfileError::Unavailable => ApiError::new(StatusCode::SERVICE_UNAVAILABLE, "PROFILE_UNAVAILABLE", "Profile service is temporarily unavailable"),
    }
}
```

- [ ] **Step 2: Register routes**

Modify `services/core-api/src/http/mod.rs`:

```rust
.route("/v1/profile/bootstrap", axum::routing::post(profile::bootstrap))
.route("/v1/me", axum::routing::get(profile::me).patch(profile::update))
```

- [ ] **Step 3: Refactor order auth**

Modify `services/core-api/src/http/orders.rs` so `create` and `transition` call:

```rust
let user = super::auth_extract::require_auth(&state, &headers).await?;
```

Remove duplicate bearer-token parsing from `orders.rs`.

- [ ] **Step 4: Add route tests with fake auth and fake profile repository**

Inside `services/core-api/src/http/profile.rs`, add tests that create `AppState` with:

```rust
#[derive(Clone)]
struct AllowAuth;

#[async_trait::async_trait]
impl crate::auth::AuthVerifier for AllowAuth {
    async fn verify(&self, _bearer_token: &str) -> Result<crate::auth::AuthUser, crate::auth::AuthError> {
        Ok(crate::auth::AuthUser { user_id: uuid::uuid!("11111111-1111-1111-1111-111111111111") })
    }
}
```

and a fake profile repo that records bootstrap/update calls.

Test names:

```rust
#[tokio::test]
async fn bootstrap_requires_bearer_token() {}

#[tokio::test]
async fn bootstrap_returns_profile_for_valid_token() {}

#[tokio::test]
async fn update_rejects_blank_display_name() {}
```

- [ ] **Step 5: Run backend tests**

Run: `cargo test -p core-api -q`
Expected: PASS.

---

### Task 4: Frontend Auth Configuration And API Client

**Files:**
- Modify: `apps/web/package.json`
- Create: `apps/web/src/lib/auth-config.ts`
- Create: `apps/web/src/lib/supabase-browser.ts`
- Create: `apps/web/src/lib/api-client.ts`
- Create: `apps/web/src/lib/auth-state.ts`
- Create: `apps/web/src/lib/auth-state.test.ts`
- Create: `apps/web/src/lib/api-client.test.ts`

**Interfaces:**
- Produces: `getAuthConfig`, `isAuthConfigured`, `createBrowserSupabaseClient`, `apiFetch`, `describeAuthGate`.

- [ ] **Step 1: Add Supabase client dependency**

Run: `npm install @supabase/supabase-js --workspace apps/web`
Expected: `apps/web/package.json` and the lockfile include `@supabase/supabase-js`.

- [ ] **Step 2: Add auth config helper**

Create `apps/web/src/lib/auth-config.ts`:

```ts
export type AuthConfig = {
  supabaseUrl: string;
  supabaseAnonKey: string;
  apiBaseUrl: string;
  staticDemo: boolean;
};

export function getAuthConfig(env: NodeJS.ProcessEnv = process.env): AuthConfig {
  return {
    supabaseUrl: env.NEXT_PUBLIC_SUPABASE_URL ?? "",
    supabaseAnonKey: env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? "",
    apiBaseUrl: (env.NEXT_PUBLIC_API_BASE_URL ?? "").replace(/\/$/, ""),
    staticDemo: env.NEXT_PUBLIC_STATIC_DEMO === "true",
  };
}

export function isAuthConfigured(config: AuthConfig): boolean {
  return Boolean(config.supabaseUrl && config.supabaseAnonKey && config.apiBaseUrl && !config.staticDemo);
}
```

- [ ] **Step 3: Add browser Supabase client**

Create `apps/web/src/lib/supabase-browser.ts`:

```ts
"use client";

import { createClient, type SupabaseClient } from "@supabase/supabase-js";
import { getAuthConfig, isAuthConfigured } from "./auth-config";

let client: SupabaseClient | null = null;

export function createBrowserSupabaseClient(): SupabaseClient | null {
  const config = getAuthConfig();
  if (!isAuthConfigured(config)) return null;
  client ??= createClient(config.supabaseUrl, config.supabaseAnonKey);
  return client;
}
```

- [ ] **Step 4: Add API client helper**

Create `apps/web/src/lib/api-client.ts`:

```ts
import { getAuthConfig } from "./auth-config";

export class ApiClientError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly code: string,
  ) {
    super(message);
  }
}

export async function apiFetch<T>(
  path: string,
  accessToken: string,
  init: RequestInit = {},
): Promise<T> {
  const config = getAuthConfig();
  if (!config.apiBaseUrl) {
    throw new ApiClientError("The online backend is not connected for this demo.", 0, "API_NOT_CONFIGURED");
  }
  const response = await fetch(`${config.apiBaseUrl}${path}`, {
    ...init,
    headers: {
      "content-type": "application/json",
      authorization: `Bearer ${accessToken}`,
      ...(init.headers ?? {}),
    },
  });
  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    throw new ApiClientError(
      body.message ?? "The BSR Hub backend could not complete this action.",
      response.status,
      body.code ?? "API_ERROR",
    );
  }
  return response.json() as Promise<T>;
}
```

- [ ] **Step 5: Add pure auth state helper and tests**

Create `apps/web/src/lib/auth-state.ts`:

```ts
export type AuthGateState = "signed-in" | "signed-out" | "api-unavailable";

export function describeAuthGate(input: {
  authConfigured: boolean;
  signedIn: boolean;
}): AuthGateState {
  if (!input.authConfigured) return "api-unavailable";
  return input.signedIn ? "signed-in" : "signed-out";
}
```

Create `apps/web/src/lib/auth-state.test.ts`:

```ts
import assert from "node:assert/strict";
import { test } from "node:test";
import { describeAuthGate } from "./auth-state.ts";

test("auth gate reports unavailable when online auth is not configured", () => {
  assert.equal(describeAuthGate({ authConfigured: false, signedIn: false }), "api-unavailable");
});

test("auth gate reports signed out when auth exists but no session exists", () => {
  assert.equal(describeAuthGate({ authConfigured: true, signedIn: false }), "signed-out");
});

test("auth gate reports signed in when auth exists and a session exists", () => {
  assert.equal(describeAuthGate({ authConfigured: true, signedIn: true }), "signed-in");
});
```

- [ ] **Step 6: Run frontend tests**

Run: `npm --workspace apps/web test`
Expected: PASS.

---

### Task 5: Frontend Auth Provider And Modal

**Files:**
- Create: `apps/web/src/components/AuthProvider.tsx`
- Create: `apps/web/src/components/AuthModal.tsx`
- Modify: `apps/web/src/app/layout.tsx`

**Interfaces:**
- Consumes: `createBrowserSupabaseClient`, `apiFetch`, `isAuthConfigured`.
- Produces: `useAuth`, `AuthProvider`, `AuthModal`.

- [ ] **Step 1: Create auth provider**

Create `apps/web/src/components/AuthProvider.tsx` with client-side state:

```tsx
"use client";

import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import type { Session, SupabaseClient, User } from "@supabase/supabase-js";
import { apiFetch } from "../lib/api-client";
import { getAuthConfig, isAuthConfigured } from "../lib/auth-config";
import { createBrowserSupabaseClient } from "../lib/supabase-browser";

export type BsrProfile = {
  id: string;
  authUserId: string;
  displayName: string;
  avatarUrl: string | null;
  role: "buyer" | "seller" | "runner" | "admin";
  trustLevel: number;
};

type AuthContextValue = {
  configured: boolean;
  loading: boolean;
  user: User | null;
  profile: BsrProfile | null;
  accessToken: string | null;
  signIn: (email: string, password: string) => Promise<void>;
  signUp: (email: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;
  bootstrapProfile: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const config = getAuthConfig();
  const configured = isAuthConfigured(config);
  const supabase = useMemo<SupabaseClient | null>(() => createBrowserSupabaseClient(), []);
  const [session, setSession] = useState<Session | null>(null);
  const [profile, setProfile] = useState<BsrProfile | null>(null);
  const [loading, setLoading] = useState(configured);

  async function bootstrapProfile(nextSession = session) {
    if (!nextSession?.access_token) return;
    const nextProfile = await apiFetch<BsrProfile>("/v1/profile/bootstrap", nextSession.access_token, {
      method: "POST",
    });
    setProfile(nextProfile);
  }

  useEffect(() => {
    if (!supabase || !configured) {
      setLoading(false);
      return;
    }
    let mounted = true;
    supabase.auth.getSession().then(async ({ data }) => {
      if (!mounted) return;
      setSession(data.session);
      if (data.session) await bootstrapProfile(data.session);
      setLoading(false);
    });
    const { data } = supabase.auth.onAuthStateChange((_event, nextSession) => {
      setSession(nextSession);
      if (nextSession) void bootstrapProfile(nextSession);
      else setProfile(null);
    });
    return () => {
      mounted = false;
      data.subscription.unsubscribe();
    };
  }, [supabase, configured]);

  const value: AuthContextValue = {
    configured,
    loading,
    user: session?.user ?? null,
    profile,
    accessToken: session?.access_token ?? null,
    async signIn(email, password) {
      if (!supabase) throw new Error("Online accounts are not configured for this demo.");
      const { error } = await supabase.auth.signInWithPassword({ email, password });
      if (error) throw error;
    },
    async signUp(email, password) {
      if (!supabase) throw new Error("Online accounts are not configured for this demo.");
      const { error } = await supabase.auth.signUp({ email, password });
      if (error) throw error;
    },
    async signOut() {
      if (supabase) await supabase.auth.signOut();
      setSession(null);
      setProfile(null);
    },
    bootstrapProfile,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const value = useContext(AuthContext);
  if (!value) throw new Error("useAuth must be used inside AuthProvider");
  return value;
}
```

- [ ] **Step 2: Create auth modal**

Create `apps/web/src/components/AuthModal.tsx` as a client component with:

```tsx
"use client";

import { useState } from "react";
import { useAuth } from "./AuthProvider";

export function AuthModal({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { configured, signIn, signUp } = useAuth();
  const [mode, setMode] = useState<"signin" | "signup">("signin");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  if (!open) return null;

  async function submit() {
    setError("");
    setBusy(true);
    try {
      if (mode === "signin") await signIn(email, password);
      else await signUp(email, password);
      onClose();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Account action failed.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/35 px-4 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-2xl bg-white p-6 shadow-2xl">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-2xl font-bold text-zinc-950">{mode === "signin" ? "Sign in" : "Create account"}</h2>
            <p className="mt-1 text-sm text-zinc-600">Use BSR Hub as a real account demo. Do not enter private payment or identity data.</p>
          </div>
          <button className="text-2xl leading-none text-zinc-500" onClick={onClose} aria-label="Close">x</button>
        </div>
        {!configured ? (
          <p className="mt-5 rounded-xl bg-violet-50 p-4 text-sm text-violet-800">
            Online accounts are not connected in this environment. The guest marketplace demo is still available.
          </p>
        ) : (
          <div className="mt-5 space-y-4">
            <input className="w-full rounded-xl border border-zinc-200 px-4 py-3" type="email" value={email} onChange={(event) => setEmail(event.target.value)} placeholder="Email" />
            <input className="w-full rounded-xl border border-zinc-200 px-4 py-3" type="password" value={password} onChange={(event) => setPassword(event.target.value)} placeholder="Password" />
            {error ? <p className="text-sm text-red-600">{error}</p> : null}
            <button className="w-full rounded-xl bg-violet-600 px-4 py-3 font-semibold text-white transition hover:bg-violet-700 disabled:opacity-60" disabled={busy} onClick={submit}>
              {busy ? "Working..." : mode === "signin" ? "Sign in" : "Create account"}
            </button>
            <button className="w-full text-sm font-medium text-violet-700" onClick={() => setMode(mode === "signin" ? "signup" : "signin")}>
              {mode === "signin" ? "Need an account? Create one" : "Already have an account? Sign in"}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Wrap app layout**

Modify `apps/web/src/app/layout.tsx`:

```tsx
import { AuthProvider } from "../components/AuthProvider";

// inside body:
<AuthProvider>{children}</AuthProvider>
```

- [ ] **Step 4: Run frontend typecheck**

Run: `npm --workspace apps/web typecheck`
Expected: PASS.

---

### Task 6: Protected Navigation, Create Page, Orders Page, And Booking Flow

**Files:**
- Modify: `apps/web/src/components/GlobalNav.tsx`
- Modify: `apps/web/src/app/create/page.tsx`
- Modify: `apps/web/src/app/orders/page.tsx`
- Modify: `apps/web/src/components/BookingCard.tsx`

**Interfaces:**
- Consumes: `useAuth`, `AuthModal`, `apiFetch`.
- Produces: visible signed-in account state and protected action handling.

- [ ] **Step 1: Update navbar auth entry**

Modify `GlobalNav.tsx` to:

- show `+ List something` as a button that opens `AuthModal` when signed out
- navigate to `/create/` when signed in
- show profile initials/avatar when signed in
- show `Sign in` when signed out
- include `Sign out` in the account dropdown

The click handler shape:

```tsx
function handleListSomething(event: React.MouseEvent<HTMLAnchorElement>) {
  if (!user) {
    event.preventDefault();
    setAuthOpen(true);
  }
}
```

- [ ] **Step 2: Protect create page**

Modify `apps/web/src/app/create/page.tsx` as a client page or split a client component from the server page. The protected state should render:

```tsx
if (!configured) return <DemoBackendNotice />;
if (loading) return <LoadingPanel label="Checking account..." />;
if (!user) return <SignedOutCreatePrompt onSignIn={() => setAuthOpen(true)} />;
return <CreateListingDemoForm profile={profile} />;
```

The form can remain a demo form for the first version, but its copy must say that real listing persistence requires the online Rust API to be connected.

- [ ] **Step 3: Protect orders page**

Modify `apps/web/src/app/orders/page.tsx` with the same gate:

```tsx
if (!configured) return <DemoBackendNotice />;
if (loading) return <LoadingPanel label="Loading your orders..." />;
if (!user) return <SignedOutOrdersPrompt onSignIn={() => setAuthOpen(true)} />;
return <OrdersDemoPanel profile={profile} />;
```

- [ ] **Step 4: Update booking API call**

Modify `BookingCard.tsx` so order creation uses:

```tsx
if (!accessToken) {
  setError("Please sign in before booking this item.");
  setAuthOpen(true);
  return;
}

const response = await apiFetch<CreateOrderResponse>("/v1/orders", accessToken, {
  method: "POST",
  body: JSON.stringify(orderRequest),
});
```

When `ApiClientError.code === "API_NOT_CONFIGURED"`, show: `Online checkout is not connected in this classroom demo. You can still explore pricing and product details.`

- [ ] **Step 5: Run frontend verification**

Run:

```bash
npm --workspace apps/web test
npm --workspace apps/web typecheck
npm --workspace apps/web build
```

Expected: all PASS.

---

### Task 7: Environment, Deployment, And Documentation

**Files:**
- Modify: `.env.example`
- Modify: `.env.local.example`
- Modify: `.env.mongodb.example`
- Modify: `render.yaml`
- Modify: `README.md`

**Interfaces:**
- Produces: documented setup for local demo and public real-account demo.

- [ ] **Step 1: Add frontend env examples**

Ensure env examples include:

```bash
NEXT_PUBLIC_SUPABASE_URL=https://your-project.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=your-supabase-anon-key
NEXT_PUBLIC_API_BASE_URL=https://your-render-service.onrender.com
NEXT_PUBLIC_STATIC_DEMO=false
```

- [ ] **Step 2: Add backend env examples**

Ensure backend examples include:

```bash
MONGODB_URI=mongodb+srv://user:password@cluster.mongodb.net/?retryWrites=true&w=majority
MONGODB_DATABASE=bsr_hub
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-supabase-anon-key
ALLOWED_ORIGIN=https://zhixuanlucasfeng-cmyk.github.io
```

- [ ] **Step 3: Fix Render config**

Modify `render.yaml` so the Rust backend includes:

```yaml
- key: MONGODB_URI
  sync: false
- key: MONGODB_DATABASE
  value: bsr_hub
- key: SUPABASE_URL
  sync: false
- key: SUPABASE_ANON_KEY
  sync: false
- key: ALLOWED_ORIGIN
  value: https://zhixuanlucasfeng-cmyk.github.io
```

- [ ] **Step 4: Update README**

Add a "Real account demo" section:

```md
### Real account demo

The GitHub Pages site can run as a guest demo or as a real-account demo.
Guest mode uses fictional listings and browser-only state.
Real-account mode uses Supabase Auth, the Rust API, and MongoDB Atlas.
Do not enter real payment, identity, or private address data in the class demo.
```

- [ ] **Step 5: Run final verification**

Run:

```bash
cargo test -p core-api -q
npm --workspace apps/web test
npm --workspace apps/web typecheck
npm --workspace apps/web build
git status --short
```

Expected: tests and builds pass; `git status --short` shows only intended files plus pre-existing untracked folders.

---

### Task 8: Commit And Handoff

**Files:**
- Commit all files intentionally changed by this plan.

**Interfaces:**
- Produces: one implementation commit and a short handoff summary.

- [ ] **Step 1: Review changed files**

Run:

```bash
git diff --stat
git diff --name-only
```

Expected: changed files match this plan.

- [ ] **Step 2: Commit implementation**

Run:

```bash
git add services/core-api apps/web .env.example .env.local.example .env.mongodb.example render.yaml README.md package-lock.json package.json
git commit -m "feat: add real demo account foundation"
```

If one of those files does not exist, omit only that missing file from `git add`.

- [ ] **Step 3: Handoff**

Report:

- what works locally
- what needs real Supabase/MongoDB/Render credentials before the public URL has live accounts
- exact test commands that passed
- the commit hash

---

## Self-Review Notes

Spec coverage:

- Supabase Auth: Tasks 4 and 5.
- MongoDB user profiles: Tasks 2 and 3.
- Rust protected API: Tasks 1, 2, and 3.
- Guest demo fallback: Tasks 4, 5, and 6.
- Deployment variables: Task 7.
- Security constraints: Global Constraints and Task 7.
- Testing: Tasks 3, 4, 5, 6, and 7.

The plan intentionally does not build real payments, identity verification, insurance, or admin moderation because the approved spec marks those as outside the first version.
