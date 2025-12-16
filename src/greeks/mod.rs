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
//! use optionstratlib::greeks::{
//!     delta, gamma, rho, theta, vanna, vega, veta, vomma, charm, color
//! };
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
//! These Greeks can be visualized using curves and surfaces:
//!
//! | Metric | Curve (by strike) | Surface (price vs. volatility) | Surface (price vs. time) |
//! |--------|-------------------|-------------------------------|-------------------------|
//! | Vanna  | ✓ `vanna_curve()` | ✓ `vanna_surface()`          | -                       |
//! | Vomma  | -                 | ✓ `vomma_surface()`          | -                       |
//! | Veta   | ✓ `veta_curve()`  | -                             | ✓ `veta_time_surface()` |
//! | Charm  | ✓ `charm_curve()` | -                             | ✓ `charm_time_surface()` |
//! | Color  | ✓ `color_curve()` | -                             | ✓ `color_time_surface()` |
//!
//! ### Example: Generating Volatility Surfaces
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::pos;
//!
//! // Create an option chain with options at various strikes
//! let chain = OptionChain::new("SPY", pos!(450.0), "2024-03-15".to_string(), None, None);
//!
//! // Generate Vanna surface across different volatility levels
//! let volatilities = vec![pos!(0.1), pos!(0.2), pos!(0.3), pos!(0.4), pos!(0.5)];
//! let vanna_surface = chain.vanna_surface(volatilities)?;
//!
//! // Generate Vomma surface
//! let volatilities = vec![pos!(0.15), pos!(0.20), pos!(0.25), pos!(0.30)];
//! let vomma_surface = chain.vomma_surface(volatilities)?;
//!
//! // Generate Veta time surface across different time horizons
//! let days = vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)];
//! let veta_surface = chain.veta_time_surface(days)?;
//!
//! // Generate Charm time surface across different time horizons
//! let days = vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)];
//! let charm_surface = chain.charm_time_surface(days)?;
//!
//! // Generate Color time surface across different time horizons
//! let days = vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)];
//! let color_surface = chain.color_time_surface(days)?;
//! ```

mod equations;
mod utils;

pub use equations::{
    Greek, Greeks, GreeksSnapshot, charm, color, delta, gamma, rho, rho_d, theta, vanna, vega,
    veta, vomma,
};
pub(crate) use utils::calculate_d_values;
pub use utils::calculate_delta_neutral_sizes;
pub use utils::{big_n, d1, d2, n};
