/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 21/1/25
 ******************************************************************************/
use std::collections::BTreeSet;
use rust_decimal::Decimal;
use crate::geometrics::ConstructionMethod;

/// Represents a geometric object composed of points
pub trait GeometricObject<Point: Clone, Input> {
    /// Type for any errors that might occur during construction
    type Error;

    /// Returns a reference to the set of points that make up this object
    fn get_points(&self) -> BTreeSet<&Point>;
    
    fn vector(&self) -> Vec<&Point> {
        self.get_points().into_iter().collect()
    }

    /// Creates a new geometric object from a vector of points
    fn from_vector<T>(points: Vec<T>) -> Self
    where
        Self: Sized,
        T: Into<Point> + Clone;

    /// Constructs a geometric object using a specific method
    fn construct<T>(method: T) -> Result<Self, Self::Error>
    where
        Self: Sized,
        T: Into<ConstructionMethod<Point, Input>>;

    /// Returns the points as a vector
    fn to_vector(&self) -> Vec<&Point> {
        self.vector()
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

pub trait Len {
    fn len(&self) -> usize;
}