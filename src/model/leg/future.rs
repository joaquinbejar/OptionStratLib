/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Future Position Module
//!
//! This module provides the `FuturePosition` struct for representing standardized
//! exchange-traded futures contracts.
//!
//! ## Characteristics
//!
//! - Fixed expiration date
//! - Margin-based (leveraged)
//! - Mark-to-market daily settlement
//! - Delta ≈ ±1.0 per contract (adjusted for contract size)
//! - Rho sensitivity for interest rate changes
//!
//! ## Example
//!
//! ```rust
//! use optionstratlib::model::leg::FuturePosition;
//! use optionstratlib::model::types::Side;
//! use optionstratlib::model::ExpirationDate;
//! use optionstratlib::pos;
//! use chrono::Utc;
//!
//! let future = FuturePosition::new(
//!     "ES".to_string(),        // E-mini S&P 500
//!     pos!(2.0),               // 2 contracts
//!     pos!(4500.0),            // entry price
//!     Side::Long,
//!     ExpirationDate::Days(pos!(30.0)),
//!     pos!(50.0),              // contract multiplier
//!     pos!(15000.0),           // initial margin
//!     pos!(12000.0),           // maintenance margin
//!     Utc::now(),
//!     pos!(5.0),               // fees
//! );
//! ```

use crate::Positive;
use crate::error::GreeksError;
use crate::model::ExpirationDate;
use crate::model::leg::traits::{Expirable, LegAble, Marginable};
use crate::model::types::Side;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a futures contract position.
///
/// Futures are standardized contracts traded on exchanges that obligate the buyer
/// to purchase (or seller to sell) an asset at a predetermined price on a specific
/// future date.
///
/// # Fields
///
/// * `symbol` - Contract symbol (e.g., "ES", "CL", "BTC-QUARTERLY")
/// * `quantity` - Number of contracts
/// * `entry_price` - Average entry price
/// * `side` - Long or Short position
/// * `expiration_date` - Contract expiry date
/// * `contract_size` - Multiplier (e.g., 50 for ES, 1000 for CL)
/// * `initial_margin` - Initial margin requirement
/// * `maintenance_margin` - Maintenance margin level
/// * `date` - Position open timestamp
/// * `fees` - Commission and exchange fees
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct FuturePosition {
    /// Contract symbol (e.g., "ES", "CL", "GC").
    pub symbol: String,

    /// Number of contracts held.
    pub quantity: Positive,

    /// Average entry price per contract.
    pub entry_price: Positive,

    /// Position direction (Long or Short).
    pub side: Side,

    /// Contract expiration date.
    pub expiration_date: ExpirationDate,

    /// Contract size multiplier (notional per point).
    pub contract_size: Positive,

    /// Initial margin requirement per contract.
    pub initial_margin_req: Positive,

    /// Maintenance margin requirement per contract.
    pub maintenance_margin_req: Positive,

    /// Timestamp when the position was opened.
    pub date: DateTime<Utc>,

    /// Total trading fees paid.
    pub fees: Positive,
}

impl FuturePosition {
    /// Creates a new futures position.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Contract symbol
    /// * `quantity` - Number of contracts
    /// * `entry_price` - Average entry price
    /// * `side` - Long or Short
    /// * `expiration_date` - Contract expiry
    /// * `contract_size` - Contract multiplier
    /// * `initial_margin_req` - Initial margin per contract
    /// * `maintenance_margin_req` - Maintenance margin per contract
    /// * `date` - Position open timestamp
    /// * `fees` - Trading fees
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: String,
        quantity: Positive,
        entry_price: Positive,
        side: Side,
        expiration_date: ExpirationDate,
        contract_size: Positive,
        initial_margin_req: Positive,
        maintenance_margin_req: Positive,
        date: DateTime<Utc>,
        fees: Positive,
    ) -> Self {
        Self {
            symbol,
            quantity,
            entry_price,
            side,
            expiration_date,
            contract_size,
            initial_margin_req,
            maintenance_margin_req,
            date,
            fees,
        }
    }

    /// Creates a long futures position with default settings.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Contract symbol
    /// * `quantity` - Number of contracts
    /// * `entry_price` - Entry price
    /// * `expiration_date` - Contract expiry
    /// * `contract_size` - Contract multiplier
    /// * `margin` - Margin requirement
    #[must_use]
    pub fn long(
        symbol: String,
        quantity: Positive,
        entry_price: Positive,
        expiration_date: ExpirationDate,
        contract_size: Positive,
        margin: Positive,
    ) -> Self {
        Self::new(
            symbol,
            quantity,
            entry_price,
            Side::Long,
            expiration_date,
            contract_size,
            margin,
            margin * Decimal::new(8, 1), // 80% of initial margin
            Utc::now(),
            Positive::ZERO,
        )
    }

    /// Creates a short futures position with default settings.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Contract symbol
    /// * `quantity` - Number of contracts
    /// * `entry_price` - Entry price
    /// * `expiration_date` - Contract expiry
    /// * `contract_size` - Contract multiplier
    /// * `margin` - Margin requirement
    #[must_use]
    pub fn short(
        symbol: String,
        quantity: Positive,
        entry_price: Positive,
        expiration_date: ExpirationDate,
        contract_size: Positive,
        margin: Positive,
    ) -> Self {
        Self::new(
            symbol,
            quantity,
            entry_price,
            Side::Short,
            expiration_date,
            contract_size,
            margin,
            margin * Decimal::new(8, 1),
            Utc::now(),
            Positive::ZERO,
        )
    }

    /// Returns the notional value of the position at entry.
    ///
    /// Notional = quantity × entry_price × contract_size
    #[must_use]
    pub fn notional_value_at_entry(&self) -> Positive {
        self.quantity * self.entry_price * self.contract_size
    }

    /// Returns the notional value at a given price.
    ///
    /// # Arguments
    ///
    /// * `current_price` - Current market price
    #[must_use]
    pub fn notional_value_at_price(&self, current_price: Positive) -> Positive {
        self.quantity * current_price * self.contract_size
    }

    /// Calculates the unrealized P&L at a given price.
    ///
    /// # Arguments
    ///
    /// * `current_price` - Current market price
    #[must_use]
    pub fn unrealized_pnl(&self, current_price: Positive) -> Decimal {
        let price_change = current_price.to_dec() - self.entry_price.to_dec();
        let pnl = price_change * self.quantity.to_dec() * self.contract_size.to_dec();

        match self.side {
            Side::Long => pnl,
            Side::Short => -pnl,
        }
    }

    /// Calculates the tick value (value of one minimum price movement).
    ///
    /// # Arguments
    ///
    /// * `tick_size` - Minimum price increment
    #[must_use]
    pub fn tick_value(&self, tick_size: Positive) -> Positive {
        tick_size * self.contract_size
    }

    /// Returns the total margin required for this position.
    #[must_use]
    pub fn total_margin_required(&self) -> Positive {
        self.initial_margin_req * self.quantity
    }

    /// Calculates the basis (futures price - spot price).
    ///
    /// # Arguments
    ///
    /// * `spot_price` - Current spot price of the underlying
    #[must_use]
    pub fn basis(&self, spot_price: Positive) -> Decimal {
        self.entry_price.to_dec() - spot_price.to_dec()
    }

    /// Calculates the implied leverage of the position.
    ///
    /// Leverage = Notional Value / Margin Required
    #[must_use]
    pub fn implied_leverage(&self) -> Decimal {
        let margin = self.total_margin_required();
        if margin == Positive::ZERO {
            return Decimal::ZERO;
        }

        self.notional_value_at_entry().to_dec() / margin.to_dec()
    }
}

impl LegAble for FuturePosition {
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
        self.unrealized_pnl(price) - self.fees.to_dec()
    }

    fn total_cost(&self) -> Positive {
        self.total_margin_required() + self.fees
    }

    fn fees(&self) -> Positive {
        self.fees
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        let delta_per_contract = match self.side {
            Side::Long => self.contract_size.to_dec(),
            Side::Short => -self.contract_size.to_dec(),
        };
        Ok(delta_per_contract * self.quantity.to_dec())
    }

    fn rho(&self) -> Result<Decimal, GreeksError> {
        let time_to_exp = self.time_to_expiration_years();
        let notional = self.notional_value_at_entry().to_dec();

        let rho_value = match self.side {
            Side::Long => notional * time_to_exp,
            Side::Short => -notional * time_to_exp,
        };

        Ok(rho_value / Decimal::ONE_HUNDRED)
    }
}

impl Marginable for FuturePosition {
    fn initial_margin(&self) -> Positive {
        self.initial_margin_req * self.quantity
    }

    fn maintenance_margin(&self) -> Positive {
        self.maintenance_margin_req * self.quantity
    }

    fn leverage(&self) -> Positive {
        let lev = self.implied_leverage();
        Positive::new_decimal(lev).unwrap_or(crate::pos!(1.0))
    }

    fn liquidation_price(&self, _current_price: Positive) -> Positive {
        let margin_buffer = self.initial_margin().to_dec() - self.maintenance_margin().to_dec();
        let price_buffer = margin_buffer / (self.quantity.to_dec() * self.contract_size.to_dec());

        match self.side {
            Side::Long => {
                let liq = self.entry_price.to_dec() - price_buffer;
                Positive::new_decimal(liq).unwrap_or(Positive::ZERO)
            }
            Side::Short => {
                let liq = self.entry_price.to_dec() + price_buffer;
                Positive::new_decimal(liq).unwrap_or(Positive::ZERO)
            }
        }
    }

    fn is_liquidation_risk(&self, current_price: Positive, _margin_ratio: Decimal) -> bool {
        let liq_price = self.liquidation_price(current_price);

        match self.side {
            Side::Long => current_price <= liq_price,
            Side::Short => current_price >= liq_price,
        }
    }
}

impl Expirable for FuturePosition {
    fn expiration_timestamp(&self) -> i64 {
        self.expiration_date
            .get_date()
            .map(|d| d.timestamp())
            .unwrap_or(0)
    }

    fn days_to_expiration(&self) -> Positive {
        self.expiration_date.get_years().unwrap_or(Positive::ZERO) * Decimal::from(365)
    }

    fn is_expired(&self) -> bool {
        self.expiration_date
            .get_date()
            .map(|d| d < Utc::now())
            .unwrap_or(false)
    }
}

impl std::fmt::Display for FuturePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} @ {} (size: {}, exp: {})",
            self.side,
            self.quantity,
            self.symbol,
            self.entry_price,
            self.contract_size,
            self.expiration_date
        )
    }
}

impl Default for FuturePosition {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            quantity: Positive::ZERO,
            entry_price: Positive::ZERO,
            side: Side::Long,
            expiration_date: ExpirationDate::Days(crate::pos!(30.0)),
            contract_size: crate::pos!(1.0),
            initial_margin_req: Positive::ZERO,
            maintenance_margin_req: Positive::ZERO,
            date: Utc::now(),
            fees: Positive::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_future_position_new() {
        let future = FuturePosition::new(
            "ES".to_string(),
            pos!(2.0),
            pos!(4500.0),
            Side::Long,
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
            pos!(12000.0),
            Utc::now(),
            pos!(5.0),
        );

        assert_eq!(future.symbol, "ES");
        assert_eq!(future.quantity, pos!(2.0));
        assert_eq!(future.entry_price, pos!(4500.0));
        assert_eq!(future.side, Side::Long);
        assert_eq!(future.contract_size, pos!(50.0));
    }

    #[test]
    fn test_future_long_convenience() {
        let future = FuturePosition::long(
            "ES".to_string(),
            pos!(1.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        assert_eq!(future.side, Side::Long);
        assert_eq!(future.initial_margin_req, pos!(15000.0));
    }

    #[test]
    fn test_future_short_convenience() {
        let future = FuturePosition::short(
            "ES".to_string(),
            pos!(1.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        assert_eq!(future.side, Side::Short);
    }

    #[test]
    fn test_notional_value() {
        let future = FuturePosition::long(
            "ES".to_string(),
            pos!(2.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        assert_eq!(future.notional_value_at_entry(), pos!(450000.0));
    }

    #[test]
    fn test_unrealized_pnl_long() {
        let future = FuturePosition::long(
            "ES".to_string(),
            pos!(1.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        let pnl = future.unrealized_pnl(pos!(4510.0));
        assert_eq!(pnl, Decimal::from(500));

        let pnl_loss = future.unrealized_pnl(pos!(4490.0));
        assert_eq!(pnl_loss, Decimal::from(-500));
    }

    #[test]
    fn test_unrealized_pnl_short() {
        let future = FuturePosition::short(
            "ES".to_string(),
            pos!(1.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        let pnl = future.unrealized_pnl(pos!(4490.0));
        assert_eq!(pnl, Decimal::from(500));

        let pnl_loss = future.unrealized_pnl(pos!(4510.0));
        assert_eq!(pnl_loss, Decimal::from(-500));
    }

    #[test]
    fn test_delta_long() {
        let future = FuturePosition::long(
            "ES".to_string(),
            pos!(2.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        assert_eq!(future.delta().unwrap(), Decimal::from(100));
    }

    #[test]
    fn test_delta_short() {
        let future = FuturePosition::short(
            "ES".to_string(),
            pos!(2.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        assert_eq!(future.delta().unwrap(), Decimal::from(-100));
    }

    #[test]
    fn test_implied_leverage() {
        let future = FuturePosition::long(
            "ES".to_string(),
            pos!(1.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        let leverage = future.implied_leverage();
        assert_eq!(leverage, Decimal::from(15));
    }

    #[test]
    fn test_basis() {
        let future = FuturePosition::long(
            "ES".to_string(),
            pos!(1.0),
            pos!(4510.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        let basis = future.basis(pos!(4500.0));
        assert_eq!(basis, Decimal::from(10));
    }

    #[test]
    fn test_tick_value() {
        let future = FuturePosition::long(
            "ES".to_string(),
            pos!(1.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        let tick_val = future.tick_value(pos!(0.25));
        assert_eq!(tick_val, pos!(12.5));
    }

    #[test]
    fn test_total_margin_required() {
        let future = FuturePosition::long(
            "ES".to_string(),
            pos!(2.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        assert_eq!(future.total_margin_required(), pos!(30000.0));
    }

    #[test]
    fn test_display() {
        let future = FuturePosition::long(
            "ES".to_string(),
            pos!(1.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        );

        let display = format!("{}", future);
        assert!(display.contains("Long"));
        assert!(display.contains("ES"));
        assert!(display.contains("4500"));
    }
}
