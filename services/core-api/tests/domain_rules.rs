use core_api::domain::order_state::{OrderAction, OrderState, TransitionError};
use core_api::domain::pricing::BillingUnit;
use core_api::domain::quote::{
    FulfillmentMethod, PricingSnapshot, QuoteError, QuoteInput, calculate_quote,
};

#[test]
fn rental_quote_uses_integer_cents_and_six_percent_fee() {
    let quote = calculate_quote(
        PricingSnapshot {
            unit_price_cents: 2_500,
            deposit_cents: 10_000,
            delivery_fee_cents: 1_500,
            service_fee_bps: 600,
            billing_unit: BillingUnit::Day,
        },
        QuoteInput {
            units: 2,
            fulfillment: FulfillmentMethod::Delivery,
        },
    )
    .unwrap();
    assert_eq!(quote.base_cents, 5_000);
    assert_eq!(quote.service_fee_cents, 300);
    assert_eq!(quote.delivery_fee_cents, 1_500);
    assert_eq!(quote.deposit_cents, 10_000);
    assert_eq!(quote.total_cents, 16_800);
    assert_eq!(quote.unit_price_cents, 2_500);
    assert_eq!(quote.billable_units, 2);
    assert_eq!(quote.billing_unit, BillingUnit::Day);
}

#[test]
fn invalid_quote_inputs_are_rejected() {
    let zero_units = calculate_quote(
        PricingSnapshot {
            unit_price_cents: 2_500,
            deposit_cents: 0,
            delivery_fee_cents: 0,
            service_fee_bps: 600,
            billing_unit: BillingUnit::ThirtyMinutes,
        },
        QuoteInput {
            units: 0,
            fulfillment: FulfillmentMethod::Pickup,
        },
    );
    assert_eq!(zero_units, Err(QuoteError::InvalidUnits));

    let negative_money = calculate_quote(
        PricingSnapshot {
            unit_price_cents: -1,
            deposit_cents: 0,
            delivery_fee_cents: 0,
            service_fee_bps: 600,
            billing_unit: BillingUnit::ThirtyMinutes,
        },
        QuoteInput {
            units: 1,
            fulfillment: FulfillmentMethod::Pickup,
        },
    );
    assert_eq!(negative_money, Err(QuoteError::NegativeAmount));
}

#[test]
fn order_state_machine_accepts_only_valid_transitions() {
    assert_eq!(
        OrderState::PendingPayment.transition(OrderAction::MarkPaid),
        Ok(OrderState::Paid)
    );
    assert_eq!(
        OrderState::Paid.transition(OrderAction::Confirm),
        Ok(OrderState::Confirmed)
    );
    assert_eq!(
        OrderState::PendingPayment.transition(OrderAction::Expire),
        Ok(OrderState::Expired)
    );
    assert_eq!(
        OrderState::Completed.transition(OrderAction::Cancel),
        Err(TransitionError::Invalid)
    );
}
