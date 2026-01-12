/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

//! # Shared Strategy Traits and Utilities
//!
//! This module provides shared traits for different strategy categories,
//! reducing code duplication across strategy implementations.
//!
//! ## Strategy Categories
//!
//! - **Spread strategies**: Two-leg strategies with upper and lower strikes
//! - **Butterfly strategies**: Three-strike strategies with wings and body
//! - **Condor strategies**: Four-strike strategies
//! - **Straddle/Strangle strategies**: Volatility-based strategies
//!
//! ## Usage
//!
//! Strategies implement these traits to gain access to common calculations
//! and reduce boilerplate code.

use crate::error::strategies::StrategyError;
use crate::model::position::Position;
use positive::Positive;
use rust_decimal::Decimal;

/// Trait for vertical spread strategies (two legs with different strikes).
///
/// Vertical spreads involve buying and selling options of the same type
/// (calls or puts) with different strike prices but the same expiration.
///
/// # Examples
///
/// - Bull Call Spread
/// - Bear Call Spread
/// - Bull Put Spread
/// - Bear Put Spread
pub trait SpreadStrategy {
    /// Returns the lower strike price of the spread.
    fn lower_strike(&self) -> Positive;

    /// Returns the upper strike price of the spread.
    fn upper_strike(&self) -> Positive;

    /// Returns the spread width (difference between strikes).
    ///
    /// # Returns
    ///
    /// The difference between upper and lower strike prices.
    fn spread_width(&self) -> Positive {
        self.upper_strike() - self.lower_strike()
    }

    /// Returns the short leg position.
    fn short_leg(&self) -> &Position;

    /// Returns the long leg position.
    fn long_leg(&self) -> &Position;
}

/// Trait for butterfly-type strategies (three strikes with wings and body).
///
/// Butterfly strategies involve three strike prices where the middle strike
/// (body) has twice the position size of the outer strikes (wings).
///
/// # Examples
///
/// - Long Call Butterfly
/// - Short Call Butterfly
/// - Iron Butterfly
pub trait ButterflyStrategy {
    /// Returns the wing strikes (lower, upper).
    fn wing_strikes(&self) -> (Positive, Positive);

    /// Returns the body (middle) strike.
    fn body_strike(&self) -> Positive;

    /// Returns the wing width (distance from body to each wing).
    ///
    /// # Returns
    ///
    /// The distance from the body strike to either wing.
    fn wing_width(&self) -> Positive {
        let (lower, upper) = self.wing_strikes();
        (upper - lower) / Decimal::TWO
    }

    /// Returns all positions in the butterfly.
    fn get_butterfly_positions(&self) -> Vec<&Position>;
}

/// Trait for condor-type strategies (four strikes).
///
/// Condor strategies involve four strike prices, typically with two
/// inner strikes (short positions) and two outer strikes (long positions).
///
/// # Examples
///
/// - Iron Condor
/// - Long Call Condor
/// - Long Put Condor
pub trait CondorStrategy {
    /// Returns all four strikes (lowest to highest).
    fn strikes(&self) -> (Positive, Positive, Positive, Positive);

    /// Returns the inner spread width (between the two middle strikes).
    fn inner_width(&self) -> Positive {
        let (_, lower_mid, upper_mid, _) = self.strikes();
        upper_mid - lower_mid
    }

    /// Returns the outer spread width (total width of the condor).
    fn outer_width(&self) -> Positive {
        let (lowest, _, _, highest) = self.strikes();
        highest - lowest
    }

    /// Returns the put spread width (lower wing).
    fn put_spread_width(&self) -> Positive {
        let (lowest, lower_mid, _, _) = self.strikes();
        lower_mid - lowest
    }

    /// Returns the call spread width (upper wing).
    fn call_spread_width(&self) -> Positive {
        let (_, _, upper_mid, highest) = self.strikes();
        highest - upper_mid
    }

    /// Returns all positions in the condor.
    fn get_condor_positions(&self) -> Vec<&Position>;
}

/// Trait for straddle-type strategies (same strike for call and put).
///
/// Straddle strategies involve buying or selling both a call and put
/// at the same strike price and expiration.
///
/// # Examples
///
/// - Long Straddle
/// - Short Straddle
pub trait StraddleStrategy {
    /// Returns the strike price (same for both call and put).
    fn strike(&self) -> Positive;

    /// Returns the call position.
    fn call_position(&self) -> &Position;

    /// Returns the put position.
    fn put_position(&self) -> &Position;

    /// Returns true if this is a long straddle (buying both options).
    fn is_long(&self) -> bool;
}

/// Trait for strangle-type strategies (different strikes for call and put).
///
/// Strangle strategies involve buying or selling a call and put with
/// different strike prices but the same expiration.
///
/// # Examples
///
/// - Long Strangle
/// - Short Strangle
pub trait StrangleStrategy {
    /// Returns the call strike price.
    fn call_strike(&self) -> Positive;

    /// Returns the put strike price.
    fn put_strike(&self) -> Positive;

    /// Returns the strangle width (distance between strikes).
    fn strangle_width(&self) -> Positive {
        self.call_strike() - self.put_strike()
    }

    /// Returns the call position.
    fn call_position(&self) -> &Position;

    /// Returns the put position.
    fn put_position(&self) -> &Position;

    /// Returns true if this is a long strangle (buying both options).
    fn is_long(&self) -> bool;
}

/// Helper function to calculate break-even for a credit spread.
///
/// # Arguments
///
/// * `short_strike` - Strike price of the short option
/// * `net_credit` - Net credit received from the spread
/// * `is_call_spread` - True if this is a call spread, false for put spread
///
/// # Returns
///
/// The break-even price for the spread.
pub fn credit_spread_break_even(
    short_strike: Positive,
    net_credit: Positive,
    is_call_spread: bool,
) -> Positive {
    if is_call_spread {
        short_strike + net_credit
    } else {
        short_strike - net_credit
    }
}

/// Helper function to calculate break-even for a debit spread.
///
/// # Arguments
///
/// * `long_strike` - Strike price of the long option
/// * `net_debit` - Net debit paid for the spread
/// * `is_call_spread` - True if this is a call spread, false for put spread
///
/// # Returns
///
/// The break-even price for the spread.
pub fn debit_spread_break_even(
    long_strike: Positive,
    net_debit: Positive,
    is_call_spread: bool,
) -> Positive {
    if is_call_spread {
        long_strike + net_debit
    } else {
        long_strike - net_debit
    }
}

/// Helper function to calculate the profit ratio.
///
/// # Arguments
///
/// * `max_profit` - Maximum profit of the strategy
/// * `max_loss` - Maximum loss of the strategy
///
/// # Returns
///
/// The profit ratio as a percentage, or an error if calculation fails.
pub fn calculate_profit_ratio(
    max_profit: Positive,
    max_loss: Positive,
) -> Result<Decimal, StrategyError> {
    if max_loss == Positive::ZERO {
        return Ok(Decimal::MAX);
    }
    if max_profit == Positive::ZERO {
        return Ok(Decimal::ZERO);
    }
    Ok(max_profit.to_dec() / max_loss.to_dec() * Decimal::ONE_HUNDRED)
}

/// Helper function to aggregate fees from multiple positions.
///
/// # Arguments
///
/// * `positions` - Slice of position references
///
/// # Returns
///
/// Total fees (open + close) for all positions.
pub fn aggregate_fees(positions: &[&Position]) -> Positive {
    positions
        .iter()
        .map(|p| p.open_fee + p.close_fee)
        .fold(Positive::ZERO, |acc, fee| acc + fee)
}

/// Helper function to aggregate premiums from multiple positions.
///
/// # Arguments
///
/// * `positions` - Slice of position references
///
/// # Returns
///
/// Total premium for all positions.
pub fn aggregate_premiums(positions: &[&Position]) -> Positive {
    positions
        .iter()
        .map(|p| p.premium)
        .fold(Positive::ZERO, |acc, premium| acc + premium)
}

#[cfg(test)]
mod tests_shared {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_credit_spread_break_even_call() {
        let short_strike = Positive::new(100.0).unwrap();
        let net_credit = Positive::new(5.0).unwrap();
        let break_even = credit_spread_break_even(short_strike, net_credit, true);
        assert_eq!(break_even, Positive::new(105.0).unwrap());
    }

    #[test]
    fn test_credit_spread_break_even_put() {
        let short_strike = Positive::new(100.0).unwrap();
        let net_credit = Positive::new(5.0).unwrap();
        let break_even = credit_spread_break_even(short_strike, net_credit, false);
        assert_eq!(break_even, Positive::new(95.0).unwrap());
    }

    #[test]
    fn test_debit_spread_break_even_call() {
        let long_strike = Positive::new(100.0).unwrap();
        let net_debit = Positive::new(3.0).unwrap();
        let break_even = debit_spread_break_even(long_strike, net_debit, true);
        assert_eq!(break_even, Positive::new(103.0).unwrap());
    }

    #[test]
    fn test_debit_spread_break_even_put() {
        let long_strike = Positive::new(100.0).unwrap();
        let net_debit = Positive::new(3.0).unwrap();
        let break_even = debit_spread_break_even(long_strike, net_debit, false);
        assert_eq!(break_even, Positive::new(97.0).unwrap());
    }

    #[test]
    fn test_calculate_profit_ratio() {
        let max_profit = Positive::new(50.0).unwrap();
        let max_loss = Positive::new(100.0).unwrap();
        let ratio = calculate_profit_ratio(max_profit, max_loss).unwrap();
        assert_eq!(ratio, dec!(50));
    }

    #[test]
    fn test_calculate_profit_ratio_zero_loss() {
        let max_profit = Positive::new(50.0).unwrap();
        let max_loss = Positive::ZERO;
        let ratio = calculate_profit_ratio(max_profit, max_loss).unwrap();
        assert_eq!(ratio, Decimal::MAX);
    }

    #[test]
    fn test_calculate_profit_ratio_zero_profit() {
        let max_profit = Positive::ZERO;
        let max_loss = Positive::new(100.0).unwrap();
        let ratio = calculate_profit_ratio(max_profit, max_loss).unwrap();
        assert_eq!(ratio, Decimal::ZERO);
    }
}
