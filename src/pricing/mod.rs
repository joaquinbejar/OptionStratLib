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
//! use optionstratlib::{ExpirationDate, Options};
//! use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
//! use positive::Positive;
//! use positive::pos_or_panic;
//!
//! // Create a Telegraph Process with transition rates
//! let process = TelegraphProcess::new(dec!(0.5), dec!(0.3));
//!
//! // Price an option using the Telegraph Process
//! let option = Options {
//!             option_type: OptionType::European,
//!             side: Side::Long,
//!             underlying_symbol: "AAPL".to_string(),
//!             strike_price: Positive::HUNDRED,
//!             expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
//!             implied_volatility: pos_or_panic!(0.2),
//!             quantity: Positive::ONE,
//!             underlying_price: pos_or_panic!(105.0),
//!             risk_free_rate: dec!(0.05),
//!             option_style: OptionStyle::Call,
//!             dividend_yield: pos_or_panic!(0.01),
//!             exotic_params: None,
//!         };
//! let price = telegraph(&option, 1000, Some(dec!(0.5)), Some(dec!(0.3)));
//! ```
//!
//! ### Combined Model Analysis
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use optionstratlib::{ExpirationDate, Options};
//! use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
//! use positive::Positive;
//! use positive::pos_or_panic;
//! use optionstratlib::pricing::{
//!     black_scholes_model::black_scholes,
//!     monte_carlo::monte_carlo_option_pricing,
//!     telegraph::telegraph
//! };
//! let option = Options {
//!             option_type: OptionType::European,
//!             side: Side::Long,
//!             underlying_symbol: "AAPL".to_string(),
//!             strike_price: Positive::HUNDRED,
//!             expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
//!             implied_volatility: pos_or_panic!(0.2),
//!             quantity: Positive::ONE,
//!             underlying_price: pos_or_panic!(105.0),
//!             risk_free_rate: dec!(0.05),
//!             option_style: OptionStyle::Call,
//!             dividend_yield: pos_or_panic!(0.01),
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

/// American option pricing using analytical approximations.
///
/// This module provides fast analytical methods for pricing American options,
/// including the Barone-Adesi-Whaley (BAW) approximation which offers O(1)
/// complexity compared to O(n²) for binomial tree methods.
///
/// American options can be exercised at any time before expiration, making them
/// more valuable than European options but also more complex to price.
pub mod american;

/// Binomial Tree model for option pricing.
pub mod binomial_model;

/// Barrier option pricing using analytical extensions.
pub mod barrier;

/// Asian option pricing with geometric and arithmetic averaging.
pub mod asian;

/// Binary option pricing (cash-or-nothing, asset-or-nothing, gap).
pub mod binary;

/// Lookback option pricing (floating and fixed strike).
pub mod lookback;

/// Compound option pricing (options on options).
pub mod compound;

/// Black-Scholes model for option pricing and analysis.
///
/// This module implements the Black-Scholes-Merton model for European option pricing
/// and related calculations such as the Greeks (delta, gamma, theta, vega, rho).
///
/// The Black-Scholes model provides closed-form solutions for European option prices
/// under specific assumptions about market behavior and asset price dynamics.
pub mod black_scholes_model;

/// Constants used throughout the financial models.
///
/// Contains mathematical and financial constants required by various pricing models,
/// such as day count conventions, numerical approximation parameters, and defaults.
pub(crate) mod constants;

/// Monte Carlo simulation methods for financial modeling.
///
/// This module provides tools for pricing options and other derivatives using
/// Monte Carlo simulations, which generate random paths for underlying asset prices
/// to estimate expected payoffs.
///
/// Monte Carlo methods are particularly valuable for complex derivatives where
/// closed-form solutions don't exist.
pub mod monte_carlo;

/// Payoff functions for different option types and derivatives.
///
/// Defines payoff calculations for various financial instruments, including
/// standard calls and puts, as well as more exotic payoff structures.
///
/// These payoff functions are used by the pricing models to determine the value
/// of options at expiration or exercise.
pub(crate) mod payoff;

/// Telegraph process model for asset price movement.
///
/// Implements a telegraph process model which can be used as an alternative to
/// geometric Brownian motion for modeling asset price movements with specific
/// jump characteristics.
///
/// The telegraph model is particularly useful for capturing market regimes with
/// distinct volatility states.
pub mod telegraph;

/// Utility functions for financial calculations.
///
/// Provides helper functions and common utilities used across the library,
/// including numerical methods, date handling, and data transformation tools.
pub(crate) mod utils;

/// Unified pricing system for options.
///
/// This module provides a single, consistent API for pricing options using different models.
/// It includes the `PricingEngine` enum for selecting pricing methods, the `price_option`
/// function as the main entry point, and the `Priceable` trait for trait-based pricing.
///
/// ## Features
/// - Black-Scholes closed-form pricing
/// - Monte Carlo simulation with various stochastic models
/// - Unified error handling via `PricingError`
/// - Extensible design for adding new pricing models
///
/// ## Example
/// ```rust
/// use optionstratlib::pricing::{PricingEngine, Priceable};
/// use optionstratlib::{Options, ExpirationDate};
/// use positive::{Positive, pos_or_panic};
/// use optionstratlib::model::types::{OptionStyle, OptionType, Side};
/// use rust_decimal_macros::dec;
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_symbol: "AAPL".to_string(),
///     strike_price: Positive::HUNDRED,
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     quantity: Positive::ONE,
///     underlying_price: pos_or_panic!(105.0),
///     risk_free_rate: dec!(0.05),
///     option_style: OptionStyle::Call,
///     dividend_yield: pos_or_panic!(0.01),
///     exotic_params: None,
/// };
///
/// let engine = PricingEngine::ClosedFormBS;
/// let price = option.price(&engine)?;
/// # Ok::<(), optionstratlib::error::PricingError>(())
/// ```
pub mod unified;

pub use american::barone_adesi_whaley;
pub use asian::asian_black_scholes;
pub use barrier::barrier_black_scholes;
pub use binary::binary_black_scholes;
pub use binomial_model::{BinomialPricingParams, generate_binomial_tree, price_binomial};
pub use black_scholes_model::{BlackScholes, black_scholes};
pub use compound::compound_black_scholes;
pub use lookback::lookback_black_scholes;
pub use monte_carlo::monte_carlo_option_pricing;
pub use payoff::{Payoff, PayoffInfo, Profit};
pub use telegraph::{TelegraphProcess, telegraph};
pub use unified::{Priceable, PricingEngine, price_option};
pub use utils::{probability_keep_under_strike, simulate_returns};
