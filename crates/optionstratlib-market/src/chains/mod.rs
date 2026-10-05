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
//! # fn run() -> Result<(), Box<dyn std::error::Error>> {
//! use rust_decimal::Decimal;
//! use rust_decimal_macros::dec;
//! use optionstratlib_market::chains::OptionChain;
//! use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
//! use optionstratlib_core::{pos_or_panic, spos, model::Positive};
//! use optionstratlib_core::model::ExpirationDate;
//!
//! let option_chain_params = OptionChainBuildParams::new(
//!             "SP500".to_string(),
//!             None,
//!             10,
//!             spos!(1.0),
//!             dec!(-0.2),
//!             Decimal::ZERO,
//!             pos_or_panic!(0.02),
//!             2,
//!             OptionDataPriceParams::new(
//!                 Some(Box::new(Positive::HUNDRED)),
//!                 Some(ExpirationDate::Days(pos_or_panic!(30.0))),
//!                 Some(dec!(0.0)),
//!                 spos!(0.05),
//!                 Some("SP500".to_string()),
//!             ),
//!             pos_or_panic!(0.2),
//!         );
//!
//! let built_chain = OptionChain::build_chain(&option_chain_params)?;
//! assert_eq!(built_chain.symbol, "SP500");
//! assert_eq!(built_chain.underlying_price, Positive::new(100.0)?);
//! # Ok(())
//! # }
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
//!
//! The Risk-Neutral Density analyses over a chain (`RNDAnalysis`,
//! `RNDParameters`, `RNDResult`) are analytics, not market data: see the
//! `optionstratlib` facade's `analytics::rnd`.

/// * `chain` - Public module for handling option chains and related functionalities
pub mod chain;

/// * `legs` - Private module implementing multi-leg option strategies and combinations
mod legs;

/// * `utils` - Public module containing utility functions and helpers for financial calculations
pub mod utils;

/// OHLCV candles read from a zipped CSV; market data I/O (ADR-0001 D2).
#[cfg(feature = "io")]
pub mod csv;

/// * `options` - Private module with core option pricing models and option-specific functionality
mod options;

mod optiondata;

pub(crate) mod model_impls;

pub use chain::OptionChain;
pub use legs::StrategyLegs;
pub use model_impls::UpdateFromOptionData;
pub use optiondata::OptionData;
pub use options::{DeltasInStrike, OptionsInStrike};
pub use utils::{FindOptimalSide, OptionChainBuildParams};
