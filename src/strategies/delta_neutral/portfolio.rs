/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Portfolio-Level Greeks Module
//!
//! Provides aggregated Greeks calculations at the portfolio level,
//! enabling risk management across multiple positions and strategies.
//!
//! ## Overview
//!
//! This module provides:
//!
//! - [`PortfolioGreeks`]: Aggregated Greeks for a collection of positions
//! - [`AdjustmentTarget`]: Target Greeks for adjustment optimization
//!
//! ## Usage
//!
//! ```ignore
//! use optionstratlib::strategies::delta_neutral::portfolio::PortfolioGreeks;
//!
//! let greeks = PortfolioGreeks::from_positions(&positions)?;
//! if !greeks.is_delta_neutral(dec!(0.01)) {
//!     // Need to adjust
//! }
//! ```

use crate::error::GreeksError;
use crate::greeks::Greeks;
use crate::model::decimal::{d_add, d_sub};
use crate::model::position::Position;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

/// Aggregated Greeks at portfolio level.
///
/// This structure holds the combined Greeks for a collection of positions,
/// enabling portfolio-level risk analysis and management.
///
/// ## Fields
///
/// All fields represent the sum of individual position Greeks, accounting
/// for position size and direction (long/short).
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PortfolioGreeks {
    /// Net delta exposure (sensitivity to underlying price)
    pub delta: Decimal,
    /// Net gamma exposure (rate of delta change)
    pub gamma: Decimal,
    /// Net theta exposure (time decay per day)
    pub theta: Decimal,
    /// Net vega exposure (sensitivity to volatility)
    pub vega: Decimal,
    /// Net rho exposure (sensitivity to interest rates)
    pub rho: Decimal,
}

impl PortfolioGreeks {
    /// Creates a new PortfolioGreeks with specified values.
    #[inline]
    #[must_use]
    pub fn new(
        delta: Decimal,
        gamma: Decimal,
        theta: Decimal,
        vega: Decimal,
        rho: Decimal,
    ) -> Self {
        Self {
            delta,
            gamma,
            theta,
            vega,
            rho,
        }
    }

    /// Calculates aggregated Greeks from a set of positions.
    ///
    /// # Arguments
    ///
    /// * `positions` - Slice of Position references to aggregate
    ///
    /// # Returns
    ///
    /// * `Ok(PortfolioGreeks)` - Aggregated Greeks
    /// * `Err(GreeksError)` - If any Greek calculation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let positions = strategy.get_positions()?;
    /// let greeks = PortfolioGreeks::from_positions(&positions)?;
    /// info!("Portfolio delta: {}", greeks.delta);
    /// ```
    ///
    /// # Errors
    ///
    /// Propagates any [`GreeksError`] returned by [`Greeks::delta`],
    /// [`Greeks::gamma`], [`Greeks::theta`], [`Greeks::vega`] or
    /// [`Greeks::rho`] on the input positions — typically
    /// [`GreeksError::Pricing`] when Black–Scholes fails on a leg — and
    /// [`GreeksError::CalculationError`] with [`crate::error::greeks::CalculationErrorKind::DecimalError`] when the running total of a greek leaves
    /// the `Decimal` range.
    pub fn from_positions(positions: &[Position]) -> Result<Self, GreeksError> {
        let mut greeks = Self::default();

        for pos in positions {
            // `Position` implements `Greeks`, and every greek already carries
            // the leg's `Side` and `quantity`. Re-applying a `quantity * sign`
            // multiplier here squared the size and cancelled the sign.
            greeks.delta = d_add(greeks.delta, pos.delta()?, "PortfolioGreeks::delta")?;
            greeks.gamma = d_add(greeks.gamma, pos.gamma()?, "PortfolioGreeks::gamma")?;
            greeks.theta = d_add(greeks.theta, pos.theta()?, "PortfolioGreeks::theta")?;
            greeks.vega = d_add(greeks.vega, pos.vega()?, "PortfolioGreeks::vega")?;
            greeks.rho = d_add(greeks.rho, pos.rho()?, "PortfolioGreeks::rho")?;
        }

        Ok(greeks)
    }

    /// Calculates aggregated Greeks from a set of positions with underlying.
    ///
    /// # Arguments
    ///
    /// * `positions` - Slice of Position references to aggregate
    /// * `underlying_quantity` - Quantity of underlying shares (negative for short)
    ///
    /// # Returns
    ///
    /// * `Ok(PortfolioGreeks)` - Aggregated Greeks including underlying
    ///
    /// # Errors
    ///
    /// Same failure surface as [`PortfolioGreeks::from_positions`], plus
    /// [`GreeksError::CalculationError`] with [`crate::error::greeks::CalculationErrorKind::DecimalError`] when adding the underlying delta leaves
    /// the `Decimal` range.
    pub fn from_positions_with_underlying(
        positions: &[Position],
        underlying_quantity: Decimal,
    ) -> Result<Self, GreeksError> {
        let mut greeks = Self::from_positions(positions)?;
        // Each share of underlying has delta = 1
        greeks.delta = d_add(
            greeks.delta,
            underlying_quantity,
            "PortfolioGreeks::from_positions_with_underlying",
        )?;
        Ok(greeks)
    }

    /// Checks if the portfolio is approximately delta neutral.
    ///
    /// # Arguments
    ///
    /// * `tolerance` - Maximum absolute delta value to consider neutral
    ///
    /// # Returns
    ///
    /// `true` if absolute delta is within tolerance
    #[inline]
    #[must_use]
    pub fn is_delta_neutral(&self, tolerance: Decimal) -> bool {
        self.delta.abs() <= tolerance
    }

    /// Checks if the portfolio is approximately gamma neutral.
    ///
    /// # Arguments
    ///
    /// * `tolerance` - Maximum absolute gamma value to consider neutral
    ///
    /// # Returns
    ///
    /// `true` if absolute gamma is within tolerance
    #[inline]
    #[must_use]
    pub fn is_gamma_neutral(&self, tolerance: Decimal) -> bool {
        self.gamma.abs() <= tolerance
    }

    /// Checks if the portfolio is approximately vega neutral.
    ///
    /// # Arguments
    ///
    /// * `tolerance` - Maximum absolute vega value to consider neutral
    ///
    /// # Returns
    ///
    /// `true` if absolute vega is within tolerance
    #[inline]
    #[must_use]
    pub fn is_vega_neutral(&self, tolerance: Decimal) -> bool {
        self.vega.abs() <= tolerance
    }

    /// Returns the delta gap from a target value.
    ///
    /// # Arguments
    ///
    /// * `target` - Target delta value
    ///
    /// # Returns
    ///
    /// The difference between current delta and target
    ///
    /// # Decision (issue #471): return `Result`
    ///
    /// All five Greek fields on this struct are `pub` and are written
    /// directly by callers that aggregate Greeks from their own sources, so
    /// [`PortfolioGreeks::from_positions`] guarantees nothing about the value
    /// this subtracts from. A `Decimal` gap has no sentinel for "did not
    /// fit", so the error channel is the only place the overflow can be
    /// reported.
    ///
    /// # Errors
    ///
    /// Returns [`GreeksError::CalculationError`] when the difference leaves the
    /// representable `Decimal` range.
    #[inline]
    pub fn delta_gap(&self, target: Decimal) -> Result<Decimal, GreeksError> {
        Ok(d_sub(target, self.delta, "PortfolioGreeks::delta_gap")?)
    }

    /// Returns the gamma gap from a target value.
    ///
    /// # Arguments
    ///
    /// * `target` - Target gamma value
    ///
    /// # Returns
    ///
    /// The difference between current gamma and target
    ///
    /// # Errors
    ///
    /// Returns [`GreeksError::CalculationError`] when the difference leaves the
    /// representable `Decimal` range. See [`PortfolioGreeks::delta_gap`] for
    /// why the `pub` fields make that reachable.
    #[inline]
    pub fn gamma_gap(&self, target: Decimal) -> Result<Decimal, GreeksError> {
        Ok(d_sub(target, self.gamma, "PortfolioGreeks::gamma_gap")?)
    }

    /// Adds another PortfolioGreeks to this one.
    ///
    /// Useful for combining Greeks from multiple sources.
    ///
    /// On failure `self` is left unchanged: the five sums are computed into
    /// a temporary before any field is written, so a portfolio is never left
    /// holding a partially applied aggregation.
    ///
    /// # Errors
    ///
    /// Returns [`GreeksError::CalculationError`] when any of the five sums leaves the
    /// representable `Decimal` range. See [`PortfolioGreeks::delta_gap`] for
    /// why the `pub` fields make that reachable.
    #[inline]
    pub fn add(&mut self, other: &PortfolioGreeks) -> Result<(), GreeksError> {
        *self = self.combined(other)?;
        Ok(())
    }

    /// Returns a new PortfolioGreeks that is the sum of this and another.
    ///
    /// # Errors
    ///
    /// Returns [`GreeksError::CalculationError`] when any of the five sums leaves the
    /// representable `Decimal` range. See [`PortfolioGreeks::delta_gap`] for
    /// why the `pub` fields make that reachable.
    #[inline]
    pub fn combined(&self, other: &PortfolioGreeks) -> Result<PortfolioGreeks, GreeksError> {
        Ok(PortfolioGreeks {
            delta: d_add(self.delta, other.delta, "PortfolioGreeks::combined/delta")?,
            gamma: d_add(self.gamma, other.gamma, "PortfolioGreeks::combined/gamma")?,
            theta: d_add(self.theta, other.theta, "PortfolioGreeks::combined/theta")?,
            vega: d_add(self.vega, other.vega, "PortfolioGreeks::combined/vega")?,
            rho: d_add(self.rho, other.rho, "PortfolioGreeks::combined/rho")?,
        })
    }
}

impl fmt::Display for PortfolioGreeks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Portfolio Greeks:")?;
        writeln!(f, "  Delta: {:.4}", self.delta)?;
        writeln!(f, "  Gamma: {:.6}", self.gamma)?;
        writeln!(f, "  Theta: {:.4}", self.theta)?;
        writeln!(f, "  Vega:  {:.4}", self.vega)?;
        writeln!(f, "  Rho:   {:.4}", self.rho)?;
        Ok(())
    }
}

/// Target Greeks for adjustment optimization.
///
/// Specifies the desired Greek values after adjustment. Use `None` for
/// Greeks that should not be targeted (i.e., only optimize for specified Greeks).
///
/// ## Common Targets
///
/// - Delta neutral: `AdjustmentTarget::delta_neutral()`
/// - Delta-gamma neutral: `AdjustmentTarget::delta_gamma_neutral()`
/// - Custom: Use builder methods
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AdjustmentTarget {
    /// Target delta value (None = don't optimize for delta)
    pub delta: Option<Decimal>,
    /// Target gamma value (None = don't optimize for gamma)
    pub gamma: Option<Decimal>,
    /// Target vega value (None = don't optimize for vega)
    pub vega: Option<Decimal>,
    /// Target theta value (None = don't optimize for theta)
    pub theta: Option<Decimal>,
}

impl AdjustmentTarget {
    /// Creates a target for delta neutrality only.
    ///
    /// # Returns
    ///
    /// An AdjustmentTarget with delta = 0 and other Greeks unconstrained
    #[inline]
    #[must_use]
    pub fn delta_neutral() -> Self {
        Self {
            delta: Some(Decimal::ZERO),
            ..Default::default()
        }
    }

    /// Creates a target for delta and gamma neutrality.
    ///
    /// # Returns
    ///
    /// An AdjustmentTarget with delta = 0 and gamma = 0
    #[inline]
    #[must_use]
    pub fn delta_gamma_neutral() -> Self {
        Self {
            delta: Some(Decimal::ZERO),
            gamma: Some(Decimal::ZERO),
            ..Default::default()
        }
    }

    /// Creates a target for delta, gamma, and vega neutrality.
    ///
    /// # Returns
    ///
    /// An AdjustmentTarget with delta = 0, gamma = 0, and vega = 0
    #[inline]
    #[must_use]
    pub fn full_neutral() -> Self {
        Self {
            delta: Some(Decimal::ZERO),
            gamma: Some(Decimal::ZERO),
            vega: Some(Decimal::ZERO),
            theta: None,
        }
    }

    /// Creates a custom target with specified delta.
    ///
    /// # Arguments
    ///
    /// * `delta` - Target delta value
    #[inline]
    #[must_use]
    pub fn with_delta(mut self, delta: Decimal) -> Self {
        self.delta = Some(delta);
        self
    }

    /// Creates a custom target with specified gamma.
    ///
    /// # Arguments
    ///
    /// * `gamma` - Target gamma value
    #[inline]
    #[must_use]
    pub fn with_gamma(mut self, gamma: Decimal) -> Self {
        self.gamma = Some(gamma);
        self
    }

    /// Creates a custom target with specified vega.
    ///
    /// # Arguments
    ///
    /// * `vega` - Target vega value
    #[inline]
    #[must_use]
    pub fn with_vega(mut self, vega: Decimal) -> Self {
        self.vega = Some(vega);
        self
    }

    /// Creates a custom target with specified theta.
    ///
    /// # Arguments
    ///
    /// * `theta` - Target theta value
    #[inline]
    #[must_use]
    pub fn with_theta(mut self, theta: Decimal) -> Self {
        self.theta = Some(theta);
        self
    }

    /// Calculates the delta gap from current Greeks.
    ///
    /// # Arguments
    ///
    /// * `current` - Current portfolio Greeks
    ///
    /// # Returns
    ///
    /// The delta gap if delta target is set, otherwise zero
    #[inline]
    #[must_use]
    pub fn delta_gap(&self, current: &PortfolioGreeks) -> Decimal {
        self.delta
            .map(|t| t - current.delta)
            .unwrap_or(Decimal::ZERO)
    }

    /// Calculates the gamma gap from current Greeks.
    ///
    /// # Arguments
    ///
    /// * `current` - Current portfolio Greeks
    ///
    /// # Returns
    ///
    /// The gamma gap if gamma target is set, otherwise None
    #[inline]
    #[must_use]
    pub fn gamma_gap(&self, current: &PortfolioGreeks) -> Option<Decimal> {
        self.gamma.map(|t| t - current.gamma)
    }

    /// Calculates the vega gap from current Greeks.
    ///
    /// # Arguments
    ///
    /// * `current` - Current portfolio Greeks
    ///
    /// # Returns
    ///
    /// The vega gap if vega target is set, otherwise None
    #[inline]
    #[must_use]
    pub fn vega_gap(&self, current: &PortfolioGreeks) -> Option<Decimal> {
        self.vega.map(|t| t - current.vega)
    }

    /// Checks if the current Greeks meet all targets within tolerance.
    ///
    /// # Arguments
    ///
    /// * `current` - Current portfolio Greeks
    /// * `tolerance` - Maximum deviation from target
    ///
    /// # Returns
    ///
    /// `true` if all specified targets are met within tolerance
    #[must_use]
    pub fn is_satisfied(&self, current: &PortfolioGreeks, tolerance: Decimal) -> bool {
        let delta_ok = self
            .delta
            .map(|t| (current.delta - t).abs() <= tolerance)
            .unwrap_or(true);
        let gamma_ok = self
            .gamma
            .map(|t| (current.gamma - t).abs() <= tolerance)
            .unwrap_or(true);
        let vega_ok = self
            .vega
            .map(|t| (current.vega - t).abs() <= tolerance)
            .unwrap_or(true);
        let theta_ok = self
            .theta
            .map(|t| (current.theta - t).abs() <= tolerance)
            .unwrap_or(true);

        delta_ok && gamma_ok && vega_ok && theta_ok
    }
}

impl fmt::Display for AdjustmentTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Adjustment Target:")?;
        if let Some(d) = self.delta {
            writeln!(f, "  Delta: {:.4}", d)?;
        }
        if let Some(g) = self.gamma {
            writeln!(f, "  Gamma: {:.6}", g)?;
        }
        if let Some(v) = self.vega {
            writeln!(f, "  Vega:  {:.4}", v)?;
        }
        if let Some(t) = self.theta {
            writeln!(f, "  Theta: {:.4}", t)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests_portfolio_greeks {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType};
    use crate::{ExpirationDate, Options, Side};
    use chrono::Utc;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn position(side: Side, quantity: Positive) -> Position {
        let option = Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            quantity,
            pos_or_panic!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            pos_or_panic!(0.02),
            None,
        );
        Position::new(
            option,
            pos_or_panic!(3.5),
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        )
    }

    /// The aggregate must be the plain sum of the per-position greeks. Each one
    /// already carries its `Side` and `quantity`, so re-applying a
    /// `quantity * sign` multiplier squared the size and cancelled the sign.
    #[test]
    fn test_from_positions_sums_signed_greeks_without_rescaling() {
        let long = position(Side::Long, Positive::TWO);
        let short = position(Side::Short, pos_or_panic!(3.0));
        let positions = vec![long.clone(), short.clone()];

        let Ok(portfolio) = PortfolioGreeks::from_positions(&positions) else {
            panic!("portfolio greeks should compute");
        };

        for (name, aggregate, a, b) in [
            ("delta", portfolio.delta, long.delta(), short.delta()),
            ("gamma", portfolio.gamma, long.gamma(), short.gamma()),
            ("theta", portfolio.theta, long.theta(), short.theta()),
            ("vega", portfolio.vega, long.vega(), short.vega()),
            ("rho", portfolio.rho, long.rho(), short.rho()),
        ] {
            let (Ok(a), Ok(b)) = (a, b) else {
                panic!("{name} should compute for both legs");
            };
            assert_eq!(aggregate, a + b, "{name} must be the unscaled sum");
        }
    }

    /// Offsetting legs are a closed position, so every aggregate greek is zero.
    #[test]
    fn test_from_positions_offsetting_legs_net_to_zero() {
        let positions = vec![
            position(Side::Long, pos_or_panic!(4.0)),
            position(Side::Short, pos_or_panic!(4.0)),
        ];
        let Ok(portfolio) = PortfolioGreeks::from_positions(&positions) else {
            panic!("portfolio greeks should compute");
        };
        assert_eq!(portfolio.delta, Decimal::ZERO);
        assert_eq!(portfolio.gamma, Decimal::ZERO);
        assert_eq!(portfolio.theta, Decimal::ZERO);
        assert_eq!(portfolio.vega, Decimal::ZERO);
        assert_eq!(portfolio.rho, Decimal::ZERO);
    }

    /// A short leg must reduce, not inflate, the portfolio's gamma.
    #[test]
    fn test_from_positions_short_leg_subtracts_gamma() {
        let long_only = vec![position(Side::Long, Positive::TWO)];
        let hedged = vec![
            position(Side::Long, Positive::TWO),
            position(Side::Short, Positive::ONE),
        ];
        let (Ok(a), Ok(b)) = (
            PortfolioGreeks::from_positions(&long_only),
            PortfolioGreeks::from_positions(&hedged),
        ) else {
            panic!("portfolio greeks should compute");
        };
        assert!(
            b.gamma < a.gamma,
            "adding a short leg must lower gamma, got {} against {}",
            b.gamma,
            a.gamma
        );
    }

    #[test]
    fn test_portfolio_greeks_default() {
        let greeks = PortfolioGreeks::default();
        assert_eq!(greeks.delta, Decimal::ZERO);
        assert_eq!(greeks.gamma, Decimal::ZERO);
        assert_eq!(greeks.theta, Decimal::ZERO);
        assert_eq!(greeks.vega, Decimal::ZERO);
        assert_eq!(greeks.rho, Decimal::ZERO);
    }

    #[test]
    fn test_portfolio_greeks_new() {
        let greeks =
            PortfolioGreeks::new(dec!(0.5), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.01));
        assert_eq!(greeks.delta, dec!(0.5));
        assert_eq!(greeks.gamma, dec!(0.02));
        assert_eq!(greeks.theta, dec!(-0.05));
        assert_eq!(greeks.vega, dec!(0.15));
        assert_eq!(greeks.rho, dec!(0.01));
    }

    #[test]
    fn test_is_delta_neutral() {
        let greeks =
            PortfolioGreeks::new(dec!(0.005), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.01));
        assert!(greeks.is_delta_neutral(dec!(0.01)));
        assert!(!greeks.is_delta_neutral(dec!(0.001)));
    }

    #[test]
    fn test_is_gamma_neutral() {
        let greeks =
            PortfolioGreeks::new(dec!(0.5), dec!(0.005), dec!(-0.05), dec!(0.15), dec!(0.01));
        assert!(greeks.is_gamma_neutral(dec!(0.01)));
        assert!(!greeks.is_gamma_neutral(dec!(0.001)));
    }

    #[test]
    fn test_delta_gap() {
        let greeks =
            PortfolioGreeks::new(dec!(0.3), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.01));
        assert_eq!(greeks.delta_gap(Decimal::ZERO).ok(), Some(dec!(-0.3)));
        assert_eq!(greeks.delta_gap(dec!(0.5)).ok(), Some(dec!(0.2)));
    }

    #[test]
    fn test_combined() {
        let greeks1 =
            PortfolioGreeks::new(dec!(0.3), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.01));
        let greeks2 =
            PortfolioGreeks::new(dec!(0.2), dec!(0.01), dec!(-0.03), dec!(0.10), dec!(0.005));
        let Ok(combined) = greeks1.combined(&greeks2) else {
            unreachable!("hand-written Greeks well inside the Decimal range")
        };

        assert_eq!(combined.delta, dec!(0.5));
        assert_eq!(combined.gamma, dec!(0.03));
        assert_eq!(combined.theta, dec!(-0.08));
        assert_eq!(combined.vega, dec!(0.25));
        assert_eq!(combined.rho, dec!(0.015));
    }

    #[test]
    fn test_add() {
        let mut greeks1 =
            PortfolioGreeks::new(dec!(0.3), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.01));
        let greeks2 =
            PortfolioGreeks::new(dec!(0.2), dec!(0.01), dec!(-0.03), dec!(0.10), dec!(0.005));
        assert!(greeks1.add(&greeks2).is_ok());

        assert_eq!(greeks1.delta, dec!(0.5));
        assert_eq!(greeks1.gamma, dec!(0.03));
    }

    /// `add` writes through `combined`, so an overflow on any one of the five
    /// Greeks has to leave all five as they were rather than commit the ones
    /// that happened to be summed first.
    #[test]
    fn test_portfolio_greeks_add_overflow_leaves_the_receiver_untouched() {
        let mut greeks = PortfolioGreeks::new(
            Decimal::MAX,
            dec!(0.02),
            dec!(-0.05),
            dec!(0.15),
            dec!(0.01),
        );
        let other = PortfolioGreeks::new(
            Decimal::MAX,
            dec!(0.01),
            dec!(-0.03),
            dec!(0.10),
            dec!(0.005),
        );

        assert!(greeks.add(&other).is_err());
        assert_eq!(greeks.delta, Decimal::MAX);
        assert_eq!(greeks.gamma, dec!(0.02));
        assert_eq!(greeks.theta, dec!(-0.05));
        assert_eq!(greeks.vega, dec!(0.15));
        assert_eq!(greeks.rho, dec!(0.01));
    }

    /// A gap against a target at the far end of the range is not a number
    /// this type can report, and the `Result` says so instead of aborting.
    #[test]
    fn test_portfolio_greeks_delta_gap_overflow_is_reported() {
        let greeks = PortfolioGreeks::new(
            -Decimal::MAX,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        );
        assert!(greeks.delta_gap(Decimal::MAX).is_err());
    }
}

#[cfg(test)]
mod tests_adjustment_target {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_delta_neutral() {
        let target = AdjustmentTarget::delta_neutral();
        assert_eq!(target.delta, Some(Decimal::ZERO));
        assert_eq!(target.gamma, None);
        assert_eq!(target.vega, None);
        assert_eq!(target.theta, None);
    }

    #[test]
    fn test_delta_gamma_neutral() {
        let target = AdjustmentTarget::delta_gamma_neutral();
        assert_eq!(target.delta, Some(Decimal::ZERO));
        assert_eq!(target.gamma, Some(Decimal::ZERO));
        assert_eq!(target.vega, None);
    }

    #[test]
    fn test_full_neutral() {
        let target = AdjustmentTarget::full_neutral();
        assert_eq!(target.delta, Some(Decimal::ZERO));
        assert_eq!(target.gamma, Some(Decimal::ZERO));
        assert_eq!(target.vega, Some(Decimal::ZERO));
        assert_eq!(target.theta, None);
    }

    #[test]
    fn test_builder_methods() {
        let target = AdjustmentTarget::default()
            .with_delta(dec!(0.1))
            .with_gamma(dec!(0.02))
            .with_vega(dec!(0.5));

        assert_eq!(target.delta, Some(dec!(0.1)));
        assert_eq!(target.gamma, Some(dec!(0.02)));
        assert_eq!(target.vega, Some(dec!(0.5)));
    }

    #[test]
    fn test_delta_gap() {
        let target = AdjustmentTarget::delta_neutral();
        let greeks =
            PortfolioGreeks::new(dec!(0.3), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.01));

        assert_eq!(target.delta_gap(&greeks), dec!(-0.3));
    }

    #[test]
    fn test_gamma_gap() {
        let target = AdjustmentTarget::delta_gamma_neutral();
        let greeks =
            PortfolioGreeks::new(dec!(0.3), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.01));

        assert_eq!(target.gamma_gap(&greeks), Some(dec!(-0.02)));
    }

    #[test]
    fn test_is_satisfied() {
        let target = AdjustmentTarget::delta_neutral();

        let neutral_greeks =
            PortfolioGreeks::new(dec!(0.005), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.01));
        assert!(target.is_satisfied(&neutral_greeks, dec!(0.01)));

        let non_neutral_greeks =
            PortfolioGreeks::new(dec!(0.5), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.01));
        assert!(!target.is_satisfied(&non_neutral_greeks, dec!(0.01)));
    }
}
