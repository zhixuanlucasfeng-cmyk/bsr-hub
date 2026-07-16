use mongodb::{Database, bson::doc};
use serde::Serialize;
use time::OffsetDateTime;

use super::{MongoAdapterError, bson_datetime, models::ListingDocument};

pub const FICTIONAL_OWNER_ID: &str = "11111111-1111-4111-8111-111111111111";
pub const FICTIONAL_RENTAL_ID: &str = "22222222-2222-4222-8222-222222222221";
pub const FICTIONAL_WORKSPACE_ID: &str = "22222222-2222-4222-8222-222222222222";
pub const FICTIONAL_SECOND_HAND_ID: &str = "22222222-2222-4222-8222-222222222223";

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SeedReport {
    pub listings: u64,
}

pub async fn seed_fictional_catalog(database: &Database) -> Result<SeedReport, MongoAdapterError> {
    let listings = database.collection::<ListingDocument>("listings");
    let now = bson_datetime(OffsetDateTime::now_utc());
    let documents = [
        listing(
            FICTIONAL_RENTAL_ID,
            "rental",
            "gaming",
            "Fictional PS5 Slim Kit",
            "Classroom demo console with two controllers.",
            1,
            10_000,
            1_500,
            vec!["pickup", "delivery", "owner_location"],
            doc! { "model": "slim", "condition": "like_new" },
            now,
        ),
        listing(
            FICTIONAL_WORKSPACE_ID,
            "workspace",
            "creative_space",
            "Fictional Photo Studio",
            "Classroom demo studio; use occurs on site.",
            1,
            15_000,
            0,
            vec!["on_site"],
            doc! { "square_feet": 500_i64, "equipment_score": 4_i64 },
            now,
        ),
        listing(
            FICTIONAL_SECOND_HAND_ID,
            "second_hand",
            "computers",
            "Fictional Refurbished Monitor",
            "Classroom demo second-hand display.",
            2,
            0,
            1_000,
            vec!["pickup", "delivery"],
            doc! { "condition": "good" },
            now,
        ),
    ];
    for document in documents {
        let id = document.id.clone();
        listings
            .replace_one(doc! { "_id": &id }, document)
            .upsert(true)
            .await?;
    }
    Ok(SeedReport {
        listings: listings.count_documents(doc! {}).await?,
    })
}

#[allow(clippy::too_many_arguments)]
fn listing(
    id: &str,
    listing_type: &str,
    category: &str,
    title: &str,
    description: &str,
    inventory: i64,
    deposit_cents: i64,
    delivery_fee_cents: i64,
    fulfillment: Vec<&str>,
    attributes: mongodb::bson::Document,
    now: mongodb::bson::DateTime,
) -> ListingDocument {
    ListingDocument {
        id: id.to_owned(),
        owner_id: FICTIONAL_OWNER_ID.to_owned(),
        listing_type: listing_type.to_owned(),
        category: category.to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        status: "active".to_owned(),
        inventory,
        deposit_cents,
        delivery_fee_cents,
        allowed_fulfillment_methods: fulfillment.into_iter().map(str::to_owned).collect(),
        attributes,
        created_at: now,
        updated_at: now,
        schema_version: 1,
    }
}
