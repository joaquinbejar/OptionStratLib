use crate::curves::Point2D;
use crate::error::{CurvesError, InterpolationError};
use crate::geometrics::{BiLinearInterpolation, CubicInterpolation, GeometricObject, InterpolationType, LinearInterpolation, SplineInterpolation};
use rust_decimal::Decimal;

/// A trait for performing various types of interpolation on a set of 2D points.
///
/// # Overview
/// The `Interpolate` trait unifies methods for interpolating values in 2D Cartesian space.
/// Implementers of this trait must support linear, bilinear, cubic, and spline interpolation methods.
///
/// This trait is designed for use with numerical and graphical applications requiring
/// high-precision interpolation of data points. It provides functionality
/// to retrieve the points (`get_points`), interpolate a value (`interpolate`), and
/// find bracketing points for a given x-coordinate (`find_bracket_points`).
///
/// # Associated Methods
/// - [`get_points`](Self::get_points): Returns the collection of points for interpolation.
/// - [`interpolate`](Self::interpolate): Interpolates a value for a given x-coordinate using
///   a specified interpolation method.
/// - [`find_bracket_points`](Self::find_bracket_points): Identifies the pair of points
///   that bracket the target x-coordinate for interpolation.
///
/// # Requirements
/// Implementers must also implement the following traits:
/// - `LinearInterpolation`
/// - `BiLinearInterpolation`
/// - `CubicInterpolation`
/// - `SplineInterpolation`
///
/// These sub-traits are expected to define the actual interpolation algorithms for their
/// specific methods (e.g., linear interpolation, cubic spline interpolation, etc.).
///
/// # Error Handling
/// Methods in this trait return `CurvesError` to represent various issues during interpolation:
/// - **`InterpolationError`**: Indicates issues such as insufficient points, an x-coordinate
///   outside the valid range, or failure to bracket points for interpolation.
///
/// # Example Implementation
/// This trait is used to define a general interface for interpolation operations, which
/// can then be implemented by various structs managing interpolation algorithms.
///
pub trait Interpolate<Point, Input>:
    LinearInterpolation<Point, Input>
    + BiLinearInterpolation<Point, Input>
    + CubicInterpolation<Point, Input>
    + SplineInterpolation<Point, Input>
    + GeometricObject<Point>
{
    type Error;
    
    /// Interpolates a value at the specified x-coordinate using the given interpolation method.
    ///
    /// # Parameters
    /// - `x`: The target x-coordinate where interpolation is to be performed.
    /// - `interpolation_type`: Specifies the interpolation method to be used. This is provided
    ///   as an [`InterpolationType`] enum, which defines supported methods such as linear,
    ///   cubic, spline, and bilinear interpolation.
    ///
    /// # Returns
    /// * On Success: Returns a [`Point2D`] containing the interpolated (x, y) values.
    /// * On Error: Returns a `CurvesError` providing context on why the interpolation failed.
    ///    Possible reasons include:
    ///    - Insufficient points
    ///    - The target `x` is outside the range of points
    ///    - No bracketing points could be found
    ///
    /// # Example Behavior
    /// The method chooses the appropriate interpolation algorithm based on the provided
    /// `interpolation_type`. It relies on the sub-traits (`LinearInterpolation`, etc.) for
    /// the actual interpolation calculation.
    fn interpolate(
        &self,
        x: Decimal,
        interpolation_type: InterpolationType,
    ) -> Result<Point, Self::Error> {
        match interpolation_type {
            InterpolationType::Linear => self.linear_interpolate(x),
            InterpolationType::Cubic => self.cubic_interpolate(x),
            InterpolationType::Spline => self.spline_interpolate(x),
            InterpolationType::Bilinear => self.bilinear_interpolate(x),
        }
    }

    /// Finds the indices of two points that bracket the target x-coordinate.
    ///
    /// # Parameters
    /// - `x`: The target x-coordinate for which the bracketing points are sought.
    ///
    /// # Returns
    /// * On Success: Returns a tuple `(usize, usize)` representing the indices of the
    ///   bracketing points in the dataset.
    /// * On Error: Returns a `CurvesError` explaining the issue, such as:
    ///    - The dataset contains fewer than two points.
    ///    - The target `x` is outside the range of the dataset.
    ///    - No bracketing points could be identified.
    ///
    /// # Behavior
    /// This method assumes the points are sorted by their x-coordinate. It performs
    /// a linear search to locate the two consecutive points that enclose the target `x`.
    ///
    /// # Edge Cases
    /// - If fewer than two points are present, an error is returned.
    /// - If `x` is outside the domain of the dataset's x-coordinates (less than the minimum
    ///   x-coordinate or greater than the maximum), an error is returned.
    fn find_bracket_points(&self, x: Input) -> Result<(usize, usize), Self::Error> {
        let points = self.get_points();

        // Edge cases
        if points.len() < 2 {
            return Err(InterpolationError::StdError(
                "Need at least two points for interpolation".to_string(),
            ));
        }

        if x < points[0].x || x > points[points.len() - 1].x {
            return Err(InterpolationError::StdError(
                "x is outside the range of points".to_string(),
            ));
        }

        // Find points that bracket x
        for i in 0..points.len() - 1 {
            if points[i].x <= x && x <= points[i + 1].x {
                return Ok((i, i + 1));
            }
        }

        Err(InterpolationError::StdError(
            "Could not find bracketing points".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests_interpolate {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;
    use crate::error::InterpolationError;

    struct MockInterpolator {
        points: BTreeSet<Point2D>,
    }

    impl LinearInterpolation<Point2D, Decimal> for MockInterpolator {

        fn linear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError::Linear> {
            // Validate points first
            if self.points.len() < 2 {
                return Err(InterpolationError::Linear(
                    "Need at least two points for interpolation".to_string(),
                ));
            }
            Ok(Point2D::new(x, x))
        }
    }

    impl BiLinearInterpolation<Point2D, Decimal> for MockInterpolator {

        fn bilinear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError::Bilinear> {
            if self.points.len() < 4 {
                return Err(InterpolationError::Bilinear(
                    "Need at least four points for bilinear interpolation".to_string(),
                ));
            }
            Ok(Point2D::new(x, x))
        }
    }

    impl CubicInterpolation<Point2D, Decimal> for MockInterpolator {


        fn cubic_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError::Cubic> {
            if self.points.len() < 4 {
                return Err(InterpolationError::Cubic(
                    "Need at least four points for cubic interpolation".to_string(),
                ));
            }
            Ok(Point2D::new(x, x))
        }
    }

    impl SplineInterpolation<Point2D, Decimal> for MockInterpolator {
        
        fn spline_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError::Spline> {
            if self.points.len() < 3 {
                return Err(InterpolationError::Spline(
                    "Need at least three points for spline interpolation".to_string(),
                ));
            }
            Ok(Point2D::new(x, x))
        }
    }

    impl GeometricObject<Point2D> for MockInterpolator {
        type Error = ();

        fn get_points(&self) -> &BTreeSet<Point2D> {
            &self.points
        }

        fn from_vector(points: Vec<Point2D>) -> Self
        where
            Self: Sized,
        {
            unimplemented!()
        }

        fn construct<T>(method: T) -> Result<Self, Self::Error>
        where
            Self: Sized,
        {
            unimplemented!()
        }
    }

    impl Interpolate<Point2D, Decimal> for MockInterpolator { type Error = CurvesError; }



    fn create_mock_interpolator(points: BTreeSet<Point2D>) -> MockInterpolator {
        MockInterpolator { points }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_interpolate_empty_points() {
        let interpolator = create_mock_interpolator(BTreeSet::new());

        let linear = interpolator.interpolate(dec!(0.5), InterpolationType::Linear);
        assert!(linear.is_err());

        let bilinear = interpolator.interpolate(dec!(0.5), InterpolationType::Bilinear);
        assert!(bilinear.is_err());

        let cubic = interpolator.interpolate(dec!(0.5), InterpolationType::Cubic);
        assert!(cubic.is_err());

        let spline = interpolator.interpolate(dec!(0.5), InterpolationType::Spline);
        assert!(spline.is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_interpolate_insufficient_points() {
        // Test with only one point
        let interpolator =
            create_mock_interpolator(BTreeSet::from_iter(vec![Point2D::new(dec!(0), dec!(0))]));

        let linear = interpolator.interpolate(dec!(0.5), InterpolationType::Linear);
        assert!(linear.is_err());

        // Test with two points (should fail for all except linear)
        let interpolator = create_mock_interpolator(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
        ]));

        assert!(interpolator
            .interpolate(dec!(0.5), InterpolationType::Bilinear)
            .is_err());
        assert!(interpolator
            .interpolate(dec!(0.5), InterpolationType::Cubic)
            .is_err());
        assert!(interpolator
            .interpolate(dec!(0.5), InterpolationType::Spline)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_points() {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
        ]);
        let interpolator = create_mock_interpolator(points.clone());

        assert_eq!(
            interpolator.vector(),
            points.iter().collect::<Vec<_>>().as_slice()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_interpolate_routing() {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)), // Point 1
            Point2D::new(dec!(1), dec!(1)), // Point 2
            Point2D::new(dec!(2), dec!(2)), // Point 3
            Point2D::new(dec!(3), dec!(3)), // Point 4
        ]);
        let interpolator = create_mock_interpolator(points);
        let x = dec!(0.5);

        // Test that each interpolation type routes to its corresponding method
        let linear = interpolator
            .interpolate(x, InterpolationType::Linear)
            .unwrap();
        let bilinear = interpolator
            .interpolate(x, InterpolationType::Bilinear)
            .unwrap();
        let cubic = interpolator
            .interpolate(x, InterpolationType::Cubic)
            .unwrap();
        let spline = interpolator
            .interpolate(x, InterpolationType::Spline)
            .unwrap();

        // In our mock implementation, all methods return (x, x)
        assert_eq!(linear.x, x);
        assert_eq!(bilinear.x, x);
        assert_eq!(cubic.x, x);
        assert_eq!(spline.x, x);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_bracket_points_success() {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
            Point2D::new(dec!(2), dec!(2)),
        ]);
        let interpolator = create_mock_interpolator(points);

        // Test finding brackets for a point in the middle
        let (i, j) = interpolator.find_bracket_points(dec!(0.5)).unwrap();
        assert_eq!(i, 0);
        assert_eq!(j, 1);

        // Test finding brackets for a point exactly on a known point
        let (i, j) = interpolator.find_bracket_points(dec!(1.0)).unwrap();
        assert_eq!(i, 0);
        assert_eq!(j, 1);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_bracket_points_insufficient_points() {
        let points = BTreeSet::from_iter(vec![Point2D::new(dec!(0), dec!(0))]);
        let interpolator = create_mock_interpolator(points);

        assert!(interpolator.find_bracket_points(dec!(0.5)).is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_bracket_points_out_of_range() {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
        ]);
        let interpolator = create_mock_interpolator(points);

        // Test x below range
        assert!(interpolator.find_bracket_points(dec!(-0.5)).is_err());

        // Test x above range
        assert!(interpolator.find_bracket_points(dec!(1.5)).is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_bracket_points_edge_cases() {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
            Point2D::new(dec!(2), dec!(2)),
        ]);
        let interpolator = create_mock_interpolator(points);

        // Test at lower boundary
        let (i, j) = interpolator.find_bracket_points(dec!(0)).unwrap();
        assert_eq!(i, 0);
        assert_eq!(j, 1);

        // Test at upper boundary
        let (i, j) = interpolator.find_bracket_points(dec!(2)).unwrap();
        assert_eq!(i, 1);
        assert_eq!(j, 2);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_interpolate_with_empty_points() {
        let interpolator = create_mock_interpolator(BTreeSet::from_iter(vec![]));
        let result = interpolator.interpolate(dec!(0.5), InterpolationType::Linear);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_interpolate_routing_minimum_points() {
        // Test each interpolation type with its minimum required points

        // Linear needs 2 points
        let linear_points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
        ]);
        let interpolator = create_mock_interpolator(linear_points);
        let linear = interpolator.interpolate(dec!(0.5), InterpolationType::Linear);
        assert!(linear.is_ok());

        // Spline needs 3 points
        let spline_points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
            Point2D::new(dec!(2), dec!(2)),
        ]);
        let interpolator = create_mock_interpolator(spline_points);
        let spline = interpolator.interpolate(dec!(0.5), InterpolationType::Spline);
        assert!(spline.is_ok());

        // Bilinear and Cubic need 4 points
        let four_points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
            Point2D::new(dec!(2), dec!(2)),
            Point2D::new(dec!(3), dec!(3)),
        ]);
        let interpolator = create_mock_interpolator(four_points);
        let bilinear = interpolator.interpolate(dec!(0.5), InterpolationType::Bilinear);
        let cubic = interpolator.interpolate(dec!(0.5), InterpolationType::Cubic);
        assert!(bilinear.is_ok());
        assert!(cubic.is_ok());
    }
}
