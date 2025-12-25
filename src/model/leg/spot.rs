/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Spot Position Module
//!
//! This module provides the `SpotPosition` struct for representing direct ownership
//! of underlying assets such as stocks, cryptocurrency spot holdings, or forex spot positions.
//!
//! ## Characteristics
//!
//! - No expiration date
//! - No margin requirements (fully funded)
//! - Delta = ±1.0 per unit (Long = +1, Short = -1)
//! - No funding costs or time decay
//!
//! ## Example
//!
//! ```rust
//! use optionstratlib::model::leg::SpotPosition;
//! use optionstratlib::model::types::Side;
//! use optionstratlib::pos_or_panic;
//! use chrono::Utc;
//!
//! let spot = SpotPosition::new(
//!     "AAPL".to_string(),
//!     Positive::HUNDRED,      // 100 shares
//!     pos_or_panic!(150.0),      // cost basis $150 per share
//!     Side::Long,
//!     Utc::now(),
//!     Positive::ONE,        // $1 open fee
//!     Positive::ONE,        // $1 close fee
//! );
//! ```


use crate::error::GreeksError;
use crate::model::leg::traits::LegAble;
use crate::model::types::Side;
use positive::pos_or_panic;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a spot/underlying asset position.
///
/// A spot position represents direct ownership of an asset without any derivative
/// characteristics. This includes stocks, cryptocurrency spot holdings, forex spot,
/// and commodities held for delivery.
///
/// # Fields
///
/// * `symbol` - The ticker symbol or identifier of the asset
/// * `quantity` - The number of units held
/// * `cost_basis` - The average acquisition price per unit
/// * `side` - Whether the position is Long (owned) or Short (borrowed/sold)
/// * `date` - The timestamp when the position was opened
/// * `open_fee` - Transaction fee paid to open the position
/// * `close_fee` - Transaction fee to be paid when closing the position
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct SpotPosition {
    /// The ticker symbol or identifier of the underlying asset.
    pub symbol: String,

    /// The number of units held in this position.
    pub quantity: Positive,

    /// The average cost per unit at which the position was acquired.
    pub cost_basis: Positive,

    /// The direction of the position (Long = owned, Short = borrowed).
    pub side: Side,

    /// The timestamp when this position was opened.
    pub date: DateTime<Utc>,

    /// The fee paid to open this position.
    pub open_fee: Positive,

    /// The fee to be paid when closing this position.
    pub close_fee: Positive,
}

impl SpotPosition {
    /// Creates a new spot position.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The ticker symbol of the asset
    /// * `quantity` - Number of units to hold
    /// * `cost_basis` - Average price per unit
    /// * `side` - Long or Short position
    /// * `date` - Position open timestamp
    /// * `open_fee` - Fee to open the position
    /// * `close_fee` - Fee to close the position
    ///
    /// # Returns
    ///
    /// A new `SpotPosition` instance.
    #[must_use]
    pub fn new(
        symbol: String,
        quantity: Positive,
        cost_basis: Positive,
        side: Side,
        date: DateTime<Utc>,
        open_fee: Positive,
        close_fee: Positive,
    ) -> Self {
        Self {
            symbol,
            quantity,
            cost_basis,
            side,
            date,
            open_fee,
            close_fee,
        }
    }

    /// Creates a long spot position with default fees.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The ticker symbol
    /// * `quantity` - Number of units
    /// * `cost_basis` - Price per unit
    ///
    /// # Returns
    ///
    /// A new long `SpotPosition` with zero fees.
    #[must_use]
    pub fn long(symbol: String, quantity: Positive, cost_basis: Positive) -> Self {
        Self::new(
            symbol,
            quantity,
            cost_basis,
            Side::Long,
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    /// Creates a short spot position with default fees.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The ticker symbol
    /// * `quantity` - Number of units
    /// * `cost_basis` - Price per unit (entry price for short)
    ///
    /// # Returns
    ///
    /// A new short `SpotPosition` with zero fees.
    #[must_use]
    pub fn short(symbol: String, quantity: Positive, cost_basis: Positive) -> Self {
        Self::new(
            symbol,
            quantity,
            cost_basis,
            Side::Short,
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    /// Returns the total value of the position at the cost basis.
    ///
    /// This represents the initial investment (for long) or proceeds (for short).
    #[must_use]
    pub fn initial_value(&self) -> Positive {
        self.quantity * self.cost_basis
    }

    /// Returns the current market value at a given price.
    ///
    /// # Arguments
    ///
    /// * `current_price` - The current market price per unit
    #[must_use]
    pub fn market_value(&self, current_price: Positive) -> Positive {
        self.quantity * current_price
    }

    /// Calculates the percentage return on the position.
    ///
    /// # Arguments
    ///
    /// * `current_price` - The current market price per unit
    ///
    /// # Returns
    ///
    /// The percentage return as a Decimal (e.g., 0.10 = 10% gain).
    #[must_use]
    pub fn percentage_return(&self, current_price: Positive) -> Decimal {
        if self.cost_basis == Positive::ZERO {
            return Decimal::ZERO;
        }

        let price_change = current_price.to_dec() - self.cost_basis.to_dec();
        let return_pct = price_change / self.cost_basis.to_dec();

        match self.side {
            Side::Long => return_pct,
            Side::Short => -return_pct,
        }
    }

    /// Calculates the break-even price including fees.
    ///
    /// For long positions: cost_basis + (total_fees / quantity)
    /// For short positions: cost_basis - (total_fees / quantity)
    #[must_use]
    pub fn break_even_price(&self) -> Positive {
        if self.quantity == Positive::ZERO {
            return self.cost_basis;
        }

        let fee_per_unit = (self.open_fee + self.close_fee) / self.quantity;

        match self.side {
            Side::Long => self.cost_basis + fee_per_unit,
            Side::Short => {
                if self.cost_basis > fee_per_unit {
                    self.cost_basis - fee_per_unit
                } else {
                    Positive::ZERO
                }
            }
        }
    }
}

impl LegAble for SpotPosition {
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
        let price_change = price.to_dec() - self.cost_basis.to_dec();
        let gross_pnl = price_change * self.quantity.to_dec();
        let total_fees = self.open_fee.to_dec() + self.close_fee.to_dec();

        match self.side {
            Side::Long => gross_pnl - total_fees,
            Side::Short => -gross_pnl - total_fees,
        }
    }

    fn total_cost(&self) -> Positive {
        match self.side {
            Side::Long => self.quantity * self.cost_basis + self.open_fee + self.close_fee,
            Side::Short => self.open_fee + self.close_fee,
        }
    }

    fn fees(&self) -> Positive {
        self.open_fee + self.close_fee
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        let delta_per_unit = match self.side {
            Side::Long => Decimal::ONE,
            Side::Short => -Decimal::ONE,
        };
        Ok(delta_per_unit * self.quantity.to_dec())
    }
}

impl std::fmt::Display for SpotPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} @ {} (fees: {})",
            self.side,
            self.quantity,
            self.symbol,
            self.cost_basis,
            self.open_fee + self.close_fee
        )
    }
}

impl Default for SpotPosition {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            quantity: Positive::ZERO,
            cost_basis: Positive::ZERO,
            side: Side::Long,
            date: Utc::now(),
            open_fee: Positive::ZERO,
            close_fee: Positive::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot_position_new() {
        let spot = SpotPosition::new(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
            Side::Long,
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
        );

        assert_eq!(spot.symbol, "AAPL");
        assert_eq!(spot.quantity, Positive::HUNDRED);
        assert_eq!(spot.cost_basis, pos_or_panic!(150.0));
        assert_eq!(spot.side, Side::Long);
        assert_eq!(spot.open_fee, Positive::ONE);
        assert_eq!(spot.close_fee, Positive::ONE);
    }

    #[test]
    fn test_spot_position_long_convenience() {
        let spot = SpotPosition::long(
            "BTC".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
        );

        assert_eq!(spot.symbol, "BTC");
        assert_eq!(spot.quantity, Positive::ONE);
        assert_eq!(spot.cost_basis, pos_or_panic!(50000.0));
        assert_eq!(spot.side, Side::Long);
        assert_eq!(spot.open_fee, Positive::ZERO);
    }

    #[test]
    fn test_spot_position_short_convenience() {
        let spot = SpotPosition::short(
            "ETH".to_string(),
            pos_or_panic!(10.0),
            pos_or_panic!(3000.0),
        );

        assert_eq!(spot.symbol, "ETH");
        assert_eq!(spot.quantity, pos_or_panic!(10.0));
        assert_eq!(spot.cost_basis, pos_or_panic!(3000.0));
        assert_eq!(spot.side, Side::Short);
    }

    #[test]
    fn test_initial_value() {
        let spot = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        assert_eq!(spot.initial_value(), pos_or_panic!(15000.0));
    }

    #[test]
    fn test_market_value() {
        let spot = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        assert_eq!(
            spot.market_value(pos_or_panic!(160.0)),
            pos_or_panic!(16000.0)
        );
    }

    #[test]
    fn test_long_pnl_profit() {
        let spot = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        let pnl = spot.pnl_at_price(pos_or_panic!(160.0));
        assert_eq!(pnl, Decimal::from(1000)); // (160-150) * 100
    }

    #[test]
    fn test_long_pnl_loss() {
        let spot = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        let pnl = spot.pnl_at_price(pos_or_panic!(140.0));
        assert_eq!(pnl, Decimal::from(-1000)); // (140-150) * 100
    }

    #[test]
    fn test_short_pnl_profit() {
        let spot = SpotPosition::short(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        let pnl = spot.pnl_at_price(pos_or_panic!(140.0));
        assert_eq!(pnl, Decimal::from(1000)); // Short profits when price drops
    }

    #[test]
    fn test_short_pnl_loss() {
        let spot = SpotPosition::short(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        let pnl = spot.pnl_at_price(pos_or_panic!(160.0));
        assert_eq!(pnl, Decimal::from(-1000)); // Short loses when price rises
    }

    #[test]
    fn test_pnl_with_fees() {
        let spot = SpotPosition::new(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
            Side::Long,
            Utc::now(),
            pos_or_panic!(10.0),
            pos_or_panic!(10.0),
        );
        let pnl = spot.pnl_at_price(pos_or_panic!(160.0));
        assert_eq!(pnl, Decimal::from(980)); // 1000 profit - 20 fees
    }

    #[test]
    fn test_delta_long() {
        let spot = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        assert_eq!(spot.delta().unwrap(), Decimal::from(100));
    }

    #[test]
    fn test_delta_short() {
        let spot = SpotPosition::short(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        assert_eq!(spot.delta().unwrap(), Decimal::from(-100));
    }

    #[test]
    fn test_total_cost_long() {
        let spot = SpotPosition::new(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
            Side::Long,
            Utc::now(),
            pos_or_panic!(10.0),
            pos_or_panic!(10.0),
        );
        assert_eq!(spot.total_cost(), pos_or_panic!(15020.0)); // 15000 + 20 fees
    }

    #[test]
    fn test_total_cost_short() {
        let spot = SpotPosition::new(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
            Side::Short,
            Utc::now(),
            pos_or_panic!(10.0),
            pos_or_panic!(10.0),
        );
        assert_eq!(spot.total_cost(), pos_or_panic!(20.0)); // Only fees for short
    }

    #[test]
    fn test_percentage_return_long() {
        let spot = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            Positive::HUNDRED,
        );
        let return_pct = spot.percentage_return(pos_or_panic!(110.0));
        assert_eq!(return_pct, Decimal::new(1, 1)); // 10% = 0.1
    }

    #[test]
    fn test_percentage_return_short() {
        let spot = SpotPosition::short(
            "AAPL".to_string(),
            Positive::HUNDRED,
            Positive::HUNDRED,
        );
        let return_pct = spot.percentage_return(pos_or_panic!(90.0));
        assert_eq!(return_pct, Decimal::new(1, 1)); // 10% profit for short when price drops
    }

    #[test]
    fn test_break_even_long() {
        let spot = SpotPosition::new(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
            Side::Long,
            Utc::now(),
            pos_or_panic!(50.0),
            pos_or_panic!(50.0),
        );
        assert_eq!(spot.break_even_price(), pos_or_panic!(151.0)); // 150 + (100/100)
    }

    #[test]
    fn test_break_even_short() {
        let spot = SpotPosition::new(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
            Side::Short,
            Utc::now(),
            pos_or_panic!(50.0),
            pos_or_panic!(50.0),
        );
        assert_eq!(spot.break_even_price(), pos_or_panic!(149.0)); // 150 - (100/100)
    }

    #[test]
    fn test_display() {
        let spot = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        let display = format!("{}", spot);
        assert!(display.contains("Long"));
        assert!(display.contains("AAPL"));
        assert!(display.contains("100"));
    }

    #[test]
    fn test_is_long_short() {
        let long = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        let short = SpotPosition::short(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );

        assert!(long.is_long());
        assert!(!long.is_short());
        assert!(!short.is_long());
        assert!(short.is_short());
    }

    #[test]
    fn test_gamma_is_zero() {
        let spot = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        assert_eq!(spot.gamma().unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_theta_is_zero() {
        let spot = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        assert_eq!(spot.theta().unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_vega_is_zero() {
        let spot = SpotPosition::long(
            "AAPL".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(150.0),
        );
        assert_eq!(spot.vega().unwrap(), Decimal::ZERO);
    }
}
