use anyhow::Result;
use backend::models::domain::Market;
use exchange_sdk::ExchangeClient;
use tracing::{error, info, warn};

/// Fetch market configuration and auto-faucet initial funds for a bot
pub async fn fetch_market_and_faucet(
    client: &ExchangeClient,
    market_id: &str,
    user_address: &str,
) -> Result<Market> {
    // Fetch market configuration from backend
    let market = client.get_market(market_id).await?;

    info!(
        "Market: {} (base) / {} (quote)",
        market.base_ticker, market.quote_ticker
    );

    // Auto-faucet initial funds (large amounts for testing)
    info!("💰 Auto-fauceting initial funds for {}", user_address);
    let faucet_amount = "10000000000000000000000000"; // Large amount for testing

    for token in [&market.base_ticker, &market.quote_ticker] {
        match client
            .admin_faucet(
                user_address.to_string(),
                token.to_string(),
                faucet_amount.to_string(),
            )
            .await
        {
            Ok(_) => info!("  ✓ Fauceted {} for {}", token, user_address),
            Err(e) => {
                // Might already be funded, that's ok
                warn!("  ⚠ Faucet {} failed (may already be funded): {}", token, e);
            }
        }
    }

    Ok(market)
}

/// Auto-faucet funds if we detect insufficient balance error
/// Returns true if faucet was triggered
pub async fn auto_faucet_on_error(
    client: &ExchangeClient,
    user_address: &str,
    market: &Market,
    error_msg: &str,
) -> bool {
    // Check if error is about insufficient balance
    if error_msg.contains("Insufficient balance") || error_msg.contains("insufficient") {
        warn!("⚠ Insufficient balance detected, auto-fauceting...");

        // Try to parse which token from error message
        let tokens = if error_msg.contains(&market.base_ticker) {
            vec![&market.base_ticker]
        } else if error_msg.contains(&market.quote_ticker) {
            vec![&market.quote_ticker]
        } else {
            // Not sure which token, faucet both
            vec![&market.base_ticker, &market.quote_ticker]
        };

        let faucet_amount = "10000000000000000000000000";

        for token_name in tokens {
            match client
                .admin_faucet(
                    user_address.to_string(),
                    token_name.to_string(),
                    faucet_amount.to_string(),
                )
                .await
            {
                Ok(_) => {
                    info!("✓ Auto-fauceted {} for {}", token_name, user_address);
                    return true;
                }
                Err(e) => {
                    error!("Failed to auto-faucet {}: {}", token_name, e);
                }
            }
        }
    }

    false
}
