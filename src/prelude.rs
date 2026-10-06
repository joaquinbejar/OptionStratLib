/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/8/25
******************************************************************************/

//! # OptionStratLib Prelude
//!
//! The prelude module provides a convenient way to import the most commonly used
//! types, traits, and functions from the OptionStratLib library. This module is
//! designed to reduce the amount of boilerplate imports needed when working with
//! the library.
//!
//! ## Usage
//!
//! Add this to your imports to get access to the most commonly used items:
//!
//! ```rust
//! use optionstratlib::prelude::*;
//! ```
//!
//! This will import all the essential types and traits you need for most
//! options trading and strategy development tasks.

// Core model types
pub use crate::model::{
    BasicAxisTypes, ExpirationDate, Options, Position, Trade,
    types::{Action, OptionStyle, OptionType, Side},
};
#[cfg(feature = "strategies")]
pub use crate::strategies::{
    StrategyConstructor,
    base::{
        BasicAble, BreakEvenable, Optimizable, Positionable, Strategable, Strategies, StrategyType,
        Validable,
    },
    // Specific strategy implementations (commonly used)
    bear_call_spread::BearCallSpread,
    bear_put_spread::BearPutSpread,
    bull_call_spread::BullCallSpread,
    bull_put_spread::BullPutSpread,
    call_butterfly::CallButterfly,
    collar::Collar,
    covered_call::CoveredCall,
    custom::CustomStrategy,
    delta_neutral::{
        AdjustmentAction, AdjustmentConfig, AdjustmentError, AdjustmentOptimizer, AdjustmentPlan,
        AdjustmentTarget, DeltaNeutrality, PortfolioGreeks,
    },
    iron_butterfly::IronButterfly,
    iron_condor::IronCondor,
    long_butterfly_spread::LongButterflySpread,
    long_call::LongCall,
    long_put::LongPut,
    long_straddle::LongStraddle,
    long_strangle::LongStrangle,
    poor_mans_covered_call::PoorMansCoveredCall,
    probabilities::ProbabilityAnalysis,
    protective_put::ProtectivePut,
    short_butterfly_spread::ShortButterflySpread,
    short_call::ShortCall,
    short_put::ShortPut,
    short_straddle::ShortStraddle,
    short_strangle::ShortStrangle,
};

// Greeks calculations
#[cfg(feature = "pricing")]
pub use crate::greeks::*;

// Pricing and profit calculations
pub use crate::model::payoff::{Payoff, PayoffInfo};
#[cfg(feature = "pricing")]
pub use crate::pricing::*;

// PnL calculations
#[cfg(feature = "analytics")]
pub use crate::pnl::{PnL, PnLCalculator};

#[cfg(feature = "backtest")]
pub use crate::backtesting::*;

// Visualization
#[cfg(feature = "visualization")]
pub use crate::visualization::{Graph, GraphData, Series2D, Surface3D, TraceMode};

#[cfg(feature = "plotly")]
pub use crate::visualization::make_surface;

// Chain operations
#[cfg(feature = "market")]
pub use crate::chains::{
    OptionData, StrategyLegs,
    chain::OptionChain,
    utils::{FindOptimalSide, OptionChainParams},
};

// Curves and surfaces
#[cfg(feature = "analytics")]
pub use crate::analytics::{BasicCurves, BasicSurfaces};
#[cfg(feature = "math")]
pub use crate::curves::{Curvable, Curve, Point2D, StatisticalCurve};
#[cfg(feature = "math")]
pub use crate::surfaces::{Point3D, Surfacable, Surface};

// Geometrics (commonly used in curve examples)
#[cfg(feature = "math")]
pub use crate::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject};
#[cfg(feature = "visualization")]
pub use crate::visualization::Plottable;

// Volatility models
#[cfg(feature = "pricing")]
pub use crate::volatility::*;

// Performance metrics
#[cfg(feature = "analytics")]
pub use crate::metrics::*;

// Error types (most commonly encountered)
#[cfg(feature = "market")]
pub use crate::error::ChainError;
#[cfg(feature = "market")]
pub use crate::error::OhlcvError;
#[cfg(feature = "strategies")]
pub use crate::error::StrategyError;
#[cfg(feature = "math")]
pub use crate::error::{CurveError, InterpolationError, MetricsError, SurfaceError};
pub use crate::error::{DecimalError, OperationErrorKind, OptionsError, PositionError};
#[cfg(feature = "visualization")]
pub use crate::error::{Error, GraphError};
#[cfg(feature = "pricing")]
pub use crate::error::{GreeksError, PricingError, VolatilityError};
#[cfg(feature = "analytics")]
pub use crate::error::{ProbabilityError, TransactionError};

// Utility functions and traits
#[cfg(feature = "io")]
pub use crate::chains::csv::{OhlcvCandle, read_ohlcv_from_zip};
pub use crate::model::utils::ToRound;
pub use crate::utils::{
    Len, TimeFrame,
    numeric::calculate_log_returns,
    time::{convert_time_frame, get_tomorrow_formatted, get_x_days_formatted},
};

// Commonly used external dependencies
pub use chrono::Utc;
pub use optionstratlib_core::model::Positive;
pub use optionstratlib_core::{assert_pos_relative_eq, pos_or_panic, spos};
pub use rust_decimal::Decimal;
pub use rust_decimal::prelude::ToPrimitive;
pub use rust_decimal_macros::dec;
pub use std::path::Path;

// Simulation types and functions
#[cfg(feature = "backtest")]
pub use crate::backtesting::{Simulate, SimulationStats};
#[cfg(feature = "simulation")]
pub use crate::simulation::{
    ExitPolicy, WalkParams, WalkPath, WalkType, WalkTypeAble, WalkTypeAbleClone, check_exit_policy,
    expanding_window_vols, generator_positive,
    randomwalk::RandomWalk,
    simulator::Simulator,
    steps::{Step, Xstep, Ystep},
    walk_steps, walk_steps_par,
};

// Chain and series types and generators
#[cfg(feature = "synthetic")]
pub use crate::chains::generator_optionchain;
#[cfg(feature = "market")]
pub use crate::chains::{OptionChainBuildParams, utils::OptionDataPriceParams};
#[cfg(feature = "synthetic")]
pub use crate::series::generator_optionseries;
#[cfg(feature = "market")]
pub use crate::series::{OptionSeries, OptionSeriesBuildParams};

// Volatility functions
#[cfg(feature = "pricing")]
pub use crate::volatility::{adjust_volatility, constant_volatility};
pub use tracing::{debug, error, info, trace, warn};
