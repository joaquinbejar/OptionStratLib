//! Analytics owned by no strategy: neutral probability kernels and models
//! that any caller can evaluate from prices, volatilities and dates alone.
//!
//! This module holds the analytics code that has no natural place in `pnl`,
//! `risk` or `metrics`. Like the rest of `optionstratlib-analytics` it depends
//! downward only (core, math, pricing and market); the strategy layer consumes it through its `ProbabilityAnalysis` trait.
//! The kernels and their inputs have one public path, `analytics::…`; the
//! strategy layer does not re-export them.

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

pub use projections::{BasicCurves, BasicSurfaces, OptionChainProjections};
pub use rnd::{RNDAnalysis, RNDParameters, RNDResult, RNDStatistics};

/// * `profit_range` - [`profit_range::ProfitLossRange`] and the
///   [`profit_range::ProfitRangeProbability`] extension
///   trait that fills a core `ProfitLossRange` with its expiry probability.
pub mod profit_range;

pub use profit_range::{ProfitLossRange, ProfitRangeProbability};
