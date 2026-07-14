use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::pricing::BillingUnit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FulfillmentMethod {
    Pickup,
    Delivery,
    OwnerLocation,
    OnSite,
}

impl FulfillmentMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pickup => "pickup",
            Self::Delivery => "delivery",
            Self::OwnerLocation => "owner_location",
            Self::OnSite => "on_site",
        }
    }
}

impl TryFrom<&str> for FulfillmentMethod {
    type Error = QuoteError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pickup" => Ok(Self::Pickup),
            "delivery" => Ok(Self::Delivery),
            "owner_location" => Ok(Self::OwnerLocation),
            "on_site" => Ok(Self::OnSite),
            _ => Err(QuoteError::InvalidFulfillment),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PricingSnapshot {
    pub unit_price_cents: i64,
    pub deposit_cents: i64,
    pub delivery_fee_cents: i64,
    pub service_fee_bps: i64,
    pub billing_unit: BillingUnit,
    pub allowed_fulfillment_methods: Vec<FulfillmentMethod>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteInput {
    pub units: i64,
    pub fulfillment: FulfillmentMethod,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteBreakdown {
    pub unit_price_cents: i64,
    pub billable_units: i64,
    pub billing_unit: BillingUnit,
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
    #[error("fulfillment method is not allowed")]
    InvalidFulfillment,
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
    let delivery_fee_cents = if input.fulfillment == FulfillmentMethod::Delivery {
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
        unit_price_cents: pricing.unit_price_cents,
        billable_units: input.units,
        billing_unit: pricing.billing_unit,
        base_cents,
        service_fee_cents,
        delivery_fee_cents,
        deposit_cents: pricing.deposit_cents,
        total_cents,
        currency: "USD".to_owned(),
    })
}
