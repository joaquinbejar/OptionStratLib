/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Charm (Delta Decay) Metrics
//!
//! This module provides traits for computing charm curves and surfaces,
//! which measure the rate of change of delta with respect to time.
//!
//! ## Overview
//!
//! Charm (also known as DdeltaDtime) measures how delta changes as time
//! passes. This is crucial for:
//!
//! - **Delta hedging**: Understanding how hedge ratios drift over time
//! - **Position management**: Anticipating delta changes
//! - **Risk management**: Planning rehedging frequency
//!
//! ## Mathematical Background
//!
//! Charm is defined as:
//!
//! ```text
//! Charm = ∂Δ/∂t = ∂²V/∂S∂t
//! ```
//!
//! Key characteristics:
//! - Charm shows how delta drifts toward 0 or 1 as expiration approaches
//! - ATM options have highest charm sensitivity
//! - Effects are more pronounced near expiration
//!
//! ## Curve Representation
//!
//! The curve shows charm by strike:
//! - **X-axis**: Strike price
//! - **Y-axis**: Charm (delta decay rate)
//!
//! ## Surface Representation
//!
//! The surface shows charm across price and time:
//! - **X-axis**: Underlying price
//! - **Y-axis**: Days to expiration
//! - **Z-axis**: Charm value


use crate::curves::Curve;
use crate::error::CurveError;
use crate::error::SurfaceError;
use crate::surfaces::Surface;
use positive::Positive;

/// A trait for computing charm curves by strike price.
///
/// The curve shows charm at each strike, helping identify where delta
/// decay is most significant.
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Charm (delta decay rate)
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::CharmCurve;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let charm_curve = chain.charm_curve()?;
/// ```
pub trait CharmCurve {
    /// Computes the charm curve by strike price.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The charm curve with strike on x-axis and charm on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid charm values
    /// - The option chain is empty
    fn charm_curve(&self) -> Result<Curve, CurveError>;
}

/// A trait for computing charm surfaces.
///
/// The surface shows how charm evolves across both underlying price
/// and time to expiration.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Underlying price in currency units
/// - **Y-axis**: Days to expiration
/// - **Z-axis**: Charm value
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::CharmSurface;
/// use optionstratlib::pos_or_panic;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
/// let days = vec![Positive::ONE, pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];
/// let surface = chain.charm_surface(price_range, days, 20)?;
/// ```
pub trait CharmSurface {
    /// Computes the charm surface (price vs time).
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `days_to_expiry`: Vector of days to expiration values
    /// - `price_steps`: Number of steps along the price axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The charm surface with price on x-axis,
    ///   days on y-axis, and charm on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn charm_surface(
        &self,
        price_range: (Positive, Positive),
        days_to_expiry: Vec<Positive>,
        price_steps: usize,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_charm {
    use super::*;
    use crate::curves::Point2D;

    use crate::surfaces::Point3D;
    use rust_decimal::Decimal;
    use rust_decimal::MathematicalOps;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;
use positive::pos_or_panic;

    struct TestCharm {
        underlying_price: Positive,
    }

    impl CharmCurve for TestCharm {
        fn charm_curve(&self) -> Result<Curve, CurveError> {
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
                // Charm is positive for OTM calls, negative for ITM calls
                let moneyness = (spot - strike) / spot;
                let charm = moneyness * dec!(0.02) * (-moneyness.abs() * dec!(5.0)).exp();
                points.insert(Point2D::new(strike, charm));
            }

            Ok(Curve::new(points))
        }
    }

    impl CharmSurface for TestCharm {
        fn charm_surface(
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
                let time_factor = (dec!(30.0) / days.to_dec()).sqrt().unwrap_or(Decimal::ONE);

                for p in 0..=price_steps {
                    let price = price_range.0.to_dec() + price_step * Decimal::from(p);
                    let moneyness = (price - strike) / strike;

                    let charm = moneyness * dec!(0.01) * time_factor;

                    points.insert(Point3D::new(price, days.to_dec(), charm));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_charm_curve_creation() {
        let charm = TestCharm {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = charm.charm_curve();
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 9);
    }

    #[test]
    fn test_charm_curve_sign_change() {
        let charm = TestCharm {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = charm.charm_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // ITM calls (low strikes) should have different sign than OTM calls (high strikes)
        let itm_charm = points.iter().find(|p| p.x == dec!(400.0)).map(|p| p.y);
        let otm_charm = points.iter().find(|p| p.x == dec!(500.0)).map(|p| p.y);

        if let (Some(itm), Some(otm)) = (itm_charm, otm_charm) {
            assert!(itm > Decimal::ZERO); // ITM call charm positive
            assert!(otm < Decimal::ZERO); // OTM call charm negative
        }
    }

    #[test]
    fn test_charm_surface_creation() {
        let charm = TestCharm {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];

        let surface = charm.charm_surface(price_range, days, 10);
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        assert_eq!(surface.points.len(), 33);
    }

    #[test]
    fn test_charm_surface_empty_days() {
        let charm = TestCharm {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days: Vec<Positive> = vec![];

        let surface = charm.charm_surface(price_range, days, 10).unwrap();
        assert!(surface.points.is_empty());
    }
}
