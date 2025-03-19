/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/3/25
******************************************************************************/
use crate::chains::chain::{OptionChain, OptionData};
use crate::chains::utils::OptionChainParams;
use crate::simulation::types::Walktypable;
use crate::simulation::{RandomWalkGraph, Walkable};
use crate::utils::Len;
use crate::{Positive, pos};
use rust_decimal::Decimal;
use std::collections::BTreeSet;
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
        let next_underlying_price =
            pos!(self.underlying_price.to_f64() * f64::exp(exp)).max(Positive::ZERO);

        let mut chain = OptionChain::new(
            &self.symbol,
            next_underlying_price,
            self.get_expiration_date(),
            self.risk_free_rate,
            self.dividend_yield,
        );

        let mut options: BTreeSet<OptionData> = BTreeSet::new();

        // Recalculate prices for all existing options with the new underlying price
        for option in self.get_single_iter() {
            let mut price_params = self.get_params(option.strike_price)?;
            price_params.underlying_price = next_underlying_price;

            let mut new_option = option.clone();
            new_option.calculate_prices(&price_params, true)?;
            options.insert(new_option);
        }

        // Get the current strike range
        let mut min_strike = Positive::INFINITY;
        let mut max_strike = Positive::ZERO;

        for option in &options {
            min_strike = min_strike.min(option.strike_price);
            max_strike = max_strike.max(option.strike_price);
        }

        let strike_interval = self.get_strike_interval();

        // Calculate how many strikes we should have above and below the underlying price
        // Aim to have the underlying price roughly centered
        let desired_strikes_per_side = self.len() / 2;

        // Calculate the desired minimum and maximum strikes
        let desired_min_strike_f64 = next_underlying_price.to_f64()
            - (desired_strikes_per_side as f64 * strike_interval.to_f64());
        let desired_min_strike = match desired_min_strike_f64.is_sign_positive() {
            true => pos!(desired_min_strike_f64),
            false => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Underlying price is too low to maintain desired strike distribution",
                )));
            }
        };
        let desired_max_strike = pos!(
            next_underlying_price.to_f64()
                + (desired_strikes_per_side as f64 * strike_interval.to_f64())
        );

        // Add missing lower strikes if needed
        let mut current_strike = min_strike;
        while current_strike > desired_min_strike {
            current_strike = current_strike - strike_interval;
            let price_params =
                self.get_params_for_new_strike(current_strike, next_underlying_price)?;

            let mut new_option = OptionData::default();
            new_option.strike_price = current_strike;
            new_option.implied_volatility = price_params.implied_volatility;
            new_option.calculate_prices(&price_params, true)?;
            options.insert(new_option);
        }

        // Add missing upper strikes if needed
        let mut current_strike = max_strike;
        while current_strike < desired_max_strike {
            current_strike += strike_interval;
            let price_params =
                self.get_params_for_new_strike(current_strike, next_underlying_price)?;

            let mut new_option = OptionData::default();
            new_option.strike_price = current_strike;
            new_option.implied_volatility = price_params.implied_volatility;
            new_option.calculate_prices(&price_params, true)?;
            options.insert(new_option);
        }

        // Remove excess strikes to maintain a reasonable number
        // First, identify strikes too far from the underlying
        let mut strikes_to_remove = Vec::new();

        for option in &options {
            if option.strike_price < desired_min_strike || option.strike_price > desired_max_strike
            {
                strikes_to_remove.push(option.strike_price);
            }
        }

        // Then remove them
        for strike in strikes_to_remove {
            options.retain(|opt| opt.strike_price != strike);
        }

        chain.options = options;
        chain.update_greeks();
        
        // TODO: apply volatility smile base in option_chain.curve volatility
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

        assert_eq!(next_chain.options.len(), 20);

        // Verify strikes are unchanged
        let strikes: Vec<f64> = next_chain
            .options
            .iter()
            .map(|opt| opt.strike_price.to_f64())
            .collect();
        assert_eq!(
            strikes,
            vec![
                93.0, 94.0, 95.0, 96.0, 97.0, 98.0, 99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 105.0,
                106.0, 107.0, 108.0, 109.0, 110.0, 111.0, 112.0
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
        assert_eq!(next_chain.options.len(), 20);

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
        assert_eq!(next_chain.options.len(), 20);

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
        assert_eq!(next_chain.options.len(), 20);
    }

    #[test]
    fn test_walk_next_zero_crossing() {
        setup_logger();
        let mut chain = create_test_chain();

        // First modify the chain to have a low starting price
        chain.underlying_price = pos!(10.0);

        // Try a large downward move that would push price negative
        let exp = -2.0;
        let result = chain.walk_next(exp);

        assert!(result.is_err());
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
