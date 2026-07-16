use core_api::{
    adapters::mongo::{MongoOrderRepository, seed::seed_fictional_catalog},
    config::Config,
};

#[tokio::main]
async fn main() {
    let config = Config::from_env().expect("invalid core API configuration");
    let repository = MongoOrderRepository::connect(
        &config.mongodb_uri,
        &config.mongodb_database,
        config.service_fee_bps,
        config.reservation_minutes,
    )
    .await
    .expect("connect to MongoDB and bootstrap collections");
    let report = seed_fictional_catalog(repository.database())
        .await
        .expect("seed fictional catalog");
    println!(
        "MongoDB bootstrap complete: {} fictional listings",
        report.listings
    );
}
