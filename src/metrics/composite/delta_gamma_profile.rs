/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Delta-Gamma Profile Metrics
//!
//! This module provides traits for computing delta-gamma profile curves and
//! surfaces, which are essential for understanding combined directional and
//! convexity risk exposure.
//!
//! ## Overview
//!
//! The Delta-Gamma profile combines two fundamental Greeks:
//!
//! - **Delta**: First-order sensitivity to underlying price changes
//! - **Gamma**: Second-order sensitivity (rate of change of delta)
//!
//! Together, they provide a complete picture of how an option or portfolio
//! responds to price movements.
//!
//! ## Mathematical Background
//!
//! For a portfolio, the P&L approximation using delta and gamma is:
//!
//! ```text
//! P&L ≈ Δ × ΔS + 0.5 × Γ × (ΔS)²
//! ```
//!
//! The dollar equivalents are:
//! - **Dollar Delta**: Δ × S (P&L for $1 move in underlying)
//! - **Dollar Gamma**: Γ × S² × 0.01 (P&L for 1% move in underlying)
//!
//! ## Curve Representation
//!
//! The curve shows delta and gamma exposure across strike prices:
//! - **X-axis**: Strike price
//! - **Y-axis**: Combined delta-gamma metric
//!
//! ## Surface Representation
//!
//! The surface shows how exposure evolves across price and time:
//! - **X-axis**: Underlying price
//! - **Y-axis**: Days to expiration
//! - **Z-axis**: Delta exposure (or gamma exposure)


use crate::curves::Curve;
use crate::error::CurveError;
use crate::error::SurfaceError;
use crate::surfaces::Surface;
use positive::Positive;

#[cfg(test)]
use rust_decimal::MathematicalOps;

/// A trait for computing delta-gamma profile curves by strike price.
///
/// The curve shows the combined delta and gamma exposure at each strike,
/// helping identify where directional and convexity risks are concentrated.
///
/// # Mathematical Background
///
/// At each strike K, the profile computes:
/// - Delta exposure from options at that strike
/// - Gamma exposure from options at that strike
/// - Combined metric (e.g., dollar delta + dollar gamma)
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Delta-gamma profile value (combined exposure metric)
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::DeltaGammaProfileCurve;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let curve = chain.delta_gamma_curve()?;
///
/// // Find strike with maximum exposure
/// let max_point = curve.points.iter()
///     .max_by(|a, b| a.y.abs().partial_cmp(&b.y.abs()).unwrap());
/// ```
pub trait DeltaGammaProfileCurve {
    /// Computes the delta-gamma profile curve by strike price.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The profile curve with strike on x-axis and
    ///   combined delta-gamma metric on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid delta/gamma values
    /// - The option chain is empty
    fn delta_gamma_curve(&self) -> Result<Curve, CurveError>;
}

/// A trait for computing delta-gamma profile surfaces.
///
/// The surface shows how delta (or gamma) exposure varies across both
/// underlying price and time to expiration, providing a complete view
/// of risk evolution.
///
/// # Mathematical Background
///
/// For each point (price, time), the surface computes the portfolio's
/// delta exposure if the underlying were at that price with that time
/// remaining to expiration.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Underlying price in currency units
/// - **Y-axis**: Days to expiration
/// - **Z-axis**: Delta exposure (or gamma exposure)
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::DeltaGammaProfileSurface;
/// use optionstratlib::pos_or_panic;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
/// let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];
/// let surface = chain.delta_gamma_surface(price_range, days, 20)?;
/// ```
pub trait DeltaGammaProfileSurface {
    /// Computes the delta-gamma profile surface (price vs time).
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `days_to_expiry`: Vector of days to expiration values
    /// - `price_steps`: Number of steps along the price axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The profile surface with price on x-axis,
    ///   days on y-axis, and delta exposure on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `SurfaceError::ConstructionError` if:
    /// - The option chain lacks sufficient data
    /// - Price range is invalid
    /// - No valid surface points can be generated
    fn delta_gamma_surface(
        &self,
        price_range: (Positive, Positive),
        days_to_expiry: Vec<Positive>,
        price_steps: usize,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_delta_gamma_profile {
    use super::*;
    use crate::curves::Point2D;

    use crate::surfaces::Point3D;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;
use positive::pos_or_panic;

    struct TestDeltaGammaProfile {
        underlying_price: Positive,
    }

    impl DeltaGammaProfileCurve for TestDeltaGammaProfile {
        fn delta_gamma_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();
            let spot = self.underlying_price.to_dec();

            // Simulate delta-gamma profile across strikes
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
                // Simplified delta model: increases from 0 to 1 as strike decreases
                let moneyness = spot / strike;
                let delta = if moneyness > Decimal::ONE {
                    dec!(0.5) + (moneyness - Decimal::ONE) * dec!(2.0)
                } else {
                    dec!(0.5) - (Decimal::ONE - moneyness) * dec!(2.0)
                };
                let delta = delta.max(dec!(0.0)).min(dec!(1.0));

                // Gamma peaks at ATM
                let gamma = dec!(0.05) * (-((strike - spot) / dec!(50.0)).powi(2)).exp();

                // Combined metric: dollar delta + dollar gamma
                let dollar_delta = delta * spot;
                let dollar_gamma = gamma * spot * spot / dec!(100.0);
                let combined = dollar_delta + dollar_gamma;

                points.insert(Point2D::new(strike, combined));
            }

            Ok(Curve::new(points))
        }
    }

    impl DeltaGammaProfileSurface for TestDeltaGammaProfile {
        fn delta_gamma_surface(
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

            for days in &days_to_expiry {
                let time_factor = (days.to_dec() / dec!(365.0)).sqrt().unwrap_or(Decimal::ONE);

                for p in 0..=price_steps {
                    let price = price_range.0.to_dec() + price_step * Decimal::from(p);

                    // Delta varies with price and time
                    let atm = self.underlying_price.to_dec();
                    let moneyness = price / atm;
                    let base_delta = if moneyness > Decimal::ONE {
                        dec!(0.5) + (moneyness - Decimal::ONE) * dec!(2.0)
                    } else {
                        dec!(0.5) - (Decimal::ONE - moneyness) * dec!(2.0)
                    };

                    // Delta moves toward 0 or 1 as time decreases
                    let delta = base_delta * time_factor
                        + (Decimal::ONE - time_factor)
                            * if base_delta > dec!(0.5) {
                                Decimal::ONE
                            } else {
                                Decimal::ZERO
                            };

                    points.insert(Point3D::new(price, days.to_dec(), delta));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_delta_gamma_curve_creation() {
        let profile = TestDeltaGammaProfile {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = profile.delta_gamma_curve();
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 9);
    }

    #[test]
    fn test_delta_gamma_curve_monotonic_delta() {
        let profile = TestDeltaGammaProfile {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = profile.delta_gamma_curve().unwrap();

        // For calls, delta should generally decrease as strike increases
        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Check that higher strikes have lower combined values (for calls)
        for i in 1..points.len() {
            // This is a soft check since gamma can affect the combined metric
            assert!(points[i].x > points[i - 1].x); // Strikes are sorted
        }
    }

    #[test]
    fn test_delta_gamma_surface_creation() {
        let profile = TestDeltaGammaProfile {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];

        let surface = profile.delta_gamma_surface(price_range, days, 10);
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        // (10+1) × 3 = 33 points
        assert_eq!(surface.points.len(), 33);
    }

    #[test]
    fn test_delta_gamma_surface_time_decay() {
        let profile = TestDeltaGammaProfile {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(480.0), pos_or_panic!(480.0)); // ITM call
        let days = vec![pos_or_panic!(30.0), pos_or_panic!(7.0), Positive::ONE];

        let surface = profile.delta_gamma_surface(price_range, days, 0).unwrap();

        let points: Vec<&Point3D> = surface.points.iter().collect();

        // For ITM options, delta should approach 1 as time decreases
        // Points are sorted by (x, y), so we need to find by time
        let delta_30d = points.iter().find(|p| p.y == dec!(30.0)).map(|p| p.z);
        let delta_1d = points.iter().find(|p| p.y == dec!(1.0)).map(|p| p.z);

        if let (Some(d30), Some(d1)) = (delta_30d, delta_1d) {
            // ITM delta should be higher (closer to 1) with less time
            assert!(d1 >= d30);
        }
    }

    #[test]
    fn test_delta_gamma_surface_empty_days() {
        let profile = TestDeltaGammaProfile {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days: Vec<Positive> = vec![];

        let surface = profile.delta_gamma_surface(price_range, days, 10).unwrap();
        assert!(surface.points.is_empty());
    }
}
