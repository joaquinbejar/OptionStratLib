/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Volume Profile Metrics
//!
//! This module provides traits for computing volume profile curves and surfaces,
//! which are essential for understanding trading activity and market interest.
//!
//! ## Overview
//!
//! Volume profile shows the distribution of trading activity across different
//! price levels and time periods. It helps identify:
//!
//! - **High volume nodes**: Price levels with significant trading activity
//! - **Low volume nodes**: Price levels with minimal activity (potential breakout zones)
//! - **Value areas**: Where most trading occurs
//!
//! ## Curve Representation
//!
//! The curve shows volume distribution by strike:
//! - **X-axis**: Strike price
//! - **Y-axis**: Trading volume (number of contracts)
//!
//! ## Surface Representation
//!
//! The surface shows volume evolution over time:
//! - **X-axis**: Strike price
//! - **Y-axis**: Time (days)
//! - **Z-axis**: Trading volume

use crate::Positive;
use crate::curves::Curve;
use crate::error::CurveError;
use crate::error::SurfaceError;
use crate::surfaces::Surface;

/// A trait for computing volume profile curves by strike price.
///
/// The volume curve shows how trading activity is distributed across
/// different strike prices, helping identify active trading zones.
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Trading volume (number of contracts)
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::VolumeProfileCurve;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let volume_curve = chain.volume_profile_curve()?;
///
/// // Find strike with highest volume
/// let max_volume = volume_curve.points.iter()
///     .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
/// ```
pub trait VolumeProfileCurve {
    /// Computes the volume profile curve by strike price.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The volume curve with strike on x-axis and
    ///   volume on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid volume data
    /// - The option chain is empty
    fn volume_profile_curve(&self) -> Result<Curve, CurveError>;
}

/// A trait for computing volume profile surfaces.
///
/// The volume surface shows how trading activity evolves across both
/// strike prices and time, providing insights into market dynamics.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Days (time dimension)
/// - **Z-axis**: Trading volume
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::VolumeProfileSurface;
/// use optionstratlib::pos_or_panic;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let days = vec![Positive::ONE, pos_or_panic!(5.0), pos_or_panic!(10.0), pos_or_panic!(20.0)];
/// let volume_surface = chain.volume_profile_surface(days)?;
/// ```
pub trait VolumeProfileSurface {
    /// Computes the volume profile surface (strike vs time).
    ///
    /// # Parameters
    ///
    /// - `days`: Vector of time points (in days) to include in the surface
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The volume surface with strike on x-axis,
    ///   days on y-axis, and volume on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `SurfaceError::ConstructionError` if:
    /// - No options have valid volume data
    /// - The days vector is empty
    fn volume_profile_surface(&self, days: Vec<Positive>) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_volume_profile {
    use super::*;
    use crate::curves::Point2D;

    use crate::surfaces::Point3D;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;
    use positive::pos_or_panic;

    struct TestVolumeProfile;

    impl VolumeProfileCurve for TestVolumeProfile {
        fn volume_profile_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();

            // Simulate typical volume pattern: higher near ATM
            let strikes = [
                (dec!(380.0), dec!(500.0)),
                (dec!(400.0), dec!(1200.0)),
                (dec!(420.0), dec!(2500.0)),
                (dec!(440.0), dec!(4000.0)),
                (dec!(450.0), dec!(5000.0)), // ATM - highest volume
                (dec!(460.0), dec!(4200.0)),
                (dec!(480.0), dec!(2800.0)),
                (dec!(500.0), dec!(1500.0)),
                (dec!(520.0), dec!(600.0)),
            ];

            for (strike, volume) in strikes {
                points.insert(Point2D::new(strike, volume));
            }

            Ok(Curve::new(points))
        }
    }

    impl VolumeProfileSurface for TestVolumeProfile {
        fn volume_profile_surface(&self, days: Vec<Positive>) -> Result<Surface, SurfaceError> {
            let mut points = BTreeSet::new();

            let strikes = [
                dec!(400.0),
                dec!(420.0),
                dec!(440.0),
                dec!(450.0),
                dec!(460.0),
                dec!(480.0),
                dec!(500.0),
            ];

            let base_volumes = [
                dec!(1200.0),
                dec!(2500.0),
                dec!(4000.0),
                dec!(5000.0),
                dec!(4200.0),
                dec!(2800.0),
                dec!(1500.0),
            ];

            for day in &days {
                // Volume typically increases closer to expiration
                let time_factor = Decimal::ONE + (dec!(30.0) - day.to_dec()) / dec!(30.0);

                for (strike, base_vol) in strikes.iter().zip(base_volumes.iter()) {
                    let volume = *base_vol * time_factor;
                    points.insert(Point3D::new(*strike, day.to_dec(), volume));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_volume_profile_curve_creation() {
        let volume = TestVolumeProfile;
        let curve = volume.volume_profile_curve();
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 9);
    }

    #[test]
    fn test_volume_profile_atm_highest() {
        let volume = TestVolumeProfile;
        let curve = volume.volume_profile_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Find maximum volume
        let max_vol = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        if let Some(max) = max_vol {
            // ATM should have highest volume (around 450)
            assert_eq!(max.x, dec!(450.0));
            assert_eq!(max.y, dec!(5000.0));
        }
    }

    #[test]
    fn test_volume_profile_surface_creation() {
        let volume = TestVolumeProfile;
        let days = vec![pos_or_panic!(5.0), pos_or_panic!(10.0), pos_or_panic!(20.0)];

        let surface = volume.volume_profile_surface(days);
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        // 7 strikes × 3 days = 21 points
        assert_eq!(surface.points.len(), 21);
    }

    #[test]
    fn test_volume_profile_surface_time_effect() {
        let volume = TestVolumeProfile;
        let days = vec![pos_or_panic!(5.0), pos_or_panic!(25.0)];

        let surface = volume.volume_profile_surface(days).unwrap();

        // Find ATM volume at different times
        let points: Vec<&Point3D> = surface.points.iter().collect();

        let vol_5d = points
            .iter()
            .find(|p| p.x == dec!(450.0) && p.y == dec!(5.0))
            .map(|p| p.z);
        let vol_25d = points
            .iter()
            .find(|p| p.x == dec!(450.0) && p.y == dec!(25.0))
            .map(|p| p.z);

        // Volume should be higher closer to expiration (5 days vs 25 days)
        if let (Some(v5), Some(v25)) = (vol_5d, vol_25d) {
            assert!(v5 > v25);
        }
    }

    #[test]
    fn test_volume_profile_surface_empty_days() {
        let volume = TestVolumeProfile;
        let days: Vec<Positive> = vec![];

        let surface = volume.volume_profile_surface(days).unwrap();
        assert!(surface.points.is_empty());
    }

    #[test]
    fn test_volume_profile_positive_values() {
        let volume = TestVolumeProfile;
        let curve = volume.volume_profile_curve().unwrap();

        // All volumes should be positive
        for point in curve.points.iter() {
            assert!(point.y > Decimal::ZERO);
        }
    }
}
