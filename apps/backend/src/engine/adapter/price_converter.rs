//! Price and size conversion between domain u128 atomic units and OrderBook-rs u64 types
//!
//! Uses tick_size and lot_size from market configuration to scale values.
//! This approach ensures:
//! - Zero precision loss (orders must be tick/lot multiples)
//! - Overflow prevention in orderbook-rs statistics
//! - Business-meaningful units (ticks and lots)

/// Handles conversion between u128 atomic units and u64 for OrderBook-rs
///
/// Domain types use u128 for prices/sizes in atomic units.
/// OrderBook-rs uses u64 internally for prices and quantities.
///
/// By dividing by tick_size/lot_size, we convert to "ticks" and "lots"
/// which are the minimum tradeable units, ensuring the conversion is lossless.
#[derive(Debug, Clone)]
pub struct PriceConverter {
    tick_size: u128,
    lot_size: u128,
}

impl PriceConverter {
    /// Create a new price converter with the given market configuration
    ///
    /// # Arguments
    /// * `tick_size` - Minimum price increment in atomic units
    /// * `lot_size` - Minimum size increment in atomic units
    pub fn new(tick_size: u128, lot_size: u128) -> Self {
        debug_assert!(tick_size > 0, "tick_size must be positive");
        debug_assert!(lot_size > 0, "lot_size must be positive");
        Self { tick_size, lot_size }
    }

    /// Convert price (atomic units) → ticks for orderbook-rs
    ///
    /// # Example
    /// With tick_size = 1_000_000 (0.01 USDC with 8 decimals):
    /// - price 50_000_000_000 ($500) → 50_000 ticks
    pub fn price_to_ticks(&self, price: u128) -> u64 {
        (price / self.tick_size) as u64
    }

    /// Convert ticks → price (atomic units)
    ///
    /// Used when converting orderbook-rs statistics back to domain units.
    pub fn ticks_to_price(&self, ticks: u64) -> u128 {
        (ticks as u128) * self.tick_size
    }

    /// Convert size (atomic units) → lots for orderbook-rs
    ///
    /// # Example
    /// With lot_size = 10_000 (0.0001 BTC with 8 decimals):
    /// - size 100_000_000 (1 BTC) → 10_000 lots
    pub fn size_to_lots(&self, size: u128) -> u64 {
        (size / self.lot_size) as u64
    }

    /// Convert lots → size (atomic units)
    ///
    /// Used when converting orderbook-rs statistics back to domain units.
    pub fn lots_to_size(&self, lots: u64) -> u128 {
        (lots as u128) * self.lot_size
    }

    /// Get the tick size
    pub fn tick_size(&self) -> u128 {
        self.tick_size
    }

    /// Get the lot size
    pub fn lot_size(&self) -> u128 {
        self.lot_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // BTC/USDC market config (from config.toml):
    // tick_size = 1_000_000 (0.01 USDC with 8 decimals)
    // lot_size = 10_000 (0.0001 BTC with 8 decimals)
    const BTC_TICK_SIZE: u128 = 1_000_000;
    const BTC_LOT_SIZE: u128 = 10_000;

    #[test]
    fn test_price_to_ticks() {
        let converter = PriceConverter::new(BTC_TICK_SIZE, BTC_LOT_SIZE);

        // $500.00 = 50_000_000_000 atomic units (8 decimals)
        // 50_000_000_000 / 1_000_000 = 50_000 ticks
        assert_eq!(converter.price_to_ticks(50_000_000_000), 50_000);

        // $0.01 = 1_000_000 atomic units = 1 tick (minimum)
        assert_eq!(converter.price_to_ticks(1_000_000), 1);

        // $100,000 = 10_000_000_000_000 atomic units = 10_000_000 ticks
        assert_eq!(converter.price_to_ticks(10_000_000_000_000), 10_000_000);
    }

    #[test]
    fn test_ticks_to_price() {
        let converter = PriceConverter::new(BTC_TICK_SIZE, BTC_LOT_SIZE);

        // 50_000 ticks = $500.00 = 50_000_000_000 atomic units
        assert_eq!(converter.ticks_to_price(50_000), 50_000_000_000);

        // 1 tick = $0.01 = 1_000_000 atomic units
        assert_eq!(converter.ticks_to_price(1), 1_000_000);
    }

    #[test]
    fn test_size_to_lots() {
        let converter = PriceConverter::new(BTC_TICK_SIZE, BTC_LOT_SIZE);

        // 1 BTC = 100_000_000 atomic units (8 decimals)
        // 100_000_000 / 10_000 = 10_000 lots
        assert_eq!(converter.size_to_lots(100_000_000), 10_000);

        // 0.0001 BTC = 10_000 atomic units = 1 lot (minimum)
        assert_eq!(converter.size_to_lots(10_000), 1);

        // 100 BTC = 10_000_000_000 atomic units = 1_000_000 lots
        assert_eq!(converter.size_to_lots(10_000_000_000), 1_000_000);
    }

    #[test]
    fn test_lots_to_size() {
        let converter = PriceConverter::new(BTC_TICK_SIZE, BTC_LOT_SIZE);

        // 10_000 lots = 1 BTC = 100_000_000 atomic units
        assert_eq!(converter.lots_to_size(10_000), 100_000_000);

        // 1 lot = 0.0001 BTC = 10_000 atomic units
        assert_eq!(converter.lots_to_size(1), 10_000);
    }

    #[test]
    fn test_roundtrip_conversions() {
        let converter = PriceConverter::new(BTC_TICK_SIZE, BTC_LOT_SIZE);

        // Price roundtrip (must be multiple of tick_size)
        let original_price: u128 = 50_000_000_000; // $500
        let ticks = converter.price_to_ticks(original_price);
        let back = converter.ticks_to_price(ticks);
        assert_eq!(original_price, back);

        // Size roundtrip (must be multiple of lot_size)
        let original_size: u128 = 100_000_000; // 1 BTC
        let lots = converter.size_to_lots(original_size);
        let back = converter.lots_to_size(lots);
        assert_eq!(original_size, back);
    }

    #[test]
    fn test_overflow_safety() {
        let converter = PriceConverter::new(BTC_TICK_SIZE, BTC_LOT_SIZE);

        // Extreme values that would overflow without scaling:
        // Price: $100,000 = 10^13 atomic units
        // Size: 100 BTC = 10^10 atomic units
        // Direct product: 10^23 >> u64::MAX
        //
        // With tick/lot scaling:
        // Price in ticks: 10^13 / 10^6 = 10^7
        // Size in lots: 10^10 / 10^4 = 10^6
        // Product: 10^13 << u64::MAX (1.8 * 10^19)

        let price = 10_000_000_000_000u128; // $100,000
        let size = 10_000_000_000u128; // 100 BTC

        let ticks = converter.price_to_ticks(price);
        let lots = converter.size_to_lots(size);

        // Verify the scaled values are safe for multiplication
        let product = (ticks as u128) * (lots as u128);
        assert!(product < u64::MAX as u128);

        // Verify actual values
        assert_eq!(ticks, 10_000_000); // 10^7
        assert_eq!(lots, 1_000_000); // 10^6
    }

    #[test]
    fn test_bp_usdc_market() {
        // BP/USDC market config (from config.toml):
        // tick_size = 100_000 (0.001 USDC with 8 decimals)
        // lot_size = 1_000_000 (1 BP with 8 decimals - minimum 1 BP)
        let converter = PriceConverter::new(100_000, 1_000_000);

        // Price $0.50 = 50_000_000 atomic units = 500 ticks
        assert_eq!(converter.price_to_ticks(50_000_000), 500);

        // Size 100 BP = 10_000_000_000 atomic units = 10_000 lots
        assert_eq!(converter.size_to_lots(10_000_000_000), 10_000);
    }
}
