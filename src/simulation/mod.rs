//! # Random Walk Simulation Library
//!
//! This library provides tools for simulating and analyzing random walk processes
//! and other stochastic models. It includes implementations of various random walk algorithms,
//! statistical utilities, and visualization capabilities.
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
//! - Visualization support
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
mod stats;

pub use exit::{check_exit_policy, ExitPolicy};
pub use model::WalkType;
pub use params::WalkParams;
pub use stats::SimulationStats;
pub use traits::WalkTypeAble;
