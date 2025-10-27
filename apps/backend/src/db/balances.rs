use crate::db::Db;
use crate::errors::Result;
use crate::models::domain::Balance;
use chrono::Utc;
use sqlx::Row;

impl Db {
    /// Get balance for a specific user and token
    pub async fn get_balance(&self, user_address: &str, token_ticker: &str) -> Result<Balance> {
        let row = sqlx::query(
            r#"
            SELECT user_address, token_ticker, amount, open_interest, updated_at
            FROM balances
            WHERE user_address = $1 AND token_ticker = $2
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .fetch_one(&self.postgres)
        .await?;

        let amount_str: String = row.get("amount");
        let open_interest_str: String = row.get("open_interest");

        Ok(Balance {
            user_address: row.get("user_address"),
            token_ticker: row.get("token_ticker"),
            amount: amount_str.parse().unwrap_or(0),
            open_interest: open_interest_str.parse().unwrap_or(0),
            updated_at: row.get("updated_at"),
        })
    }

    /// List all balances for a user
    pub async fn list_balances_by_user(&self, user_address: &str) -> Result<Vec<Balance>> {
        let rows = sqlx::query(
            r#"
            SELECT user_address, token_ticker, amount, open_interest, updated_at
            FROM balances
            WHERE user_address = $1
            "#,
        )
        .bind(user_address)
        .fetch_all(&self.postgres)
        .await?;

        let balances = rows
            .iter()
            .map(|row| {
                let amount_str: String = row.get("amount");
                let open_interest_str: String = row.get("open_interest");

                Balance {
                    user_address: row.get("user_address"),
                    token_ticker: row.get("token_ticker"),
                    amount: amount_str.parse().unwrap_or(0),
                    open_interest: open_interest_str.parse().unwrap_or(0),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect();

        Ok(balances)
    }

    /// Update or insert balance (upsert)
    pub async fn update_balance(
        &self,
        user_address: &str,
        token_ticker: &str,
        amount: u128,
    ) -> Result<Balance> {
        let amount_str = amount.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO balances (user_address, token_ticker, amount, open_interest, updated_at)
            VALUES ($1, $2, $3::numeric, 0, $4)
            ON CONFLICT (user_address, token_ticker)
            DO UPDATE SET
                amount = $3::numeric,
                updated_at = $4
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&amount_str)
        .bind(now)
        .execute(&self.postgres)
        .await?;

        Ok(Balance {
            user_address: user_address.to_string(),
            token_ticker: token_ticker.to_string(),
            amount,
            open_interest: 0,
            updated_at: now,
        })
    }

    /// Add to existing balance (for deposits/credits)
    pub async fn add_balance(
        &self,
        user_address: &str,
        token_ticker: &str,
        amount_delta: u128,
    ) -> Result<Balance> {
        let delta_str = amount_delta.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO balances (user_address, token_ticker, amount, open_interest, updated_at)
            VALUES ($1, $2, $3::numeric, 0, $4)
            ON CONFLICT (user_address, token_ticker)
            DO UPDATE SET
                amount = balances.amount + $3::numeric,
                updated_at = $4
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&delta_str)
        .bind(now)
        .execute(&self.postgres)
        .await?;

        self.get_balance(user_address, token_ticker).await
    }

    /// Subtract from existing balance (for withdrawals/debits)
    pub async fn subtract_balance(
        &self,
        user_address: &str,
        token_ticker: &str,
        amount_delta: u128,
    ) -> Result<Balance> {
        let delta_str = amount_delta.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE balances
            SET amount = amount - $3::numeric, updated_at = $4
            WHERE user_address = $1 AND token_ticker = $2
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&delta_str)
        .bind(now)
        .execute(&self.postgres)
        .await?;

        self.get_balance(user_address, token_ticker).await
    }
}
