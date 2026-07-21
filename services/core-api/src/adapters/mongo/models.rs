use mongodb::bson::{DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListingDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub owner_id: String,
    pub listing_type: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub inventory: i64,
    pub deposit_cents: i64,
    pub delivery_fee_cents: i64,
    pub allowed_fulfillment_methods: Vec<String>,
    pub attributes: Document,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub schema_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingProfileDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub listing_id: String,
    pub attributes: Document,
    pub recommended_unit_price_cents: i64,
    pub seller_adjustment_cents: i64,
    pub final_unit_price_cents: i64,
    pub minimum_allowed_cents: i64,
    pub maximum_allowed_cents: i64,
    pub deposit_cents: i64,
    pub delivery_fee_cents: i64,
    pub service_fee_bps: i64,
    pub billing_unit: String,
    pub ruleset_version: String,
    pub reason_codes: Vec<String>,
    pub allowed_fulfillment_methods: Vec<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub schema_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteDocument {
    pub unit_price_cents: i64,
    pub billable_units: i64,
    pub billing_unit: String,
    pub base_cents: i64,
    pub service_fee_cents: i64,
    pub delivery_fee_cents: i64,
    pub deposit_cents: i64,
    pub total_cents: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub listing_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub state: String,
    pub fulfillment: String,
    pub start_at: Option<DateTime>,
    pub end_at: Option<DateTime>,
    pub quote: QuoteDocument,
    pub reservation_expires_at: DateTime,
    pub payment_provider: Option<String>,
    pub payment_event_id: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub schema_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingSlotDocument {
    pub listing_id: String,
    pub slot_start: DateTime,
    pub order_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime>,
    pub schema_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderEventDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub order_id: String,
    pub event_type: String,
    pub actor_id: Option<String>,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
    pub provider: Option<String>,
    pub provider_event_id: Option<String>,
    pub metadata: Document,
    pub created_at: DateTime,
    pub schema_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub auth_user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub trust_level: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub schema_version: i32,
}
