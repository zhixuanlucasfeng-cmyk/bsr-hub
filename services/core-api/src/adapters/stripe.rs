use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;

use crate::ports::payment_gateway::{
    CheckoutRequest, CheckoutSession, PaymentError, PaymentGateway,
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripeEvent {
    pub id: String,
    pub event_type: String,
    pub order_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WebhookError {
    #[error("malformed Stripe signature header")]
    MalformedHeader,
    #[error("webhook timestamp is outside the accepted window")]
    Expired,
    #[error("invalid Stripe signature")]
    InvalidSignature,
    #[error("invalid Stripe event payload")]
    InvalidPayload,
}

#[derive(Deserialize)]
struct RawStripeEvent {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    data: Option<StripeEventData>,
}

#[derive(Deserialize)]
struct StripeEventData {
    object: StripeEventObject,
}

#[derive(Deserialize)]
struct StripeEventObject {
    #[serde(default)]
    metadata: StripeMetadata,
}

#[derive(Default, Deserialize)]
struct StripeMetadata {
    order_id: Option<uuid::Uuid>,
}

pub fn verify_webhook(
    payload: &[u8],
    signature_header: &str,
    secret: &[u8],
    now_unix: i64,
) -> Result<StripeEvent, WebhookError> {
    let mut timestamp = None;
    let mut signature = None;
    for item in signature_header.split(',') {
        if let Some(value) = item.strip_prefix("t=") {
            timestamp = value.parse::<i64>().ok();
        } else if let Some(value) = item.strip_prefix("v1=") {
            signature = hex::decode(value).ok();
        }
    }
    let timestamp = timestamp.ok_or(WebhookError::MalformedHeader)?;
    let signature = signature.ok_or(WebhookError::MalformedHeader)?;
    if now_unix.abs_diff(timestamp) > 300 {
        return Err(WebhookError::Expired);
    }

    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| WebhookError::InvalidSignature)?;
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(payload);
    let expected = mac.finalize().into_bytes();
    if expected.as_slice().ct_eq(signature.as_slice()).unwrap_u8() != 1 {
        return Err(WebhookError::InvalidSignature);
    }

    let raw: RawStripeEvent =
        serde_json::from_slice(payload).map_err(|_| WebhookError::InvalidPayload)?;
    Ok(StripeEvent {
        id: raw.id,
        event_type: raw.event_type,
        order_id: raw.data.and_then(|data| data.object.metadata.order_id),
    })
}

pub fn test_signature_header(timestamp: i64, payload: &[u8], secret: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(payload);
    format!(
        "t={timestamp},v1={}",
        hex::encode(mac.finalize().into_bytes())
    )
}

#[derive(Debug, Clone)]
pub struct StripePaymentGateway {
    client: reqwest::Client,
    secret_key: String,
    success_url: String,
    cancel_url: String,
}

impl StripePaymentGateway {
    pub fn new(secret_key: String, success_url: String, cancel_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            secret_key,
            success_url,
            cancel_url,
        }
    }
}

#[derive(Deserialize)]
struct StripeCheckoutResponse {
    id: String,
    url: String,
}

#[async_trait::async_trait]
impl PaymentGateway for StripePaymentGateway {
    async fn create_checkout(
        &self,
        request: CheckoutRequest,
    ) -> Result<CheckoutSession, PaymentError> {
        if request.amount_cents <= 0 {
            return Err(PaymentError::InvalidAmount);
        }
        let fields = [
            ("mode", "payment".to_owned()),
            ("success_url", self.success_url.clone()),
            ("cancel_url", self.cancel_url.clone()),
            ("line_items[0][quantity]", "1".to_owned()),
            ("line_items[0][price_data][currency]", "usd".to_owned()),
            (
                "line_items[0][price_data][unit_amount]",
                request.amount_cents.to_string(),
            ),
            (
                "line_items[0][price_data][product_data][name]",
                "BSR Hub order".to_owned(),
            ),
            ("metadata[order_id]", request.order_id.to_string()),
        ];
        let response = self
            .client
            .post("https://api.stripe.com/v1/checkout/sessions")
            .bearer_auth(&self.secret_key)
            .form(&fields)
            .send()
            .await
            .map_err(|_| PaymentError::Unavailable)?;
        if !response.status().is_success() {
            return Err(PaymentError::Unavailable);
        }
        let response: StripeCheckoutResponse = response
            .json()
            .await
            .map_err(|_| PaymentError::Unavailable)?;
        Ok(CheckoutSession {
            payment_intent_id: response.id,
            checkout_url: response.url,
        })
    }
}
