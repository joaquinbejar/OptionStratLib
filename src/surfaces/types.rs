use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub strike: f64,
    pub maturity: f64,
    pub value: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Surface {
    pub points: Vec<Point>,
    pub strike_range: (f64, f64),
    pub maturity_range: (f64, f64),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum InterpolationType {
    Linear,
    Bilinear,
    Cubic,
    Spline,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum SurfaceConstructionMethod {
    FromData,
    Parametric,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum SurfaceType {
    Volatility,
    Price,
    Delta,
    Gamma,
    // Other types as needed
}

#[allow(dead_code)]
// Configuration for surface building
pub struct SurfaceConfig {
    pub surface_type: SurfaceType,
    pub interpolation: InterpolationType,
    pub construction_method: SurfaceConstructionMethod,
    pub extra_params: HashMap<String, f64>,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct SurfaceAnalysisResult {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub skew: f64,
    pub kurtosis: f64,
}

#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
pub enum SurfaceError {
    InterpolationError(String),
    ConstructionError(String),
    AnalysisError(String),
    // Other types of errors as needed
}

#[cfg(test)]
mod tests_surfaces {
    use super::*;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_point_creation() {
        let point = Point {
            strike: 100.0,
            maturity: 1.0,
            value: 0.2,
        };
        assert_eq!(point.strike, 100.0);
        assert_eq!(point.maturity, 1.0);
        assert_eq!(point.value, 0.2);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_surface_creation() {
        let points = vec![
            Point {
                strike: 90.0,
                maturity: 0.5,
                value: 0.15,
            },
            Point {
                strike: 100.0,
                maturity: 1.0,
                value: 0.2,
            },
            Point {
                strike: 110.0,
                maturity: 1.5,
                value: 0.25,
            },
        ];
        let surface = Surface::new(points);
        assert_eq!(surface.points.len(), 3);
        assert_eq!(surface.strike_range, (90.0, 110.0));
        assert_eq!(surface.maturity_range, (0.5, 1.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_surface_range_calculation() {
        let range = Surface::calculate_range(vec![1.0, 2.0, 3.0, 4.0, 5.0].into_iter());
        assert_eq!(range, (1.0, 5.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_surface_config() {
        let mut extra_params = HashMap::new();
        extra_params.insert("param1".to_string(), 1.0);
        extra_params.insert("param2".to_string(), 2.0);

        let config = SurfaceConfig {
            surface_type: SurfaceType::Volatility,
            interpolation: InterpolationType::Linear,
            construction_method: SurfaceConstructionMethod::FromData,
            extra_params,
        };

        assert_eq!(config.extra_params.len(), 2);
        assert_eq!(config.extra_params.get("param1"), Some(&1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_surface_analysis_result() {
        let analysis = SurfaceAnalysisResult {
            mean: 0.2,
            median: 0.19,
            std_dev: 0.05,
            skew: 0.1,
            kurtosis: 3.0,
        };
        assert_eq!(analysis.mean, 0.2);
        assert_eq!(analysis.median, 0.19);
        assert_eq!(analysis.std_dev, 0.05);
        assert_eq!(analysis.skew, 0.1);
        assert_eq!(analysis.kurtosis, 3.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_surface_error() {
        let error = SurfaceError::InterpolationError("Test error".to_string());
        match error {
            SurfaceError::InterpolationError(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Unexpected error type"),
        }
    }
}
