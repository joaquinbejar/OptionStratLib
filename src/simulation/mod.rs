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
mod simulator;

/// Utility functions and helpers for statistical analysis and data manipulation.
///
/// This module includes tools for data processing, statistical calculations,
/// result formatting, and other helper functions used across the library.
mod utils;

/// Module for defining custom types and traits used throughout the application.
///
/// This module contains the fundamental trait definitions and type structures that
/// establish the core interfaces for data traversal, value extraction, and sequential
/// access patterns within the application. The centerpiece of this module is the
/// `Walktypable` trait, which provides a standardized way to interact with various
/// data structures and sequences.
///
/// # Key Components
///
/// * `Walktypable` - A trait for traversing and extracting values from various data structures,
///   providing methods to access sequential or calculated values in a consistent manner.
///
/// # Usage
///
/// The types defined in this module are typically used as the foundation for implementing
/// more complex behaviors in other parts of the application, particularly in simulation
/// contexts, financial calculations, or other scenarios where structured data traversal
/// is required.
///
/// # Related Modules
///
/// * `walk` - Likely contains implementations of walking algorithms using the traits defined here
/// * `model` - Might use these types for data modeling
/// * `simulator` - Probably leverages these types for simulation scenarios
pub mod types;

/// Public API for creating and running random walk simulations.
///
/// This module exposes the main interfaces for setting up and executing
/// various types of random walks, analyzing results, and extracting statistics.
/// It serves as the primary entry point for library users.
pub mod walk;

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
pub mod step;
mod display;
mod traits;

pub use traits::WalkTypeAble;


