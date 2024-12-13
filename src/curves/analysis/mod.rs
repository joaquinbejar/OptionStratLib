//! # Analysis Module
//!
//! This module provides functionality for statistical analysis and metric calculations,
//! primarily focused on financial curve analysis and performance metrics.
//!
//! ## Core Components
//!
//! * `metrics` - Handles various performance and financial metrics calculations
//! * `statistics` - Provides statistical analysis tools and curve analysis functionality
//!
//! ## Statistical Analysis
//!
//! The module includes the `CurveAnalysisResult` structure which provides essential
//! statistical measures:
//!
//! * Mean
//! * Median
//! * Standard Deviation
//! * Skewness
//! * Kurtosis
//!
//! ## Applications
//!
//! The analysis module is particularly useful for:
//!
//! * Analyzing return distributions
//! * Calculating risk metrics
//! * Evaluating strategy performance
//! * Performing curve fitting analysis
//! * Statistical hypothesis testing

pub(crate) mod metrics;
pub(crate) mod statistics;
