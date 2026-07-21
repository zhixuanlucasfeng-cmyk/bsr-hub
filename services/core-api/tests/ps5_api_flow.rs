use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::{body::Body, http::Request};
use core_api::{
    AppState,
    auth::{AuthError, AuthUser, AuthVerifier},
    domain::{
        pricing::BillingUnit,
        quote::{FulfillmentMethod, PricingSnapshot, QuoteBreakdown},
    },
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

#[derive(Default)]
struct FakeOrders {
    reserved: Mutex<bool>,
}

#[async_trait]
impl OrderRepository for FakeOrders {
    async fn pricing(&self, _listing_id: Uuid) -> Result<PricingSnapshot, ReserveError> {
        Ok(PricingSnapshot {
            unit_price_cents: 2_500,
            deposit_cents: 10_000,
            delivery_fee_cents: 1_500,
            service_fee_bps: 600,
            billing_unit: BillingUnit::ThirtyMinutes,
            allowed_fulfillment_methods: vec![
                FulfillmentMethod::Pickup,
                FulfillmentMethod::Delivery,
            ],
        })
    }

    async fn reserve(&self, order: CreateOrder) -> Result<ReservedOrder, ReserveError> {
        assert!(order.start_at.is_some());
        assert!(order.end_at.is_some());
        let mut reserved = self.reserved.lock().unwrap();
        if *reserved {
            return Err(ReserveError::Unavailable);
        }
        *reserved = true;
        Ok(ReservedOrder {
            order_id: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap(),
            expires_at: time::OffsetDateTime::UNIX_EPOCH,
        })
    }
}

struct FakePayments;

#[async_trait]
impl PaymentGateway for FakePayments {
    async fn create_checkout(
        &self,
        request: CheckoutRequest,
    ) -> Result<CheckoutSession, PaymentError> {
        Ok(CheckoutSession {
            checkout_session_id: format!("cs_test_{}", request.order_id),
            checkout_url: "https://checkout.stripe.test/session".into(),
        })
    }
}

struct FakeAuth;

#[async_trait]
impl AuthVerifier for FakeAuth {
    async fn verify(&self, token: &str) -> Result<AuthUser, AuthError> {
        if token != "good-token" {
            return Err(AuthError::Invalid);
        }
        Ok(AuthUser {
            user_id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
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
        orders: Arc::new(FakeOrders::default()),
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
async fn ps5_quote_and_reservation_flow_is_authoritative() {
    let listing = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
    let quote_request = Request::builder()
        .method("POST")
        .uri("/v1/quotes")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"listingId":"{listing}","startAt":"2026-07-20T10:00:00Z","endAt":"2026-07-20T10:31:00Z","fulfillment":"delivery"}}"#
        )))
        .unwrap();
    let quote_response = app().oneshot(quote_request).await.unwrap();
    assert_eq!(quote_response.status(), 200);
    let quote: QuoteBreakdown = serde_json::from_value(json(quote_response).await).unwrap();
    assert_eq!(quote.total_cents, 16_800);
    assert_eq!(quote.service_fee_cents, 300);
    assert_eq!(quote.currency, "USD");
    assert_eq!(quote.billable_units, 2);

    let create_order = || {
        Request::builder()
            .method("POST")
            .uri("/v1/orders")
            .header("content-type", "application/json")
            .header("authorization", "Bearer good-token")
            .body(Body::from(format!(
                r#"{{"listingId":"{listing}","startAt":"2026-07-20T10:00:00Z","endAt":"2026-07-20T10:31:00Z","fulfillment":"delivery"}}"#
            )))
            .unwrap()
    };
    let application = app();
    let created = application.clone().oneshot(create_order()).await.unwrap();
    assert_eq!(created.status(), 201);
    let created_body = json(created).await;
    assert_eq!(created_body["totalCents"], 16_800);
    assert_eq!(
        created_body["checkoutUrl"],
        "https://checkout.stripe.test/session"
    );

    let conflict = application.oneshot(create_order()).await.unwrap();
    assert_eq!(conflict.status(), 409);
    assert_eq!(json(conflict).await["error"]["code"], "LISTING_UNAVAILABLE");
}

#[tokio::test]
async fn order_creation_requires_authentication() {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/orders")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"listingId":"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa","startAt":"2026-07-20T10:00:00Z","endAt":"2026-07-20T10:20:00Z","fulfillment":"pickup"}"#,
        ))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), 401);
    assert_eq!(json(response).await["error"]["code"], "AUTH_REQUIRED");
}

#[tokio::test]
async fn order_transition_route_requires_authentication() {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/orders/cccccccc-cccc-cccc-cccc-cccccccccccc/transitions")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"action":"confirm"}"#))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), 401);
    assert_eq!(json(response).await["error"]["code"], "AUTH_REQUIRED");
}

#[tokio::test]
async fn partial_rental_window_is_rejected() {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/orders")
        .header("content-type", "application/json")
        .header("authorization", "Bearer good-token")
        .body(Body::from(
            r#"{"listingId":"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa","startAt":"2026-07-20T10:00:00Z","fulfillment":"pickup"}"#,
        ))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), 422);
    assert_eq!(
        json(response).await["error"]["code"],
        "INVALID_RENTAL_WINDOW"
    );
}

#[tokio::test]
async fn client_controlled_units_are_rejected() {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/quotes")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"listingId":"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa","startAt":"2026-07-20T10:00:00Z","endAt":"2026-07-20T10:20:00Z","fulfillment":"pickup","units":1}"#,
        ))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), 422);
}
