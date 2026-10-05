/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
******************************************************************************/
//! Greeks for the leg types, owned by the pricing layer.
//!
//! [`optionstratlib_core::model::leg::LegAble`] describes what a leg *is*: its side,
//! quantity, fees and cost basis. Its Greeks are a different question,
//! answered by a pricing model, and answering it inside `model` made the core
//! layer depend on `greeks` and on [`crate::error::GreeksError`] (roadmap
//! M1-01, ADR-0001 D6). The methods live here instead, as an extension trait
//! the pricing layer owns.
//!
//! Bring [`LegGreeks`] into scope to call them:
//!
//! ```rust
//! use optionstratlib_pricing::greeks::LegGreeks;
//! use optionstratlib_core::model::leg::SpotPosition;
//! use optionstratlib_core::{pos_or_panic, model::Positive};
//!
//! let spot = SpotPosition::long("AAPL".to_string(), Positive::ONE, pos_or_panic!(150.0));
//! // A long spot unit carries one delta per unit and no convexity.
//! assert_eq!(spot.delta()?, Positive::ONE.to_dec());
//! assert_eq!(spot.gamma()?, rust_decimal::Decimal::ZERO);
//! # Ok::<(), optionstratlib_pricing::error::GreeksError>(())
//! ```

use crate::error::GreeksError;
use crate::greeks::Greeks;
use optionstratlib_core::model::Side;
use optionstratlib_core::model::leg::traits::{Expirable, Fundable};
use optionstratlib_core::model::leg::{FuturePosition, Leg, PerpetualPosition, SpotPosition};
use rust_decimal::Decimal;

/// The Greeks of a position leg.
///
/// Linear instruments answer in closed form: a spot unit is one delta and
/// nothing else, a future adds rate sensitivity through its notional, and a
/// perpetual's time decay is its funding payment. Option legs defer to
/// [`Greeks`], the full pricing-model implementation.
///
/// Every method but `delta` defaults to zero, which is the correct answer for
/// an instrument with no exposure to that parameter, so an implementor
/// overrides only what it actually has.
pub trait LegGreeks {
    /// Sensitivity of the position value to the underlying price.
    ///
    /// # Errors
    ///
    /// Propagates any [`GreeksError`] from the underlying pricing kernel,
    /// typically [`GreeksError::Pricing`] for option legs whose Black–Scholes
    /// evaluation fails, or [`GreeksError::ExpirationDate`] when the
    /// expiration cannot be resolved.
    fn delta(&self) -> Result<Decimal, GreeksError>;

    /// Rate of change of delta with the underlying price.
    ///
    /// Zero for linear instruments; only options have convexity.
    ///
    /// # Errors
    ///
    /// The default implementation is infallible. Option legs propagate
    /// [`GreeksError::Pricing`] or [`GreeksError::ExpirationDate`].
    fn gamma(&self) -> Result<Decimal, GreeksError> {
        Ok(Decimal::ZERO)
    }

    /// Sensitivity of the position value to the passage of time.
    ///
    /// # Errors
    ///
    /// The default implementation is infallible. Option legs propagate
    /// [`GreeksError::Pricing`] or [`GreeksError::ExpirationDate`].
    fn theta(&self) -> Result<Decimal, GreeksError> {
        Ok(Decimal::ZERO)
    }

    /// Sensitivity of the position value to implied volatility.
    ///
    /// Zero for linear instruments, which carry no volatility exposure.
    ///
    /// # Errors
    ///
    /// The default implementation is infallible. Option legs propagate
    /// [`GreeksError::Pricing`] or [`GreeksError::ExpirationDate`].
    fn vega(&self) -> Result<Decimal, GreeksError> {
        Ok(Decimal::ZERO)
    }

    /// Sensitivity of the position value to the risk-free rate.
    ///
    /// # Errors
    ///
    /// The default implementation is infallible. Option legs propagate
    /// [`GreeksError::Pricing`] or [`GreeksError::ExpirationDate`].
    fn rho(&self) -> Result<Decimal, GreeksError> {
        Ok(Decimal::ZERO)
    }
}

impl LegGreeks for SpotPosition {
    /// One delta per unit held, signed by the side. A spot position has no
    /// other Greek.
    fn delta(&self) -> Result<Decimal, GreeksError> {
        let delta_per_unit = match self.side {
            Side::Long => Decimal::ONE,
            Side::Short => -Decimal::ONE,
        };
        Ok(delta_per_unit * self.quantity.to_dec())
    }
}

impl LegGreeks for FuturePosition {
    /// One delta per unit of contract size, signed by the side.
    fn delta(&self) -> Result<Decimal, GreeksError> {
        let delta_per_contract = match self.side {
            Side::Long => self.contract_size.to_dec(),
            Side::Short => -self.contract_size.to_dec(),
        };
        Ok(delta_per_contract * self.quantity.to_dec())
    }

    /// Rate sensitivity of the notional over the remaining life, per one
    /// percentage point.
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

impl LegGreeks for PerpetualPosition {
    /// One delta per unit held, signed by the side.
    fn delta(&self) -> Result<Decimal, GreeksError> {
        let delta_per_unit = match self.side {
            Side::Long => Decimal::ONE,
            Side::Short => -Decimal::ONE,
        };
        Ok(delta_per_unit * self.quantity.to_dec())
    }

    /// A perpetual has no expiry, so its carry is the funding payment: what
    /// the holder pays is what the position decays by.
    fn theta(&self) -> Result<Decimal, GreeksError> {
        Ok(-self.funding_payment(self.entry_price))
    }
}

impl LegGreeks for Leg {
    fn delta(&self) -> Result<Decimal, GreeksError> {
        match self {
            Self::Option(pos) => Greeks::delta(pos.as_ref()),
            Self::Spot(pos) => pos.delta(),
            Self::Future(pos) => pos.delta(),
            Self::Perpetual(pos) => pos.delta(),
        }
    }

    fn gamma(&self) -> Result<Decimal, GreeksError> {
        match self {
            Self::Option(pos) => Greeks::gamma(pos.as_ref()),
            Self::Spot(_) | Self::Future(_) | Self::Perpetual(_) => Ok(Decimal::ZERO),
        }
    }

    fn theta(&self) -> Result<Decimal, GreeksError> {
        match self {
            Self::Option(pos) => Greeks::theta(pos.as_ref()),
            Self::Spot(pos) => pos.theta(),
            Self::Future(pos) => pos.theta(),
            Self::Perpetual(pos) => pos.theta(),
        }
    }

    fn vega(&self) -> Result<Decimal, GreeksError> {
        match self {
            Self::Option(pos) => Greeks::vega(pos.as_ref()),
            Self::Spot(_) | Self::Future(_) | Self::Perpetual(_) => Ok(Decimal::ZERO),
        }
    }

    fn rho(&self) -> Result<Decimal, GreeksError> {
        match self {
            Self::Option(pos) => Greeks::rho(pos.as_ref()),
            Self::Spot(pos) => pos.rho(),
            Self::Future(pos) => pos.rho(),
            Self::Perpetual(pos) => pos.rho(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn spot_long() -> SpotPosition {
        SpotPosition::long("AAPL".to_string(), Positive::HUNDRED, pos_or_panic!(150.0))
    }

    fn future(long: bool) -> FuturePosition {
        let build = if long {
            FuturePosition::long
        } else {
            FuturePosition::short
        };
        build(
            "ES".to_string(),
            Positive::TWO,
            pos_or_panic!(4500.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(15000.0),
        )
    }

    fn perpetual(long: bool) -> PerpetualPosition {
        let build = if long {
            PerpetualPosition::long
        } else {
            PerpetualPosition::short
        };
        build(
            "BTC-USDT-PERP".to_string(),
            Positive::TWO,
            pos_or_panic!(50000.0),
            pos_or_panic!(10.0),
            pos_or_panic!(10000.0),
        )
    }

    fn ok(result: Result<Decimal, GreeksError>) -> Decimal {
        match result {
            Ok(value) => value,
            Err(error) => panic!("greek must compute: {error}"),
        }
    }

    #[test]
    fn test_spot_delta_is_one_per_unit_signed_by_side() {
        assert_eq!(ok(spot_long().delta()), Decimal::from(100));
        let short =
            SpotPosition::short("AAPL".to_string(), Positive::HUNDRED, pos_or_panic!(150.0));
        assert_eq!(ok(short.delta()), Decimal::from(-100));
    }

    #[test]
    fn test_spot_has_no_other_greek() {
        let spot = spot_long();
        assert_eq!(ok(spot.gamma()), Decimal::ZERO);
        assert_eq!(ok(spot.theta()), Decimal::ZERO);
        assert_eq!(ok(spot.vega()), Decimal::ZERO);
        assert_eq!(ok(spot.rho()), Decimal::ZERO);
    }

    #[test]
    fn test_future_delta_scales_with_contract_size() {
        assert_eq!(ok(future(true).delta()), Decimal::from(100));
        assert_eq!(ok(future(false).delta()), Decimal::from(-100));
    }

    #[test]
    fn test_future_rho_is_signed_by_side() {
        let long = ok(future(true).rho());
        let short = ok(future(false).rho());
        assert!(
            long > Decimal::ZERO,
            "a long future gains with rates: {long}"
        );
        assert_eq!(long, -short);
    }

    #[test]
    fn test_perpetual_delta_is_one_per_unit_signed_by_side() {
        assert_eq!(ok(perpetual(true).delta()), dec!(2));
        assert_eq!(ok(perpetual(false).delta()), dec!(-2));
    }

    #[test]
    fn test_perpetual_theta_is_the_negated_funding_payment() {
        let perp = perpetual(true);
        let expected = -perp.funding_payment(perp.entry_price);
        assert_eq!(ok(perp.theta()), expected);
    }

    #[test]
    fn test_leg_forwards_to_the_wrapped_position() {
        assert_eq!(ok(Leg::spot(spot_long()).delta()), Decimal::from(100));
        let short =
            SpotPosition::short("AAPL".to_string(), Positive::HUNDRED, pos_or_panic!(150.0));
        assert_eq!(ok(Leg::spot(short).delta()), Decimal::from(-100));
    }

    /// Only option legs have convexity; the three linear ones answer zero.
    #[test]
    fn test_leg_gamma_is_zero_for_every_linear_leg() {
        assert_eq!(ok(Leg::spot(spot_long()).gamma()), Decimal::ZERO);
        assert_eq!(ok(Leg::future(future(true)).gamma()), Decimal::ZERO);
        assert_eq!(ok(Leg::perpetual(perpetual(true)).gamma()), Decimal::ZERO);
    }
}
