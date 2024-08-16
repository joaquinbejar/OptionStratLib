use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub strike: f64,
    pub maturity: f64,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct Surface {
    pub points: Vec<Point>,
    pub strike_range: (f64, f64),
    pub maturity_range: (f64, f64),
}

#[derive(Debug, Clone, Copy)]
pub enum InterpolationType {
    Linear,
    Bilinear,
    Cubic,
    Spline,
}

#[derive(Debug, Clone, Copy)]
pub enum SurfaceConstructionMethod {
    FromData,
    Parametric,
}

#[derive(Debug, Clone, Copy)]
pub enum SurfaceType {
    Volatility,
    Price,
    Delta,
    Gamma,
    // Other types as needed
}

// Configuration for surface building
pub struct SurfaceConfig {
    pub surface_type: SurfaceType,
    pub interpolation: InterpolationType,
    pub construction_method: SurfaceConstructionMethod,
    pub extra_params: HashMap<String, f64>,
}

impl Surface {
    pub fn new(points: Vec<Point>) -> Self {
        let strike_range = Surface::calculate_range(points.iter().map(|p| p.strike));
        let maturity_range = Surface::calculate_range(points.iter().map(|p| p.maturity));

        Surface {
            points,
            strike_range,
            maturity_range,
        }
    }

    fn calculate_range<I>(iter: I) -> (f64, f64)
    where
        I: Iterator<Item = f64>,
    {
        iter.fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), val| {
            (min.min(val), max.max(val))
        })
    }

    pub fn get_value(
        &self,
        _strike: f64,
        _maturity: f64,
        _interpolation: InterpolationType,
    ) -> Option<f64> {
        todo!(" Implement interpolation logic here");
        // This would be a placeholder, the actual implementation would go in the interpolation modules
    }
}

// Surface analysis result
pub struct SurfaceAnalysisResult {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub skew: f64,
    pub kurtosis: f64,
}

// Custom error for surface operations
#[derive(Debug)]
pub enum SurfaceError {
    InterpolationError(String),
    ConstructionError(String),
    AnalysisError(String),
    // Other types of errors as needed
}
