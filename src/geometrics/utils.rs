/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 21/1/25
 ******************************************************************************/
use std::collections::BTreeSet;
use rust_decimal::Decimal;
use crate::surfaces::Point3D;

/// Represents a geometric object composed of points
pub trait GeometricObject<Point> {
    /// Type for any errors that might occur during construction
    type Error;

    /// Returns a reference to the set of points that make up this object
    fn get_points(&self) -> &BTreeSet<Point>;

    /// Converts the points of a geometric object into a vector of references to `Point3D`.
    ///
    /// # Overview
    /// This method collects all the points of the geometric object represented
    /// by the implementing struct and returns them as a `Vec<&Point3D>`. 
    /// It leverages the `get_points` method from the `GeometricObject` trait,
    /// which provides a reference to a `BTreeSet` of points, ensuring that the points
    /// remain ordered and unique.
    ///
    /// # Returns
    /// - A `Vec<&Point3D>` containing references to the points of the object in the same order
    ///   as maintained by the original `BTreeSet` implementation.
    ///
    /// # Usage
    /// This method is useful when you need to process or iterate over the points of a geometric 
    /// object in a sequential manner and require them in a vector format.
    ///
    /// # Example
    /// ```
    /// use optionstratlib::surfaces::Point3D;
    /// use optionstratlib::geometrics::GeometricObject;
    /// use std::collections::BTreeSet;
    /// use rust_decimal::Decimal;
    ///
    /// struct ExampleObject {
    ///     points: BTreeSet<Point3D>,
    /// }
    ///
    /// impl GeometricObject<Point3D> for ExampleObject {type Error = ();
    ///
    /// fn get_points(&self) -> &BTreeSet<Point3D> {
    ///         &self.points
    ///     }
    ///
    /// fn from_vector(points: Vec<Point3D>) -> Self where Self: Sized {
    ///         unimplemented!("This is just an example implementation")
    ///     }
    ///
    /// fn construct<T>(method: T) -> Result<Self, Self::Error> where Self: Sized {
    ///         unimplemented!("This is just an example implementation")
    ///     }
    /// }
    ///
    /// impl ExampleObject {
    ///     fn vector(&self) -> Vec<&Point3D> {
    ///         self.get_points().iter().collect()
    ///     }
    /// }
    ///
    /// let mut points = BTreeSet::new();
    /// points.insert(Point3D {
    ///     x: Decimal::new(1, 0),
    ///     y: Decimal::new(2, 0),
    ///     z: Decimal::new(3, 0),
    /// });
    ///
    /// let obj = ExampleObject { points };
    /// let point_vector = obj.vector();
    /// assert_eq!(point_vector.len(), 1);
    /// ```
    fn vector(&self) -> Vec<&Point> {
        self.get_points().iter().collect()
    }

    /// Creates a new geometric object from a vector of points
    fn from_vector(points: Vec<Point>) -> Self
    where
        Self: Sized;

    /// Constructs a geometric object using a specific method
    fn construct<T>(method: T) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Returns the points as a vector
    fn to_vector(&self) -> Vec<&Point> {
        self.get_points().iter().collect()
    }

    /// Utility function to calculate the range of decimal values
    fn calculate_range<I>(iter: I) -> (Decimal, Decimal)
    where
        I: Iterator<Item = Decimal>,
    {
        iter.fold((Decimal::MAX, Decimal::MIN), |(min, max), val| {
            (min.min(val), max.max(val))
        })
    }
}