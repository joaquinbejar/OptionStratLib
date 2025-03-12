use rust_decimal::Decimal;
/// # Geometric Transformations
///
/// A trait that defines common geometric transformations and operations for geometric objects
/// in any number of dimensions. This trait provides methods for manipulating objects in space
/// and analyzing their properties.
///
/// ## Type Parameters
///
/// * `Point` - The point type used to represent positions in the geometric space.
///
pub trait GeometricTransformations<Point> {
    /// The error type that can be returned by geometric operations.
    type Error;

    /// Translates the geometric object by specified amounts along each dimension.
    ///
    /// # Arguments
    ///
    /// * `deltas` - A vector of decimal values representing the translation distance
    ///   along each dimension. The length of this vector should match the dimensionality
    ///   of the geometric object.
    ///
    /// # Returns
    ///
    /// A new instance of the geometric object after translation, or an error if
    /// the transformation could not be applied.
    fn translate(&self, deltas: Vec<&Decimal>) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Scales the geometric object by specified factors along each dimension.
    ///
    /// # Arguments
    ///
    /// * `factors` - A vector of decimal values representing the scaling factor
    ///   for each dimension. The length of this vector should match the dimensionality
    ///   of the geometric object.
    ///
    /// # Returns
    ///
    /// A new instance of the geometric object after scaling, or an error if
    /// the transformation could not be applied.
    fn scale(&self, factors: Vec<&Decimal>) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Finds all intersection points between this geometric object and another.
    ///
    /// # Arguments
    ///
    /// * `other` - The other geometric object to find intersections with.
    ///
    /// # Returns
    ///
    /// A vector of intersection points, or an error if the intersections
    /// could not be determined.
    fn intersect_with(&self, other: &Self) -> Result<Vec<Point>, Self::Error>;

    /// Calculates the derivative at a specific point on the geometric object.
    ///
    /// For curves, this represents the tangent. For surfaces, this can represent
    /// partial derivatives.
    ///
    /// # Arguments
    ///
    /// * `point` - The point at which to calculate the derivative.
    ///
    /// # Returns
    ///
    /// A vector containing the derivative values along each dimension, or an error
    /// if the derivative could not be calculated.
    fn derivative_at(&self, point: &Point) -> Result<Vec<Decimal>, Self::Error>;

    /// Finds the extrema (minimum and maximum points) of the geometric object.
    ///
    /// # Returns
    ///
    /// A tuple containing the minimum and maximum points of the geometric object,
    /// or an error if the extrema could not be determined.
    fn extrema(&self) -> Result<(Point, Point), Self::Error>;

    /// Calculates the area or volume under the geometric object relative to a base value.
    ///
    /// For curves, this calculates the area. For higher-dimensional objects,
    /// this calculates volume.
    ///
    /// # Arguments
    ///
    /// * `base_value` - The reference value to measure from.
    ///
    /// # Returns
    ///
    /// The calculated area or volume, or an error if the calculation failed.
    fn measure_under(&self, base_value: &Decimal) -> Result<Decimal, Self::Error>;
}
