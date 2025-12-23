/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/12/25
******************************************************************************/

//! # Metrics Module
//!
//! This module provides comprehensive performance metrics tools for financial applications:
//! - Price Metrics (Volatility Skew)
//!
//! ## Core Features
//!
//! ### Price Metrics
//!
//! - Volatility Skew: indicates variations in moneyness across options, revealing
//!   insights into market sentiment and expectations. Swek patterns serve as valuable tools
//!   for developing effective trading strategies. Volatility skew reflects differences in
//!   moneyness among options with the same expiration but different strike prices, highlighting
//!   market sentiment and expectations.
//!
//! ### Usage Example Volatility Skew
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::greeks::CalculationErrorKind::DecimalError;
//! use optionstratlib::metrics::VolatilitySkew;
//!
//! struct MySkew;
//!
//! impl VolatilitySkew for MySkew {
//!     fn volatility_skew(&self) -> Curve {
//!         // Custom logic to build and return a Curve representing the skew
//!         Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) }
//!     }
//! }
//! ```
mod price;

pub use price::volatility_skew::VolatilitySkew;
