use crate::db::Db;
use crate::errors::Result;
use crate::models::domain::{Order, OrderStatus, OrderType, Side};
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

impl Db {
    /// Insert a new order into the database
    pub async fn create_order(&self, order: &Order) -> Result<()> {
        // For market orders, use price 1 in DB (actual price doesn't matter for market orders)
        let price_for_db = if order.order_type == OrderType::Market && order.price == 0 {
            1
        } else {
            order.price
        };

        let price_str = price_for_db.to_string();
        let size_str = order.size.to_string();
        let filled_size_str = order.filled_size.to_string();

        let side_str = match order.side {
            Side::Buy => "buy",
            Side::Sell => "sell",
        };

        let order_type_str = match order.order_type {
            OrderType::Limit => "limit",
            OrderType::Market => "market",
        };

        let status_str = match order.status {
            OrderStatus::Pending => "pending",
            OrderStatus::PartiallyFilled => "partially_filled",
            OrderStatus::Filled => "filled",
            OrderStatus::Cancelled => "cancelled",
        };

        sqlx::query(
            r#"
            INSERT INTO orders (id, user_address, market_id, price, size, side, type, status, filled_size, created_at, updated_at)
            VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6::side, $7::order_type, $8::order_status, $9::numeric, $10, $11)
            "#
        )
        .bind(order.id)
        .bind(&order.user_address)
        .bind(&order.market_id)
        .bind(price_str)
        .bind(size_str)
        .bind(side_str)
        .bind(order_type_str)
        .bind(status_str)
        .bind(filled_size_str)
        .bind(order.created_at)
        .bind(order.updated_at)
        .execute(&self.postgres)
        .await?;

        Ok(())
    }

    /// Update an order's filled size and status
    pub async fn update_order_fill(
        &self,
        order_id: Uuid,
        filled_size: u128,
        status: OrderStatus,
    ) -> Result<()> {
        let filled_size_str = filled_size.to_string();
        let status_str = match status {
            OrderStatus::Pending => "pending",
            OrderStatus::PartiallyFilled => "partially_filled",
            OrderStatus::Filled => "filled",
            OrderStatus::Cancelled => "cancelled",
        };

        sqlx::query(
            r#"
            UPDATE orders
            SET filled_size = $1::numeric, status = $2::order_status, updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(filled_size_str)
        .bind(status_str)
        .bind(Utc::now())
        .bind(order_id)
        .execute(&self.postgres)
        .await?;

        Ok(())
    }

    pub async fn get_order(&self, _order_id: &Uuid) -> Result<Order> {
        todo!()
    }

    pub async fn cancel_order(&self, _order_id: &Uuid) -> Result<()> {
        todo!()
    }

    pub async fn get_user_orders(
        &self,
        _user_address: &str,
        _market_id: Option<&str>,
        _status: Option<OrderStatus>,
        _limit: u32,
    ) -> Result<Vec<Order>> {
        // TODO: Implement user orders retrieval
        Ok(vec![])
    }
}
