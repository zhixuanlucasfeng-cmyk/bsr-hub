use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct PricingSnapshot {
    pub unit_price_cents: i64,
    pub deposit_cents: i64,
    pub delivery_fee_cents: i64,
    pub service_fee_bps: i64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteInput {
    pub units: i64,
    pub wants_delivery: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteBreakdown {
    pub base_cents: i64,
    pub service_fee_cents: i64,
    pub delivery_fee_cents: i64,
    pub deposit_cents: i64,
    pub total_cents: i64,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum QuoteError {
    #[error("units must be positive")]
    InvalidUnits,
    #[error("pricing values must be non-negative")]
    NegativeAmount,
    #[error("quote arithmetic overflow")]
    Overflow,
}

pub fn calculate_quote(
    pricing: PricingSnapshot,
    input: QuoteInput,
) -> Result<QuoteBreakdown, QuoteError> {
    if input.units <= 0 {
        return Err(QuoteError::InvalidUnits);
    }
    if [
        pricing.unit_price_cents,
        pricing.deposit_cents,
        pricing.delivery_fee_cents,
        pricing.service_fee_bps,
    ]
    .iter()
    .any(|value| *value < 0)
    {
        return Err(QuoteError::NegativeAmount);
    }

    let base_cents = pricing
        .unit_price_cents
        .checked_mul(input.units)
        .ok_or(QuoteError::Overflow)?;
    let service_fee_cents = base_cents
        .checked_mul(pricing.service_fee_bps)
        .ok_or(QuoteError::Overflow)?
        / 10_000;
    let delivery_fee_cents = if input.wants_delivery {
        pricing.delivery_fee_cents
    } else {
        0
    };
    let total_cents = base_cents
        .checked_add(service_fee_cents)
        .and_then(|value| value.checked_add(delivery_fee_cents))
        .and_then(|value| value.checked_add(pricing.deposit_cents))
        .ok_or(QuoteError::Overflow)?;

    Ok(QuoteBreakdown {
        base_cents,
        service_fee_cents,
        delivery_fee_cents,
        deposit_cents: pricing.deposit_cents,
        total_cents,
        currency: "USD".to_owned(),
    })
}
