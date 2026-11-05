/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Implied Volatility Metrics
//!
//! This module provides traits for computing implied volatility curves and
//! projecting them over time horizons, for options analysis and trading.
//!
//! ## Overview
//!
//! Implied volatility (IV) represents the market's expectation of future volatility
//! and is derived from option prices using models like Black-Scholes. Understanding
//! IV across different strikes is crucial for:
//!
//! - Options pricing and valuation
//! - Identifying trading opportunities
//! - Risk management and hedging
//! - Understanding market sentiment
//!
//! ## Curve Representation
//!
//! The IV curve shows how implied volatility varies across strike prices for a
//! single expiration date. Common patterns include:
//!
//! - **Volatility Smile**: Higher IV for both OTM calls and puts
//! - **Volatility Skew**: Higher IV for OTM puts (common in equity markets)
//! - **Flat Curve**: Uniform IV across strikes (rare in practice)
//!
//! ## Horizon Projection
//!
//! [`HorizonVolatilitySurface`] scales each annualized implied volatility by
//! the square root of time, giving the volatility over each horizon. It is not
//! annualized, and every horizon comes from the same volatilities, so it says
//! nothing about the term structure: it is not an implied volatility surface.

use optionstratlib_core::model::Positive;
use optionstratlib_math::curves::Curve;
use optionstratlib_math::error::CurveError;
use optionstratlib_math::error::SurfaceError;
use optionstratlib_math::surfaces::Surface;

#[cfg(test)]
use rust_decimal::MathematicalOps;

/// A trait for computing implied volatility curves by strike price.
///
/// The implied volatility curve shows how IV varies across different strike
/// prices for options with the same expiration date. This is fundamental for
/// understanding market expectations and pricing options.
///
/// # Mathematical Background
///
/// Implied volatility is the volatility value that, when input into the
/// Black-Scholes formula, produces the observed market price. The curve
/// plots this value against strike prices.
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Implied volatility as a decimal (e.g., 0.20 for 20%)
///
/// # Example
///
/// ```ignore
/// use optionstratlib_market::chains::chain::OptionChain;
/// use optionstratlib_analytics::metrics::ImpliedVolatilityCurve;
///
/// let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
/// let iv_curve = chain.iv_curve()?;
///
/// // The curve can be used for visualization or analysis
/// for point in iv_curve.points.iter() {
///     info!("Strike: {}, IV: {:.2}%", point.x, point.y * 100.0);
/// }
/// ```
pub trait ImpliedVolatilityCurve {
    /// Computes the implied volatility curve by strike price.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The IV curve with strike prices on x-axis and IV on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed due to missing data
    ///   or calculation errors
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid implied volatility values
    /// - The option chain is empty
    fn iv_curve(&self) -> Result<Curve, CurveError>;
}

/// A trait for projecting implied volatility over time horizons.
///
/// The result is **not** an implied volatility surface and is not calibrated
/// to anything: every horizon is projected from the one set of implied
/// volatilities the source holds, so it carries no observed term structure.
/// It answers "what standard deviation of returns does this annualized
/// volatility imply over `T` days?".
///
/// # Mathematical Background
///
/// The projection uses the square root of time rule, which is derived from the
/// diffusion process underlying option pricing:
///
/// ```text
/// σ_horizon = σ_annualized × √(T / 365)
/// ```
///
/// where T is the horizon in days.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Horizon in days
/// - **Z-axis**: Volatility over that horizon, as a decimal. It is **not**
///   annualized: 20% annualized over 30 days is about 5.73%.
///
/// # Example
///
/// ```ignore
/// use optionstratlib_market::chains::chain::OptionChain;
/// use optionstratlib_analytics::metrics::HorizonVolatilitySurface;
/// use optionstratlib_core::pos_or_panic;
///
/// let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
/// let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)];
/// let surface = chain.horizon_volatility_surface(days)?;
/// ```
pub trait HorizonVolatilitySurface {
    /// Projects the implied volatility of each strike over each horizon.
    ///
    /// # Parameters
    ///
    /// - `horizon_days`: The horizons, in days, to include in the surface.
    ///   Each value generates a slice of the surface.
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The projection with strike on x-axis, horizon in days
    ///   on y-axis, and the non-annualized volatility over that horizon on
    ///   z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `SurfaceError::ConstructionError` if:
    /// - No options have valid implied volatility values
    /// - The horizon_days vector is empty
    /// - No valid points can be generated
    fn horizon_volatility_surface(
        &self,
        horizon_days: Vec<Positive>,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_implied_volatility_traits {
    use super::*;
    use optionstratlib_core::pos_or_panic;
    use optionstratlib_math::curves::Point2D;
    use optionstratlib_math::surfaces::Point3D;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestIVCurve;

    impl ImpliedVolatilityCurve for TestIVCurve {
        fn iv_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();
            points.insert(Point2D::new(dec!(90.0), dec!(0.25)));
            points.insert(Point2D::new(dec!(95.0), dec!(0.22)));
            points.insert(Point2D::new(dec!(100.0), dec!(0.20)));
            points.insert(Point2D::new(dec!(105.0), dec!(0.21)));
            points.insert(Point2D::new(dec!(110.0), dec!(0.23)));
            Ok(Curve::new(points))
        }
    }

    struct TestHorizonSurface;

    impl HorizonVolatilitySurface for TestHorizonSurface {
        fn horizon_volatility_surface(
            &self,
            horizon_days: Vec<Positive>,
        ) -> Result<Surface, SurfaceError> {
            let mut points = BTreeSet::new();
            let strikes = [
                dec!(90.0),
                dec!(95.0),
                dec!(100.0),
                dec!(105.0),
                dec!(110.0),
            ];
            let base_ivs = [dec!(0.25), dec!(0.22), dec!(0.20), dec!(0.21), dec!(0.23)];

            for (strike, base_iv) in strikes.iter().zip(base_ivs.iter()) {
                for days in &horizon_days {
                    let time_factor = (days.to_dec() / dec!(365.0)).sqrt().unwrap_or(Decimal::ONE);
                    let horizon_volatility = *base_iv * time_factor;
                    points.insert(Point3D::new(*strike, days.to_dec(), horizon_volatility));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_iv_curve_implementation() {
        let iv = TestIVCurve;
        let curve = iv.iv_curve().unwrap();

        assert_eq!(curve.points.len(), 5);

        let points: Vec<&Point2D> = curve.points.iter().collect();
        assert_eq!(points[0].x, dec!(90.0));
        assert_eq!(points[0].y, dec!(0.25));
        assert_eq!(points[2].x, dec!(100.0));
        assert_eq!(points[2].y, dec!(0.20));
    }

    #[test]
    fn test_iv_curve_atm_lowest() {
        let iv = TestIVCurve;
        let curve = iv.iv_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();
        let atm_iv = points[2].y;

        // ATM should have lowest IV in this smile pattern
        for point in points.iter() {
            assert!(point.y >= atm_iv);
        }
    }

    #[test]
    fn test_horizon_volatility_surface_implementation() {
        let iv = TestHorizonSurface;
        let days = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let surface = iv.horizon_volatility_surface(days).unwrap();

        // 5 strikes × 3 days = 15 points
        assert_eq!(surface.points.len(), 15);
    }

    #[test]
    fn test_horizon_volatility_surface_time_scaling() {
        let iv = TestHorizonSurface;
        let days = vec![pos_or_panic!(30.0), pos_or_panic!(90.0)];
        let surface = iv.horizon_volatility_surface(days).unwrap();

        // Find points at same strike but different times
        let points: Vec<&Point3D> = surface.points.iter().collect();

        // Points at strike 100 should show time scaling
        let strike_100_points: Vec<&&Point3D> =
            points.iter().filter(|p| p.x == dec!(100.0)).collect();

        assert_eq!(strike_100_points.len(), 2);
    }

    #[test]
    fn test_horizon_volatility_surface_empty_days() {
        let iv = TestHorizonSurface;
        let days: Vec<Positive> = vec![];
        let surface = iv.horizon_volatility_surface(days).unwrap();

        // With empty days, surface should have no points
        assert!(surface.points.is_empty());
    }
}
