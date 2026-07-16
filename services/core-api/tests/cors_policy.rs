#[test]
fn production_cors_allows_pricing_updates() {
    let main_source = include_str!("../src/main.rs");

    assert!(
        main_source.contains("Method::GET, Method::POST, Method::PUT"),
        "production CORS must allow PUT requests used by /v1/listings/:id/pricing"
    );
}
