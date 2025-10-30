use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Backend configuration (from apps/backend/config.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub markets: Vec<MarketConfig>,
    pub tokens: Vec<TokenConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfig {
    pub base_ticker: String,
    pub quote_ticker: String,
    pub tick_size: String,
    pub lot_size: String,
    pub min_size: String,
    pub maker_fee_bps: i32,
    pub taker_fee_bps: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub ticker: String,
    pub decimals: u8,
    pub name: String,
}

impl Config {
    /// Load backend configuration from config.toml
    /// Uses CARGO_MANIFEST_DIR so the path is consistent regardless of where the binary is run from
    pub fn load() -> Result<Self> {
        let config_path = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
        let contents = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
