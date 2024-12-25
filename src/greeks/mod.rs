/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/

//! # Greeks Module
//!
//! This module provides functionality for calculating option Greeks and related metrics
//! used in options trading and risk management.
//!
//! ## Core Components
//!
//! * `equations` - Implementation of Greek calculations (delta, gamma, theta, vega, rho)
//! * `utils` - Utility functions for Greek calculations and related math
//!
//! ## Greeks Provided
//!
//! The module calculates the following Greeks:
//!
//! * Delta (Δ) - Measures the rate of change in option value with respect to the underlying price
//! * Gamma (Γ) - Measures the rate of change in delta with respect to the underlying price
//! * Theta (Θ) - Measures the rate of change in option value with respect to time
//! * Vega  (V) - Measures the rate of change in option value with respect to volatility
//! * Rho   (ρ) - Measures the rate of change in option value with respect to the risk-free rate
//! * Rho_d     - Measures sensitivity to dividend yield changes
//!
//! ## Utilities Included
//!
//! The utilities module provides essential mathematical functions for Greek calculations:
//!
//! * d1/d2 calculations for Black-Scholes model
//! * Normal distribution functions (PDF, CDF)
//! * Mathematical helper functions
//!
//! ## Example Usage
//!
//! ```rust
//! use optionstratlib::greeks::equations::{delta, gamma, rho, theta, vega};
//! use optionstratlib::model::option::Options;
//! use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side, SIZE_ONE};
//! use optionstratlib::pos;
//! use optionstratlib::model::types::PositiveF64;
//!
//! // Create a sample option
//! let option = Options {
//!             option_type: OptionType::European,
//!             side: Side::Long,
//!             underlying_symbol: "AAPL".to_string(),
//!             strike_price: pos!(100.0),
//!             expiration_date: ExpirationDate::Days(30.0),
//!             implied_volatility: 0.2,
//!             quantity: SIZE_ONE,
//!             underlying_price: pos!(105.0),
//!             risk_free_rate: 0.05,
//!             option_style: OptionStyle::Call,
//!             dividend_yield: 0.01,
//!             exotic_params: None,
//!         };
//!
//! // Calculate Greeks
//! let delta_value = delta(&option);
//! let gamma_value = gamma(&option);
//! let theta_value = theta(&option);
//! let vega_value = vega(&option);
//! let rho_value = rho(&option);
//! ```
//!
//! ## Mathematical Background
//!
//! The Greeks are calculated using the Black-Scholes model and its derivatives.
//! Each Greek represents a different dimension of risk:
//!
//! * Delta: First-order price sensitivity
//! * Gamma: Second-order price sensitivity
//! * Theta: Time decay
//! * Vega: Volatility sensitivity
//! * Rho: Interest rate sensitivity
//!
//! ## Additional Features
//!
//! * Support for both European and American options
//! * Handling of zero volatility cases
//! * Adjustments for dividends
//! * Special case handling for extreme values

pub mod equations;
pub mod utils;

pub use utils::{d1, d2};