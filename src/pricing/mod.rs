//! # Options Pricing Module
//!
//! This module provides implementations for various financial models and utilities
//! to calculate and simulate option pricing. The module includes support for several
//! well-known mathematical models such as the Binomial Tree Model, Black-Scholes Model,
//! Monte Carlo Simulations, and Telegraph Process.
//!
//! ## Core Models
//!
//! ### Binomial Model (`binomial_model`)
//! Contains the implementation of the Binomial Tree Model for option pricing. This model
//! supports both European and American style options and allows for customization of steps
//! and parameters like volatility, interest rates, and time increments.
//!
//! ### Black-Scholes Model (`black_scholes_model`)
//! Implements the Black-Scholes option pricing model, a widely used formula for pricing
//! European-style options. This module provides tools to calculate option prices and
//! associated Greek values.
//!
//! ### Monte Carlo Simulations (`monte_carlo`)
//! Provides Monte Carlo simulation capabilities for option pricing. This module
//! supports simulation of stock price paths and uses statistical methods to estimate
//! option values under various stochastic processes.
//!
//! ### Telegraph Process (`telegraph`)
//! Implements the Telegraph process, a two-state stochastic process for modeling price movements.
//! Key features include:
//! - State transitions between +1 and -1 based on transition rates
//! - Parameter estimation from historical data
//! - Support for asymmetric transition rates
//! - Applications in regime-switching scenarios
//!
//! The Telegraph Process is particularly useful for:
//! - Modeling regime changes in volatility
//! - Capturing market sentiment switches
//! - Simulating discrete state transitions
//!
//! ## Supporting Modules
//!
//! ### Payoff Calculations (`payoff`)
//! Defines payoff structures and calculations for:
//! - Standard options (calls and puts)
//! - Exotic options
//! - Custom payoff functions
//!
//! ### Utility Functions (`utils`)
//! Provides essential mathematical and financial utilities:
//! - Probability calculations
//! - Discount factor computations
//! - Statistical functions
//! - Parameter estimation tools
//!
//! ### Constants (`constants`)
//! Defines model parameters and limits used across the pricing implementations:
//! - Numerical bounds
//! - Default values
//! - Calculation constraints
//!
//! ## Usage Examples
//!
//! ### Using the Telegraph Process
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use optionstratlib::pricing::telegraph::{TelegraphProcess, telegraph};
//! use optionstratlib::model::option::Options;
//! use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side, Positive::ONE};
//! use optionstratlib::Positive;
//! use optionstratlib::f2p;
//!
//! // Create a Telegraph Process with transition rates
//! let process = TelegraphProcess::new(dec!(0.5), dec!(0.3));
//!
//! // Price an option using the Telegraph Process
//! let option = Options {
//!             option_type: OptionType::European,
//!             side: Side::Long,
//!             underlying_symbol: "AAPL".to_string(),
//!             strike_price: f2p!(100.0),
//!             expiration_date: ExpirationDate::Days(30.0),
//!             implied_volatility: 0.2,
//!             quantity: Positive::ONE,
//!             underlying_price: f2p!(105.0),
//!             risk_free_rate: 0.05,
//!             option_style: OptionStyle::Call,
//!             dividend_yield: 0.01,
//!             exotic_params: None,
//!         };
//! let price = telegraph(&option, 1000, Some(dec!(0.5)), Some(dec!(0.3)));
//! ```
//!
//! ### Combined Model Analysis
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use optionstratlib::model::option::Options;
//! use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side, Positive::ONE};
//! use optionstratlib::Positive;
//! use optionstratlib::f2p;
//! use optionstratlib::pricing::{
//!     black_scholes_model::black_scholes,
//!     monte_carlo::monte_carlo_option_pricing,
//!     telegraph::telegraph
//! };
//! let option = Options {
//!             option_type: OptionType::European,
//!             side: Side::Long,
//!             underlying_symbol: "AAPL".to_string(),
//!             strike_price: f2p!(100.0),
//!             expiration_date: ExpirationDate::Days(30.0),
//!             implied_volatility: 0.2,
//!             quantity: Positive::ONE,
//!             underlying_price: f2p!(105.0),
//!             risk_free_rate: 0.05,
//!             option_style: OptionStyle::Call,
//!             dividend_yield: 0.01,
//!             exotic_params: None,
//!         };
//! // Compare prices across different models
//! let bs_price = black_scholes(&option);
//! let mc_price = monte_carlo_option_pricing(&option, 2, 2);
//! let tp_price = telegraph(&option, 1000, Some(dec!(0.5)), Some(dec!(0.3)));
//! ```
//!
//! ## Implementation Notes
//!
//! - All models support standard market conventions for option pricing
//! - Parameter validation and bounds checking are implemented
//! - Error handling follows Rust's Result pattern
//! - Performance optimizations are included for numerical calculations
//!
//! ## Model Selection Guidelines
//!
//! Choose the appropriate model based on your needs:
//! - Black-Scholes: Quick pricing of European options
//! - Binomial: American options and early exercise
//! - Monte Carlo: Complex path-dependent options
//! - Telegraph: Regime-switching and discrete state transitions
//!
//! ## Performance Considerations
//!
//! - Telegraph Process: O(n) complexity where n is the number of steps
//! - Monte Carlo: O(m*n) where m is the number of simulations
//! - Binomial: O(n²) where n is the number of steps
//! - Black-Scholes: O(1) constant time calculation
//!
//! For high-frequency calculations, consider using the Black-Scholes model
//! when applicable, as it provides the fastest computation times.

pub mod binomial_model;
pub mod black_scholes_model;
pub(crate) mod constants;
pub mod monte_carlo;
pub(crate) mod payoff;
pub mod telegraph;
pub(crate) mod utils;

pub use binomial_model::{generate_binomial_tree, price_binomial, BinomialPricingParams};
pub use black_scholes_model::{black_scholes, BlackScholes};
pub use monte_carlo::monte_carlo_option_pricing;
pub use payoff::{Payoff, PayoffInfo, Profit};
pub use telegraph::{telegraph, TelegraphProcess};
pub use utils::{probability_keep_under_strike, simulate_returns};
