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

/// * `projections` - Option projections onto curves and surfaces
///   (`BasicCurves`, `BasicSurfaces`) and their `OptionChain` implementations.
pub mod projections;

/// * `rnd` - Risk-neutral density and volatility-skew extraction from an
///   option chain (Breeden-Litzenberger), together with the
///   `impl RNDAnalysis for OptionChain`.
pub mod rnd;

pub use projections::{BasicCurves, BasicSurfaces};
pub use rnd::{RNDAnalysis, RNDParameters, RNDResult, RNDStatistics};

/// * `profit_range` - [`profit_range::ProfitLossRange`] and the
///   [`profit_range::ProfitRangeProbability`] extension
///   trait that fills a core `ProfitLossRange` with its expiry probability.
pub mod profit_range;

pub use profit_range::{ProfitLossRange, ProfitRangeProbability};
