use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CheckoutRequest {
    pub order_id: Uuid,
    pub amount_cents: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutSession {
    pub payment_intent_id: String,
    pub checkout_url: String,
}

#[derive(Debug, Error)]
pub enum PaymentError {
    #[error("invalid checkout amount")]
    InvalidAmount,
    #[error("payment provider unavailable")]
    Unavailable,
}

#[async_trait]
pub trait PaymentGateway: Send + Sync {
    async fn create_checkout(
        &self,
        request: CheckoutRequest,
    ) -> Result<CheckoutSession, PaymentError>;
}
