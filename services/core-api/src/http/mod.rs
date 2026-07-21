mod auth_extract;
mod health;
mod orders;
mod pricing;
mod profile;
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
        .route("/ready", axum::routing::get(health::ready))
        .route("/v1/quotes", axum::routing::post(quotes::create))
        .route(
            "/v1/profile/bootstrap",
            axum::routing::post(profile::bootstrap),
        )
        .route(
            "/v1/me",
            axum::routing::get(profile::me).patch(profile::update),
        )
        .route("/v1/orders", axum::routing::post(orders::create))
        .route(
            "/v1/listings/{id}/pricing",
            axum::routing::put(pricing::save),
        )
        .route(
            "/v1/orders/{id}/transitions",
            axum::routing::post(orders::transition),
        )
        .route(
            "/v1/stripe/webhook",
            axum::routing::post(stripe_webhook::receive),
        )
}
