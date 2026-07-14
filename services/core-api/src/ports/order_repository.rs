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
    pub quote: QuoteBreakdown,
}

#[derive(Debug, Clone)]
pub struct ReservedOrder {
    pub order_id: Uuid,
    pub expires_at: OffsetDateTime,
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
    Database(#[from] sqlx::Error),
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
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
        _event_id: &str,
        _order_id: Option<Uuid>,
    ) -> Result<bool, ReserveError> {
        Ok(true)
    }
}
