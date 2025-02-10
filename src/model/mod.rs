/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 1/8/24
******************************************************************************/

//! # Model Module
//!
//! This module provides core data structures and implementations for financial options modeling.
//! It includes fundamental components for option pricing, position management, and type definitions.
//!
//! ## Core Components
//!
//! * `option` - Implementation of the core Options structure and related functionality
//! * `position` - Management of financial positions and their properties
//! * `types` - Essential type definitions and enums
//! * `utils` - Utility functions for model operations and calculations
//! * `format` - Display and Debug implementations for model types
//! * `profit_range` - Calculations for profit/loss ranges
//!
//! ## Key Features
//!
//! ### Options
//!
//! Comprehensive implementation of financial options including:
//!
//! * Multiple option types (European, American, Asian, etc.)
//! * Greeks calculation (Delta, Gamma, Theta, etc.)
//! * Option pricing using various models
//! * Position management and profit/loss calculations
//!
//! ### Position Management
//!
//! Tools for managing financial positions:
//!
//! * Position tracking
//! * Cost basis calculations
//! * Profit/Loss analysis
//! * Break-even calculations
//! * Fee management
//!
//! ### Type System
//!
//! Robust type definitions ensuring type safety:
//!
//! * `Positive` for non-negative numbers
//! * `ExpirationDate` handling
//! * Option styles and types
//! * Side (Long/Short) definitions
//!
//! ### Formatting
//!
//! Comprehensive formatting support:
//!
//! * Display trait implementations for readable output
//! * Debug trait implementations for detailed inspection
//! * Consistent formatting across all types
//! * Custom format implementations for complex types
//!
//! ### Profit/Loss Analysis
//!
//! Tools for analyzing potential outcomes:
//!
//! * Profit range calculations
//! * Break-even point determination
//! * Probability calculations for price ranges
//! * Risk/reward analysis
//!
//! ## Example Usage
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use tracing::info;
//! use optionstratlib::Options;
//! use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
//! use optionstratlib::pos;
//! use optionstratlib::Positive;
//!
//! let option = Options::new(
//!     OptionType::European,
//!     Side::Long,
//!     "AAPL".to_string(),
//!     pos!(100.0),
//!     ExpirationDate::Days(pos!(30.0)),
//!     pos!(0.2),
//!     pos!(1.0),
//!     pos!(105.0),
//!     dec!(0.05),
//!     OptionStyle::Call,
//!     pos!(0.01),
//!     None,
//! );
//!
//! info!("Option Details: {}", option);
//! info!("Debug View: {:?}", option);
//! ```

pub mod decimal;
mod format;
pub mod option;
pub mod position;
pub mod positive;
mod profit_range;
pub mod types;
pub mod utils;

mod axis;

pub use axis::BasicAxisTypes;
pub use option::Options;
pub use position::Position;
pub use profit_range::ProfitLossRange;
pub use types::{ExpirationDate, OptionStyle, OptionType, Side};
