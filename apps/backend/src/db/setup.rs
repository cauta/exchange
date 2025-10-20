use std::process;

use backend::db::Db;

#[tokio::main]
async fn main() {
    // Load environment variables: .env.defaults first, then .env overrides
    dotenvy::from_filename(".env.defaults").ok();
    dotenvy::dotenv().ok();

    env_logger::init();

    if let Err(e) = run_setup().await {
        eprintln!("Database setup failed: {}", e);
        process::exit(1);
    }

    println!("✅ Database setup complete!");
}

async fn run_setup() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to both databases
    let db = Db::connect().await?;

    // Run setup for each database
    setup_postgres(&db).await?;
    setup_clickhouse(&db).await?;

    Ok(())
}

async fn setup_postgres(db: &Db) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Setting up Postgres...");

    // Run migrations
    sqlx::migrate!("./src/db/pg/migrations")
        .run(&db.postgres)
        .await?;

    println!("✅ Postgres setup complete");
    Ok(())
}

async fn setup_clickhouse(db: &Db) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Setting up ClickHouse...");

    // Create database if not exists
    db.clickhouse
        .query("CREATE DATABASE IF NOT EXISTS exchange")
        .execute()
        .await?;

    // Run schema
    let schema_sql = include_str!("ch/schema.sql");
    db.clickhouse.query(schema_sql).execute().await?;

    println!("✅ ClickHouse setup complete");
    Ok(())
}
