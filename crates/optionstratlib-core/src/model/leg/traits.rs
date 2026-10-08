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

use crate::error::PositionError;
use crate::model::decimal::{d_div, d_mul};
use crate::model::types::Side;
use positive::Positive;
use rust_decimal::Decimal;

/// Hours in a 365-day year, the numerator of the funding-period count.
const HOURS_PER_YEAR: u32 = 24 * 365;

/// Days in the year [`Expirable::time_to_expiration_years`] divides by.
const DAYS_PER_YEAR: u32 = 365;

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
    ///
    /// # Decision (issue #471): the error channel is part of the signature
    ///
    /// This method used to return a bare `Decimal`. Every implementor computes
    /// some form of `(price - basis) * quantity - fees`, and each of those
    /// operators aborts the process on overflow. The inputs are `pub` fields
    /// on the leg structs, so no constructor guard can bound them, and a
    /// `Decimal` has no value that means "this does not fit". The trait
    /// signature was the thing forbidding the report, so the trait signature
    /// is what changed.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::DecimalError`] when the price difference, the
    /// quantity scaling or the fee deduction leaves the representable
    /// `Decimal` range, and [`PositionError::PositiveError`] when a fee total
    /// leaves the `Positive` range. Option legs additionally propagate
    /// whatever [`crate::model::position::Position::pnl_at_expiration`]
    /// reports.
    fn pnl_at_price(&self, price: Positive) -> Result<Decimal, PositionError>;

    /// Returns the total cost to establish this position.
    ///
    /// For long positions, this includes the purchase price plus fees.
    /// For short positions, this typically includes only fees.
    ///
    /// # Decision (issue #471): fallible, alongside `pnl_at_price`
    ///
    /// `Position::total_cost` has returned `Result<Positive, PositionError>`
    /// since the position layer was made panic-free; the leg trait was the
    /// remaining infallible wrapper over the same sum, and `Leg::Option` had
    /// to swallow the position's error as `unwrap_or(Positive::ZERO)` to fit
    /// this signature. A cost of zero is a legitimate answer for a short
    /// leg, so that fallback was indistinguishable from a real result.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] when the size, premium or fee sum leaves the
    /// `Positive` range.
    fn total_cost(&self) -> Result<Positive, PositionError>;

    /// Returns the total fees associated with this position.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] when the open and close fees do not sum
    /// within the `Positive` range. See [`LegAble::total_cost`] for why this
    /// gained an error channel.
    fn fees(&self) -> Result<Positive, PositionError>;

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
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::PositiveError`] when the product leaves the
    /// `Positive` range. The quantity and the price are both caller data,
    /// so the product is not bounded; `Positive * Positive` aborted on it
    /// (#788).
    fn notional_value(&self, price: Positive) -> Result<Positive, PositionError> {
        Ok(self.get_quantity().checked_mul(&price)?)
    }
}

/// Trait for positions that have margin requirements.
///
/// This applies to futures, perpetuals, and CFDs where positions
/// are leveraged and require margin collateral.
///
/// # Decision (issue #788): every method is fallible
///
/// Each implementor scales a `pub` margin, price or quantity field by
/// another, and divides by the quantity for the liquidation price, so the
/// infallible signatures aborted on an overflow or on a zero quantity
/// (`PerpetualPosition::default().liquidation_price(..)` divided by zero).
/// The methods return [`PositionError`] instead, as `pnl_at_price` has since
/// #471.
pub trait Marginable: LegAble {
    /// Returns the initial margin requirement.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] when the margin computation leaves the
    /// `Positive` range.
    fn initial_margin(&self) -> Result<Positive, PositionError>;

    /// Returns the maintenance margin requirement.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] when the margin computation leaves the
    /// `Positive` or `Decimal` range.
    fn maintenance_margin(&self) -> Result<Positive, PositionError>;

    /// Returns the current leverage applied to the position.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] when the leverage computation leaves the
    /// `Decimal` range.
    fn leverage(&self) -> Result<Positive, PositionError>;

    /// Calculates the liquidation price for this position.
    ///
    /// # Arguments
    ///
    /// * `current_price` - The current market price
    ///
    /// # Returns
    ///
    /// `Some(price)` with the price at which the position would be
    /// liquidated, or `None` when no non-negative price reaches liquidation:
    /// a long whose margin buffer is wider than its entry price survives a
    /// fall all the way to zero. `None` is not a price of zero; a price of
    /// zero would be a liquidation level a spot of zero crosses.
    ///
    /// A threshold that falls below zero on the short side is reported as
    /// `Some(Positive::ZERO)`. That is a domain floor, not a substitute: a
    /// short is liquidated once the price rises to the threshold, every
    /// non-negative price already sits at or above a negative one, and zero
    /// is the lowest price where the comparison holds.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::DecimalError`] when the quantity is zero, so
    /// the per-unit margin buffer is undefined, or when the buffer leaves the
    /// `Decimal` range.
    fn liquidation_price(&self, current_price: Positive)
    -> Result<Option<Positive>, PositionError>;

    /// Checks if the position is at risk of liquidation.
    ///
    /// # Arguments
    ///
    /// * `current_price` - The current market price
    /// * `margin_ratio` - Current margin ratio (margin / notional)
    ///
    /// # Errors
    ///
    /// Propagates the error of [`Marginable::liquidation_price`].
    fn is_liquidation_risk(
        &self,
        current_price: Positive,
        margin_ratio: Decimal,
    ) -> Result<bool, PositionError>;
}

/// Turns a computed liquidation threshold into the value
/// [`Marginable::liquidation_price`] reports.
///
/// A non-negative threshold is the liquidation price itself. A negative one
/// depends on the side: a long is liquidated once the price falls to the
/// threshold, which no non-negative price does, so there is no liquidation
/// price (`None`); a short is liquidated once the price rises to it, which
/// every non-negative price already has, so zero is the floor
/// (`Some(Positive::ZERO)`).
#[must_use]
pub(crate) fn liquidation_threshold(threshold: Decimal, side: Side) -> Option<Positive> {
    match Positive::new_decimal(threshold) {
        Ok(price) => Some(price),
        // `new_decimal` rejects only a negative value here.
        Err(_) => match side {
            Side::Long => None,
            Side::Short => Some(Positive::ZERO),
        },
    }
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
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::DecimalError`] when the notional or the
    /// payment leaves the `Decimal` range (#788).
    fn funding_payment(&self, mark_price: Positive) -> Result<Decimal, PositionError>;

    /// Calculates the annualized funding cost/income.
    ///
    /// # Arguments
    ///
    /// * `mark_price` - The current mark price
    ///
    /// The number of funding periods in a year is `24 × 365` hours divided
    /// by [`Fundable::funding_interval_hours`], truncated to a whole number
    /// of periods.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::ValidationError`] when
    /// [`Fundable::funding_interval_hours`] is zero, so a year has no whole
    /// number of periods (the integer division aborted on it, #788), and
    /// propagates [`Fundable::funding_payment`] and the
    /// [`PositionError::DecimalError`] of an annualised payment outside the
    /// `Decimal` range.
    fn annualized_funding(&self, mark_price: Positive) -> Result<Decimal, PositionError> {
        let payment = self.funding_payment(mark_price)?;
        let periods_per_year = HOURS_PER_YEAR
            .checked_div(self.funding_interval_hours())
            .ok_or_else(|| PositionError::invalid_position("funding interval is zero hours"))?;
        Ok(d_mul(
            payment,
            Decimal::from(periods_per_year),
            "Fundable::annualized_funding",
        )?)
    }
}

/// Trait for positions that have an expiration date.
///
/// This applies to futures, forwards, and options.
pub trait Expirable: LegAble {
    /// Returns the expiration date as a Unix timestamp in seconds.
    ///
    /// # Errors
    ///
    /// Returns a [`PositionError`] when the expiration date cannot be
    /// resolved to a calendar instant. The timestamp is never replaced by
    /// `0`, which would read as the Unix epoch.
    fn expiration_timestamp(&self) -> Result<i64, PositionError>;

    /// Returns the number of days until expiration.
    ///
    /// # Errors
    ///
    /// Returns a [`PositionError`] when the time to expiration cannot be
    /// computed. The day count is never replaced by zero, which would read
    /// as an expired position.
    fn days_to_expiration(&self) -> Result<Positive, PositionError>;

    /// Checks if the position has expired.
    ///
    /// # Errors
    ///
    /// Returns a [`PositionError`] when the expiration date cannot be
    /// resolved to a calendar instant. The answer is never replaced by
    /// `false`, which would read as a live position.
    fn is_expired(&self) -> Result<bool, PositionError>;

    /// Returns the time to expiration in years (for pricing calculations).
    ///
    /// # Errors
    ///
    /// Propagates the [`PositionError`] of
    /// [`Expirable::days_to_expiration`].
    fn time_to_expiration_years(&self) -> Result<Decimal, PositionError> {
        Ok(d_div(
            self.days_to_expiration()?.to_dec(),
            Decimal::from(DAYS_PER_YEAR),
            "Expirable::time_to_expiration_years",
        )?)
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

        fn pnl_at_price(&self, price: Positive) -> Result<Decimal, PositionError> {
            let value_change = (price.to_dec() - self.cost_basis.to_dec()) * self.quantity.to_dec();
            Ok(match self.side {
                Side::Long => value_change,
                Side::Short => -value_change,
            })
        }

        fn total_cost(&self) -> Result<Positive, PositionError> {
            Ok(self
                .cost_basis
                .checked_mul(&self.quantity)?
                .checked_add(&self.fees)?)
        }

        fn fees(&self) -> Result<Positive, PositionError> {
            Ok(self.fees)
        }
    }

    #[test]
    fn test_mock_leg_long_pnl() {
        let leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: positive::Positive::ONE,
            side: Side::Long,
            cost_basis: positive::pos_or_panic!(50000.0),
            fees: positive::pos_or_panic!(10.0),
        };

        // Price goes up - profit
        let pnl = leg.pnl_at_price(positive::pos_or_panic!(55000.0));
        assert_eq!(pnl.ok(), Some(Decimal::from(5000)));

        // Price goes down - loss
        let pnl = leg.pnl_at_price(positive::pos_or_panic!(45000.0));
        assert_eq!(pnl.ok(), Some(Decimal::from(-5000)));
    }

    #[test]
    fn test_mock_leg_short_pnl() {
        let leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: positive::Positive::ONE,
            side: Side::Short,
            cost_basis: positive::pos_or_panic!(50000.0),
            fees: positive::pos_or_panic!(10.0),
        };

        // Price goes up - loss for short
        let pnl = leg.pnl_at_price(positive::pos_or_panic!(55000.0));
        assert_eq!(pnl.ok(), Some(Decimal::from(-5000)));

        // Price goes down - profit for short
        let pnl = leg.pnl_at_price(positive::pos_or_panic!(45000.0));
        assert_eq!(pnl.ok(), Some(Decimal::from(5000)));
    }

    #[test]
    fn test_is_long_short() {
        let long_leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: positive::Positive::ONE,
            side: Side::Long,
            cost_basis: positive::pos_or_panic!(50000.0),
            fees: positive::pos_or_panic!(10.0),
        };

        let short_leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: Positive::ONE,
            side: Side::Short,
            cost_basis: positive::pos_or_panic!(50000.0),
            fees: positive::pos_or_panic!(10.0),
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
            quantity: positive::Positive::TWO,
            side: Side::Long,
            cost_basis: positive::pos_or_panic!(50000.0),
            fees: positive::pos_or_panic!(10.0),
        };

        let notional = leg.notional_value(positive::pos_or_panic!(55000.0));
        assert_eq!(notional.ok(), Some(positive::pos_or_panic!(110000.0)));
    }

    #[test]
    fn test_notional_value_overflow_is_error() {
        let leg = MockLeg {
            symbol: "BTC".to_string(),
            quantity: positive::Positive::TWO,
            side: Side::Long,
            cost_basis: positive::Positive::ONE,
            fees: positive::Positive::ZERO,
        };
        assert!(leg.notional_value(Positive::MAX).is_err());
    }

    struct ZeroIntervalLeg(MockLeg);

    impl LegAble for ZeroIntervalLeg {
        fn get_symbol(&self) -> &str {
            self.0.get_symbol()
        }

        fn get_quantity(&self) -> Positive {
            self.0.get_quantity()
        }

        fn get_side(&self) -> Side {
            self.0.get_side()
        }

        fn pnl_at_price(&self, price: Positive) -> Result<Decimal, PositionError> {
            self.0.pnl_at_price(price)
        }

        fn total_cost(&self) -> Result<Positive, PositionError> {
            self.0.total_cost()
        }

        fn fees(&self) -> Result<Positive, PositionError> {
            self.0.fees()
        }
    }

    impl Fundable for ZeroIntervalLeg {
        fn funding_rate(&self) -> Decimal {
            Decimal::ONE
        }

        fn funding_interval_hours(&self) -> u32 {
            0
        }

        fn funding_payment(&self, _mark_price: Positive) -> Result<Decimal, PositionError> {
            Ok(Decimal::ONE)
        }
    }

    // `24 * 365 / 0` aborted with `attempt to divide by zero` (#788).
    #[test]
    fn test_annualized_funding_zero_interval_is_error() {
        let leg = ZeroIntervalLeg(MockLeg {
            symbol: "BTC".to_string(),
            quantity: positive::Positive::ONE,
            side: Side::Long,
            cost_basis: positive::Positive::ONE,
            fees: positive::Positive::ZERO,
        });
        assert!(matches!(
            leg.annualized_funding(Positive::ONE),
            Err(PositionError::ValidationError(_))
        ));
    }
}
