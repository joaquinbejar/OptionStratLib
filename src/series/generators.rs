use crate::error::ChainError;
use crate::series::OptionSeries;
use crate::simulation::steps::{Step, Ystep};
use crate::simulation::{WalkParams, WalkType};
use crate::utils::TimeFrame;
use crate::utils::others::calculate_log_returns;
use crate::volatility::{adjust_volatility, constant_volatility};
use crate::{Positive, pos};
use core::option::Option;
use rust_decimal::Decimal;
use tracing::debug;

/// Creates a new `OptionSeries` from a given previous `Ystep` series, a new price,
/// and an optional volatility value.
///
/// # Parameters
/// - `previous_y_step`: A reference to the previous `Ystep` series of type `OptionSeries`.
///   It represents the state of the series prior to this computation.
/// - `new_price`: A reference to a `Positive` value representing the new price to use as an input
///   for the computation.
/// - `volatility`: An optional `Positive` value representing the volatility used in the calculation.
///   If `None`, a default behavior or calculation is assumed.
///
/// # Returns
/// Returns a `Result` that, on success, contains the newly created `OptionSeries`. On failure,
/// it contains a boxed dynamic error with details about what went wrong.
///
/// # Errors
/// This function can return an error if:
/// - The `previous_y_step` contains invalid or inconsistent data for creating the new series.
/// - The `new_price` or optional `volatility`, if provided, result in an invalid computation.
/// - Any other unexpected error occurs during the processing.
///
fn create_series_from_step(
    previous_y_step: &Ystep<OptionSeries>,
    new_price: &Positive,
    volatility: Option<Positive>,
) -> Result<OptionSeries, ChainError> {
    let series = previous_y_step.value();
    let mut series_params = series.to_build_params()?;
    series_params.set_underlying_price(new_price);
    if let Some(volatility) = volatility {
        series_params.set_implied_volatility(volatility);
    }
    let new_chain = OptionSeries::build_series(&series_params);
    Ok(new_chain)
}

/// Generates a series of steps based on the given `WalkParams` and the specific type of walk
/// defined within. This function supports various stochastic processes like Brownian motion,
/// Geometric Brownian motion, mean-reverting processes, jump diffusion, GARCH, Heston, and
/// others, as well as historical data-based simulations.
///
/// # Parameters
///
/// - `walk_params`: A reference to a `WalkParams<Positive, OptionSeries>` structure that defines the
///   initialization parameters, type of walk, and the associated step generator (`walker`).
///
/// # Returns
///
/// Returns a `Vec<Step<Positive, OptionSeries>>` containing a series of steps generated based
/// on the specified type of walk and its associated parameters. Each step combines both the
/// progression in the x-axis and the calculated output (y-axis) using the mathematical rules
/// of the given walk type. Returns an empty vector if the walk type does not yield any results
/// (e.g., due to insufficient historical price data).
///
/// # Walk Types
///
/// Depending on the variant of the `walk_params.walk_type` field, the function performs different types
/// of stochastic processes:
///
/// 1. `Brownian`: Simulates a basic Brownian motion with a given volatility.
/// 2. `GeometricBrownian`: Calculates geometric Brownian motion.
/// 3. `LogReturns`: Generates a process based on log-normal returns.
/// 4. `MeanReverting`: Simulates a mean-reverting process.
/// 5. `JumpDiffusion`: Implements Merton's Jump Diffusion.
/// 6. `Garch`: Utilizes the GARCH (Generalized Autoregressive Conditional Heteroskedasticity) model.
/// 7. `Heston`: Applies the Heston stochastic volatility model.
/// 8. `Custom`: Allows for custom walk logic implemented through the user-defined walker.
/// 9. `Historical`: Uses historical price data adjusted for log returns and implied volatility.
///
/// # Implementation Details
///
/// - Volatility is extracted or calculated for each walk type to guide the stochastic process.
/// - For the `Historical` walk type, log returns are calculated from the given price data.
///   If the `prices` array is empty or has insufficient data, the resulting steps vector will be empty.
///
/// - The initial step is removed from the generated steps to avoid duplication with the input
///   initialization step (`init_step`).
///
/// - The x-coordinates (`x`) and y-coordinates (`y`, i.e., `OptionSeries`) are iteratively calculated
///   and adjusted using the respective walk model.
///
/// # Panics
///
/// This function ensures that the total size of the generated steps does not exceed the `walk_params.size`.
/// If any unexpected behavior occurs during step generation or transformation, it will panic.
///
pub fn generator_optionseries(
    walk_params: &WalkParams<Positive, OptionSeries>,
) -> Vec<Step<Positive, OptionSeries>> {
    debug!("{}", walk_params);
    let (mut y_steps, volatility) = match &walk_params.walk_type {
        WalkType::Brownian { volatility, .. } => (
            walk_params.walker.brownian(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::GeometricBrownian { volatility, .. } => (
            walk_params.walker.geometric_brownian(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::LogReturns { volatility, .. } => (
            walk_params.walker.log_returns(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::MeanReverting { volatility, .. } => (
            walk_params.walker.mean_reverting(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::JumpDiffusion { volatility, .. } => (
            walk_params.walker.jump_diffusion(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::Garch { volatility, .. } => (
            walk_params.walker.garch(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::Heston { volatility, .. } => (
            walk_params.walker.heston(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::Custom { volatility, .. } => (
            walk_params.walker.custom(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::Telegraph { volatility, .. } => (
            walk_params.walker.telegraph(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::Historical {
            timeframe, prices, ..
        } => {
            if prices.is_empty() || prices.len() < walk_params.size {
                (Vec::new(), None)
            } else {
                let log_returns: Vec<Decimal> = calculate_log_returns(prices)
                    .unwrap()
                    .iter()
                    .map(|p| p.to_dec())
                    .collect();
                let constant_volatility = constant_volatility(&log_returns).unwrap();
                let implied_volatility =
                    adjust_volatility(constant_volatility, *timeframe, TimeFrame::Year).unwrap();
                (
                    walk_params.walker.historical(walk_params).unwrap(),
                    Some(implied_volatility),
                )
            }
        }
    };
    if y_steps.is_empty() {
        return vec![];
    }

    let _ = y_steps.remove(0); // remove initial step from y_steps to avoid early return
    let mut steps: Vec<Step<Positive, OptionSeries>> = vec![walk_params.init_step.clone()];
    let mut previous_x_step = walk_params.init_step.x;
    let mut previous_y_step = walk_params.ystep();

    if let Some(volatility) = volatility {
        volatility
    } else {
        pos!(0.20)
    };

    for y_step in y_steps.iter() {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            Err(_) => break,
        };
        // convert y_step to OptionSeries
        let y_step_series: OptionSeries =
            create_series_from_step(&previous_y_step, y_step, volatility).unwrap();
        previous_y_step = previous_y_step.next(y_step_series).clone();
        let step = Step {
            x: previous_x_step,
            y: previous_y_step.clone(),
        };

        steps.push(step)
    }

    assert!(steps.len() <= walk_params.size);
    steps
}

#[cfg(test)]
mod tests_generator_optionseries {
    use super::*;
    use crate::chains::utils::OptionChainBuildParams;
    use crate::chains::utils::OptionDataPriceParams;
    use crate::series::{OptionSeries, OptionSeriesBuildParams};
    use crate::simulation::steps::{Step, Xstep, Ystep};
    use crate::simulation::{WalkParams, WalkType, WalkTypeAble};
    use crate::utils::TimeFrame;
    use crate::utils::time::convert_time_frame;
    use crate::{ExpirationDate, assert_pos_relative_eq, spos};
    use rust_decimal_macros::dec;

    // Mock Walker for testing
    #[derive(Clone)]
    struct TestWalker {}
    impl TestWalker {
        fn new() -> Self {
            TestWalker {}
        }
    }
    impl WalkTypeAble<Positive, OptionSeries> for TestWalker {}

    // Helper function to create a test OptionSeries
    fn create_test_option_series() -> OptionSeries {
        // Create basic chain parameters
        let price_params = OptionDataPriceParams::new(
            Some(Box::new(pos!(100.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("TEST".to_string()),
        );

        let chain_params = OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            5,
            spos!(5.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.02),
            2,
            price_params,
            pos!(0.2),
        );

        // Create series with different expirations
        let series = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
        let series_params = OptionSeriesBuildParams::new(chain_params, series);

        // Build the option series
        OptionSeries::build_series(&series_params)
    }

    #[test]
    fn test_generator_optionseries_basic() {
        // Setup
        let n_steps = 5;
        let initial_series = create_test_option_series();
        let std_dev = pos!(0.2);
        let days = pos!(30.0);
        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: n_steps,
            init_step: Step {
                x: Xstep::new(Positive::ONE, TimeFrame::Day, ExpirationDate::Days(days)),
                y: Ystep::new(0, initial_series),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0), &TimeFrame::Day, &TimeFrame::Day),
                drift: dec!(0.0),
                volatility: std_dev,
            },
            walker,
        };

        // Execute
        let steps = generator_optionseries(&walk_params);

        // Verify
        assert!(!steps.is_empty(), "Steps should not be empty");
        assert_eq!(
            steps.len(),
            5,
            "Should start with just the initial step since we're mocking"
        );

        // The first step should be the initial step
        let first_step = &steps[0];
        assert_eq!(first_step.x.datetime().get_days().unwrap(), pos!(30.0));
        assert_eq!(*first_step.y.index(), 0);
    }

    #[test]
    fn test_generator_optionseries_empty_result() {
        struct TestWalker {}
        // Create a walk with empty y_steps to test early return
        let initial_series = create_test_option_series();
        let walker = Box::new(TestWalker {});

        impl WalkTypeAble<Positive, OptionSeries> for TestWalker {}

        let walk_params = WalkParams {
            size: 5,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos!(30.0)),
                ),
                y: Ystep::new(0, initial_series),
            },
            walk_type: WalkType::Brownian {
                dt: pos!(0.01),
                drift: dec!(0.0),
                volatility: pos!(0.2),
            },
            walker,
        };

        // Execute
        let steps = generator_optionseries(&walk_params);

        // Verify
        assert!(!steps.is_empty(), "Steps shouldn't be empty");
    }

    #[test]
    fn test_generator_optionseries_historical_empty_prices() {
        // Test with historical walk type but empty prices
        let initial_series = create_test_option_series();
        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 5,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos!(30.0)),
                ),
                y: Ystep::new(0, initial_series),
            },
            walk_type: WalkType::Historical {
                timeframe: TimeFrame::Day,
                prices: Vec::new(), // Empty prices
                symbol: None,
            },
            walker,
        };

        // Execute
        let steps = generator_optionseries(&walk_params);

        // Verify
        assert!(
            steps.is_empty(),
            "Steps should be empty when historical prices are empty"
        );
    }

    #[test]
    fn test_generator_optionseries_historical_insufficient_prices() {
        // Test with historical walk type but insufficient prices
        let initial_series = create_test_option_series();
        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 5,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos!(30.0)),
                ),
                y: Ystep::new(0, initial_series),
            },
            walk_type: WalkType::Historical {
                timeframe: TimeFrame::Day,
                prices: vec![pos!(100.0), pos!(101.0)], // Less than size
                symbol: None,
            },
            walker,
        };

        // Execute
        let steps = generator_optionseries(&walk_params);

        // Verify
        assert!(
            steps.is_empty(),
            "Steps should be empty when historical prices are insufficient"
        );
    }

    #[test]
    fn test_generator_optionseries_all_walk_types() {
        // This is more of an integration test checking that all walk types are handled
        let initial_series = create_test_option_series();
        let walker = Box::new(TestWalker::new());
        let volatility = pos!(0.2);

        // Define all walk types to test
        let walk_types = vec![
            WalkType::Brownian {
                dt: pos!(0.01),
                drift: dec!(0.0),
                volatility,
            },
            WalkType::GeometricBrownian {
                dt: pos!(0.01),
                drift: dec!(0.0),
                volatility,
            },
            WalkType::LogReturns {
                dt: pos!(0.01),
                expected_return: dec!(0.0),
                volatility,
                autocorrelation: Some(dec!(0.0)),
            },
            WalkType::MeanReverting {
                dt: pos!(0.01),
                volatility,
                speed: pos!(0.1),
                mean: pos!(100.0),
            },
            WalkType::JumpDiffusion {
                dt: pos!(0.01),
                drift: dec!(0.0),
                volatility,
                intensity: pos!(0.1),
                jump_mean: dec!(0.0),
                jump_volatility: pos!(0.1),
            },
            WalkType::Garch {
                dt: pos!(0.01),
                drift: dec!(0.0),
                volatility,
                alpha: pos!(0.1),
                beta: pos!(0.8),
            },
            WalkType::Heston {
                dt: pos!(0.01),
                drift: dec!(0.0),
                volatility,
                kappa: pos!(2.0),
                theta: pos!(0.04),
                xi: pos!(0.1),
                rho: dec!(-0.7),
            },
            WalkType::Custom {
                dt: pos!(0.01),
                drift: dec!(0.0),
                volatility,
                vov: pos!(0.1),
                vol_speed: pos!(0.1),
                vol_mean: pos!(0.2),
            },
        ];

        // Make sure each walk type is handled by checking that the function runs
        for walk_type in walk_types {
            let walk_params = WalkParams {
                size: 5,
                init_step: Step {
                    x: Xstep::new(
                        Positive::ONE,
                        TimeFrame::Day,
                        ExpirationDate::Days(pos!(30.0)),
                    ),
                    y: Ystep::new(0, initial_series.clone()),
                },
                walk_type,
                walker: walker.clone(),
            };

            // Function should run without panicking for all walk types
            let _ = generator_optionseries(&walk_params);
            // We're not checking the result as we're just verifying the function handles all types
        }
    }

    #[test]
    fn test_generator_optionseries_historical() {
        // Setup for testing historical walk type
        let initial_series = create_test_option_series();
        let walker = Box::new(TestWalker {});
        let historical_prices = vec![
            pos!(100.0),
            pos!(102.0),
            pos!(98.0),
            pos!(105.0),
            pos!(110.0),
            pos!(115.0),
            pos!(112.0),
            pos!(118.0),
            pos!(120.0),
            pos!(125.0),
        ];

        let walk_params = WalkParams {
            size: 5,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos!(30.0)),
                ),
                y: Ystep::new(0, initial_series),
            },
            walk_type: WalkType::Historical {
                timeframe: TimeFrame::Day,
                prices: historical_prices,
                symbol: None,
            },
            walker,
        };

        // Execute
        let steps = generator_optionseries(&walk_params);

        // Verify
        assert!(!steps.is_empty(), "Should have at least the initial step");
        assert_eq!(
            steps.len(),
            5,
            "Should have just the initial step with our mock"
        );
    }

    #[test]
    fn test_create_series_from_step() {
        // Test the create_series_from_step function directly
        let initial_series = create_test_option_series();
        let y_step = Ystep::new(0, initial_series);
        let new_price = pos!(105.0);
        let volatility = spos!(0.22);

        // Execute
        let result = create_series_from_step(&y_step, &new_price, volatility);

        // Verify
        assert!(result.is_ok(), "create_series_from_step should succeed");
        let new_series = result.unwrap();
        assert_eq!(
            new_series.underlying_price, new_price,
            "New series should have updated price"
        );

        // Verify the implied volatility was updated if we can access it
        if let Ok(params) = new_series.to_build_params() {
            let iv = params.chain_params.get_implied_volatility();
            assert_pos_relative_eq!(iv, volatility.unwrap(), pos!(0.01));
        }
    }

    #[test]
    fn test_assert_steps_length() {
        // Setup
        let n_steps = 3;
        let initial_series = create_test_option_series();
        let walker = Box::new(TestWalker {});

        let walk_params = WalkParams {
            size: n_steps,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos!(30.0)),
                ),
                y: Ystep::new(0, initial_series),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos!(0.01),
                drift: dec!(0.0),
                volatility: pos!(0.2),
            },
            walker,
        };

        // Execute
        let steps = generator_optionseries(&walk_params);

        // Verify
        assert!(
            steps.len() <= n_steps,
            "Steps length should not exceed the specified size"
        );
    }
}
