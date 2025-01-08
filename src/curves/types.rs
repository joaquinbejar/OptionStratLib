/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/

use crate::curves::construction::types::CurveConstructionMethod;
use crate::curves::interpolation::types::InterpolationType;
use crate::error::curves::CurvesError;
use crate::model::positive::is_positive;
use rayon::prelude::IntoParallelIterator;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use rayon::iter::ParallelIterator;

/// A point in 2D space represented by Decimal coordinates
///
/// # Examples
/// ```
/// use rust_decimal_macros::dec;
/// use optionstratlib::curves::Point2D;
/// let point = Point2D::new(dec!(1.0), dec!(2.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: Decimal,
    pub y: Decimal,
}

impl Point2D {
    /// Creates a new Point2D from Decimal coordinates
    pub fn new<T: Into<Decimal>, U: Into<Decimal>>(x: T, y: U) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    pub fn to_tuple<T: From<Decimal> + 'static, U: From<Decimal> + 'static>(
        &self,
    ) -> Result<(T, U), CurvesError> {
        if is_positive::<T>() && self.x <= Decimal::ZERO {
            return Err(CurvesError::Point2DError {
                reason: "x must be positive for type T",
            });
        }

        if is_positive::<U>() && self.y <= Decimal::ZERO {
            return Err(CurvesError::Point2DError {
                reason: "y must be positive for type U",
            });
        }

        Ok((T::from(self.x), U::from(self.y)))
    }

    pub fn from_tuple<T: Into<Decimal>, U: Into<Decimal>>(x: T, y: U) -> Result<Self, CurvesError> {
        Ok(Self::new(x, y))
    }

    pub fn to_f64_tuple(&self) -> Result<(f64, f64), CurvesError> {
        let x = self.x.to_f64();
        let y = self.y.to_f64();

        match (x, y) {
            (Some(x), Some(y)) => Ok((x, y)),
            _ => Err(CurvesError::Point2DError {
                reason: "Error converting Decimal to f64",
            }),
        }
    }

    pub fn from_f64_tuple(x: f64, y: f64) -> Result<Self, CurvesError> {
        let x = Decimal::from_f64(x);
        let y = Decimal::from_f64(y);
        match (x, y) {
            (Some(x), Some(y)) => Ok(Self::new(x, y)),
            _ => Err(CurvesError::Point2DError {
                reason: "Error converting f64 to Decimal",
            }),
        }
    }
}

/// A curve represented by a collection of points
#[derive(Debug, Clone)]
pub struct Curve {
    pub points: Vec<Point2D>,
    pub x_range: (Decimal, Decimal),
}

impl Curve {
    /// Creates a new curve from a vector of points
    pub fn new(points: Vec<Point2D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        Curve { points, x_range }
    }

    /// Calculates the range of x values in the curve
    fn calculate_range<I>(iter: I) -> (Decimal, Decimal)
    where
        I: Iterator<Item = Decimal>,
    {
        iter.fold((Decimal::MAX, Decimal::MIN), |(min, max), val| {
            (min.min(val), max.max(val))
        })
    }

    /// Gets the interpolated value at x using the specified interpolation method
    pub fn get_value(&self, x: Decimal, interpolation: InterpolationType) -> Option<Decimal> {
        match interpolation {
            InterpolationType::Linear => self.linear_interpolation(x),
            InterpolationType::Cubic => todo!("Implement cubic interpolation"),
            InterpolationType::Spline => todo!("Implement spline interpolation"),
            InterpolationType::Bilinear => todo!("Implement bilinear interpolation"),
        }
    }

    /// Performs linear interpolation at x
    fn linear_interpolation(&self, x: Decimal) -> Option<Decimal> {
        if self.points.is_empty() {
            return None;
        }

        if x < self.x_range.0 || x > self.x_range.1 {
            return None;
        }

        let (i, _j) = self
            .points
            .windows(2)
            .enumerate()
            .find(|(_, w)| w[0].x <= x && x <= w[1].x)?;

        let (x1, y1) = (self.points[i].x, self.points[i].y);
        let (x2, y2) = (self.points[i + 1].x, self.points[i + 1].y);

        Some(y1 + (x - x1) * (y2 - y1) / (x2 - x1))
    }

    /// Constructs a curve using the specified construction method and returns the result.
    ///
    /// This function supports two distinct curve construction modes:
    /// 1. **FromData**: Requires an explicit set of 2D points. Each point is provided
    ///    in the form of `Point2D`, representing the curve's data.
    /// 2. **Parametric**: Builds the curve algorithmically based on a parameterized
    ///    function (`f`) over a specified time range (`t_start` to `t_end`) and a defined
    ///    number of steps.
    ///
    /// # Parameters
    ///
    /// - `method` (`CurveConstructionMethod`): Specifies how the curve should be constructed.
    ///   The method determines either direct construction (`FromData`) or parametric
    ///   construction (`Parametric`).
    ///
    /// # Returns
    ///
    /// - `Result<Self, CurvesError>`:
    ///     - `Ok(Self)`: A successfully constructed curve.
    ///     - `Err(CurvesError)`: Indicates an issue during curve construction, such as an invalid
    ///       parameter or a failure to generate valid points.
    ///
    /// # Behavior and Details
    ///
    /// - **FromData**:  
    ///   * Checks if the `points` vector is empty. Returns a `CurvesError::Point2D` error
    ///     with the reason `"Empty points array"` if no points are provided.
    ///   * Constructs a `Curve` instance directly from the provided points.
    ///
    /// - **Parametric**:  
    ///   * Divides the `[t_start, t_end]` range into `steps` intervals to evaluate the
    ///     parameterized function `f`, which generates individual `(x, y)` points.
    ///   * Uses parallel computation (via `rayon`) to efficiently compute the points.
    ///   * Maps each computed `(x, y)` pair into a `Point2D` structure using the `Point2D::new`
    ///     constructor.
    ///   * Returns a `CurvesError` in case the function `f` fails to produce valid points.
    ///
    /// # Errors
    ///
    /// - **Empty Data** (FromData):  
    ///   Returns `CurvesError::Point2D` with the reason `"Empty points array"` if the
    ///   input vector of points is empty in `FromData`.
    ///
    /// - **Parametric Function Error**:  
    ///   Returns `CurvesError` if the parametric function `f` encounters an issue during
    ///   computation (e.g., invalid evaluation or input).
    ///
    /// # Parallelism
    ///
    /// The parametric construction mode leverages parallel iteration (`rayon`) to calculate
    /// points efficiently. This improves performance when dealing with larger ranges or
    /// higher step counts.
    ///
    /// # Examples
    ///
    /// **Note**: Usage examples are intentionally omitted as per user request.
    ///
    /// # See Also
    ///
    /// - [`CurveConstructionMethod`]: Defines supported methods for curve construction.
    /// - [`CurvesError`]: Represents possible errors that may occur during the construction process.
    /// - [`Point2D`]: Represents a 2D point object used to define or evaluate the curve.
    pub fn construct(method: CurveConstructionMethod) -> Result<Self, CurvesError> {
        match method {
            CurveConstructionMethod::FromData { points } => {
                if points.is_empty() {
                    return Err(CurvesError::Point2DError {
                        reason: "Empty points array",
                    });
                }
                Ok(Curve::new(points))
            }

            CurveConstructionMethod::Parametric {
                f,
                t_start,
                t_end,
                steps,
            } => {
                let step_size = (t_end - t_start) / Decimal::from(steps);

                let points: Result<Vec<Point2D>, CurvesError> = (0..=steps)
                    .into_par_iter()
                    .map(|i| {
                        let t = t_start + step_size * Decimal::from(i);
                        f(t).map_err(|e| CurvesError::ConstructionError(e.to_string()))
                    })
                    .collect();

                points.map(|points| Curve::new(points))
            }
        }
    }
}

/// Different types of financial curves
#[derive(Debug, Clone, Copy)]
pub enum CurveType {
    Volatility,
    Delta,
    Gamma,
    Theta,
    Rho,
    RhoD,
    Vega,
    Binomial,
    BlackScholes,
    Telegraph,
    Payoff,
    IntrinsicValue,
    TimeValue,
}

/// Configuration for curve construction and interpolation
pub struct CurveConfig {
    pub curve_type: CurveType,
    pub interpolation: InterpolationType,
    pub construction_method: CurveConstructionMethod,
    pub extra_params: HashMap<String, Decimal>,
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_point2d_creation() {
        let point = Point2D::new(dec!(1.0), dec!(2.0));
        assert_eq!(point.x, dec!(1.0));
        assert_eq!(point.y, dec!(2.0));
    }

    #[test]
    fn test_curve_creation() {
        let points = vec![
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ];
        let curve = Curve::new(points);
        assert_eq!(curve.points.len(), 3);
        assert_eq!(curve.x_range, (dec!(1.0), dec!(3.0)));
    }

    #[test]
    fn test_empty_curve() {
        let curve = Curve::new(vec![]);
        assert_eq!(curve.get_value(dec!(1.0), InterpolationType::Linear), None);
    }

    #[test]
    fn test_linear_interpolation() {
        let points = vec![
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ];
        let curve = Curve::new(points);

        assert_eq!(
            curve.get_value(dec!(1.5), InterpolationType::Linear),
            Some(dec!(2.5))
        );
        assert_eq!(
            curve.get_value(dec!(2.5), InterpolationType::Linear),
            Some(dec!(6.5))
        );
    }

    #[test]
    fn test_out_of_range_interpolation() {
        let points = vec![
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ];
        let curve = Curve::new(points);

        assert_eq!(curve.get_value(dec!(0.5), InterpolationType::Linear), None);
        assert_eq!(curve.get_value(dec!(2.5), InterpolationType::Linear), None);
    }

    #[test]
    fn test_single_point_interpolation() {
        let points = vec![Point2D::new(dec!(1.0), dec!(1.0))];
        let curve = Curve::new(points);
        assert_eq!(curve.get_value(dec!(1.0), InterpolationType::Linear), None);
    }
}

#[cfg(test)]
mod tests_curves {
    use super::*;
    use crate::{pos, Positive};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_with_decimal() {
        let x = dec!(1.5);
        let y = dec!(2.5);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    fn test_new_with_positive() {
        let x = pos!(1.5_f64);
        let y = pos!(2.5_f64);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    fn test_to_tuple_with_decimal() {
        let point = Point2D::new(dec!(1.5), dec!(2.5));
        let tuple: (Decimal, Decimal) = point.to_tuple().unwrap();
        assert_eq!(tuple, (dec!(1.5), dec!(2.5)));
    }

    #[test]
    fn test_to_tuple_with_positive() {
        let point = Point2D::new(dec!(1.5), dec!(2.5));
        let tuple: (Positive, Positive) = point.to_tuple().unwrap();
        assert_eq!(tuple, (pos!(1.5), pos!(2.5)));
    }

    #[test]
    fn test_from_tuple_with_decimal() {
        let x = dec!(1.5);
        let y = dec!(2.5);
        let point = Point2D::from_tuple(x, y).unwrap();
        assert_eq!(point, Point2D::new(dec!(1.5), dec!(2.5)));
    }

    #[test]
    fn test_from_tuple_with_positive() {
        let x = pos!(1.5_f64);
        let y = pos!(2.5_f64);
        let point = Point2D::from_tuple(x, y).unwrap();
        assert_eq!(point, Point2D::new(dec!(1.5), dec!(2.5)));
    }

    #[test]
    fn test_new_with_mixed_types() {
        let x = dec!(1.5);
        let y = pos!(2.5_f64);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }
}
