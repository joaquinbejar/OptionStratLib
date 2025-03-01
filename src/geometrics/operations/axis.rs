/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/1/25
******************************************************************************/
use crate::geometrics::InterpolationType;
use itertools::Itertools;
use rust_decimal::Decimal;
use std::collections::HashSet;
use std::hash::Hash;

/// Trait for handling axis-based operations on geometric structures.
///
/// This trait provides methods for efficient lookups and manipulations
/// of points based on their coordinate values. It is designed to work
/// with both 2D curves and 3D surfaces.
///
/// # Type Parameters
/// * `Point` - The complete point type (Point2D for curves, Point3D for surfaces)
/// * `Input` - The input coordinate type (Decimal for curves, Point2D for surfaces)
pub trait AxisOperations<Point, Input>
where
    Input: Hash + Eq + Clone + Ord,
{
    /// The type of error that can occur during point operations
    type Error;

    /// Checks if a coordinate value exists in the structure.
    ///
    /// # Arguments
    /// * `x` - The coordinate value to check (x for curves, xy-point for surfaces)
    ///
    /// # Returns
    /// * `bool` - `true` if the coordinate exists, `false` otherwise
    fn contains_point(&self, x: &Input) -> bool;

    /// Returns a vector of references to all index values in the structure.
    ///
    /// For curves, this returns x-coordinates.
    /// For surfaces, this returns xy-coordinates.
    ///
    /// # Returns
    /// * `Vec<&Input>` - Vector of references to index values
    fn get_index_values(&self) -> Vec<Input>;

    /// Returns a vector of references to dependent values for a given coordinate.
    ///
    /// For curves, returns y-values for a given x-coordinate.
    /// For surfaces, returns z-values for a given xy-coordinate.
    ///
    /// # Arguments
    /// * `x` - The coordinate value to lookup
    ///
    /// # Returns
    /// * `Vec<&Decimal>` - Vector of references to dependent values
    fn get_values(&self, x: Input) -> Vec<&Decimal>;

    /// Finds the closest point to the given coordinate value.
    ///
    /// # Arguments
    /// * `x` - The reference coordinate value
    ///
    /// # Returns
    /// * `Result<&Point, Self::Error>` - The closest point or an error if no points exist
    fn get_closest_point(&self, x: &Input) -> Result<&Point, Self::Error>;

    /// Finds the closest point to the given coordinate value.
    ///
    /// # Arguments
    /// * `x` - The reference coordinate value
    ///
    /// # Returns
    /// * `Result<&Point, Self::Error>` - The closest point or an error if no points exist
    fn get_point(&self, x: &Input) -> Option<&Point>;

    /// Merges the index values from the current structure with an additional set of indices.
    /// This combines self.get_index_values() with the provided axis vector to create
    /// a single vector of unique indices.
    ///
    /// # Arguments
    /// * `axis` - Additional index values to merge with current structure's indices
    ///
    /// # Returns
    /// * `Vec<&Input>` - Vector containing unique combined indices
    fn merge_indexes(&self, axis: Vec<Input>) -> Vec<Input> {
        let self_indexes: Vec<Input> = self.get_index_values();
        let other_indexes: Vec<Input> = axis;

        match (self_indexes.len(), other_indexes.len()) {
            (0, _) => vec![],
            (_, 0) => vec![],
            _ => {
                // Find the overlapping range
                let min_self = self_indexes.first().unwrap();
                let max_self = self_indexes.last().unwrap();
                let min_other = other_indexes.first().unwrap();
                let max_other = other_indexes.last().unwrap();

                // Determine the common range
                let start = std::cmp::max(min_self, min_other);
                let end = std::cmp::min(max_self, max_other);

                // Collect points within the common range from both sets
                self_indexes
                    .iter()
                    .chain(other_indexes.iter())
                    .filter(|&x| x >= start && x <= end)
                    .cloned()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .sorted()
                    .collect()
            }
        }
    }
}

pub trait MergeAxisInterpolate<Point, Input>: AxisOperations<Point, Input>
where
    Point: Clone,
    Input: Hash + Eq + Clone + Ord,
{
    fn merge_axis_index<'a>(&'a self, other: &'a Self) -> Vec<Input> {
        let self_indexes: Vec<Input> = other.get_index_values();
        self.merge_indexes(self_indexes)
    }

    fn merge_axis_interpolate(
        &self,
        other: &Self,
        interpolation: InterpolationType,
    ) -> Result<(Self, Self), Self::Error>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests_merge_indexes {
    use super::*;
    use crate::curves::{Point2D, create_linear_curve};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    // Mock struct for testing
    struct TestCurve {
        points: BTreeSet<Point2D>,
    }

    impl AxisOperations<Point2D, Decimal> for TestCurve {
        type Error = String;

        fn contains_point(&self, x: &Decimal) -> bool {
            self.points.iter().any(|p| p.x == *x)
        }

        fn get_index_values(&self) -> Vec<Decimal> {
            self.points.iter().map(|p| p.x).collect()
        }

        fn get_values(&self, x: Decimal) -> Vec<&Decimal> {
            self.points
                .iter()
                .filter(|p| p.x == x)
                .map(|p| &p.y)
                .collect()
        }

        fn get_closest_point(&self, x: &Decimal) -> Result<&Point2D, Self::Error> {
            self.points
                .iter()
                .min_by(|a, b| {
                    let dist_a = (a.x - x).abs();
                    let dist_b = (b.x - x).abs();
                    dist_a.partial_cmp(&dist_b).unwrap()
                })
                .ok_or_else(|| "No points available".to_string())
        }

        fn get_point(&self, _x: &Decimal) -> Option<&Point2D> {
            unimplemented!("Not implemented for testing")
        }
    }

    fn create_test_curve_1() -> TestCurve {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(1.0), dec!(2.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(4.0), dec!(8.0)),
            Point2D::new(dec!(5.0), dec!(10.0)),
        ]);
        TestCurve { points }
    }

    fn create_test_curve_2() -> TestCurve {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(2.0), dec!(3.0)),
            Point2D::new(dec!(3.0), dec!(6.0)),
            Point2D::new(dec!(5.0), dec!(9.0)),
            Point2D::new(dec!(6.0), dec!(12.0)),
        ]);
        TestCurve { points }
    }

    #[test]
    fn test_merge_indexes_common_points() {
        let curve1 = create_test_curve_1();
        let curve2 = create_test_curve_2();

        let merged_indexes = curve1.merge_indexes(curve2.get_index_values());

        // Should only contain common points
        assert_eq!(merged_indexes.len(), 4);
        let x1 = dec!(2.0);
        let x2 = dec!(5.0);
        assert!(merged_indexes.contains(&x1));
        assert!(merged_indexes.contains(&x2));
    }

    #[test]
    fn test_merge_indexes_empty_curve() {
        let curve1 = create_test_curve_1();
        let empty_curve: TestCurve = TestCurve {
            points: BTreeSet::new(),
        };
        let merged_indexes = curve1.merge_indexes(empty_curve.get_index_values());

        // Should be empty when one curve has no points
        assert!(merged_indexes.is_empty());
    }

    #[test]
    fn test_merge_indexes_no_common_points() {
        let curve1 = TestCurve {
            points: BTreeSet::from_iter(vec![Point2D::new(dec!(1.0), dec!(2.0))]),
        };
        let curve2 = TestCurve {
            points: BTreeSet::from_iter(vec![Point2D::new(dec!(7.0), dec!(3.0))]),
        };

        let merged_indexes = curve1.merge_indexes(curve2.get_index_values());

        // Should be empty when no common points
        assert!(merged_indexes.is_empty());
    }

    #[test]
    fn test_merge_indexes_normal() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(0.5));
        let curve2 = create_linear_curve(dec!(5.0), dec!(15.0), dec!(1.0));
        let merged_indexes = curve1.merge_indexes(curve2.get_index_values());

        assert_eq!(merged_indexes.len(), 6);
        assert_eq!(merged_indexes[0].to_f64().unwrap(), 5.0);
        assert_eq!(merged_indexes[5].to_f64().unwrap(), 10.0);
    }

    #[test]
    fn test_merge_indexes_normal_bis() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(0.5));
        let curve2 = create_linear_curve(dec!(4.0), dec!(20.0), dec!(1.0));
        let merged_indexes = curve1.merge_indexes(curve2.get_index_values());

        assert_eq!(merged_indexes.len(), 10);
        // [4.0, 5.0, 5.6, 6.0, 7.0, 7.2, 8.0, 8.8, 9.0, 10.0]
        assert_eq!(merged_indexes[0].to_f64().unwrap(), 4.0);
        assert_eq!(merged_indexes[5].to_f64().unwrap(), 7.2);
        assert_eq!(merged_indexes[9].to_f64().unwrap(), 10.0);
    }

    #[test]
    fn test_merge_indexes_identical_curves() {
        let curve = create_test_curve_1();
        let merged_indexes = curve.merge_indexes(curve.get_index_values());

        // Should return all points from the curve
        assert_eq!(merged_indexes.len(), 4);
    }
}
