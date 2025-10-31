mod bots;
mod config;
mod hyperliquid;

use anyhow::{Context, Result};
use bots::{OrderbookMirrorBot, OrderbookMirrorConfig, TradeMirrorBot, TradeMirrorConfig};
use config::Config;
use exchange_sdk::ExchangeClient;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🤖 Starting Exchange Bots");

    // Load configuration
    let config = Config::load().context("Failed to load apps/bots/config.toml")?;

    // Exchange URL - can be overridden by env var
    let exchange_url =
        std::env::var("EXCHANGE_URL").unwrap_or_else(|_| config.exchange.url.clone());

    info!("📡 Exchange URL: {}", exchange_url);
    info!("👤 Maker address: {}", config.accounts.maker_address);
    info!("👤 Taker address: {}", config.accounts.taker_address);

    // Create exchange clients
    let maker_client = ExchangeClient::new(&exchange_url);
    let taker_client = ExchangeClient::new(&exchange_url);

    // Start bots in parallel
    let mut handles = vec![];

    // Start orderbook mirror bot if enabled
    if config.orderbook_mirror.enabled {
        let bot_config = OrderbookMirrorConfig {
            market_id: config.orderbook_mirror.market_id.clone(),
            user_address: config.accounts.maker_address.clone(),
            depth_levels: config.orderbook_mirror.depth_levels,
            update_interval_ms: config.orderbook_mirror.update_interval_ms,
        };

        info!(
            "📖 Initializing orderbook mirror bot for {}",
            bot_config.market_id
        );
        let mut orderbook_bot = OrderbookMirrorBot::new(bot_config, maker_client.clone())
            .await
            .context("Failed to initialize orderbook mirror bot")?;

        let handle = tokio::spawn(async move {
            if let Err(e) = orderbook_bot.start().await {
                tracing::error!("❌ Orderbook bot error: {}", e);
            }
        });
        handles.push(handle);
    }

    // Start trade mirror bot if enabled
    if config.trade_mirror.enabled {
        let bot_config = TradeMirrorConfig {
            market_id: config.trade_mirror.market_id.clone(),
            user_address: config.accounts.taker_address.clone(),
        };

        info!(
            "💱 Initializing trade mirror bot for {}",
            bot_config.market_id
        );
        let mut trade_bot = TradeMirrorBot::new(bot_config, taker_client.clone())
            .await
            .context("Failed to initialize trade mirror bot")?;

        let handle = tokio::spawn(async move {
            if let Err(e) = trade_bot.start().await {
                tracing::error!("❌ Trade bot error: {}", e);
            }
        });
        handles.push(handle);
    }

    if handles.is_empty() {
        info!("❌ No bots enabled in config.toml");
        return Ok(());
    }

    info!("✅ All enabled bots are running");

    // Wait for any bot to finish
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
