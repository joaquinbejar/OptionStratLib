/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/12/24
******************************************************************************/

//! # Delta Neutral Strategies Module
//!
//! This module provides tools for managing delta-neutral strategies in options trading.
//! It includes definitions for representing delta adjustments, calculating delta-neutrality,
//! and suggesting adjustments to achieve or maintain delta-neutrality.
//!
//! ## Key Components
//!
//! - **DeltaAdjustment**: Represents the adjustments (buying/selling options or the underlying asset)
//!   required to move a strategy toward delta neutrality.
//! - **DeltaInfo**: Provides detailed information about the delta status of a strategy,
//!   including the net delta, individual position deltas, and current neutrality status.
//! - **DeltaNeutrality Trait**: A trait for calculating net delta, checking delta-neutrality,
//!   and suggesting adjustments to achieve delta-neutrality. It extends the `Greeks` trait for
//!   options calculations.
//!
//! ## Overview
//!
//! Delta neutrality is a core concept in options trading, where traders aim to balance
//! long and short deltas to minimize directional risk. Achieving delta neutrality
//! often involves adjusting position sizes, adding new positions, or trading the underlying asset.
//!
//! The module provides:
//! - Tools to calculate net delta for multi-position strategies.
//! - Utilities to evaluate whether a strategy is delta-neutral within a given threshold.
//! - Suggestions for adjustments (buy/sell options or the underlying) to achieve neutrality.
//!
//! ## Example Usage
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use tracing::info;
//! use optionstratlib::greeks::equations::{Greek, Greeks};
//! use optionstratlib::Positive;
//! use optionstratlib::{d2fu, pos};
//! use optionstratlib::strategies::delta_neutral::{DeltaNeutrality, DeltaAdjustment, DeltaInfo};
//!
//! struct MyStrategy { /* Implementation specifics */ }
//!
//! let strategy = MyStrategy { /* Implementation specifics */ };
//!
//! impl Greeks for MyStrategy {fn greeks(&self) -> Greek {
//!     Greek { delta: dec!(0.5),
//!             gamma: dec!(0.2),
//!             theta: dec!(0.1),
//!             vega: dec!(0.3),
//!             rho: dec!(0.4),
//!             rho_d: dec!(0.0)
//!           }
//!     }
//! }
//!
//! impl DeltaNeutrality for MyStrategy {
//!     fn calculate_net_delta(&self) -> DeltaInfo {
//!         DeltaInfo {
//!            net_delta: d2fu!(self.greeks().delta).unwrap(),
//!            individual_deltas: vec![],
//!            is_neutral: false,
//!            neutrality_threshold: 0.0,
//!            underlying_price: pos!(0.0),
//!         }
//!     }
//!
//! fn get_atm_strike(&self) -> Positive { pos!(0.0) } }
//!
//! // Calculate net delta
//! let delta_info = strategy.calculate_net_delta();
//! info!("{}", delta_info);
//!
//! // Check delta-neutrality within a 0.1 threshold
//! if !strategy.is_delta_neutral() {
//!     let adjustments = strategy.suggest_delta_adjustments();
//!     for adj in adjustments {
//!         info!("{:?}", adj);
//!     }
//! }
//! ```
//!
mod model;

pub use model::{DeltaAdjustment, DeltaInfo, DeltaNeutrality, DELTA_THRESHOLD};
