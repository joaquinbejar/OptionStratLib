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
//! ```
//!
//! ### Time (`time.rs`)
//!
//! Handles various time frames for financial calculations:
//!
//! ```rust
//! use positive::pos_or_panic;
//! use optionstratlib::utils::time::TimeFrame;
//!
//! let daily = TimeFrame::Day;
//! let trading_days_per_year = daily.periods_per_year(); // Returns 252.0
//!
//! let custom = TimeFrame::Custom(pos_or_panic!(365.0));
//! let periods = custom.periods_per_year(); // Returns 365.0
//! ```
//!
//! ### Testing (`tests.rs`)
//!
//! Provides testing utilities and macros for relative equality assertions:
//!
//! ```rust
//! use positive::{Positive,assert_pos_relative_eq,pos_or_panic};
//!
//! let a = Positive::ONE;
//! let b = pos_or_panic!(1.0001);
//! let epsilon = pos_or_panic!(0.001);
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
//! use positive::pos_or_panic;
//! use optionstratlib::utils::time::TimeFrame;
//!
//! let timeframes = vec![
//!     TimeFrame::Day,
//!     TimeFrame::Week,
//!     TimeFrame::Month,
//!     TimeFrame::Custom(pos_or_panic!(360.0))
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
//!
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
//! use positive::{Positive,pos_or_panic,assert_pos_relative_eq};
//!
//! fn test_values() {
//!     let a = Positive::ONE;
//!     let b = pos_or_panic!(1.0001);
//!     let epsilon = pos_or_panic!(0.001);
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

/// This module contains the logger setup and configuration.  It provides functionality for
/// initializing the logger, setting log levels, and formatting log messages.  It uses the `tracing`
/// crate for structured logging and supports various log levels.
pub mod logger;

/// This module contains other miscellaneous modules and functions.  It acts as a container for
/// functionality that doesn't fit neatly into the main project structure.  More specific
/// documentation can be found within each sub-module.
pub mod others;

/// This module contains the CSV reader and writer for OHLCV data.  It provides functionality for
/// reading and writing OHLCV data in CSV format, as well as handling errors related to CSV
/// parsing.
mod csv;
/// This module contains the file reader and writer for OHLCV data.  It provides functionality for
/// reading and writing OHLCV data in various file formats, including CSV and JSON.
pub mod file;
/// Module for time-related utilities.
pub mod time;

/// This module contains traits and type definitions used throughout the library.  It provides
/// functionality for defining and implementing common traits, as well as type aliases for
/// convenience.
mod traits;

#[cfg(feature = "async")]
pub use csv::read_ohlcv_from_zip_async;
pub use csv::{OhlcvCandle, read_ohlcv_from_zip};
pub use logger::{setup_logger, setup_logger_with_level};
pub use others::{approx_equal, get_random_element, process_n_times_iter, random_decimal};
pub use time::TimeFrame;
pub use traits::Len;
