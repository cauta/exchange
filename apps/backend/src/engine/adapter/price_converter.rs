//! Price and size conversion between domain u128 atomic units and OrderBook-rs Decimal types

use rust_decimal::Decimal;

/// Handles conversion between u128 atomic units and Decimal for OrderBook-rs
///
/// Domain types use u128 for prices/sizes in atomic units (e.g., 1.50 USD = 1_500_000 with 6 decimals).
/// OrderBook-rs uses Decimal for high-precision arithmetic.
#[derive(Debug, Clone)]
pub struct PriceConverter {
    base_decimals: u8,
    quote_decimals: u8,
    base_multiplier: Decimal,
    quote_multiplier: Decimal,
}

impl PriceConverter {
    /// Create a new price converter with the given decimal configurations
    ///
    /// # Arguments
    /// * `base_decimals` - Number of decimal places for the base token (e.g., 8 for BTC)
    /// * `quote_decimals` - Number of decimal places for the quote token (e.g., 6 for USDC)
    pub fn new(base_decimals: u8, quote_decimals: u8) -> Self {
        let base_multiplier = Decimal::from(10u64.pow(base_decimals as u32));
        let quote_multiplier = Decimal::from(10u64.pow(quote_decimals as u32));

        Self {
            base_decimals,
            quote_decimals,
            base_multiplier,
            quote_multiplier,
        }
    }

    /// Convert a u128 atomic price to OrderBook-rs Decimal
    ///
    /// # Example
    /// With quote_decimals=6: 1_500_000 -> Decimal(1.5)
    #[inline]
    pub fn to_orderbook_price(&self, atomic_price: u128) -> Decimal {
        Decimal::from(atomic_price) / self.quote_multiplier
    }

    /// Convert an OrderBook-rs Decimal price back to u128 atomic units
    ///
    /// # Example
    /// With quote_decimals=6: Decimal(1.5) -> 1_500_000
    #[inline]
    pub fn from_orderbook_price(&self, decimal_price: Decimal) -> u128 {
        let result = decimal_price * self.quote_multiplier;
        // Round to nearest integer to handle any floating point imprecision
        result
            .round()
            .to_string()
            .parse::<u128>()
            .expect("price conversion overflow")
    }

    /// Convert a u128 atomic size to OrderBook-rs Decimal
    ///
    /// # Example
    /// With base_decimals=8: 100_000_000 -> Decimal(1.0)
    #[inline]
    pub fn to_orderbook_size(&self, atomic_size: u128) -> Decimal {
        Decimal::from(atomic_size) / self.base_multiplier
    }

    /// Convert an OrderBook-rs Decimal size back to u128 atomic units
    ///
    /// # Example
    /// With base_decimals=8: Decimal(1.0) -> 100_000_000
    #[inline]
    pub fn from_orderbook_size(&self, decimal_size: Decimal) -> u128 {
        let result = decimal_size * self.base_multiplier;
        result
            .round()
            .to_string()
            .parse::<u128>()
            .expect("size conversion overflow")
    }

    /// Get the base token decimals
    pub fn base_decimals(&self) -> u8 {
        self.base_decimals
    }

    /// Get the quote token decimals
    pub fn quote_decimals(&self) -> u8 {
        self.quote_decimals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_roundtrip() {
        let converter = PriceConverter::new(18, 6);

        // Test common price: 1.50 USD
        let original: u128 = 1_500_000;
        let decimal = converter.to_orderbook_price(original);
        let back = converter.from_orderbook_price(decimal);
        assert_eq!(original, back);

        // Test whole number: 100 USD
        let original: u128 = 100_000_000;
        let decimal = converter.to_orderbook_price(original);
        let back = converter.from_orderbook_price(decimal);
        assert_eq!(original, back);

        // Test small price: 0.000001 USD
        let original: u128 = 1;
        let decimal = converter.to_orderbook_price(original);
        let back = converter.from_orderbook_price(decimal);
        assert_eq!(original, back);
    }

    #[test]
    fn test_size_roundtrip() {
        let converter = PriceConverter::new(8, 6);

        // Test 1 BTC
        let original: u128 = 100_000_000;
        let decimal = converter.to_orderbook_size(original);
        let back = converter.from_orderbook_size(decimal);
        assert_eq!(original, back);

        // Test 0.00000001 BTC (1 satoshi)
        let original: u128 = 1;
        let decimal = converter.to_orderbook_size(original);
        let back = converter.from_orderbook_size(decimal);
        assert_eq!(original, back);
    }

    #[test]
    fn test_large_values() {
        let converter = PriceConverter::new(18, 6);

        // Test large price: 1 billion USD
        let original: u128 = 1_000_000_000_000_000; // 1B with 6 decimals
        let decimal = converter.to_orderbook_price(original);
        let back = converter.from_orderbook_price(decimal);
        assert_eq!(original, back);
    }

    #[test]
    fn test_decimal_values() {
        let converter = PriceConverter::new(8, 6);

        // Verify decimal representation is correct
        let atomic_price: u128 = 50_000_000_000; // $50,000.00 with 6 decimals
        let decimal = converter.to_orderbook_price(atomic_price);
        assert_eq!(decimal, Decimal::from(50_000));
    }
}
