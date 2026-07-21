use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn demo_listings_include_frontend_image_contract() {
    let response = core_api::app()
        .oneshot(
            Request::builder()
                .uri("/v1/demo/listings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let listings: Vec<Value> = serde_json::from_slice(&body).unwrap();

    assert!(!listings.is_empty());
    for listing in listings {
        let image_src = listing["imageSrc"].as_str().unwrap_or_default();
        let image_alt = listing["imageAlt"].as_str().unwrap_or_default();
        assert!(
            image_src.starts_with("/images/listings/") && image_src.ends_with(".jpg"),
            "listing {} must include a supported imageSrc, got {image_src:?}",
            listing["id"]
        );
        assert!(
            !image_alt.trim().is_empty(),
            "listing {} must include non-empty imageAlt",
            listing["id"]
        );
    }
}
