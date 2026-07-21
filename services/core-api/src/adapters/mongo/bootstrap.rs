use std::time::Duration;

use mongodb::{
    Database, IndexModel,
    bson::{Bson, Document, doc},
    options::IndexOptions,
};

use super::MongoAdapterError;

const COLLECTIONS: &[&str] = &[
    "listings",
    "pricing_profiles",
    "orders",
    "booking_slots",
    "order_events",
    "user_profiles",
];

pub async fn bootstrap(database: &Database) -> Result<(), MongoAdapterError> {
    let existing = database.list_collection_names().await?;
    for name in COLLECTIONS {
        if !existing.iter().any(|candidate| candidate == name) {
            database.create_collection(*name).await?;
        }
    }

    database
        .collection::<Document>("pricing_profiles")
        .create_index(index(
            doc! { "listing_id": 1 },
            "pricing_listing_unique",
            true,
        ))
        .await?;
    database
        .collection::<Document>("booking_slots")
        .create_indexes([
            index(
                doc! { "listing_id": 1, "slot_start": 1 },
                "booking_slot_unique",
                true,
            ),
            IndexModel::builder()
                .keys(doc! { "expires_at": 1 })
                .options(
                    IndexOptions::builder()
                        .name("booking_slot_expiry".to_owned())
                        .expire_after(Duration::ZERO)
                        .build(),
                )
                .build(),
        ])
        .await?;
    database
        .collection::<Document>("order_events")
        .create_indexes([
            index(
                doc! { "order_id": 1, "created_at": 1 },
                "order_event_timeline",
                false,
            ),
            IndexModel::builder()
                .keys(doc! { "provider": 1, "provider_event_id": 1 })
                .options(
                    IndexOptions::builder()
                        .name("provider_event_unique".to_owned())
                        .unique(true)
                        .partial_filter_expression(doc! {
                            "provider_event_id": { "$type": "string" }
                        })
                        .build(),
                )
                .build(),
        ])
        .await?;
    database
        .collection::<Document>("user_profiles")
        .create_index(index(
            doc! { "auth_user_id": 1 },
            "user_profile_auth_user_unique",
            true,
        ))
        .await?;

    apply_validators(database).await?;
    Ok(())
}

fn index(keys: Document, name: &str, unique: bool) -> IndexModel {
    IndexModel::builder()
        .keys(keys)
        .options(
            IndexOptions::builder()
                .name(name.to_owned())
                .unique(unique)
                .build(),
        )
        .build()
}

async fn apply_validators(database: &Database) -> Result<(), MongoAdapterError> {
    let schemas = [
        (
            "listings",
            json_schema(
                &[
                    "_id",
                    "owner_id",
                    "listing_type",
                    "status",
                    "inventory",
                    "schema_version",
                ],
                doc! {
                    "_id": { "bsonType": "string" },
                    "owner_id": { "bsonType": "string" },
                    "listing_type": { "enum": ["rental", "workspace", "second_hand"] },
                    "status": { "enum": ["active", "paused", "sold", "archived"] },
                    "inventory": { "bsonType": "long", "minimum": 0_i64 },
                    "deposit_cents": { "bsonType": "long", "minimum": 0_i64 },
                    "delivery_fee_cents": { "bsonType": "long", "minimum": 0_i64 },
                    "schema_version": { "bsonType": "int", "minimum": 1 },
                },
            ),
        ),
        (
            "pricing_profiles",
            json_schema(
                &[
                    "listing_id",
                    "final_unit_price_cents",
                    "billing_unit",
                    "schema_version",
                ],
                doc! {
                    "listing_id": { "bsonType": "string" },
                    "final_unit_price_cents": { "bsonType": "long", "minimum": 0_i64 },
                    "billing_unit": { "enum": ["thirty_minutes", "day"] },
                    "schema_version": { "bsonType": "int", "minimum": 1 },
                },
            ),
        ),
        (
            "orders",
            json_schema(
                &[
                    "_id",
                    "listing_id",
                    "buyer_id",
                    "seller_id",
                    "state",
                    "quote",
                    "schema_version",
                ],
                doc! {
                    "_id": { "bsonType": "string" },
                    "listing_id": { "bsonType": "string" },
                    "buyer_id": { "bsonType": "string" },
                    "seller_id": { "bsonType": "string" },
                    "state": { "enum": ["pending_payment", "paid", "confirmed", "active", "fulfilled", "returned", "completed", "cancelled", "expired"] },
                    "quote": { "bsonType": "object", "required": ["total_cents", "currency"] },
                    "schema_version": { "bsonType": "int", "minimum": 1 },
                },
            ),
        ),
        (
            "booking_slots",
            json_schema(
                &["listing_id", "slot_start", "order_id", "schema_version"],
                doc! {
                    "listing_id": { "bsonType": "string" },
                    "slot_start": { "bsonType": "date" },
                    "order_id": { "bsonType": "string" },
                    "expires_at": { "bsonType": ["date", "null"] },
                    "schema_version": { "bsonType": "int", "minimum": 1 },
                },
            ),
        ),
        (
            "order_events",
            json_schema(
                &[
                    "_id",
                    "order_id",
                    "event_type",
                    "created_at",
                    "schema_version",
                ],
                doc! {
                    "_id": { "bsonType": "string" },
                    "order_id": { "bsonType": "string" },
                    "event_type": { "bsonType": "string" },
                    "created_at": { "bsonType": "date" },
                    "schema_version": { "bsonType": "int", "minimum": 1 },
                },
            ),
        ),
        (
            "user_profiles",
            json_schema(
                &[
                    "_id",
                    "auth_user_id",
                    "display_name",
                    "role",
                    "trust_level",
                    "created_at",
                    "updated_at",
                    "schema_version",
                ],
                doc! {
                    "_id": { "bsonType": "string" },
                    "auth_user_id": { "bsonType": "string" },
                    "display_name": { "bsonType": "string", "minLength": 1, "maxLength": 80 },
                    "avatar_url": { "bsonType": ["string", "null"] },
                    "role": { "enum": ["buyer", "seller", "runner", "admin"] },
                    "trust_level": { "bsonType": "int", "minimum": 0 },
                    "created_at": { "bsonType": "date" },
                    "updated_at": { "bsonType": "date" },
                    "schema_version": { "bsonType": "int", "minimum": 1 },
                },
            ),
        ),
    ];

    for (name, schema) in schemas {
        database
            .run_command(doc! {
                "collMod": name,
                "validator": { "$jsonSchema": schema },
                "validationLevel": "strict",
                "validationAction": "error",
            })
            .await?;
    }
    Ok(())
}

fn json_schema(required: &[&str], properties: Document) -> Document {
    doc! {
        "bsonType": "object",
        "required": required.iter().map(|value| Bson::String((*value).to_owned())).collect::<Vec<_>>(),
        "properties": properties,
    }
}
