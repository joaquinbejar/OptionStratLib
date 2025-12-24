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
//! ## Extended Adjustment Features (Issue #187)
//!
//! - **AdjustmentAction**: Extended actions including adding new legs, rolling positions,
//!   and using underlying shares for hedging.
//! - **AdjustmentConfig**: Configuration for adjustment behavior and constraints.
//! - **PortfolioGreeks**: Aggregated Greeks at portfolio level.
//! - **AdjustmentTarget**: Target Greeks for optimization.
//! - **AdjustmentOptimizer**: Optimizer for finding best adjustment plans.
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
//! - Extended adjustment capabilities for multi-leg strategies.
//! - Portfolio-level Greeks aggregation and optimization.
//!
pub mod adjustment;
mod model;
pub mod optimizer;
pub mod portfolio;

pub use adjustment::{AdjustmentAction, AdjustmentConfig, AdjustmentError, AdjustmentPlan};
pub use model::{
    DELTA_THRESHOLD, DeltaAdjustment, DeltaInfo, DeltaNeutralResponse, DeltaNeutrality,
    DeltaPositionInfo,
};
pub use optimizer::AdjustmentOptimizer;
pub use portfolio::{AdjustmentTarget, PortfolioGreeks};
