use std::sync::Arc;

use axum::http::{HeaderValue, Method, header};
use core_api::{
    AppState,
    adapters::{postgres_orders::PostgresOrderRepository, stripe::StripePaymentGateway},
    auth::SupabaseAuthVerifier,
    config::Config,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let config = Config::from_env().expect("invalid core API configuration");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("connect to PostgreSQL");
    let state = AppState {
        orders: Arc::new(PostgresOrderRepository::with_rules(
            pool,
            config.service_fee_bps,
            config.reservation_minutes,
        )),
        payments: Arc::new(StripePaymentGateway::new(
            config.stripe_secret_key,
            config.web_success_url,
            config.web_cancel_url,
        )),
        auth: Arc::new(SupabaseAuthVerifier::new(
            &config.supabase_url,
            config.supabase_anon_key,
        )),
        stripe_webhook_secret: Arc::from(config.stripe_webhook_secret),
    };
    let allowed_origin: HeaderValue = config
        .allowed_origin
        .parse()
        .expect("ALLOWED_ORIGIN must be a valid origin");
    let cors = CorsLayer::new()
        .allow_origin(allowed_origin)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);
    let application = core_api::app_with_state(state).layer(cors);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port))
        .await
        .expect("bind core API");
    axum::serve(listener, application)
        .await
        .expect("serve core API");
}
