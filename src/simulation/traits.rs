use crate::backtesting::results::SimulationStatsResult;
use crate::error::SimulationError;
use crate::model::decimal::decimal_normal_sample;
use crate::simulation::simulator::Simulator;
use crate::simulation::{ExitPolicy, WalkParams, WalkType};
use crate::volatility::generate_ou_process;
use num_traits::{FromPrimitive, ToPrimitive};
use positive::Positive;
use rust_decimal::{Decimal, MathematicalOps};
use std::convert::TryInto;
use std::fmt::{Debug, Display};
use std::ops::AddAssign;

/// Object-safe helper trait that exposes a `Clone`-compatible operation for
/// [`WalkTypeAble`] trait objects.
///
/// `Clone::clone` requires `Self: Sized`, so it cannot be invoked through
/// `dyn WalkTypeAble<X, Y>` directly. `WalkTypeAbleClone::clone_box` returns an
/// owned `Box<dyn WalkTypeAble<X, Y>>` instead, which lets the generic
/// `Clone for Box<dyn WalkTypeAble<X, Y>>` implementation work uniformly for
/// any concrete walker that is `Clone + 'static`.
///
/// You should never implement this trait by hand: a blanket impl covers every
/// `T: WalkTypeAble<X, Y> + Clone + 'static`. Just `#[derive(Clone)]` on your
/// concrete walker and the trait object gains `Clone` for free.
pub trait WalkTypeAbleClone<X, Y>
where
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
{
    /// Returns a boxed clone of this walker as a trait object.
    #[must_use]
    fn clone_box(&self) -> Box<dyn WalkTypeAble<X, Y>>;
}

impl<T, X, Y> WalkTypeAbleClone<X, Y> for T
where
    T: 'static + WalkTypeAble<X, Y> + Clone,
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
{
    #[inline]
    fn clone_box(&self) -> Box<dyn WalkTypeAble<X, Y>> {
        Box::new(self.clone())
    }
}

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
///
/// # Object safety and cloning
///
/// `WalkTypeAble` is object-safe: you can hold it behind `Box<dyn WalkTypeAble<X, Y>>`
/// (e.g., inside [`WalkParams`]). The super-trait [`WalkTypeAbleClone`] provides an
/// object-safe `clone_box` hook that forwards to `Clone` on the concrete walker, which
/// is how `Box<dyn WalkTypeAble<X, Y>>` implements `Clone`. Any concrete walker you
/// want to store behind a trait object must therefore be `Clone + 'static` — in
/// practice that means adding `#[derive(Clone)]` to your walker struct.
///
/// The blanket implementation of `WalkTypeAbleClone` is automatic; you never
/// implement `clone_box` by hand.
pub trait WalkTypeAble<X, Y>: WalkTypeAbleClone<X, Y>
where
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
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
    /// * `Result<Vec<Positive>, SimulationError>` - A vector of positive values representing
    ///   the generated Brownian motion path, or an error if parameters are invalid.
    fn brownian(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        match params.walk_type {
            WalkType::Brownian {
                dt,
                drift,
                volatility,
            } => {
                let mut values = Vec::with_capacity(params.size + 1);
                let start: Positive = params.ystep_as_positive()?;
                values.push(start);
                let mut x: Decimal = start.to_dec();
                let sigma_abs = (volatility * start).to_dec();
                let sqrt_dt = dt.to_f64().sqrt();
                let sqrt_dt_dec = Decimal::from_f64(sqrt_dt).unwrap_or(Decimal::ZERO);

                for _ in 1..params.size {
                    let z = decimal_normal_sample();
                    let diffusion = sigma_abs * sqrt_dt_dec * z;
                    let drift_term = drift * dt;
                    x += drift_term + diffusion;
                    values.push(
                        Positive::new_decimal(x.max(Decimal::ZERO)).unwrap_or(Positive::ZERO),
                    );
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
    /// * `Result<Vec<Positive>, SimulationError>` - A vector of positive values representing
    ///   the generated Geometric Brownian motion path, or an error if parameters are invalid.
    fn geometric_brownian(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, SimulationError> {
        match params.walk_type {
            WalkType::GeometricBrownian {
                dt,
                drift,
                volatility,
            } => {
                let mut values = Vec::with_capacity(params.size);
                let mut current_value: Positive = params.ystep_as_positive()?;
                values.push(current_value);
                let sqrt_dt = dt.sqrt();

                for _ in 1..params.size {
                    // σ * √dt * Z
                    let diffusion = decimal_normal_sample() * volatility * sqrt_dt;
                    // μ * dt
                    let drift_term = (drift * dt) + diffusion;
                    current_value *= Decimal::exp(&drift_term);
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
    /// * `Result<Vec<Positive>, SimulationError>` - A vector of positive values representing
    ///   the generated Log Returns path, or an error if parameters are invalid.
    fn log_returns(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        match params.walk_type {
            WalkType::LogReturns {
                dt,
                expected_return,
                volatility,
                autocorrelation,
            } => {
                let mut values = Vec::with_capacity(params.size + 1);
                let mut price: Positive = params.ystep_as_positive()?;
                values.push(price);

                let sqrt_dt = dt.to_f64().sqrt();
                let sqrt_dt_dec = Decimal::from_f64(sqrt_dt).unwrap_or(Decimal::ZERO);
                let mut prev_log_ret = Decimal::ZERO;

                for _ in 1..params.size {
                    let z = decimal_normal_sample();
                    let diffusion = z * volatility * sqrt_dt_dec;
                    let mut log_ret = (expected_return * dt) + diffusion;

                    if let Some(ac) = autocorrelation {
                        if !(-Decimal::ONE..=Decimal::ONE).contains(&ac) {
                            return Err(format!(
                                "LogReturns: autocorrelation {ac} must lie in [-1, 1]"
                            )
                            .into());
                        }
                        log_ret += ac * prev_log_ret;
                    }

                    // actualizar precio
                    price *= log_ret.exp();
                    values.push(price);

                    prev_log_ret = log_ret;
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
    /// * `Result<Vec<Positive>, SimulationError>` - A vector of positive values representing
    ///   the generated Mean Reverting path, or an error if parameters are invalid.
    fn mean_reverting(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        match params.walk_type {
            WalkType::MeanReverting {
                dt,
                volatility,
                speed,
                mean, // mean level or initial value
            } => {
                let sigma_abs = volatility * mean;
                Ok(generate_ou_process(
                    params.ystep_as_positive()?,
                    mean,
                    speed,
                    sigma_abs,
                    dt,
                    params.size,
                ))
            }

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
    /// * `Result<Vec<Positive>, SimulationError>` - A vector of positive values representing
    ///   the generated Jump Diffusion path, or an error if parameters are invalid.
    fn jump_diffusion(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        match params.walk_type {
            WalkType::JumpDiffusion {
                dt,
                drift,
                volatility,
                intensity,
                jump_mean,
                jump_volatility,
            } => {
                let mut values = Vec::with_capacity(params.size + 1);
                let mut x: Decimal = params.ystep_as_positive()?.to_dec();
                values.push(Positive::new_decimal(x).unwrap_or(Positive::ZERO));

                let sqrt_dt = dt.sqrt();
                let lambda_dt = intensity * dt;

                for _ in 1..params.size {
                    let z = decimal_normal_sample();
                    let sigma_abs = volatility.to_dec() * x;
                    let diffusion = sigma_abs * sqrt_dt.to_dec() * z;

                    let drift_term = drift * dt;
                    let jump = if decimal_normal_sample() < lambda_dt.to_dec() {
                        // Bernoulli(λdt)
                        jump_mean + decimal_normal_sample() * jump_volatility
                    } else {
                        Decimal::ZERO
                    };

                    x += drift_term + diffusion + jump;
                    x = x.max(Decimal::ZERO);
                    values.push(Positive::new_decimal(x).unwrap_or(Positive::ZERO));
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
    /// * `Result<Vec<Positive>, SimulationError>` - A vector of positive values representing
    ///   the generated GARCH path, or an error if parameters are invalid.
    ///
    /// # Note
    ///
    /// This implementation is currently a placeholder and returns an empty vector.
    fn garch(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        match params.walk_type {
            WalkType::Garch {
                dt,
                drift,
                volatility,
                alpha,
                beta,
            } => {
                if alpha + beta >= Decimal::ONE {
                    return Err("alpha + beta must be < 1 for stationarity".into());
                }

                let mut path = Vec::with_capacity(params.size + 1);
                let mut price = params.ystep_as_positive()?.to_dec();
                path.push(Positive::new_decimal(price).unwrap_or(Positive::ZERO));

                // --- initial conditional variance (annualised) ---
                let mut var = volatility * volatility; // σ₀²
                let mut prev_eps2 = Decimal::ZERO;
                let omega = volatility.powu(2) * (Decimal::ONE - alpha - beta); // 0.002

                // pre-compute √dt
                let sqrt_dt = dt.to_f64().sqrt();
                let sqrt_dt_dec = Decimal::from_f64(sqrt_dt).unwrap_or(Decimal::ZERO);

                for _ in 1..params.size {
                    // 1) update variance
                    var = omega + alpha * prev_eps2 + beta * var;

                    // 2) shock with the right scale σ√dt·Z
                    let z = decimal_normal_sample();
                    let eps = z * var.sqrt() * sqrt_dt_dec; // εₜ

                    // 3) drift  (use μ dt, or μ dt − ½σ² dt if μ is arithmetic)
                    let ret = drift * dt + eps;

                    // 4) price update
                    price *= (ret).exp();
                    path.push(Positive::new_decimal(price).unwrap_or(Positive::ZERO));

                    // 5) store ε²
                    prev_eps2 = eps.powu(2); // εₜ²
                }
                Ok(path)
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
    /// * `Result<Vec<Positive>, SimulationError>` - A vector of positive values representing
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
    /// * `Result<Vec<Positive>, SimulationError>` - A vector of positive values representing
    ///   the generated Heston model path, or an error if parameters are invalid.
    ///
    /// # Notes
    ///
    /// The Heston model is described by the following SDEs:
    /// dS_t = μS_t dt + √v_t S_t dW^1_t
    /// dv_t = κ(θ - v_t) dt + ξ√v_t dW^2_t
    /// with dW^1_t dW^2_t = ρ dt
    fn heston(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
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
                let mut price: Positive = params.ystep_as_positive()?;

                // Initial variance is the square of initial volatility
                let mut variance = volatility.to_dec() * volatility.to_dec();

                values.push(price); // Add initial value

                let dt_sqrt = dt
                    .to_dec()
                    .sqrt()
                    .ok_or_else(|| SimulationError::other("Heston: sqrt(dt) failed (overflow)"))?;
                // sqrt(1 - rho^2) depends only on `rho`, hoist out of the
                // hot loop so we don't recompute it per step.
                let one_minus_rho_sq_sqrt = (Decimal::ONE - rho * rho).sqrt().ok_or_else(|| {
                    SimulationError::other(
                        "Heston: sqrt(1 - rho^2) failed (rho out of range or overflow)",
                    )
                })?;
                for _ in 0..params.size - 1 {
                    // Generate correlated random numbers
                    let z1 = decimal_normal_sample();
                    let z2 = rho * z1 + one_minus_rho_sq_sqrt * decimal_normal_sample();

                    // Ensure variance stays positive (modified Euler scheme with truncation)
                    let variance_sqrt = variance.sqrt().ok_or_else(|| {
                        SimulationError::other("Heston: sqrt(variance) failed (overflow)")
                    })?;
                    let variance_new = (variance
                        + kappa.to_dec() * (theta.to_dec() - variance) * dt.to_dec()
                        + xi.to_dec() * variance_sqrt * z2 * dt_sqrt)
                        .max(Decimal::ZERO);

                    // Update price using the average variance over the step
                    let avg_variance = (variance + variance_new) / Decimal::TWO;
                    let avg_variance_sqrt = avg_variance.sqrt().ok_or_else(|| {
                        SimulationError::other("Heston: sqrt(avg_variance) failed (overflow)")
                    })?;
                    let price_change = drift * dt.to_dec() + avg_variance_sqrt * z1 * dt_sqrt;

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
    /// * `Result<Vec<Positive>, SimulationError>` - A vector of positive values representing
    ///   the generated custom process path, or an error if parameters are invalid.
    fn custom(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        match params.walk_type {
            WalkType::Custom {
                dt,
                drift,
                volatility,
                vov,
                vol_speed,
                vol_mean,
            } => {
                let vols =
                    generate_ou_process(volatility, vol_mean, vol_speed, vov, dt, params.size);

                let sqrt_dt = dt.sqrt();
                let mut price = params.ystep_as_positive()?.to_dec();
                let mut path = Vec::with_capacity(params.size + 1);
                path.push(Positive::new_decimal(price).unwrap_or(Positive::ZERO));

                for &vol in vols.iter().take(params.size - 1) {
                    let z = decimal_normal_sample();
                    let sigma_abs = vol.to_dec() * price;
                    let random_step = z * sigma_abs * sqrt_dt.to_dec();

                    price += drift * dt + random_step;
                    path.push(
                        Positive::new_decimal(price.max(Decimal::ZERO)).unwrap_or(Positive::ZERO),
                    );
                }

                Ok(path)
            }
            _ => Err("Invalid walk type for Custom motion".into()),
        }
    }

    /// Generates a Telegraph process (two-state regime switching model).
    ///
    /// The Telegraph process alternates between two states (+1 and -1) with specified transition rates,
    /// affecting the volatility of the price path. This model captures regime-switching behavior
    /// in financial markets where volatility can suddenly change between high and low regimes.
    ///
    /// # Parameters
    ///
    /// * `params` - Walk parameters including initial value, drift, base volatility, transition rates,
    ///   and optional volatility multipliers for each state.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Positive>, SimulationError>` - A vector of positive values representing
    ///   the generated Telegraph process path, or an error if parameters are invalid.
    fn telegraph(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        match params.walk_type {
            WalkType::Telegraph {
                dt,
                drift,
                volatility,
                lambda_up,
                lambda_down,
                vol_multiplier_up,
                vol_multiplier_down,
            } => {
                let mut values = Vec::with_capacity(params.size);
                let mut price = params.ystep_as_positive()?.to_dec();
                values.push(Positive::new_decimal(price).unwrap_or(Positive::ZERO));

                // Initialize telegraph state randomly
                let mut state: i8 = if decimal_normal_sample().to_f64().unwrap_or(0.0) < 0.0 {
                    1
                } else {
                    -1
                };

                let sqrt_dt = dt.sqrt();
                let vol_mult_up = vol_multiplier_up.unwrap_or(Positive::ONE);
                let vol_mult_down = vol_multiplier_down.unwrap_or(Positive::ONE);

                for _ in 1..params.size {
                    // Calculate transition probabilities
                    let lambda = if state == 1 {
                        lambda_down.to_dec()
                    } else {
                        lambda_up.to_dec()
                    };

                    let transition_prob = Decimal::ONE - (-lambda * dt.to_dec()).exp();

                    // Check for state transition using uniform random sample
                    let uniform_sample =
                        (decimal_normal_sample().abs() + Decimal::ONE) / Decimal::TWO; // Convert normal to uniform [0,1]
                    if uniform_sample < transition_prob {
                        state *= -1;
                    }

                    // Apply volatility multiplier based on current state
                    let current_vol = if state == 1 {
                        volatility * vol_mult_up
                    } else {
                        volatility * vol_mult_down
                    };

                    // Generate price change
                    let z = decimal_normal_sample();
                    let diffusion = current_vol.to_dec() * sqrt_dt.to_dec() * z;
                    let drift_term = drift * dt.to_dec();

                    // Update price using geometric Brownian motion with regime-dependent volatility
                    let price_change = drift_term + diffusion;
                    price *= price_change.exp();

                    values.push(Positive::new_decimal(price).unwrap_or(Positive::ZERO));
                }

                Ok(values)
            }
            _ => Err("Invalid walk type for Telegraph process".into()),
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
    fn historical(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        match &params.walk_type {
            WalkType::Historical {
                timeframe: _timeframe,
                prices,
                symbol: _symbol,
            } => {
                if prices.len() >= params.size {
                    Ok(prices[0..params.size].to_vec())
                } else {
                    Err("Historical prices are not enough to generate the walk".into())
                }
            }
            _ => Err("Invalid walk type for Historical motion".into()),
        }
    }
}

impl<X, Y> Debug for Box<dyn WalkTypeAble<X, Y>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WalkTypeAble")
    }
}

impl<X, Y> Clone for Box<dyn WalkTypeAble<X, Y>>
where
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
{
    fn clone(&self) -> Self {
        // Delegate to the object-safe `clone_box` hook on `WalkTypeAbleClone`.
        // The blanket impl for `T: WalkTypeAble<X, Y> + Clone + 'static`
        // forwards to `Clone::clone` on the concrete walker.
        self.clone_box()
    }
}

/// Trait for simulating trading strategies across multiple price paths.
///
/// This trait enables strategies to be tested against various market scenarios
/// by running them through multiple simulated price paths (random walks) and
/// evaluating their performance based on defined exit policies.
///
/// # Type Parameters
///
/// * `X` - The type representing time steps in the simulation
/// * `Y` - The type representing price values in the simulation
///
/// # Examples
///
/// ```ignore
/// use optionstratlib::simulation::{Simulate, ExitPolicy};
/// use rust_decimal_macros::dec;
///
/// let strategy = ShortPut::new(/* ... */);
/// let simulator = Simulator::new(/* ... */);
/// let exit_policy = ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0));
///
/// let results = strategy.simulate(&simulator, exit_policy)?;
/// ```
pub trait Simulate<X, Y>
where
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
{
    /// Simulates the strategy across multiple price paths.
    ///
    /// Evaluates the strategy's performance by running it through each random walk
    /// in the simulator, checking exit conditions at each step, and calculating
    /// final P&L based on either exit triggers or expiration.
    ///
    /// # Parameters
    ///
    /// * `sim` - The simulator containing multiple random walks to test against
    /// * `exit` - The exit policy defining when to close positions
    ///
    /// # Returns
    ///
    /// A `SimulationStats` struct containing:
    /// - Individual `SimulationResult` for each run (with P&L, exit reason, holding period, etc.)
    /// - Aggregate statistics (average P&L, win rate, std deviation, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Option pricing calculations fail
    /// - P&L calculations encounter errors
    /// - Invalid strategy parameters are detected
    fn simulate(
        &self,
        sim: &Simulator<X, Y>,
        exit: ExitPolicy,
    ) -> Result<SimulationStatsResult, SimulationError>;
}

#[cfg(test)]
mod tests_walk_type_able {
    use super::*;
    use crate::ExpirationDate;

    use crate::simulation::model::WalkType;
    use crate::simulation::params::WalkParams;
    use crate::simulation::steps::Step;
    use crate::simulation::traits::WalkTypeAble;
    use crate::utils::TimeFrame;
    use positive::pos_or_panic;
    use rust_decimal::Decimal;
    use std::error::Error;
    use std::fmt::Display;
    use std::ops::AddAssign;

    #[derive(Debug, Clone)]
    struct TestWalker {}

    impl<X, Y> WalkTypeAble<X, Y> for TestWalker
    where
        X: Copy + TryInto<Positive> + AddAssign + Display,
        Y: Copy + TryInto<Positive> + Display,
    {
    }

    fn create_test_params<X, Y>(
        size: usize,
        x_value: X,
        y_value: Y,
        walk_type: WalkType,
    ) -> WalkParams<X, Y>
    where
        X: Copy + TryInto<Positive> + AddAssign + Display,
        Y: Copy + TryInto<Positive> + Display,
    {
        let init_step = Step::new(
            x_value,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
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
                dt: Positive::ONE,
                drift: Decimal::ZERO,
                volatility: pos_or_panic!(0.2),
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
                dt: Positive::ONE,
                drift: Decimal::new(5, 2), // 0.05
                volatility: pos_or_panic!(0.2),
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
                dt: Positive::ONE,
                expected_return: Decimal::new(5, 2), // 0.05
                volatility: pos_or_panic!(0.2),
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
        let mean_value = pos_or_panic!(150.0);
        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::MeanReverting {
                dt: Positive::ONE,
                volatility: pos_or_panic!(0.2),
                speed: pos_or_panic!(0.1),
                mean: mean_value,
            },
        );

        let walker = TestWalker {};
        let result = walker.mean_reverting(&params)?;

        assert_eq!(result.len(), 5);
        assert_eq!(result[0], Positive::HUNDRED);
        Ok(())
    }

    #[test]
    fn test_jump_diffusion_walk() -> Result<(), Box<dyn Error>> {
        let params = create_test_params(
            6,
            10.0,
            100.0,
            WalkType::JumpDiffusion {
                dt: Positive::ONE,
                drift: Decimal::ZERO,
                volatility: pos_or_panic!(0.2),
                intensity: pos_or_panic!(0.1),
                jump_mean: Decimal::ZERO,
                jump_volatility: Positive::ONE,
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
                dt: Positive::ONE,
                drift: Decimal::ZERO,
                volatility: pos_or_panic!(0.2),
                alpha: pos_or_panic!(0.1),
                beta: pos_or_panic!(0.8),
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
                dt: Positive::ONE,
                drift: Decimal::ZERO,
                volatility: pos_or_panic!(0.2),
                kappa: Positive::TWO,
                theta: pos_or_panic!(0.04),
                xi: pos_or_panic!(0.4),
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
                dt: Positive::ONE,
                drift: Decimal::ZERO,
                volatility: pos_or_panic!(0.2),
                vov: pos_or_panic!(0.4),
                vol_speed: Positive::ONE,
                vol_mean: pos_or_panic!(0.2),
            },
        );

        let walker = TestWalker {};
        let result = walker.custom(&params)?;

        assert_eq!(result.len(), 5);
        Ok(())
    }

    #[test]
    fn test_telegraph_walk() -> Result<(), Box<dyn Error>> {
        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::Telegraph {
                dt: Positive::ONE,
                drift: Decimal::new(5, 2), // 0.05
                volatility: pos_or_panic!(0.2),
                lambda_up: pos_or_panic!(0.5),
                lambda_down: pos_or_panic!(0.3),
                vol_multiplier_up: Some(pos_or_panic!(1.5)),
                vol_multiplier_down: Some(pos_or_panic!(0.8)),
            },
        );

        let walker = TestWalker {};
        let result = walker.telegraph(&params)?;

        assert_eq!(result.len(), 5);
        assert_eq!(result[0], Positive::HUNDRED); // Initial value should be preserved
        Ok(())
    }

    #[test]
    fn test_with_different_types() -> Result<(), Box<dyn Error>> {
        #[derive(Debug, Copy, Clone, PartialEq)]
        struct XType(f64);

        impl From<XType> for Positive {
            fn from(val: XType) -> Self {
                pos_or_panic!(val.0)
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
                pos_or_panic!(val.0)
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
                dt: Positive::ONE,
                drift: Decimal::ZERO,
                volatility: pos_or_panic!(0.2),
            },
        );

        let walker = TestWalker {};
        let result = walker.brownian(&params)?;

        assert_eq!(result.len(), 5);
        Ok(())
    }

    #[test]
    fn test_error_handling() {
        #[derive(Clone)]
        struct ErrorWalker {}

        impl<X, Y> WalkTypeAble<X, Y> for ErrorWalker
        where
            X: Copy + TryInto<Positive> + AddAssign + Display,
            Y: Copy + TryInto<Positive> + Display,
        {
            fn brownian(
                &self,
                _params: &WalkParams<X, Y>,
            ) -> Result<Vec<Positive>, SimulationError> {
                Err("Error simulado para prueba".into())
            }

            fn geometric_brownian(
                &self,
                params: &WalkParams<X, Y>,
            ) -> Result<Vec<Positive>, SimulationError> {
                self.brownian(params)
            }

            fn log_returns(
                &self,
                params: &WalkParams<X, Y>,
            ) -> Result<Vec<Positive>, SimulationError> {
                self.brownian(params)
            }

            fn mean_reverting(
                &self,
                params: &WalkParams<X, Y>,
            ) -> Result<Vec<Positive>, SimulationError> {
                self.brownian(params)
            }

            fn jump_diffusion(
                &self,
                params: &WalkParams<X, Y>,
            ) -> Result<Vec<Positive>, SimulationError> {
                self.brownian(params)
            }

            fn garch(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
                self.brownian(params)
            }

            fn heston(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
                self.brownian(params)
            }

            fn custom(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
                self.brownian(params)
            }
        }

        let params = create_test_params(
            5,
            10.0,
            100.0,
            WalkType::Brownian {
                dt: Positive::ONE,
                drift: Decimal::ZERO,
                volatility: pos_or_panic!(0.2),
            },
        );

        let error_walker = ErrorWalker {};
        assert!(error_walker.brownian(&params).is_err());
    }

    /// Regression for issue #358: cloning through `Box<dyn WalkTypeAble>`
    /// delegates to `clone_box`, which forwards to the concrete walker's
    /// `Clone::clone`. Calling `.clone()` must not panic and must produce a
    /// walker that behaves identically to the original.
    #[test]
    fn test_box_dyn_walktypeable_clone_roundtrip() {
        #[derive(Clone, Debug, PartialEq)]
        struct CountingWalker {
            label: &'static str,
        }
        impl<X, Y> WalkTypeAble<X, Y> for CountingWalker
        where
            X: Copy + TryInto<Positive> + AddAssign + Display,
            Y: Copy + TryInto<Positive> + Display,
        {
        }

        let original: Box<dyn WalkTypeAble<Positive, Positive>> =
            Box::new(CountingWalker { label: "seed" });
        // Would previously panic; now forwards through clone_box to
        // CountingWalker::clone.
        let cloned = original.clone();

        // We cannot compare trait objects directly, so verify via downcast-ish
        // behaviour: both walkers expose the same `brownian` result for the
        // same WalkParams input.
        let params = create_test_params(
            3,
            pos_or_panic!(1.0),
            Positive::HUNDRED,
            WalkType::Brownian {
                dt: pos_or_panic!(0.01),
                drift: Decimal::ZERO,
                volatility: pos_or_panic!(0.2),
            },
        );
        let original_len = original
            .brownian(&params)
            .map(|v| v.len())
            .unwrap_or(usize::MAX);
        let cloned_len = cloned
            .brownian(&params)
            .map(|v| v.len())
            .unwrap_or(usize::MAX);
        assert_eq!(original_len, cloned_len);

        // The default `brownian` returns `Ok(Vec::new())` for
        // `WalkType::Brownian` when ystep cannot be coerced, which is fine —
        // we are asserting parity, not a specific length.
    }
}
