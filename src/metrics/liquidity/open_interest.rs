/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Open Interest Distribution Metrics
//!
//! This module provides traits for computing open interest distribution curves,
//! which are essential for understanding market positioning and potential
//! support/resistance levels.
//!
//! ## Overview
//!
//! Open interest represents the total number of outstanding option contracts
//! that have not been settled. It provides insights into:
//!
//! - **Market positioning**: Where traders have established positions
//! - **Support/Resistance levels**: High OI strikes often act as magnets
//! - **Max pain theory**: Strike where most options expire worthless
//! - **Market sentiment**: Put/Call OI ratio indicates bullish/bearish bias
//!
//! ## Mathematical Background
//!
//! ### Total Open Interest
//! ```text
//! Total OI at Strike K = Call OI(K) + Put OI(K)
//! ```
//!
//! ### Put/Call OI Ratio
//! ```text
//! P/C Ratio = Put OI / Call OI
//! ```
//! - Ratio > 1: More puts than calls (bearish positioning)
//! - Ratio < 1: More calls than puts (bullish positioning)
//!
//! ## Curve Representation
//!
//! The curve shows OI distribution across strikes:
//! - **X-axis**: Strike price
//! - **Y-axis**: Open interest (number of contracts)

use crate::curves::Curve;
use crate::error::CurveError;

/// A trait for computing open interest distribution curves by strike price.
///
/// The OI curve shows how outstanding contracts are distributed across
/// different strike prices, helping identify key levels and market positioning.
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Open interest (number of contracts)
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::OpenInterestCurve;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let oi_curve = chain.open_interest_curve()?;
///
/// // Find strike with maximum OI (potential support/resistance)
/// let max_oi = oi_curve.points.iter()
///     .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
/// ```
pub trait OpenInterestCurve {
    /// Computes the open interest distribution curve by strike price.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The OI curve with strike on x-axis and
    ///   open interest on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid open interest data
    /// - The option chain is empty
    fn open_interest_curve(&self) -> Result<Curve, CurveError>;
}

#[cfg(test)]
mod tests_open_interest {
    use super::*;
    use crate::curves::Point2D;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestOpenInterest;

    impl OpenInterestCurve for TestOpenInterest {
        fn open_interest_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();

            // Simulate typical OI pattern with concentration at round strikes
            let strikes = [
                (dec!(380.0), dec!(2500.0)),
                (dec!(400.0), dec!(8000.0)), // Round strike - higher OI
                (dec!(420.0), dec!(5000.0)),
                (dec!(440.0), dec!(6500.0)),
                (dec!(450.0), dec!(15000.0)), // ATM round strike - highest OI
                (dec!(460.0), dec!(7000.0)),
                (dec!(480.0), dec!(5500.0)),
                (dec!(500.0), dec!(10000.0)), // Round strike - higher OI
                (dec!(520.0), dec!(3000.0)),
            ];

            for (strike, oi) in strikes {
                points.insert(Point2D::new(strike, oi));
            }

            Ok(Curve::new(points))
        }
    }

    #[test]
    fn test_open_interest_curve_creation() {
        let oi = TestOpenInterest;
        let curve = oi.open_interest_curve();
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 9);
    }

    #[test]
    fn test_open_interest_max_at_atm() {
        let oi = TestOpenInterest;
        let curve = oi.open_interest_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Find maximum OI
        let max_oi = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        if let Some(max) = max_oi {
            // ATM round strike should have highest OI
            assert_eq!(max.x, dec!(450.0));
            assert_eq!(max.y, dec!(15000.0));
        }
    }

    #[test]
    fn test_open_interest_round_strikes_higher() {
        let oi = TestOpenInterest;
        let curve = oi.open_interest_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Round strikes (400, 450, 500) should have higher OI than neighbors
        let oi_400 = points.iter().find(|p| p.x == dec!(400.0)).unwrap().y;
        let oi_420 = points.iter().find(|p| p.x == dec!(420.0)).unwrap().y;
        let oi_380 = points.iter().find(|p| p.x == dec!(380.0)).unwrap().y;

        assert!(oi_400 > oi_420);
        assert!(oi_400 > oi_380);
    }

    #[test]
    fn test_open_interest_positive_values() {
        let oi = TestOpenInterest;
        let curve = oi.open_interest_curve().unwrap();

        // All OI values should be positive
        for point in curve.points.iter() {
            assert!(point.y > Decimal::ZERO);
        }
    }

    #[test]
    fn test_open_interest_strike_ordering() {
        let oi = TestOpenInterest;
        let curve = oi.open_interest_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x > points[i - 1].x);
        }
    }
}
