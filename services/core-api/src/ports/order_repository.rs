use async_trait::async_trait;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::order_state::{OrderAction, OrderState};
use crate::domain::quote::{PricingSnapshot, QuoteBreakdown};

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
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn pricing(&self, listing_id: Uuid) -> Result<PricingSnapshot, ReserveError>;

    async fn reserve(&self, order: CreateOrder) -> Result<ReservedOrder, ReserveError>;

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
