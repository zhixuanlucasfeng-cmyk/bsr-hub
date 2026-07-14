use serde::{Deserialize, Serialize};
use thiserror::Error;

const BASIS_POINTS: i64 = 10_000;
const MAX_SELLER_ADJUSTMENT_CENTS: i64 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingUnit {
    ThirtyMinutes,
    Day,
}

impl BillingUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ThirtyMinutes => "thirty_minutes",
            Self::Day => "day",
        }
    }
}

impl TryFrom<&str> for BillingUnit {
    type Error = PricingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "thirty_minutes" => Ok(Self::ThirtyMinutes),
            "day" => Ok(Self::Day),
            _ => Err(PricingError::InvalidAttributes),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ps5Model {
    Original,
    Slim,
    Pro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    LikeNew,
    Good,
    Fair,
    Worn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationTier {
    Residential,
    Suburban,
    Urban,
    Premium,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ps5Input {
    pub model: Ps5Model,
    pub age_months: i64,
    pub condition: Condition,
    pub cleanliness: i64,
    pub fully_operational: bool,
    pub missing_nonessential_features: i64,
    pub controller_count: i64,
    pub billing_unit: BillingUnit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceInput {
    pub square_feet: i64,
    pub location_tier: LocationTier,
    pub cleanliness: i64,
    pub equipment_score: i64,
    pub amenity_count: i64,
    pub billing_unit: BillingUnit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum PricingCategoryInput {
    Ps5(Ps5Input),
    Workspace(WorkspaceInput),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    pub recommended_unit_price_cents: i64,
    pub minimum_allowed_cents: i64,
    pub maximum_allowed_cents: i64,
    pub billing_unit: BillingUnit,
    pub ruleset_version: String,
    pub reason_codes: Vec<String>,
}

impl Recommendation {
    pub fn final_price_cents(&self, seller_adjustment_cents: i64) -> Result<i64, PricingError> {
        if !(-MAX_SELLER_ADJUSTMENT_CENTS..=MAX_SELLER_ADJUSTMENT_CENTS)
            .contains(&seller_adjustment_cents)
        {
            return Err(PricingError::AdjustmentOutOfRange);
        }
        self.recommended_unit_price_cents
            .checked_add(seller_adjustment_cents)
            .filter(|value| *value >= 0)
            .ok_or(PricingError::Overflow)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PricingError {
    #[error("pricing attributes are invalid")]
    InvalidAttributes,
    #[error("item must be fully operational")]
    NotOperational,
    #[error("seller adjustment must be between -500 and 500 cents")]
    AdjustmentOutOfRange,
    #[error("pricing arithmetic overflow")]
    Overflow,
}

pub fn recommend(input: PricingCategoryInput) -> Result<Recommendation, PricingError> {
    match input {
        PricingCategoryInput::Ps5(input) => recommend_ps5(input),
        PricingCategoryInput::Workspace(input) => recommend_workspace(input),
    }
}

fn recommend_ps5(input: Ps5Input) -> Result<Recommendation, PricingError> {
    if !input.fully_operational {
        return Err(PricingError::NotOperational);
    }
    if !(0..=120).contains(&input.age_months)
        || !(1..=5).contains(&input.cleanliness)
        || !(0..=3).contains(&input.missing_nonessential_features)
        || !(1..=4).contains(&input.controller_count)
    {
        return Err(PricingError::InvalidAttributes);
    }

    let base_cents = match (input.model, input.billing_unit) {
        (Ps5Model::Original, BillingUnit::ThirtyMinutes) => 500,
        (Ps5Model::Original, BillingUnit::Day) => 2_500,
        (Ps5Model::Slim, BillingUnit::ThirtyMinutes) => 600,
        (Ps5Model::Slim, BillingUnit::Day) => 3_000,
        (Ps5Model::Pro, BillingUnit::ThirtyMinutes) => 800,
        (Ps5Model::Pro, BillingUnit::Day) => 4_000,
    };
    let age_penalty = (input.age_months / 12).saturating_mul(500).min(2_000);
    let condition_adjustment = match input.condition {
        Condition::LikeNew => 1_000,
        Condition::Good => 0,
        Condition::Fair => -1_500,
        Condition::Worn => -3_000,
    };
    let missing_penalty = input
        .missing_nonessential_features
        .saturating_mul(1_500)
        .min(4_500);
    let multiplier = (BASIS_POINTS - age_penalty
        + condition_adjustment
        + cleanliness_adjustment(input.cleanliness)?
        - missing_penalty)
        .clamp(4_000, 13_000);
    let scaled = apply_basis_points(base_cents, multiplier)?;
    let extra_controllers = (input.controller_count - 1).clamp(0, 2);
    let controller_unit_cents = match input.billing_unit {
        BillingUnit::ThirtyMinutes => 100,
        BillingUnit::Day => 500,
    };
    let controller_cents = extra_controllers
        .checked_mul(controller_unit_cents)
        .ok_or(PricingError::Overflow)?;
    let price = round_to_50(
        scaled
            .checked_add(controller_cents)
            .ok_or(PricingError::Overflow)?,
    )?;

    let mut reasons = vec![match input.model {
        Ps5Model::Original => "MODEL_ORIGINAL".to_owned(),
        Ps5Model::Slim => "MODEL_SLIM".to_owned(),
        Ps5Model::Pro => "MODEL_PRO".to_owned(),
    }];
    reasons.push(format!("AGE_{}_MONTHS", input.age_months));
    reasons.push(match input.condition {
        Condition::LikeNew => "CONDITION_LIKE_NEW".to_owned(),
        Condition::Good => "CONDITION_GOOD".to_owned(),
        Condition::Fair => "CONDITION_FAIR".to_owned(),
        Condition::Worn => "CONDITION_WORN".to_owned(),
    });
    reasons.push(format!("CLEANLINESS_{}", input.cleanliness));
    if input.missing_nonessential_features > 0 {
        reasons.push("MISSING_NONESSENTIAL_FEATURE".to_owned());
    }
    if extra_controllers > 0 {
        reasons.push("EXTRA_CONTROLLER".to_owned());
    }
    Ok(recommendation(price, input.billing_unit, reasons))
}

fn recommend_workspace(input: WorkspaceInput) -> Result<Recommendation, PricingError> {
    if !(50..=20_000).contains(&input.square_feet)
        || !(1..=5).contains(&input.cleanliness)
        || !(0..=5).contains(&input.equipment_score)
        || !(0..=10).contains(&input.amenity_count)
    {
        return Err(PricingError::InvalidAttributes);
    }
    let half_hour_base = input
        .square_feet
        .checked_mul(2)
        .and_then(|value| value.checked_add(300))
        .ok_or(PricingError::Overflow)?;
    let base_cents = match input.billing_unit {
        BillingUnit::ThirtyMinutes => half_hour_base,
        BillingUnit::Day => half_hour_base
            .checked_mul(12)
            .ok_or(PricingError::Overflow)?,
    };
    let location_adjustment = match input.location_tier {
        LocationTier::Residential => -1_000,
        LocationTier::Suburban => 0,
        LocationTier::Urban => 1_500,
        LocationTier::Premium => 2_500,
    };
    let equipment_adjustment = input.equipment_score.saturating_mul(300).min(1_500);
    let amenity_adjustment = input.amenity_count.saturating_mul(200).min(1_000);
    let multiplier = (BASIS_POINTS
        + location_adjustment
        + cleanliness_adjustment(input.cleanliness)?
        + equipment_adjustment
        + amenity_adjustment)
        .clamp(5_000, 15_000);
    let price = round_to_50(apply_basis_points(base_cents, multiplier)?)?;
    let reasons = vec![
        format!("SIZE_{}_SQFT", input.square_feet),
        match input.location_tier {
            LocationTier::Residential => "LOCATION_RESIDENTIAL".to_owned(),
            LocationTier::Suburban => "LOCATION_SUBURBAN".to_owned(),
            LocationTier::Urban => "LOCATION_URBAN".to_owned(),
            LocationTier::Premium => "LOCATION_PREMIUM".to_owned(),
        },
        format!("CLEANLINESS_{}", input.cleanliness),
        format!("EQUIPMENT_SCORE_{}", input.equipment_score),
        format!("AMENITIES_{}", input.amenity_count),
    ];
    Ok(recommendation(price, input.billing_unit, reasons))
}

fn recommendation(
    recommended_unit_price_cents: i64,
    billing_unit: BillingUnit,
    reason_codes: Vec<String>,
) -> Recommendation {
    Recommendation {
        minimum_allowed_cents: recommended_unit_price_cents
            .saturating_sub(MAX_SELLER_ADJUSTMENT_CENTS),
        maximum_allowed_cents: recommended_unit_price_cents
            .saturating_add(MAX_SELLER_ADJUSTMENT_CENTS),
        recommended_unit_price_cents,
        billing_unit,
        ruleset_version: "rules-v1".to_owned(),
        reason_codes,
    }
}

fn cleanliness_adjustment(cleanliness: i64) -> Result<i64, PricingError> {
    match cleanliness {
        1 => Ok(-1_000),
        2 => Ok(-500),
        3 => Ok(0),
        4 => Ok(300),
        5 => Ok(500),
        _ => Err(PricingError::InvalidAttributes),
    }
}

fn apply_basis_points(cents: i64, multiplier: i64) -> Result<i64, PricingError> {
    cents
        .checked_mul(multiplier)
        .map(|value| value / BASIS_POINTS)
        .ok_or(PricingError::Overflow)
}

fn round_to_50(cents: i64) -> Result<i64, PricingError> {
    cents
        .checked_add(25)
        .map(|value| (value / 50) * 50)
        .ok_or(PricingError::Overflow)
}
