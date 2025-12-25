/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Smile Dynamics Metrics
//!
//! This module provides traits for computing smile dynamics curves and surfaces,
//! which track how the volatility smile evolves over time and across market
//! conditions.
//!
//! ## Overview
//!
//! Smile dynamics captures three key aspects of volatility smile behavior:
//!
//! - **Parallel Shift**: Overall level change in ATM volatility
//! - **Skew Change**: Change in the slope of the smile (put vs call IV)
//! - **Curvature Change**: Change in the convexity of the smile
//!
//! Understanding these dynamics is crucial for:
//! - Volatility trading strategies
//! - Risk management of vega exposure
//! - Predicting smile behavior under stress
//!
//! ## Mathematical Background
//!
//! The smile can be parameterized using a quadratic model:
//!
//! ```text
//! σ(K) = σ_ATM + β × (K - K_ATM) + γ × (K - K_ATM)²
//! ```
//!
//! where:
//! - σ_ATM is the at-the-money volatility
//! - β is the skew parameter
//! - γ is the curvature parameter
//!
//! ## Curve Representation
//!
//! The curve shows the current smile shape:
//! - **X-axis**: Strike price (or moneyness)
//! - **Y-axis**: Implied volatility
//!
//! ## Surface Representation
//!
//! The surface shows smile evolution over time:
//! - **X-axis**: Strike price (or moneyness)
//! - **Y-axis**: Days to expiration
//! - **Z-axis**: Implied volatility

use crate::Positive;
use crate::curves::Curve;
use crate::error::CurveError;
use crate::error::SurfaceError;
use crate::surfaces::Surface;

#[cfg(test)]
use rust_decimal::MathematicalOps;

/// A trait for computing smile dynamics curves by strike price.
///
/// The curve represents the current shape of the volatility smile,
/// showing how implied volatility varies across strike prices.
///
/// # Mathematical Background
///
/// The smile curve is typically characterized by:
/// - A minimum near ATM (for equity indices)
/// - Higher IV for OTM puts (negative skew)
/// - Higher IV for OTM calls (positive skew or smile)
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units (or log-moneyness)
/// - **Y-axis**: Implied volatility as a decimal
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::SmileDynamicsCurve;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let curve = chain.smile_dynamics_curve()?;
///
/// // Analyze skew: compare OTM put IV to OTM call IV
/// let otm_put_iv = curve.interpolate_at(dec!(0.9) * spot)?;
/// let otm_call_iv = curve.interpolate_at(dec!(1.1) * spot)?;
/// let skew = otm_put_iv - otm_call_iv;
/// ```
pub trait SmileDynamicsCurve {
    /// Computes the smile dynamics curve by strike price.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The smile curve with strike on x-axis and IV on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid implied volatility values
    /// - The option chain is empty
    fn smile_dynamics_curve(&self) -> Result<Curve, CurveError>;
}

/// A trait for computing smile dynamics surfaces.
///
/// The surface shows how the volatility smile evolves across different
/// time horizons, providing insights into term structure and smile
/// dynamics.
///
/// # Mathematical Background
///
/// The smile surface combines:
/// - Strike dimension: Current smile shape at each expiry
/// - Time dimension: How the smile changes with time to expiration
///
/// Common patterns include:
/// - Smile flattening as expiration approaches
/// - Skew steepening during market stress
/// - Term structure inversions
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
/// use optionstratlib::metrics::SmileDynamicsSurface;
/// use optionstratlib::pos;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let days = vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)];
/// let surface = chain.smile_dynamics_surface(days)?;
/// ```
pub trait SmileDynamicsSurface {
    /// Computes the smile dynamics surface (strike vs time).
    ///
    /// # Parameters
    ///
    /// - `days_to_expiry`: Vector of days to expiration values to include
    ///   in the surface. Each value generates a smile slice.
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The smile surface with strike on x-axis,
    ///   days on y-axis, and IV on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `SurfaceError::ConstructionError` if:
    /// - No options have valid implied volatility values
    /// - The days_to_expiry vector is empty
    /// - No valid surface points can be generated
    fn smile_dynamics_surface(
        &self,
        days_to_expiry: Vec<Positive>,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_smile_dynamics {
    use super::*;
    use crate::curves::Point2D;

    use crate::surfaces::Point3D;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestSmileDynamics {
        underlying_price: Positive,
        atm_vol: Positive,
        skew: Decimal,
        curvature: Decimal,
    }

    impl SmileDynamicsCurve for TestSmileDynamics {
        fn smile_dynamics_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();
            let spot = self.underlying_price.to_dec();
            let atm_vol = self.atm_vol.to_dec();

            // Generate smile using quadratic model
            let strikes = [
                spot * dec!(0.85),
                spot * dec!(0.90),
                spot * dec!(0.95),
                spot,
                spot * dec!(1.05),
                spot * dec!(1.10),
                spot * dec!(1.15),
            ];

            for strike in strikes {
                let moneyness = (strike / spot).ln(); // Log-moneyness
                let iv = atm_vol + self.skew * moneyness + self.curvature * moneyness * moneyness;
                points.insert(Point2D::new(strike, iv.max(dec!(0.01))));
            }

            Ok(Curve::new(points))
        }
    }

    impl SmileDynamicsSurface for TestSmileDynamics {
        fn smile_dynamics_surface(
            &self,
            days_to_expiry: Vec<Positive>,
        ) -> Result<Surface, SurfaceError> {
            let mut points = BTreeSet::new();
            let spot = self.underlying_price.to_dec();
            let atm_vol = self.atm_vol.to_dec();

            let strikes = [
                spot * dec!(0.85),
                spot * dec!(0.90),
                spot * dec!(0.95),
                spot,
                spot * dec!(1.05),
                spot * dec!(1.10),
                spot * dec!(1.15),
            ];

            for days in &days_to_expiry {
                // Smile parameters evolve with time
                // Skew typically steepens for shorter expirations
                let time_factor = (days.to_dec() / dec!(30.0)).sqrt().unwrap_or(Decimal::ONE);
                let adjusted_skew = self.skew / time_factor;
                let adjusted_curvature = self.curvature / time_factor;

                for strike in strikes {
                    let moneyness = (strike / spot).ln();
                    let iv = atm_vol
                        + adjusted_skew * moneyness
                        + adjusted_curvature * moneyness * moneyness;

                    points.insert(Point3D::new(strike, days.to_dec(), iv.max(dec!(0.01))));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_smile_dynamics_curve_creation() {
        let smile = TestSmileDynamics {
            underlying_price: pos!(450.0),
            atm_vol: pos!(0.20),
            skew: dec!(-0.10),     // Negative skew (equity-like)
            curvature: dec!(0.05), // Positive curvature (smile)
        };

        let curve = smile.smile_dynamics_curve();
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 7);
    }

    #[test]
    fn test_smile_dynamics_curve_skew() {
        let smile = TestSmileDynamics {
            underlying_price: pos!(450.0),
            atm_vol: pos!(0.20),
            skew: dec!(-0.10),     // Negative skew
            curvature: dec!(0.00), // No curvature for pure skew test
        };

        let curve = smile.smile_dynamics_curve().unwrap();
        let points: Vec<&Point2D> = curve.points.iter().collect();

        // With negative skew, lower strikes should have higher IV
        let low_strike_iv = points.first().map(|p| p.y).unwrap_or(Decimal::ZERO);
        let high_strike_iv = points.last().map(|p| p.y).unwrap_or(Decimal::ZERO);

        assert!(low_strike_iv > high_strike_iv);
    }

    #[test]
    fn test_smile_dynamics_curve_curvature() {
        let smile = TestSmileDynamics {
            underlying_price: pos!(450.0),
            atm_vol: pos!(0.20),
            skew: dec!(0.00),      // No skew for pure curvature test
            curvature: dec!(0.10), // Positive curvature
        };

        let curve = smile.smile_dynamics_curve().unwrap();
        let points: Vec<&Point2D> = curve.points.iter().collect();

        // With positive curvature and no skew, ATM should have lowest IV
        let atm_point = points.iter().min_by(|a, b| {
            let spot = dec!(450.0);
            let a_dist = (a.x - spot).abs();
            let b_dist = (b.x - spot).abs();
            a_dist.partial_cmp(&b_dist).unwrap()
        });

        if let Some(atm) = atm_point {
            for point in points.iter() {
                assert!(point.y >= atm.y - dec!(0.001)); // Small tolerance
            }
        }
    }

    #[test]
    fn test_smile_dynamics_surface_creation() {
        let smile = TestSmileDynamics {
            underlying_price: pos!(450.0),
            atm_vol: pos!(0.20),
            skew: dec!(-0.10),
            curvature: dec!(0.05),
        };

        let days = vec![pos!(7.0), pos!(14.0), pos!(30.0)];
        let surface = smile.smile_dynamics_surface(days);
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        // 7 strikes × 3 days = 21 points
        assert_eq!(surface.points.len(), 21);
    }

    #[test]
    fn test_smile_dynamics_surface_term_structure() {
        let smile = TestSmileDynamics {
            underlying_price: pos!(450.0),
            atm_vol: pos!(0.20),
            skew: dec!(-0.10),
            curvature: dec!(0.05),
        };

        let days = vec![pos!(7.0), pos!(30.0)];
        let surface = smile.smile_dynamics_surface(days).unwrap();

        // Find OTM put points at different expirations
        let otm_strike = dec!(450.0) * dec!(0.90);
        let points: Vec<&Point3D> = surface
            .points
            .iter()
            .filter(|p| (p.x - otm_strike).abs() < dec!(1.0))
            .collect();

        // Skew should be steeper (higher IV) for shorter expirations
        let iv_7d = points.iter().find(|p| p.y == dec!(7.0)).map(|p| p.z);
        let iv_30d = points.iter().find(|p| p.y == dec!(30.0)).map(|p| p.z);

        if let (Some(iv7), Some(iv30)) = (iv_7d, iv_30d) {
            // OTM put IV should be higher for shorter expiration due to steeper skew
            assert!(iv7 >= iv30);
        }
    }

    #[test]
    fn test_smile_dynamics_surface_empty_days() {
        let smile = TestSmileDynamics {
            underlying_price: pos!(450.0),
            atm_vol: pos!(0.20),
            skew: dec!(-0.10),
            curvature: dec!(0.05),
        };

        let days: Vec<Positive> = vec![];
        let surface = smile.smile_dynamics_surface(days).unwrap();
        assert!(surface.points.is_empty());
    }
}
