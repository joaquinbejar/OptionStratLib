use crate::curves::construction::types::CurveConstructionMethod;
use crate::curves::interpolation::types::InterpolationType;
use std::collections::HashMap;
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Point1D {
    pub x: f64,
    pub y: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Curve {
    pub points: Vec<Point1D>,
    pub x_range: (f64, f64),
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct CurveConfig {
    pub curve_type: CurveType,
    pub interpolation: InterpolationType,
    pub construction_method: CurveConstructionMethod,
    pub extra_params: HashMap<String, f64>,
}

#[allow(dead_code)]
impl Curve {
    pub fn new(points: Vec<Point1D>) -> Self {
        let x_range = Curve::calculate_range(points.iter().map(|p| p.x));

        Curve { points, x_range }
    }

    fn calculate_range<I>(iter: I) -> (f64, f64)
    where
        I: Iterator<Item = f64>,
    {
        iter.fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), val| {
            (min.min(val), max.max(val))
        })
    }

    pub fn get_value(&self, x: f64, interpolation: InterpolationType) -> Option<f64> {
        match interpolation {
            InterpolationType::Linear => self.linear_interpolation(x),
            InterpolationType::Cubic => todo!("Implement cubic interpolation"),
            InterpolationType::Spline => todo!("Implement spline interpolation"),
            InterpolationType::Bilinear => todo!("Implement bilinear interpolation"),
        }
    }

    fn linear_interpolation(&self, x: f64) -> Option<f64> {
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

        Some(y1 + (y2 - y1) * (x - x1) / (x2 - x1))
    }
}

#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
pub enum CurveError {
    InterpolationError(String),
    ConstructionError(String),
    AnalysisError(String),
    OperationError(String),
}
