/******************************************************************************
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
//! use optionstratlib::pnl::utils::{PnL, PnLCalculator};
//! use chrono::{DateTime, Utc};
//! use optionstratlib::Positive;
//! use optionstratlib::f2p;
//!
//! // Create a new PnL instance
//! let pnl = PnL::new(
//!     Some(100.0),  // Realized PnL
//!     Some(50.0),   // Unrealized PnL
//!     25.0,         // Initial costs
//!     75.0,         // Initial income
//!     Utc::now(),   // Calculation timestamp
//! );
//!
//! // Example implementation of PnLCalculator
//! struct MyOption;
//!
//! impl PnLCalculator for MyOption {
//!     fn calculate_pnl(&self, date_time: DateTime<Utc>, market_price: Positive) -> PnL {
//!         // Implement PnL calculation logic
//!         PnL::new(None, Some(market_price.to_f64()), 10.0, 0.0, date_time)
//!     }
//!
//!     fn calculate_pnl_at_expiration(&self, underlying_price: Option<Positive>) -> PnL {
//!         // Implement expiration PnL logic
//!         PnL::new(Some(100.0), None, 10.0, 0.0, Utc::now())
//!     }
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

pub mod utils;
