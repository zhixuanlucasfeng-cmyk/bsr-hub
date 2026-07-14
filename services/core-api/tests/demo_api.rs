use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn demo_catalog_and_quote_are_available_without_credentials() {
    let app = core_api::demo_app();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/demo/listings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let listings: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(listings.as_array().unwrap().len() >= 12);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/demo/quote")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"listingId":"ps5-slim","units":2,"fulfillment":"delivery"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["baseCents"], 2400);
    assert_eq!(body["deliveryFeeCents"], 800);
    assert_eq!(body["depositCents"], 10000);
    assert_eq!(body["totalCents"], 13344);
}

#[tokio::test]
async fn demo_orders_follow_the_rust_state_machine() {
    let app = core_api::demo_app();
    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/demo/orders")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"listingId":"ps5-slim","units":2,"fulfillment":"pickup"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let value: serde_json::Value =
        serde_json::from_slice(&created.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let id = value["id"].as_str().unwrap();

    let invalid = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/demo/orders/{id}/actions"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"action":"activate"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::CONFLICT);
}
