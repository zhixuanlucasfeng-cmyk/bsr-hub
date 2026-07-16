use core_api::{
    adapters::mongo::{MongoOrderRepository, seed::FICTIONAL_RENTAL_ID},
    domain::{
        pricing::{BillingUnit, Condition, PricingCategoryInput, Ps5Input, Ps5Model, recommend},
        quote::FulfillmentMethod,
    },
    ports::order_repository::{OrderRepository, SavePricingProfile},
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires MONGODB_TEST_URI replica set"]
async fn pricing_profile_round_trips_through_mongodb() {
    let uri = std::env::var("MONGODB_TEST_URI").expect("MONGODB_TEST_URI is required");
    let database_name = format!("bsr_test_{}", Uuid::new_v4().simple());
    let repository = MongoOrderRepository::connect(&uri, &database_name, 600, 30)
        .await
        .expect("connect test repository");
    core_api::adapters::mongo::seed::seed_fictional_catalog(repository.database())
        .await
        .expect("seed fictional listing");

    let listing_id = Uuid::parse_str(FICTIONAL_RENTAL_ID).unwrap();
    let owner_id = Uuid::parse_str(core_api::adapters::mongo::seed::FICTIONAL_OWNER_ID).unwrap();
    let attributes = PricingCategoryInput::Ps5(Ps5Input {
        model: Ps5Model::Slim,
        age_months: 12,
        condition: Condition::LikeNew,
        cleanliness: 5,
        fully_operational: true,
        missing_nonessential_features: 0,
        controller_count: 2,
        billing_unit: BillingUnit::ThirtyMinutes,
    });
    let recommendation = recommend(attributes.clone()).unwrap();
    let final_unit_price_cents = recommendation.final_price_cents(100).unwrap();
    repository
        .save_pricing_profile(SavePricingProfile {
            listing_id,
            owner_id,
            attributes,
            recommendation,
            seller_adjustment_cents: 100,
            final_unit_price_cents,
            allowed_fulfillment_methods: vec![
                FulfillmentMethod::Pickup,
                FulfillmentMethod::Delivery,
            ],
        })
        .await
        .expect("save profile");

    let snapshot = repository.pricing(listing_id).await.expect("read profile");
    assert_eq!(snapshot.unit_price_cents, final_unit_price_cents);
    assert_eq!(snapshot.service_fee_bps, 600);
    assert_eq!(snapshot.deposit_cents, 10_000);
    assert!(
        snapshot
            .allowed_fulfillment_methods
            .contains(&FulfillmentMethod::Pickup)
    );

    repository
        .database()
        .drop()
        .await
        .expect("drop test database");
}
