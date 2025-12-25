/******************************************************************************
use positive::pos_or_panic;
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/

//! # PnL (Profit and Loss) Module
//!
//! This module provides structures and traits for calculating and managing profit and loss (PnL) metrics
//! in financial instruments, particularly options.
//!
//! ## Core Components
//!
//! * `PnL` - Structure representing profit and loss information
//! * `PnLCalculator` - Trait for implementing PnL calculation logic
//!
//! ## Key Features
//!
//! ### PnL Structure
//!
//! The `PnL` structure captures:
//! * Realized profits or losses
//! * Unrealized profits or losses
//! * Initial costs and income
//! * Timestamp of calculation
//!
//! ### PnL Calculator
//!
//! The `PnLCalculator` trait enables:
//! * Real-time PnL calculations based on market prices
//! * PnL calculations at expiration
//! * Custom PnL calculation strategies
//!
//! ## Example Usage
//!
//! ```rust
//! use std::error::Error;
//! use optionstratlib::pnl::utils::{PnL, PnLCalculator};
//! use chrono::{DateTime, Utc};
//! use rust_decimal_macros::dec;
//! use optionstratlib::{ExpirationDate, Positive};
//! use optionstratlib::pos_or_panic;
//! use optionstratlib::prelude::PricingError;
//!
//! // Create a new PnL instance
//! let pnl = PnL::new(
//!     Some(dec!(100.0)),   // Realized PnL
//!     Some(dec!(50.0)),   // Unrealized PnL
//!     pos_or_panic!(25.0),   // Initial costs
//!     pos_or_panic!(75.0),   // Initial income
//!     Utc::now(),   // Calculation timestamp
//! );
//!
//! // Example implementation of PnLCalculator
//! struct MyOption;
//!
//! impl PnLCalculator for MyOption {
//!
//!  fn calculate_pnl(
//!      &self,
//!      market_price: &Positive,
//!      expiration_date: ExpirationDate,
//!      _implied_volatility: &Positive,
//!  ) -> Result<PnL, PricingError> {
//!      Ok(PnL::new(
//!          Some(market_price.into()),
//!          None,
//!          pos_or_panic!(10.0),
//!          pos_or_panic!(20.0),
//!          expiration_date.get_date()?,
//!      ))
//!  }
//!  
//!  fn calculate_pnl_at_expiration(
//!      &self,
//!      underlying_price: &Positive,
//!  ) -> Result<PnL, PricingError> {
//!      let underlying_price = underlying_price.to_dec();
//!      Ok(PnL::new(
//!          Some(underlying_price),
//!          None,
//!          pos_or_panic!(10.0),
//!          pos_or_panic!(20.0),
//!          Utc::now(),
//!      ))
//!  }
//! }
//! ```
//!
//! ## Applications
//!
//! The PnL module is particularly useful for:
//! * Option position tracking
//! * Portfolio management
//! * Risk assessment
//! * Performance monitoring
//!
//! ## Features
//!
//! * Real-time PnL tracking
//! * Expiration value calculations
//! * Cost basis tracking
//! * Income tracking
//! * Timestamp-based calculations

/// * [`model`] - Core data structures for financial analysis and PnL modeling
pub mod model;

mod metrics;
mod traits;
mod transaction;
/// * [`utils`] - Utility functions for data manipulation and calculations
pub mod utils;

pub use metrics::{
    PnLMetrics, PnLMetricsDocument, PnLMetricsStep, create_pnl_metrics_document, load_pnl_metrics,
    save_pnl_metrics, save_pnl_metrics_with_document,
};
pub use traits::{PnLCalculator, TransactionAble};
pub use transaction::Transaction;
pub use utils::PnL;
