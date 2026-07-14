use core_api::domain::{
    billing::{BillingError, billable_units},
    pricing::BillingUnit,
};
use time::{Duration, OffsetDateTime};

#[test]
fn thirty_minute_billing_rounds_up_and_has_one_unit_minimum() {
    let start = OffsetDateTime::UNIX_EPOCH;
    for (minutes, expected) in [(1, 1), (20, 1), (30, 1), (31, 2), (50, 2), (60, 2), (61, 3)] {
        let end = start + Duration::minutes(minutes);
        assert_eq!(
            billable_units(start, end, BillingUnit::ThirtyMinutes).unwrap(),
            expected,
            "{minutes} minutes"
        );
    }
}

#[test]
fn daily_billing_rounds_up() {
    let start = OffsetDateTime::UNIX_EPOCH;
    assert_eq!(
        billable_units(start, start + Duration::seconds(1), BillingUnit::Day).unwrap(),
        1
    );
    assert_eq!(
        billable_units(start, start + Duration::days(1), BillingUnit::Day).unwrap(),
        1
    );
    assert_eq!(
        billable_units(
            start,
            start + Duration::days(1) + Duration::seconds(1),
            BillingUnit::Day
        )
        .unwrap(),
        2
    );
}

#[test]
fn empty_or_backwards_windows_are_rejected() {
    let start = OffsetDateTime::UNIX_EPOCH;
    assert_eq!(
        billable_units(start, start, BillingUnit::ThirtyMinutes),
        Err(BillingError::InvalidWindow)
    );
    assert_eq!(
        billable_units(start, start - Duration::minutes(1), BillingUnit::Day),
        Err(BillingError::InvalidWindow)
    );
}
