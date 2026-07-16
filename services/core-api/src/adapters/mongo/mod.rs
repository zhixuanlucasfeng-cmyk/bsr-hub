pub mod bootstrap;
pub mod models;
pub mod slots;

use mongodb::{Collection, Database};
use thiserror::Error;

use models::{
    BookingSlotDocument, ListingDocument, OrderDocument, OrderEventDocument, PricingProfileDocument,
};

#[derive(Debug, Error)]
pub enum MongoAdapterError {
    #[error("MongoDB operation failed")]
    Driver(#[from] mongodb::error::Error),
    #[error("MongoDB persistence data is invalid: {0}")]
    InvalidData(&'static str),
}

#[derive(Clone)]
pub struct MongoCollections {
    pub listings: Collection<ListingDocument>,
    pub pricing_profiles: Collection<PricingProfileDocument>,
    pub orders: Collection<OrderDocument>,
    pub booking_slots: Collection<BookingSlotDocument>,
    pub order_events: Collection<OrderEventDocument>,
}

impl MongoCollections {
    pub fn new(database: Database) -> Self {
        Self {
            listings: database.collection("listings"),
            pricing_profiles: database.collection("pricing_profiles"),
            orders: database.collection("orders"),
            booking_slots: database.collection("booking_slots"),
            order_events: database.collection("order_events"),
        }
    }
}
