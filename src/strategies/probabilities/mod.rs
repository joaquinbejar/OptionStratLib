/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/11/24
******************************************************************************/
//! # Probability Analysis Module
//!
//! This module provides comprehensive probability and risk analysis tools for
//! option strategies, including profit probability calculations, risk metrics,
//! and price movement analysis.
//!
//! ## Core Components
//!
//! ### Strategy Probability Analysis
//!
//! ```rust
//! use optionstratlib::model::types::PositiveF64;
//!
//! pub struct StrategyProbabilityAnalysis {
//!     pub probability_of_profit: PositiveF64,
//!     pub probability_of_max_profit: PositiveF64,
//!     pub probability_of_max_loss: PositiveF64,
//!     pub expected_value: PositiveF64,
//!     pub break_even_points: Vec<PositiveF64>,
//!     pub risk_reward_ratio: PositiveF64,
//! }
//! ```
//!
//! ### Probability Analysis Trait
//!
//! ```rust
//! use optionstratlib::model::types::PositiveF64;
//! use optionstratlib::pricing::Profit;
//! use optionstratlib::strategies::base::Strategies;
//!
//! use optionstratlib::strategies::probabilities::{PriceTrend, StrategyProbabilityAnalysis, VolatilityAdjustment};
//!
//! pub trait ProbabilityAnalysis: Strategies + Profit {
//!     fn analyze_probabilities(
//!         &self,
//!         volatility_adj: Option<VolatilityAdjustment>,
//!         trend: Option<PriceTrend>
//!     ) -> Result<StrategyProbabilityAnalysis, String>;
//!     
//!     fn expected_value(
//!         &self,
//!         volatility_adj: Option<VolatilityAdjustment>,
//!         trend: Option<PriceTrend>
//!     ) -> Result<PositiveF64, String>;
//! }
//! ```
//!
//! ## Usage Examples
//!
//! ### Basic Strategy Analysis
//!
//! ```rust
//! use tracing::info;
//! use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
//! use optionstratlib::strategies::probabilities::{ProbabilityAnalysis, VolatilityAdjustment, PriceTrend, StrategyProbabilityAnalysis};
//! use optionstratlib::model::types::PositiveF64;
//! use optionstratlib::pos;
//! use optionstratlib::strategies::bear_call_spread::BearCallSpread;
//!
//! let strategy = BearCallSpread::new(
//!         "SP500".to_string(),
//!         pos!(5781.88), // underlying_price
//!         pos!(5750.0),     // long_strike_itm
//!         pos!(5820.0),     // short_strike
//!         ExpirationDate::Days(2.0),
//!         0.18,      // implied_volatility
//!         0.05,      // risk_free_rate
//!         0.0,       // dividend_yield
//!         pos!(2.0), // long quantity
//!         85.04,     // premium_long
//!         29.85,     // premium_short
//!         0.78,      // open_fee_long
//!         0.78,      // open_fee_long
//!         0.73,      // close_fee_long
//!         0.73,      // close_fee_short
//!     );
//! let analysis = strategy.analyze_probabilities(None, None);
//!
//! info!("Analysis: {:?}", analysis);
//! ```
//!
//! ### Analysis with Volatility Adjustment
//!
//! ```rust
//! use optionstratlib::model::types::ExpirationDate;
//! use optionstratlib::strategies::probabilities::{ProbabilityAnalysis, VolatilityAdjustment};
//! use optionstratlib::model::types::PositiveF64;
//! use optionstratlib::pos;
//! use optionstratlib::strategies::bear_call_spread::BearCallSpread;
//!
//! let strategy = BearCallSpread::new(
//!         "SP500".to_string(),
//!         pos!(5781.88), // underlying_price
//!         pos!(5750.0),     // long_strike_itm
//!         pos!(5820.0),     // short_strike
//!         ExpirationDate::Days(2.0),
//!         0.18,      // implied_volatility
//!         0.05,      // risk_free_rate
//!         0.0,       // dividend_yield
//!         pos!(2.0), // long quantity
//!         85.04,     // premium_long
//!         29.85,     // premium_short
//!         0.78,      // open_fee_long
//!         0.78,      // open_fee_long
//!         0.73,      // close_fee_long
//!         0.73,      // close_fee_short
//!     );
//!
//! let vol_adj = Some(VolatilityAdjustment {
//!     base_volatility: pos!(0.20),         // 20% base volatility
//!     std_dev_adjustment: pos!(0.10),     // 10% adjustment
//! });
//!
//! let analysis = strategy.analyze_probabilities(vol_adj, None);
//! ```
//!
//! ### Analysis with Price Trend
//!
//! ```rust
//! use optionstratlib::model::types::ExpirationDate;
//! use optionstratlib::model::types::PositiveF64;
//! use optionstratlib::pos;
//! use optionstratlib::strategies::bear_call_spread::BearCallSpread;
//! use optionstratlib::strategies::probabilities::{PriceTrend, ProbabilityAnalysis};
//! let strategy = BearCallSpread::new(
//!         "SP500".to_string(),
//!         pos!(5781.88), // underlying_price
//!         pos!(5750.0),     // long_strike_itm
//!         pos!(5820.0),     // short_strike
//!         ExpirationDate::Days(2.0),
//!         0.18,      // implied_volatility
//!         0.05,      // risk_free_rate
//!         0.0,       // dividend_yield
//!         pos!(2.0), // long quantity
//!         85.04,     // premium_long
//!         29.85,     // premium_short
//!         0.78,      // open_fee_long
//!         0.78,      // open_fee_long
//!         0.73,      // close_fee_long
//!         0.73,      // close_fee_short
//!     );
//! let trend = Some(PriceTrend {
//!     drift_rate: 0.05,    // 5% annual drift
//!     confidence: 0.95,    // 95% confidence level
//! });
//!
//! let analysis = strategy.analyze_probabilities(None, trend).unwrap();
//! ```
//!
//! ### Price Range Probability Analysis
//!
//! ```rust
//! use tracing::info;
//! use optionstratlib::strategies::probabilities::calculate_price_probability;
//! use optionstratlib::model::types::ExpirationDate;
//! use optionstratlib::model::types::PositiveF64;
//! use optionstratlib::pos;
//!
//! let (prob_below, prob_in_range, prob_above) = calculate_price_probability(
//!     pos!(100.0),         // current price
//!     pos!(95.0),          // lower bound
//!     pos!(105.0),         // upper bound
//!     None,                // volatility adjustment
//!     None,                // trend
//!     ExpirationDate::Days(30.0),
//!     None                 // risk-free rate
//! ).unwrap();
//! info!("Probabilities: {}, {}, {}", prob_below, prob_in_range, prob_above);
//! ```
//!
//! ## Mathematical Models
//!
//! ### Expected Value Calculation
//!
//! The expected value is calculated using:
//! ```text
//! E[V] = Σ P(Si) * V(Si)
//! ```
//! where:
//! - Si: Price scenario i
//! - P(Si): Probability of scenario i
//! - V(Si): Value at scenario i
//!
//! ### Price Movement Probability
//!
//! Uses log-normal distribution with drift:
//! ```text
//! ln(ST/S0) ~ N(μT, σ²T)
//! ```
//! where:
//! - ST: Price at time T
//! - S0: Current price
//! - μ: Drift rate
//! - σ: Volatility
//! - T: Time to expiration
//!
//! ## Performance Considerations
//!
//! - Probability calculations: O(n) where n is the number of price points
//! - Expected value calculation: O(n) for n scenarios
//! - Memory usage: O(1) for single point calculations
//! - Cache results when analyzing multiple scenarios
//!
//! ## Implementation Notes
//!
//! - All probabilities are strictly positive (PositiveF64)
//! - Volatility adjustments affect both mean and standard deviation
//! - Price trends are incorporated through drift adjustment
//! - Break-even points are calculated numerically
//! - Risk metrics use absolute values for consistency
//!
//! ## Error Handling
//!
//! The module returns Result types for all main functions, with errors for:
//! - Invalid time parameters (negative or zero time to expiry)
//! - Invalid volatility (zero or negative)
//! - Invalid probability bounds (outside [\0,1\])
//! - Invalid price ranges (upper < lower bound)

mod analysis;
pub(crate) mod core;
pub(crate) mod utils;

pub use analysis::StrategyProbabilityAnalysis;
pub use core::ProbabilityAnalysis;
pub use utils::{
    calculate_price_probability, calculate_single_point_probability, PriceTrend,
    VolatilityAdjustment,
};
