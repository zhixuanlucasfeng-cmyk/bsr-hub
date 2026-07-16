use core_api::{
    adapters::mongo::{
        MongoOrderRepository,
        seed::{
            FICTIONAL_OWNER_ID, FICTIONAL_RENTAL_ID, FICTIONAL_SECOND_HAND_ID,
            seed_fictional_catalog,
        },
    },
    domain::{
        pricing::BillingUnit,
        quote::{FulfillmentMethod, QuoteBreakdown},
    },
    ports::order_repository::{CreateOrder, OrderRepository, ReserveError},
};
use time::macros::datetime;
use uuid::Uuid;

fn order(buyer_id: Uuid, start_hour: u8) -> CreateOrder {
    let start_at = datetime!(2026-07-20 00:00 UTC) + time::Duration::hours(start_hour.into());
    CreateOrder {
        listing_id: Uuid::parse_str(FICTIONAL_RENTAL_ID).unwrap(),
        buyer_id,
        start_at: Some(start_at),
        end_at: Some(start_at + time::Duration::minutes(30)),
        fulfillment: FulfillmentMethod::Pickup,
        quote: QuoteBreakdown {
            unit_price_cents: 600,
            billable_units: 1,
            billing_unit: BillingUnit::ThirtyMinutes,
            base_cents: 600,
            service_fee_cents: 36,
            delivery_fee_cents: 0,
            deposit_cents: 10_000,
            total_cents: 10_636,
            currency: "USD".to_owned(),
        },
    }
}

fn second_hand_order(buyer_id: Uuid) -> CreateOrder {
    CreateOrder {
        listing_id: Uuid::parse_str(FICTIONAL_SECOND_HAND_ID).unwrap(),
        buyer_id,
        start_at: None,
        end_at: None,
        fulfillment: FulfillmentMethod::Pickup,
        quote: QuoteBreakdown {
            unit_price_cents: 8_000,
            billable_units: 1,
            billing_unit: BillingUnit::Day,
            base_cents: 8_000,
            service_fee_cents: 480,
            delivery_fee_cents: 0,
            deposit_cents: 0,
            total_cents: 8_480,
            currency: "USD".to_owned(),
        },
    }
}

#[tokio::test]
#[ignore = "requires MONGODB_TEST_URI replica set"]
async fn one_of_twenty_overlapping_reservations_wins() {
    let uri = std::env::var("MONGODB_TEST_URI").expect("MONGODB_TEST_URI is required");
    let database_name = format!("bsr_test_{}", Uuid::new_v4().simple());
    let repository = MongoOrderRepository::connect(&uri, &database_name, 600, 30)
        .await
        .expect("connect test repository");
    seed_fictional_catalog(repository.database())
        .await
        .expect("seed fictional listing");

    let mut tasks = Vec::new();
    for _ in 0..20 {
        let repository = repository.clone();
        tasks.push(tokio::spawn(async move {
            repository.reserve(order(Uuid::new_v4(), 10)).await
        }));
    }
    let mut successes = 0;
    let mut conflicts = 0;
    for task in tasks {
        match task.await.unwrap() {
            Ok(_) => successes += 1,
            Err(ReserveError::Unavailable) => conflicts += 1,
            Err(error) => panic!("unexpected reservation error: {error:?}"),
        }
    }
    assert_eq!((successes, conflicts), (1, 19));

    assert!(repository.reserve(order(Uuid::new_v4(), 11)).await.is_ok());
    let owner_id = Uuid::parse_str(FICTIONAL_OWNER_ID).unwrap();
    assert!(matches!(
        repository.reserve(order(owner_id, 12)).await,
        Err(ReserveError::SelfTransaction)
    ));

    repository.database().drop().await.unwrap();
}

#[tokio::test]
#[ignore = "requires MONGODB_TEST_URI replica set"]
async fn expired_second_hand_hold_restores_inventory() {
    let uri = std::env::var("MONGODB_TEST_URI").expect("MONGODB_TEST_URI is required");
    let database_name = format!("bsr_test_{}", Uuid::new_v4().simple());
    let repository = MongoOrderRepository::connect(&uri, &database_name, 600, 0)
        .await
        .expect("connect test repository");
    seed_fictional_catalog(repository.database())
        .await
        .expect("seed fictional listing");
    repository
        .database()
        .collection::<mongodb::bson::Document>("listings")
        .update_one(
            mongodb::bson::doc! { "_id": FICTIONAL_SECOND_HAND_ID },
            mongodb::bson::doc! { "$set": { "inventory": 1_i64 } },
        )
        .await
        .unwrap();

    repository
        .reserve(second_hand_order(Uuid::new_v4()))
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    assert!(
        repository
            .reserve(second_hand_order(Uuid::new_v4()))
            .await
            .is_ok()
    );

    repository.database().drop().await.unwrap();
}
