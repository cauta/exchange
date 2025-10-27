use crate::db::Db;
use crate::errors::Result;
use crate::models::domain::Trade;

impl Db {
    /// Insert a new trade into the database
    pub async fn create_trade(&self, trade: &Trade) -> Result<()> {
        let price_str = trade.price.to_string();
        let size_str = trade.size.to_string();

        sqlx::query(
            r#"
            INSERT INTO trades (id, market_id, buyer_address, seller_address, buyer_order_id, seller_order_id, price, size, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8::numeric, $9)
            "#
        )
        .bind(trade.id)
        .bind(&trade.market_id)
        .bind(&trade.buyer_address)
        .bind(&trade.seller_address)
        .bind(trade.buyer_order_id)
        .bind(trade.seller_order_id)
        .bind(price_str)
        .bind(size_str)
        .bind(trade.timestamp)
        .execute(&self.postgres)
        .await?;

        Ok(())
    }

    pub async fn get_user_trades(
        &self,
        _user_address: &str,
        _market_id: Option<&str>,
        _limit: u32,
    ) -> Result<Vec<Trade>> {
        // TODO: Implement user trades retrieval
        Ok(vec![])
    }
}
