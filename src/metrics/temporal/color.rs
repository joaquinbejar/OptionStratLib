/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Color (Gamma Decay) Metrics
//!
//! This module provides traits for computing color curves and surfaces,
//! which measure the rate of change of gamma with respect to time.
//!
//! ## Overview
//!
//! Color (also known as DgammaDtime) measures how gamma changes as time
//! passes. This is crucial for:
//!
//! - **Gamma hedging**: Understanding how gamma exposure drifts over time
//! - **Position management**: Anticipating gamma changes
//! - **Risk management**: Planning gamma rehedging frequency
//!
//! ## Mathematical Background
//!
//! Color is defined as:
//!
//! ```text
//! Color = ∂Γ/∂t = ∂³V/∂S²∂t
//! ```
//!
//! Key characteristics:
//! - Color shows how gamma changes as expiration approaches
//! - ATM options have highest color sensitivity
//! - Gamma typically increases near expiration for ATM options
//!
//! ## Curve Representation
//!
//! The curve shows color by strike:
//! - **X-axis**: Strike price
//! - **Y-axis**: Color (gamma decay rate)
//!
//! ## Surface Representation
//!
//! The surface shows color across price and time:
//! - **X-axis**: Underlying price
//! - **Y-axis**: Days to expiration
//! - **Z-axis**: Color value


use crate::curves::Curve;
use crate::error::CurveError;
use crate::error::SurfaceError;
use crate::surfaces::Surface;

/// A trait for computing color curves by strike price.
///
/// The curve shows color at each strike, helping identify where gamma
/// decay is most significant.
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Color (gamma decay rate)
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::ColorCurve;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let color_curve = chain.color_curve()?;
/// ```
pub trait ColorCurve {
    /// Computes the color curve by strike price.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The color curve with strike on x-axis and color on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid color values
    /// - The option chain is empty
    fn color_curve(&self) -> Result<Curve, CurveError>;
}

/// A trait for computing color surfaces.
///
/// The surface shows how color evolves across both underlying price
/// and time to expiration.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Underlying price in currency units
/// - **Y-axis**: Days to expiration
/// - **Z-axis**: Color value
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::ColorSurface;
/// use optionstratlib::pos_or_panic;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
/// let days = vec![Positive::ONE, pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];
/// let surface = chain.color_surface(price_range, days, 20)?;
/// ```
pub trait ColorSurface {
    /// Computes the color surface (price vs time).
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `days_to_expiry`: Vector of days to expiration values
    /// - `price_steps`: Number of steps along the price axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The color surface with price on x-axis,
    ///   days on y-axis, and color on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn color_surface(
        &self,
        price_range: (Positive, Positive),
        days_to_expiry: Vec<Positive>,
        price_steps: usize,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_color {
    use super::*;
    use crate::curves::Point2D;

    use crate::surfaces::Point3D;
    use rust_decimal::Decimal;
    use rust_decimal::MathematicalOps;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;
use positive::pos_or_panic;

    struct TestColor {
        underlying_price: Positive,
    }

    impl ColorCurve for TestColor {
        fn color_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();
            let spot = self.underlying_price.to_dec();

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
                // Color peaks at ATM
                let moneyness = ((strike - spot) / spot).abs();
                let color = dec!(0.001) * (-moneyness * dec!(10.0)).exp();
                points.insert(Point2D::new(strike, color));
            }

            Ok(Curve::new(points))
        }
    }

    impl ColorSurface for TestColor {
        fn color_surface(
            &self,
            price_range: (Positive, Positive),
            days_to_expiry: Vec<Positive>,
            price_steps: usize,
        ) -> Result<Surface, SurfaceError> {
            let mut points = BTreeSet::new();

            let price_step = if price_steps > 0 {
                (price_range.1 - price_range.0).to_dec() / Decimal::from(price_steps)
            } else {
                Decimal::ZERO
            };

            let strike = self.underlying_price.to_dec();

            for days in &days_to_expiry {
                // Color increases as expiration approaches
                let time_factor = (dec!(30.0) / days.to_dec()).sqrt().unwrap_or(Decimal::ONE);

                for p in 0..=price_steps {
                    let price = price_range.0.to_dec() + price_step * Decimal::from(p);
                    let moneyness = ((price - strike) / strike).abs();

                    let color = dec!(0.0005) * (-moneyness * dec!(5.0)).exp() * time_factor;

                    points.insert(Point3D::new(price, days.to_dec(), color));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_color_curve_creation() {
        let color = TestColor {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = color.color_curve();
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 9);
    }

    #[test]
    fn test_color_curve_atm_highest() {
        let color = TestColor {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = color.color_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Find maximum color (ATM)
        let max_color = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        if let Some(max) = max_color {
            assert_eq!(max.x, dec!(450.0));
        }
    }

    #[test]
    fn test_color_curve_positive_values() {
        let color = TestColor {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = color.color_curve().unwrap();

        for point in curve.points.iter() {
            assert!(point.y >= Decimal::ZERO);
        }
    }

    #[test]
    fn test_color_surface_creation() {
        let color = TestColor {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];

        let surface = color.color_surface(price_range, days, 10);
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        assert_eq!(surface.points.len(), 33);
    }

    #[test]
    fn test_color_surface_time_effect() {
        let color = TestColor {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(450.0), pos_or_panic!(450.0));
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(30.0)];

        let surface = color.color_surface(price_range, days, 0).unwrap();

        let points: Vec<&Point3D> = surface.points.iter().collect();

        let color_7d = points.iter().find(|p| p.y == dec!(7.0)).map(|p| p.z);
        let color_30d = points.iter().find(|p| p.y == dec!(30.0)).map(|p| p.z);

        // Color should be higher near expiration
        if let (Some(c7), Some(c30)) = (color_7d, color_30d) {
            assert!(c7 > c30);
        }
    }

    #[test]
    fn test_color_surface_empty_days() {
        let color = TestColor {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days: Vec<Positive> = vec![];

        let surface = color.color_surface(price_range, days, 10).unwrap();
        assert!(surface.points.is_empty());
    }
}
