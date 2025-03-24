use crate::Positive;
use crate::simulation::steps::Step;
use crate::simulation::types::Walktypable;
use crate::simulation::{WalkType, WalkTypeAble};
use std::fmt::{Display, Formatter};
use std::ops::AddAssign;

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
/// * `size` - Number of steps or data points to generate in the simulation
/// * `init_step` - Initial step values (starting point) for the random walk
/// * `walk_type` - The specific stochastic process algorithm to use for the simulation
/// * `walker` - Implementation of the walk algorithm that satisfies the `WalkTypeAble` trait
///
/// # Usage
///
/// This struct is typically instantiated at the beginning of a simulation process to configure
/// how random walks will be generated. It provides the foundation for various financial simulations
/// including price path forecasting, Monte Carlo simulations for options pricing, and risk analysis models.
///
#[derive(Debug, Clone)]
pub struct WalkParams<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display + Sized,
    Y: Copy + Into<Positive> + Display + Sized,
{
    /// Number of steps or data points to generate in the simulation
    /// Determines the resolution and length of the resulting random walk
    pub size: usize,

    /// Initial step values (starting point) for the random walk
    /// Typically represents initial time and price/rate values
    pub init_step: Step<X, Y>,

    /// The specific stochastic process to use for generating the random walk
    /// Determines the mathematical properties and behavior of the simulated path
    pub walk_type: WalkType,

    /// Implementation of the walk algorithm that satisfies the WalkTypeAble trait
    /// Provides the concrete logic for generating steps according to the selected walk_type
    pub walker: Box<dyn WalkTypeAble<X, Y>>,
}

impl<X, Y> Display for WalkParams<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Copy + Into<Positive> + Display + Walktypable,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WalkParams {{ size: {}, init_step: {}, walk_type: {} }}",
            self.size, self.init_step, self.walk_type
        )
    }
}
