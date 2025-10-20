use clickhouse::Client;
use std::env;

/// Create a ClickHouse client
pub async fn create_client() -> Result<Client, clickhouse::error::Error> {
    let clickhouse_url = env::var("CH_URL").expect("CH_URL must be set in environment");

    Ok(Client::default()
        .with_url(&clickhouse_url)
        .with_user("default")
        .with_password("password")
        .with_database("exchange"))
}
