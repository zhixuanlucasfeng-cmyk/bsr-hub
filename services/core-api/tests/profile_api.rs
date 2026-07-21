use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode, header::AUTHORIZATION},
};
use core_api::{
    AppState,
    auth::{AuthError, AuthUser, AuthVerifier},
    domain::{
        order_state::{OrderAction, OrderState},
        pricing::BillingUnit,
        quote::{FulfillmentMethod, PricingSnapshot},
    },
    ports::{
        order_repository::{
            CreateOrder, OrderRepository, ReserveError, ReservedOrder, SavePricingProfile,
            StoredPricingProfile,
        },
        payment_gateway::{CheckoutRequest, CheckoutSession, PaymentError, PaymentGateway},
        profile_repository::{
            ProfileError, ProfilePatch, ProfileRepository, ProfileRole, UserProfile,
        },
    },
};
use http_body_util::BodyExt;
use time::OffsetDateTime;
use tower::ServiceExt;
use uuid::Uuid;

const AUTH_USER_ID: Uuid = uuid::uuid!("11111111-1111-1111-1111-111111111111");

#[tokio::test]
async fn profile_bootstrap_requires_bearer_token() {
    let response = core_api::app_with_state(test_state())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/profile/bootstrap")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn profile_bootstrap_returns_profile_for_valid_token() {
    let response = core_api::app_with_state(test_state())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/profile/bootstrap")
                .header(AUTHORIZATION, "Bearer valid-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["authUserId"], AUTH_USER_ID.to_string());
    assert_eq!(json["displayName"], "Lucas demo");
    assert_eq!(json["role"], "buyer");
}

fn test_state() -> AppState {
    AppState {
        orders: Arc::new(FakeOrders),
        profiles: Arc::new(FakeProfiles),
        payments: Arc::new(FakePayments),
        auth: Arc::new(FakeAuth),
        stripe_webhook_secret: Arc::from("whsec_test"),
    }
}

struct FakeAuth;

#[async_trait]
impl AuthVerifier for FakeAuth {
    async fn verify(&self, bearer_token: &str) -> Result<AuthUser, AuthError> {
        if bearer_token == "valid-token" {
            Ok(AuthUser {
                user_id: AUTH_USER_ID,
            })
        } else {
            Err(AuthError::Invalid)
        }
    }
}

struct FakeProfiles;

#[async_trait]
impl ProfileRepository for FakeProfiles {
    async fn bootstrap(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError> {
        Ok(profile(auth_user_id))
    }

    async fn get_by_auth_user_id(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError> {
        Ok(profile(auth_user_id))
    }

    async fn update(
        &self,
        auth_user_id: Uuid,
        _patch: ProfilePatch,
    ) -> Result<UserProfile, ProfileError> {
        Ok(profile(auth_user_id))
    }
}

fn profile(auth_user_id: Uuid) -> UserProfile {
    let now = OffsetDateTime::UNIX_EPOCH;
    UserProfile {
        id: uuid::uuid!("22222222-2222-2222-2222-222222222222"),
        auth_user_id,
        display_name: "Lucas demo".to_owned(),
        avatar_url: None,
        role: ProfileRole::Buyer,
        trust_level: 1,
        created_at: now,
        updated_at: now,
    }
}

struct FakeOrders;

#[async_trait]
impl OrderRepository for FakeOrders {
    async fn pricing(&self, _listing_id: Uuid) -> Result<PricingSnapshot, ReserveError> {
        Ok(PricingSnapshot {
            unit_price_cents: 1000,
            deposit_cents: 5000,
            delivery_fee_cents: 0,
            service_fee_bps: 600,
            billing_unit: BillingUnit::ThirtyMinutes,
            allowed_fulfillment_methods: vec![FulfillmentMethod::Pickup],
        })
    }

    async fn reserve(&self, _order: CreateOrder) -> Result<ReservedOrder, ReserveError> {
        Ok(ReservedOrder {
            order_id: Uuid::new_v4(),
            expires_at: OffsetDateTime::UNIX_EPOCH,
        })
    }

    async fn save_pricing_profile(
        &self,
        _profile: SavePricingProfile,
    ) -> Result<StoredPricingProfile, ReserveError> {
        Err(ReserveError::InvalidPricing)
    }

    async fn transition(
        &self,
        _order_id: Uuid,
        _actor_id: Uuid,
        _action: OrderAction,
    ) -> Result<OrderState, ReserveError> {
        Err(ReserveError::InvalidTransition)
    }
}

struct FakePayments;

#[async_trait]
impl PaymentGateway for FakePayments {
    async fn create_checkout(
        &self,
        _request: CheckoutRequest,
    ) -> Result<CheckoutSession, PaymentError> {
        Ok(CheckoutSession {
            checkout_session_id: "cs_test".to_owned(),
            checkout_url: "https://checkout.test".to_owned(),
        })
    }
}
