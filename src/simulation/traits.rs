use crate::Positive;
use crate::model::decimal::decimal_normal_sample;
use crate::simulation::{WalkParams, WalkType};
use crate::volatility::generate_ou_process;
use rust_decimal::{Decimal, MathematicalOps};
use std::error::Error;
use std::fmt::{Debug, Display};
use std::ops::AddAssign;

/// Trait for implementing various random walk models and stochastic processes.
///
/// This trait provides methods to generate different types of stochastic processes commonly
/// used in financial modeling, time series analysis, and simulation studies. Each method
/// implements a specific type of random walk based on the parameters provided.
///
/// The trait is generic over types `X` and `Y`, which represent the x-axis (typically time)
/// and y-axis (typically price or value) components respectively.
///
/// # Type Parameters
///
/// * `X` - The type for the x-axis values (typically time), must be `Copy`, convertible to `Positive`,
///   implement `AddAssign`, and implement `Display`.
/// * `Y` - The type for the y-axis values (typically price), must be `Copy`, convertible to `Positive`,
///   and implement `Display`.
///
/// # Methods
///
/// The trait provides methods for generating the following stochastic processes:
/// - Brownian motion (standard random walk)
/// - Geometric Brownian motion
/// - Log returns process with optional autocorrelation
/// - Mean reverting (Ornstein-Uhlenbeck) process
/// - Jump diffusion process
/// - GARCH (Generalized Autoregressive Conditional Heteroskedasticity)
/// - Heston stochastic volatility model
/// - Custom stochastic process with mean-reverting volatility
pub trait WalkTypeAble<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    /// Generates a Brownian motion (standard random walk) process.
    ///
    /// Brownian motion is a continuous-time stochastic process where changes
    /// are normally distributed with a drift term and volatility.
    ///
    /// # Parameters
    ///
    /// * `params` - Walk parameters including initial value, time step, drift, and volatility.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values representing
    ///   the generated Brownian motion path, or an error if parameters are invalid.
    fn brownian(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::Brownian {
                dt,
                drift,
                volatility,
            } => {
                let mut values = Vec::new();
                let mut current_value: Positive = params.ystep_as_positive();
                for _ in 0..params.size {
                    let random_step = decimal_normal_sample() * volatility * dt;
                    current_value += drift * dt + random_step;
                    values.push(current_value);
                }
                Ok(values)
            }
            _ => Err("Invalid walk type for Brownian motion".into()),
        }
    }

    /// Generates a Geometric Brownian motion process.
    ///
    /// Geometric Brownian motion is a continuous-time stochastic process where the logarithm of the
    /// randomly varying quantity follows Brownian motion. It's commonly used to model stock prices
    /// in the Black-Scholes options pricing model.
    ///
    /// # Parameters
    ///
    /// * `params` - Walk parameters including initial value, time step, drift, and volatility.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values representing
    ///   the generated Geometric Brownian motion path, or an error if parameters are invalid.
    fn geometric_brownian(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::GeometricBrownian {
                dt,
                drift,
                volatility,
            } => {
                let mut values = Vec::new();
                let mut current_value: Positive = params.ystep_as_positive();
                for _ in 0..params.size {
                    let random_step = decimal_normal_sample() * volatility * dt;
                    current_value *= (drift * dt + random_step).exp();
                    values.push(current_value);
                }
                Ok(values)
            }
            _ => Err("Invalid walk type for Geometric Brownian motion".into()),
        }
    }

    /// Generates a Log Returns process, potentially with autocorrelation.
    ///
    /// This process models returns (percentage changes) directly, rather than absolute values,
    /// and can include autocorrelation to capture the tendency of returns to be influenced
    /// by previous returns.
    ///
    /// # Parameters
    ///
    /// * `params` - Walk parameters including initial value, time step, expected return,
    ///   volatility, and optional autocorrelation coefficient.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values representing
    ///   the generated Log Returns path, or an error if parameters are invalid.
    fn log_returns(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::LogReturns {
                dt,
                expected_return,
                volatility,
                autocorrelation,
            } => {
                let mut values = Vec::new();
                let mut current_value: Positive = params.ystep_as_positive();
                let mut prev_log_return = Decimal::ZERO;
                for _ in 0..params.size {
                    let random_step = decimal_normal_sample() * volatility * dt;
                    let mut log_return = expected_return * dt + random_step;
                    if let Some(ac) = autocorrelation {
                        assert!(ac <= Decimal::ONE && ac >= -Decimal::ONE);
                        log_return += ac * prev_log_return;
                    }
                    current_value *= (log_return).exp();
                    values.push(current_value);
                    prev_log_return = log_return;
                }
                Ok(values)
            }
            _ => Err("Invalid walk type for Log Returns motion".into()),
        }
    }

    /// Generates a Mean Reverting (Ornstein-Uhlenbeck) process.
    ///
    /// The Ornstein-Uhlenbeck process models a value that tends to drift toward a long-term mean,
    /// with the strength of the reversion proportional to the distance from the mean.
    /// It's commonly used for interest rates, volatility, and other mean-reverting financial variables.
    ///
    /// # Parameters
    ///
    /// * `params` - Walk parameters including initial value, mean level, reversion speed,
    ///   volatility, and time step.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values representing
    ///   the generated Mean Reverting path, or an error if parameters are invalid.
    fn mean_reverting(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::MeanReverting {
                dt,
                volatility,
                speed,
                mean,
            } => Ok(generate_ou_process(
                params.ystep_as_positive(),
                mean,
                speed,
                volatility,
                dt,
                params.size,
            )),

            _ => Err("Invalid walk type for Mean Reverting motion".into()),
        }
    }

    /// Generates a Jump Diffusion process.
    ///
    /// Jump Diffusion combines continuous Brownian motion with discrete jumps that occur
    /// according to a Poisson process. This model is useful for capturing sudden market
    /// movements like crashes or spikes that standard Brownian motion cannot adequately model.
    ///
    /// # Parameters
    ///
    /// * `params` - Walk parameters including initial value, drift, volatility, jump intensity,
    ///   jump mean size, and jump volatility.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values representing
    ///   the generated Jump Diffusion path, or an error if parameters are invalid.
    fn jump_diffusion(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::JumpDiffusion {
                dt,
                drift,
                volatility,
                intensity,
                jump_mean,
                jump_volatility,
            } => {
                let mut values = Vec::new();
                let mut current_value: Positive = params.ystep_as_positive();
                for _ in 0..params.size {
                    let random_step = decimal_normal_sample() * volatility * dt;
                    current_value += drift * dt + random_step;
                    if decimal_normal_sample() < intensity.to_dec() {
                        let jump = jump_mean + jump_volatility * decimal_normal_sample();
                        current_value += jump;
                    }
                    values.push(current_value);
                }
                Ok(values)
            }
            _ => Err("Invalid walk type for Jump Diffusion motion".into()),
        }
    }

    /// Generates a GARCH (Generalized Autoregressive Conditional Heteroskedasticity) process.
    ///
    /// GARCH models time-varying volatility clustering, where periods of high volatility
    /// tend to be followed by high volatility, and low volatility by low volatility.
    ///
    /// # Parameters
    ///
    /// * `params` - Walk parameters for the GARCH process.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values representing
    ///   the generated GARCH path, or an error if parameters are invalid.
    ///
    /// # Note
    ///
    /// This implementation is currently a placeholder and returns an empty vector.
    fn garch(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::Garch {
                dt,
                drift,
                volatility,
                alpha,
                beta,
                omega,
            } => {
                // Validate GARCH parameters
                if alpha + beta >= Decimal::ONE {
                    return Err(
                        "GARCH parameters alpha + beta must be less than 1 for stationarity".into(),
                    );
                }

                let mut values = Vec::with_capacity(params.size);
                let mut current_value: Positive = params.ystep_as_positive();

                // Initialize variance using the provided initial volatility
                let mut variance = volatility * volatility;
                let mut previous_innovation_squared = Decimal::ZERO;

                values.push(current_value); // Add initial value

                for _ in 0..params.size - 1 {
                    // Update variance using GARCH(1,1) equation
                    variance = (omega.to_dec()
                        + alpha.to_dec() * previous_innovation_squared
                        + beta.to_dec() * variance)
                        .into();

                    // Generate random return with current variance
                    let innovation = decimal_normal_sample() * variance.sqrt();
                    let return_value = drift * dt.to_dec() + innovation;

                    // Update price
                    current_value *= (return_value).exp();
                    values.push(current_value);

                    // Store squared innovation for next iteration
                    previous_innovation_squared = innovation * innovation;
                }

                Ok(values)
            }
            _ => Err("Invalid walk type for GARCH model".into()),
        }
    }

    /// Generates a Heston stochastic volatility model.
    ///
    /// The Heston model extends Geometric Brownian Motion by allowing the volatility
    /// itself to be a stochastic process, following a mean-reverting square-root process.
    ///
    /// # Parameters
    ///
    /// * `params` - Walk parameters for the Heston process.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values representing
    ///   the generated Heston model path, or an error if parameters are invalid.
    ///
    /// # Note
    ///
    /// This implementation is currently a placeholder and returns an empty vector.
    /// Generates a Heston stochastic volatility model.
    ///
    /// The Heston model extends Geometric Brownian Motion by allowing the volatility
    /// itself to be a stochastic process, following a mean-reverting square-root process.
    ///
    /// # Parameters
    ///
    /// * `params` - Walk parameters for the Heston process, including:
    ///   - `dt`: Time step
    ///   - `drift`: Drift coefficient for the price process
    ///   - `v0`: Initial variance
    ///   - `kappa`: Mean reversion speed for variance
    ///   - `theta`: Long-term variance mean level
    ///   - `xi`: Volatility of variance
    ///   - `rho`: Correlation between price and variance Brownian motions
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values representing
    ///   the generated Heston model path, or an error if parameters are invalid.
    ///
    /// # Notes
    ///
    /// The Heston model is described by the following SDEs:
    /// dS_t = μS_t dt + √v_t S_t dW^1_t
    /// dv_t = κ(θ - v_t) dt + ξ√v_t dW^2_t
    /// with dW^1_t dW^2_t = ρ dt
    fn heston(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::Heston {
                dt,
                drift,
                volatility,
                kappa,
                theta,
                xi,
                rho,
            } => {
                // Validate parameters
                if rho < -Decimal::ONE || rho > Decimal::ONE {
                    return Err("Correlation rho must be between -1 and 1".into());
                }

                let mut values = Vec::with_capacity(params.size);
                let mut price: Positive = params.ystep_as_positive();

                // Initial variance is the square of initial volatility
                let mut variance = volatility.to_dec() * volatility.to_dec();

                values.push(price); // Add initial value

                for _ in 0..params.size - 1 {
                    // Generate correlated random numbers
                    let z1 = decimal_normal_sample();
                    let z2 = rho * z1
                        + (Decimal::ONE - rho * rho).sqrt().unwrap() * decimal_normal_sample();

                    // Ensure variance stays positive (modified Euler scheme with truncation)
                    let variance_new = (variance
                        + kappa.to_dec() * (theta.to_dec() - variance) * dt.to_dec()
                        + xi.to_dec()
                            * variance.sqrt().unwrap()
                            * z2
                            * dt.to_dec().sqrt().unwrap())
                    .max(Decimal::ZERO);

                    // Update price using the average variance over the step
                    let avg_variance = (variance + variance_new) / Decimal::TWO;
                    let price_change = drift * dt.to_dec()
                        + avg_variance.sqrt().unwrap() * z1 * dt.to_dec().sqrt().unwrap();

                    price *= (price_change).exp();
                    variance = variance_new;

                    values.push(price);
                }

                Ok(values)
            }
            _ => Err("Invalid walk type for Heston model".into()),
        }
    }

    /// Generates a custom stochastic process with mean-reverting volatility.
    ///
    /// This implements a process where the underlying value follows Brownian motion,
    /// but with volatility that follows an Ornstein-Uhlenbeck (mean-reverting) process.
    /// This allows for modeling more complex dynamics than standard models.
    ///
    /// # Parameters
    ///
    /// * `params` - Walk parameters including drift, initial volatility, volatility of volatility (vov),
    ///   volatility mean reversion speed, volatility mean level, and time step.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values representing
    ///   the generated custom process path, or an error if parameters are invalid.
    fn custom(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::Custom {
                dt,
                drift,
                volatility,
                vov,
                vol_speed,
                vol_mean,
            } => {
                let volatilities =
                    generate_ou_process(volatility, vol_mean, vol_speed, vov, dt, params.size);
                let mut values = Vec::new();

                let mut current_value: Positive = params.ystep_as_positive();

                // Use iterator instead of index-based loop
                for &vol in volatilities.iter().take(params.size) {
                    let random_step = decimal_normal_sample() * vol * dt;
                    current_value += drift * dt + random_step;
                    values.push(current_value);
                }
                Ok(values)
            }
            _ => Err("Invalid walk type for Custom motion".into()),
        }
    }

    /// Generates a historical walk based on the given parameters.
    ///
    /// This function processes the historical walk by extracting a specified number of elements
    /// from the provided price data (`prices`) based on the `size` defined in `params`.
    ///
    /// # Parameters
    ///
    /// * `self`: Reference to the instance of the object.
    /// * `params`: A reference to `WalkParams<X, Y>` containing the configuration details for the walk.
    ///   - Expected to have a `walk_type` of `WalkType::Historical` with associated timeframe and price data.
    ///   - `params.size` determines the number of historical prices to include in the result.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Positive>)`: A vector containing the first `params.size` elements from the given price data (`prices`),
    ///   if there are at least `params.size` elements available.
    /// * `Err(Box<dyn Error>)`: If the `walk_type` is not `WalkType::Historical` or if the provided price data
    ///   does not contain enough elements to fulfill the requested size (`params.size`).
    ///
    /// # Errors
    ///
    /// * Returns an error if:
    ///     - The `walk_type` in `params` is not `WalkType::Historical`.
    ///     - The `prices` do not contain at least `params.size` elements.
    ///
    fn historical(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match &params.walk_type {
            WalkType::Historical {
                timeframe: _timeframe,
                prices,
            } => {
                if prices.len() >= params.size {
                    Ok(prices[0..params.size].to_vec())
                } else {
                    Err("Historical prices are not enough to generate the walk".into())
                }
            }
            _ => Err("Invalid walk type for Custom motion".into()),
        }
    }
}

impl<X, Y> Debug for Box<dyn WalkTypeAble<X, Y>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WalkTypeAble")
    }
}

impl<X, Y> Clone for Box<dyn WalkTypeAble<X, Y>> {
    fn clone(&self) -> Self {
        panic!("Box<dyn WalkTypeAble<X, Y>> cannot be cloned. Use clone_box() instead.")
    }
}

#[cfg(test)]
mod tests_walk_type_able {
    use super::*;
    use crate::ExpirationDate;
    use crate::Positive;
    use crate::pos;
    use crate::simulation::model::WalkType;
    use crate::simulation::params::WalkParams;
    use crate::simulation::steps::Step;
    use crate::simulation::traits::WalkTypeAble;
    use crate::utils::TimeFrame;
    use rust_decimal::Decimal;
    use std::error::Error;
    use std::fmt::Display;
    use std::ops::AddAssign;

    #[derive(Debug)]
    struct TestWalker {}

    impl<X, Y> WalkTypeAble<X, Y> for TestWalker
    where
        X: Copy + Into<Positive> + AddAssign + Display,
        Y: Copy + Into<Positive> + Display,
    {
    }

    fn create_test_params<X, Y>(
        size: usize,
        x_value: X,
        y_value: Y,
        walk_type: WalkType,
    ) -> WalkParams<X, Y>
    where
        X: Copy + Into<Positive> + AddAssign + Display,
        Y: Copy + Into<Positive> + Display,
    {
        let init_step = Step::new(
            x_value,
            TimeFrame::Day,
            ExpirationDate::Days(pos!(30.0)),
            y_value,
        );

        WalkParams {
            size,
            init_step,
            walk_type,
            walker: Box::new(TestWalker {}),
        }
    }

    #[test]
    fn test_brownian_walk() -> Result<(), Box<dyn Error>> {
        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let walker = TestWalker {};
        let result = walker.brownian(&params)?;

        assert_eq!(result.len(), 5);
        Ok(())
    }

    #[test]
    fn test_geometric_brownian_walk() -> Result<(), Box<dyn Error>> {
        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::GeometricBrownian {
                dt: pos!(1.0),
                drift: Decimal::new(5, 2), // 0.05
                volatility: pos!(0.2),
            },
        );

        let walker = TestWalker {};
        let result = walker.geometric_brownian(&params)?;

        assert_eq!(result.len(), 5);
        Ok(())
    }

    #[test]
    fn test_log_returns_walk() -> Result<(), Box<dyn Error>> {
        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::LogReturns {
                dt: pos!(1.0),
                expected_return: Decimal::new(5, 2), // 0.05
                volatility: pos!(0.2),
                autocorrelation: None,
            },
        );

        let walker = TestWalker {};
        let result = walker.log_returns(&params)?;

        assert_eq!(result.len(), 5);
        Ok(())
    }

    #[test]
    fn test_mean_reverting_walk() -> Result<(), Box<dyn Error>> {
        let mean_value = pos!(150.0);
        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::MeanReverting {
                dt: pos!(1.0),
                volatility: pos!(0.2),
                speed: pos!(0.1),
                mean: mean_value,
            },
        );

        let walker = TestWalker {};
        let result = walker.mean_reverting(&params)?;

        assert_eq!(result.len(), 5);
        assert_eq!(result[0], pos!(100.0));

        let diff1 = (mean_value.to_dec_ref() - result[1].to_dec_ref()).abs();
        let diff4 = (mean_value.to_dec_ref() - result[4].to_dec_ref()).abs();
        assert!(diff4 < diff1, "Los valores deberían acercarse a la media");

        Ok(())
    }

    #[test]
    fn test_jump_diffusion_walk() -> Result<(), Box<dyn Error>> {
        let params = create_test_params(
            6,
            10.0,
            100.0,
            WalkType::JumpDiffusion {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
                intensity: pos!(0.1),
                jump_mean: Decimal::ZERO,
                jump_volatility: pos!(1.0),
            },
        );

        let walker = TestWalker {};
        let result = walker.jump_diffusion(&params)?;

        assert_eq!(result.len(), 6);
        Ok(())
    }

    #[test]
    fn test_garch_walk() -> Result<(), Box<dyn Error>> {
        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::Garch {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
                alpha: pos!(0.1),
                beta: pos!(0.8),
                omega: pos!(0.05),
            },
        );

        let walker = TestWalker {};
        let result = walker.garch(&params)?;

        assert_eq!(result.len(), 5);
        Ok(())
    }

    #[test]
    fn test_heston_walk() -> Result<(), Box<dyn Error>> {
        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::Heston {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
                kappa: pos!(2.0),
                theta: pos!(0.04),
                xi: pos!(0.4),
                rho: Decimal::new(-5, 1), // -0.5
            },
        );

        let walker = TestWalker {};
        let result = walker.heston(&params)?;

        assert_eq!(result.len(), 5);
        Ok(())
    }

    #[test]
    fn test_custom_walk() -> Result<(), Box<dyn Error>> {
        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::Custom {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
                vov: pos!(0.4),
                vol_speed: pos!(1.0),
                vol_mean: pos!(0.2),
            },
        );

        let walker = TestWalker {};
        let result = walker.custom(&params)?;

        assert_eq!(result.len(), 5);
        Ok(())
    }

    #[test]
    fn test_with_different_types() -> Result<(), Box<dyn Error>> {
        #[derive(Debug, Copy, Clone, PartialEq)]
        struct XType(f64);

        impl From<XType> for Positive {
            fn from(val: XType) -> Self {
                pos!(val.0)
            }
        }

        impl AddAssign for XType {
            fn add_assign(&mut self, other: Self) {
                self.0 += other.0;
            }
        }

        impl Display for XType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        #[derive(Debug, Copy, Clone, PartialEq)]
        struct YType(f64);

        impl From<YType> for Positive {
            fn from(val: YType) -> Self {
                pos!(val.0)
            }
        }

        impl Display for YType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        let params = create_test_params(
            5,
            XType(10.0),
            YType(100.0),
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let walker = TestWalker {};
        let result = walker.brownian(&params)?;

        assert_eq!(result.len(), 5);
        Ok(())
    }

    #[test]
    fn test_error_handling() {
        struct ErrorWalker {}

        impl<X, Y> WalkTypeAble<X, Y> for ErrorWalker
        where
            X: Copy + Into<Positive> + AddAssign + Display,
            Y: Copy + Into<Positive> + Display,
        {
            fn brownian(
                &self,
                _params: &WalkParams<X, Y>,
            ) -> Result<Vec<Positive>, Box<dyn Error>> {
                Err("Error simulado para prueba".into())
            }

            fn geometric_brownian(
                &self,
                params: &WalkParams<X, Y>,
            ) -> Result<Vec<Positive>, Box<dyn Error>> {
                self.brownian(params)
            }

            fn log_returns(
                &self,
                params: &WalkParams<X, Y>,
            ) -> Result<Vec<Positive>, Box<dyn Error>> {
                self.brownian(params)
            }

            fn mean_reverting(
                &self,
                params: &WalkParams<X, Y>,
            ) -> Result<Vec<Positive>, Box<dyn Error>> {
                self.brownian(params)
            }

            fn jump_diffusion(
                &self,
                params: &WalkParams<X, Y>,
            ) -> Result<Vec<Positive>, Box<dyn Error>> {
                self.brownian(params)
            }

            fn garch(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
                self.brownian(params)
            }

            fn heston(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
                self.brownian(params)
            }

            fn custom(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
                self.brownian(params)
            }
        }

        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let error_walker = ErrorWalker {};
        assert!(error_walker.brownian(&params).is_err());
    }
}
