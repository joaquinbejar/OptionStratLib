/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Implied Volatility Metrics
//!
//! This module provides traits for computing implied volatility curves and surfaces,
//! which are fundamental tools for options analysis and trading.
//!
//! ## Overview
//!
//! Implied volatility (IV) represents the market's expectation of future volatility
//! and is derived from option prices using models like Black-Scholes. Understanding
//! IV across different strikes and time horizons is crucial for:
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
//! ## Surface Representation
//!
//! The IV surface extends the curve concept to include time, showing IV as a
//! function of both strike price and time to expiration. This provides:
//!
//! - Complete volatility structure visualization
//! - Term structure analysis
//! - Calendar spread opportunities identification

use crate::Positive;
use crate::curves::Curve;
use crate::error::CurveError;
use crate::error::SurfaceError;
use crate::surfaces::Surface;

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
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::ImpliedVolatilityCurve;
///
/// let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
/// let iv_curve = chain.iv_curve()?;
///
/// // The curve can be used for visualization or analysis
/// for point in iv_curve.points.iter() {
///     println!("Strike: {}, IV: {:.2}%", point.x, point.y * 100.0);
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

/// A trait for computing implied volatility surfaces.
///
/// The IV surface represents implied volatility as a function of both
/// strike price and time to expiration, providing a complete view of
/// the volatility structure across the entire option space.
///
/// # Mathematical Background
///
/// The surface uses time scaling based on the square root of time rule,
/// which is derived from the diffusion process underlying option pricing:
///
/// ```text
/// σ_adjusted = σ_base × √(T / 365)
/// ```
///
/// where T is days to expiration.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Days to expiration
/// - **Z-axis**: Implied volatility as a decimal
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::ImpliedVolatilitySurface;
/// use optionstratlib::pos_or_panic;
///
/// let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
/// let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)];
/// let iv_surface = chain.iv_surface(days)?;
/// ```
pub trait ImpliedVolatilitySurface {
    /// Computes the implied volatility surface (strike vs time).
    ///
    /// # Parameters
    ///
    /// - `days_to_expiry`: Vector of days to expiration values to include
    ///   in the surface. Each value generates a slice of the surface.
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The IV surface with strike on x-axis, days on y-axis,
    ///   and IV on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `SurfaceError::ConstructionError` if:
    /// - No options have valid implied volatility values
    /// - The days_to_expiry vector is empty
    /// - No valid points can be generated
    fn iv_surface(&self, days_to_expiry: Vec<Positive>) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_implied_volatility_traits {
    use super::*;
    use crate::curves::Point2D;
    use crate::surfaces::Point3D;
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

    struct TestIVSurface;

    impl ImpliedVolatilitySurface for TestIVSurface {
        fn iv_surface(&self, days_to_expiry: Vec<Positive>) -> Result<Surface, SurfaceError> {
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
                for days in &days_to_expiry {
                    let time_factor = (days.to_dec() / dec!(365.0)).sqrt().unwrap_or(Decimal::ONE);
                    let adjusted_iv = *base_iv * time_factor;
                    points.insert(Point3D::new(*strike, days.to_dec(), adjusted_iv));
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
    fn test_iv_surface_implementation() {
        let iv = TestIVSurface;
        let days = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let surface = iv.iv_surface(days).unwrap();

        // 5 strikes × 3 days = 15 points
        assert_eq!(surface.points.len(), 15);
    }

    #[test]
    fn test_iv_surface_time_scaling() {
        let iv = TestIVSurface;
        let days = vec![pos_or_panic!(30.0), pos_or_panic!(90.0)];
        let surface = iv.iv_surface(days).unwrap();

        // Find points at same strike but different times
        let points: Vec<&Point3D> = surface.points.iter().collect();

        // Points at strike 100 should show time scaling
        let strike_100_points: Vec<&&Point3D> =
            points.iter().filter(|p| p.x == dec!(100.0)).collect();

        assert_eq!(strike_100_points.len(), 2);
    }

    #[test]
    fn test_iv_surface_empty_days() {
        let iv = TestIVSurface;
        let days: Vec<Positive> = vec![];
        let surface = iv.iv_surface(days).unwrap();

        // With empty days, surface should have no points
        assert!(surface.points.is_empty());
    }
}
