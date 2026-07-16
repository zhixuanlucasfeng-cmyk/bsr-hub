use core_api::{
    adapters::mongo::{
        MongoOrderRepository,
        seed::{
            FICTIONAL_OWNER_ID, FICTIONAL_RENTAL_ID, FICTIONAL_SECOND_HAND_ID,
            seed_fictional_catalog,
        },
    },
    domain::{
        order_state::{OrderAction, OrderState},
        pricing::BillingUnit,
        quote::{FulfillmentMethod, QuoteBreakdown},
    },
    ports::order_repository::{
        CreateOrder, OrderRepository, PaymentEventOutcome, PaymentValidation, ReserveError,
        VerifiedPayment,
    },
};
use time::macros::datetime;
use uuid::Uuid;

fn order(buyer_id: Uuid, hour: u8) -> CreateOrder {
    let start = datetime!(2026-07-21 00:00 UTC) + time::Duration::hours(hour.into());
    CreateOrder {
        listing_id: Uuid::parse_str(FICTIONAL_RENTAL_ID).unwrap(),
        buyer_id,
        start_at: Some(start),
        end_at: Some(start + time::Duration::minutes(30)),
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
async fn payment_is_idempotent_and_participants_control_transitions() {
    let uri = std::env::var("MONGODB_TEST_URI").expect("MONGODB_TEST_URI is required");
    let database_name = format!("bsr_test_{}", Uuid::new_v4().simple());
    let repository = MongoOrderRepository::connect(&uri, &database_name, 600, 30)
        .await
        .expect("connect test repository");
    seed_fictional_catalog(repository.database()).await.unwrap();

    let buyer_id = Uuid::new_v4();
    let seller_id = Uuid::parse_str(FICTIONAL_OWNER_ID).unwrap();
    let reserved = repository.reserve(order(buyer_id, 10)).await.unwrap();
    let payment = VerifiedPayment {
        event_id: "evt_test_lifecycle_1".to_owned(),
        order_id: reserved.order_id,
        amount_total_cents: 10_636,
        currency: "usd".to_owned(),
    };
    assert_eq!(
        repository
            .apply_payment_event(payment.clone())
            .await
            .unwrap(),
        PaymentEventOutcome::Applied
    );
    assert_eq!(
        repository.apply_payment_event(payment).await.unwrap(),
        PaymentEventOutcome::Duplicate
    );

    assert!(matches!(
        repository
            .transition(reserved.order_id, buyer_id, OrderAction::Confirm)
            .await,
        Err(ReserveError::Forbidden)
    ));
    assert_eq!(
        repository
            .transition(reserved.order_id, seller_id, OrderAction::Confirm)
            .await
            .unwrap(),
        OrderState::Confirmed
    );
    assert_eq!(
        repository
            .transition(reserved.order_id, buyer_id, OrderAction::Activate)
            .await
            .unwrap(),
        OrderState::Active
    );
    assert_eq!(
        repository
            .transition(reserved.order_id, buyer_id, OrderAction::Return)
            .await
            .unwrap(),
        OrderState::Returned
    );
    assert_eq!(
        repository
            .transition(reserved.order_id, seller_id, OrderAction::Complete)
            .await
            .unwrap(),
        OrderState::Completed
    );

    let second = repository.reserve(order(Uuid::new_v4(), 12)).await.unwrap();
    let mismatch = VerifiedPayment {
        event_id: "evt_test_lifecycle_2".to_owned(),
        order_id: second.order_id,
        amount_total_cents: 1,
        currency: "USD".to_owned(),
    };
    assert_eq!(
        repository.apply_payment_event(mismatch).await.unwrap(),
        PaymentEventOutcome::Rejected(PaymentValidation::AmountMismatch)
    );

    repository.database().drop().await.unwrap();
}

#[tokio::test]
#[ignore = "requires MONGODB_TEST_URI replica set"]
async fn cancelling_second_hand_order_restores_inventory_once() {
    let uri = std::env::var("MONGODB_TEST_URI").expect("MONGODB_TEST_URI is required");
    let database_name = format!("bsr_test_{}", Uuid::new_v4().simple());
    let repository = MongoOrderRepository::connect(&uri, &database_name, 600, 30)
        .await
        .expect("connect test repository");
    seed_fictional_catalog(repository.database()).await.unwrap();
    repository
        .database()
        .collection::<mongodb::bson::Document>("listings")
        .update_one(
            mongodb::bson::doc! { "_id": FICTIONAL_SECOND_HAND_ID },
            mongodb::bson::doc! { "$set": { "inventory": 1_i64 } },
        )
        .await
        .unwrap();

    let buyer_id = Uuid::new_v4();
    let held = repository
        .reserve(second_hand_order(buyer_id))
        .await
        .unwrap();
    assert_eq!(
        repository
            .transition(held.order_id, buyer_id, OrderAction::Cancel)
            .await
            .unwrap(),
        OrderState::Cancelled
    );
    assert!(
        repository
            .reserve(second_hand_order(Uuid::new_v4()))
            .await
            .is_ok()
    );

    repository.database().drop().await.unwrap();
}
