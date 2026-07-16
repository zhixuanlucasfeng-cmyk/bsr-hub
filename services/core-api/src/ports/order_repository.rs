use async_trait::async_trait;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::order_state::{OrderAction, OrderState};
use crate::domain::pricing::{BillingUnit, PricingCategoryInput, Recommendation};
use crate::domain::quote::{FulfillmentMethod, PricingSnapshot, QuoteBreakdown};

#[derive(Debug, Clone)]
pub struct SavePricingProfile {
    pub listing_id: Uuid,
    pub owner_id: Uuid,
    pub attributes: PricingCategoryInput,
    pub recommendation: Recommendation,
    pub seller_adjustment_cents: i64,
    pub final_unit_price_cents: i64,
    pub allowed_fulfillment_methods: Vec<FulfillmentMethod>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredPricingProfile {
    pub listing_id: Uuid,
    pub recommended_unit_price_cents: i64,
    pub seller_adjustment_cents: i64,
    pub final_unit_price_cents: i64,
    pub minimum_allowed_cents: i64,
    pub maximum_allowed_cents: i64,
    pub billing_unit: BillingUnit,
    pub ruleset_version: String,
    pub reason_codes: Vec<String>,
    pub allowed_fulfillment_methods: Vec<FulfillmentMethod>,
}

#[derive(Clone)]
pub struct CreateOrder {
    pub listing_id: Uuid,
    pub buyer_id: Uuid,
    pub start_at: Option<OffsetDateTime>,
    pub end_at: Option<OffsetDateTime>,
    pub fulfillment: FulfillmentMethod,
    pub quote: QuoteBreakdown,
}

#[derive(Debug, Clone)]
pub struct ReservedOrder {
    pub order_id: Uuid,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPayment {
    pub event_id: String,
    pub order_id: Uuid,
    pub amount_total_cents: i64,
    pub currency: String,
}

#[derive(Debug, Clone)]
pub struct StoredOrderPayment {
    pub status: String,
    pub reservation_expires_at: OffsetDateTime,
    pub total_cents: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentValidation {
    Accepted,
    WrongState,
    Expired,
    AmountMismatch,
    CurrencyMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentEventOutcome {
    Applied,
    Duplicate,
    MissingOrder,
    Rejected(PaymentValidation),
}

pub fn validate_payment(
    payment: &VerifiedPayment,
    order: &StoredOrderPayment,
    now: OffsetDateTime,
) -> PaymentValidation {
    if order.status != "pending_payment" {
        return PaymentValidation::WrongState;
    }
    if order.reservation_expires_at <= now {
        return PaymentValidation::Expired;
    }
    if payment.amount_total_cents != order.total_cents {
        return PaymentValidation::AmountMismatch;
    }
    if !payment.currency.eq_ignore_ascii_case(&order.currency) {
        return PaymentValidation::CurrencyMismatch;
    }
    PaymentValidation::Accepted
}

#[derive(Debug, Error)]
pub enum ReserveError {
    #[error("listing not found")]
    NotFound,
    #[error("listing unavailable")]
    Unavailable,
    #[error("buyer cannot purchase own listing")]
    SelfTransaction,
    #[error("order action is forbidden")]
    Forbidden,
    #[error("invalid order state transition")]
    InvalidTransition,
    #[error("invalid pricing profile")]
    InvalidPricing,
    #[error("pricing profile not found")]
    PricingNotFound,
    #[error("database error")]
    Database(String),
}

impl ReserveError {
    pub fn database(error: impl std::fmt::Display) -> Self {
        Self::Database(error.to_string())
    }
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn readiness(&self) -> Result<(), ReserveError> {
        Ok(())
    }

    async fn pricing(&self, listing_id: Uuid) -> Result<PricingSnapshot, ReserveError>;

    async fn reserve(&self, order: CreateOrder) -> Result<ReservedOrder, ReserveError>;

    async fn save_pricing_profile(
        &self,
        _profile: SavePricingProfile,
    ) -> Result<StoredPricingProfile, ReserveError> {
        Err(ReserveError::InvalidPricing)
    }

    async fn transition(
        &self,
        _order_id: Uuid,
        _actor_id: Uuid,
        _action: OrderAction,
    ) -> Result<OrderState, ReserveError> {
        Err(ReserveError::InvalidTransition)
    }

    async fn apply_payment_event(
        &self,
        _payment: VerifiedPayment,
    ) -> Result<PaymentEventOutcome, ReserveError> {
        Ok(PaymentEventOutcome::Applied)
    }
}
