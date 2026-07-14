mod health;
mod orders;
mod quotes;
mod stripe_webhook;

use axum::Router;

use crate::AppState;

pub fn health_routes() -> Router {
    Router::new().route("/health", axum::routing::get(health::get))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(health::get))
        .route("/v1/quotes", axum::routing::post(quotes::create))
        .route("/v1/orders", axum::routing::post(orders::create))
        .route(
            "/v1/orders/{id}/transitions",
            axum::routing::post(orders::transition),
        )
        .route(
            "/v1/stripe/webhook",
            axum::routing::post(stripe_webhook::receive),
        )
}
