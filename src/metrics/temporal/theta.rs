/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Theta (Time Decay) Metrics
//!
//! This module provides traits for computing theta curves and surfaces,
//! which measure the rate of change of option value with respect to time.
//!
//! ## Overview
//!
//! Theta (Θ) represents the time decay of an option - how much value the
//! option loses each day, all else being equal. Understanding theta is
//! crucial for:
//!
//! - **Option selling strategies**: Profiting from time decay
//! - **Position management**: Knowing when decay accelerates
//! - **Risk management**: Understanding time-based exposure
//!
//! ## Mathematical Background
//!
//! Theta is defined as:
//!
//! ```text
//! Θ = ∂V/∂t
//! ```
//!
//! For a call option under Black-Scholes:
//! ```text
//! Θ = -S·N'(d1)·σ/(2√T) - r·K·e^(-rT)·N(d2)
//! ```
//!
//! Key characteristics:
//! - Theta is typically negative for long options
//! - Decay accelerates near expiration
//! - ATM options have highest theta (most time value)
//!
//! ## Curve Representation
//!
//! The curve shows theta by strike:
//! - **X-axis**: Strike price
//! - **Y-axis**: Theta (daily time decay)
//!
//! ## Surface Representation
//!
//! The surface shows theta across price and time:
//! - **X-axis**: Underlying price
//! - **Y-axis**: Days to expiration
//! - **Z-axis**: Theta value

use crate::Positive;
use crate::curves::Curve;
use crate::error::CurveError;
use crate::error::SurfaceError;
use crate::surfaces::Surface;

/// A trait for computing theta curves by strike price.
///
/// The curve shows theta at each strike, helping identify where time
/// decay exposure is concentrated.
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Theta (daily decay, typically negative)
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::ThetaCurve;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let theta_curve = chain.theta_curve()?;
///
/// // Find strike with maximum theta decay
/// let max_decay = theta_curve.points.iter()
///     .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
/// ```
pub trait ThetaCurve {
    /// Computes the theta curve by strike price.
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
    fn theta_curve(&self) -> Result<Curve, CurveError>;
}

/// A trait for computing theta surfaces.
///
/// The surface shows how theta evolves across both underlying price
/// and time to expiration.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Underlying price in currency units
/// - **Y-axis**: Days to expiration
/// - **Z-axis**: Theta value
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::ThetaSurface;
/// use optionstratlib::pos_or_panic;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
/// let days = vec![pos_or_panic!(1.0), pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];
/// let surface = chain.theta_surface(price_range, days, 20)?;
/// ```
pub trait ThetaSurface {
    /// Computes the theta surface (price vs time).
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `days_to_expiry`: Vector of days to expiration values
    /// - `price_steps`: Number of steps along the price axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The theta surface with price on x-axis,
    ///   days on y-axis, and theta on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn theta_surface(
        &self,
        price_range: (Positive, Positive),
        days_to_expiry: Vec<Positive>,
        price_steps: usize,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_theta {
    use super::*;
    use crate::curves::Point2D;

    use crate::surfaces::Point3D;
    use rust_decimal::Decimal;
    use rust_decimal::MathematicalOps;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestTheta {
        underlying_price: Positive,
    }

    impl ThetaCurve for TestTheta {
        fn theta_curve(&self) -> Result<Curve, CurveError> {
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
                let moneyness = ((strike - spot) / spot).abs();
                let theta = dec!(-0.15) * (-moneyness * dec!(10.0)).exp();
                points.insert(Point2D::new(strike, theta));
            }

            Ok(Curve::new(points))
        }
    }

    impl ThetaSurface for TestTheta {
        fn theta_surface(
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
                // Theta accelerates near expiration
                let time_factor = (dec!(30.0) / days.to_dec()).sqrt().unwrap_or(Decimal::ONE);

                for p in 0..=price_steps {
                    let price = price_range.0.to_dec() + price_step * Decimal::from(p);
                    let moneyness = ((price - strike) / strike).abs();

                    // Theta peaks at ATM and accelerates near expiration
                    let theta = dec!(-0.10) * (-moneyness * dec!(5.0)).exp() * time_factor;

                    points.insert(Point3D::new(price, days.to_dec(), theta));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_theta_curve_creation() {
        let theta = TestTheta {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = theta.theta_curve();
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 9);
    }

    #[test]
    fn test_theta_curve_atm_most_negative() {
        let theta = TestTheta {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = theta.theta_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Find most negative theta
        let min_theta = points.iter().min_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        if let Some(min) = min_theta {
            assert_eq!(min.x, dec!(450.0));
        }
    }

    #[test]
    fn test_theta_curve_negative_values() {
        let theta = TestTheta {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = theta.theta_curve().unwrap();

        for point in curve.points.iter() {
            assert!(point.y <= Decimal::ZERO);
        }
    }

    #[test]
    fn test_theta_surface_creation() {
        let theta = TestTheta {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];

        let surface = theta.theta_surface(price_range, days, 10);
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        assert_eq!(surface.points.len(), 33);
    }

    #[test]
    fn test_theta_surface_time_acceleration() {
        let theta = TestTheta {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(450.0), pos_or_panic!(450.0));
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(30.0)];

        let surface = theta.theta_surface(price_range, days, 0).unwrap();

        let points: Vec<&Point3D> = surface.points.iter().collect();

        let theta_7d = points.iter().find(|p| p.y == dec!(7.0)).map(|p| p.z);
        let theta_30d = points.iter().find(|p| p.y == dec!(30.0)).map(|p| p.z);

        // Theta should be more negative (larger absolute value) near expiration
        if let (Some(t7), Some(t30)) = (theta_7d, theta_30d) {
            assert!(t7 < t30); // More negative
        }
    }
}
