use sqlx::PgPool;

use crate::db::Db;
use crate::errors::Result;
use crate::models::{db::BalanceRow, domain::Balance};

impl Db {
    pub async fn get_balance(&self, user_address: &str, token_ticker: &str) -> Result<Balance> {
        todo!()
    }

    pub async fn list_balances_by_user(&self, user_address: &str) -> Result<Vec<Balance>> {
        todo!()
    }

    pub async fn update_balance(
        &self,
        user_address: &str,
        token_ticker: &str,
        amount: u128,
    ) -> Result<Balance> {
        todo!()
    }
}
