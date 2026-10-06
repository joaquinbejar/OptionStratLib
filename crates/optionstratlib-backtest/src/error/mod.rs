//! Errors owned by the backtest layer.
//!
//! A backtest drives a strategy through a simulation, so `BacktestError`
//! wraps the strategy, simulation, pricing and core errors it can meet,
//! each typed, and those types stay with their own crates.

/// ### Backtest Errors (`BacktestError`)
mod backtesting;

pub use backtesting::BacktestError;
