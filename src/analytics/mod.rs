//! Analytics owned by no strategy: neutral probability kernels and models
//! that any caller can evaluate from prices, volatilities and dates alone.
//!
//! This module is the monolith-side home of the future
//! `optionstratlib-analytics` crate for code that has no natural place in
//! `pnl`, `risk` or `metrics`. It depends downward only (core model, Greeks);
//! the strategy layer consumes it through its `ProbabilityAnalysis` trait and
//! re-exports the historical `strategies::probabilities` paths.

pub mod probability;

pub use probability::{
    PriceTrend, VolatilityAdjustment, calculate_price_probability,
    calculate_single_point_probability,
};
