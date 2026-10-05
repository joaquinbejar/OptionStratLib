//! Errors owned by the market layer.
//!
//! Each failure here is raised while building, reading, writing or querying
//! option chains and series. Errors of the layers below (`DecimalError`,
//! `CurveError`, `GreeksError`, …) stay with core, math and pricing, and the
//! variants below that wrap them carry those types unchanged.

/// ### Chain Errors (`ChainError`)
/// Handles:
/// * Option data validation
/// * Chain construction
/// * File operations (CSV/JSON)
/// * Strategy validation
pub mod chains;

/// ### CSV/OHLCV Errors (`OhlcvError`)
/// Handles:
/// * CSV parsing errors
/// * ZIP file handling errors
/// * OHLCV data validation
/// * Date and decimal parsing issues
mod csv;

pub use chains::ChainError;
pub use csv::OhlcvError;
