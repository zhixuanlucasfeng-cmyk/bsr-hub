use std::sync::Arc;

use async_trait::async_trait;
use axum::{body::Body, http::Request};
use core_api::{
    AppState,
    auth::{AuthError, AuthUser, AuthVerifier},
    domain::quote::PricingSnapshot,
    ports::{
        order_repository::{CreateOrder, OrderRepository, ReserveError, ReservedOrder},
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

#[tokio::test]
async fn health_returns_ok_json() {
    let response = core_api::app()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], br#"{"status":"ok"}"#);
}

struct ReadinessOrders(bool);

#[async_trait]
impl OrderRepository for ReadinessOrders {
    async fn pricing(&self, _listing_id: uuid::Uuid) -> Result<PricingSnapshot, ReserveError> {
        Err(ReserveError::PricingNotFound)
    }

    async fn reserve(&self, _order: CreateOrder) -> Result<ReservedOrder, ReserveError> {
        Err(ReserveError::Unavailable)
    }

    async fn readiness(&self) -> Result<(), ReserveError> {
        self.0
            .then_some(())
            .ok_or_else(|| ReserveError::database("unavailable"))
    }
}

struct NoopPayments;

#[async_trait]
impl PaymentGateway for NoopPayments {
    async fn create_checkout(
        &self,
        _request: CheckoutRequest,
    ) -> Result<CheckoutSession, PaymentError> {
        Err(PaymentError::Unavailable)
    }
}

struct NoopAuth;

#[async_trait]
impl AuthVerifier for NoopAuth {
    async fn verify(&self, _bearer_token: &str) -> Result<AuthUser, AuthError> {
        Err(AuthError::Invalid)
    }
}

struct NoopProfiles;

#[async_trait]
impl ProfileRepository for NoopProfiles {
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

fn state(ready: bool) -> AppState {
    AppState {
        orders: Arc::new(ReadinessOrders(ready)),
        profiles: Arc::new(NoopProfiles),
        payments: Arc::new(NoopPayments),
        auth: Arc::new(NoopAuth),
        stripe_webhook_secret: Arc::from("whsec_test"),
    }
}

#[tokio::test]
async fn readiness_reports_database_state_without_leaking_details() {
    let ready = core_api::app_with_state(state(true))
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ready.status(), 200);
    let body = ready.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], br#"{"status":"ready"}"#);

    let unavailable = core_api::app_with_state(state(false))
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unavailable.status(), 503);
    let body = unavailable.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("DATABASE_UNAVAILABLE"));
    assert!(!body.contains("mongodb://"));
}
