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
    },
};
use http_body_util::BodyExt;
use tower::ServiceExt;

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

fn state(ready: bool) -> AppState {
    AppState {
        orders: Arc::new(ReadinessOrders(ready)),
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
