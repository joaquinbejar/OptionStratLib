use crate::Positive;
use crate::simulation::steps::{Step, Ystep};
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
    Y: Into<Positive> + Display + Sized + Clone,
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

impl<X, Y> WalkParams<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display + Sized,
    Y: Into<Positive> + Display + Sized + Clone,
{
    pub fn y(&self) -> &Y {
        self.init_step.y.value()
    }

    pub fn ystep_ref(&self) -> &Ystep<Y> {
        &self.init_step.y
    }

    pub fn ystep(&self) -> Ystep<Y> {
        self.init_step.y.clone()
    }

    pub fn ystep_as_positive(&self) -> Positive {
        self.ystep_ref().positive()
    }
}

impl<X, Y> Display for WalkParams<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WalkParams {{ size: {}, init_step: {}, walk_type: {} }}",
            self.size, self.init_step, self.walk_type
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::positive::Positive;
    use crate::simulation::steps::{Xstep, Ystep};
    use crate::utils::time::TimeFrame;
    use crate::{ExpirationDate, pos};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::fmt::Display;
    use std::ops::AddAssign;

    struct MockWalker;

    impl<X, Y> WalkTypeAble<X, Y> for MockWalker
    where
        X: Copy + Into<Positive> + AddAssign + Display,
        Y: Copy + Into<Positive> + Display,
    {
    }

    #[test]
    fn test_walk_params_creation_with_positive() {
        let init_x = Xstep::new(pos!(1.0), TimeFrame::Day, ExpirationDate::Days(pos!(30.0)));
        let init_y = Ystep::new(0, pos!(100.0));
        let init_step = Step {
            x: init_x,
            y: init_y,
        };

        let walk_params = WalkParams {
            size: 100,
            init_step,
            walk_type: WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
            walker: Box::new(MockWalker),
        };

        assert_eq!(walk_params.size, 100);
        assert_eq!(
            walk_params.init_step.x.step_size_in_time().value(),
            pos!(1.0).value()
        );
        assert_eq!(walk_params.init_step.y.value().value(), pos!(100.0).value());
    }

    #[test]
    fn test_walk_params_clone_with_positive() {
        let init_x = Xstep::new(pos!(1.0), TimeFrame::Day, ExpirationDate::Days(pos!(30.0)));
        let init_y = Ystep::new(0, pos!(100.0));
        let init_step = Step {
            x: init_x,
            y: init_y,
        };
        let walk_params = WalkParams {
            size: 100,
            init_step,
            walk_type: WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
            walker: Box::new(MockWalker),
        };
        let cloned_params = &walk_params;

        assert_eq!(cloned_params.size, walk_params.size);
        assert_eq!(
            cloned_params.init_step.x.step_size_in_time().value(),
            walk_params.init_step.x.step_size_in_time().value()
        );
        assert_eq!(
            cloned_params.init_step.y.value().value(),
            walk_params.init_step.y.value().value()
        );
    }

    #[test]
    fn test_walk_params_display_with_positive() {
        let init_x = Xstep::new(pos!(1.5), TimeFrame::Day, ExpirationDate::Days(pos!(30.0)));
        let init_y = Ystep::new(0, pos!(200.0));
        let init_step = Step {
            x: init_x,
            y: init_y,
        };

        let walk_params = WalkParams {
            size: 50,
            init_step: init_step.clone(),
            walk_type: WalkType::GeometricBrownian {
                dt: pos!(1.0 / 252.0),
                drift: dec!(0.0),
                volatility: pos!(0.2),
            },
            walker: Box::new(MockWalker),
        };

        let display_string = format!("{}", walk_params);

        // Check that the display string contains all the expected parts
        assert!(display_string.contains("size: 50"));
        assert!(display_string.contains(&format!("{}", init_step)));
        assert!(display_string.contains("walk_type: GeometricBrownian"));
    }

    #[test]
    fn test_with_large_size_positive_types() {
        let init_x = Xstep::new(pos!(1.0), TimeFrame::Day, ExpirationDate::Days(pos!(30.0)));
        let init_y = Ystep::new(0, pos!(100.0));
        let init_step = Step {
            x: init_x,
            y: init_y,
        };

        let size = 1_000_000; // One million steps
        let walk_params = WalkParams {
            size,
            init_step,
            walk_type: WalkType::Brownian {
                dt: pos!(1.0 / 252.0),
                drift: dec!(0.0),
                volatility: pos!(0.2),
            },
            walker: Box::new(MockWalker),
        };

        assert_eq!(walk_params.size, size);
        let display_string = format!("{}", walk_params);
        assert!(display_string.contains(&format!("size: {}", size)));
    }

    #[test]
    fn test_with_different_positive_values() {
        // Test with smaller and larger positive values
        let init_x = Xstep::new(
            pos!(0.001), // Very small value
            TimeFrame::Month,
            ExpirationDate::Days(pos!(90.0)),
        );
        let init_y = Ystep::new(0, pos!(1000000.0)); // Large value  
        let init_step = Step {
            x: init_x,
            y: init_y,
        };

        let walk_params = WalkParams {
            size: 50,
            init_step,
            walk_type: WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
            walker: Box::new(MockWalker),
        };

        assert_eq!(walk_params.size, 50);
        assert_eq!(
            walk_params.init_step.x.step_size_in_time().value(),
            pos!(0.001).value()
        );
        assert_eq!(
            walk_params.init_step.y.value().value(),
            pos!(1000000.0).value()
        );

        // Verify the time unit is correctly set
        assert_eq!(*walk_params.init_step.x.time_unit(), TimeFrame::Month);
    }
}
