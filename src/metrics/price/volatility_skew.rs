/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/12/25
******************************************************************************/

use crate::curves::Curve;
/// A trait defining a volatility skew representation.
///
/// The `VolatilitySkew` trait is designed to encapsulate the concept of a
/// volatility skew, a key phenomenon in derivatives pricing and financial
/// modeling. This trait establishes the foundation for representing and
/// retrieving the volatility skew in the form of a mathematical curve.
///
/// # Overview
/// Implementors of this trait are required to provide the `volatility_skew`
/// method, which computes and returns a `Curve` object representing the
/// volatility skew.
/// The `Curve` struct is a mathematical representation of the skew, where the
/// x-axis typically corresponds to the moneyness, and the y-axis corresponds to
/// the implied volatility.
/// The moneyness is calculated according to the following formula:
/// ```math
/// \text{moneyness} = \lparen\frac{\text{strike\_price}}
/// {\text{underlying\_price}}-1\rparen \cdot 100
/// ```
///
/// # Usage
/// This trait serves as the basis for constructing and analyzing volatility
/// skews in applications such as:
/// - Financial derivatives modeling
/// - Options pricing engines
/// - Quantitative analysis of market data
///
/// # Required Methods
/// - **`volatility_skew(&self) -> Curve`**
///   - Computes and returns the volatility skew as a `Curve`.
///   - The returned `Curve` can be used for graphical representation, numerical
///     analysis, or further mathematical operations, such as interpolation or
///     transformations.
///
/// # Integration with Other Modules
/// The `VolatilitySkew` trait makes use of the `Curve` struct, defined in the
/// `crate::curves` module. The `Curve` provides the mathematical framework
/// necessary for representing and manipulating the skew data. High-quality
/// precision (via the use of `Decimal` and ordered points) ensures that the
/// output from the `volatility_skew` method is reliable and suitable for
/// scientific or financial applications.
///
/// # See Also
/// - [`crate::curves::Curve`]: The fundamental mathematical representation of
///   the volatility skew.
/// - [`crate::curves::Point2D`]: The structure representing individual points
///   in the `Curve`.
///
/// # Examples
/// To define a custom volatility model, users can implement this trait and
/// provide their specific logic for generating a `Curve` corresponding to the
/// skew.
///
/// ```rust
/// use std::collections::BTreeSet;
/// use rust_decimal::Decimal;
/// use optionstratlib::curves::Curve;
/// use optionstratlib::error::greeks::CalculationErrorKind::DecimalError;
/// use optionstratlib::metrics::VolatilitySkew;
///
/// struct MySkew;
///
/// impl VolatilitySkew for MySkew {
///     fn volatility_skew(&self) -> Curve {
///         // Custom logic to build and return a Curve representing the skew
///         Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) }
///     }
/// }
/// ```
///
/// This enables integration of user-defined volatility models with the broader
/// ecosystem of mathematical and financial tools that utilize the `Curve` data type.
pub trait VolatilitySkew {
    /// Computes and returns a curve representing the volatility skew.
    ///
    /// # Returns
    /// - A [`Curve`] object that models the volatility skew. The x-axis typically
    ///   represents the moneyness, while the y-axis represents implied volatility.
    ///   
    /// # Note
    /// - The `Curve` returned should ideally conform to the constraints and
    ///   ordering requirements specified in the `Curve` documentation.
    fn volatility_skew(&self) -> Curve;
}

#[cfg(test)]
mod tests_volatility_skew_traits {
    use super::*;
    use crate::curves::Point2D;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestSkew;

    impl VolatilitySkew for TestSkew {
        fn volatility_skew(&self) -> Curve {
            create_sample_curve()
        }
    }

    fn create_sample_curve() -> Curve {
        let mut points = BTreeSet::new();
        points.insert(Point2D::new(dec!(90.0), dec!(0.25)));
        points.insert(Point2D::new(dec!(95.0), dec!(0.22)));
        points.insert(Point2D::new(dec!(100.0), dec!(0.20)));
        points.insert(Point2D::new(dec!(105.0), dec!(0.22)));
        points.insert(Point2D::new(dec!(110.0), dec!(0.25)));

        Curve {
            points,
            x_range: (dec!(90.0), dec!(110.0)),
        }
    }

    #[test]
    fn test_volatility_skew_implementation() {
        let skew = TestSkew;
        let curve = skew.volatility_skew();

        // Verify the curve has expected properties
        assert_eq!(curve.points.len(), 5);
        assert_eq!(curve.x_range, (dec!(90.0), dec!(110.0)));

        // Check specific points
        let points: Vec<&Point2D> = curve.points.iter().collect();
        assert_eq!(points[0].x, dec!(90.0));
        assert_eq!(points[0].y, dec!(0.25));
        assert_eq!(points[2].x, dec!(100.0));
        assert_eq!(points[2].y, dec!(0.20));
        assert_eq!(points[4].x, dec!(110.0));
        assert_eq!(points[4].y, dec!(0.25));
    }

    #[test]
    fn test_volatility_skew() {
        let skew = TestSkew;
        let curve = skew.volatility_skew();

        assert_eq!(curve.points.len(), 5);
        assert_eq!(curve.x_range, (dec!(90.0), dec!(110.0)));

        let points: Vec<&Point2D> = curve.points.iter().collect();
        assert_eq!(points[0].x, dec!(90.0));
        assert_eq!(points[0].y, dec!(0.25));
        assert_eq!(points[2].x, dec!(100.0));
        assert_eq!(points[2].y, dec!(0.20));
        assert_eq!(points[4].x, dec!(110.0));
        assert_eq!(points[4].y, dec!(0.25));
    }

    #[test]
    fn test_volatility_skew_with_empty_curve() {
        struct EmptySkew;

        impl VolatilitySkew for EmptySkew {
            fn volatility_skew(&self) -> Curve {
                Curve {
                    points: BTreeSet::new(),
                    x_range: (Decimal::ZERO, Decimal::ZERO),
                }
            }
        }

        let skew = EmptySkew;
        let curve = skew.volatility_skew();

        assert!(curve.points.is_empty());
        assert_eq!(curve.x_range, (Decimal::ZERO, Decimal::ZERO));
    }

    #[test]
    fn test_volatility_skew_with_multiple_points() {
        struct MultiPointSkew;

        impl VolatilitySkew for MultiPointSkew {
            fn volatility_skew(&self) -> Curve {
                let mut points = BTreeSet::new();
                // Simulate a typical volatility skew pattern
                points.insert(Point2D::new(dec!(-10.0), dec!(0.30))); // OTM put
                points.insert(Point2D::new(dec!(-5.0), dec!(0.25)));
                points.insert(Point2D::new(dec!(0.0), dec!(0.20))); // ATM
                points.insert(Point2D::new(dec!(5.0), dec!(0.22)));
                points.insert(Point2D::new(dec!(10.0), dec!(0.25))); // OTM call

                Curve {
                    points,
                    x_range: (dec!(-10.0), dec!(10.0)),
                }
            }
        }

        let skew = MultiPointSkew;
        let curve = skew.volatility_skew();

        assert_eq!(curve.points.len(), 5);
        assert_eq!(curve.x_range, (dec!(-10.0), dec!(10.0)));

        // Verify the skew pattern (higher IV for OTM options)
        let points: Vec<&Point2D> = curve.points.iter().collect();
        // ATM should have lowest IV
        assert_eq!(points[2].x, dec!(0.0));
        assert_eq!(points[2].y, dec!(0.20));
    }

    #[test]
    fn test_volatility_skew_negative_moneyness() {
        struct NegativeSkew;

        impl VolatilitySkew for NegativeSkew {
            fn volatility_skew(&self) -> Curve {
                let mut points = BTreeSet::new();
                // Only OTM puts (negative moneyness)
                points.insert(Point2D::new(dec!(-20.0), dec!(0.40)));
                points.insert(Point2D::new(dec!(-15.0), dec!(0.35)));
                points.insert(Point2D::new(dec!(-10.0), dec!(0.30)));
                points.insert(Point2D::new(dec!(-5.0), dec!(0.25)));

                Curve {
                    points,
                    x_range: (dec!(-20.0), dec!(-5.0)),
                }
            }
        }

        let skew = NegativeSkew;
        let curve = skew.volatility_skew();

        assert_eq!(curve.points.len(), 4);
        // All points should have negative x (OTM puts)
        for point in curve.points.iter() {
            assert!(point.x < Decimal::ZERO);
        }
    }
}
