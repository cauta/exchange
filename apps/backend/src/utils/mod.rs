use bigdecimal::{BigDecimal, ToPrimitive};
use bigdecimal::num_bigint::ToBigInt;

pub trait BigDecimalExt {
    fn to_u128(self) -> u128;
}

impl BigDecimalExt for BigDecimal {
    fn to_u128(self) -> u128 {
        // Convert to BigInt - this will panic if there's a fractional part,
        // which is correct since we should NEVER have fractions (everything is in atoms)
        let bigint = self
            .to_bigint()
            .expect("BUG: BigDecimal has fractional part - all values should be in atoms");

        // Convert BigInt to u128
        bigint
            .to_u128()
            .expect("BUG: Value out of u128 range")
    }
}
