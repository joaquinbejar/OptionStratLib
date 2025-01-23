use rust_decimal::Decimal;

pub trait GeometricTransformations<Point> {
    type Error;

    // Translation in any dimension
    fn translate(&self, deltas: Vec<&Decimal>) -> Result<Self, Self::Error>
    where Self: Sized;

    // Scaling in any dimension
    fn scale(&self, factors: Vec<&Decimal>) -> Result<Self, Self::Error>
    where Self: Sized;

    // Find intersections with another geometric object
    fn intersect_with(&self, other: &Self) -> Result<Vec<Point>, Self::Error>;

    // Calculate derivative at a point (can be partial for surfaces)
    fn derivative_at(&self, point: &Point) -> Result<Vec<Decimal>, Self::Error>;

    // Find extrema points
    fn extrema(&self) -> Result<(Point, Point), Self::Error>;

    // Calculate area/volume under the geometric object
    fn measure_under(&self, base_value: &Decimal) -> Result<Decimal, Self::Error>;
}