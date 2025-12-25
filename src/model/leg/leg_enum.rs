/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Leg Enum Module
//!
//! This module provides the `Leg` enum that unifies different position types
//! (options, spot, futures, perpetuals) into a single type for use in
//! multi-instrument strategies.
//!
//! ## Supported Leg Types
//!
//! - `Option` - Standard option positions (Call/Put)
//! - `Spot` - Direct ownership of underlying assets
//! - `Future` - Exchange-traded futures contracts
//! - `Perpetual` - Crypto perpetual swap contracts
//!
//! ## Example
//!
//! ```rust
//! use optionstratlib::model::leg::{Leg, SpotPosition};
//! use optionstratlib::model::Position;
//! use optionstratlib::model::types::Side;
//! use optionstratlib::pos;
//!
//! // Create a spot leg
//! let spot = SpotPosition::long("AAPL".to_string(), pos!(100.0), pos!(150.0));
//! let spot_leg = Leg::Spot(spot);
//!
//! // Check leg type
//! assert!(spot_leg.is_spot());
//! ```

use crate::Positive;
use crate::error::GreeksError;
use crate::model::leg::future::FuturePosition;
use crate::model::leg::perpetual::PerpetualPosition;
use crate::model::leg::spot::SpotPosition;
use crate::model::leg::traits::LegAble;
use crate::model::position::Position;
use crate::model::types::Side;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents different types of legs in a trading strategy.
///
/// This enum provides a unified interface for handling various instrument types
/// in multi-leg strategies, enabling strategies like Covered Call (spot + option),
/// Cash & Carry (spot + perpetual), and other combinations.
///
/// # Variants
///
/// * `Option` - Standard option position (Call or Put)
/// * `Spot` - Direct ownership of underlying asset
/// * `Future` - Exchange-traded futures contract
/// * `Perpetual` - Crypto perpetual swap contract
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum Leg {
    /// Standard option position (Call/Put).
    Option(Position),

    /// Spot/underlying asset position.
    Spot(SpotPosition),

    /// Exchange-traded futures contract.
    Future(FuturePosition),

    /// Crypto perpetual swap contract.
    Perpetual(PerpetualPosition),
}

impl Leg {
    /// Creates a new option leg from a Position.
    #[must_use]
    pub fn option(position: Position) -> Self {
        Self::Option(position)
    }

    /// Creates a new spot leg from a SpotPosition.
    #[must_use]
    pub fn spot(position: SpotPosition) -> Self {
        Self::Spot(position)
    }

    /// Creates a new future leg from a FuturePosition.
    #[must_use]
    pub fn future(position: FuturePosition) -> Self {
        Self::Future(position)
    }

    /// Creates a new perpetual leg from a PerpetualPosition.
    #[must_use]
    pub fn perpetual(position: PerpetualPosition) -> Self {
        Self::Perpetual(position)
    }

    /// Returns true if this is an option leg.
    #[must_use]
    pub fn is_option(&self) -> bool {
        matches!(self, Self::Option(_))
    }

    /// Returns true if this is a spot leg.
    #[must_use]
    pub fn is_spot(&self) -> bool {
        matches!(self, Self::Spot(_))
    }

    /// Returns true if this is a future leg.
    #[must_use]
    pub fn is_future(&self) -> bool {
        matches!(self, Self::Future(_))
    }

    /// Returns true if this is a perpetual leg.
    #[must_use]
    pub fn is_perpetual(&self) -> bool {
        matches!(self, Self::Perpetual(_))
    }

    /// Returns true if this is a linear instrument (spot, future, perpetual).
    ///
    /// Linear instruments have constant delta and zero gamma.
    #[must_use]
    pub fn is_linear(&self) -> bool {
        !self.is_option()
    }

    /// Returns true if this is a derivative (option, future, perpetual).
    #[must_use]
    pub fn is_derivative(&self) -> bool {
        !self.is_spot()
    }

    /// Returns true if this leg has margin requirements.
    #[must_use]
    pub fn is_margined(&self) -> bool {
        matches!(self, Self::Future(_) | Self::Perpetual(_))
    }

    /// Returns true if this leg has an expiration date.
    #[must_use]
    pub fn has_expiration(&self) -> bool {
        matches!(self, Self::Option(_) | Self::Future(_))
    }

    /// Returns the underlying option position if this is an Option leg.
    #[must_use]
    pub fn as_option(&self) -> Option<&Position> {
        match self {
            Self::Option(pos) => Some(pos),
            _ => None,
        }
    }

    /// Returns the underlying spot position if this is a Spot leg.
    #[must_use]
    pub fn as_spot(&self) -> Option<&SpotPosition> {
        match self {
            Self::Spot(pos) => Some(pos),
            _ => None,
        }
    }

    /// Returns the underlying future position if this is a Future leg.
    #[must_use]
    pub fn as_future(&self) -> Option<&FuturePosition> {
        match self {
            Self::Future(pos) => Some(pos),
            _ => None,
        }
    }

    /// Returns the underlying perpetual position if this is a Perpetual leg.
    #[must_use]
    pub fn as_perpetual(&self) -> Option<&PerpetualPosition> {
        match self {
            Self::Perpetual(pos) => Some(pos),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying option position.
    #[must_use]
    pub fn as_option_mut(&mut self) -> Option<&mut Position> {
        match self {
            Self::Option(pos) => Some(pos),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying spot position.
    #[must_use]
    pub fn as_spot_mut(&mut self) -> Option<&mut SpotPosition> {
        match self {
            Self::Spot(pos) => Some(pos),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying future position.
    #[must_use]
    pub fn as_future_mut(&mut self) -> Option<&mut FuturePosition> {
        match self {
            Self::Future(pos) => Some(pos),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying perpetual position.
    #[must_use]
    pub fn as_perpetual_mut(&mut self) -> Option<&mut PerpetualPosition> {
        match self {
            Self::Perpetual(pos) => Some(pos),
            _ => None,
        }
    }

    /// Returns a string describing the leg type.
    #[must_use]
    pub fn leg_type_name(&self) -> &'static str {
        match self {
            Self::Option(_) => "Option",
            Self::Spot(_) => "Spot",
            Self::Future(_) => "Future",
            Self::Perpetual(_) => "Perpetual",
        }
    }
}

impl LegAble for Leg {
    fn get_symbol(&self) -> &str {
        match self {
            Self::Option(pos) => &pos.option.underlying_symbol,
            Self::Spot(pos) => pos.get_symbol(),
            Self::Future(pos) => pos.get_symbol(),
            Self::Perpetual(pos) => pos.get_symbol(),
        }
    }

    fn get_quantity(&self) -> Positive {
        match self {
            Self::Option(pos) => pos.option.quantity,
            Self::Spot(pos) => pos.get_quantity(),
            Self::Future(pos) => pos.get_quantity(),
            Self::Perpetual(pos) => pos.get_quantity(),
        }
    }

    fn get_side(&self) -> Side {
        match self {
            Self::Option(pos) => pos.option.side,
            Self::Spot(pos) => pos.get_side(),
            Self::Future(pos) => pos.get_side(),
            Self::Perpetual(pos) => pos.get_side(),
        }
    }

    fn pnl_at_price(&self, price: Positive) -> Decimal {
        match self {
            Self::Option(pos) => pos
                .pnl_at_expiration(&Some(&price))
                .unwrap_or(Decimal::ZERO),
            Self::Spot(pos) => pos.pnl_at_price(price),
            Self::Future(pos) => pos.pnl_at_price(price),
            Self::Perpetual(pos) => pos.pnl_at_price(price),
        }
    }

    fn total_cost(&self) -> Positive {
        match self {
            Self::Option(pos) => pos.total_cost().unwrap_or(Positive::ZERO),
            Self::Spot(pos) => pos.total_cost(),
            Self::Future(pos) => pos.total_cost(),
            Self::Perpetual(pos) => pos.total_cost(),
        }
    }

    fn fees(&self) -> Positive {
        match self {
            Self::Option(pos) => pos.open_fee + pos.close_fee,
            Self::Spot(pos) => pos.fees(),
            Self::Future(pos) => pos.fees(),
            Self::Perpetual(pos) => pos.fees(),
        }
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        match self {
            Self::Option(pos) => {
                use crate::greeks::Greeks;
                pos.delta()
            }
            Self::Spot(pos) => pos.delta(),
            Self::Future(pos) => pos.delta(),
            Self::Perpetual(pos) => pos.delta(),
        }
    }

    fn gamma(&self) -> Result<Decimal, GreeksError> {
        match self {
            Self::Option(pos) => {
                use crate::greeks::Greeks;
                pos.gamma()
            }
            Self::Spot(_) | Self::Future(_) | Self::Perpetual(_) => Ok(Decimal::ZERO),
        }
    }

    fn theta(&self) -> Result<Decimal, GreeksError> {
        match self {
            Self::Option(pos) => {
                use crate::greeks::Greeks;
                pos.theta()
            }
            Self::Spot(pos) => pos.theta(),
            Self::Future(pos) => pos.theta(),
            Self::Perpetual(pos) => pos.theta(),
        }
    }

    fn vega(&self) -> Result<Decimal, GreeksError> {
        match self {
            Self::Option(pos) => {
                use crate::greeks::Greeks;
                pos.vega()
            }
            Self::Spot(_) | Self::Future(_) | Self::Perpetual(_) => Ok(Decimal::ZERO),
        }
    }

    fn rho(&self) -> Result<Decimal, GreeksError> {
        match self {
            Self::Option(pos) => {
                use crate::greeks::Greeks;
                pos.rho()
            }
            Self::Spot(pos) => pos.rho(),
            Self::Future(pos) => pos.rho(),
            Self::Perpetual(pos) => pos.rho(),
        }
    }
}

impl std::fmt::Display for Leg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Option(pos) => write!(f, "[Option] {}", pos),
            Self::Spot(pos) => write!(f, "[Spot] {}", pos),
            Self::Future(pos) => write!(f, "[Future] {}", pos),
            Self::Perpetual(pos) => write!(f, "[Perpetual] {}", pos),
        }
    }
}

impl From<Position> for Leg {
    fn from(position: Position) -> Self {
        Self::Option(position)
    }
}

impl From<SpotPosition> for Leg {
    fn from(position: SpotPosition) -> Self {
        Self::Spot(position)
    }
}

impl From<FuturePosition> for Leg {
    fn from(position: FuturePosition) -> Self {
        Self::Future(position)
    }
}

impl From<PerpetualPosition> for Leg {
    fn from(position: PerpetualPosition) -> Self {
        Self::Perpetual(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OptionStyle;
    use crate::model::ExpirationDate;
    use crate::model::utils::create_sample_option_simplest;

    use chrono::Utc;

    fn create_test_option_position() -> Position {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        Position::new(
            option,
            pos!(5.0),
            Utc::now(),
            pos!(0.5),
            pos!(0.5),
            None,
            None,
        )
    }

    fn create_test_spot_position() -> SpotPosition {
        SpotPosition::long("AAPL".to_string(), pos!(100.0), pos!(150.0))
    }

    fn create_test_future_position() -> FuturePosition {
        FuturePosition::long(
            "ES".to_string(),
            pos!(1.0),
            pos!(4500.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(50.0),
            pos!(15000.0),
        )
    }

    fn create_test_perpetual_position() -> PerpetualPosition {
        PerpetualPosition::long(
            "BTC-USDT-PERP".to_string(),
            pos!(1.0),
            pos!(50000.0),
            pos!(10.0),
            pos!(5000.0),
        )
    }

    #[test]
    fn test_leg_option_creation() {
        let pos = create_test_option_position();
        let leg = Leg::option(pos.clone());

        assert!(leg.is_option());
        assert!(!leg.is_spot());
        assert!(!leg.is_future());
        assert!(!leg.is_perpetual());
        assert!(!leg.is_linear());
        assert!(leg.is_derivative());
        assert!(leg.has_expiration());
    }

    #[test]
    fn test_leg_spot_creation() {
        let pos = create_test_spot_position();
        let leg = Leg::spot(pos);

        assert!(!leg.is_option());
        assert!(leg.is_spot());
        assert!(!leg.is_future());
        assert!(!leg.is_perpetual());
        assert!(leg.is_linear());
        assert!(!leg.is_derivative());
        assert!(!leg.has_expiration());
        assert!(!leg.is_margined());
    }

    #[test]
    fn test_leg_future_creation() {
        let pos = create_test_future_position();
        let leg = Leg::future(pos);

        assert!(!leg.is_option());
        assert!(!leg.is_spot());
        assert!(leg.is_future());
        assert!(!leg.is_perpetual());
        assert!(leg.is_linear());
        assert!(leg.is_derivative());
        assert!(leg.has_expiration());
        assert!(leg.is_margined());
    }

    #[test]
    fn test_leg_perpetual_creation() {
        let pos = create_test_perpetual_position();
        let leg = Leg::perpetual(pos);

        assert!(!leg.is_option());
        assert!(!leg.is_spot());
        assert!(!leg.is_future());
        assert!(leg.is_perpetual());
        assert!(leg.is_linear());
        assert!(leg.is_derivative());
        assert!(!leg.has_expiration());
        assert!(leg.is_margined());
    }

    #[test]
    fn test_leg_from_position() {
        let pos = create_test_option_position();
        let leg: Leg = pos.into();
        assert!(leg.is_option());
    }

    #[test]
    fn test_leg_from_spot() {
        let pos = create_test_spot_position();
        let leg: Leg = pos.into();
        assert!(leg.is_spot());
    }

    #[test]
    fn test_leg_from_future() {
        let pos = create_test_future_position();
        let leg: Leg = pos.into();
        assert!(leg.is_future());
    }

    #[test]
    fn test_leg_from_perpetual() {
        let pos = create_test_perpetual_position();
        let leg: Leg = pos.into();
        assert!(leg.is_perpetual());
    }

    #[test]
    fn test_leg_as_option() {
        let pos = create_test_option_position();
        let leg = Leg::option(pos);

        assert!(leg.as_option().is_some());
        assert!(leg.as_spot().is_none());
        assert!(leg.as_future().is_none());
        assert!(leg.as_perpetual().is_none());
    }

    #[test]
    fn test_leg_as_spot() {
        let pos = create_test_spot_position();
        let leg = Leg::spot(pos);

        assert!(leg.as_option().is_none());
        assert!(leg.as_spot().is_some());
        assert!(leg.as_future().is_none());
        assert!(leg.as_perpetual().is_none());
    }

    #[test]
    fn test_leg_get_symbol() {
        let spot = create_test_spot_position();
        let leg = Leg::spot(spot);
        assert_eq!(leg.get_symbol(), "AAPL");

        let future = create_test_future_position();
        let leg = Leg::future(future);
        assert_eq!(leg.get_symbol(), "ES");

        let perp = create_test_perpetual_position();
        let leg = Leg::perpetual(perp);
        assert_eq!(leg.get_symbol(), "BTC-USDT-PERP");
    }

    #[test]
    fn test_leg_get_quantity() {
        let spot = SpotPosition::long("AAPL".to_string(), pos!(100.0), pos!(150.0));
        let leg = Leg::spot(spot);
        assert_eq!(leg.get_quantity(), pos!(100.0));
    }

    #[test]
    fn test_leg_get_side() {
        let long_spot = SpotPosition::long("AAPL".to_string(), pos!(100.0), pos!(150.0));
        let leg = Leg::spot(long_spot);
        assert_eq!(leg.get_side(), Side::Long);

        let short_spot = SpotPosition::short("AAPL".to_string(), pos!(100.0), pos!(150.0));
        let leg = Leg::spot(short_spot);
        assert_eq!(leg.get_side(), Side::Short);
    }

    #[test]
    fn test_leg_delta_spot() {
        let long_spot = SpotPosition::long("AAPL".to_string(), pos!(100.0), pos!(150.0));
        let leg = Leg::spot(long_spot);
        assert_eq!(leg.delta().unwrap(), Decimal::from(100));

        let short_spot = SpotPosition::short("AAPL".to_string(), pos!(100.0), pos!(150.0));
        let leg = Leg::spot(short_spot);
        assert_eq!(leg.delta().unwrap(), Decimal::from(-100));
    }

    #[test]
    fn test_leg_gamma_linear() {
        let spot = create_test_spot_position();
        let leg = Leg::spot(spot);
        assert_eq!(leg.gamma().unwrap(), Decimal::ZERO);

        let future = create_test_future_position();
        let leg = Leg::future(future);
        assert_eq!(leg.gamma().unwrap(), Decimal::ZERO);

        let perp = create_test_perpetual_position();
        let leg = Leg::perpetual(perp);
        assert_eq!(leg.gamma().unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_leg_pnl_spot() {
        let spot = SpotPosition::long("AAPL".to_string(), pos!(100.0), pos!(150.0));
        let leg = Leg::spot(spot);

        let pnl = leg.pnl_at_price(pos!(160.0));
        assert_eq!(pnl, Decimal::from(1000));
    }

    #[test]
    fn test_leg_type_name() {
        let option_leg = Leg::option(create_test_option_position());
        assert_eq!(option_leg.leg_type_name(), "Option");

        let spot_leg = Leg::spot(create_test_spot_position());
        assert_eq!(spot_leg.leg_type_name(), "Spot");

        let future_leg = Leg::future(create_test_future_position());
        assert_eq!(future_leg.leg_type_name(), "Future");

        let perp_leg = Leg::perpetual(create_test_perpetual_position());
        assert_eq!(perp_leg.leg_type_name(), "Perpetual");
    }

    #[test]
    fn test_leg_display() {
        let spot = create_test_spot_position();
        let leg = Leg::spot(spot);
        let display = format!("{}", leg);
        assert!(display.contains("[Spot]"));
        assert!(display.contains("AAPL"));
    }

    #[test]
    fn test_leg_total_cost() {
        let spot = SpotPosition::new(
            "AAPL".to_string(),
            pos!(100.0),
            pos!(150.0),
            Side::Long,
            Utc::now(),
            pos!(10.0),
            pos!(10.0),
        );
        let leg = Leg::spot(spot);
        assert_eq!(leg.total_cost(), pos!(15020.0));
    }

    #[test]
    fn test_leg_fees() {
        let spot = SpotPosition::new(
            "AAPL".to_string(),
            pos!(100.0),
            pos!(150.0),
            Side::Long,
            Utc::now(),
            pos!(10.0),
            pos!(15.0),
        );
        let leg = Leg::spot(spot);
        assert_eq!(leg.fees(), pos!(25.0));
    }
}
