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
mod model;

pub use model::{DeltaAdjustment, DeltaInfo, DeltaNeutrality, DeltaPositionInfo, DELTA_THRESHOLD, DeltaNeutralResponse};
