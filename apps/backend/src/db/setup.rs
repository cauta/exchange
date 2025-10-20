use std::process;

use backend::db::Db;

#[tokio::main]
async fn main() {
    // Load environment variables: .env.defaults first, then .env overrides
    dotenvy::from_filename(".env.defaults").ok();
    dotenvy::dotenv().ok();

    if let Err(e) = run_setup().await {
        eprintln!("Database setup failed: {}", e);
        process::exit(1);
    }

    println!("✅ Database setup complete!");
}

async fn run_setup() -> Result<(), Box<dyn std::error::Error>> {
    let db = Db::connect().await?;

    // postgres setup
    sqlx::migrate!("./src/db/pg/migrations")
        .run(&db.postgres)
        .await?;

    // clickhouse setup
    let schema_sql = include_str!("ch/schema.sql");
    db.clickhouse.query(schema_sql).execute().await?;

    Ok(())
}
