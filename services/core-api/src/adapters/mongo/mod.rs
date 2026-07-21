pub mod bootstrap;
pub mod models;
pub mod profiles;
pub mod seed;
pub mod slots;

use async_trait::async_trait;
use mongodb::{
    Client, ClientSession, Collection, Database,
    bson::doc,
    error::{
        ErrorKind, TRANSIENT_TRANSACTION_ERROR, UNKNOWN_TRANSACTION_COMMIT_RESULT, WriteFailure,
    },
    options::ClientOptions,
};
use thiserror::Error;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    domain::{
        order_state::{OrderAction, OrderState},
        pricing::BillingUnit,
        quote::{FulfillmentMethod, PricingSnapshot, QuoteBreakdown},
    },
    ports::order_repository::{
        CreateOrder, OrderRepository, PaymentEventOutcome, PaymentValidation, ReserveError,
        ReservedOrder, SavePricingProfile, StoredOrderPayment, StoredPricingProfile,
        VerifiedPayment, validate_payment,
    },
};

use models::{
    BookingSlotDocument, ListingDocument, OrderDocument, OrderEventDocument,
    PricingProfileDocument, QuoteDocument, UserProfileDocument,
};

const MAX_TRANSACTION_ATTEMPTS: u32 = 8;

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
    pub user_profiles: Collection<UserProfileDocument>,
}

impl MongoCollections {
    pub fn new(database: Database) -> Self {
        Self {
            listings: database.collection("listings"),
            pricing_profiles: database.collection("pricing_profiles"),
            orders: database.collection("orders"),
            booking_slots: database.collection("booking_slots"),
            order_events: database.collection("order_events"),
            user_profiles: database.collection("user_profiles"),
        }
    }
}

#[derive(Clone)]
pub struct MongoOrderRepository {
    database: Database,
    collections: MongoCollections,
    service_fee_bps: i64,
    reservation_minutes: i64,
}

impl MongoOrderRepository {
    pub async fn connect(
        uri: &str,
        database_name: &str,
        service_fee_bps: i64,
        reservation_minutes: i64,
    ) -> Result<Self, MongoAdapterError> {
        let mut options = ClientOptions::parse(uri).await?;
        options.app_name = Some("bsr-hub-core-api".to_owned());
        options.server_selection_timeout = Some(std::time::Duration::from_secs(3));
        let client = Client::with_options(options)?;
        let database = client.database(database_name);
        bootstrap::bootstrap(&database).await?;
        Ok(Self {
            collections: MongoCollections::new(database.clone()),
            database,
            service_fee_bps,
            reservation_minutes,
        })
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn profile_repository(&self) -> profiles::MongoProfileRepository {
        profiles::MongoProfileRepository::new(self.collections.user_profiles.clone())
    }

    async fn reserve_once(&self, order: &CreateOrder) -> Result<ReservedOrder, TransactionError> {
        let now = OffsetDateTime::now_utc();
        let expires_at = now + Duration::minutes(self.reservation_minutes);
        let order_id = Uuid::new_v4();
        let mut session = self.database.client().start_session().await?;
        session.start_transaction().await?;

        let result = self
            .write_reservation(&mut session, order, order_id, expires_at, now)
            .await;
        if let Err(error) = result {
            let _ = session.abort_transaction().await;
            return Err(error);
        }

        let mut commit_attempts = 0;
        loop {
            match session.commit_transaction().await {
                Ok(()) => break,
                Err(error)
                    if error.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT)
                        && commit_attempts < 2 =>
                {
                    commit_attempts += 1;
                }
                Err(error) => return Err(TransactionError::Mongo(error)),
            }
        }
        Ok(ReservedOrder {
            order_id,
            expires_at,
        })
    }

    async fn write_reservation(
        &self,
        session: &mut ClientSession,
        order: &CreateOrder,
        order_id: Uuid,
        expires_at: OffsetDateTime,
        now: OffsetDateTime,
    ) -> Result<(), TransactionError> {
        let listing_id = order.listing_id.to_string();
        let listing = self
            .collections
            .listings
            .find_one(doc! { "_id": &listing_id, "status": "active" })
            .session(&mut *session)
            .await?
            .ok_or(ReserveError::NotFound)?;
        if listing.owner_id == order.buyer_id.to_string() {
            return Err(ReserveError::SelfTransaction.into());
        }
        if !listing
            .allowed_fulfillment_methods
            .iter()
            .any(|method| method == order.fulfillment.as_str())
        {
            return Err(ReserveError::InvalidPricing.into());
        }

        let expired = self
            .collections
            .orders
            .update_many(
                doc! {
                    "listing_id": &listing_id,
                    "state": "pending_payment",
                    "reservation_expires_at": { "$lte": bson_datetime(now) },
                },
                doc! { "$set": { "state": "expired", "updated_at": bson_datetime(now) } },
            )
            .session(&mut *session)
            .await?;
        if expired.modified_count > 0 {
            let quantity =
                i64::try_from(expired.modified_count).map_err(|_| ReserveError::Unavailable)?;
            self.restore_second_hand_inventory(session, &listing_id, quantity, now)
                .await?;
        }
        self.collections
            .booking_slots
            .delete_many(doc! {
                "listing_id": &listing_id,
                "expires_at": { "$lte": bson_datetime(now) },
            })
            .session(&mut *session)
            .await?;

        let is_timed = matches!(listing.listing_type.as_str(), "rental" | "workspace");
        if is_timed {
            let (start, end) = order
                .start_at
                .zip(order.end_at)
                .ok_or(ReserveError::Unavailable)?;
            let boundaries =
                slots::slot_boundaries(start, end).map_err(|_| ReserveError::Unavailable)?;
            let documents = boundaries
                .into_iter()
                .map(|slot_start| BookingSlotDocument {
                    listing_id: listing_id.clone(),
                    slot_start: bson_datetime(slot_start),
                    order_id: order_id.to_string(),
                    expires_at: Some(bson_datetime(expires_at)),
                    schema_version: 1,
                })
                .collect::<Vec<_>>();
            if let Err(error) = self
                .collections
                .booking_slots
                .insert_many(documents)
                .session(&mut *session)
                .await
            {
                if is_duplicate_key(&error) {
                    return Err(ReserveError::Unavailable.into());
                }
                return Err(error.into());
            }
        } else if listing.listing_type == "second_hand" {
            let update = self
                .collections
                .listings
                .update_one(
                    doc! { "_id": &listing_id, "status": "active", "inventory": { "$gt": 0_i64 } },
                    doc! { "$inc": { "inventory": -1_i64 }, "$set": { "updated_at": bson_datetime(now) } },
                )
                .session(&mut *session)
                .await?;
            if update.modified_count != 1 {
                return Err(ReserveError::Unavailable.into());
            }
        } else {
            return Err(ReserveError::Unavailable.into());
        }

        let document = OrderDocument {
            id: order_id.to_string(),
            listing_id: listing_id.clone(),
            buyer_id: order.buyer_id.to_string(),
            seller_id: listing.owner_id,
            state: "pending_payment".to_owned(),
            fulfillment: order.fulfillment.as_str().to_owned(),
            start_at: order.start_at.map(bson_datetime),
            end_at: order.end_at.map(bson_datetime),
            quote: quote_document(&order.quote),
            reservation_expires_at: bson_datetime(expires_at),
            payment_provider: None,
            payment_event_id: None,
            created_at: bson_datetime(now),
            updated_at: bson_datetime(now),
            schema_version: 1,
        };
        self.collections
            .orders
            .insert_one(document)
            .session(&mut *session)
            .await?;
        self.collections
            .order_events
            .insert_one(OrderEventDocument {
                id: Uuid::new_v4().to_string(),
                order_id: order_id.to_string(),
                event_type: "reservation_created".to_owned(),
                actor_id: Some(order.buyer_id.to_string()),
                from_state: None,
                to_state: Some("pending_payment".to_owned()),
                provider: None,
                provider_event_id: None,
                metadata: doc! {},
                created_at: bson_datetime(now),
                schema_version: 1,
            })
            .session(&mut *session)
            .await?;
        Ok(())
    }

    async fn restore_second_hand_inventory(
        &self,
        session: &mut ClientSession,
        listing_id: &str,
        quantity: i64,
        now: OffsetDateTime,
    ) -> Result<(), mongodb::error::Error> {
        self.collections
            .listings
            .update_one(
                doc! { "_id": listing_id, "listing_type": "second_hand" },
                doc! {
                    "$inc": { "inventory": quantity },
                    "$set": { "updated_at": bson_datetime(now) },
                },
            )
            .session(session)
            .await?;
        Ok(())
    }

    async fn apply_payment_once(
        &self,
        payment: &VerifiedPayment,
    ) -> Result<PaymentEventOutcome, TransactionError> {
        let now = OffsetDateTime::now_utc();
        let mut session = self.database.client().start_session().await?;
        session.start_transaction().await?;
        let event_id = Uuid::new_v4().to_string();
        let event = OrderEventDocument {
            id: event_id.clone(),
            order_id: payment.order_id.to_string(),
            event_type: "payment_received".to_owned(),
            actor_id: None,
            from_state: None,
            to_state: None,
            provider: Some("stripe".to_owned()),
            provider_event_id: Some(payment.event_id.clone()),
            metadata: doc! {},
            created_at: bson_datetime(now),
            schema_version: 1,
        };
        if let Err(error) = self
            .collections
            .order_events
            .insert_one(event)
            .session(&mut session)
            .await
        {
            let _ = session.abort_transaction().await;
            if is_duplicate_key(&error) {
                return Ok(PaymentEventOutcome::Duplicate);
            }
            return Err(error.into());
        }

        let order = self
            .collections
            .orders
            .find_one(doc! { "_id": payment.order_id.to_string() })
            .session(&mut session)
            .await?;
        let Some(order) = order else {
            self.collections
                .order_events
                .update_one(
                    doc! { "_id": &event_id },
                    doc! { "$set": { "event_type": "payment_missing_order" } },
                )
                .session(&mut session)
                .await?;
            session.commit_transaction().await?;
            return Ok(PaymentEventOutcome::MissingOrder);
        };
        let stored = StoredOrderPayment {
            status: order.state.clone(),
            reservation_expires_at: offset_datetime(order.reservation_expires_at)?,
            total_cents: order.quote.total_cents,
            currency: order.quote.currency.clone(),
        };
        let validation = validate_payment(payment, &stored, now);
        let outcome = if validation == PaymentValidation::Accepted {
            let update = self
                .collections
                .orders
                .update_one(
                    doc! { "_id": payment.order_id.to_string(), "state": "pending_payment" },
                    doc! { "$set": {
                        "state": "paid",
                        "payment_provider": "stripe",
                        "payment_event_id": &payment.event_id,
                        "updated_at": bson_datetime(now),
                    } },
                )
                .session(&mut session)
                .await?;
            if update.modified_count != 1 {
                let _ = session.abort_transaction().await;
                return Ok(PaymentEventOutcome::Rejected(PaymentValidation::WrongState));
            }
            self.collections
                .booking_slots
                .update_many(
                    doc! { "order_id": payment.order_id.to_string() },
                    doc! { "$unset": { "expires_at": "" } },
                )
                .session(&mut session)
                .await?;
            PaymentEventOutcome::Applied
        } else {
            if validation == PaymentValidation::Expired {
                let update = self
                    .collections
                    .orders
                    .update_one(
                        doc! { "_id": payment.order_id.to_string(), "state": "pending_payment" },
                        doc! { "$set": { "state": "expired", "updated_at": bson_datetime(now) } },
                    )
                    .session(&mut session)
                    .await?;
                if update.modified_count != 1 {
                    let _ = session.abort_transaction().await;
                    return Ok(PaymentEventOutcome::Rejected(PaymentValidation::WrongState));
                }
                self.collections
                    .booking_slots
                    .delete_many(doc! { "order_id": payment.order_id.to_string() })
                    .session(&mut session)
                    .await?;
                self.restore_second_hand_inventory(&mut session, &order.listing_id, 1, now)
                    .await?;
            }
            PaymentEventOutcome::Rejected(validation)
        };
        let event_type = match outcome {
            PaymentEventOutcome::Applied => "payment_succeeded",
            PaymentEventOutcome::Rejected(PaymentValidation::WrongState) => {
                "payment_rejected_wrong_state"
            }
            PaymentEventOutcome::Rejected(PaymentValidation::Expired) => "payment_rejected_expired",
            PaymentEventOutcome::Rejected(PaymentValidation::AmountMismatch) => {
                "payment_rejected_amount_mismatch"
            }
            PaymentEventOutcome::Rejected(PaymentValidation::CurrencyMismatch) => {
                "payment_rejected_currency_mismatch"
            }
            PaymentEventOutcome::Duplicate | PaymentEventOutcome::MissingOrder => unreachable!(),
            PaymentEventOutcome::Rejected(PaymentValidation::Accepted) => unreachable!(),
        };
        self.collections
            .order_events
            .update_one(
                doc! { "_id": &event_id },
                doc! { "$set": { "event_type": event_type, "from_state": &order.state, "to_state": match outcome { PaymentEventOutcome::Applied => "paid", PaymentEventOutcome::Rejected(PaymentValidation::Expired) => "expired", _ => &order.state } } },
            )
            .session(&mut session)
            .await?;
        session.commit_transaction().await?;
        Ok(outcome)
    }

    async fn transition_once(
        &self,
        order_id: Uuid,
        actor_id: Uuid,
        action: OrderAction,
    ) -> Result<OrderState, TransactionError> {
        let now = OffsetDateTime::now_utc();
        let mut session = self.database.client().start_session().await?;
        session.start_transaction().await?;
        let order = self
            .collections
            .orders
            .find_one(doc! { "_id": order_id.to_string() })
            .session(&mut session)
            .await?
            .ok_or(ReserveError::NotFound)?;
        let current = OrderState::try_from(order.state.as_str())
            .map_err(|_| ReserveError::InvalidTransition)?;
        let actor = actor_id.to_string();
        let authorized = match action {
            OrderAction::Confirm | OrderAction::Fulfill => actor == order.seller_id,
            OrderAction::Activate | OrderAction::Return => actor == order.buyer_id,
            OrderAction::Complete | OrderAction::Cancel => {
                actor == order.seller_id || actor == order.buyer_id
            }
            OrderAction::MarkPaid | OrderAction::Expire => false,
        };
        if !authorized {
            let _ = session.abort_transaction().await;
            return Err(ReserveError::Forbidden.into());
        }
        let next = current
            .transition(action)
            .map_err(|_| ReserveError::InvalidTransition)?;
        let update = self
            .collections
            .orders
            .update_one(
                doc! { "_id": order_id.to_string(), "state": current.as_str() },
                doc! { "$set": { "state": next.as_str(), "updated_at": bson_datetime(now) } },
            )
            .session(&mut session)
            .await?;
        if update.modified_count != 1 {
            let _ = session.abort_transaction().await;
            return Err(ReserveError::InvalidTransition.into());
        }
        if next == OrderState::Cancelled {
            self.collections
                .booking_slots
                .delete_many(doc! { "order_id": order_id.to_string() })
                .session(&mut session)
                .await?;
            self.restore_second_hand_inventory(&mut session, &order.listing_id, 1, now)
                .await?;
        }
        self.collections
            .order_events
            .insert_one(OrderEventDocument {
                id: Uuid::new_v4().to_string(),
                order_id: order_id.to_string(),
                event_type: format!("state_changed_to_{}", next.as_str()),
                actor_id: Some(actor),
                from_state: Some(current.as_str().to_owned()),
                to_state: Some(next.as_str().to_owned()),
                provider: None,
                provider_event_id: None,
                metadata: doc! {},
                created_at: bson_datetime(now),
                schema_version: 1,
            })
            .session(&mut session)
            .await?;
        session.commit_transaction().await?;
        Ok(next)
    }
}

#[async_trait]
impl OrderRepository for MongoOrderRepository {
    async fn readiness(&self) -> Result<(), ReserveError> {
        self.database
            .run_command(doc! { "ping": 1 })
            .await
            .map(|_| ())
            .map_err(ReserveError::database)
    }

    async fn pricing(&self, listing_id: Uuid) -> Result<PricingSnapshot, ReserveError> {
        let listing_id = listing_id.to_string();
        let listing = self
            .collections
            .listings
            .find_one(doc! { "_id": &listing_id, "status": "active" })
            .await
            .map_err(ReserveError::database)?
            .ok_or(ReserveError::NotFound)?;
        let profile = self
            .collections
            .pricing_profiles
            .find_one(doc! { "listing_id": &listing_id })
            .await
            .map_err(ReserveError::database)?
            .ok_or(ReserveError::PricingNotFound)?;

        if profile.final_unit_price_cents < 0
            || listing.deposit_cents < 0
            || listing.delivery_fee_cents < 0
            || self.service_fee_bps < 0
        {
            return Err(ReserveError::InvalidPricing);
        }
        let billing_unit = BillingUnit::try_from(profile.billing_unit.as_str())
            .map_err(|_| ReserveError::InvalidPricing)?;
        let allowed_fulfillment_methods = profile
            .allowed_fulfillment_methods
            .iter()
            .map(|value| {
                FulfillmentMethod::try_from(value.as_str())
                    .map_err(|_| ReserveError::InvalidPricing)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PricingSnapshot {
            unit_price_cents: profile.final_unit_price_cents,
            deposit_cents: listing.deposit_cents,
            delivery_fee_cents: listing.delivery_fee_cents,
            service_fee_bps: self.service_fee_bps,
            billing_unit,
            allowed_fulfillment_methods,
        })
    }

    async fn reserve(&self, order: CreateOrder) -> Result<ReservedOrder, ReserveError> {
        for attempt in 0..MAX_TRANSACTION_ATTEMPTS {
            match self.reserve_once(&order).await {
                Ok(reserved) => return Ok(reserved),
                Err(TransactionError::Domain(error)) => return Err(error),
                Err(TransactionError::Mongo(error))
                    if error.contains_label(TRANSIENT_TRANSACTION_ERROR)
                        && attempt + 1 < MAX_TRANSACTION_ATTEMPTS =>
                {
                    let backoff_ms = u64::from((attempt + 1).pow(2)) * 2;
                    tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                }
                Err(TransactionError::Mongo(error)) => {
                    return Err(ReserveError::database(error));
                }
            }
        }
        Err(ReserveError::database("transaction retry limit exceeded"))
    }

    async fn save_pricing_profile(
        &self,
        profile: SavePricingProfile,
    ) -> Result<StoredPricingProfile, ReserveError> {
        let listing_id = profile.listing_id.to_string();
        let owner_id = profile.owner_id.to_string();
        let listing = self
            .collections
            .listings
            .find_one(doc! { "_id": &listing_id, "status": "active" })
            .await
            .map_err(ReserveError::database)?
            .ok_or(ReserveError::NotFound)?;
        if listing.owner_id != owner_id {
            return Err(ReserveError::Forbidden);
        }
        if profile.final_unit_price_cents
            != profile
                .recommendation
                .final_price_cents(profile.seller_adjustment_cents)
                .map_err(|_| ReserveError::InvalidPricing)?
        {
            return Err(ReserveError::InvalidPricing);
        }

        let now = bson_datetime(OffsetDateTime::now_utc());
        let document = PricingProfileDocument {
            id: listing_id.clone(),
            listing_id: listing_id.clone(),
            attributes: mongodb::bson::to_document(&profile.attributes)
                .map_err(|_| ReserveError::InvalidPricing)?,
            recommended_unit_price_cents: profile.recommendation.recommended_unit_price_cents,
            seller_adjustment_cents: profile.seller_adjustment_cents,
            final_unit_price_cents: profile.final_unit_price_cents,
            minimum_allowed_cents: profile.recommendation.minimum_allowed_cents,
            maximum_allowed_cents: profile.recommendation.maximum_allowed_cents,
            deposit_cents: listing.deposit_cents,
            delivery_fee_cents: listing.delivery_fee_cents,
            service_fee_bps: self.service_fee_bps,
            billing_unit: profile.recommendation.billing_unit.as_str().to_owned(),
            ruleset_version: profile.recommendation.ruleset_version.clone(),
            reason_codes: profile.recommendation.reason_codes.clone(),
            allowed_fulfillment_methods: profile
                .allowed_fulfillment_methods
                .iter()
                .map(|method| method.as_str().to_owned())
                .collect(),
            created_at: now,
            updated_at: now,
            schema_version: 1,
        };
        self.collections
            .pricing_profiles
            .replace_one(doc! { "listing_id": &listing_id }, document)
            .upsert(true)
            .await
            .map_err(ReserveError::database)?;

        Ok(StoredPricingProfile {
            listing_id: profile.listing_id,
            recommended_unit_price_cents: profile.recommendation.recommended_unit_price_cents,
            seller_adjustment_cents: profile.seller_adjustment_cents,
            final_unit_price_cents: profile.final_unit_price_cents,
            minimum_allowed_cents: profile.recommendation.minimum_allowed_cents,
            maximum_allowed_cents: profile.recommendation.maximum_allowed_cents,
            billing_unit: profile.recommendation.billing_unit,
            ruleset_version: profile.recommendation.ruleset_version,
            reason_codes: profile.recommendation.reason_codes,
            allowed_fulfillment_methods: profile.allowed_fulfillment_methods,
        })
    }

    async fn transition(
        &self,
        order_id: Uuid,
        actor_id: Uuid,
        action: OrderAction,
    ) -> Result<OrderState, ReserveError> {
        self.transition_once(order_id, actor_id, action)
            .await
            .map_err(transaction_error)
    }

    async fn apply_payment_event(
        &self,
        payment: VerifiedPayment,
    ) -> Result<PaymentEventOutcome, ReserveError> {
        self.apply_payment_once(&payment)
            .await
            .map_err(transaction_error)
    }
}

pub(crate) fn bson_datetime(value: OffsetDateTime) -> mongodb::bson::DateTime {
    let milliseconds = value.unix_timestamp_nanos() / 1_000_000;
    mongodb::bson::DateTime::from_millis(milliseconds as i64)
}

fn quote_document(quote: &QuoteBreakdown) -> QuoteDocument {
    QuoteDocument {
        unit_price_cents: quote.unit_price_cents,
        billable_units: quote.billable_units,
        billing_unit: quote.billing_unit.as_str().to_owned(),
        base_cents: quote.base_cents,
        service_fee_cents: quote.service_fee_cents,
        delivery_fee_cents: quote.delivery_fee_cents,
        deposit_cents: quote.deposit_cents,
        total_cents: quote.total_cents,
        currency: quote.currency.clone(),
    }
}

fn is_duplicate_key(error: &mongodb::error::Error) -> bool {
    match error.kind.as_ref() {
        ErrorKind::Write(WriteFailure::WriteError(error)) => error.code == 11_000,
        ErrorKind::InsertMany(error) => error
            .write_errors
            .as_ref()
            .is_some_and(|errors| errors.iter().any(|error| error.code == 11_000)),
        _ => false,
    }
}

fn offset_datetime(value: mongodb::bson::DateTime) -> Result<OffsetDateTime, TransactionError> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(value.timestamp_millis()) * 1_000_000)
        .map_err(|_| ReserveError::InvalidTransition.into())
}

fn transaction_error(error: TransactionError) -> ReserveError {
    match error {
        TransactionError::Domain(error) => error,
        TransactionError::Mongo(error) => ReserveError::database(error),
    }
}

#[derive(Debug)]
enum TransactionError {
    Domain(ReserveError),
    Mongo(mongodb::error::Error),
}

impl From<ReserveError> for TransactionError {
    fn from(value: ReserveError) -> Self {
        Self::Domain(value)
    }
}

impl From<mongodb::error::Error> for TransactionError {
    fn from(value: mongodb::error::Error) -> Self {
        Self::Mongo(value)
    }
}
