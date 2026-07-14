use core_api::ports::order_repository::{
    PaymentValidation, StoredOrderPayment, VerifiedPayment, validate_payment,
};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

fn payment() -> VerifiedPayment {
    VerifiedPayment {
        event_id: "evt_test".to_owned(),
        order_id: Uuid::nil(),
        amount_total_cents: 16_800,
        currency: "usd".to_owned(),
    }
}

fn stored(now: OffsetDateTime) -> StoredOrderPayment {
    StoredOrderPayment {
        status: "pending_payment".to_owned(),
        reservation_expires_at: now + Duration::minutes(5),
        total_cents: 16_800,
        currency: "USD".to_owned(),
    }
}

#[test]
fn exact_unexpired_payment_is_accepted() {
    let now = OffsetDateTime::UNIX_EPOCH;
    assert_eq!(
        validate_payment(&payment(), &stored(now), now),
        PaymentValidation::Accepted
    );
}

#[test]
fn amount_currency_status_and_expiration_must_match() {
    let now = OffsetDateTime::UNIX_EPOCH;

    let mut changed = payment();
    changed.amount_total_cents -= 1;
    assert_eq!(
        validate_payment(&changed, &stored(now), now),
        PaymentValidation::AmountMismatch
    );

    let mut changed = payment();
    changed.currency = "eur".to_owned();
    assert_eq!(
        validate_payment(&changed, &stored(now), now),
        PaymentValidation::CurrencyMismatch
    );

    let mut changed_order = stored(now);
    changed_order.status = "paid".to_owned();
    assert_eq!(
        validate_payment(&payment(), &changed_order, now),
        PaymentValidation::WrongState
    );

    let mut changed_order = stored(now);
    changed_order.reservation_expires_at = now;
    assert_eq!(
        validate_payment(&payment(), &changed_order, now),
        PaymentValidation::Expired
    );
}
