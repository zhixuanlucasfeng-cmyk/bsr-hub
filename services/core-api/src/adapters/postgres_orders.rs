use async_trait::async_trait;
use sqlx::{PgPool, Row};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    domain::{
        order_state::{OrderAction, OrderState},
        pricing::BillingUnit,
        quote::PricingSnapshot,
    },
    ports::order_repository::{CreateOrder, OrderRepository, ReserveError, ReservedOrder},
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
        ReserveError::Database(error)
    }
}

#[async_trait]
impl OrderRepository for PostgresOrderRepository {
    async fn pricing(&self, listing_id: Uuid) -> Result<PricingSnapshot, ReserveError> {
        let row = sqlx::query(
            "SELECT unit_price_cents, deposit_cents, delivery_fee_cents \
             FROM listings WHERE id = $1 AND status = 'active'",
        )
        .bind(listing_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_sql)?
        .ok_or(ReserveError::NotFound)?;
        Ok(PricingSnapshot {
            unit_price_cents: row.get("unit_price_cents"),
            deposit_cents: row.get("deposit_cents"),
            delivery_fee_cents: row.get("delivery_fee_cents"),
            service_fee_bps: self.service_fee_bps,
            billing_unit: BillingUnit::Day,
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

    async fn apply_payment_event(
        &self,
        event_id: &str,
        order_id: Option<Uuid>,
    ) -> Result<bool, ReserveError> {
        let mut transaction = self.pool.begin().await.map_err(Self::map_sql)?;
        let inserted =
            sqlx::query("INSERT INTO stripe_events (event_id) VALUES ($1) ON CONFLICT DO NOTHING")
                .bind(event_id)
                .execute(&mut *transaction)
                .await
                .map_err(Self::map_sql)?
                .rows_affected()
                == 1;
        if !inserted {
            transaction.commit().await.map_err(Self::map_sql)?;
            return Ok(false);
        }
        if let Some(order_id) = order_id {
            sqlx::query(
                "UPDATE orders SET status = 'paid', updated_at = now() \
                 WHERE id = $1 AND status = 'pending_payment'",
            )
            .bind(order_id)
            .execute(&mut *transaction)
            .await
            .map_err(Self::map_sql)?;
            sqlx::query(
                "INSERT INTO order_events (order_id, event_type) VALUES ($1, 'payment_succeeded')",
            )
            .bind(order_id)
            .execute(&mut *transaction)
            .await
            .map_err(Self::map_sql)?;
        }
        transaction.commit().await.map_err(Self::map_sql)?;
        Ok(true)
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
