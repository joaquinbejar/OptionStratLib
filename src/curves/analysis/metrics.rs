use rust_decimal::Decimal;
use crate::curves::analysis::CurveAnalysisResult;
use crate::curves::Point2D;
use crate::error::CurvesError;

#[derive(Debug, Clone)]
pub struct CurveMetrics {
    // Basic statistics
    pub basic: BasicMetrics,
    // Shape characteristics 
    pub shape: ShapeMetrics,
    // Range information
    pub range: RangeMetrics,
    // Trend analysis
    pub trend: TrendMetrics,
    // Risk metrics
    pub risk: RiskMetrics,
}

impl CurveMetrics {
    pub fn new(
        basic: BasicMetrics,
        shape: ShapeMetrics,
        range: RangeMetrics,
        trend: TrendMetrics,
        risk: RiskMetrics,
    ) -> Self {
        Self {
            basic,
            shape,
            range,
            trend,
            risk,
        }
    }
    
    pub fn curve_analysis_result(&self) -> Result<CurveAnalysisResult, CurvesError> {
        Ok(CurveAnalysisResult {
            statistics: self.basic,
            shape_metrics: self.shape.clone(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BasicMetrics {
    pub mean: Decimal,
    pub median: Decimal,
    pub mode: Decimal,
    pub std_dev: Decimal,
}

#[derive(Clone, Debug)]
pub struct ShapeMetrics {
    pub skewness: Decimal,
    pub kurtosis: Decimal,
    pub peaks: Vec<Point2D>,
    pub valleys: Vec<Point2D>,
    pub inflection_points: Vec<Point2D>,
}

#[derive(Clone, Copy, Debug)]
pub struct RangeMetrics {
    pub min: Point2D,
    pub max: Point2D,
    pub range: Decimal,
    pub quartiles: (Decimal, Decimal, Decimal),
    pub interquartile_range: Decimal,
}

#[derive(Clone, Debug)]
pub struct TrendMetrics {
    pub slope: Decimal,
    pub intercept: Decimal,
    pub r_squared: Decimal,
    pub moving_average: Vec<Point2D>,
}

#[derive(Clone, Copy, Debug)]
pub struct RiskMetrics {
    pub volatility: Decimal,
    pub value_at_risk: Decimal,
    pub expected_shortfall: Decimal,
    pub beta: Decimal,
    pub sharpe_ratio: Decimal,
}