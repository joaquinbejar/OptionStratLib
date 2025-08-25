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
    positive::Positive,
    types::{Action, OptionStyle, OptionType, Side},
};
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
    custom::CustomStrategy,
    delta_neutral::DeltaNeutrality,
    iron_butterfly::IronButterfly,
    iron_condor::IronCondor,
    long_butterfly_spread::LongButterflySpread,
    long_straddle::LongStraddle,
    long_strangle::LongStrangle,
    poor_mans_covered_call::PoorMansCoveredCall,
    probabilities::ProbabilityAnalysis,
    short_butterfly_spread::ShortButterflySpread,
    short_straddle::ShortStraddle,
    short_strangle::ShortStrangle,
    utils::FindOptimalSide,
};

// Greeks calculations
pub use crate::greeks::*;

// Pricing and profit calculations
pub use crate::pricing::payoff::*;
pub use crate::pricing::*;

// PnL calculations
pub use crate::pnl::PnLCalculator;

// Visualization
pub use crate::visualization::{Graph, GraphData, Series2D, Surface3D, TraceMode};

#[cfg(feature = "plotly")]
pub use crate::visualization::utils::make_surface;

// Chain operations
pub use crate::chains::{OptionData, StrategyLegs, chain::OptionChain, utils::OptionChainParams};

// Curves and surfaces
pub use crate::curves::{BasicCurves, Curvable, Curve, Point2D, StatisticalCurve};
pub use crate::surfaces::{BasicSurfaces, Point3D, Surfacable, Surface};

// Geometrics (commonly used in curve examples)
pub use crate::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject, Plottable};

// Volatility models
pub use crate::volatility::*;
// Error types (most commonly encountered)
pub use crate::error::*;

// Utility functions and traits
pub use crate::model::utils::ToRound;
pub use crate::utils::{
    Len, TimeFrame,
    others::calculate_log_returns,
    read_ohlcv_from_zip, setup_logger,
    time::{convert_time_frame, get_tomorrow_formatted, get_x_days_formatted},
};

// Commonly used external dependencies
pub use chrono::Utc;
pub use rust_decimal::Decimal;
pub use rust_decimal_macros::dec;

// Simulation types and functions
pub use crate::simulation::{
    WalkParams, WalkType, WalkTypeAble,
    randomwalk::RandomWalk,
    steps::{Step, Xstep, Ystep},
};

// Chain and series types and generators
pub use crate::chains::{
    OptionChainBuildParams, generator_optionchain, utils::OptionDataPriceParams,
};
pub use crate::series::{OptionSeries, OptionSeriesBuildParams, generator_optionseries};

// Volatility functions
pub use crate::volatility::{adjust_volatility, constant_volatility};

// Re-export the pos! and spos! macros for creating Positive values
pub use crate::pos;
pub use crate::spos;
