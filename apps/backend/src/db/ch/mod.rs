use anyhow::Context;
use clickhouse::Client;
use std::env;

/// Create a ClickHouse client
pub async fn create_client() -> anyhow::Result<Client> {
    let clickhouse_url = env::var("CH_URL").context("CH_URL must be set in environment")?;

    let client = Client::default()
        .with_url(&clickhouse_url)
        .with_user("default")
        .with_password("password")
        .with_database("exchange");

    Ok(client)
}
