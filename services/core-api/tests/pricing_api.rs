use std::sync::Arc;

use async_trait::async_trait;
use axum::{body::Body, http::Request};
use core_api::{
    AppState,
    auth::{AuthError, AuthUser, AuthVerifier},
    domain::{pricing::BillingUnit, quote::PricingSnapshot},
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

struct FakeOrders;

#[async_trait]
impl OrderRepository for FakeOrders {
    async fn pricing(&self, _listing_id: Uuid) -> Result<PricingSnapshot, ReserveError> {
        Ok(PricingSnapshot {
            unit_price_cents: 1_300,
            deposit_cents: 0,
            delivery_fee_cents: 0,
            service_fee_bps: 600,
            billing_unit: BillingUnit::ThirtyMinutes,
            allowed_fulfillment_methods: vec![core_api::domain::quote::FulfillmentMethod::OnSite],
        })
    }

    async fn reserve(&self, _order: CreateOrder) -> Result<ReservedOrder, ReserveError> {
        unreachable!()
    }

    async fn save_pricing_profile(
        &self,
        profile: SavePricingProfile,
    ) -> Result<StoredPricingProfile, ReserveError> {
        Ok(StoredPricingProfile {
            listing_id: profile.listing_id,
            recommended_unit_price_cents: profile.recommendation.recommended_unit_price_cents,
            seller_adjustment_cents: profile.seller_adjustment_cents,
            final_unit_price_cents: profile.final_unit_price_cents,
            minimum_allowed_cents: profile.recommendation.minimum_allowed_cents,
            maximum_allowed_cents: profile.recommendation.maximum_allowed_cents,
            billing_unit: profile.recommendation.billing_unit,
            ruleset_version: profile.recommendation.ruleset_version,
            reason_codes: profile.recommendation.reason_codes,
            allowed_fulfillment_methods: profile.allowed_fulfillment_methods,
        })
    }
}

struct FakePayments;

#[async_trait]
impl PaymentGateway for FakePayments {
    async fn create_checkout(
        &self,
        _request: CheckoutRequest,
    ) -> Result<CheckoutSession, PaymentError> {
        unreachable!()
    }
}

struct FakeAuth;

#[async_trait]
impl AuthVerifier for FakeAuth {
    async fn verify(&self, token: &str) -> Result<AuthUser, AuthError> {
        if token != "owner-token" {
            return Err(AuthError::Invalid);
        }
        Ok(AuthUser {
            user_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
        })
    }
}

struct FakeProfiles;

#[async_trait]
impl ProfileRepository for FakeProfiles {
    async fn bootstrap(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError> {
        Ok(test_profile(auth_user_id))
    }

    async fn get_by_auth_user_id(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError> {
        Ok(test_profile(auth_user_id))
    }

    async fn update(
        &self,
        auth_user_id: Uuid,
        _patch: ProfilePatch,
    ) -> Result<UserProfile, ProfileError> {
        Ok(test_profile(auth_user_id))
    }
}

fn test_profile(auth_user_id: Uuid) -> UserProfile {
    UserProfile {
        id: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
        auth_user_id,
        display_name: "Test member".to_owned(),
        avatar_url: None,
        role: ProfileRole::Buyer,
        trust_level: 1,
        created_at: OffsetDateTime::UNIX_EPOCH,
        updated_at: OffsetDateTime::UNIX_EPOCH,
    }
}

fn app() -> axum::Router {
    core_api::app_with_state(AppState {
        orders: Arc::new(FakeOrders),
        profiles: Arc::new(FakeProfiles),
        payments: Arc::new(FakePayments),
        auth: Arc::new(FakeAuth),
        stripe_webhook_secret: Arc::from("whsec_test"),
    })
}

async fn json(response: axum::response::Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn owner_can_save_explainable_workspace_pricing() {
    let request = Request::builder()
        .method("PUT")
        .uri("/v1/listings/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/pricing")
        .header("content-type", "application/json")
        .header("authorization", "Bearer owner-token")
        .body(Body::from(
            r#"{
                "attributes": {
                    "category": "workspace",
                    "squareFeet": 500,
                    "locationTier": "suburban",
                    "cleanliness": 3,
                    "equipmentScore": 0,
                    "amenityCount": 0,
                    "billingUnit": "thirty_minutes"
                },
                "sellerAdjustmentCents": 500,
                "allowedFulfillmentMethods": ["on_site"]
            }"#,
        ))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), 200);
    let body = json(response).await;
    assert_eq!(body["recommendedUnitPriceCents"], 1_300);
    assert_eq!(body["finalUnitPriceCents"], 1_800);
    assert_eq!(body["minimumAllowedCents"], 800);
    assert_eq!(body["maximumAllowedCents"], 1_800);
    assert_eq!(body["rulesetVersion"], "rules-v1");
}

#[tokio::test]
async fn pricing_requires_authentication() {
    let request = Request::builder()
        .method("PUT")
        .uri("/v1/listings/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/pricing")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{
                "attributes": {
                    "category": "workspace",
                    "squareFeet": 500,
                    "locationTier": "suburban",
                    "cleanliness": 3,
                    "equipmentScore": 0,
                    "amenityCount": 0,
                    "billingUnit": "thirty_minutes"
                },
                "sellerAdjustmentCents": 0,
                "allowedFulfillmentMethods": ["on_site"]
            }"#,
        ))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn seller_adjustment_cannot_exceed_five_dollars() {
    let request = Request::builder()
        .method("PUT")
        .uri("/v1/listings/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/pricing")
        .header("content-type", "application/json")
        .header("authorization", "Bearer owner-token")
        .body(Body::from(
            r#"{
                "attributes": {
                    "category": "ps5",
                    "model": "slim",
                    "ageMonths": 12,
                    "condition": "good",
                    "cleanliness": 3,
                    "fullyOperational": true,
                    "missingNonessentialFeatures": 0,
                    "controllerCount": 1,
                    "billingUnit": "day"
                },
                "sellerAdjustmentCents": 501,
                "allowedFulfillmentMethods": ["pickup"]
            }"#,
        ))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), 422);
    assert_eq!(
        json(response).await["error"]["code"],
        "SELLER_ADJUSTMENT_OUT_OF_RANGE"
    );
}

#[tokio::test]
async fn immovable_workspace_rejects_delivery() {
    let request = Request::builder()
        .method("PUT")
        .uri("/v1/listings/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/pricing")
        .header("content-type", "application/json")
        .header("authorization", "Bearer owner-token")
        .body(Body::from(
            r#"{
                "attributes": {
                    "category": "workspace",
                    "squareFeet": 500,
                    "locationTier": "suburban",
                    "cleanliness": 3,
                    "equipmentScore": 0,
                    "amenityCount": 0,
                    "billingUnit": "thirty_minutes"
                },
                "sellerAdjustmentCents": 0,
                "allowedFulfillmentMethods": ["delivery"]
            }"#,
        ))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), 422);
    assert_eq!(
        json(response).await["error"]["code"],
        "FULFILLMENT_NOT_ALLOWED"
    );
}
