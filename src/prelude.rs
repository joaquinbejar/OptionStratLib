/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/8/25
******************************************************************************/

//! # OptionStratLib Prelude
//!
//! `use optionstratlib::prelude::*;` brings in the domain vocabulary and the
//! traits whose methods you call on library types, and nothing else:
//!
//! ```rust
//! use optionstratlib::prelude::*;
//!
//! let strike = pos_or_panic!(100.0);
//! let expiry = ExpirationDate::Days(pos_or_panic!(30.0));
//! let side = Side::Long;
//! # let _ = (strike, expiry, side, dec!(0.05));
//! ```
//!
//! ## What is in it, and why
//!
//! Every item is re-exported from its canonical path (#550): the flat parent
//! path where the defining crate has one (`crate::strategies::BullCallSpread`),
//! the defining module otherwise (`crate::strategies::base::BreakEvenable`).
//! An item is here for one of three reasons:
//!
//! 1. **Domain vocabulary** used by almost every program: the contract and
//!    position types, `Positive` and its macros, `Decimal`/`dec!`, `Utc`.
//! 2. **Extension traits** whose methods are the way a library type is used,
//!    and which must be in scope for the method call to compile:
//!    `OptionPricing` (`Options::calculate_price_*`), `Greeks`, `Profit`,
//!    `VolatilitySmile` (`OptionChain::smile`), `PnLCalculator`,
//!    `BasicCurves`/`BasicSurfaces` (`curve`/`surface` on chains and
//!    strategies), the strategy traits, `GeometricObject`, `WalkTypeAble`,
//!    `Len` (`RandomWalk::len`), `Simulate`, `Graph`, `Plottable`.
//! 3. **The entry type of each capability**: `Curve`/`Surface`, `OptionChain`
//!    and `OptionSeries` with their build parameters, the concrete strategies,
//!    `RandomWalk`/`Simulator` with their step and walk types.
//!
//! The third-party re-exports (`Decimal`, `dec!`, `Utc` and the `tracing`
//! macros `debug!`, `error!`, `info!`, `trace!`, `warn!`) are the ones
//! ADR-0001 D7 keeps in the facade prelude; `Positive` and its macros are
//! core's foundational re-exports (ADR-0001 D8).
//!
//! **`dec!` needs `rust_decimal` in your manifest** (#777). The re-exported
//! `rust_decimal_macros::dec` checks its literal at compile time and expands
//! to `::rust_decimal` paths, so a crate that writes `dec!` must depend on
//! `rust_decimal` directly, next to the facade; without it the build fails
//! with ``cannot find `rust_decimal` ``. `Decimal` alone needs nothing extra.
//!
//! ```toml
//! [dependencies]
//! optionstratlib = "0.22.0"
//! rust_decimal = "1.43"
//! ```
//!
//! `make check-release-notes` proves both halves against the compiler.
//!
//! The prelude has no free functions, no error types (import them from
//! [`crate::error`]), no glob re-exports and no standard-library items, so a
//! new public item in a component never enters it silently.
//!
//! ## Features
//!
//! Each capability's items are present only when its facade feature is on;
//! the prelude never enables a capability by itself, so
//! `default-features = false` gets only the core items.
//!
//! | Items | Facade feature |
//! | --- | --- |
//! | `Options`, `Position`, `ExpirationDate`, `OptionStyle`, `OptionType`, `Side`, `TimeFrame`, `Len`, `Positive`, `pos_or_panic!`, `spos!`, `assert_pos_relative_eq!`, `Decimal`, `dec!`, `Utc`, `debug!`, `error!`, `info!`, `trace!`, `warn!` | always |
//! | `Curve`, `Point2D`, `Surface`, `Point3D`, `GeometricObject`, `ConstructionMethod`, `ConstructionParams` | `math` |
//! | `OptionPricing`, `Greeks`, `Profit`, `VolatilitySmile` | `pricing` |
//! | `OptionChain`, `OptionData`, `OptionChainBuildParams`, `OptionDataPriceParams`, `FindOptimalSide`, `OptionSeries`, `OptionSeriesBuildParams` | `market` |
//! | `PnL`, `PnLCalculator`, `BasicCurves`, `BasicSurfaces` | `analytics` |
//! | the 22 concrete strategies and the traits `BasicAble`, `BreakEvenable`, `Strategies`, `Validable`, `Positionable`, `Strategable`, `Optimizable`, `StrategyConstructor`, `ProbabilityAnalysis`, `DeltaNeutrality` | `strategies` |
//! | `RandomWalk`, `Simulator`, `WalkParams`, `WalkType`, `WalkTypeAble`, `Step`, `Xstep`, `Ystep`, `ExitPolicy` | `simulation` |
//! | `Simulate` | `backtest` |
//! | `Graph`, `Plottable` | `visualization` |
//!
//! No prelude item depends on `io`, `synthetic`, `plotly`, `static_export`
//! or `async`.
//!
//! ## When to import directly
//!
//! Import from the canonical module instead of the prelude for anything
//! else: free functions (`optionstratlib::pricing::black_scholes`,
//! `optionstratlib::greeks::delta`, `optionstratlib::chains::generator_optionchain`),
//! errors (`optionstratlib::error::{Error, PricingError, ..}`; the aggregate
//! `Error` exists only with `visualization`), the chart
//! models you build when implementing `Graph`
//! (`optionstratlib::visualization::{GraphData, Series2D, ..}`), the
//! option-chain metric traits (`optionstratlib::metrics::..`), the
//! delta-adjustment types (`optionstratlib::strategies::Adjustment..`), and
//! whenever you depend on a component crate directly rather than on the
//! facade. Naming the module also documents which capability, and so which
//! feature, the code relies on.
//!
//! ## Removed in 0.22
//!
//! The 0.21 prelude also glob-exported `pricing`, `greeks`, `volatility`,
//! `metrics` and `backtesting`, and re-exported errors, free functions,
//! chart models, `std::path::Path` and `ToPrimitive`. Those now come from
//! their modules; the CHANGELOG lists each with its canonical path (#551).
//!
//! ```rust
//! # #[cfg(feature = "visualization")]
//! use optionstratlib::error::Error;
//! use std::path::Path;
//! # #[cfg(feature = "pricing")]
//! use optionstratlib::pricing::black_scholes;
//! ```
//!
//! ```compile_fail,E0432
//! use optionstratlib::prelude::Error;
//! ```
//!
//! ```compile_fail,E0432
//! use optionstratlib::prelude::Path;
//! ```
//!
//! ```compile_fail,E0432
//! use optionstratlib::prelude::black_scholes;
//! ```
//!
//! ```compile_fail,E0432
//! use optionstratlib::prelude::black_scholes_model;
//! ```

// 1. Core: always present.
pub use crate::model::{ExpirationDate, OptionStyle, OptionType, Options, Position, Side};
pub use crate::utils::{Len, TimeFrame};
pub use chrono::Utc;
pub use optionstratlib_core::model::Positive;
pub use optionstratlib_core::{assert_pos_relative_eq, pos_or_panic, spos};
pub use rust_decimal::Decimal;
/// Needs `rust_decimal` as a direct dependency of the crate that writes it:
/// the macro expands to `::rust_decimal` paths (#777, see the module docs).
pub use rust_decimal_macros::dec;
pub use tracing::{debug, error, info, trace, warn};

// 2. Math.
#[cfg(feature = "math")]
pub use crate::curves::{Curve, Point2D};
#[cfg(feature = "math")]
pub use crate::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject};
#[cfg(feature = "math")]
pub use crate::surfaces::{Point3D, Surface};

// 3. Pricing: the traits that put pricing, Greeks and profit methods on
// `Options`, `Position` and the strategies.
#[cfg(feature = "pricing")]
pub use crate::greeks::Greeks;
#[cfg(feature = "pricing")]
pub use crate::pricing::{OptionPricing, Profit};
#[cfg(feature = "pricing")]
pub use crate::volatility::VolatilitySmile;

// 4. Market.
#[cfg(feature = "market")]
pub use crate::chains::utils::OptionDataPriceParams;
#[cfg(feature = "market")]
pub use crate::chains::{FindOptimalSide, OptionChain, OptionChainBuildParams, OptionData};
#[cfg(feature = "market")]
pub use crate::series::{OptionSeries, OptionSeriesBuildParams};

// 5. Analytics.
#[cfg(feature = "analytics")]
pub use crate::analytics::{BasicCurves, BasicSurfaces};
#[cfg(feature = "analytics")]
pub use crate::pnl::{PnL, PnLCalculator};

// 6. Strategies: the concrete strategies and the traits that carry their
// methods.
#[cfg(feature = "strategies")]
pub use crate::strategies::base::{BreakEvenable, Optimizable, Positionable};
#[cfg(feature = "strategies")]
pub use crate::strategies::custom::CustomStrategy;
#[cfg(feature = "strategies")]
pub use crate::strategies::probabilities::ProbabilityAnalysis;
#[cfg(feature = "strategies")]
pub use crate::strategies::{
    BasicAble, BearCallSpread, BearPutSpread, BullCallLadder, BullCallSpread, BullPutSpread,
    Collar, CoveredCall, DeltaNeutrality, IronButterfly, IronCondor, LongButterflySpread, LongCall,
    LongPut, LongStraddle, LongStrangle, PoorMansCoveredCall, ProtectivePut, ShortButterflySpread,
    ShortCall, ShortPut, ShortStraddle, ShortStrangle, Strategable, Strategies,
    StrategyConstructor, Validable,
};

// 7. Simulation.
#[cfg(feature = "simulation")]
pub use crate::simulation::randomwalk::RandomWalk;
#[cfg(feature = "simulation")]
pub use crate::simulation::simulator::Simulator;
#[cfg(feature = "simulation")]
pub use crate::simulation::steps::{Step, Xstep, Ystep};
#[cfg(feature = "simulation")]
pub use crate::simulation::{ExitPolicy, WalkParams, WalkType, WalkTypeAble};

// 8. Backtest.
#[cfg(feature = "backtest")]
pub use crate::backtesting::Simulate;

// 9. Visualization.
#[cfg(feature = "visualization")]
pub use crate::visualization::{Graph, Plottable};
