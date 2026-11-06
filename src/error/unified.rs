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
//!
//! Target crate (ADR-0001 D6, roadmap M1-14): **facade**. Owns `Error`, the aggregate over every component error.

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

    /// Delta-neutral adjustment errors.
    #[error(transparent)]
    Adjustment(#[from] crate::error::AdjustmentError),

    /// Probability calculation errors.
    #[error(transparent)]
    Probability(#[from] crate::error::ProbabilityError),

    /// Curve-related errors.
    #[error(transparent)]
    Curve(#[from] crate::error::CurveError),

    /// Curve and surface projections built from option data.
    #[error(transparent)]
    Projection(#[from] crate::error::ProjectionError),

    /// Risk-neutral density and volatility-skew analysis.
    #[error(transparent)]
    Rnd(#[from] crate::error::RNDError),

    /// Backtest runs, which compose a strategy and a simulation.
    #[error(transparent)]
    Backtest(#[from] crate::error::BacktestError),

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
    Positive(#[from] optionstratlib_core::model::PositiveError),

    /// Trade errors.
    #[error(transparent)]
    Trade(#[from] crate::error::TradeError),

    /// The current time was asked for and this build cannot provide it.
    #[error(transparent)]
    Clock(#[from] crate::error::ClockError),

    /// Expiry, exercise and settlement obligations.
    #[error(transparent)]
    Settlement(#[from] crate::error::SettlementError),

    /// Raw SVI parameter validation and slice calibration.
    #[error(transparent)]
    Svi(#[from] crate::error::SviError),

    /// A butterfly or calendar check that cannot be evaluated.
    #[error(transparent)]
    StaticArbitrage(#[from] crate::error::StaticArbitrageError),

    /// A calibrated implied volatility surface that cannot be built, read or shocked.
    #[error(transparent)]
    ImpliedVolatilitySurface(#[from] crate::error::ImpliedVolatilitySurfaceError),

    /// A quadratic rough Heston kernel, simulation or price that cannot be produced.
    #[error(transparent)]
    QuadraticRoughHeston(#[from] crate::error::QuadraticRoughHestonError),

    /// Empty input collection supplied to a utility that requires at least one element.
    #[error("empty collection: {context}")]
    EmptyCollection {
        /// Description of the collection that was empty (e.g. `"positions"`).
        context: &'static str,
    },

    /// I/O error surfaced during file or network operations invoked from examples
    /// and documentation snippets (plot saving, chain loading, etc.).
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{
        ClockError, ImpliedVolatilitySurfaceError, QuadraticRoughHestonError, SettlementError,
        StaticArbitrageError, SviError,
    };

    #[test]
    fn test_error_clock_variant_keeps_the_message_of_its_source() {
        let error = Error::from(ClockError::Unavailable);
        assert!(matches!(error, Error::Clock(ClockError::Unavailable)));
        assert_eq!(error.to_string(), ClockError::Unavailable.to_string());
    }

    #[test]
    fn test_error_converts_from_every_error_added_in_0_24() {
        fn converts<E: Into<Error>>() {}
        converts::<SettlementError>();
        converts::<SviError>();
        converts::<StaticArbitrageError>();
        converts::<ImpliedVolatilitySurfaceError>();
        converts::<QuadraticRoughHestonError>();
    }
}
