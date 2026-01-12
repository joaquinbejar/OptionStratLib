/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 1/8/24
******************************************************************************/

//! # Options Strategies Module
//!
//! This module contains implementations for various options trading strategies.
//! These strategies combine multiple options contracts to form risk-managed trades
//! suitable for different market conditions. Each strategy provides calculation
//! utilities for profit/loss scenarios and risk assessment.
//!
//! ## Sub-modules
//!
//! - `base`: Provides the base traits and structures for the strategies.
//! - `bear_call_spread`: Implements the Bear Call Spread strategy.
//! - `bear_put_spread`: Implements the Bear Put Spread strategy.
//! - `bull_call_spread`: Implements the Bull Call Spread strategy.
//! - `bull_put_spread`: Implements the Bull Put Spread strategy.
//! - `butterfly_spread`: Implements the Butterfly Spread strategy.
//! - `call_butterfly`: Implements the Call Butterfly strategy.
//! - `collar`: Implements the Collar strategy.
//! - `covered_call`: Implements the Covered Call strategy.
//! - `custom`: Provides utilities for creating custom strategies.
//! - `iron_butterfly`: Implements the Iron Butterfly strategy.
//! - `iron_condor`: Implements the Iron Condor strategy.
//! - `poor_mans_covered_call`: Implements the Poor Man's Covered Call strategy.
//! - `probabilities`: Provides probability calculations for the strategies.
//! - `protective_put`: Implements the Protective Put strategy.
//! - `straddle`: Implements the Straddle strategy.
//! - `strangle`: Implements the Strangle strategy.
//! - `utils`: Provides utility functions for the strategies.
//!
//! ## Usage
//!
//! To use a specific strategy, import the corresponding module and create an instance
//! of the strategy struct. Each strategy provides methods for calculating key metrics
//! such as maximum profit, maximum loss, and breakeven points.
//!
//! Example usage of the Bull Call Spread strategy:
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use tracing::info;
//! use optionstratlib::ExpirationDate;
//! use optionstratlib::strategies::bull_call_spread::BullCallSpread;
//! use positive::Positive;
//! use positive::pos_or_panic;
//! use optionstratlib::strategies::Strategies;
//!
//! let spread = BullCallSpread::new(
//!     "SP500".to_string(),
//!     pos_or_panic!(5780.0),   // underlying_price
//!     pos_or_panic!(5750.0),   // long_strike_itm  
//!     pos_or_panic!(5820.0),   // short_strike
//!     ExpirationDate::Days(Positive::TWO),
//!     pos_or_panic!(0.18),   // implied_volatility
//!     dec!(0.05),   // risk_free_rate
//!     Positive::ZERO,   // dividend_yield
//!     Positive::TWO,   // long quantity
//!     pos_or_panic!(85.04),   // premium_long
//!     pos_or_panic!(29.85),   // premium_short
//!     pos_or_panic!(0.78),   // open_fee_long
//!     pos_or_panic!(0.78),   // open_fee_long
//!     pos_or_panic!(0.73),   // close_fee_long
//!     pos_or_panic!(0.73),   // close_fee_short
//! );
//!
//! let profit = spread.get_max_profit().unwrap_or(Positive::ZERO);
//! let loss = spread.get_max_loss().unwrap_or(Positive::ZERO);
//! info!("Max Profit: {}, Max Loss: {}", profit, loss);
//! ```
//!
//! Refer to the documentation of each sub-module for more details on the specific
//! strategies and their usage.
//! # Base Module
//!
//! This module provides the base traits and structures for defining and working with
//! options trading strategies. It includes the `StrategyType` enum, `Strategy` struct,
//! and the `Strategies`, `Validable`, and `Optimizable` traits.
//!
//! ## StrategyType
//!
//! The `StrategyType` enum represents different types of trading strategies. Each variant
//! corresponds to a specific strategy, such as `BullCallSpread`, `BearPutSpread`, `IronCondor`, etc.
//!
//! ## Strategy
//!
//! The `Strategy` struct represents a trading strategy. It contains properties such as the strategy's
//! name, type, description, legs (positions), maximum profit, maximum loss, and break-even points.
//!
//! ## Strategies Trait
//!
//! The `Strategies` trait defines the common methods that a trading strategy should implement.
//! It includes methods for adding legs, retrieving legs, calculating break-even points, maximum profit,
//! maximum loss, total cost, and more.
//!
//! ## Validable Trait
//!
//! The `Validable` trait provides a method for validating a trading strategy. Strategies should implement
//! this trait to ensure they are valid before being used.
//!
//! ## Optimizable Trait
//!
//! The `Optimizable` trait extends the `Validable` and `Strategies` traits and adds methods for optimizing
//! a trading strategy. It includes methods for finding the optimal strategy based on different criteria,
//! such as best ratio or best area.
//!
//! ## Usage
//!
//! To define a new trading strategy, create a struct that implements the `Strategies`, `Validable`, and
//! optionally, the `Optimizable` traits. Implement the required methods for each trait based on the specific
//! behavior of your strategy.
//!
//! Example:
//!
//! ```rust
//! use optionstratlib::error::position::PositionError;
//! use optionstratlib::model::position::Position;
//! use positive::Positive;
//! use optionstratlib::strategies::base::{BreakEvenable, Positionable, Strategies, Validable};
//! use optionstratlib::strategies::{BasicAble, Strategable};
//!
//! struct MyStrategy {
//!     legs: Vec<Position>,
//!     // Other strategy-specific fields
//! }
//!
//! impl Validable for MyStrategy {
//!     fn validate(&self) -> bool {
//!        true
//!     }
//! }
//!
//!
//! impl Positionable for MyStrategy {
//!     fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
//!         Ok(self.legs.push(position.clone()))
//!     }
//!
//!  fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
//!         Ok(self.legs.iter().collect())
//!     }
//! }
//!
//! impl BreakEvenable for MyStrategy {}
//!
//!
//! impl BasicAble for MyStrategy {}
//!
//! impl Strategies for MyStrategy {}
//! ```
//! //! Example usage of the Iron Condor strategy:
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use tracing::info;
//! use optionstratlib::ExpirationDate;
//! use optionstratlib::strategies::iron_condor::IronCondor;
//! use positive::Positive;
//! use positive::pos_or_panic;
//! use optionstratlib::strategies::Strategies;
//!
//! let condor = IronCondor::new(
//!     "AAPL".to_string(),
//!     pos_or_panic!(150.0),   // underlying_price
//!     pos_or_panic!(155.0),   // short_call_strike
//!     pos_or_panic!(145.0),   // short_put_strike  
//!     pos_or_panic!(160.0),   // long_call_strike
//!     pos_or_panic!(140.0),   // long_put_strike
//!     ExpirationDate::Days(pos_or_panic!(30.0)),
//!     pos_or_panic!(0.2),   // implied_volatility
//!     dec!(0.01),   // risk_free_rate
//!     pos_or_panic!(0.02),   // dividend_yield
//!     Positive::ONE,   // quantity
//!     pos_or_panic!(1.5),   // premium_short_call
//!     Positive::ONE,   // premium_short_put
//!     Positive::TWO,   // premium_long_call
//!     pos_or_panic!(1.8),   // premium_long_put
//!     pos_or_panic!(5.0),   // open_fee
//!     pos_or_panic!(5.0),   // close_fee
//! );
//!
//! let max_profit = condor.get_max_profit().unwrap_or(Positive::ZERO);
//! let max_loss = condor.get_max_loss().unwrap_or(Positive::ZERO);
//! info!("Max Profit: {}, Max Loss: {}", max_profit, max_loss);
//! ```
//!
//! Refer to the documentation of each sub-module for more details on the specific
//! strategies and their usage.
//!

/// Options trading strategies module collection
///
/// This module provides implementations of various options trading strategies and utility functions
/// for options trading analysis. Each submodule represents a specific strategy or utility.
pub mod base;
/// Bear Call Spread strategy implementation
pub mod bear_call_spread;
/// Bear Put Spread strategy implementation  
pub mod bear_put_spread;
/// Internal module for strategy building utilities
mod build;
/// Bull Call Spread strategy implementation
pub mod bull_call_spread;
/// Bull Put Spread strategy implementation
pub mod bull_put_spread;
/// Call Butterfly strategy implementation  
pub mod call_butterfly;
/// Collar strategy implementation
pub mod collar;
/// Covered Call strategy implementation
pub mod covered_call;
/// Custom strategy implementation and utilities
pub mod custom;
/// Default implementation for strategies
pub mod default;
/// Delta-neutral strategy implementation and utilities
pub mod delta_neutral;

/// The `graph` module provides functionality for creating, managing, and
/// manipulating graph data structures. Common use cases include representing
/// networks, dependency graphs, and other graph-based relationships.
///
/// # Features
/// - Supports various graph representations (e.g., directed, undirected).
/// - Includes methods for traversing graphs (e.g., DFS, BFS).
/// - Provides utilities for adding and removing nodes and edges.
///
/// # Usage
/// To utilize this module, include it in your project and access its functions
/// to build and interact with graph structures:
///
/// Note: Implementations within the `graph` module may depend on specific
/// traits or types relevant to the graph operations.
///
/// For details on available graph types, functionalities, and examples, refer
/// to the corresponding methods and structs within the module.
pub mod graph;
/// Iron Butterfly strategy implementation
pub mod iron_butterfly;
/// Iron Condor strategy implementation
pub mod iron_condor;
/// Butterfly Spread strategy implementation
pub mod long_butterfly_spread;
/// Long Call strategy implementation
pub mod long_call;
/// Long Put strategy implementation
pub mod long_put;
/// Long Straddle strategy implementation
pub mod long_straddle;
/// Strangle strategy implementation
pub mod long_strangle;
/// Macros for options strategies
pub mod macros;
/// Poor Man's Covered Call strategy implementation
pub mod poor_mans_covered_call;
/// Probability calculations for options strategies
pub mod probabilities;
/// Protective Put strategy implementation
pub mod protective_put;
/// Shared traits for strategy categories
pub mod shared;
/// Short Call strategy implementation
pub mod short_butterfly_spread;
/// Short Call strategy implementation
pub mod short_call;
/// Short Put strategy implementation
pub mod short_put;
/// Short Straddle strategy implementation
pub mod short_straddle;
/// Short Strangle strategy implementation
pub mod short_strangle;
/// Utility functions for options calculations and analysis
pub mod utils;

pub use base::{BasicAble, Strategable, Strategies, StrategyBasics, Validable};
pub use bear_call_spread::BearCallSpread;
pub use bear_put_spread::BearPutSpread;
pub use build::model::StrategyRequest;
pub use build::traits::StrategyConstructor;
pub use bull_call_spread::BullCallSpread;
pub use bull_put_spread::BullPutSpread;
pub use call_butterfly::CallButterfly;
pub use delta_neutral::{
    AdjustmentAction, AdjustmentConfig, AdjustmentError, AdjustmentOptimizer, AdjustmentPlan,
    AdjustmentTarget, DELTA_THRESHOLD, DeltaAdjustment, DeltaInfo, DeltaNeutrality,
    PortfolioGreeks,
};
pub use iron_butterfly::IronButterfly;
pub use iron_condor::IronCondor;
pub use long_butterfly_spread::LongButterflySpread;
pub use long_call::LongCall;
pub use long_put::LongPut;
pub use long_straddle::LongStraddle;
pub use long_strangle::LongStrangle;
pub use poor_mans_covered_call::PoorMansCoveredCall;
pub use shared::{
    ButterflyStrategy, CondorStrategy, SpreadStrategy, StraddleStrategy, StrangleStrategy,
    aggregate_fees, aggregate_premiums, calculate_profit_ratio, credit_spread_break_even,
    debit_spread_break_even,
};
pub use short_butterfly_spread::ShortButterflySpread;
pub use short_call::ShortCall;
pub use short_put::ShortPut;
pub use short_straddle::ShortStraddle;
pub use short_strangle::ShortStrangle;
pub use utils::FindOptimalSide;
