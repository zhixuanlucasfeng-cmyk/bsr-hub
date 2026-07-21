use std::sync::Arc;

use axum::http::{HeaderValue, Method, header};
use core_api::{
    AppState,
    adapters::{mongo::MongoOrderRepository, stripe::StripePaymentGateway},
    auth::SupabaseAuthVerifier,
    config::Config,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    if std::env::var("BSR_DEMO_MODE").as_deref() == Ok("true") {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8080);
        let application = core_api::demo_app().layer(CorsLayer::permissive());
        let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
            .await
            .expect("bind demo API");
        axum::serve(listener, application)
            .await
            .expect("serve demo API");
        return;
    }
    let config = Config::from_env().expect("invalid core API configuration");
    let orders = MongoOrderRepository::connect(
        &config.mongodb_uri,
        &config.mongodb_database,
        config.service_fee_bps,
        config.reservation_minutes,
    )
    .await
    .expect("connect to MongoDB and bootstrap collections");
    let profiles = orders.profile_repository();
    let state = AppState {
        orders: Arc::new(orders),
        profiles: Arc::new(profiles),
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
