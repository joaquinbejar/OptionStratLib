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
//! use optionstratlib::Positive;
//! use optionstratlib::pos;
//! use optionstratlib::strategies::Strategies;
//!
//! let spread = BullCallSpread::new(
//!     "SP500".to_string(),
//!     pos!(5780.0),   // underlying_price
//!     pos!(5750.0),   // long_strike_itm  
//!     pos!(5820.0),   // short_strike
//!     ExpirationDate::Days(pos!(2.0)),
//!     pos!(0.18),   // implied_volatility
//!     dec!(0.05),   // risk_free_rate
//!     Positive::ZERO,   // dividend_yield
//!     pos!(2.0),   // long quantity
//!     85.04,   // premium_long
//!     29.85,   // premium_short
//!     0.78,   // open_fee_long
//!     0.78,   // open_fee_long
//!     0.73,   // close_fee_long
//!     0.73,   // close_fee_short
//! );
//!
//! let profit = spread.max_profit().unwrap_or(Positive::ZERO);
//! let loss = spread.max_loss().unwrap_or(Positive::ZERO);
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
//! use optionstratlib::Positive;
//! use optionstratlib::strategies::base::{Positionable, Strategies, Validable};
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
//! impl Strategies for MyStrategy {}
//! ```
//! //! Example usage of the Iron Condor strategy:
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use tracing::info;
//! use optionstratlib::ExpirationDate;
//! use optionstratlib::strategies::iron_condor::IronCondor;
//! use optionstratlib::Positive;
//! use optionstratlib::pos;
//! use optionstratlib::strategies::Strategies;
//!
//! let condor = IronCondor::new(
//!     "AAPL".to_string(),
//!     pos!(150.0),   // underlying_price
//!     pos!(155.0),   // short_call_strike
//!     pos!(145.0),   // short_put_strike  
//!     pos!(160.0),   // long_call_strike
//!     pos!(140.0),   // long_put_strike
//!     ExpirationDate::Days(pos!(30.0)),
//!     pos!(0.2),   // implied_volatility
//!     dec!(0.01),   // risk_free_rate
//!     pos!(0.02),   // dividend_yield
//!     pos!(1.0),   // quantity
//!     1.5,   // premium_short_call
//!     1.0,   // premium_short_put
//!     2.0,   // premium_long_call
//!     1.8,   // premium_long_put
//!     5.0,   // open_fee
//!     5.0,   // close_fee
//! );
//!
//! let max_profit = condor.max_profit().unwrap_or(Positive::ZERO);
//! let max_loss = condor.max_loss().unwrap_or(Positive::ZERO);
//! info!("Max Profit: {}, Max Loss: {}", max_profit, max_loss);
//! ```
//!
//! Refer to the documentation of each sub-module for more details on the specific
//! strategies and their usage.
pub mod base;
pub mod bear_call_spread;
pub mod bear_put_spread;
pub mod bull_call_spread;
pub mod bull_put_spread;
pub mod butterfly_spread;
pub mod call_butterfly;
pub mod collar;
pub mod covered_call;
pub mod custom;
pub mod delta_neutral;
pub mod iron_butterfly;
pub mod iron_condor;
pub mod poor_mans_covered_call;
pub mod probabilities;
pub mod protective_put;
pub mod straddle;
pub mod strangle;
pub mod utils;

pub use base::Strategies;
pub use bear_call_spread::BearCallSpread;
pub use bear_put_spread::BearPutSpread;
pub use bull_call_spread::BullCallSpread;
pub use bull_put_spread::BullPutSpread;
pub use butterfly_spread::{LongButterflySpread, ShortButterflySpread};
pub use call_butterfly::CallButterfly;
// pub use collar::Collar;
// pub use covered_call::CoveredCall;
pub use custom::CustomStrategy;
pub use delta_neutral::{DeltaAdjustment, DeltaInfo, DeltaNeutrality, DELTA_THRESHOLD};
pub use iron_butterfly::IronButterfly;
pub use iron_condor::IronCondor;
pub use poor_mans_covered_call::PoorMansCoveredCall;
pub use straddle::{LongStraddle, ShortStraddle};
pub use strangle::{LongStrangle, ShortStrangle};
pub use utils::FindOptimalSide;
