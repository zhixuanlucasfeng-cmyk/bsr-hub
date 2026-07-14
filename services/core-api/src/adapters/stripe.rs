use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;

use crate::ports::order_repository::VerifiedPayment;
use crate::ports::payment_gateway::{
    CheckoutRequest, CheckoutSession, PaymentError, PaymentGateway,
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripeEvent {
    pub id: String,
    pub event_type: String,
    pub order_id: Option<uuid::Uuid>,
    pub payment_status: Option<String>,
    pub amount_total_cents: Option<i64>,
    pub currency: Option<String>,
}

impl StripeEvent {
    pub fn verified_payment(&self) -> Option<VerifiedPayment> {
        if self.event_type != "checkout.session.completed"
            || self.payment_status.as_deref() != Some("paid")
        {
            return None;
        }
        Some(VerifiedPayment {
            event_id: self.id.clone(),
            order_id: self.order_id?,
            amount_total_cents: self.amount_total_cents?,
            currency: self.currency.clone()?,
        })
    }
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
    payment_status: Option<String>,
    amount_total: Option<i64>,
    currency: Option<String>,
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
    let object = raw.data.map(|data| data.object);
    Ok(StripeEvent {
        id: raw.id,
        event_type: raw.event_type,
        order_id: object.as_ref().and_then(|item| item.metadata.order_id),
        payment_status: object.as_ref().and_then(|item| item.payment_status.clone()),
        amount_total_cents: object.as_ref().and_then(|item| item.amount_total),
        currency: object.and_then(|item| item.currency),
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
            checkout_session_id: response.id,
            checkout_url: response.url,
        })
    }
}
