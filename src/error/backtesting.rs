/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/9/25
******************************************************************************/
//! Target crate (ADR-0001 D2): **backtest**. Owns `BacktestError`.
//!
//! A backtest drives a strategy through a simulation, so it can fail for a
//! strategy reason or a simulation reason. Backtesting is the layer that
//! composes both, so it owns the error and keeps each cause typed instead of
//! flattening it into a message (#511). Neither `StrategyError` nor
//! `SimulationError` may name the other.

use crate::error::{DecimalError, OptionsError, PricingError, SimulationError, StrategyError};
use thiserror::Error;

/// Failure of a backtest run.
#[derive(Debug, Error)]
pub enum BacktestError {
    /// The strategy under test failed, for example while reading its legs or
    /// its fees.
    #[error(transparent)]
    Strategy(Box<StrategyError>),

    /// The simulation driving the backtest failed.
    #[error(transparent)]
    Simulation(Box<SimulationError>),

    /// Pricing the leg at a step failed.
    #[error(transparent)]
    Pricing(Box<PricingError>),

    /// The option being priced is invalid.
    #[error(transparent)]
    Options(Box<OptionsError>),

    /// Decimal arithmetic on a premium or a statistic overflowed.
    #[error(transparent)]
    Decimal(Box<DecimalError>),

    /// A value that must stay strictly positive left its range.
    #[error(transparent)]
    Positive(#[from] positive::PositiveError),
}

impl From<PricingError> for BacktestError {
    #[inline]
    fn from(error: PricingError) -> Self {
        BacktestError::Pricing(Box::new(error))
    }
}

impl From<OptionsError> for BacktestError {
    #[inline]
    fn from(error: OptionsError) -> Self {
        BacktestError::Options(Box::new(error))
    }
}

impl From<DecimalError> for BacktestError {
    #[inline]
    fn from(error: DecimalError) -> Self {
        BacktestError::Decimal(Box::new(error))
    }
}

impl From<StrategyError> for BacktestError {
    #[inline]
    fn from(error: StrategyError) -> Self {
        BacktestError::Strategy(Box::new(error))
    }
}

impl From<SimulationError> for BacktestError {
    #[inline]
    fn from(error: SimulationError) -> Self {
        BacktestError::Simulation(Box::new(error))
    }
}
