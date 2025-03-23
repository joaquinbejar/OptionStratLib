/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/10/24
******************************************************************************/


use crate::chains::utils::OptionDataPriceParams;
use crate::constants::ZERO;
use crate::curves::{Curvable, Curve, Point2D};
use crate::error::CurveError;
use crate::geometrics::GeometricObject;
use crate::model::types::ExpirationDate;
use crate::pricing::payoff::Profit;
use crate::simulation::step::Step;
use crate::simulation::types::Walktypable;
use crate::strategies::Strategable;
use crate::utils::Len;
use crate::utils::time::{TimeFrame, convert_time_frame, units_per_year};
use crate::visualization::model::ChartPoint;
use crate::visualization::utils::Graph;
use crate::{Positive, pos};
use num_traits::FromPrimitive;
use rand::rng;
use rand_distr::{Distribution, Normal};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use tracing::{debug, info, trace, warn};


/// Enum defining different types of random walks
#[derive(Debug, Clone, Copy)]
pub enum WalkType {
    /// Standard Brownian motion (normal increments)
    Brownian {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return or growth rate)
        drift: Decimal,
        /// Volatility parameter (annualized standard deviation)
        volatility: Positive,
    },

    /// Geometric Brownian motion (log-normal increments)
    GeometricBrownian {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return or growth rate)
        drift: Decimal,
        /// Volatility parameter (annualized standard deviation)
        volatility: Positive,
    },

    /// Mean-reverting process (Ornstein-Uhlenbeck)
    MeanReverting {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (contribution to expected change outside of mean reversion)
        drift: Decimal,
        /// Volatility parameter (annualized standard deviation)
        volatility: Positive,
        /// Mean reversion speed (rate at which process reverts to mean)
        speed: Positive,
        /// Long-term mean (equilibrium level)
        mean: Positive,
    },

    /// Jump diffusion process (normal increments with occasional jumps)
    JumpDiffusion {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return of continuous part)
        drift: Decimal,
        /// Volatility parameter (annualized standard deviation of continuous part)
        volatility: Positive,
        /// Jump intensity (annual frequency of jumps)
        intensity: Positive,
        /// Jump size mean (average jump magnitude)
        jump_mean: Decimal,
        /// Jump size volatility (standard deviation of jump size)
        jump_volatility: Positive,
    },

    /// GARCH process (time-varying volatility)
    Garch {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return)
        drift: Decimal,
        /// Initial volatility parameter (starting volatility level)
        volatility: Positive,
        /// GARCH alpha parameter (impact of past observations)
        alpha: Positive,
        /// GARCH beta parameter (persistence of volatility)
        beta: Positive,
        /// Long-term variance (unconditional variance)
        omega: Positive,
    },

    /// Heston model (stochastic volatility)
    Heston {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return)
        drift: Decimal,
        /// Initial volatility parameter (starting volatility level)
        volatility: Positive,
        /// Mean reversion speed of volatility
        kappa: Positive,
        /// Long-term variance (equilibrium level of variance)
        theta: Positive,
        /// Volatility of volatility (standard deviation of variance process)
        xi: Positive,
        /// Correlation between price and volatility processes
        rho: Decimal,
    },

    /// Custom process defined by a function
    Custom {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected change)
        drift: Decimal,
        /// Volatility parameter (may be interpreted differently based on custom implementation)
        volatility: Positive,
    },
}


/// Parameters for stochastic process simulations (random walks).
///
/// This struct defines the configuration for generating random walks or price path simulations,
/// particularly useful in financial modeling, option pricing, and risk analysis contexts.
///
/// The generic parameters allow flexibility in the types of steps and values used in the walk,
/// with appropriate trait bounds to ensure mathematical operations can be performed correctly.
///
/// # Type Parameters
///
/// * `X` - The type for the x-axis steps (typically time), must support addition and conversion to positive values
/// * `Y` - The type for the y-axis values (typically price or rate), must support addition, conversion to positive values,
///         and implement the `Walktypable` trait for traversal operations
///
/// # Fields
///
/// * `size` - Number of steps or data points to generate in the simulation, determining the resolution and length of the walk
/// * `init_step` - Initial step values (starting point) for the random walk, typically representing initial time and price/rate values
/// * `walk_type` - The specific stochastic process algorithm to use for the simulation, defined by the `WalkType` enum
///
/// # Usage
///
/// This struct is typically instantiated at the beginning of a simulation process to configure how random walks will be generated.
/// It provides the foundation for various financial simulations including price path forecasting, Monte Carlo simulations for
/// options pricing, and risk analysis models.
///
#[derive(Debug, Copy, Clone)]
pub struct WalkParams<X, Y>
where
    X: std::ops::AddAssign + Into<Positive> + Copy,
    Y: std::ops::AddAssign + Into<Positive> + Copy + Walktypable,
{
    /// Number of steps or data points to generate in the simulation
    /// Determines the resolution and length of the resulting random walk
    pub size: usize,

    /// Initial step values (starting point) for the random walk
    /// Typically represents initial time and price/rate values
    pub init_step: Step<X, Y>,

    /// The specific stochastic process to use for generating the random walk
    /// Determines the mathematical properties and behavior of the simulated path
    pub walk_type: WalkType
}

