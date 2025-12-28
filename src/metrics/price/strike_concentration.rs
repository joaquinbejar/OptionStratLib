/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Strike Concentration Metrics
//!
//! This module provides traits for computing Strike Concentration curves,
//! which are essential for understanding market positioning.
//!
//! ## Overview
//!
//! The Strike Concentration identifies specific strike prices with unusually high open
//! interest or trading volume. It looks for clusters of activity where a large number of
//! contracts are concentrated.
//! It is often used to understand which are the key price levels where market positioning
//! is the heaviest.
//!
//! ## Mathematical Background
//!
//! The following common Strike Concentration measures are provided:
//!
//! ### Premium Weighted Strike Concentration
//!
//! ```text
//! Strike Concentration = Strike premium / Mean average of all chain's strike premiums
//! where Strike premium = Put mid price + Call mid price
//! ```
//!
//! ## Curve Representation
//!
//! The curve shows Strike Concentration across strike prices:
//! - **X-axis**: Strike price
//! - **Y-axis**: Strike Concentration

use crate::curves::Curve;
use crate::error::CurveError;

/// A trait defining a Strike Concentration representation.
///
/// # Overview
/// Implementors of this trait are required to provide the following methods:
/// - `premium_concentration`: computes and returns a `Curve` object representing
///   the strike concentration calculated based on option's premium.
///
/// The `Curve` struct is a mathematical representation of the strike concentration, where the
/// x-axis typically corresponds to the strike price, and the y-axis corresponds to
/// the strike concentration.
///
/// # Required Methods
/// - **`premium_concentration(&self) -> Result<Curve, CurveError>`**
///   - Computes and returns the premium-based strike concentration ratio as a `Curve`.
///   - The returned `Ok(Curve)` can be used for graphical representation, numerical
///     analysis, or further mathematical operations, such as interpolation or
///     transformations.
///   - Returns a `CurveError` if it is not possible to calculate the strike concentration.
///
/// # Integration with Other Modules
/// The `StrikeConcentrationCurve` trait makes use of the `Curve` struct, defined in the
/// `crate::curves` module. The `Curve` provides the mathematical framework
/// necessary for representing and manipulating the Strike Concentration data. High-quality
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
/// a `Curve` corresponding to the strike concentration. The strike concentration can be implemented
/// as premium-weighted.
///
/// ```rust
/// use std::collections::BTreeSet;
/// use rust_decimal::Decimal;
/// use optionstratlib::curves::Curve;
/// use optionstratlib::error::CurveError;
/// use optionstratlib::metrics::StrikeConcentrationCurve;
///
/// struct MyStrikeConcentration;
///
/// impl StrikeConcentrationCurve for MyStrikeConcentration {
///     fn premium_concentration(&self) -> Result<Curve, CurveError> {
///         // Custom logic to build and return a Curve representing the premium
///         // weighted strike concentration
///         let curve = Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) };
///         Ok(curve)
///     }
/// }
/// ```
///
/// This enables integration of user-defined volatility models with the broader
/// ecosystem of mathematical and financial tools that utilize the `Curve` data type.
pub trait StrikeConcentrationCurve {
    /// Computes and returns a curve representing the premium-weighted Strike Concentration.
    ///
    /// # Returns
    /// - A [`Curve`] object that models the Strike Concentration. The x-axis typically
    ///   represents the option's strike, while the y-axis represents the premium-weighted
    ///   Strike Concentration.
    /// - A [`CurveError`] object if the Strike Concentration calculation is not possible.
    ///   
    /// # Note
    /// - The `Curve` returned should ideally conform to the constraints and
    ///   ordering requirements specified in the `Curve` documentation.
    fn premium_concentration(&self) -> Result<Curve, CurveError>;
}

#[cfg(test)]
mod tests_strike_concentration_traits {
    use super::*;
    use crate::{curves::Point2D, utils::Len};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestStrikeConcentration;

    impl StrikeConcentrationCurve for TestStrikeConcentration {
        fn premium_concentration(&self) -> Result<Curve, CurveError> {
            let curve = create_strike_concentration_sample_curve();
            Ok(curve)
        }
    }

    // Sample Strike Concentration curve creation
    fn create_strike_concentration_sample_curve() -> Curve {
        let mut points = BTreeSet::new();
        points.insert(Point2D::new(dec!(5750.0), dec!(1.05727))); // highest concentration
        points.insert(Point2D::new(dec!(5760.0), dec!(1.03272)));
        points.insert(Point2D::new(dec!(5770.0), dec!(1.01157)));
        points.insert(Point2D::new(dec!(5780.0), dec!(0.99399))); // ATM
        points.insert(Point2D::new(dec!(5790.0), dec!(0.98291)));

        Curve {
            points,
            x_range: (dec!(5750.0), dec!(5790.0)),
        }
    }

    #[test]
    fn test_strike_concentration_implementation() {
        let strike_concentration = TestStrikeConcentration;
        let curve_strike_concentration = strike_concentration.premium_concentration().unwrap();

        // Verify the curve has expected properties
        assert_eq!(curve_strike_concentration.points.len(), 5);
        assert_eq!(
            curve_strike_concentration.x_range,
            (dec!(5750.0), dec!(5790.0))
        );

        // Check specific points
        let points: Vec<&Point2D> = curve_strike_concentration.points.iter().collect();
        assert_eq!(points[0].x, dec!(5750.0));
        assert_eq!(points[0].y, dec!(1.05727));
        assert_eq!(points[2].x, dec!(5770.0));
        assert_eq!(points[2].y, dec!(1.01157));
        assert_eq!(points[4].x, dec!(5790.0));
        assert_eq!(points[4].y, dec!(0.98291));
    }

    #[test]
    fn test_strike_concentration_with_empty_curve() {
        struct EmptyStrikeConcentration;

        impl StrikeConcentrationCurve for EmptyStrikeConcentration {
            fn premium_concentration(&self) -> Result<Curve, CurveError> {
                let curve = Curve {
                    points: BTreeSet::new(),
                    x_range: (Decimal::ZERO, Decimal::ZERO),
                };
                Ok(curve)
            }
        }

        let strike_concentration = EmptyStrikeConcentration;
        let strike_concentration_curve = strike_concentration.premium_concentration().unwrap();

        assert!(strike_concentration_curve.is_empty());
        assert_eq!(
            strike_concentration_curve.x_range,
            (Decimal::ZERO, Decimal::ZERO)
        );
    }
}
