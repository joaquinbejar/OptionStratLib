/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Time Decay Profile Metrics
//!
//! This module provides traits for computing time decay profile curves and
//! surfaces, which are essential for understanding how option value erodes
//! as time passes.
//!
//! ## Overview
//!
//! Time decay (theta) measures how option prices change as time passes,
//! holding all else constant. Understanding theta is crucial for:
//!
//! - **Option selling strategies**: Profiting from time decay
//! - **Position management**: Knowing when decay accelerates
//! - **Risk management**: Understanding time-based exposure
//!
//! ## Mathematical Background
//!
//! Theta represents the rate of time decay:
//!
//! ```text
//! Theta = ∂V/∂t
//! ```
//!
//! Key characteristics:
//! - Theta is typically negative for long options (value decays)
//! - Decay accelerates near expiration (gamma effect)
//! - ATM options have highest theta (most time value to lose)
//!
//! ## Curve Representation
//!
//! The curve shows theta by strike:
//! - **X-axis**: Strike price
//! - **Y-axis**: Theta (daily time decay in dollars)
//!
//! ## Surface Representation
//!
//! The surface shows option value across price and time:
//! - **X-axis**: Underlying price
//! - **Y-axis**: Days to expiration
//! - **Z-axis**: Option value


use crate::curves::Curve;
use crate::error::CurveError;
use crate::error::SurfaceError;
use crate::surfaces::Surface;

/// A trait for computing time decay profile curves by strike price.
///
/// The curve shows theta at each strike, helping identify where time
/// decay exposure is concentrated.
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Theta (daily decay in dollars, typically negative)
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::TimeDecayCurve;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let theta_curve = chain.time_decay_curve()?;
///
/// // Find strike with maximum theta decay
/// let max_theta = theta_curve.points.iter()
///     .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap()); // Most negative
/// ```
pub trait TimeDecayCurve {
    /// Computes the time decay profile curve by strike price.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The theta curve with strike on x-axis and theta on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid theta values
    /// - The option chain is empty
    fn time_decay_curve(&self) -> Result<Curve, CurveError>;
}

/// A trait for computing time decay profile surfaces.
///
/// The surface shows how option value evolves across both underlying price
/// and time to expiration, visualizing the decay process.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Underlying price in currency units
/// - **Y-axis**: Days to expiration
/// - **Z-axis**: Option value
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::TimeDecaySurface;
/// use optionstratlib::pos_or_panic;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
/// let days = vec![Positive::ONE, pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];
/// let surface = chain.time_decay_surface(price_range, days, 20)?;
/// ```
pub trait TimeDecaySurface {
    /// Computes the time decay profile surface (price vs time).
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `days_to_expiry`: Vector of days to expiration values
    /// - `price_steps`: Number of steps along the price axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The decay surface with price on x-axis,
    ///   days on y-axis, and option value on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn time_decay_surface(
        &self,
        price_range: (Positive, Positive),
        days_to_expiry: Vec<Positive>,
        price_steps: usize,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_time_decay {
    use super::*;
    use crate::curves::Point2D;

    use crate::surfaces::Point3D;
    use rust_decimal::Decimal;
    use rust_decimal::MathematicalOps;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;
use positive::pos_or_panic;

    struct TestTimeDecay {
        underlying_price: Positive,
    }

    impl TimeDecayCurve for TestTimeDecay {
        fn time_decay_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();
            let spot = self.underlying_price.to_dec();

            // Theta is most negative at ATM
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
                // Simplified theta model: most negative at ATM
                let moneyness = ((strike - spot) / spot).abs();
                let theta = dec!(-0.15) * (-moneyness * dec!(10.0)).exp();
                points.insert(Point2D::new(strike, theta));
            }

            Ok(Curve::new(points))
        }
    }

    impl TimeDecaySurface for TestTimeDecay {
        fn time_decay_surface(
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
            let vol = dec!(0.20);

            for days in &days_to_expiry {
                let time_sqrt = (days.to_dec() / dec!(365.0))
                    .sqrt()
                    .unwrap_or(Decimal::ZERO);

                for p in 0..=price_steps {
                    let price = price_range.0.to_dec() + price_step * Decimal::from(p);

                    // Simplified option value: intrinsic + time value
                    let intrinsic = (price - strike).max(Decimal::ZERO);
                    let time_value = vol * price * time_sqrt * dec!(0.4);
                    let option_value = intrinsic + time_value;

                    points.insert(Point3D::new(price, days.to_dec(), option_value));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_time_decay_curve_creation() {
        let td = TestTimeDecay {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = td.time_decay_curve();
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 9);
    }

    #[test]
    fn test_time_decay_curve_atm_most_negative() {
        let td = TestTimeDecay {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = td.time_decay_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Find most negative theta (ATM)
        let min_theta = points.iter().min_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        if let Some(min) = min_theta {
            // ATM should have most negative theta
            assert_eq!(min.x, dec!(450.0));
        }
    }

    #[test]
    fn test_time_decay_curve_negative_theta() {
        let td = TestTimeDecay {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = td.time_decay_curve().unwrap();

        // All theta values should be negative for long options
        for point in curve.points.iter() {
            assert!(point.y <= Decimal::ZERO);
        }
    }

    #[test]
    fn test_time_decay_surface_creation() {
        let td = TestTimeDecay {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];

        let surface = td.time_decay_surface(price_range, days, 10);
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        // (10+1) × 3 = 33 points
        assert_eq!(surface.points.len(), 33);
    }

    #[test]
    fn test_time_decay_surface_time_effect() {
        let td = TestTimeDecay {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(480.0), pos_or_panic!(480.0)); // ITM call
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(30.0)];

        let surface = td.time_decay_surface(price_range, days, 0).unwrap();

        let points: Vec<&Point3D> = surface.points.iter().collect();

        // Find option value at different times
        let val_7d = points.iter().find(|p| p.y == dec!(7.0)).map(|p| p.z);
        let val_30d = points.iter().find(|p| p.y == dec!(30.0)).map(|p| p.z);

        // Option should be worth more with more time (more time value)
        if let (Some(v7), Some(v30)) = (val_7d, val_30d) {
            assert!(v30 >= v7);
        }
    }

    #[test]
    fn test_time_decay_surface_empty_days() {
        let td = TestTimeDecay {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days: Vec<Positive> = vec![];

        let surface = td.time_decay_surface(price_range, days, 10).unwrap();
        assert!(surface.points.is_empty());
    }
}
