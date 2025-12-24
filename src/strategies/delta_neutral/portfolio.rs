/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/24
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
use crate::model::position::Position;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
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
    /// println!("Portfolio delta: {}", greeks.delta);
    /// ```
    pub fn from_positions(positions: &[Position]) -> Result<Self, GreeksError> {
        let mut greeks = Self::default();

        for pos in positions {
            let qty = pos.option.quantity.to_dec();
            let sign = if pos.option.is_long() {
                dec!(1)
            } else {
                dec!(-1)
            };
            let mult = qty * sign;

            greeks.delta += pos.option.delta()? * mult;
            greeks.gamma += pos.option.gamma()? * mult;
            greeks.theta += pos.option.theta()? * mult;
            greeks.vega += pos.option.vega()? * mult;
            greeks.rho += pos.option.rho()? * mult;
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
    pub fn from_positions_with_underlying(
        positions: &[Position],
        underlying_quantity: Decimal,
    ) -> Result<Self, GreeksError> {
        let mut greeks = Self::from_positions(positions)?;
        // Each share of underlying has delta = 1
        greeks.delta += underlying_quantity;
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
    #[must_use]
    pub fn delta_gap(&self, target: Decimal) -> Decimal {
        target - self.delta
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
    #[must_use]
    pub fn gamma_gap(&self, target: Decimal) -> Decimal {
        target - self.gamma
    }

    /// Adds another PortfolioGreeks to this one.
    ///
    /// Useful for combining Greeks from multiple sources.
    pub fn add(&mut self, other: &PortfolioGreeks) {
        self.delta += other.delta;
        self.gamma += other.gamma;
        self.theta += other.theta;
        self.vega += other.vega;
        self.rho += other.rho;
    }

    /// Returns a new PortfolioGreeks that is the sum of this and another.
    #[must_use]
    pub fn combined(&self, other: &PortfolioGreeks) -> PortfolioGreeks {
        PortfolioGreeks {
            delta: self.delta + other.delta,
            gamma: self.gamma + other.gamma,
            theta: self.theta + other.theta,
            vega: self.vega + other.vega,
            rho: self.rho + other.rho,
        }
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
        assert_eq!(greeks.delta_gap(Decimal::ZERO), dec!(-0.3));
        assert_eq!(greeks.delta_gap(dec!(0.5)), dec!(0.2));
    }

    #[test]
    fn test_combined() {
        let greeks1 =
            PortfolioGreeks::new(dec!(0.3), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.01));
        let greeks2 =
            PortfolioGreeks::new(dec!(0.2), dec!(0.01), dec!(-0.03), dec!(0.10), dec!(0.005));
        let combined = greeks1.combined(&greeks2);

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
        greeks1.add(&greeks2);

        assert_eq!(greeks1.delta, dec!(0.5));
        assert_eq!(greeks1.gamma, dec!(0.03));
    }
}

#[cfg(test)]
mod tests_adjustment_target {
    use super::*;

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
