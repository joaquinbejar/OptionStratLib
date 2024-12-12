/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/10/24
******************************************************************************/

//! # Chains Module
//!
//! This module provides functionality for working with option chains and their components.
//! It includes tools for building, managing, and manipulating option chains, as well as
//! handling multiple-leg option strategies.
//!
//! ## Core Components
//!
//! * `chain` - Implements core option chain functionality (`OptionChain` and `OptionData` structures)
//! * `legs` - Provides strategy leg combinations through the `StrategyLegs` enum
//! * `utils` - Contains utility functions and parameter structures for chain operations
//!
//! ## Main Features
//!
//! * Option chain construction and management
//! * Support for various option data formats
//! * Import/export capabilities (CSV, JSON)
//! * Multiple-leg strategy support
//! * Price calculation and volatility adjustments
//!
//! ## Example Usage
//!
//! ```rust
//! use optionstratlib::chains::chain::{OptionChain, OptionData};
//! use optionstratlib::chains::utils::OptionChainBuildParams;
//! use optionstratlib::model::types::{ExpirationDate, PositiveF64};
//!
//! // Create a new option chain
//! let chain = OptionChain::new(
//!     "SP500",
//!     PositiveF64::new(100.0).unwrap(),
//!     "2024-12-31".to_string()
//! );
//!
//! // Build chain with specific parameters
//! let params = OptionChainBuildParams::new("SP500".to_string(),
//! None,
//! 10,
//! PositiveF64::new(1.0).unwrap(),
//! 0.0,
//! PositiveF64::new(0.02).unwrap(), 2, Default::default());
//!
//! let built_chain = OptionChain::build_chain(&params);
//! ```
//!
//! ## Strategy Legs Support
//!
//! The module supports various option strategy combinations through the `StrategyLegs` enum:
//!
//! * Two-leg strategies (e.g., spreads)
//! * Four-leg strategies (e.g., iron condors)
//! * Six-leg strategies (e.g., butterfly variations)
//!
//! ## Utility Functions
//!
//! The module provides various utility functions for:
//!
//! * Strike price generation
//! * Volatility adjustment
//! * Price calculations
//! * Data parsing and formatting
//!
//! ## File Handling
//!
//! Supports both CSV and JSON formats for:
//!
//! * Importing option chain data
//! * Exporting option chain data
//! * Maintaining consistent data formats
pub mod chain;
mod legs;
pub mod utils;

mod options;

pub use legs::StrategyLegs;

pub use options::{OptionsInStrike, DeltasInStrike};
