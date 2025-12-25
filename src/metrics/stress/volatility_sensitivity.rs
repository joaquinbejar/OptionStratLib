/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Volatility Sensitivity Metrics
//!
//! This module provides traits for computing volatility sensitivity curves and
//! surfaces, which are essential for understanding how options respond to
//! changes in implied volatility.
//!
//! ## Overview
//!
//! Volatility sensitivity measures how option prices change when implied
//! volatility moves. This is crucial for:
//!
//! - **Vega hedging**: Managing exposure to volatility changes
//! - **Volatility trading**: Profiting from vol moves
//! - **Risk management**: Understanding worst-case scenarios
//!
//! ## Mathematical Background
//!
//! The P&L from a volatility shock is approximated using Taylor expansion:
//!
//! ```text
//! P&L ≈ Vega × Δσ + 0.5 × Vomma × Δσ²
//! ```
//!
//! where:
//! - Vega: First-order sensitivity to volatility
//! - Vomma: Second-order sensitivity (rate of change of vega)
//! - Δσ: Change in implied volatility
//!
//! ## Curve Representation
//!
//! The curve shows vega exposure by strike:
//! - **X-axis**: Strike price
//! - **Y-axis**: Vega (dollar change per 1% vol move)
//!
//! ## Surface Representation
//!
//! The surface shows P&L across price and volatility:
//! - **X-axis**: Underlying price
//! - **Y-axis**: Volatility level
//! - **Z-axis**: Option value or P&L


use crate::curves::Curve;
use crate::error::CurveError;
use crate::error::SurfaceError;
use crate::surfaces::Surface;

#[cfg(test)]
use rust_decimal::MathematicalOps;

/// A trait for computing volatility sensitivity curves by strike price.
///
/// The curve shows vega exposure at each strike, helping identify where
/// volatility risk is concentrated.
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Vega (sensitivity to 1% volatility change)
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::VolatilitySensitivityCurve;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let vega_curve = chain.volatility_sensitivity_curve()?;
///
/// // Find strike with maximum vega exposure
/// let max_vega = vega_curve.points.iter()
///     .max_by(|a, b| a.y.abs().partial_cmp(&b.y.abs()).unwrap());
/// ```
pub trait VolatilitySensitivityCurve {
    /// Computes the volatility sensitivity curve by strike price.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The vega curve with strike on x-axis and vega on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid vega values
    /// - The option chain is empty
    fn volatility_sensitivity_curve(&self) -> Result<Curve, CurveError>;
}

/// A trait for computing volatility sensitivity surfaces.
///
/// The surface shows how option value changes across both underlying price
/// and volatility levels, providing a complete view of vol risk.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Underlying price in currency units
/// - **Y-axis**: Implied volatility as a decimal
/// - **Z-axis**: Option value or P&L
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::VolatilitySensitivitySurface;
/// use optionstratlib::pos_or_panic;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
/// let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));
/// let surface = chain.volatility_sensitivity_surface(price_range, vol_range, 20, 20)?;
/// ```
pub trait VolatilitySensitivitySurface {
    /// Computes the volatility sensitivity surface (price vs volatility).
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `vol_range`: Tuple of (min_vol, max_vol) for implied volatility
    /// - `price_steps`: Number of steps along the price axis
    /// - `vol_steps`: Number of steps along the volatility axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The sensitivity surface with price on x-axis,
    ///   volatility on y-axis, and option value on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn volatility_sensitivity_surface(
        &self,
        price_range: (Positive, Positive),
        vol_range: (Positive, Positive),
        price_steps: usize,
        vol_steps: usize,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_volatility_sensitivity {
    use super::*;
    use crate::curves::Point2D;

    use crate::surfaces::Point3D;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;
use positive::pos_or_panic;

    struct TestVolatilitySensitivity {
        underlying_price: Positive,
    }

    impl VolatilitySensitivityCurve for TestVolatilitySensitivity {
        fn volatility_sensitivity_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();
            let spot = self.underlying_price.to_dec();

            // Vega peaks at ATM and decreases for OTM options
            let strikes = [
                dec!(380.0),
                dec!(400.0),
                dec!(420.0),
                dec!(440.0),
                dec!(450.0),
                dec!(460.0),
                dec!(480.0),
                dec!(500.0),
                dec!(520.0),
            ];

            for strike in strikes {
                // Simplified vega model: peaks at ATM
                let moneyness = ((strike - spot) / spot).abs();
                let vega = dec!(50.0) * (-moneyness * dec!(10.0)).exp();
                points.insert(Point2D::new(strike, vega));
            }

            Ok(Curve::new(points))
        }
    }

    impl VolatilitySensitivitySurface for TestVolatilitySensitivity {
        fn volatility_sensitivity_surface(
            &self,
            price_range: (Positive, Positive),
            vol_range: (Positive, Positive),
            price_steps: usize,
            vol_steps: usize,
        ) -> Result<Surface, SurfaceError> {
            let mut points = BTreeSet::new();

            let price_step = if price_steps > 0 {
                (price_range.1 - price_range.0).to_dec() / Decimal::from(price_steps)
            } else {
                Decimal::ZERO
            };

            let vol_step = if vol_steps > 0 {
                (vol_range.1 - vol_range.0).to_dec() / Decimal::from(vol_steps)
            } else {
                Decimal::ZERO
            };

            let strike = self.underlying_price.to_dec(); // ATM strike

            for p in 0..=price_steps {
                let price = price_range.0.to_dec() + price_step * Decimal::from(p);

                for v in 0..=vol_steps {
                    let vol = vol_range.0.to_dec() + vol_step * Decimal::from(v);

                    // Simplified option value model
                    let moneyness = price - strike;
                    let intrinsic = moneyness.max(Decimal::ZERO);
                    let time_value = vol * price * dec!(0.1);
                    let option_value = intrinsic + time_value;

                    points.insert(Point3D::new(price, vol, option_value));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_volatility_sensitivity_curve_creation() {
        let vs = TestVolatilitySensitivity {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = vs.volatility_sensitivity_curve();
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 9);
    }

    #[test]
    fn test_volatility_sensitivity_curve_atm_highest() {
        let vs = TestVolatilitySensitivity {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = vs.volatility_sensitivity_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Find maximum vega
        let max_vega = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        if let Some(max) = max_vega {
            // ATM should have highest vega
            assert_eq!(max.x, dec!(450.0));
        }
    }

    #[test]
    fn test_volatility_sensitivity_surface_creation() {
        let vs = TestVolatilitySensitivity {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let surface = vs.volatility_sensitivity_surface(price_range, vol_range, 10, 10);
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        // (10+1) × (10+1) = 121 points
        assert_eq!(surface.points.len(), 121);
    }

    #[test]
    fn test_volatility_sensitivity_surface_vol_effect() {
        let vs = TestVolatilitySensitivity {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(450.0), pos_or_panic!(450.0)); // Fixed price
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let surface = vs
            .volatility_sensitivity_surface(price_range, vol_range, 0, 10)
            .unwrap();

        let points: Vec<&Point3D> = surface.points.iter().collect();

        // Higher vol should mean higher option value (more time value)
        for i in 1..points.len() {
            assert!(points[i].z >= points[i - 1].z);
        }
    }

    #[test]
    fn test_volatility_sensitivity_curve_positive_vega() {
        let vs = TestVolatilitySensitivity {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = vs.volatility_sensitivity_curve().unwrap();

        // All vega values should be positive for long options
        for point in curve.points.iter() {
            assert!(point.y >= Decimal::ZERO);
        }
    }
}
