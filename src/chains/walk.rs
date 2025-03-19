/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/3/25
******************************************************************************/
use crate::chains::chain::OptionChain;
use crate::simulation::types::Walktypable;
use crate::simulation::{RandomWalkGraph, Walkable};
use crate::{Positive, pos};
use rust_decimal::Decimal;
use std::error::Error;

impl Walkable<Positive, OptionChain> for OptionChain {
    fn get_y_values(&self) -> &Vec<OptionChain> {
        todo!()
    }

    fn get_y_values_ref(&mut self) -> &mut Vec<OptionChain> {
        todo!()
    }

    fn get_random_walk(&self) -> Result<RandomWalkGraph<OptionChain>, Box<dyn Error>> {
        todo!()
    }
}

/// Trait for types that can be "walked" or iterated through, typically used for
/// option chains to simulate price movements and rebalance strikes.
impl Walktypable for OptionChain {
    /// Calculates the next state of the option chain based on an exponential change
    /// in the underlying asset's price.
    ///
    /// This function takes an exponential factor `exp` and calculates the new underlying
    /// price by multiplying the current underlying price by `f64::exp(exp)`. It then
    /// recalculates option prices based on the new underlying price, rebalancing
    /// the strike prices if necessary to maintain a balanced distribution of strikes
    /// around the underlying price.
    ///
    /// # Arguments
    ///
    /// * `exp` - A `f64` representing the exponential factor by which the underlying
    ///   price should change.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn Error>>` - A new `OptionChain` with updated option prices
    ///   and potentially rebalanced strikes, or an error if any calculation fails.
    fn walk_next(&self, exp: f64) -> Result<Self, Box<dyn Error>> {
        let mut build_params = self.to_build_params()?;
        build_params.skew_factor = 0.000001;
        build_params.price_params.underlying_price =
            pos!(self.underlying_price.to_f64() * f64::exp(exp)).max(Positive::ZERO);

        let chain = OptionChain::build_chain(&build_params);
        // TODO: Expiration Date: should be ingressed  to reduce in each iteration
        Ok(chain)
    }

    fn walk_dec(&self) -> Result<Decimal, Box<dyn Error>> {
        Ok(self.underlying_price.to_dec())
    }

    fn walk_positive(&self) -> Result<Positive, Box<dyn Error>> {
        Ok(self.underlying_price)
    }
}

#[cfg(test)]
mod tests_walktype {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::utils::logger::setup_logger;
    use crate::{ExpirationDate, pos, spos};
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    // Helper function to create a test option chain
    fn create_test_chain() -> OptionChain {
        let option_chain_params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            pos!(1.0),
            0.00001,
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                pos!(100.0),
                ExpirationDate::Days(pos!(30.0)),
                spos!(0.17),
                Decimal::ZERO,
                pos!(0.05),
                None,
            ),
        );

        OptionChain::build_chain(&option_chain_params)
    }

    #[test]
    fn test_walk_next_small_move() {
        let chain = create_test_chain();

        // Small upward move (~20%)
        let exp = 0.02;
        let result = chain.walk_next(exp);

        assert!(result.is_ok());
        let next_chain = result.unwrap();
        // Verify the new underlying price
        let expected_price = 100.0 * f64::exp(exp);
        assert_relative_eq!(
            next_chain.underlying_price.to_f64(),
            expected_price,
            epsilon = 0.01
        );

        assert_eq!(next_chain.options.len(), 21);

        // Verify strikes are unchanged
        let strikes: Vec<f64> = next_chain
            .options
            .iter()
            .map(|opt| opt.strike_price.to_f64())
            .collect();
        assert_eq!(
            strikes,
            vec![
                92.0, 93.0, 94.0, 95.0, 96.0, 97.0, 98.0, 99.0, 100.0, 101.0, 102.0, 103.0, 104.0,
                105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0, 112.0
            ]
        );
    }

    #[test]
    fn test_walk_next_large_up_move() {
        setup_logger();
        let chain = create_test_chain();

        // Large upward move (~22%)
        let exp = 0.2;
        let result = chain.walk_next(exp);

        assert!(result.is_ok());
        let next_chain = result.unwrap();

        // Verify the new underlying price
        let expected_price = 100.0 * f64::exp(exp);
        assert_relative_eq!(
            next_chain.underlying_price.to_f64(),
            expected_price,
            epsilon = 0.01
        );

        // Large move should trigger rebalancing
        assert_eq!(next_chain.options.len(), 21);

        // Should have removed the lowest strike and added a new higher one
        let strikes: Vec<f64> = next_chain
            .options
            .iter()
            .map(|opt| opt.strike_price.to_f64())
            .collect();

        // Verify the lowest strike is removed (90.0) and a new higher one is added
        assert!(
            !strikes.contains(&90.0),
            "Should have removed the 90.0 strike"
        );
        assert!(
            strikes.contains(&115.0),
            "Should have added the 115.0 strike"
        );
    }

    #[test]
    fn test_walk_next_large_down_move() {
        let chain = create_test_chain();

        // Large downward move (~18%)
        let exp = -0.2;
        let result = chain.walk_next(exp);

        assert!(result.is_ok());
        let next_chain = result.unwrap();

        // Verify the new underlying price
        let expected_price = 100.0 * f64::exp(exp);
        assert_relative_eq!(
            next_chain.underlying_price.to_f64(),
            expected_price,
            epsilon = 0.01
        );

        // Large move should trigger rebalancing
        assert_eq!(next_chain.options.len(), 21);

        // Should have removed the highest strike and added a new lower one
        let strikes: Vec<f64> = next_chain
            .options
            .iter()
            .map(|opt| opt.strike_price.to_f64())
            .collect();

        // Verify the highest strike is removed (110.0) and a new lower one is added
        assert!(
            !strikes.contains(&110.0),
            "Should have removed the 110.0 strike"
        );
        assert!(strikes.contains(&85.0), "Should have added the 85.0 strike");
    }

    #[test]
    fn test_walk_next_extreme_move() {
        let chain = create_test_chain();

        // Extreme upward move (over 100%)
        let exp = 1.0;
        let result = chain.walk_next(exp);

        assert!(result.is_ok());
        let next_chain = result.unwrap();

        // Verify the new underlying price
        let expected_price = 100.0 * f64::exp(exp);
        assert_relative_eq!(
            next_chain.underlying_price.to_f64(),
            expected_price,
            epsilon = 0.01
        );

        // Should still have 5 strikes but with significant rebalancing
        assert_eq!(next_chain.options.len(), 21);
    }

    #[test]
    fn test_walk_dec() {
        let chain = create_test_chain();
        let result = chain.walk_dec();

        assert!(result.is_ok());
        let decimal = result.unwrap();

        // Verify it returns the underlying price as a Decimal
        assert_eq!(decimal, dec!(100.0));
    }

    #[test]
    fn test_walk_positive() {
        let chain = create_test_chain();
        let result = chain.walk_positive();

        assert!(result.is_ok());
        let positive = result.unwrap();

        // Verify it returns the underlying price as a Positive
        assert_eq!(positive, pos!(100.0));
    }
}
