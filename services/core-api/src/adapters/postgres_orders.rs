use async_trait::async_trait;
use sqlx::{PgPool, Row};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    domain::{
        order_state::{OrderAction, OrderState},
        pricing::{BillingUnit, PricingCategoryInput},
        quote::{FulfillmentMethod, PricingSnapshot},
    },
    ports::order_repository::{
        CreateOrder, OrderRepository, PaymentEventOutcome, PaymentValidation, ReserveError,
        ReservedOrder, SavePricingProfile, StoredOrderPayment, StoredPricingProfile,
        VerifiedPayment, validate_payment,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresOrderRepository {
    pool: PgPool,
    service_fee_bps: i64,
    reservation_minutes: i64,
}

impl PostgresOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            service_fee_bps: 600,
            reservation_minutes: 30,
        }
    }

    pub fn with_rules(pool: PgPool, service_fee_bps: i64, reservation_minutes: i64) -> Self {
        Self {
            pool,
            service_fee_bps,
            reservation_minutes,
        }
    }

    fn map_sql(error: sqlx::Error) -> ReserveError {
        if let sqlx::Error::Database(database) = &error
            && matches!(database.code().as_deref(), Some("23P01" | "23505"))
        {
            return ReserveError::Unavailable;
        }
        ReserveError::database(error)
    }
}

#[async_trait]
impl OrderRepository for PostgresOrderRepository {
    async fn pricing(&self, listing_id: Uuid) -> Result<PricingSnapshot, ReserveError> {
        let row = sqlx::query(
            "SELECT p.final_unit_price_cents AS unit_price_cents, p.billing_unit, \
                    p.allowed_fulfillment_methods, l.deposit_cents, l.delivery_fee_cents \
             FROM listings l \
             JOIN listing_pricing_profiles p ON p.listing_id = l.id \
             WHERE l.id = $1 AND l.status = 'active'",
        )
        .bind(listing_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_sql)?
        .ok_or(ReserveError::PricingNotFound)?;
        let billing_unit = BillingUnit::try_from(row.get::<&str, _>("billing_unit"))
            .map_err(|_| ReserveError::InvalidPricing)?;
        let methods: Vec<String> = row.get("allowed_fulfillment_methods");
        let allowed_fulfillment_methods = methods
            .iter()
            .map(|method| {
                FulfillmentMethod::try_from(method.as_str())
                    .map_err(|_| ReserveError::InvalidPricing)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PricingSnapshot {
            unit_price_cents: row.get("unit_price_cents"),
            deposit_cents: row.get("deposit_cents"),
            delivery_fee_cents: row.get("delivery_fee_cents"),
            service_fee_bps: self.service_fee_bps,
            billing_unit,
            allowed_fulfillment_methods,
        })
    }

    async fn reserve(&self, order: CreateOrder) -> Result<ReservedOrder, ReserveError> {
        if matches!((order.start_at, order.end_at), (Some(start), Some(end)) if start >= end) {
            return Err(ReserveError::Unavailable);
        }
        let mut transaction = self.pool.begin().await.map_err(Self::map_sql)?;
        let listing = sqlx::query(
            "SELECT owner_id FROM listings \
             WHERE id = $1 AND status = 'active' FOR UPDATE",
        )
        .bind(order.listing_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(Self::map_sql)?
        .ok_or(ReserveError::NotFound)?;
        let owner_id: Uuid = listing.get("owner_id");
        if owner_id == order.buyer_id {
            return Err(ReserveError::SelfTransaction);
        }

        sqlx::query(
            "UPDATE orders SET status = 'expired', updated_at = now() \
             WHERE listing_id = $1 AND status = 'pending_payment' \
             AND reservation_expires_at <= now()",
        )
        .bind(order.listing_id)
        .execute(&mut *transaction)
        .await
        .map_err(Self::map_sql)?;

        if let (Some(start), Some(end)) = (order.start_at, order.end_at) {
            let conflict: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM orders WHERE listing_id = $1 \
                 AND status IN ('pending_payment','paid','confirmed','active') \
                 AND tstzrange(start_at, end_at, '[)') && tstzrange($2, $3, '[)'))",
            )
            .bind(order.listing_id)
            .bind(start)
            .bind(end)
            .fetch_one(&mut *transaction)
            .await
            .map_err(Self::map_sql)?;
            if conflict {
                return Err(ReserveError::Unavailable);
            }
        }

        let order_id = Uuid::new_v4();
        let expires_at = OffsetDateTime::now_utc() + Duration::minutes(self.reservation_minutes);
        sqlx::query(
            "INSERT INTO orders \
             (id, listing_id, buyer_id, status, start_at, end_at, reservation_expires_at) \
             VALUES ($1, $2, $3, 'pending_payment', $4, $5, $6)",
        )
        .bind(order_id)
        .bind(order.listing_id)
        .bind(order.buyer_id)
        .bind(order.start_at)
        .bind(order.end_at)
        .bind(expires_at)
        .execute(&mut *transaction)
        .await
        .map_err(Self::map_sql)?;
        sqlx::query(
            "INSERT INTO order_amounts \
             (order_id, base_cents, service_fee_cents, delivery_fee_cents, deposit_cents, total_cents, currency) \
             VALUES ($1, $2, $3, $4, $5, $6, 'USD')",
        )
        .bind(order_id)
        .bind(order.quote.base_cents)
        .bind(order.quote.service_fee_cents)
        .bind(order.quote.delivery_fee_cents)
        .bind(order.quote.deposit_cents)
        .bind(order.quote.total_cents)
        .execute(&mut *transaction)
        .await
        .map_err(Self::map_sql)?;
        sqlx::query(
            "INSERT INTO order_events (order_id, event_type, actor_id) \
             VALUES ($1, 'reservation_created', $2)",
        )
        .bind(order_id)
        .bind(order.buyer_id)
        .execute(&mut *transaction)
        .await
        .map_err(Self::map_sql)?;
        transaction.commit().await.map_err(Self::map_sql)?;
        Ok(ReservedOrder {
            order_id,
            expires_at,
        })
    }

    async fn save_pricing_profile(
        &self,
        profile: SavePricingProfile,
    ) -> Result<StoredPricingProfile, ReserveError> {
        let mut transaction = self.pool.begin().await.map_err(Self::map_sql)?;
        let owner_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT owner_id FROM listings WHERE id = $1 AND status = 'active' FOR UPDATE",
        )
        .bind(profile.listing_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(Self::map_sql)?;
        let owner_id = owner_id.ok_or(ReserveError::NotFound)?;
        if owner_id != profile.owner_id {
            return Err(ReserveError::Forbidden);
        }

        let category = match &profile.attributes {
            PricingCategoryInput::Ps5(_) => "ps5",
            PricingCategoryInput::Workspace(_) => "workspace",
        };
        let attributes =
            serde_json::to_value(&profile.attributes).map_err(|_| ReserveError::InvalidPricing)?;
        let allowed_fulfillment_methods: Vec<&str> = profile
            .allowed_fulfillment_methods
            .iter()
            .map(|method| method.as_str())
            .collect();
        sqlx::query(
            "INSERT INTO listing_pricing_profiles (\
               listing_id, category, billing_unit, attributes, ruleset_version, \
               recommended_unit_price_cents, seller_adjustment_cents, \
               allowed_fulfillment_methods, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now()) \
             ON CONFLICT (listing_id) DO UPDATE SET \
               category = EXCLUDED.category, \
               billing_unit = EXCLUDED.billing_unit, \
               attributes = EXCLUDED.attributes, \
               ruleset_version = EXCLUDED.ruleset_version, \
               recommended_unit_price_cents = EXCLUDED.recommended_unit_price_cents, \
               seller_adjustment_cents = EXCLUDED.seller_adjustment_cents, \
               allowed_fulfillment_methods = EXCLUDED.allowed_fulfillment_methods, \
               updated_at = now()",
        )
        .bind(profile.listing_id)
        .bind(category)
        .bind(profile.recommendation.billing_unit.as_str())
        .bind(attributes)
        .bind(&profile.recommendation.ruleset_version)
        .bind(profile.recommendation.recommended_unit_price_cents)
        .bind(profile.seller_adjustment_cents)
        .bind(allowed_fulfillment_methods)
        .execute(&mut *transaction)
        .await
        .map_err(Self::map_sql)?;
        transaction.commit().await.map_err(Self::map_sql)?;

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

    async fn apply_payment_event(
        &self,
        payment: VerifiedPayment,
    ) -> Result<PaymentEventOutcome, ReserveError> {
        let mut transaction = self.pool.begin().await.map_err(Self::map_sql)?;
        let inserted =
            sqlx::query("INSERT INTO stripe_events (event_id) VALUES ($1) ON CONFLICT DO NOTHING")
                .bind(&payment.event_id)
                .execute(&mut *transaction)
                .await
                .map_err(Self::map_sql)?
                .rows_affected()
                == 1;
        if !inserted {
            transaction.commit().await.map_err(Self::map_sql)?;
            return Ok(PaymentEventOutcome::Duplicate);
        }

        let row = sqlx::query(
            "SELECT o.status, o.reservation_expires_at, a.total_cents, a.currency \
             FROM orders o JOIN order_amounts a ON a.order_id = o.id \
             WHERE o.id = $1 FOR UPDATE OF o",
        )
        .bind(payment.order_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(Self::map_sql)?;
        let Some(row) = row else {
            transaction.commit().await.map_err(Self::map_sql)?;
            return Ok(PaymentEventOutcome::MissingOrder);
        };
        let stored = StoredOrderPayment {
            status: row.get("status"),
            reservation_expires_at: row.get("reservation_expires_at"),
            total_cents: row.get("total_cents"),
            currency: row.get("currency"),
        };
        let validation = validate_payment(&payment, &stored, OffsetDateTime::now_utc());
        if validation == PaymentValidation::Accepted {
            sqlx::query(
                "UPDATE orders SET status = 'paid', updated_at = now() \
                 WHERE id = $1 AND status = 'pending_payment'",
            )
            .bind(payment.order_id)
            .execute(&mut *transaction)
            .await
            .map_err(Self::map_sql)?;
            sqlx::query(
                "INSERT INTO order_events (order_id, event_type) VALUES ($1, 'payment_succeeded')",
            )
            .bind(payment.order_id)
            .execute(&mut *transaction)
            .await
            .map_err(Self::map_sql)?;
            transaction.commit().await.map_err(Self::map_sql)?;
            return Ok(PaymentEventOutcome::Applied);
        }

        if validation == PaymentValidation::Expired {
            sqlx::query(
                "UPDATE orders SET status = 'expired', updated_at = now() \
                 WHERE id = $1 AND status = 'pending_payment'",
            )
            .bind(payment.order_id)
            .execute(&mut *transaction)
            .await
            .map_err(Self::map_sql)?;
        }
        let event_type = match validation {
            PaymentValidation::Accepted => unreachable!(),
            PaymentValidation::WrongState => "payment_rejected_wrong_state",
            PaymentValidation::Expired => "payment_rejected_expired",
            PaymentValidation::AmountMismatch => "payment_rejected_amount_mismatch",
            PaymentValidation::CurrencyMismatch => "payment_rejected_currency_mismatch",
        };
        sqlx::query("INSERT INTO order_events (order_id, event_type) VALUES ($1, $2)")
            .bind(payment.order_id)
            .bind(event_type)
            .execute(&mut *transaction)
            .await
            .map_err(Self::map_sql)?;
        transaction.commit().await.map_err(Self::map_sql)?;
        Ok(PaymentEventOutcome::Rejected(validation))
    }

    async fn transition(
        &self,
        order_id: Uuid,
        actor_id: Uuid,
        action: OrderAction,
    ) -> Result<OrderState, ReserveError> {
        let mut transaction = self.pool.begin().await.map_err(Self::map_sql)?;
        let row = sqlx::query(
            "SELECT o.status, o.buyer_id, l.owner_id FROM orders o \
             JOIN listings l ON l.id = o.listing_id WHERE o.id = $1 FOR UPDATE",
        )
        .bind(order_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(Self::map_sql)?
        .ok_or(ReserveError::NotFound)?;
        let current = OrderState::try_from(row.get::<&str, _>("status"))
            .map_err(|_| ReserveError::InvalidTransition)?;
        let buyer_id: Uuid = row.get("buyer_id");
        let owner_id: Uuid = row.get("owner_id");
        let authorized = match action {
            OrderAction::Confirm | OrderAction::Fulfill => actor_id == owner_id,
            OrderAction::Activate | OrderAction::Return => actor_id == buyer_id,
            OrderAction::Complete | OrderAction::Cancel => {
                actor_id == owner_id || actor_id == buyer_id
            }
            OrderAction::MarkPaid | OrderAction::Expire => false,
        };
        if !authorized {
            return Err(ReserveError::Forbidden);
        }
        let next = current
            .transition(action)
            .map_err(|_| ReserveError::InvalidTransition)?;
        sqlx::query("UPDATE orders SET status = $2, updated_at = now() WHERE id = $1")
            .bind(order_id)
            .bind(next.as_str())
            .execute(&mut *transaction)
            .await
            .map_err(Self::map_sql)?;
        sqlx::query(
            "INSERT INTO order_events (order_id, event_type, actor_id) VALUES ($1, $2, $3)",
        )
        .bind(order_id)
        .bind(format!("state_changed_to_{}", next.as_str()))
        .bind(actor_id)
        .execute(&mut *transaction)
        .await
        .map_err(Self::map_sql)?;
        transaction.commit().await.map_err(Self::map_sql)?;
        Ok(next)
    }
}
