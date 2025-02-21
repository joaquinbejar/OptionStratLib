/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/1/25
******************************************************************************/
use crate::geometrics::ConstructionMethod;
use rust_decimal::Decimal;
use std::collections::BTreeSet;

/// Defines a geometric object constructed from a set of points.
/// Provides methods for creating, accessing, and manipulating these objects.
pub trait GeometricObject<Point: Clone, Input> {
    /// Type alias for any errors that might occur during the construction of the geometric object.
    type Error;

    /// Returns a `BTreeSet` containing references to the points that constitute the geometric object.
    /// The `BTreeSet` ensures that the points are ordered and unique.
    fn get_points(&self) -> BTreeSet<&Point>;

    /// Returns a `Vec` containing references to the points that constitute the geometric object.
    /// This method simply converts the `BTreeSet` from `get_points` into a `Vec`.
    fn vector(&self) -> Vec<&Point> {
        self.get_points().into_iter().collect()
    }

    /// Creates a new geometric object from a `Vec` of points.
    ///
    /// The generic type `T` represents the input point type, which can be converted into the `Point`
    /// type associated with the geometric object.
    ///
    fn from_vector<T>(points: Vec<T>) -> Self
    where
        Self: Sized,
        T: Into<Point> + Clone;

    /// Constructs a geometric object using a specific construction method.
    ///
    /// The generic type `T` represents a type that can be converted into a `ConstructionMethod`.
    /// The `ConstructionMethod` enum provides different strategies for building geometric objects,
    /// such as constructing from a set of data points or from a parametric function.
    ///
    /// This method returns a `Result` to handle potential errors during construction.
    fn construct<T>(method: T) -> Result<Self, Self::Error>
    where
        Self: Sized,
        T: Into<ConstructionMethod<Point, Input>>;

    /// Returns the points of the geometric object as a `Vec` of references.
    /// Equivalent to calling the `vector()` method.
    fn to_vector(&self) -> Vec<&Point> {
        self.vector()
    }

    /// Calculates the minimum and maximum decimal values from an iterator of decimals.
    ///
    /// This is a utility function that can be used to find the range of values in a set of points.
    fn calculate_range<I>(iter: I) -> (Decimal, Decimal)
    where
        I: Iterator<Item = Decimal>,
    {
        iter.fold((Decimal::MAX, Decimal::MIN), |(min, max), val| {
            (min.min(val), max.max(val))
        })
    }
}

pub trait Len {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests_geometric_object {
    use super::*;
    use rust_decimal_macros::dec;

    // Mock Point type for testing
    #[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
    struct TestPoint {
        x: Decimal,
        y: Decimal,
    }

    impl From<(Decimal, Decimal)> for TestPoint {
        fn from(tuple: (Decimal, Decimal)) -> Self {
            TestPoint {
                x: tuple.0,
                y: tuple.1,
            }
        }
    }

    // Mock Geometric Object for testing
    #[derive(Debug, Clone)]
    struct TestGeometricObject {
        points: BTreeSet<TestPoint>,
    }

    impl GeometricObject<TestPoint, Decimal> for TestGeometricObject {
        type Error = String;

        fn get_points(&self) -> BTreeSet<&TestPoint> {
            self.points.iter().collect()
        }

        fn from_vector<T>(points: Vec<T>) -> Self
        where
            T: Into<TestPoint> + Clone,
        {
            let points_set: BTreeSet<TestPoint> = points.into_iter().map(|p| p.into()).collect();
            TestGeometricObject { points: points_set }
        }

        fn construct<T>(method: T) -> Result<Self, Self::Error>
        where
            Self: Sized,
            T: Into<ConstructionMethod<TestPoint, Decimal>>,
        {
            let method = method.into();
            match method {
                ConstructionMethod::FromData { points } => {
                    if points.is_empty() {
                        Err("Cannot create object from empty points".to_string())
                    } else {
                        Ok(TestGeometricObject { points })
                    }
                }
                _ => Err("Unsupported construction method".to_string()),
            }
        }
    }

    impl Len for TestGeometricObject {
        fn len(&self) -> usize {
            self.points.len()
        }
    }

    #[test]
    fn test_get_points() {
        let points = BTreeSet::from([
            TestPoint {
                x: dec!(1.0),
                y: dec!(2.0),
            },
            TestPoint {
                x: dec!(3.0),
                y: dec!(4.0),
            },
        ]);

        let obj = TestGeometricObject {
            points: points.clone(),
        };
        let retrieved_points = obj.get_points();

        assert_eq!(retrieved_points.len(), points.len());
        for point in &points {
            assert!(retrieved_points.contains(point));
        }
    }

    #[test]
    fn test_from_vector() {
        let points = vec![(dec!(1.0), dec!(2.0)), (dec!(3.0), dec!(4.0))];
        let obj = TestGeometricObject::from_vector(points);

        assert_eq!(obj.points.len(), 2);
        assert!(obj.points.contains(&TestPoint {
            x: dec!(1.0),
            y: dec!(2.0)
        }));
        assert!(obj.points.contains(&TestPoint {
            x: dec!(3.0),
            y: dec!(4.0)
        }));
    }

    #[test]
    fn test_construct_from_data() {
        let points = BTreeSet::from([
            TestPoint {
                x: dec!(1.0),
                y: dec!(2.0),
            },
            TestPoint {
                x: dec!(3.0),
                y: dec!(4.0),
            },
        ]);

        let result = TestGeometricObject::construct(ConstructionMethod::FromData {
            points: points.clone(),
        });

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.points, points);
    }

    #[test]
    fn test_construct_empty_points_fails() {
        let points: BTreeSet<TestPoint> = BTreeSet::new();
        let result = TestGeometricObject::construct(ConstructionMethod::FromData { points });

        assert!(result.is_err());
    }

    #[test]
    fn test_to_vector() {
        let points = BTreeSet::from([
            TestPoint {
                x: dec!(1.0),
                y: dec!(2.0),
            },
            TestPoint {
                x: dec!(3.0),
                y: dec!(4.0),
            },
        ]);

        let obj = TestGeometricObject {
            points: points.clone(),
        };
        let vector = obj.to_vector();

        assert_eq!(vector.len(), points.len());
        for point in &points {
            assert!(vector.contains(&point));
        }
    }

    #[test]
    fn test_calculate_range() {
        let values = vec![dec!(1.0), dec!(5.0), dec!(3.0), dec!(-2.0), dec!(10.0)];

        let (min, max) = TestGeometricObject::calculate_range(values.into_iter());

        assert_eq!(min, dec!(-2.0));
        assert_eq!(max, dec!(10.0));
    }

    #[test]
    fn test_len_trait() {
        let points = BTreeSet::from([
            TestPoint {
                x: dec!(1.0),
                y: dec!(2.0),
            },
            TestPoint {
                x: dec!(3.0),
                y: dec!(4.0),
            },
        ]);

        let obj = TestGeometricObject { points };
        assert_eq!(obj.len(), 2);
        assert!(!obj.is_empty());
    }

    #[test]
    fn test_len_empty() {
        let points: BTreeSet<TestPoint> = BTreeSet::new();

        let obj = TestGeometricObject { points };
        assert_eq!(obj.len(), 0);
        assert!(obj.is_empty());
    }
}
