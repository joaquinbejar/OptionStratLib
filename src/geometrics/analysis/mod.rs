//! # Curve Analysis Module
//!
//! Provides comprehensive statistical and financial analysis tools for mathematical curves.
//!
//! ## Core Features
//! - Statistical metric calculations
//! - Performance analysis
//! - Risk assessment
//! - Curve characterization
//!
//! ## Metrics Provided
//! - Basic Metrics: Mean, Median, Mode
//! - Shape Metrics: Skewness, Kurtosis
//! - Range Metrics: Minimum, Maximum, Quartiles
//! - Trend Metrics: Slope, Intercept, R-squared
//! - Risk Metrics: Volatility, Value at Risk
//!
//! ## Use Cases
//! - Financial instrument analysis
//! - Performance evaluation
//! - Statistical modeling
//! - Risk management
//!
//! ## Example
//! ```rust
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::{Curve, Point2D};
//! use optionstratlib::geometrics::{GeometricObject, MetricsExtractor};
//!
//! let curve = Curve::from_vector(vec![
//!            Point2D::new(Decimal::ZERO, Decimal::ZERO), // p11
//!            Point2D::new(Decimal::ONE, Decimal::ONE),   // p12
//!            Point2D::new(Decimal::ZERO, Decimal::ONE),  // p21
//!            Point2D::new(Decimal::ONE, Decimal::TWO),   // p22
//!        ]);
//! let analysis_result = curve.compute_basic_metrics();
//! let shape_metrics = curve.compute_shape_metrics();
//! ```

mod metrics;
mod statistics;
mod traits;

pub use metrics::Metrics;
pub use metrics::{BasicMetrics, RangeMetrics, RiskMetrics, ShapeMetrics, TrendMetrics};
pub use statistics::AnalysisResult;
pub use traits::MetricsExtractor;
