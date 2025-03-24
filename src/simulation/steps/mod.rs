//! # Financial Time Series Framework
//!
//! This module provides a comprehensive framework for handling time series data
//! in financial applications. It offers a structured approach to manage and process
//! time-based financial information with support for different time units, step sizes,
//! and value representations.
//!
//! The framework is designed to be flexible, type-safe, and easy to use, making it suitable
//! for various financial modeling tasks including option pricing, risk analysis, and time
//! series forecasting.
//!
//! ## Architecture
//!
//! The framework consists of three primary components:
//!
//! * The `Step` structure which combines time and value information
//! * The `Xstep` structure for managing time-related components
//! * The `Ystep` structure for managing value-related components

/// Module containing the `Step` structure which combines time and value components.
///
/// The `Step` structure serves as the main container for time series data points,
/// integrating both temporal information (via `Xstep`) and value information (via `Ystep`).
/// It provides methods for creating sequences of steps and navigating through time series.
mod step;

/// Module containing the `Xstep` structure for handling time components.
///
/// The `Xstep` structure manages temporal aspects of time series data, including:
/// - Time unit representation (days, weeks, months, etc.)
/// - Step size in appropriate time units
/// - Date/time tracking capabilities
/// - Methods for time progression and regression
mod x;

/// Module containing the `Ystep` structure for handling value components.
///
/// The `Ystep` structure manages value-related aspects of time series data, including:
/// - Numeric values of different types
/// - Indexing for sequential tracking
/// - Value manipulation and transformation
/// - Type-safe positive value enforcement
mod y;

pub use step::Step;
pub use x::Xstep;
pub use y::Ystep;
