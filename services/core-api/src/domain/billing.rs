use thiserror::Error;
use time::OffsetDateTime;

use super::pricing::BillingUnit;

const NANOS_PER_SECOND: i128 = 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BillingError {
    #[error("rental end must be after start")]
    InvalidWindow,
    #[error("billing duration overflow")]
    Overflow,
}

pub fn billable_units(
    start: OffsetDateTime,
    end: OffsetDateTime,
    billing_unit: BillingUnit,
) -> Result<i64, BillingError> {
    let duration_nanos = end
        .unix_timestamp_nanos()
        .checked_sub(start.unix_timestamp_nanos())
        .filter(|value| *value > 0)
        .ok_or(BillingError::InvalidWindow)?;
    let divisor = match billing_unit {
        BillingUnit::ThirtyMinutes => 30 * 60 * NANOS_PER_SECOND,
        BillingUnit::Day => 24 * 60 * 60 * NANOS_PER_SECOND,
    };
    let units = duration_nanos
        .checked_add(divisor - 1)
        .map(|value| value / divisor)
        .ok_or(BillingError::Overflow)?;
    i64::try_from(units).map_err(|_| BillingError::Overflow)
}
