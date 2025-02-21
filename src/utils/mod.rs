//! # Utils Module
//!
//! This module provides various utility functions, types, and tools for common tasks
//! across the library, including logging, time handling, testing, and general-purpose
//! utilities.
//!
//! ## Core Components
//!
//! ### Logger (`logger.rs`)
//!
//! Provides logging functionality with configurable log levels:
//!
//! ```rust
//! use optionstratlib::utils::logger::{setup_logger, setup_logger_with_level};
//!
//! // Initialize logger with environment variable
//! setup_logger();
//!
//! // Initialize logger with specific level
//! setup_logger_with_level("DEBUG");
//! ```
//!
//! ### Time (`time.rs`)
//!
//! Handles various time frames for financial calculations:
//!
//! ```rust
//! use optionstratlib::pos;
//! use optionstratlib::utils::time::TimeFrame;
//!
//! let daily = TimeFrame::Day;
//! let trading_days_per_year = daily.periods_per_year(); // Returns 252.0
//!
//! let custom = TimeFrame::Custom(pos!(365.0));
//! let periods = custom.periods_per_year(); // Returns 365.0
//! ```
//!
//! ### Testing (`tests.rs`)
//!
//! Provides testing utilities and macros for relative equality assertions:
//!
//! ```rust
//! use optionstratlib::Positive;
//! use optionstratlib::{assert_pos_relative_eq, pos};
//!
//! let a = pos!(1.0);
//! let b = pos!(1.0001);
//! let epsilon = pos!(0.001);
//! assert_pos_relative_eq!(a, b, epsilon);
//! ```
//!
//! ### Other Utilities (`others.rs`)
//!
//! General-purpose utility functions:
//!
//! ```rust
//! use optionstratlib::utils::others::{approx_equal, get_random_element, process_n_times_iter};
//! use std::collections::BTreeSet;
//!
//! // Approximate equality comparison
//! let equal = approx_equal(1.0, 1.0001);
//!
//! // Get random element from a set
//! let mut set = BTreeSet::new();
//! set.insert(1);
//! set.insert(2);
//! let random = get_random_element(&set);
//!
//! // Process combinations
//! let numbers = vec![1, 2, 3];
//! let result = process_n_times_iter(&numbers, 2, |combination| {
//!     vec![combination[0] + combination[1]]
//! });
//! ```
//!
//! ## Time Frame Support
//!
//! The module supports various time frames for financial calculations:
//!
//! - Microsecond
//! - Millisecond
//! - Second
//! - Minute
//! - Hour
//! - Day
//! - Week
//! - Month
//! - Quarter
//! - Year
//! - Custom periods
//!
//! ### Example: Time Frame Usage
//!
//! ```rust
//! use tracing::info;
//! use optionstratlib::pos;
//! use optionstratlib::utils::time::TimeFrame;
//!
//! let timeframes = vec![
//!     TimeFrame::Day,
//!     TimeFrame::Week,
//!     TimeFrame::Month,
//!     TimeFrame::Custom(pos!(360.0))
//! ];
//!
//! for tf in timeframes {
//!     info!("Periods per year: {}", tf.periods_per_year());
//! }
//! ```
//!
//! ## Logging Configuration
//!
//! Log levels can be configured through:
//! - Environment variable `LOGLEVEL`
//! - Direct specification in code
//!
//! Supported levels:
//! - DEBUG
//! - INFO
//! - WARN
//! - ERROR
//! - TRACE
//!
//! ### Example: Logging Setup
//!
//! ```rust
//! use optionstratlib::utils::logger::setup_logger_with_level;
//! use tracing::{debug, info, warn};
//!
//! // Setup with specific level
//! setup_logger_with_level("DEBUG");
//!
//! // Log messages
//! debug!("Detailed information for debugging");
//! info!("General information about program execution");
//! warn!("Warning messages for potentially harmful situations");
//! ```
//!
//! ## Testing Utilities
//!
//! The module provides testing utilities for:
//! - Relative equality comparisons for Positive
//! - Approximate floating-point comparisons
//! - Random element selection testing
//!
//! ### Example: Testing Positive Values
//!
//! ```rust
//! use optionstratlib::Positive;
//! use optionstratlib::pos;
//! use optionstratlib::assert_pos_relative_eq;
//!
//!
//! fn test_values() {
//!     let a = pos!(1.0);
//!     let b = pos!(1.0001);
//!     let epsilon = pos!(0.001);
//!     assert_pos_relative_eq!(a, b, epsilon);
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - Logger initialization is thread-safe and happens only once
//! - Time frame calculations are constant-time operations
//! - Random element selection is O(n) where n is the set size
//! - Process combinations has complexity based on combination size
//!
//! ## Implementation Notes
//!
//! - Logger uses the `tracing` crate for structured logging
//! - Time frames use predefined constants for standard periods
//! - Testing utilities provide accurate floating-point comparisons
//! - Utility functions handle edge cases and error conditions

pub(crate) mod error;
pub mod logger;
pub mod others;
pub(crate) mod tests;
pub mod time;
mod traits;

pub use logger::{setup_logger, setup_logger_with_level};
pub use others::{approx_equal, get_random_element, process_n_times_iter};
pub use time::TimeFrame;
pub use traits::Len;
