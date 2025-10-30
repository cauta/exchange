use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::str::FromStr;

/// Test the bot's price conversion logic
#[cfg(test)]
mod price_conversion_tests {
    use super::*;

    fn convert_price(price: Decimal) -> String {
        // This mirrors the bot's convert_price() logic
        let scaled = price * Decimal::from(1_000_000);
        scaled.to_u128().unwrap_or(0).to_string()
    }

    #[test]
    fn test_normal_btc_price() {
        // BTC price around $95,000
        let price = Decimal::from_str("95000.0").unwrap();
        let result = convert_price(price);
        assert_eq!(result, "95000000000"); // 95000 * 1_000_000
    }

    #[test]
    fn test_fractional_price() {
        // Price with decimals
        let price = Decimal::from_str("95123.456789").unwrap();
        let result = convert_price(price);
        // Should preserve precision: 95123.456789 * 1_000_000 = 95123456789
        assert_eq!(result, "95123456789");
    }

    #[test]
    fn test_very_small_price() {
        // Very small price (like a low-value token)
        let price = Decimal::from_str("0.000001").unwrap();
        let result = convert_price(price);
        assert_eq!(result, "1"); // 0.000001 * 1_000_000 = 1
    }

    #[test]
    fn test_tiny_price_rounds_to_zero() {
        // Price smaller than minimum precision
        let price = Decimal::from_str("0.0000001").unwrap();
        let result = convert_price(price);
        // BUG: This rounds to 0 instead of erroring!
        assert_eq!(result, "0");
    }

    #[test]
    fn test_very_large_price() {
        // Very large price that might overflow
        let price = Decimal::from_str("999999999999999.0").unwrap();
        let result = convert_price(price);
        // Should handle large values
        assert_eq!(result, "999999999999999000000");
    }

    #[test]
    fn test_very_large_price_edge_case() {
        // Prices at the edge of what Decimal can represent
        // Decimal max is 79,228,162,514,264,337,593,543,950,335
        let price = Decimal::from_str("79228162514264337593.0").unwrap();
        let result = convert_price(price);
        // This works but is at the edge of valid range
        assert!(!result.is_empty());
    }
}

/// Test the bot's size conversion logic
#[cfg(test)]
mod size_conversion_tests {
    use super::*;

    fn convert_size(size: Decimal, size_multiplier: Decimal) -> String {
        // This mirrors the bot's convert_size() logic
        let adjusted = size * size_multiplier;
        let scaled = adjusted * Decimal::from(1_000_000);
        scaled.to_u128().unwrap_or(0).to_string()
    }

    #[test]
    fn test_normal_btc_size() {
        // Normal BTC order size: 0.1 BTC
        let size = Decimal::from_str("0.1").unwrap();
        let multiplier = Decimal::from_str("1.0").unwrap();
        let result = convert_size(size, multiplier);
        assert_eq!(result, "100000"); // 0.1 * 1_000_000
    }

    #[test]
    fn test_size_with_multiplier() {
        // 0.1 BTC with 10% multiplier = 0.01 BTC
        let size = Decimal::from_str("0.1").unwrap();
        let multiplier = Decimal::from_str("0.1").unwrap();
        let result = convert_size(size, multiplier);
        assert_eq!(result, "10000"); // 0.01 * 1_000_000
    }

    #[test]
    fn test_size_below_minimum() {
        // Size that becomes smaller than min_size after multiplier
        // 0.005 BTC * 0.1 = 0.0005 BTC < min_size (0.001 BTC)
        let size = Decimal::from_str("0.005").unwrap();
        let multiplier = Decimal::from_str("0.1").unwrap();
        let result = convert_size(size, multiplier);
        assert_eq!(result, "500"); // 0.0005 * 1_000_000

        // This order would be rejected because 500 < min_size (1000)
        let min_size = 1000;
        let result_val: u128 = result.parse().unwrap();
        assert!(result_val < min_size, "Order size {} is below min_size {}", result_val, min_size);
    }

    #[test]
    fn test_very_small_size_rounds_to_zero() {
        // Size that rounds to zero after multiplier
        let size = Decimal::from_str("0.0000001").unwrap();
        let multiplier = Decimal::from_str("0.1").unwrap();
        let result = convert_size(size, multiplier);
        // BUG: This rounds to 0 instead of erroring!
        assert_eq!(result, "0");
    }

    #[test]
    fn test_fractional_precision() {
        // Test that we preserve precision correctly
        let size = Decimal::from_str("0.123456").unwrap();
        let multiplier = Decimal::from_str("1.0").unwrap();
        let result = convert_size(size, multiplier);
        assert_eq!(result, "123456"); // 0.123456 * 1_000_000
    }

    #[test]
    fn test_size_with_small_multiplier() {
        // Small multiplier that could cause precision issues
        let size = Decimal::from_str("1.0").unwrap();
        let multiplier = Decimal::from_str("0.001").unwrap();
        let result = convert_size(size, multiplier);
        assert_eq!(result, "1000"); // 0.001 * 1_000_000 = 1000
    }

    #[test]
    #[should_panic(expected = "overflowed")]
    fn test_size_overflow_panics() {
        // Size that would overflow - this panics instead of returning 0!
        let size = Decimal::from_str("999999999999999999999999.0").unwrap();
        let multiplier = Decimal::from_str("1.0").unwrap();
        let _result = convert_size(size, multiplier);
        // The test shows that Decimal multiplication panics on overflow
    }
}

/// Test market constraint validation
#[cfg(test)]
mod market_constraint_tests {
    use super::*;

    /// Check if a size respects lot_size and min_size
    fn validate_size(size_str: &str, lot_size: u128, min_size: u128) -> Result<(), String> {
        let size: u128 = size_str.parse().map_err(|e| format!("Parse error: {}", e))?;

        if size < min_size {
            return Err(format!("Size {} is below minimum {}", size, min_size));
        }

        if size % lot_size != 0 {
            return Err(format!("Size {} is not a multiple of lot_size {}", size, lot_size));
        }

        Ok(())
    }

    #[test]
    fn test_btc_market_constraints() {
        // BTC/USDC market from config.toml:
        // lot_size = 1000 (0.001 BTC)
        // min_size = 1000 (0.001 BTC)
        let lot_size = 1000;
        let min_size = 1000;

        // Valid order
        assert!(validate_size("1000", lot_size, min_size).is_ok());
        assert!(validate_size("2000", lot_size, min_size).is_ok());
        assert!(validate_size("100000", lot_size, min_size).is_ok());

        // Invalid: below min_size
        assert!(validate_size("500", lot_size, min_size).is_err());
        assert!(validate_size("999", lot_size, min_size).is_err());

        // Invalid: not multiple of lot_size
        assert!(validate_size("1500", lot_size, min_size).is_err());
        assert!(validate_size("2001", lot_size, min_size).is_err());
    }

    #[test]
    fn test_price_respects_tick_size() {
        // BTC/USDC tick_size = 1000000 (1 USDC)
        let tick_size = 1000000;

        let validate_price = |price_str: &str, tick_size: u128| -> bool {
            let price: u128 = price_str.parse().unwrap_or(0);
            price % tick_size == 0
        };

        // Valid prices (multiples of 1 USDC)
        assert!(validate_price("95000000000", tick_size)); // $95,000
        assert!(validate_price("95001000000", tick_size)); // $95,001

        // Invalid prices (not multiples of tick_size)
        assert!(!validate_price("95000500000", tick_size)); // $95,000.50
        assert!(!validate_price("95000000001", tick_size)); // $95,000.000001
    }
}

/// Test end-to-end scenarios that bots encounter
#[cfg(test)]
mod integration_scenarios {
    use rust_decimal::Decimal;
    use rust_decimal::prelude::ToPrimitive;
    use std::str::FromStr;

    fn convert_size(size: Decimal, size_multiplier: Decimal) -> String {
        let adjusted = size * size_multiplier;
        let scaled = adjusted * Decimal::from(1_000_000);
        scaled.to_u128().unwrap_or(0).to_string()
    }

    #[test]
    fn test_hyperliquid_trade_mirror_scenario() {
        // Scenario: Hyperliquid trade of 0.05 BTC at $95,123.45
        // Bot config: size_multiplier = 0.1 (10% of size)
        // Expected: 0.005 BTC order on our exchange

        let hl_size = Decimal::from_str("0.05").unwrap();
        let multiplier = Decimal::from_str("0.1").unwrap();
        let expected_size = Decimal::from_str("0.005").unwrap();

        let result = convert_size(hl_size, multiplier);
        let expected = (expected_size * Decimal::from(1_000_000)).to_u128().unwrap().to_string();

        assert_eq!(result, expected);
        assert_eq!(result, "5000"); // 0.005 * 1_000_000
    }

    #[test]
    fn test_small_trade_becomes_invalid() {
        // Scenario: Very small Hyperliquid trade that becomes too small
        // after size_multiplier
        let hl_size = Decimal::from_str("0.005").unwrap(); // 0.005 BTC
        let multiplier = Decimal::from_str("0.05").unwrap(); // 5%
        // Result: 0.00025 BTC which is below min_size (0.001 BTC = 1000)

        let result = convert_size(hl_size, multiplier);
        assert_eq!(result, "250"); // 0.00025 * 1_000_000

        let min_size = 1000;
        let result_val: u128 = result.parse().unwrap();
        assert!(result_val < min_size,
            "This order would be rejected: size {} < min_size {}",
            result_val, min_size);
    }

    #[test]
    fn test_orderbook_level_with_rounding() {
        // Scenario: Orderbook level with price that needs rounding
        let price = Decimal::from_str("95123.789").unwrap(); // $95,123.789

        // Convert price (should round to tick_size)
        let price_scaled = price * Decimal::from(1_000_000);
        let price_result = price_scaled.to_u128().unwrap();
        assert_eq!(price_result, 95123789000);

        // This price (95123789000) is NOT a multiple of tick_size (1000000)
        // It would need to be rounded to 95123000000 or 95124000000
        let tick_size = 1000000;
        assert_ne!(price_result % tick_size, 0,
            "Price needs rounding to respect tick_size");
    }
}
