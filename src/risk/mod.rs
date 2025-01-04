/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 1/8/24
******************************************************************************/

//! # SPAN (Standard Portfolio Analysis of Risk) Module
//!
//! This module implements the SPAN® (Standard Portfolio Analysis of Risk) methodology,
//! a system developed by the Chicago Mercantile Exchange (CME) for calculating margin
//! requirements for derivatives portfolios.
//!
//! ## Overview
//!
//! SPAN calculates margin requirements by analyzing the potential losses a portfolio might
//! experience under various market scenarios. It considers:
//! - Price changes in the underlying asset
//! - Changes in volatility
//! - Extreme market moves
//! - Time decay effects
//! - Short option exposure
//!
//! ## Core Components
//!
//! ### SPANMargin Structure
//!
//! ```rust
//! pub struct SPANMargin {
//!     scanning_range: f64,   // Overall market move range
//!     short_option_minimum: f64,   // Minimum charge for short options
//!     price_scan_range: f64,   // Range for price scenarios
//!     volatility_scan_range: f64,   // Range for volatility scenarios
//! }
//! ```
//!
//! ### Risk Scenarios
//!
//! The module evaluates positions under multiple scenarios combining:
//! - Price movements (up/down/unchanged)
//! - Volatility changes (up/down/unchanged)
//! - Time decay effects
//!
//! ## Usage Examples
//!
//! ### Basic Margin Calculation
//!
//! ```rust
//! use optionstratlib::Options;
//! use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
//! use optionstratlib::Positive;
//! use optionstratlib::model::position::Position;
//! use optionstratlib::pos;
//! use chrono::Utc;
//! use rust_decimal_macros::dec;
//! use optionstratlib::risk::SPANMargin;
//!
//! // Create an option position
//! let option = Options::new(
//!     OptionType::European,
//!     Side::Short,
//!     "STOCK".to_string(),
//!     pos!(150.0),   // Strike price
//!     ExpirationDate::Days(pos!(30.0)),
//!     pos!(0.2),   // Volatility
//!     Positive::ONE,   // Quantity
//!     pos!(155.0),   // Current price
//!     dec!(0.05),   // Risk-free rate
//!     OptionStyle::Call,
//!     Positive::ZERO,   // Dividend yield
//!     None,   // Exotic parameters
//! );
//!
//! let position = Position {
//!     option,
//!     premium: pos!(5.0),
//!     date: Utc::now(),
//!     open_fee: pos!(0.5),
//!     close_fee: pos!(0.5),
//! };
//!
//! // Create SPAN calculator
//! let span = SPANMargin::new(
//!     0.15,   // 15% scanning range
//!     0.10,   // 10% short option minimum
//!     0.05,   // 5% price scan range
//!     0.10   // 10% volatility scan range
//! );
//!
//! // Calculate margin requirement
//! let margin = span.calculate_margin(&position);
//! ```
//!
//! ### Portfolio Analysis
//!
//! ```rust
//! use chrono::Utc;
//! use rust_decimal_macros::dec;
//! use optionstratlib::Options;
//! use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
//! use optionstratlib::model::position::Position;
//! use optionstratlib::Positive;
//! use optionstratlib::pos;
//! use optionstratlib::risk::SPANMargin;
//!
//! let option = Options {
//!             option_type: OptionType::European,
//!             side: Side::Long,
//!             underlying_symbol: "AAPL".to_string(),
//!             strike_price: pos!(100.0),
//!             expiration_date: ExpirationDate::Days(pos!(30.0)),
//!             implied_volatility: pos!(0.2),
//!             quantity: Positive::ONE,
//!             underlying_price: pos!(105.0),
//!             risk_free_rate: dec!(0.05),
//!             option_style: OptionStyle::Call,
//!             dividend_yield: pos!(0.01),
//!             exotic_params: None,
//!         };
//! // Create multiple positions
//! let positions = vec![
//!     Position {
//!         option: option.clone(),
//!         premium: pos!(5.0),
//!         date: Utc::now(),
//!         open_fee: pos!(0.5),
//!         close_fee: pos!(0.5),
//!     },
//!     Position {
//!         option,
//!         premium: pos!(3.0),
//!         date: Utc::now(),
//!         open_fee: pos!(0.5),
//!         close_fee: pos!(0.5),
//!     },
//! ];
//!
//! let span = SPANMargin::new(0.15, 0.10, 0.05, 0.10);
//!
//! // Calculate margin for each position
//! let margins: Vec<f64> = positions.iter()
//!     .map(|pos| span.calculate_margin(pos))
//!     .collect();
//! ```
//!
//! ## Implementation Details
//!
//! ### Risk Array Calculation
//!
//! The risk array is calculated by:
//! 1. Generating price scenarios
//! 2. Generating volatility scenarios
//! 3. Calculating potential loss in each scenario
//! 4. Taking the maximum loss as the base margin requirement
//!
//! ### Short Option Minimum
//!
//! Additional protection against short option positions:
//! - Applied when the position is short
//! - Based on the underlying price and quantity
//! - Acts as a floor for the margin requirement
//!
//!
//! ## Performance Considerations
//!
//! - Time complexity: O(n * m) where n is the number of price scenarios and m is the number of volatility scenarios
//! - Memory complexity: O(n * m) for storing the risk array
//! - Calculation intensive due to multiple option pricing calculations per position
//!
//! ## Notes
//!
//! - All parameters should be provided as decimals (e.g., 0.15 for 15%)
//! - The module uses Black-Scholes pricing for scenario calculations
//! - Short option minimum is always enforced for short positions
//! - Results are conservative estimates of potential losses

mod span;

pub use span::SPANMargin;
