//! Profit contracts and the historical home of the payoff contracts.
//!
//! [`Payoff`] and [`PayoffInfo`] are owned by
//! [`crate::model::payoff`] and re-exported here so that
//! `optionstratlib::pricing::{Payoff, PayoffInfo}` keeps working.

use crate::error::PricingError;
use crate::model::{Options, Position};
use positive::Positive;
use rust_decimal::Decimal;
use tracing::trace;

pub use crate::model::payoff::{Payoff, PayoffInfo};

/// Defines the profit calculation behavior for financial instruments.
///
/// This trait is used to calculate and visualize profit values at different price points
/// for various financial instruments and strategies. It provides:
/// 1. A required method to calculate the actual profit value at a given price
/// 2. A default implementation to convert the profit calculation into a visualization point
///
/// # Usage
///
/// Implement this trait for any type that can calculate profit at a specific price point,
/// such as options contracts, spreads, or complex trading strategies.
///
pub trait Profit {
    /// Calculates the profit at a specified price.
    ///
    /// # Parameters
    ///
    /// * `price` - A positive value representing the price at which to calculate profit
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, PricingError>` - The calculated profit as a Decimal value,
    ///   or an error if the calculation fails
    ///
    /// # Errors
    ///
    /// Returns `PricingError::MethodError` when the strategy cannot
    /// evaluate its payoff at `price` (typically propagated from the
    /// underlying `Position::pnl_at_expiration` or Black–Scholes
    /// evaluation).
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError>;

    /// Creates a chart point representation of the profit at the given price.
    ///
    /// This method automatically determines the appropriate visualization properties based
    /// on the profit value, such as color (green for positive profit, red for negative).
    ///
    /// # Parameters
    ///
    /// * `price` - A positive value representing the price for which to create a chart point
    ///
    /// # Returns
    ///
    /// * `ChartPoint<(f64, f64)>` - A formatted chart point with coordinates (price, profit),
    ///   styling, and a formatted profit label
    ///
    /// # Errors
    ///
    /// Propagates any `PricingError` returned by
    /// [`Profit::calculate_profit_at`].
    fn get_point_at_price(&self, _price: &Positive) -> Result<(Decimal, Decimal), PricingError> {
        let profit = self.calculate_profit_at(_price)?;
        let price: Decimal = _price.into();
        let point = (price, profit);
        trace!("get_point_at_price - point: {:?}", point);
        Ok(point)
    }
}

impl Profit for Options {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        Ok(self.payoff_at_price(price)?)
    }
}

/// Implementation of the Profit trait for the Position struct.
///
/// This allows calculating the profit of a position at a given price by using the
/// position's profit and loss (PnL) calculation at expiration.
///
impl Profit for Position {
    /// Calculates the profit of the position at a specific price.
    ///
    /// This method computes the profit or loss that would be realized if the position
    /// were to expire with the underlying asset at the specified price.
    ///
    /// # Parameters
    ///
    /// * `price` - The price at which to calculate the profit, represented as a Positive value.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, PricingError>` - The calculated profit as a Decimal if successful,
    ///   or an error if the calculation fails.
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        self.pnl_at_expiration(&Some(price))
    }
}
