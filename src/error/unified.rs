/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2024
******************************************************************************/

//! Unified error type for the entire library.
//!
//! This module provides a top-level `Error` enum that encapsulates all specific
//! error types from different modules, enabling unified error handling across
//! the library.

use thiserror::Error;

/// Top-level error type that encompasses all errors in the library.
///
/// This enum uses `#[error(transparent)]` to delegate error display to the
/// underlying error types, maintaining their original error messages while
/// providing a unified error type for the entire library.
///
/// # Examples
///
/// ```
/// use optionstratlib::error::{Error, PricingError};
///
/// fn example() -> Result<(), Error> {
///     // Errors are automatically converted using From trait
///     Err(PricingError::method_error("test", "failed").into())
/// }
/// ```
#[derive(Error, Debug)]
pub enum Error {
    /// Options-related errors.
    #[error(transparent)]
    Options(#[from] crate::error::OptionsError),

    /// Greeks calculation errors.
    #[error(transparent)]
    Greeks(#[from] crate::error::GreeksError),

    /// Volatility calculation errors.
    #[error(transparent)]
    Volatility(#[from] crate::error::VolatilityError),

    /// Option chain errors.
    #[error(transparent)]
    Chain(#[from] crate::error::ChainError),

    /// Position management errors.
    #[error(transparent)]
    Position(#[from] crate::error::PositionError),

    /// Strategy errors.
    #[error(transparent)]
    Strategy(#[from] crate::error::StrategyError),

    /// Probability calculation errors.
    #[error(transparent)]
    Probability(#[from] crate::error::ProbabilityError),

    /// Curve-related errors.
    #[error(transparent)]
    Curve(#[from] crate::error::CurveError),

    /// Decimal operation errors.
    #[error(transparent)]
    Decimal(#[from] crate::error::DecimalError),

    /// Interpolation errors.
    #[error(transparent)]
    Interpolation(#[from] crate::error::InterpolationError),

    /// Metrics calculation errors.
    #[error(transparent)]
    Metrics(#[from] crate::error::MetricsError),

    /// Surface construction errors.
    #[error(transparent)]
    Surface(#[from] crate::error::SurfaceError),

    /// Graph rendering errors.
    #[error(transparent)]
    Graph(#[from] crate::error::GraphError),

    /// Transaction errors.
    #[error(transparent)]
    Transaction(#[from] crate::error::TransactionError),

    /// Pricing errors.
    #[error(transparent)]
    Pricing(#[from] crate::error::PricingError),

    /// OHLCV data errors.
    #[error(transparent)]
    Ohlcv(#[from] crate::error::OhlcvError),

    /// Simulation-related errors.
    #[error(transparent)]
    Simulation(#[from] crate::error::SimulationError),

    /// Positive value errors.
    #[error(transparent)]
    Positive(#[from] positive::error::PositiveError),

    /// Trade errors.
    #[error(transparent)]
    Trade(#[from] crate::error::TradeError),

    /// Generic error with a custom message.
    #[error("{0}")]
    Other(String),
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Other(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Other(msg.to_string())
    }
}
