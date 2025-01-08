/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/

use rust_decimal::Decimal;
use std::collections::HashMap;
use crate::curves::construction::types::CurveConstructionMethod;
use crate::curves::interpolation::types::InterpolationType;
use rust_decimal::prelude::*;
use crate::error::curves::CurvesError;
use crate::model::positive::is_positive;

/// A point in 2D space represented by Decimal coordinates
///
/// # Examples
/// ```
/// use rust_decimal_macros::dec;
/// use optionstratlib::curves::Point2D;
/// let point = Point2D::new(dec!(1.0), dec!(2.0));
/// ```
#[derive(Debug, Clone, PartialEq)]
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

    pub fn to_tuple<T: From<Decimal> + 'static, U: From<Decimal> + 'static>(&self) -> Result<(T, U), CurvesError> {
        if is_positive::<T>() && self.x <= Decimal::ZERO {
            return Err(CurvesError::Point2D {
                reason: "x must be positive for type T",
            });
        }

        if is_positive::<U>() && self.y <= Decimal::ZERO {
            return Err(CurvesError::Point2D {
                reason: "y must be positive for type U",
            });
        }

        Ok((T::from(self.x), U::from(self.y)))
    }

    pub fn from_tuple<T: Into<Decimal>, U: Into<Decimal>>(x: T, y: U) -> Result<Self, CurvesError>  {
        Ok(Self::new(x, y))
    }
    
    pub fn to_f64_tuple(&self) -> Result<(f64, f64), CurvesError> {
        let x = self.x.to_f64();
        let y = self.y.to_f64();
        
        match (x,y) {
            (Some(x), Some(y)) => Ok((x, y)),
            _ => Err(CurvesError::Point2D {
                reason: "Error converting Decimal to f64",
            }),
        }
        
    }
    
    pub fn from_f64_tuple(x: f64, y: f64) -> Result<Self, CurvesError> {
        let x = Decimal::from_f64(x);
        let y = Decimal::from_f64(y);
        match (x,y) {
            (Some(x), Some(y)) => Ok(Self::new(x, y)),
            _ => Err(CurvesError::Point2D {
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
        iter.fold(
            (Decimal::MAX, Decimal::MIN),
            |(min, max), val| (min.min(val), max.max(val)),
        )
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

/// Error types for curve operations
#[derive(Debug)]
pub enum CurveError {
    InterpolationError(String),
    ConstructionError(String),
    AnalysisError(String),
    OperationError(String),
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
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use crate::{pos, Positive};

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