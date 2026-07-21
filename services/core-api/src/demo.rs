use std::sync::{Mutex, OnceLock};

use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    order_state::{OrderAction, OrderState},
    pricing::BillingUnit,
    quote::{FulfillmentMethod, PricingSnapshot, QuoteBreakdown, QuoteInput, calculate_quote},
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoListing {
    pub id: &'static str,
    pub owner_id: &'static str,
    pub title: &'static str,
    pub listing_type: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub city: &'static str,
    pub state: &'static str,
    pub condition: &'static str,
    pub unit_price_cents: i64,
    pub deposit_cents: i64,
    pub delivery_fee_cents: i64,
    pub billing_unit: BillingUnit,
    pub fulfillment: Vec<FulfillmentMethod>,
    pub accent: &'static str,
    pub icon: &'static str,
    pub image_src: &'static str,
    pub image_alt: &'static str,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoOrder {
    pub id: String,
    pub listing_id: String,
    pub listing_title: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub state: OrderState,
    pub fulfillment: FulfillmentMethod,
    pub quote: QuoteBreakdown,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuoteRequest {
    listing_id: String,
    units: i64,
    fulfillment: FulfillmentMethod,
}

#[derive(Deserialize)]
struct ActionRequest {
    action: OrderAction,
}

#[derive(Default, Deserialize)]
struct OrderQuery {
    persona: Option<String>,
}

static ORDERS: OnceLock<Mutex<Vec<DemoOrder>>> = OnceLock::new();

fn orders() -> &'static Mutex<Vec<DemoOrder>> {
    ORDERS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn router() -> Router {
    Router::new()
        .route("/v1/demo/listings", get(listings_handler))
        .route("/v1/demo/listings/{id}", get(listing_handler))
        .route("/v1/demo/quote", post(quote_handler))
        .route(
            "/v1/demo/orders",
            get(orders_handler).post(create_order_handler),
        )
        .route("/v1/demo/orders/{id}/actions", post(action_handler))
        .route("/v1/demo/reset", post(reset_handler))
}

async fn listings_handler() -> Json<Vec<DemoListing>> {
    Json(listings())
}

async fn listing_handler(Path(id): Path<String>) -> Result<Json<DemoListing>, StatusCode> {
    listing(&id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

async fn quote_handler(
    Json(request): Json<QuoteRequest>,
) -> Result<Json<QuoteBreakdown>, StatusCode> {
    quote(&request).map(Json)
}

async fn create_order_handler(
    Json(request): Json<QuoteRequest>,
) -> Result<(StatusCode, Json<DemoOrder>), StatusCode> {
    let listing = listing(&request.listing_id).ok_or(StatusCode::NOT_FOUND)?;
    let quote = quote(&request)?;
    let order = DemoOrder {
        id: Uuid::new_v4().to_string(),
        listing_id: listing.id.to_owned(),
        listing_title: listing.title.to_owned(),
        buyer_id: "buyer-demo".into(),
        seller_id: listing.owner_id.into(),
        state: OrderState::PendingPayment,
        fulfillment: request.fulfillment,
        quote,
    };
    orders()
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .push(order.clone());
    Ok((StatusCode::CREATED, Json(order)))
}

async fn orders_handler(
    Query(query): Query<OrderQuery>,
) -> Result<Json<Vec<DemoOrder>>, StatusCode> {
    let persona = query.persona.as_deref().unwrap_or("buyer-demo");
    let values = orders()
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .iter()
        .filter(|order| order.buyer_id == persona || order.seller_id == persona)
        .cloned()
        .collect();
    Ok(Json(values))
}

async fn action_handler(
    Path(id): Path<String>,
    Json(request): Json<ActionRequest>,
) -> Result<Json<DemoOrder>, StatusCode> {
    let mut values = orders()
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let order = values
        .iter_mut()
        .find(|order| order.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    order.state = order
        .state
        .transition(request.action)
        .map_err(|_| StatusCode::CONFLICT)?;
    Ok(Json(order.clone()))
}

async fn reset_handler() -> StatusCode {
    if let Ok(mut values) = orders().lock() {
        values.clear();
    }
    StatusCode::NO_CONTENT
}

fn quote(request: &QuoteRequest) -> Result<QuoteBreakdown, StatusCode> {
    let listing = listing(&request.listing_id).ok_or(StatusCode::NOT_FOUND)?;
    if !listing.fulfillment.contains(&request.fulfillment) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    calculate_quote(
        PricingSnapshot {
            unit_price_cents: listing.unit_price_cents,
            deposit_cents: listing.deposit_cents,
            delivery_fee_cents: listing.delivery_fee_cents,
            service_fee_bps: 600,
            billing_unit: listing.billing_unit,
            allowed_fulfillment_methods: listing.fulfillment,
        },
        QuoteInput {
            units: request.units,
            fulfillment: request.fulfillment,
        },
    )
    .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)
}

fn listing(id: &str) -> Option<DemoListing> {
    listings().into_iter().find(|listing| listing.id == id)
}

pub fn listings() -> Vec<DemoListing> {
    use BillingUnit::{Day, ThirtyMinutes};
    use FulfillmentMethod::{Delivery, OnSite, OwnerLocation, Pickup};
    vec![
        DemoListing {
            id: "ps5-slim",
            owner_id: "seller-demo",
            title: "PS5 Slim + Two Controllers",
            listing_type: "rental",
            category: "Gaming",
            description: "Quiet, clean console with Spider-Man 2 and HDMI cable. Great for a weekend with friends.",
            city: "Wellesley",
            state: "MA",
            condition: "Like new",
            unit_price_cents: 1200,
            deposit_cents: 10000,
            delivery_fee_cents: 800,
            billing_unit: ThirtyMinutes,
            fulfillment: vec![Pickup, Delivery, OwnerLocation],
            accent: "#6d5dfc",
            icon: "🎮",
            image_src: "/images/listings/ps5-slim.jpg",
            image_alt: "White PlayStation 5 console with two controllers",
        },
        DemoListing {
            id: "ps5-pro",
            owner_id: "seller-demo",
            title: "PS5 Pro Creator Setup",
            listing_type: "rental",
            category: "Gaming",
            description: "High-performance console with headset and capture card for streams or tournaments.",
            city: "Newton",
            state: "MA",
            condition: "Excellent",
            unit_price_cents: 1800,
            deposit_cents: 15000,
            delivery_fee_cents: 1200,
            billing_unit: ThirtyMinutes,
            fulfillment: vec![Pickup, Delivery],
            accent: "#8b5cf6",
            icon: "🕹️",
            image_src: "/images/listings/ps5-pro.jpg",
            image_alt: "Gaming console, controller, and streaming headset setup",
        },
        DemoListing {
            id: "macbook",
            owner_id: "tech-demo",
            title: "MacBook Pro M2 14-inch",
            listing_type: "rental",
            category: "Computers",
            description: "Portable editing and development laptop with charger and protective sleeve.",
            city: "Needham",
            state: "MA",
            condition: "Excellent",
            unit_price_cents: 5200,
            deposit_cents: 20000,
            delivery_fee_cents: 1000,
            billing_unit: Day,
            fulfillment: vec![Pickup, Delivery],
            accent: "#0ea5e9",
            icon: "💻",
            image_src: "/images/listings/macbook.jpg",
            image_alt: "Open MacBook Pro laptop on a clean desk",
        },
        DemoListing {
            id: "gaming-pc",
            owner_id: "tech-demo",
            title: "RTX Gaming & Rendering PC",
            listing_type: "rental",
            category: "Computers",
            description: "RTX 4070 desktop for gaming, 3D work, and short production projects.",
            city: "Waltham",
            state: "MA",
            condition: "Very good",
            unit_price_cents: 6800,
            deposit_cents: 25000,
            delivery_fee_cents: 1800,
            billing_unit: Day,
            fulfillment: vec![Delivery, OwnerLocation],
            accent: "#06b6d4",
            icon: "🖥️",
            image_src: "/images/listings/gaming-pc.jpg",
            image_alt: "RGB desktop computer and monitor gaming setup",
        },
        DemoListing {
            id: "sony-camera",
            owner_id: "creator-demo",
            title: "Sony A7 IV Camera Kit",
            listing_type: "rental",
            category: "Cameras",
            description: "Full-frame camera, 28–70mm lens, batteries, SD card, and compact tripod.",
            city: "Brookline",
            state: "MA",
            condition: "Excellent",
            unit_price_cents: 7500,
            deposit_cents: 30000,
            delivery_fee_cents: 1200,
            billing_unit: Day,
            fulfillment: vec![Pickup, Delivery],
            accent: "#f97316",
            icon: "📷",
            image_src: "/images/listings/sony-camera.jpg",
            image_alt: "Mirrorless camera with lens and creator accessories",
        },
        DemoListing {
            id: "podcast-kit",
            owner_id: "creator-demo",
            title: "Two-Person Podcast Kit",
            listing_type: "rental",
            category: "Cameras",
            description: "Two microphones, audio interface, table arms, headphones, and cables.",
            city: "Boston",
            state: "MA",
            condition: "Good",
            unit_price_cents: 4200,
            deposit_cents: 12000,
            delivery_fee_cents: 1000,
            billing_unit: Day,
            fulfillment: vec![Pickup, Delivery],
            accent: "#fb7185",
            icon: "🎙️",
            image_src: "/images/listings/podcast-kit.jpg",
            image_alt: "Two microphones and headphones arranged for a podcast",
        },
        DemoListing {
            id: "tool-set",
            owner_id: "maker-demo",
            title: "Cordless Home Project Tool Set",
            listing_type: "rental",
            category: "Tools",
            description: "Drill, driver, sander, batteries, safety glasses, and organized case.",
            city: "Dedham",
            state: "MA",
            condition: "Good",
            unit_price_cents: 2800,
            deposit_cents: 8000,
            delivery_fee_cents: 900,
            billing_unit: Day,
            fulfillment: vec![Pickup, Delivery],
            accent: "#f59e0b",
            icon: "🛠️",
            image_src: "/images/listings/tool-set.jpg",
            image_alt: "Cordless drill and home project tools on a workbench",
        },
        DemoListing {
            id: "laser-cutter",
            owner_id: "maker-demo",
            title: "Desktop Laser Cutter Session",
            listing_type: "workspace",
            category: "Maker space",
            description: "Supervised laser cutter and ventilation station. Materials charged separately.",
            city: "Cambridge",
            state: "MA",
            condition: "Professional",
            unit_price_cents: 2200,
            deposit_cents: 0,
            delivery_fee_cents: 0,
            billing_unit: ThirtyMinutes,
            fulfillment: vec![OnSite],
            accent: "#eab308",
            icon: "⚙️",
            image_src: "/images/listings/laser-cutter.jpg",
            image_alt: "Laser cutting machine in a supervised maker workshop",
        },
        DemoListing {
            id: "photo-studio",
            owner_id: "seller-demo",
            title: "Natural-Light Photo Studio",
            listing_type: "workspace",
            category: "Studio",
            description: "500 sq ft studio with backdrops, lights, changing area, and freight elevator.",
            city: "Boston",
            state: "MA",
            condition: "Professional",
            unit_price_cents: 3500,
            deposit_cents: 5000,
            delivery_fee_cents: 0,
            billing_unit: ThirtyMinutes,
            fulfillment: vec![OnSite],
            accent: "#ec4899",
            icon: "🎬",
            image_src: "/images/listings/photo-studio.jpg",
            image_alt: "Bright photography studio with lights and backdrop",
        },
        DemoListing {
            id: "print-shop",
            owner_id: "business-demo",
            title: "Small-Batch Print Workshop",
            listing_type: "workspace",
            category: "Printing",
            description: "Book presses, cutters, worktables, and help from an experienced operator.",
            city: "Somerville",
            state: "MA",
            condition: "Professional",
            unit_price_cents: 3000,
            deposit_cents: 0,
            delivery_fee_cents: 0,
            billing_unit: ThirtyMinutes,
            fulfillment: vec![OnSite],
            accent: "#14b8a6",
            icon: "🖨️",
            image_src: "/images/listings/print-shop.jpg",
            image_alt: "Small print workshop with presses and worktables",
        },
        DemoListing {
            id: "monitor-sale",
            owner_id: "tech-demo",
            title: "Used 27-inch 4K Monitor",
            listing_type: "sale",
            category: "Second-hand",
            description: "Color-accurate display with stand and cables. Small cosmetic mark on rear shell.",
            city: "Newton",
            state: "MA",
            condition: "Very good",
            unit_price_cents: 16500,
            deposit_cents: 0,
            delivery_fee_cents: 1500,
            billing_unit: Day,
            fulfillment: vec![Pickup, Delivery],
            accent: "#3b82f6",
            icon: "🖥️",
            image_src: "/images/listings/monitor-sale.jpg",
            image_alt: "Twenty-seven inch monitor on a desktop",
        },
        DemoListing {
            id: "camera-sale",
            owner_id: "creator-demo",
            title: "Second-Hand Instant Camera",
            listing_type: "sale",
            category: "Second-hand",
            description: "Tested instant camera with case and one unopened film pack.",
            city: "Wellesley",
            state: "MA",
            condition: "Good",
            unit_price_cents: 6500,
            deposit_cents: 0,
            delivery_fee_cents: 700,
            billing_unit: Day,
            fulfillment: vec![Pickup, Delivery],
            accent: "#a855f7",
            icon: "📸",
            image_src: "/images/listings/camera-sale.jpg",
            image_alt: "Second-hand instant camera with film pack",
        },
    ]
}
