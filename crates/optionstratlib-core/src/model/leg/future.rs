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
//! use optionstratlib_core::model::leg::FuturePosition;
//! use optionstratlib_core::model::types::Side;
//! use optionstratlib_core::model::ExpirationDate;
//! use positive::{pos_or_panic,Positive};
//! use chrono::Utc;
//!
//! let future = FuturePosition::new(
//!     "ES".to_string(),        // E-mini S&P 500
//!     Positive::TWO,               // 2 contracts
//!     pos_or_panic!(4500.0),            // entry price
//!     Side::Long,
//!     ExpirationDate::Days(pos_or_panic!(30.0)),
//!     pos_or_panic!(50.0),              // contract multiplier
//!     pos_or_panic!(15000.0),           // initial margin
//!     pos_or_panic!(12000.0),           // maintenance margin
//!     Utc::now(),
//!     pos_or_panic!(5.0),               // fees
//! );
//! ```

use crate::error::{DecimalError, PositionError};
use crate::model::ExpirationDate;
use crate::model::decimal::{d_mul, d_sub};
use crate::model::expiration::resolve_expiration_date;
use crate::model::leg::traits::{Expirable, LegAble, Marginable, liquidation_threshold};
use crate::model::types::Side;
use chrono::{DateTime, Utc};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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
    ///
    /// # Decision (issue #471): fallible because `pnl_at_price` is
    ///
    /// This is the whole body of [`LegAble::pnl_at_price`] for a future, so
    /// leaving it infallible would have moved that method's abort one frame
    /// down rather than removing it. `entry_price`, `quantity` and
    /// `contract_size` are `pub`, so no constructor guard bounds the product.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::DecimalError`] when the price difference, the
    /// quantity scaling or the contract-size scaling leaves the representable
    /// `Decimal` range.
    pub fn unrealized_pnl(&self, current_price: Positive) -> Result<Decimal, PositionError> {
        let price_change = d_sub(
            current_price.to_dec(),
            self.entry_price.to_dec(),
            "FuturePosition::unrealized_pnl/price_change",
        )?;
        let pnl = d_mul(
            d_mul(
                price_change,
                self.quantity.to_dec(),
                "FuturePosition::unrealized_pnl/quantity",
            )?,
            self.contract_size.to_dec(),
            "FuturePosition::unrealized_pnl/contract_size",
        )?;

        Ok(match self.side {
            Side::Long => pnl,
            // Negating a representable `Decimal` is always representable.
            Side::Short => -pnl,
        })
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

    fn pnl_at_price(&self, price: Positive) -> Result<Decimal, PositionError> {
        Ok(d_sub(
            self.unrealized_pnl(price)?,
            self.fees.to_dec(),
            "FuturePosition::pnl_at_price/net",
        )?)
    }

    fn total_cost(&self) -> Result<Positive, PositionError> {
        // The product is formed here rather than read from
        // `total_margin_required`, which multiplies raw and would abort on
        // `initial_margin_req = Positive::MAX` before this `Result` could
        // carry the failure. The infallible helper keeps its signature; what
        // this fixes is the error channel promising something it could not
        // deliver.
        let margin = self
            .initial_margin_req
            .checked_mul(&self.quantity)
            .map_err(PositionError::from)?;
        Ok(margin.checked_add(&self.fees)?)
    }

    fn fees(&self) -> Result<Positive, PositionError> {
        Ok(self.fees)
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
        Positive::new_decimal(lev).unwrap_or(positive::Positive::ONE)
    }

    /// `None` for a long whose margin buffer per unit exceeds the entry
    /// price: no non-negative price liquidates it. A short threshold below
    /// zero is floored at zero, the documented domain floor of
    /// [`Marginable::liquidation_price`].
    fn liquidation_price(&self, _current_price: Positive) -> Option<Positive> {
        let margin_buffer = self.initial_margin().to_dec() - self.maintenance_margin().to_dec();
        let price_buffer = margin_buffer / (self.quantity.to_dec() * self.contract_size.to_dec());

        let threshold = match self.side {
            Side::Long => self.entry_price.to_dec() - price_buffer,
            Side::Short => self.entry_price.to_dec() + price_buffer,
        };
        liquidation_threshold(threshold, self.side)
    }

    fn is_liquidation_risk(&self, current_price: Positive, _margin_ratio: Decimal) -> bool {
        // No reachable liquidation price means no price is at risk.
        let Some(liq_price) = self.liquidation_price(current_price) else {
            return false;
        };

        match self.side {
            Side::Long => current_price <= liq_price,
            Side::Short => current_price >= liq_price,
        }
    }
}

impl Expirable for FuturePosition {
    fn expiration_timestamp(&self) -> i64 {
        // The resolver rather than `get_date`, which aborts on a day count no
        // calendar instant can hold instead of reporting it. The `unwrap_or`
        // was never reached for that input, because there was nothing to
        // unwrap: the process was already gone.
        resolve_expiration_date(&self.expiration_date)
            .map(|date| date.timestamp())
            .unwrap_or(0)
    }

    fn days_to_expiration(&self) -> Result<Positive, PositionError> {
        let years = self
            .expiration_date
            .get_years()
            .map_err(DecimalError::from)?;
        let days = d_mul(
            years.to_dec(),
            Decimal::from(365),
            "FuturePosition::days_to_expiration",
        )?;
        Ok(Positive::new_decimal(days)?)
    }

    fn is_expired(&self) -> bool {
        resolve_expiration_date(&self.expiration_date)
            .map(|date| date < Utc::now())
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
            // `dec!(30.0)` is a compile-time positive literal; the checked
            // constructor never fails, so the `Positive::ZERO` fallback is
            // unreachable and only exists to keep the call site free of
            // `.unwrap()`/`.expect()`.
            expiration_date: ExpirationDate::Days(
                Positive::new_decimal(dec!(30.0)).unwrap_or(Positive::ZERO),
            ),
            contract_size: positive::Positive::ONE,
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
    use positive::pos_or_panic;

    /// A margin requirement at the top of the range with more than one
    /// contract overflows the product `total_cost` needs. The signature
    /// promises a `Result`, so it has to arrive as one rather than as an
    /// abort inside the raw multiplication `total_margin_required` performs.
    #[test]
    fn test_future_total_cost_reports_a_margin_product_that_overflows() {
        let future = FuturePosition::new(
            "ES".to_string(),
            Positive::TWO,
            pos_or_panic!(4500.0),
            Side::Long,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            Positive::MAX,
            pos_or_panic!(12000.0),
            Utc::now(),
            pos_or_panic!(5.0),
        );

        assert!(future.total_cost().is_err());
    }

    #[test]
    fn test_future_position_new() {
        let future = FuturePosition::new(
            "ES".to_string(),
            Positive::TWO,
            pos_or_panic!(4500.0),
            Side::Long,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
            pos_or_panic!(12000.0),
            Utc::now(),
            pos_or_panic!(5.0),
        );

        assert_eq!(future.symbol, "ES");
        assert_eq!(future.quantity, Positive::TWO);
        assert_eq!(future.entry_price, pos_or_panic!(4500.0));
        assert_eq!(future.side, Side::Long);
        assert_eq!(future.contract_size, pos_or_panic!(50.0));
    }

    #[test]
    fn test_future_long_convenience() {
        let future = FuturePosition::long(
            "ES".to_string(),
            Positive::ONE,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        assert_eq!(future.side, Side::Long);
        assert_eq!(future.initial_margin_req, pos_or_panic!(15000.0));
    }

    #[test]
    fn test_future_short_convenience() {
        let future = FuturePosition::short(
            "ES".to_string(),
            Positive::ONE,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        assert_eq!(future.side, Side::Short);
    }

    #[test]
    fn test_notional_value() {
        let future = FuturePosition::long(
            "ES".to_string(),
            Positive::TWO,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        assert_eq!(future.notional_value_at_entry(), pos_or_panic!(450000.0));
    }

    #[test]
    fn test_unrealized_pnl_long() {
        let future = FuturePosition::long(
            "ES".to_string(),
            Positive::ONE,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        let pnl = future.unrealized_pnl(pos_or_panic!(4510.0));
        assert_eq!(pnl.ok(), Some(Decimal::from(500)));

        let pnl_loss = future.unrealized_pnl(pos_or_panic!(4490.0));
        assert_eq!(pnl_loss.ok(), Some(Decimal::from(-500)));
    }

    #[test]
    fn test_unrealized_pnl_short() {
        let future = FuturePosition::short(
            "ES".to_string(),
            Positive::ONE,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        let pnl = future.unrealized_pnl(pos_or_panic!(4490.0));
        assert_eq!(pnl.ok(), Some(Decimal::from(500)));

        let pnl_loss = future.unrealized_pnl(pos_or_panic!(4510.0));
        assert_eq!(pnl_loss.ok(), Some(Decimal::from(-500)));
    }

    #[test]
    fn test_implied_leverage() {
        let future = FuturePosition::long(
            "ES".to_string(),
            Positive::ONE,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        let leverage = future.implied_leverage();
        assert_eq!(leverage, Decimal::from(15));
    }

    #[test]
    fn test_basis() {
        let future = FuturePosition::long(
            "ES".to_string(),
            Positive::ONE,
            pos_or_panic!(4510.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        let basis = future.basis(pos_or_panic!(4500.0));
        assert_eq!(basis, Decimal::from(10));
    }

    #[test]
    fn test_tick_value() {
        let future = FuturePosition::long(
            "ES".to_string(),
            Positive::ONE,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        let tick_val = future.tick_value(pos_or_panic!(0.25));
        assert_eq!(tick_val, pos_or_panic!(12.5));
    }

    #[test]
    fn test_total_margin_required() {
        let future = FuturePosition::long(
            "ES".to_string(),
            Positive::TWO,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        assert_eq!(future.total_margin_required(), pos_or_panic!(30000.0));
    }

    #[test]
    fn test_display() {
        let future = FuturePosition::long(
            "ES".to_string(),
            Positive::ONE,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        let display = format!("{}", future);
        assert!(display.contains("Long"));
        assert!(display.contains("ES"));
        assert!(display.contains("4500"));
    }

    fn future_with_margins(
        side: Side,
        entry_price: Positive,
        initial_margin_req: Positive,
        maintenance_margin_req: Positive,
    ) -> FuturePosition {
        FuturePosition::new(
            "ES".to_string(),
            Positive::ONE,
            entry_price,
            side,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::ONE,
            initial_margin_req,
            maintenance_margin_req,
            Utc::now(),
            Positive::ZERO,
        )
    }

    #[test]
    fn test_future_liquidation_price_reachable_is_unchanged() {
        let long = FuturePosition::long(
            "ES".to_string(),
            Positive::ONE,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );
        let short = FuturePosition::short(
            "ES".to_string(),
            Positive::ONE,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        );

        // (15000 - 12000) / 50 = 60 of price buffer either side of 4500.
        assert_eq!(
            long.liquidation_price(pos_or_panic!(4500.0)),
            Some(pos_or_panic!(4440.0))
        );
        assert_eq!(
            short.liquidation_price(pos_or_panic!(4500.0)),
            Some(pos_or_panic!(4560.0))
        );
        assert!(long.is_liquidation_risk(pos_or_panic!(4440.0), Decimal::ZERO));
        assert!(!long.is_liquidation_risk(pos_or_panic!(4441.0), Decimal::ZERO));
        assert!(short.is_liquidation_risk(pos_or_panic!(4560.0), Decimal::ZERO));
        assert!(!short.is_liquidation_risk(pos_or_panic!(4559.0), Decimal::ZERO));
    }

    #[test]
    fn test_future_long_liquidation_price_below_zero_is_unreachable() {
        // A buffer of 90 per unit under an entry of 10 puts the threshold at
        // -80: the long survives a fall all the way to zero.
        let long = future_with_margins(
            Side::Long,
            pos_or_panic!(10.0),
            pos_or_panic!(100.0),
            pos_or_panic!(10.0),
        );

        assert_eq!(long.liquidation_price(pos_or_panic!(10.0)), None);
        assert!(!long.is_liquidation_risk(Positive::ZERO, Decimal::ZERO));
        assert!(!long.is_liquidation_risk(pos_or_panic!(10.0), Decimal::ZERO));
    }

    #[test]
    fn test_future_short_liquidation_price_below_zero_floors_at_zero() {
        // A maintenance requirement 90 above the initial one puts the short
        // threshold at -80: every non-negative price already crosses it.
        let short = future_with_margins(
            Side::Short,
            pos_or_panic!(10.0),
            pos_or_panic!(10.0),
            pos_or_panic!(100.0),
        );

        assert_eq!(
            short.liquidation_price(pos_or_panic!(10.0)),
            Some(Positive::ZERO)
        );
        assert!(short.is_liquidation_risk(Positive::ZERO, Decimal::ZERO));
        assert!(short.is_liquidation_risk(pos_or_panic!(10.0), Decimal::ZERO));
    }

    #[test]
    fn test_future_days_to_expiration_is_unchanged() {
        let future = future_with_margins(
            Side::Long,
            pos_or_panic!(4500.0),
            pos_or_panic!(15000.0),
            pos_or_panic!(12000.0),
        );
        let expected = ExpirationDate::Days(pos_or_panic!(30.0))
            .get_years()
            .expect("30 days")
            * Decimal::from(365);

        let days = future.days_to_expiration().expect("30 days");
        assert_eq!(days, expected);
        assert_eq!(
            future.time_to_expiration_years().expect("30 days"),
            expected.to_dec() / Decimal::from(365)
        );
    }

    #[test]
    fn test_future_days_to_expiration_overflow_is_an_error_not_zero() {
        let mut future = future_with_margins(
            Side::Long,
            pos_or_panic!(4500.0),
            pos_or_panic!(15000.0),
            pos_or_panic!(12000.0),
        );
        future.expiration_date = ExpirationDate::Days(Positive::MAX);

        // `get_years` resolves, its product with 365 does not fit `Decimal`.
        let result = future.days_to_expiration();
        assert!(
            matches!(
                result,
                Err(PositionError::DecimalError(DecimalError::Overflow { .. }))
            ),
            "{result:?}"
        );
        assert!(future.time_to_expiration_years().is_err());
    }
}
