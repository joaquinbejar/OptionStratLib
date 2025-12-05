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
//! * `equations` - Implementation of Greek calculations (delta, gamma, theta, vega, rho, vanna,
//! vomma, veta)
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
//! * Vanna     - Measures the rate of change in delta with respect to volatility
//! * Vomma     - Measures the rate of change in vega with respect to volatility
//! * Veta      - Measures the rate of change in vega with respect to time
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
//! use rust_decimal_macros::dec;
//! use optionstratlib::greeks::{delta, gamma, rho, theta, vanna, vega, veta, vomma};
//! use optionstratlib::{ExpirationDate, Options};
//! use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
//! use optionstratlib::pos;
//! use optionstratlib::Positive;
//!
//! // Create a sample option
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
//!
//! // Calculate Greeks
//! let delta_value = delta(&option);
//! let gamma_value = gamma(&option);
//! let theta_value = theta(&option);
//! let vega_value = vega(&option);
//! let rho_value = rho(&option);
//! let vanna_value = vanna(&option);
//! let vomma = vomma(&option);
//! let veta_value = veta(&option);
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
//! * Vanna: Second-order volatility sensitivity
//! * Vomma: Second-order volatility sensitivity
//! * Veta: Second-order time sensitivity
//!
//! ## Additional Features
//!
//! * Support for both European and American options
//! * Handling of zero volatility cases
//! * Adjustments for dividends
//! * Special case handling for extreme values

mod equations;
mod utils;

pub use equations::{
    Greek, Greeks, GreeksSnapshot, delta, gamma, rho, rho_d, theta, vanna, vega, veta, vomma,
};
pub(crate) use utils::calculate_d_values;
pub use utils::calculate_delta_neutral_sizes;
pub use utils::{big_n, d1, d2, n};
