/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Perpetual Position Module
//!
//! This module provides the `PerpetualPosition` struct for representing perpetual
//! swap contracts, commonly used in cryptocurrency markets.
//!
//! ## Characteristics
//!
//! - No expiration date (perpetual)
//! - Margin-based (leveraged)
//! - Funding rate mechanism to anchor price to spot
//! - Funding payments typically every 8 hours
//! - Liquidation risk based on margin ratio
//!
//! ## Example
//!
//! ```rust
//! use optionstratlib::model::leg::{PerpetualPosition, MarginType};
//! use optionstratlib::model::types::Side;
//! use positive::{pos_or_panic,Positive};
//! use chrono::Utc;
//! use rust_decimal_macros::dec;
//!
//! let perp = PerpetualPosition::new(
//!     "BTC-USDT-PERP".to_string(),
//!     Positive::ONE,           // 1 BTC position size
//!     pos_or_panic!(50000.0),       // entry price
//!     Side::Long,
//!     pos_or_panic!(10.0),          // 10x leverage
//!     pos_or_panic!(5000.0),        // margin posted
//!     MarginType::Isolated,
//!     dec!(0.0001),        // 0.01% funding rate
//!     Utc::now(),
//!     pos_or_panic!(5.0),           // $5 fees
//! );
//! ```

use crate::error::{GreeksError, PositionError, PricingError};
use crate::model::decimal::{d_add, d_div, d_mul, d_sub};
use crate::model::leg::traits::{Fundable, LegAble, Marginable};
use crate::model::types::Side;
use chrono::{DateTime, Utc};
use positive::Positive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Margin type for perpetual positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum MarginType {
    /// Cross margin - shares margin across all positions
    Cross,
    /// Isolated margin - margin is specific to this position
    #[default]
    Isolated,
}

impl std::fmt::Display for MarginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cross => write!(f, "Cross"),
            Self::Isolated => write!(f, "Isolated"),
        }
    }
}

/// Represents a perpetual swap contract position.
///
/// Perpetual swaps are derivative contracts common in cryptocurrency markets
/// that allow traders to speculate on price movements with leverage, without
/// an expiration date. They use a funding rate mechanism to keep the contract
/// price anchored to the spot price.
///
/// # Fields
///
/// * `symbol` - Trading pair (e.g., "BTC-USDT-PERP")
/// * `quantity` - Position size in base currency
/// * `entry_price` - Average entry price
/// * `side` - Long or Short position
/// * `leverage` - Applied leverage (1x to 125x typical)
/// * `margin` - Collateral posted for this position
/// * `margin_type` - Cross or Isolated margin mode
/// * `funding_rate` - Current funding rate (updated periodically)
/// * `date` - Position open timestamp
/// * `fees` - Trading fees (maker/taker)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PerpetualPosition {
    /// Trading pair symbol (e.g., "BTC-USDT-PERP").
    pub symbol: String,

    /// Position size in base currency units.
    pub quantity: Positive,

    /// Average entry price for the position.
    pub entry_price: Positive,

    /// Position direction (Long or Short).
    pub side: Side,

    /// Leverage multiplier applied to the position.
    pub leverage: Positive,

    /// Margin/collateral posted for this position.
    pub margin: Positive,

    /// Margin mode (Cross or Isolated).
    pub margin_type: MarginType,

    /// Current funding rate as a decimal (e.g., 0.0001 = 0.01%).
    pub funding_rate: Decimal,

    /// Timestamp when the position was opened.
    pub date: DateTime<Utc>,

    /// Total trading fees paid.
    pub fees: Positive,
}

impl PerpetualPosition {
    /// Default funding interval in hours (8 hours for most exchanges).
    pub const DEFAULT_FUNDING_INTERVAL_HOURS: u32 = 8;

    /// Default maintenance margin ratio (typically 0.5% to 1%).
    pub const DEFAULT_MAINTENANCE_MARGIN_RATIO: Decimal = Decimal::ONE_HUNDRED;

    /// Creates a new perpetual position.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `quantity` - Position size in base currency
    /// * `entry_price` - Average entry price
    /// * `side` - Long or Short
    /// * `leverage` - Leverage multiplier
    /// * `margin` - Collateral posted
    /// * `margin_type` - Cross or Isolated
    /// * `funding_rate` - Current funding rate
    /// * `date` - Position open timestamp
    /// * `fees` - Trading fees
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: String,
        quantity: Positive,
        entry_price: Positive,
        side: Side,
        leverage: Positive,
        margin: Positive,
        margin_type: MarginType,
        funding_rate: Decimal,
        date: DateTime<Utc>,
        fees: Positive,
    ) -> Self {
        Self {
            symbol,
            quantity,
            entry_price,
            side,
            leverage,
            margin,
            margin_type,
            funding_rate,
            date,
            fees,
        }
    }

    /// Creates a long perpetual position with default settings.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `quantity` - Position size
    /// * `entry_price` - Entry price
    /// * `leverage` - Leverage multiplier
    /// * `margin` - Collateral posted
    #[must_use]
    pub fn long(
        symbol: String,
        quantity: Positive,
        entry_price: Positive,
        leverage: Positive,
        margin: Positive,
    ) -> Self {
        Self::new(
            symbol,
            quantity,
            entry_price,
            Side::Long,
            leverage,
            margin,
            MarginType::Isolated,
            Decimal::ZERO,
            Utc::now(),
            Positive::ZERO,
        )
    }

    /// Creates a short perpetual position with default settings.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `quantity` - Position size
    /// * `entry_price` - Entry price
    /// * `leverage` - Leverage multiplier
    /// * `margin` - Collateral posted
    #[must_use]
    pub fn short(
        symbol: String,
        quantity: Positive,
        entry_price: Positive,
        leverage: Positive,
        margin: Positive,
    ) -> Self {
        Self::new(
            symbol,
            quantity,
            entry_price,
            Side::Short,
            leverage,
            margin,
            MarginType::Isolated,
            Decimal::ZERO,
            Utc::now(),
            Positive::ZERO,
        )
    }

    /// Returns the notional value of the position.
    ///
    /// Notional = quantity × entry_price
    #[must_use]
    pub fn notional_value_at_entry(&self) -> Positive {
        self.quantity * self.entry_price
    }

    /// Returns the current notional value at a given price.
    ///
    /// # Arguments
    ///
    /// * `current_price` - Current market price
    #[must_use]
    pub fn notional_value_at_price(&self, current_price: Positive) -> Positive {
        self.quantity * current_price
    }

    /// Calculates the unrealized P&L at a given price.
    ///
    /// # Arguments
    ///
    /// * `current_price` - Current market price
    ///
    /// # Decision (issue #471): fallible because `pnl_at_price` is
    ///
    /// This is the whole body of [`LegAble::pnl_at_price`] for a perpetual,
    /// so leaving it infallible would have moved that method's abort one
    /// frame down rather than removing it. `entry_price` and `quantity` are
    /// `pub`, so no constructor guard bounds the product.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError::Decimal`] when the price difference or the
    /// quantity scaling leaves the representable `Decimal` range.
    pub fn unrealized_pnl(&self, current_price: Positive) -> Result<Decimal, PricingError> {
        let price_change = d_sub(
            current_price.to_dec(),
            self.entry_price.to_dec(),
            "PerpetualPosition::unrealized_pnl/price_change",
        )?;
        let pnl = d_mul(
            price_change,
            self.quantity.to_dec(),
            "PerpetualPosition::unrealized_pnl/quantity",
        )?;

        Ok(match self.side {
            Side::Long => pnl,
            // Negating a representable `Decimal` is always representable.
            Side::Short => -pnl,
        })
    }

    /// Calculates the ROE (Return on Equity) percentage.
    ///
    /// ROE = (Unrealized P&L / Margin) × 100
    ///
    /// A zero margin has no return on equity to report, and the function
    /// keeps its existing answer of `Decimal::ZERO` for that case rather than
    /// turning it into an error.
    ///
    /// # Arguments
    ///
    /// * `current_price` - Current market price
    ///
    /// # Errors
    ///
    /// Propagates [`PerpetualPosition::unrealized_pnl`], and returns
    /// [`PricingError::Decimal`] when the ratio or its percentage scaling
    /// leaves the representable `Decimal` range.
    pub fn roe_percentage(&self, current_price: Positive) -> Result<Decimal, PricingError> {
        if self.margin == Positive::ZERO {
            return Ok(Decimal::ZERO);
        }

        let pnl = self.unrealized_pnl(current_price)?;
        let ratio = d_div(
            pnl,
            self.margin.to_dec(),
            "PerpetualPosition::roe_percentage/ratio",
        )?;
        Ok(d_mul(
            ratio,
            Decimal::ONE_HUNDRED,
            "PerpetualPosition::roe_percentage/percent",
        )?)
    }

    /// Calculates the margin ratio at a given price.
    ///
    /// Margin Ratio = (Margin + Unrealized P&L) / Notional Value
    ///
    /// A zero notional has no margin ratio to report, and the function keeps
    /// its existing answer of `Decimal::ZERO` for that case rather than
    /// turning it into an error.
    ///
    /// # Arguments
    ///
    /// * `current_price` - Current market price
    ///
    /// # Errors
    ///
    /// Propagates [`PerpetualPosition::unrealized_pnl`], and returns
    /// [`PricingError::Decimal`] when the equity sum or the ratio leaves the
    /// representable `Decimal` range.
    pub fn margin_ratio(&self, current_price: Positive) -> Result<Decimal, PricingError> {
        // Formed here rather than read from `notional_value_at_price`, whose
        // raw product aborts for a quantity and price whose product leaves
        // the `Positive` range — before this `Result` could report it.
        let notional = self.quantity.checked_mul(&current_price)?;
        if notional == Positive::ZERO {
            return Ok(Decimal::ZERO);
        }

        let equity = d_add(
            self.margin.to_dec(),
            self.unrealized_pnl(current_price)?,
            "PerpetualPosition::margin_ratio/equity",
        )?;
        Ok(d_div(
            equity,
            notional.to_dec(),
            "PerpetualPosition::margin_ratio/ratio",
        )?)
    }

    /// Updates the funding rate.
    ///
    /// # Arguments
    ///
    /// * `new_rate` - New funding rate
    pub fn update_funding_rate(&mut self, new_rate: Decimal) {
        self.funding_rate = new_rate;
    }

    /// Calculates the effective leverage at a given price.
    ///
    /// Effective Leverage = Notional Value / (Margin + Unrealized P&L)
    ///
    /// A position whose equity has been wiped out is infinitely levered, and
    /// the function keeps its existing answer of `Decimal::MAX` for that case
    /// rather than turning it into an error.
    ///
    /// # Arguments
    ///
    /// * `current_price` - Current market price
    ///
    /// # Errors
    ///
    /// Propagates [`PerpetualPosition::unrealized_pnl`], and returns
    /// [`PricingError::Decimal`] when the equity sum or the quotient leaves
    /// the representable `Decimal` range.
    pub fn effective_leverage(&self, current_price: Positive) -> Result<Decimal, PricingError> {
        let equity = d_add(
            self.margin.to_dec(),
            self.unrealized_pnl(current_price)?,
            "PerpetualPosition::effective_leverage/equity",
        )?;
        if equity <= Decimal::ZERO {
            return Ok(Decimal::MAX);
        }

        let notional = self.quantity.checked_mul(&current_price)?;
        Ok(d_div(
            notional.to_dec(),
            equity,
            "PerpetualPosition::effective_leverage/leverage",
        )?)
    }
}

impl LegAble for PerpetualPosition {
    fn get_symbol(&self) -> &str {
        &self.symbol
    }

    fn get_quantity(&self) -> Positive {
        self.quantity
    }

    fn get_side(&self) -> Side {
        self.side
    }

    fn pnl_at_price(&self, price: Positive) -> Result<Decimal, PricingError> {
        Ok(d_sub(
            self.unrealized_pnl(price)?,
            self.fees.to_dec(),
            "PerpetualPosition::pnl_at_price/net",
        )?)
    }

    fn total_cost(&self) -> Result<Positive, PositionError> {
        Ok(self.margin.checked_add(&self.fees)?)
    }

    fn fees(&self) -> Result<Positive, PositionError> {
        Ok(self.fees)
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        let delta_per_unit = match self.side {
            Side::Long => Decimal::ONE,
            Side::Short => -Decimal::ONE,
        };
        Ok(delta_per_unit * self.quantity.to_dec())
    }

    fn theta(&self) -> Result<Decimal, GreeksError> {
        Ok(-self.funding_payment(self.entry_price))
    }
}

impl Marginable for PerpetualPosition {
    fn initial_margin(&self) -> Positive {
        self.margin
    }

    fn maintenance_margin(&self) -> Positive {
        let notional = self.notional_value_at_entry();
        let maintenance_ratio = Decimal::new(5, 3);
        Positive::new_decimal(notional.to_dec() * maintenance_ratio).unwrap_or(Positive::ZERO)
    }

    fn leverage(&self) -> Positive {
        self.leverage
    }

    fn liquidation_price(&self, _current_price: Positive) -> Positive {
        let maintenance = self.maintenance_margin();

        match self.side {
            Side::Long => {
                let liq_price = self.entry_price.to_dec()
                    - (self.margin.to_dec() - maintenance.to_dec()) / self.quantity.to_dec();
                Positive::new_decimal(liq_price).unwrap_or(Positive::ZERO)
            }
            Side::Short => {
                let liq_price = self.entry_price.to_dec()
                    + (self.margin.to_dec() - maintenance.to_dec()) / self.quantity.to_dec();
                Positive::new_decimal(liq_price).unwrap_or(Positive::ZERO)
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

impl Fundable for PerpetualPosition {
    fn funding_rate(&self) -> Decimal {
        self.funding_rate
    }

    fn funding_interval_hours(&self) -> u32 {
        Self::DEFAULT_FUNDING_INTERVAL_HOURS
    }

    fn funding_payment(&self, mark_price: Positive) -> Decimal {
        let notional = self.quantity.to_dec() * mark_price.to_dec();
        let payment = notional * self.funding_rate;

        match self.side {
            Side::Long => payment,
            Side::Short => -payment,
        }
    }
}

impl std::fmt::Display for PerpetualPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} @ {} ({}x {}, margin: {})",
            self.side,
            self.quantity,
            self.symbol,
            self.entry_price,
            self.leverage,
            self.margin_type,
            self.margin
        )
    }
}

impl Default for PerpetualPosition {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            quantity: Positive::ZERO,
            entry_price: Positive::ZERO,
            side: Side::Long,
            leverage: positive::Positive::ONE,
            margin: Positive::ZERO,
            margin_type: MarginType::default(),
            funding_rate: Decimal::ZERO,
            date: Utc::now(),
            fees: Positive::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use positive::pos_or_panic;

    use rust_decimal_macros::dec;

    /// Both ratios divide by a notional whose product leaves the `Positive`
    /// range for a large quantity at a large price. Both signatures promise a
    /// `Result`, so both have to report it rather than abort inside the raw
    /// multiplication `notional_value_at_price` performs.
    #[test]
    fn test_perpetual_ratios_report_a_notional_that_overflows() {
        let perp = PerpetualPosition::new(
            "BTC-USDT-PERP".to_string(),
            Positive::MAX,
            pos_or_panic!(50000.0),
            Side::Long,
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
            MarginType::Isolated,
            dec!(0.0001),
            Utc::now(),
            pos_or_panic!(5.0),
        );

        assert!(perp.margin_ratio(pos_or_panic!(50000.0)).is_err());
        assert!(perp.effective_leverage(pos_or_panic!(50000.0)).is_err());
    }

    #[test]
    fn test_perpetual_position_new() {
        let perp = PerpetualPosition::new(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            Side::Long,
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
            MarginType::Isolated,
            dec!(0.0001),
            Utc::now(),
            pos_or_panic!(5.0),
        );

        assert_eq!(perp.symbol, "BTC-USDT-PERP");
        assert_eq!(perp.quantity, Positive::ONE);
        assert_eq!(perp.entry_price, pos_or_panic!(50000.0));
        assert_eq!(perp.side, Side::Long);
        assert_eq!(perp.leverage, pos_or_panic!(10.0));
        assert_eq!(perp.margin, pos_or_panic!(5000.0));
        assert_eq!(perp.margin_type, MarginType::Isolated);
        assert_eq!(perp.funding_rate, dec!(0.0001));
    }

    #[test]
    fn test_perpetual_long_convenience() {
        let perp = PerpetualPosition::long(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
        );

        assert_eq!(perp.side, Side::Long);
        assert_eq!(perp.margin_type, MarginType::Isolated);
    }

    #[test]
    fn test_perpetual_short_convenience() {
        let perp = PerpetualPosition::short(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
        );

        assert_eq!(perp.side, Side::Short);
    }

    #[test]
    fn test_notional_value() {
        let perp = PerpetualPosition::long(
            "BTC-USDT-PERP".to_string(),
            Positive::TWO,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(10000.0),
        );

        assert_eq!(perp.notional_value_at_entry(), pos_or_panic!(100000.0));
        assert_eq!(
            perp.notional_value_at_price(pos_or_panic!(55000.0)),
            pos_or_panic!(110000.0)
        );
    }

    #[test]
    fn test_unrealized_pnl_long() {
        let perp = PerpetualPosition::long(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
        );

        assert_eq!(
            perp.unrealized_pnl(pos_or_panic!(55000.0)).ok(),
            Some(dec!(5000))
        );
        assert_eq!(
            perp.unrealized_pnl(pos_or_panic!(45000.0)).ok(),
            Some(dec!(-5000))
        );
    }

    #[test]
    fn test_unrealized_pnl_short() {
        let perp = PerpetualPosition::short(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
        );

        assert_eq!(
            perp.unrealized_pnl(pos_or_panic!(45000.0)).ok(),
            Some(dec!(5000))
        );
        assert_eq!(
            perp.unrealized_pnl(pos_or_panic!(55000.0)).ok(),
            Some(dec!(-5000))
        );
    }

    #[test]
    fn test_roe_percentage() {
        let perp = PerpetualPosition::long(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
        );

        let roe = perp.roe_percentage(pos_or_panic!(55000.0));
        assert_eq!(roe.ok(), Some(dec!(100)));
    }

    #[test]
    fn test_delta_long() {
        let perp = PerpetualPosition::long(
            "BTC-USDT-PERP".to_string(),
            Positive::TWO,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(10000.0),
        );

        assert_eq!(perp.delta().unwrap(), dec!(2));
    }

    #[test]
    fn test_delta_short() {
        let perp = PerpetualPosition::short(
            "BTC-USDT-PERP".to_string(),
            Positive::TWO,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(10000.0),
        );

        assert_eq!(perp.delta().unwrap(), dec!(-2));
    }

    #[test]
    fn test_funding_payment_long() {
        let perp = PerpetualPosition::new(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            Side::Long,
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
            MarginType::Isolated,
            dec!(0.0001),
            Utc::now(),
            Positive::ZERO,
        );

        let payment = perp.funding_payment(pos_or_panic!(50000.0));
        assert_eq!(payment, dec!(5));
    }

    #[test]
    fn test_funding_payment_short() {
        let perp = PerpetualPosition::new(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            Side::Short,
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
            MarginType::Isolated,
            dec!(0.0001),
            Utc::now(),
            Positive::ZERO,
        );

        let payment = perp.funding_payment(pos_or_panic!(50000.0));
        assert_eq!(payment, dec!(-5));
    }

    #[test]
    fn test_liquidation_price_long() {
        let perp = PerpetualPosition::long(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
        );

        let liq_price = perp.liquidation_price(pos_or_panic!(50000.0));
        assert!(liq_price < pos_or_panic!(50000.0));
        assert!(liq_price > Positive::ZERO);
    }

    #[test]
    fn test_liquidation_price_short() {
        let perp = PerpetualPosition::short(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
        );

        let liq_price = perp.liquidation_price(pos_or_panic!(50000.0));
        assert!(liq_price > pos_or_panic!(50000.0));
    }

    #[test]
    fn test_effective_leverage() {
        let perp = PerpetualPosition::long(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
        );

        let eff_lev = perp.effective_leverage(pos_or_panic!(50000.0));
        assert_eq!(eff_lev.ok(), Some(dec!(10)));
    }

    #[test]
    fn test_margin_type_display() {
        assert_eq!(format!("{}", MarginType::Cross), "Cross");
        assert_eq!(format!("{}", MarginType::Isolated), "Isolated");
    }

    #[test]
    fn test_display() {
        let perp = PerpetualPosition::long(
            "BTC-USDT-PERP".to_string(),
            Positive::ONE,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(5000.0),
        );

        let display = format!("{}", perp);
        assert!(display.contains("Long"));
        assert!(display.contains("BTC-USDT-PERP"));
        assert!(display.contains("10x"));
    }
}
