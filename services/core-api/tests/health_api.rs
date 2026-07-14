use axum::{body::Body, http::Request};
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
