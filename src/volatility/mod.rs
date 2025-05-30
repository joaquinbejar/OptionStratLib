/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 14/8/24
******************************************************************************/

//! # Volatility Module
//!
//! This module provides comprehensive volatility calculation and modeling tools
//! for financial applications, including historical, implied, and stochastic
//! volatility models.
//!
//! ## Core Features
//!
//! ### Volatility Calculation Methods
//!
//! - Constant Volatility
//! - Historical Volatility (Moving Window)
//! - EWMA (Exponentially Weighted Moving Average)
//! - GARCH(1,1)
//! - Heston Stochastic Volatility
//! - Implied Volatility
//! - Uncertain Volatility Bounds
//! - Volatility Surface Interpolation
//!
//! ## Usage Examples
//!
//! ### Basic Volatility Calculations
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use optionstratlib::Positive;
//! use optionstratlib::volatility::constant_volatility;
//!
//! let returns = [dec!(0.02), dec!(0.02), dec!(0.02), dec!(0.02)];
//! let vol = constant_volatility(&returns);
//! ```
//!
//! ### Implied Volatility Calculation
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use optionstratlib::{ExpirationDate, Options};
//! use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
//! use optionstratlib::volatility::implied_volatility;
//! use optionstratlib::Positive;
//! use optionstratlib::pos;
//!
//! let mut option = Options::new(
//!     OptionType::European,
//!     Side::Long,
//!     "STOCK".to_string(),
//!     pos!(100.0),   // Strike price
//!     ExpirationDate::Days(pos!(30.0)),
//!     pos!(0.2),   // Initial volatility guess
//!     Positive::ONE,   // Quantity
//!     pos!(100.0),   // Current price
//!     dec!(0.05),   // Risk-free rate
//!     OptionStyle::Call,
//!     Positive::ZERO,   // Dividend yield
//!     None,   // Exotic parameters
//! );
//!
//! let market_price = pos!(30.0);
//! let iv = implied_volatility(market_price, &mut option, 100);
//! ```
//!
//! ### Historical Volatility with Moving Window
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use optionstratlib::volatility::historical_volatility;
//!
//! let returns = [dec!(0.02), dec!(0.02), dec!(-0.02), dec!(0.02)];
//! let window_size = 3;
//! let hist_vol = historical_volatility(&returns, window_size);
//! ```
//!
//! ### EWMA Volatility
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use optionstratlib::volatility::ewma_volatility;
//!
//! let returns = vec![dec!(0.01), dec!(-0.02), dec!(0.015), dec!(-0.01)];
//! let lambda = dec!(0.94); // Standard decay factor for daily data
//! let ewma_vol = ewma_volatility(&returns, lambda);
//! ```
//!
//!
//! ## Mathematical Models
//!
//! ### GARCH(1,1)
//!
//! The GARCH(1,1) model is implemented as:
//! σ²(t) = ω + α * r²(t-1) + β * σ²(t-1)
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use optionstratlib::volatility::garch_volatility;
//!
//! let returns = vec![dec!(0.01), dec!(-0.02), dec!(0.015)];
//! let omega = dec!(0.1);  // Long-term variance weight
//! let alpha = dec!(0.2);  // Recent shock weight
//! let beta = dec!(0.7);   // Previous variance weight
//! let garch_vol = garch_volatility(&returns, omega, alpha, beta);
//! ```
//!
//! ### Heston Stochastic Volatility
//!
//! Implements the Heston model:
//! dv(t) = κ(θ - v(t))dt + ξ√v(t)dW(t)
//!
//! ```rust
//! use rust_decimal::Decimal;
//! use rust_decimal_macros::dec;
//! use optionstratlib::assert_decimal_eq;
//! use optionstratlib::volatility::simulate_heston_volatility;
//!
//! let kappa = dec!(2.0);      // Mean reversion speed
//! let theta = dec!(0.04);     // Long-term variance
//! let xi = dec!(0.3);         // Volatility of volatility
//! let v0 = dec!(0.04);        // Initial variance
//! let dt = Decimal::ONE / dec!(252.0);   // Daily time step
//! let steps = 252;      // Number of steps
//!
//! let heston_vol = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps);
//! ```
//!
//! ## Time Frame Handling
//!
//! The module includes utilities for converting between different time frames:
//!
//! ```rust
//! use optionstratlib::pos;
//! use optionstratlib::utils::time::TimeFrame;
//! use optionstratlib::volatility::{annualized_volatility, de_annualized_volatility};
//!
//! let daily_vol = pos!(0.01);
//! let annual_vol = annualized_volatility(daily_vol, TimeFrame::Day).unwrap();
//! let daily_vol_again = de_annualized_volatility(annual_vol, TimeFrame::Day);
//! ```
//!
//! ## Performance Considerations
//!
//! - Implied volatility calculation: O(n) where n is max_iterations
//! - Historical volatility: O(n*w) where n is returns length and w is window size
//! - EWMA: O(n) where n is returns length
//! - GARCH: O(n) where n is returns length
//! - Heston simulation: O(n) where n is number of steps
//!
//! ## Implementation Notes
//!
//! - All volatility calculations ensure non-negative results
//! - Implied volatility uses Newton-Raphson method with bounds
//! - Surface interpolation uses bilinear interpolation
//! - Time scaling follows the square root of time rule
//! - Numerical stability is ensured through bounds checking
//!
//! ## References
//!
//! The implementations are based on standard financial mathematics literature:
//! - Black-Scholes-Merton option pricing model
//! - RiskMetrics™ Technical Document for EWMA
//! - Heston (1993) stochastic volatility model
//! - GARCH by Bollerslev (1986)

mod traits;
mod utils;

pub use utils::{
    adjust_volatility, annualized_volatility, calculate_iv, constant_volatility,
    de_annualized_volatility, ewma_volatility, garch_volatility, generate_ou_process,
    historical_volatility, implied_volatility, simulate_heston_volatility,
    uncertain_volatility_bounds,
};

pub use traits::{AtmIvProvider, VolatilitySmile};
