/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/3/25
******************************************************************************/
use crate::chains::chain::{OptionChain, OptionData};
use crate::chains::utils::OptionChainParams;
use crate::simulation::types::Walktypable;
use crate::simulation::{RandomWalkGraph, Walkable};
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
        let next_underlying_price = pos!(self.underlying_price.to_f64() * f64::exp(exp)).max(Positive::ZERO);
        
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

        // Count strikes above and below the new underlying price
        let upper_strikes = options
            .iter()
            .filter(|opt| opt.strike_price >= next_underlying_price)
            .count();
        let lower_strikes = options.len() - upper_strikes;

        // Check if we need to rebalance the strikes
        if (upper_strikes as isize - lower_strikes as isize).abs() <= 1 {
            // Strikes are balanced, no adjustment needed
            chain.options = options;
            return Ok(chain);
        }

        // Need to rebalance by adding/removing strikes
        if upper_strikes > lower_strikes {
            // Too many upper strikes, remove highest and add a new lower one
            if let Some(highest_strike) = options.iter().max_by_key(|opt| opt.strike_price) {
                let new_lowest_strike = options
                    .iter()
                    .min_by_key(|opt| opt.strike_price)
                    .map(|opt| opt.strike_price - self.get_strike_interval())
                    .unwrap_or_else(|| pos!(next_underlying_price.to_f64() * 0.9));

                // Remove highest strike option
                let highest_strike_price = highest_strike.strike_price;
                options.retain(|opt| opt.strike_price != highest_strike_price);

                // Create and add new lower strike option
                let price_params =
                    self.get_params_for_new_strike(new_lowest_strike, next_underlying_price)?;
                let mut new_option = OptionData::default();
                new_option.strike_price = new_lowest_strike;
                new_option.implied_volatility = price_params.implied_volatility;
                new_option.calculate_prices(&price_params, true)?;
                options.insert(new_option);
            }
        } else {
            // Too many lower strikes, remove lowest and add a new upper one
            if let Some(lowest_strike) = options.iter().min_by_key(|opt| opt.strike_price) {
                let new_highest_strike = options
                    .iter()
                    .max_by_key(|opt| opt.strike_price)
                    .map(|opt| opt.strike_price + self.get_strike_interval())
                    .unwrap_or_else(|| pos!(next_underlying_price.to_f64() * 1.1));

                // Remove lowest strike option
                let lowest_strike_price = lowest_strike.strike_price;
                options.retain(|opt| opt.strike_price != lowest_strike_price);

                // Create and add new upper strike option
                let price_params =
                    self.get_params_for_new_strike(new_highest_strike, next_underlying_price)?;
                let mut new_option = OptionData::default();
                new_option.strike_price = new_highest_strike;
                new_option.implied_volatility = price_params.implied_volatility;
                new_option.calculate_prices(&price_params, true)?;
                options.insert(new_option);
            }
        }

        chain.options = options;
        Ok(chain)
    }

    fn walk_dec(&self) -> Result<Decimal, Box<dyn Error>> {
        Ok(self.underlying_price.to_dec())
    }

    fn walk_positive(&self) -> Result<Positive, Box<dyn Error>> {
        Ok(self.underlying_price)
    }
}
