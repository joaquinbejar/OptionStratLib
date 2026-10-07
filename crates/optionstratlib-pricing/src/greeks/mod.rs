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
//! * Charm     - Measures the rate of change in delta with respect to time
//! * Color     - Measures the rate of change in gamma with respect to time
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
//! use optionstratlib_pricing::greeks::{
//!     delta, gamma, rho, theta, vanna, vega, veta, vomma, charm, color
//! };
//! use optionstratlib_core::model::{ExpirationDate, Options};
//! use optionstratlib_core::model::types::{ OptionStyle, OptionType, Side};
//! use optionstratlib_core::pos_or_panic;
//! use optionstratlib_core::model::Positive;
//!
//! // Create a sample option
//! let option = Options {
//!             option_type: OptionType::European,
//!             side: Side::Long,
//!             underlying_symbol: "AAPL".to_string(),
//!             strike_price: Positive::HUNDRED,
//!             expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
//!             implied_volatility: pos_or_panic!(0.2),
//!             quantity: Positive::ONE,
//!             contract_size: Positive::ONE,
//!             underlying_price: pos_or_panic!(105.0),
//!             risk_free_rate: dec!(0.05),
//!             option_style: OptionStyle::Call,
//!             dividend_yield: pos_or_panic!(0.01),
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
//! let charm_value = charm(&option);
//! let color_value = color(&option);
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
//! * Charm: Second-order time sensitivity
//! * Color: Third-order time sensitivity
//!
//! ## Additional Features
//!
//! * Support for both European and American options
//! * Handling of zero volatility cases
//! * Adjustments for dividends
//! * Special case handling for extreme values
//!
//! ## Second-Order Volatility Greeks
//!
//! The module provides specialized second-order Greeks for volatility analysis:
//!
//! ### Vanna (∂Δ/∂σ)
//! Measures the sensitivity of delta to changes in implied volatility. Useful for:
//! - Anticipating changes to delta-hedge effectiveness as volatility changes
//! - Understanding how delta exposure shifts in volatile markets
//!
//! ### Vomma (Volga) (∂²V/∂σ²)
//! Measures the second-order sensitivity of option price to volatility (rate of change
//! of vega with respect to volatility). Key characteristics:
//! - Options far out-of-the-money have the highest Vomma
//! - Long options benefit from positive Vomma in rising volatility environments
//! - Useful for volatility trading strategies
//!
//! ### Veta (∂Vega/∂t)
//! Measures the rate of change of vega with respect to time. Important for:
//! - Understanding how volatility sensitivity decays over time
//! - Managing time-dependent volatility exposure
//!
//! ### Charm (Delta Decay) (∂Δ/∂t)
//! Measures the rate of change of delta with respect to the passage of time. Key aspects:
//! - Predicts how delta exposure evolves daily even if the underlying price remains unchanged
//! - Particularly important for delta-hedged positions over weekends or holidays
//! - Helps anticipate rehedging needs as time decay affects option moneyness
//!
//! ## Third-Order Volatility Greeks
//!
//! The module provides specialized third-order Greeks for volatility analysis:
//!
//! ### Color (Gamma Decay) (∂Γ/∂t)
//! Measures the rate of change of gamma with respect to time (third-order Greek). Useful for:
//! - Understanding how gamma exposure (and thus convexity) decays over time
//! - Managing gamma-hedged portfolios where hedge effectiveness changes with time passage
//! - Anticipating adjustments in dynamic hedging strategies near expiration
//!
//! ## Curve and Surface Representations
//!
//! Projecting these Greeks across strikes, volatilities or horizons
//! (`vanna_curve`, `vomma_surface`, `veta_time_surface`, `charm_curve`,
//! `color_time_surface`, …) needs an option chain, so those projections
//! belong to the market layer: see `OptionChain` in the `optionstratlib`
//! facade's `chains` module.

mod black_76;
mod equations;
mod garman_kohlhagen;
/// Greeks for the leg types, owned by pricing rather than by `model`.
mod legs;
mod model_impls;
pub mod numerical;
mod utils;

pub use black_76::{Black76Greeks, delta_b76, gamma_b76, rho_b76, theta_b76, vega_b76};
pub use equations::{
    Greek, Greeks, GreeksSnapshot, alpha, charm, color, delta, gamma, rho, rho_d, theta, vanna,
    vega, veta, vomma,
};
pub use garman_kohlhagen::{
    GarmanKohlhagenGreeks, delta_gk, gamma_gk, rho_domestic_gk, rho_foreign_gk, theta_gk, vega_gk,
};
pub use legs::LegGreeks;
pub use utils::{DELTA_THRESHOLD, calculate_delta_neutral_sizes, n};
// The kernels shared with `pricing` live in the private `crate::kernels`;
// these are their public paths (#523).
pub use crate::kernels::{big_n, calculate_d_values_black_76, d1, d2};
