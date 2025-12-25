/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Leg Traits Module
//!
//! This module defines common traits for different types of trading legs
//! (positions in various instruments like spot, futures, perpetuals, etc.).
//!
//! The `LegAble` trait provides a unified interface for calculating profit/loss,
//! retrieving position information, and computing Greeks across different
//! instrument types.

use crate::error::GreeksError;
use crate::model::types::Side;
use positive::Positive;
use rust_decimal::Decimal;

/// Common trait for all leg types in a trading strategy.
///
/// This trait provides a unified interface for different position types
/// (spot, futures, perpetuals, options, etc.) enabling polymorphic handling
/// in multi-leg strategies.
///
/// # Implementors
///
/// - `SpotPosition` - Direct ownership of underlying asset
/// - `FuturePosition` - Exchange-traded futures contracts
/// - `PerpetualPosition` - Crypto perpetual swap contracts
/// - `Position` - Option positions (via adapter)
pub trait LegAble {
    /// Returns the symbol/ticker of the underlying instrument.
    fn get_symbol(&self) -> &str;

    /// Returns the position quantity (number of units/contracts).
    fn get_quantity(&self) -> Positive;

    /// Returns the position side (Long or Short).
    fn get_side(&self) -> Side;

    /// Calculates the profit/loss at a given price.
    ///
    /// # Arguments
    ///
    /// * `price` - The price at which to calculate P&L
    ///
    /// # Returns
    ///
    /// The profit (positive) or loss (negative) as a Decimal value.
    fn pnl_at_price(&self, price: Positive) -> Decimal;

    /// Returns the total cost to establish this position.
    ///
    /// For long positions, this includes the purchase price plus fees.
    /// For short positions, this typically includes only fees.
    fn total_cost(&self) -> Positive;

    /// Returns the total fees associated with this position.
    fn fees(&self) -> Positive;

    /// Returns the delta of this position.
    ///
    /// - Spot positions: ±1.0 per unit (Long = +1, Short = -1)
    /// - Futures/Perpetuals: ±1.0 × contract_size × leverage
    /// - Options: Calculated from Black-Scholes or other models
    ///
    /// # Returns
    ///
    /// The position delta as a Decimal, or an error if calculation fails.
    fn delta(&self) -> Result<Decimal, GreeksError>;

    /// Returns the gamma of this position.
    ///
    /// For linear instruments (spot, futures, perpetuals), gamma is always 0.
    /// Only options have non-zero gamma.
    ///
    /// # Returns
    ///
    /// The position gamma as a Decimal, or an error if calculation fails.
    fn gamma(&self) -> Result<Decimal, GreeksError> {
        Ok(Decimal::ZERO)
    }

    /// Returns the theta of this position.
    ///
    /// - Spot: 0 (no time decay)
    /// - Futures: ~0 (basis converges over time)
    /// - Perpetuals: Funding rate impact
    /// - Options: Calculated time decay
    ///
    /// # Returns
    ///
    /// The position theta as a Decimal, or an error if calculation fails.
    fn theta(&self) -> Result<Decimal, GreeksError> {
        Ok(Decimal::ZERO)
    }

    /// Returns the vega of this position.
    ///
    /// For linear instruments (spot, futures, perpetuals), vega is always 0.
    /// Only options have non-zero vega.
    ///
    /// # Returns
    ///
    /// The position vega as a Decimal, or an error if calculation fails.
    fn vega(&self) -> Result<Decimal, GreeksError> {
        Ok(Decimal::ZERO)
    }

    /// Returns the rho of this position.
    ///
    /// - Spot: 0
    /// - Futures/Forwards: Interest rate sensitivity
    /// - Options: Calculated interest rate sensitivity
    ///
    /// # Returns
    ///
    /// The position rho as a Decimal, or an error if calculation fails.
    fn rho(&self) -> Result<Decimal, GreeksError> {
        Ok(Decimal::ZERO)
    }

    /// Checks if this is a long position.
    #[must_use]
    fn is_long(&self) -> bool {
        matches!(self.get_side(), Side::Long)
    }

    /// Checks if this is a short position.
    #[must_use]
    fn is_short(&self) -> bool {
        matches!(self.get_side(), Side::Short)
    }

    /// Returns the notional value of the position at a given price.
    ///
    /// Notional = quantity × price
    fn notional_value(&self, price: Positive) -> Positive {
        self.get_quantity() * price
    }
}

/// Trait for positions that have margin requirements.
///
/// This applies to futures, perpetuals, and CFDs where positions
/// are leveraged and require margin collateral.
pub trait Marginable: LegAble {
    /// Returns the initial margin requirement.
    fn initial_margin(&self) -> Positive;

    /// Returns the maintenance margin requirement.
    fn maintenance_margin(&self) -> Positive;

    /// Returns the current leverage applied to the position.
    fn leverage(&self) -> Positive;

    /// Calculates the liquidation price for this position.
    ///
    /// # Arguments
    ///
    /// * `current_price` - The current market price
    ///
    /// # Returns
    ///
    /// The price at which the position would be liquidated.
    fn liquidation_price(&self, current_price: Positive) -> Positive;

    /// Checks if the position is at risk of liquidation.
    ///
    /// # Arguments
    ///
    /// * `current_price` - The current market price
    /// * `margin_ratio` - Current margin ratio (margin / notional)
    fn is_liquidation_risk(&self, current_price: Positive, margin_ratio: Decimal) -> bool;
}

/// Trait for positions that have funding rate payments.
///
/// This applies primarily to perpetual swap contracts in crypto markets.
pub trait Fundable: LegAble {
    /// Returns the current funding rate (as a decimal, e.g., 0.0001 = 0.01%).
    fn funding_rate(&self) -> Decimal;

    /// Returns the funding interval in hours (typically 8 for most exchanges).
    fn funding_interval_hours(&self) -> u32;

    /// Calculates the funding payment for the current period.
    ///
    /// Positive value means the position pays funding.
    /// Negative value means the position receives funding.
    ///
    /// # Arguments
    ///
    /// * `mark_price` - The current mark price for funding calculation
    fn funding_payment(&self, mark_price: Positive) -> Decimal;

    /// Calculates the annualized funding cost/income.
    ///
    /// # Arguments
    ///
    /// * `mark_price` - The current mark price
    fn annualized_funding(&self, mark_price: Positive) -> Decimal {
        let payment = self.funding_payment(mark_price);
        let periods_per_year = Decimal::from(24 * 365 / self.funding_interval_hours());
        payment * periods_per_year
    }
}

/// Trait for positions that have an expiration date.
///
/// This applies to futures, forwards, and options.
pub trait Expirable: LegAble {
    /// Returns the expiration date as a timestamp.
    fn expiration_timestamp(&self) -> i64;

    /// Returns the number of days until expiration.
    fn days_to_expiration(&self) -> Positive;

    /// Checks if the position has expired.
    fn is_expired(&self) -> bool;

    /// Returns the time to expiration in years (for pricing calculations).
    fn time_to_expiration_years(&self) -> Decimal {
        self.days_to_expiration().to_dec() / Decimal::from(365)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock implementation for testing
    struct MockLeg {
        symbol: String,
        quantity: Positive,
        side: Side,
        cost_basis: Positive,
        fees: Positive,
    }

    impl LegAble for MockLeg {
        fn get_symbol(&self) -> &str {
            &self.symbol
        }

        fn get_quantity(&self) -> Positive {
            self.quantity
        }

        fn get_side(&self) -> Side {
            self.side
        }

        fn pnl_at_price(&self, price: Positive) -> Decimal {
            let value_change = (price.to_dec() - self.cost_basis.to_dec()) * self.quantity.to_dec();
            match self.side {
                Side::Long => value_change,
                Side::Short => -value_change,
            }
        }

        fn total_cost(&self) -> Positive {
            self.cost_basis * self.quantity + self.fees
        }

        fn fees(&self) -> Positive {
            self.fees
        }

        fn delta(&self) -> Result<Decimal, GreeksError> {
            let delta_per_unit = match self.side {
                Side::Long => Decimal::ONE,
                Side::Short => -Decimal::ONE,
            };
            Ok(delta_per_unit * self.quantity.to_dec())
        }
    }

    #[test]
    fn test_mock_leg_long_pnl() {
        let leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: crate::Positive::ONE,
            side: Side::Long,
            cost_basis: crate::pos_or_panic!(50000.0),
            fees: crate::pos_or_panic!(10.0),
        };

        // Price goes up - profit
        let pnl = leg.pnl_at_price(crate::pos_or_panic!(55000.0));
        assert_eq!(pnl, Decimal::from(5000));

        // Price goes down - loss
        let pnl = leg.pnl_at_price(crate::pos_or_panic!(45000.0));
        assert_eq!(pnl, Decimal::from(-5000));
    }

    #[test]
    fn test_mock_leg_short_pnl() {
        let leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: crate::Positive::ONE,
            side: Side::Short,
            cost_basis: crate::pos_or_panic!(50000.0),
            fees: crate::pos_or_panic!(10.0),
        };

        // Price goes up - loss for short
        let pnl = leg.pnl_at_price(crate::pos_or_panic!(55000.0));
        assert_eq!(pnl, Decimal::from(-5000));

        // Price goes down - profit for short
        let pnl = leg.pnl_at_price(crate::pos_or_panic!(45000.0));
        assert_eq!(pnl, Decimal::from(5000));
    }

    #[test]
    fn test_mock_leg_delta() {
        let long_leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: crate::Positive::TWO,
            side: Side::Long,
            cost_basis: crate::pos_or_panic!(50000.0),
            fees: crate::pos_or_panic!(10.0),
        };

        let short_leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: crate::Positive::TWO,
            side: Side::Short,
            cost_basis: crate::pos_or_panic!(50000.0),
            fees: crate::pos_or_panic!(10.0),
        };

        assert_eq!(long_leg.delta().unwrap(), Decimal::from(2));
        assert_eq!(short_leg.delta().unwrap(), Decimal::from(-2));
    }

    #[test]
    fn test_is_long_short() {
        let long_leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: crate::Positive::ONE,
            side: Side::Long,
            cost_basis: crate::pos_or_panic!(50000.0),
            fees: crate::pos_or_panic!(10.0),
        };

        let short_leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: crate::Positive::ONE,
            side: Side::Short,
            cost_basis: crate::pos_or_panic!(50000.0),
            fees: crate::pos_or_panic!(10.0),
        };

        assert!(long_leg.is_long());
        assert!(!long_leg.is_short());
        assert!(!short_leg.is_long());
        assert!(short_leg.is_short());
    }

    #[test]
    fn test_notional_value() {
        let leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: crate::Positive::TWO,
            side: Side::Long,
            cost_basis: crate::pos_or_panic!(50000.0),
            fees: crate::pos_or_panic!(10.0),
        };

        let notional = leg.notional_value(crate::pos_or_panic!(55000.0));
        assert_eq!(notional, crate::pos_or_panic!(110000.0));
    }
}
