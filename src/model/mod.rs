/******************************************************************************
use positive::pos_or_panic;
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
//! use optionstratlib::{ExpirationDate, Options};
//! use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
//! use positive::pos_or_panic;
//! use positive::Positive;
//!
//! let option = Options::new(
//!     OptionType::European,
//!     Side::Long,
//!     "AAPL".to_string(),
//!     Positive::HUNDRED,
//!     ExpirationDate::Days(pos_or_panic!(30.0)),
//!     pos_or_panic!(0.2),
//!     Positive::ONE,
//!     pos_or_panic!(105.0),
//!     dec!(0.05),
//!     OptionStyle::Call,
//!     pos_or_panic!(0.01),
//!     None,
//! );
//!
//! info!("Option Details: {}", option);
//! info!("Debug View: {:?}", option);
//! ```

/// Core utilities for handling decimal numbers in financial calculations.
pub mod decimal;

/// Formatting utilities for displaying financial data and calculations.
mod format;

/// Components for options contract modeling and analysis, including Greeks and pricing models.
pub mod option;

/// Definitions and utilities for managing trading positions, including risk metrics and exposure tracking.
pub mod position;

mod positive_ext;

/// Tools for analyzing and visualizing profit ranges across different market scenarios.
mod profit_range;

/// Common type definitions used throughout the options strategy library.
pub mod types;

/// Utility functions supporting various operations across the library.
pub mod utils;

/// Components for defining and working with chart axes in strategy visualizations.
mod axis;

mod balance;
/// Components for defining and working with expiration dates.
mod expiration;
/// Components for different types of trading legs (spot, futures, perpetuals).
pub mod leg;
mod trade;

pub use axis::BasicAxisTypes;
pub use balance::*;
pub use expiration::ExpirationDate;
pub use option::Options;
pub use position::Position;
pub use profit_range::ProfitLossRange;
pub use trade::{Trade, TradeAble, TradeStatus, TradeStatusAble, save_trades};
pub use types::{OptionStyle, OptionType, Side};
