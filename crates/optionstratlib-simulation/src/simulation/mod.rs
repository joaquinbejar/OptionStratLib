//! # Random Walk Simulation Library
//!
//! This library provides tools for simulating and analyzing random walk processes
//! and other stochastic models. It includes implementations of various random walk algorithms
//! and statistical utilities; charts of a walk are drawn by the `optionstratlib` facade.
//!
//! The library is organized into several modules:
//! - `model`: Contains the data structures and types that represent stochastic processes
//! - `simulator`: Provides simulation engines and algorithms for running the models
//! - `utils`: Utility functions and helpers for statistical analysis and data manipulation
//! - `walk`: Public API for creating and running random walk simulations
//!
//! ## Core Components
//!
//! ## Mathematical Background
//!
//! The random walk implementation follows the geometric Brownian motion model with:
//!
//! 1. Price changes: dS = μSdt + σSdW
//!    - S: Asset price
//!    - μ: Drift (mean return)
//!    - σ: Volatility
//!    - dW: Wiener process increment
//!
//! 2. Volatility updates: σ(t) ~ N(σ, σ_change)
//!    - Stochastic volatility component
//!    - Updates based on volatility_window
//!
//! ## Features
//!
//! - Geometric Brownian motion simulation
//! - Stochastic volatility modeling
//! - Real-time volatility estimation
//! - Integration with option pricing parameters
//! - Iterator interface for sequential processing
//!
//! ## Performance Considerations
//!
//! - Time Complexity: O(n) for generation, where n is the number of steps
//! - Space Complexity: O(n) for storing the price path
//! - Volatility calculation: O(w) where w is the volatility window size
//!
//! ## Implementation Notes
//!
//! - All prices are strictly positive (enforced by Positive)
//! - Volatility is estimated using rolling windows
//! - The iterator provides option pricing parameters for each step
//! - Thread-safe random number generation
//! - Supports various time frames (daily, weekly, monthly)

/// Contains data structures and types that represent stochastic processes.
///
/// This module defines the mathematical models and their parameters used in
/// random walk simulations, including different types of distributions and
/// process configurations.
mod model;

/// Provides simulation engines and algorithms for running stochastic models.
///
/// This module contains the core simulation logic that powers the random walk
/// implementations, including time-stepping algorithms and state management.
pub mod simulator;

/// # Random Walk Module
///
/// This module provides implementations of various random walk algorithms and related utilities.
/// Random walks are mathematical objects that describe a path consisting of a succession of random steps
/// in some mathematical space.
///
/// ## Usage
///
/// Typically used for stochastic process simulation and analysis, Monte Carlo methods,
/// and modeling natural phenomena with random components.
///
pub mod randomwalk;

/// Module containing functionality for stepping through data or calculations.
///
/// This module provides components and utilities for managing step-based operations,
/// such as iterative calculations, data processing steps, or any process that requires
/// incremental progression through a series of operations.
///
/// The stepping functionality is particularly useful for scenarios where:
/// - Operations need to be performed in a specific sequence
/// - Progress tracking is required through a multi-stage process
/// - Incremental state changes need to be managed
///
pub mod steps;

/// Module containing trait definitions for the financial modeling library.
///
/// This module defines the core traits that establish behavior contracts for
/// various components of the financial modeling system. These traits provide
/// interfaces for implementing different financial models, pricing methods,
/// and data processing techniques.
///
///
/// # Usage
///
/// Traits defined in this module are typically implemented by concrete types
/// throughout the library to ensure consistent behavior and interoperability
/// between different components of the system.
mod traits;

/// Module containing parameter definitions and structures for financial models.
///
/// This module defines the various parameter types, configurations, and constants
/// used across different financial models in the library. It provides structured
/// representation of inputs required for financial calculations and simulations.
///
/// # Usage
///
/// Parameter structures from this module are used as inputs to the various
/// financial models, pricing functions, and simulation methods throughout the library.
/// They encapsulate all the necessary inputs while ensuring proper validation.
mod params;

/// Module containing exit policy definitions for option trading strategies.
///
/// This module defines various exit conditions and policies that can be used
/// to determine when to close an option position during simulations or live trading.
/// Exit policies include percentage-based targets, fixed prices, time-based exits,
/// and composite conditions using AND/OR logic.
///
/// # Usage
///
/// Exit policies are used in simulations to automatically close positions when
/// specific conditions are met, enabling systematic strategy testing and risk management.
pub mod exit;

/// Generic path evaluation contracts: [`PathEvaluator`], [`PathOutcome`],
/// [`PathStatistics`] and the [`evaluate_paths`] driver. They let a
/// simulation be evaluated and summarised without naming a strategy; the
/// backtesting layer adapts its strategy-bound result types to them.
mod outcome;

/// Generic walk driver shared by every step generator.
///
/// Contains [`walk_steps`], the single implementation of the
/// dispatch/advance/build loop that the chain, series and positive
/// generators share, plus [`generator_positive`] (relocated here from
/// `chains::` — it never depended on option chains).
mod walk_driver;

/// Deterministic walkers shared by the generator test suites.
#[cfg(test)]
pub(crate) mod walk_test_support;

pub use exit::{ExitPolicy, check_exit_policy};
pub use model::{WalkPath, WalkType};
pub use outcome::{PathEvaluator, PathOutcome, PathStatistics, evaluate_paths};
pub use params::WalkParams;
pub use traits::{
    WalkTypeAble, WalkTypeAbleClone, custom_walk, garch_walk, heston_walk, telegraph_walk,
};
pub mod ou;
pub use ou::generate_ou_process;
pub use walk_driver::{expanding_window_vols, generator_positive, walk_steps, walk_steps_par};

use crate::error::SimulationError;

/// An empty buffer able to hold `len + extra` path points without growing.
///
/// A walk's length is caller input. `Vec::with_capacity` panics with
/// `capacity overflow` when the points do not fit in `isize::MAX` bytes, and
/// aborts the process when the allocator refuses the request, so a walk of
/// `usize::MAX` steps used to take its caller down. `try_reserve_exact`
/// reserves the same capacity and reports both cases (#788).
///
/// # Errors
///
/// Returns [`SimulationError::InvalidParameters`] when `len + extra`
/// overflows `usize` or the buffer cannot be allocated.
pub(crate) fn path_buffer<T>(len: usize, extra: usize) -> Result<Vec<T>, SimulationError> {
    let capacity = len.checked_add(extra).ok_or_else(|| {
        SimulationError::invalid_parameters(&format!(
            "walk size {len} leaves no room for {extra} more point(s)"
        ))
    })?;
    let mut buffer = Vec::new();
    buffer.try_reserve_exact(capacity).map_err(|e| {
        SimulationError::invalid_parameters(&format!(
            "cannot allocate a path of {capacity} points: {e}"
        ))
    })?;
    Ok(buffer)
}

#[cfg(test)]
mod tests_path_buffer {
    use super::*;

    #[test]
    fn test_path_buffer_reserves_the_requested_points() -> Result<(), SimulationError> {
        let buffer: Vec<u64> = path_buffer(4, 1)?;
        assert!(buffer.is_empty());
        assert!(buffer.capacity() >= 5);
        Ok(())
    }

    #[test]
    fn test_path_buffer_overflowing_or_unallocatable_size_returns_error() {
        for (len, extra) in [(usize::MAX, 1), (usize::MAX / 2, 0)] {
            let result: Result<Vec<u64>, SimulationError> = path_buffer(len, extra);
            assert!(
                matches!(result, Err(SimulationError::InvalidParameters { .. })),
                "{len} + {extra}"
            );
        }
    }
}
