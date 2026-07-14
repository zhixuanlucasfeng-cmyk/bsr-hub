pub mod adapters;
pub mod auth;
pub mod config;
pub mod demo;
pub mod domain;
pub mod error;
pub mod http;
pub mod ports;

use std::sync::Arc;

use auth::AuthVerifier;
use axum::Router;
use ports::{order_repository::OrderRepository, payment_gateway::PaymentGateway};

#[derive(Clone)]
pub struct AppState {
    pub orders: Arc<dyn OrderRepository>,
    pub payments: Arc<dyn PaymentGateway>,
    pub auth: Arc<dyn AuthVerifier>,
    pub stripe_webhook_secret: Arc<str>,
}

pub fn app() -> Router {
    http::health_routes().merge(demo::router())
}

pub fn demo_app() -> Router {
    app()
}

pub fn app_with_state(state: AppState) -> Router {
    http::routes().with_state(state).merge(demo::router())
}
