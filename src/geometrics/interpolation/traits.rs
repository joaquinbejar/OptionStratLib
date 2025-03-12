use crate::error::InterpolationError;
use crate::geometrics::{
    BiLinearInterpolation, CubicInterpolation, GeometricObject, InterpolationType,
    LinearInterpolation, SplineInterpolation,
};
use rust_decimal::Decimal;

/// A trait for performing various types of interpolation on a set of 2D points.
///
/// # Overview
/// The `Interpolate` trait unifies methods for interpolating values in 2D Cartesian space.
/// Implementers of this trait must support linear, bilinear, cubic, and spline interpolation methods.
///
/// This trait is designed for use with numerical and graphical applications requiring
/// high-precision interpolation of data points. It provides functionality
/// to retrieve the points, interpolate a value, and find bracketing points for a given x-coordinate.
///
/// # Type Parameters
/// - `Point`: The point type that contains coordinates, must implement `HasX` and `Clone`
/// - `Input`: The input type for interpolation queries, must implement `HasX`
///
/// # Methods
///
/// ## `interpolate`
/// Interpolates a value for a given x-coordinate using a specified interpolation method.
///
/// ### Parameters
/// - `x`: The x-coordinate at which to interpolate
/// - `interpolation_type`: The type of interpolation algorithm to use
///
/// ### Returns
/// - `Result<Point, InterpolationError>`: The interpolated point or an error
///
/// ## `find_bracket_points`
/// Identifies the pair of points that bracket the target x-coordinate for interpolation.
///
/// ### Parameters
/// - `x`: The x-coordinate to bracket
///
/// ### Returns
/// - `Result<(usize, usize), InterpolationError>`: The indices of the bracketing points or an error
///
/// # Error Handling
/// Methods in this trait return `InterpolationError` to represent various issues during interpolation:
/// - Insufficient points for interpolation (need at least two points)
/// - X-coordinate outside the valid range of data points
/// - Failure to find bracketing points
/// - Specific errors from the various interpolation algorithms
///
/// # Requirements
/// Implementers must also implement the following traits:
/// - `LinearInterpolation`
/// - `BiLinearInterpolation`
/// - `CubicInterpolation`
/// - `SplineInterpolation`
/// - `GeometricObject`
///
pub trait Interpolate<Point, Input>:
LinearInterpolation<Point, Input>
+ BiLinearInterpolation<Point, Input>
+ CubicInterpolation<Point, Input>
+ SplineInterpolation<Point, Input>
+ GeometricObject<Point, Input>
where
    Input: HasX,
    Point: HasX + Clone,
{
    /// Interpolates a value at the given x-coordinate using the specified interpolation method.
    ///
    /// This method acts as a facade over the individual interpolation algorithms, delegating
    /// to the appropriate method based on the requested interpolation type.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate at which to interpolate
    /// - `interpolation_type`: The interpolation algorithm to use
    ///
    /// # Returns
    /// - `Ok(Point)`: The successfully interpolated point
    /// - `Err(InterpolationError)`: If interpolation fails for any reason
    fn interpolate(
        &self,
        x: Input,
        interpolation_type: InterpolationType,
    ) -> Result<Point, InterpolationError> {
        match interpolation_type {
            InterpolationType::Linear => self.linear_interpolate(x),
            InterpolationType::Cubic => self.cubic_interpolate(x),
            InterpolationType::Spline => self.spline_interpolate(x),
            InterpolationType::Bilinear => self.bilinear_interpolate(x),
        }
    }

    /// Finds the indices of points that bracket the given x-coordinate.
    ///
    /// This utility method identifies the pair of consecutive points in the dataset
    /// where the first point's x-coordinate is less than or equal to the target x,
    /// and the second point's x-coordinate is greater than or equal to the target x.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate for which to find bracketing points
    ///
    /// # Returns
    /// - `Ok((usize, usize))`: The indices of the two bracketing points
    /// - `Err(InterpolationError)`: If bracketing points cannot be found
    ///
    /// # Errors
    /// - If there are fewer than 2 points in the dataset
    /// - If the requested x-coordinate is outside the range of available points
    /// - If bracketing points cannot be determined for any other reason
    fn find_bracket_points(&self, x: Input) -> Result<(usize, usize), InterpolationError> {
        let points: Vec<&Point> = self.get_points().into_iter().collect();
        // Edge cases
        if points.len() < 2 {
            return Err(InterpolationError::StdError(
                "Need at least two points for interpolation".to_string(),
            ));
        }
        if x.get_x() < points[0].get_x() || x.get_x() > points[points.len() - 1].get_x() {
            return Err(InterpolationError::StdError(
                "x is outside the range of points".to_string(),
            ));
        }
        // Find points that bracket x
        for i in 0..points.len() - 1 {
            if points[i].get_x() <= x.get_x() && x.get_x() <= points[i + 1].get_x() {
                return Ok((i, i + 1));
            }
        }
        Err(InterpolationError::StdError(
            "Could not find bracketing points".to_string(),
        ))
    }
}

/// A trait for types that provide access to an X-coordinate value.
///
/// This trait defines a standard interface for any type that contains or can compute
/// an X-coordinate represented as a `Decimal` value. Implementing this trait allows
/// objects to be used in contexts where X-coordinate access is required, such as:
///
/// - Geometric calculations
/// - Plotting and visualization
/// - Interpolation algorithms
/// - Data point analysis
///
/// # Required Method
///
/// ## `get_x`
///
/// Returns the X-coordinate value of the implementing type.
///
/// ### Returns
/// - `Decimal`: The X-coordinate value, typically representing a position along the x-axis.
///
/// # Implementation Notes
///
/// When implementing this trait, ensure that:
/// - The returned value accurately represents the X-coordinate in the appropriate scale and units
/// - The implementation handles any necessary internal conversions or calculations
/// - The method is computationally efficient if it will be called frequently
///
/// # Usage Examples
///
/// This trait can be implemented for various types such as:
/// - 2D or 3D points
/// - Data samples with timestamps or sequential positions
/// - Geometric shapes with a defined reference point
///
/// # See Also
/// - `Decimal`: The numeric type used to represent the X-coordinate value
pub trait HasX {
    /// Returns the X-coordinate value of this object.
    ///
    /// # Returns
    /// - `Decimal`: The X-coordinate value as a `Decimal` type.
    fn get_x(&self) -> Decimal;
}

#[cfg(test)]
mod tests_interpolate {
    use super::*;
    use crate::curves::Point2D;
    use crate::error::InterpolationError;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct MockInterpolator {
        points: BTreeSet<Point2D>,
    }

    impl LinearInterpolation<Point2D, Decimal> for MockInterpolator {
        fn linear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
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
        fn bilinear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
            if self.points.len() < 4 {
                return Err(InterpolationError::Bilinear(
                    "Need at least four points for bilinear interpolation".to_string(),
                ));
            }
            Ok(Point2D::new(x, x))
        }
    }

    impl CubicInterpolation<Point2D, Decimal> for MockInterpolator {
        fn cubic_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
            if self.points.len() < 4 {
                return Err(InterpolationError::Cubic(
                    "Need at least four points for cubic interpolation".to_string(),
                ));
            }
            Ok(Point2D::new(x, x))
        }
    }

    impl SplineInterpolation<Point2D, Decimal> for MockInterpolator {
        fn spline_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
            if self.points.len() < 3 {
                return Err(InterpolationError::Spline(
                    "Need at least three points for spline interpolation".to_string(),
                ));
            }
            Ok(Point2D::new(x, x))
        }
    }

    impl GeometricObject<Point2D, Decimal> for MockInterpolator {
        type Error = ();

        fn get_points(&self) -> BTreeSet<&Point2D> {
            self.points.iter().collect()
        }

        fn from_vector<T>(_points: Vec<T>) -> Self
        where
            Self: Sized,
            T: Into<Point2D> + Clone,
        {
            unimplemented!()
        }

        fn construct<T>(_method: T) -> Result<Self, Self::Error>
        where
            Self: Sized,
        {
            unimplemented!()
        }
    }

    impl Interpolate<Point2D, Decimal> for MockInterpolator {}

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

        assert!(
            interpolator
                .interpolate(dec!(0.5), InterpolationType::Bilinear)
                .is_err()
        );
        assert!(
            interpolator
                .interpolate(dec!(0.5), InterpolationType::Cubic)
                .is_err()
        );
        assert!(
            interpolator
                .interpolate(dec!(0.5), InterpolationType::Spline)
                .is_err()
        );
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
