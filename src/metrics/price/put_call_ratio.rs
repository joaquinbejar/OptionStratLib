/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Put/Call Ratio Metrics
//!
//! This module provides traits for computing Put/Call Ratio curves,
//! which are essential for understanding market positioning.
//!
//! ## Overview
//!
//! The Put/Call Ratio is the ratio between the put open interest and the call open
//! interest.
//! It is often used to measure the market sentiment. Generally speaking, traders buying
//! more puts than calls indicates a bearish market. Conversely, traders buying more calls
//! than puts indicates a bullish market.
//!
//! ## Mathematical Background
//!
//! The following common Put/Call Ratio measures are provided:
//!
//! ### Premium Weighted Put/Call Ratio
//! ```text
//! Put/Call Ratio = Put mid price / Call mid price
//! where Put mid price = (Put Bid + Put Ask) / 2 and
//! Call mid price = (Call Bid + Call Ask) / 2
//! ```
//!
//! ## Curve Representation
//!
//! The curve shows Put/Call Ratio across strike prices:
//! - **X-axis**: Strike price
//! - **Y-axis**: Put/Call Ratio

use crate::curves::Curve;
use crate::error::CurveError;

/// A trait defining a Put/Call ratio representation.
///
/// The `PutCallRatioCurve` trait is designed to encapsulate the concept of an
/// indicator of whether the investors are placing more bets on prices falling
/// or rising. It has been conceived to be a measure of market sentiment.
/// This trait establishes the foundation for representing and
/// retrieving the put/call ratio in the form of a mathematical curve.
///
/// # Overview
/// Implementors of this trait are required to provide the following methods:
/// - `premium_weighted_pcr`: computes and returns a `Curve` object representing
///   the put/call ratio calculated based on option's premium.
///
/// The `Curve` struct is a mathematical representation of the put/call ratio, where the
/// x-axis typically corresponds to the strike price, and the y-axis corresponds to
/// the put/call ratio.
/// The premium weighted put/call ratio is calculated according to the following formula:
/// ```math
/// \text{premium weighted PCR} = \frac{\text{put mid price}}{\text{call mid price}}
/// ```
///
/// # Usage
/// This trait serves as the basis for constructing and analyzing Put/Call ratio
/// in applications such as:
/// - Financial derivatives modeling
/// - Options pricing engines
/// - Quantitative analysis of market data
///
/// # Required Methods
/// - **`premium_weighted_pcr(&self) -> Curve`**
///   - Computes and returns the premium-based put/call ratio as a `Curve`.
///   - The returned `Ok(Curve)` can be used for graphical representation, numerical
///     analysis, or further mathematical operations, such as interpolation or
///     transformations.
///   - Returns a `CurveError` if it is not possible to calculate the Put/Call Ratio.
///
/// # Integration with Other Modules
/// The `PutCallRatioCurve` trait makes use of the `Curve` struct, defined in the
/// `crate::curves` module. The `Curve` provides the mathematical framework
/// necessary for representing and manipulating the Put/Call ratio data. High-quality
/// precision (via the use of `Decimal` and ordered points) ensures that the
/// output from the `premium_weighted_pcr` methods is reliable and suitable for scientific
/// or financial applications.
///
/// # See Also
/// - [`crate::curves::Curve`]: The fundamental mathematical representation of
///   the Put/Call ratio.
/// - [`crate::curves::Point2D`]: The structure representing individual points
///   in the `Curve`.
///
/// # Examples
/// Users can implement this trait and provide their specific logic for generating
/// a `Curve` corresponding to the put/call ratio. The put/call ratio can be implemented
/// as premium-weighted or open interest based.
///
/// ```rust
/// use std::collections::BTreeSet;
/// use rust_decimal::Decimal;
/// use optionstratlib::curves::Curve;
/// use optionstratlib::error::CurveError;
/// use optionstratlib::metrics::PutCallRatioCurve;
///
/// struct MyPcr;
///
/// impl PutCallRatioCurve for MyPcr {
///     fn premium_weighted_pcr(&self) -> Result<Curve, CurveError> {
///         // Custom logic to build and return a Curve representing the premium
///         // weighted put/call ratio
///         let curve = Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) };
///         Ok(curve)
///     }
/// }
/// ```
///
/// This enables integration of user-defined volatility models with the broader
/// ecosystem of mathematical and financial tools that utilize the `Curve` data type.
pub trait PutCallRatioCurve {
    /// Computes and returns a curve representing the premium-weighted Put/Call ratio.
    ///
    /// # Returns
    /// - A [`Curve`] object that models the Put/Call ratio. The x-axis typically
    ///   represents the option's strike, while the y-axis represents the premium-weighted
    ///   Put/Call ratio.
    /// - A [`CurveError`] object if the Put/Call Ratio calculation is not possible.
    ///   
    /// # Note
    /// - The `Curve` returned should ideally conform to the constraints and
    ///   ordering requirements specified in the `Curve` documentation.
    fn premium_weighted_pcr(&self) -> Result<Curve, CurveError>;
}

#[cfg(test)]
mod tests_put_call_ratio_traits {
    use super::*;
    use crate::{curves::Point2D, utils::Len};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestPutCallRatio;

    impl PutCallRatioCurve for TestPutCallRatio {
        fn premium_weighted_pcr(&self) -> Result<Curve, CurveError> {
            let curve = create_pcr_sample_curve();
            Ok(curve)
        }
    }

    // Sample Put/Call ratio curve creation
    fn create_pcr_sample_curve() -> Curve {
        let mut points = BTreeSet::new();
        points.insert(Point2D::new(dec!(5750.0), dec!(0.622))); // calls are more expensive
        points.insert(Point2D::new(dec!(5760.0), dec!(0.719)));
        points.insert(Point2D::new(dec!(5770.0), dec!(0.834)));
        points.insert(Point2D::new(dec!(5780.0), dec!(0.971))); // nearly price parity ATM
        points.insert(Point2D::new(dec!(5790.0), dec!(1.136))); // puts are now more expensive

        Curve {
            points,
            x_range: (dec!(5750.0), dec!(5790.0)),
        }
    }

    #[test]
    fn test_put_call_ratio_implementation() {
        let put_call_ratio = TestPutCallRatio;
        let curve_premium_weighted_pcr = put_call_ratio.premium_weighted_pcr().unwrap();

        // Verify the curve has expected properties
        assert_eq!(curve_premium_weighted_pcr.points.len(), 5);
        assert_eq!(
            curve_premium_weighted_pcr.x_range,
            (dec!(5750.0), dec!(5790.0))
        );

        // Check specific points
        let points: Vec<&Point2D> = curve_premium_weighted_pcr.points.iter().collect();
        assert_eq!(points[0].x, dec!(5750.0));
        assert_eq!(points[0].y, dec!(0.622));
        assert_eq!(points[2].x, dec!(5770.0));
        assert_eq!(points[2].y, dec!(0.834));
        assert_eq!(points[4].x, dec!(5790.0));
        assert_eq!(points[4].y, dec!(1.136));
    }

    #[test]
    fn test_put_call_ratio_with_empty_curve() {
        struct EmptyPutCallRatio;

        impl PutCallRatioCurve for EmptyPutCallRatio {
            fn premium_weighted_pcr(&self) -> Result<Curve, CurveError> {
                let curve = Curve {
                    points: BTreeSet::new(),
                    x_range: (Decimal::ZERO, Decimal::ZERO),
                };
                Ok(curve)
            }
        }

        let put_call_ratio = EmptyPutCallRatio;
        let premium_weighted_pcr_curve = put_call_ratio.premium_weighted_pcr().unwrap();

        assert!(premium_weighted_pcr_curve.is_empty());
        assert_eq!(
            premium_weighted_pcr_curve.x_range,
            (Decimal::ZERO, Decimal::ZERO)
        );
    }
}
