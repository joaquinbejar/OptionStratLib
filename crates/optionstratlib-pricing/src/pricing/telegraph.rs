/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/8/24
******************************************************************************/

//! # Telegraph Process
//!
//! A Telegraph Process (also known as a two-state process) is a stochastic process
//! that alternates between two states, typically represented as +1 and -1.
//!
//! ## Key Parameters
//!
//! - `lambda_up`: Transition rate from state -1 to +1
//! - `lambda_down`: Transition rate from state +1 to -1
//!
//! These parameters are always positive (λ_up, λ_down > 0) and typically range
//! from 0 to 10 in practice.
//!
//! ## Algorithm
//!
//! 1. The process starts in one of the two states (+1 or -1), usually chosen randomly.
//!
//! 2. At each time step dt:
//!    - If the current state is +1, there's a probability of changing to -1.
//!    - If the current state is -1, there's a probability of changing to +1.
//!
//! 3. The probability of change in an interval dt is calculated as:
//!    P(change) = 1 - e^(-λ * dt)
//!    Where λ is λ_up if the current state is -1, or λ_down if the current state is +1.
//!
//! ## Parameter Interpretation
//!
//! - Higher values indicate more frequent changes between states.
//! - Lower values indicate that the process tends to remain in a state for longer.
//!
//! Typical value ranges:
//! - Infrequent changes: 0.1 to 1
//! - Moderate changes: 1 to 5
//! - Very frequent changes: 5 to 10
//!
//! ## Relationship Between Parameters
//!
//! - If λ_up = λ_down, the process is symmetric.
//! - If λ_up > λ_down, the process tends to spend more time in the +1 state.
//! - If λ_up < λ_down, the process tends to spend more time in the -1 state.
//!
//! ## Use in Financial Modeling
//!
//! The [`telegraph()`] pricer uses the process as a regime-switching
//! volatility: the underlying diffuses at `sigma_plus` while the state is
//! +1 and at `sigma_minus` while it is -1 (see [`RegimeVolatility`]), with
//! the risk-neutral drift of the active regime, so discounted prices stay
//! martingales. More generally the process can model:
//! - Changes in volatility (high/low volatility regime)
//! - Changes in market direction (bullish/bearish trend)
//! - Changes in interest rates (high/low)
//!
//! ## Parameter Estimation
//!
//! Parameters can be estimated from historical data:
//! 1. Classify historical periods into +1 and -1 states based on a threshold.
//! 2. Calculate the average duration of each state.
//! 3. Estimate λ_up as 1 / (average duration of -1 state).
//! 4. Estimate λ_down as 1 / (average duration of +1 state).
//!
//! ## Advantages
//!
//! - Allows modeling of abrupt changes in the market.
//! - Captures "regime change" behaviors that continuous models can't easily represent.
//! - Relatively simple to implement and understand.
//!
//! ## Considerations
//!
//! - The choice of λ_up and λ_down significantly affects the model's behavior.
//! - These parameters may need to be calibrated with historical or market data.
//! - In more advanced models, λ_up and λ_down could be dynamically adjusted based on changing market conditions.
//!
//! Remember that the choice of these parameters depends heavily on the specific asset
//! being modeled and the time horizon of your analysis. It's common to experiment with
//! different values and validate results against real data to find the best configuration
//! for your specific model.

use crate::error::PricingError;
use crate::kernels::discount_factor;
use crate::pricing::utils::simulate_returns;
use num_traits::{FromPrimitive, ToPrimitive};
use optionstratlib_core::error::DecimalError;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{
    d_add, d_div, d_mul, d_powd, d_sqrt, d_sub, d_sum_iter, finite_decimal,
};
use rand::{Rng, RngExt};
use rand_distr::{Distribution, StandardNormal};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;
use tracing::{debug, warn};

/// Represents a Telegraph Process, a two-state continuous-time Markov chain model
/// used to simulate stochastic processes with discrete state transitions.
///
/// The Telegraph Process alternates between two states (-1 and +1), modeling regime
/// switches or market sentiment changes. State transitions are governed by Poisson
/// processes with specified rates. This model is particularly useful for simulating
/// financial markets that exhibit distinct behavioral regimes.
///
/// # Applications
///
/// - Modeling regime changes in market volatility
/// - Simulating discrete sentiment shifts in financial markets
/// - Representing asymmetric transition behaviors in stochastic systems
/// - Pricing financial derivatives under regime-switching assumptions
///
#[derive(Debug, Clone)]
pub struct TelegraphProcess {
    /// Transition rate from state -1 to +1, representing the intensity
    /// of the underlying Poisson process for upward state changes
    lambda_up: Decimal,

    /// Transition rate from state +1 to -1, representing the intensity
    /// of the underlying Poisson process for downward state changes
    lambda_down: Decimal,

    /// Current state of the process, which can be either -1 or +1,
    /// representing the two possible regimes of the system
    current_state: i8,
}

impl TelegraphProcess {
    /// Creates a new TelegraphProcess with the given transition rates.
    ///
    /// # Arguments
    ///
    /// * `lambda_up` - Transition rate from state -1 to +1
    /// * `lambda_down` - Transition rate from state +1 to -1
    /// * `rng` - The generator the initial state is drawn from (one uniform
    ///   draw). A seeded generator such as
    ///   [`optionstratlib_core::utils::deterministic_rng`] makes it
    ///   reproducible; pass `&mut rand::rng()` to draw from the thread-local
    ///   RNG.
    ///
    /// # Returns
    ///
    /// A new TelegraphProcess whose initial state is +1 or -1 with equal
    /// probability.
    #[must_use]
    pub fn new<R: Rng + ?Sized>(lambda_up: Decimal, lambda_down: Decimal, rng: &mut R) -> Self {
        let initial_state = if rng.random::<f64>() < 0.5 { 1 } else { -1 };
        TelegraphProcess {
            lambda_up,
            lambda_down,
            current_state: initial_state,
        }
    }

    /// Calculates the next state of the process.
    ///
    /// # Arguments
    ///
    /// * `dt` - Time step
    /// * `rng` - The generator the transition is drawn from (one uniform draw
    ///   per call).
    ///
    /// # Returns
    ///
    /// The new state of the process (-1 or 1)
    pub fn next_state<R: Rng + ?Sized>(&mut self, dt: Decimal, rng: &mut R) -> i8 {
        let lambda = if self.current_state == 1 {
            self.lambda_down
        } else {
            self.lambda_up
        };
        if rng.random::<f64>() < flip_probability(lambda, dt) {
            self.current_state *= -1;
        }

        self.current_state
    }

    /// Returns the current state of the process.
    ///
    /// # Returns
    ///
    /// The current state (-1 or 1)
    #[must_use]
    pub fn get_current_state(&self) -> i8 {
        self.current_state
    }
}

/// Probability that a telegraph state with exit rate `lambda` flips within
/// one step of length `dt`: `1 - exp(-lambda * dt)`, as an `f64` ready to be
/// compared against a uniform draw.
///
/// Infallible by design: every unrepresentable intermediate degrades to the
/// limit its sign implies rather than aborting.
fn flip_probability(lambda: Decimal, dt: Decimal) -> f64 {
    // lambda_dt is non-positive for the physical case (lambda, dt >= 0).
    // For very-negative values exp(lambda_dt) underflows to 0; treat as a
    // guaranteed flip (probability = 1). Otherwise use the standard
    // Poisson transition: P(flip in dt) = 1 - exp(-lambda * dt).
    //
    // An exponent that overflows downward is a certain flip, one that
    // overflows upward drives `1 - exp(..)` far below zero, i.e. no flip.
    let probability = match (-lambda).checked_mul(dt) {
        None if lambda.is_sign_negative() == dt.is_sign_negative() => Decimal::ONE,
        None => Decimal::ZERO,
        Some(lambda_dt) if lambda_dt < dec!(-11.7) => Decimal::ONE,
        Some(lambda_dt) => match lambda_dt.checked_exp() {
            Some(decay) => Decimal::ONE.checked_sub(decay).unwrap_or(Decimal::ZERO),
            None if lambda_dt.is_sign_negative() => Decimal::ONE,
            None => Decimal::ZERO,
        },
    };

    // probability is mathematically in [0, 1] (Decimal::ONE or 1 - exp(neg)); to_f64 is
    // expected to succeed. If conversion ever fails we log and treat the period as
    // "no transition" rather than panicking.
    probability.to_f64().unwrap_or_else(|| {
        warn!(
            probability = %probability,
            "telegraph::flip_probability: probability.to_f64() returned None; treating as 0.0"
        );
        0.0
    })
}

/// Estimates the Telegraph Process parameters from historical data.
///
/// # Arguments
///
/// * `returns` - A slice of historical returns
/// * `threshold` - The threshold used to classify states
///
/// # Description
///
/// This method updates the `lambda_up` and `lambda_down` parameters of the process
/// based on the provided historical data. It classifies each return as belonging to
/// state +1 or -1 based on the threshold, then calculates the average duration of
/// each state to estimate the transition rates.
pub(crate) fn estimate_telegraph_parameters(
    returns: &[Decimal],
    threshold: Decimal,
) -> Result<(Decimal, Decimal), DecimalError> {
    // Allow threshold to be zero - it's a valid threshold for classification
    // Returns are classified as +1 if > threshold, -1 if <= threshold
    let first = returns.first().ok_or_else(|| {
        DecimalError::invalid_value(0.0, "returns must contain at least one observation")
    })?;
    let mut current_state = if *first > threshold {
        Decimal::ONE
    } else {
        Decimal::NEGATIVE_ONE
    };
    let mut current_duration = Decimal::ONE;
    let mut up_durations = Vec::new();
    let mut down_durations = Vec::new();

    for &ret in returns.iter().skip(1) {
        let new_state = if ret > threshold {
            Decimal::ONE
        } else {
            Decimal::NEGATIVE_ONE
        };
        if new_state == current_state {
            current_duration = d_add(
                current_duration,
                Decimal::ONE,
                "pricing::telegraph::estimate::duration",
            )?;
        } else {
            if current_state == Decimal::ONE {
                up_durations.push(current_duration);
            } else {
                down_durations.push(current_duration);
            }
            current_state = new_state;
            current_duration = Decimal::ONE;
        }
    }

    if current_state == Decimal::ONE {
        up_durations.push(current_duration);
    } else {
        down_durations.push(current_duration);
    }

    // Check if we have transitions in both directions
    if down_durations.is_empty() {
        return Err(DecimalError::InvalidValue {
            value: 0.0,
            reason: "No transitions from state +1 to -1 found. All returns are above threshold."
                .to_string(),
        });
    }

    if up_durations.is_empty() {
        return Err(DecimalError::InvalidValue {
            value: 0.0,
            reason: "No transitions from state -1 to +1 found. All returns are below threshold."
                .to_string(),
        });
    }

    let sum_down = d_sum_iter(
        down_durations.iter().copied(),
        "pricing::telegraph::estimate::sum_down",
    )?;
    let sum_up = d_sum_iter(
        up_durations.iter().copied(),
        "pricing::telegraph::estimate::sum_up",
    )?;

    if sum_down == Decimal::ZERO {
        return Err(DecimalError::InvalidValue {
            value: sum_down.to_f64().unwrap_or(0.0),
            reason: "Sum of down durations must be non-zero".to_string(),
        });
    }

    if sum_up == Decimal::ZERO {
        return Err(DecimalError::InvalidValue {
            value: sum_up.to_f64().unwrap_or(0.0),
            reason: "Sum of up durations must be non-zero".to_string(),
        });
    }

    let down_len = Decimal::from_usize(down_durations.len()).ok_or_else(|| {
        DecimalError::invalid_value(
            down_durations.len() as f64,
            "down_durations length not representable as Decimal",
        )
    })?;
    let up_len = Decimal::from_usize(up_durations.len()).ok_or_else(|| {
        DecimalError::invalid_value(
            up_durations.len() as f64,
            "up_durations length not representable as Decimal",
        )
    })?;
    let lambda_up = d_mul(
        d_div(
            Decimal::ONE,
            sum_down,
            "pricing::telegraph::estimate::inv_sum_down",
        )?,
        down_len,
        "pricing::telegraph::estimate::lambda_up",
    )?;
    let lambda_down = d_mul(
        d_div(
            Decimal::ONE,
            sum_up,
            "pricing::telegraph::estimate::inv_sum_up",
        )?,
        up_len,
        "pricing::telegraph::estimate::lambda_down",
    )?;
    Ok((lambda_up, lambda_down))
}

/// The two volatility levels the [`telegraph()`] pricer switches between.
///
/// While the telegraph state is +1 the underlying diffuses at
/// `sigma_plus`, while it is -1 at `sigma_minus`. Both levels are annualised
/// and strictly positive. With equal levels the regime has no effect and the
/// pricer converges to Black-Scholes at that volatility.
///
/// # Examples
///
/// ```rust
/// use optionstratlib_core::pos_or_panic;
/// use optionstratlib_pricing::pricing::RegimeVolatility;
///
/// let regimes = RegimeVolatility::new(pos_or_panic!(0.35), pos_or_panic!(0.15)).unwrap();
/// assert_eq!(regimes.sigma_plus(), pos_or_panic!(0.35));
/// assert_eq!(regimes.sigma_minus(), pos_or_panic!(0.15));
/// assert!(RegimeVolatility::new(pos_or_panic!(0.35), pos_or_panic!(0.0)).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegimeVolatility {
    /// Volatility of the +1 regime.
    sigma_plus: Positive,
    /// Volatility of the -1 regime.
    sigma_minus: Positive,
}

impl RegimeVolatility {
    /// Builds the pair of regime volatilities.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError::InvalidParameter`] when either level is zero.
    pub fn new(sigma_plus: Positive, sigma_minus: Positive) -> Result<Self, PricingError> {
        Ok(Self {
            sigma_plus: strictly_positive_sigma("sigma_plus", sigma_plus)?,
            sigma_minus: strictly_positive_sigma("sigma_minus", sigma_minus)?,
        })
    }

    /// Builds a pair whose two regimes share the volatility `sigma`, under
    /// which the telegraph pricer converges to Black-Scholes at `sigma`.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError::InvalidParameter`] when `sigma` is zero.
    pub fn constant(sigma: Positive) -> Result<Self, PricingError> {
        Self::new(sigma, sigma)
    }

    /// Volatility of the +1 regime.
    #[must_use]
    #[inline]
    pub fn sigma_plus(&self) -> Positive {
        self.sigma_plus
    }

    /// Volatility of the -1 regime.
    #[must_use]
    #[inline]
    pub fn sigma_minus(&self) -> Positive {
        self.sigma_minus
    }
}

/// Returns `sigma` when it is strictly positive, the typed rejection
/// otherwise.
fn strictly_positive_sigma(
    parameter: &'static str,
    sigma: Positive,
) -> Result<Positive, PricingError> {
    if sigma.is_zero() {
        return Err(PricingError::invalid_parameter(
            parameter,
            sigma.to_dec(),
            "a regime volatility must be strictly positive",
        ));
    }
    Ok(sigma)
}

/// Returns a caller-supplied transition rate when it is non-negative, the
/// typed rejection otherwise. Zero is admissible: the regime never leaves.
fn non_negative_rate(
    parameter: &'static str,
    rate: Option<Decimal>,
) -> Result<Option<Decimal>, PricingError> {
    match rate {
        Some(value) if value.is_sign_negative() && !value.is_zero() => {
            Err(PricingError::invalid_parameter(
                parameter,
                value,
                "a transition rate must be non-negative",
            ))
        }
        _ => Ok(rate),
    }
}

/// Per-step log-Euler coefficients of one volatility regime, as `f64` for
/// the path loop.
#[derive(Debug, Clone, Copy)]
struct RegimeStep {
    /// `(carry - sigma^2 / 2) * dt`.
    drift_dt: f64,
    /// `sigma * sqrt(dt)`.
    diffusion: f64,
}

impl RegimeStep {
    /// Coefficients for volatility `sigma` under the risk-neutral `carry`
    /// (the drift rate of the underlying before the Ito correction).
    fn new(
        carry: Decimal,
        sigma: Positive,
        dt: Decimal,
        sqrt_dt: Decimal,
    ) -> Result<Self, PricingError> {
        let sigma = sigma.to_dec();
        let drift = d_sub(
            carry,
            d_mul(
                dec!(0.5),
                d_powd(sigma, Decimal::TWO, "pricing::telegraph::variance")?,
                "pricing::telegraph::half_variance",
            )?,
            "pricing::telegraph::drift",
        )?;
        let drift_dt = d_mul(drift, dt, "pricing::telegraph::drift_dt")?;
        let diffusion = d_mul(sigma, sqrt_dt, "pricing::telegraph::diffusion")?;
        Ok(Self {
            drift_dt: drift_dt.to_f64().ok_or_else(|| {
                PricingError::method_error("telegraph", "drift * dt not representable as f64")
            })?,
            diffusion: diffusion.to_f64().ok_or_else(|| {
                PricingError::method_error("telegraph", "sigma * sqrt(dt) not representable as f64")
            })?,
        })
    }
}

/// Number of Monte-Carlo paths
/// [`OptionPricing::calculate_price_telegraph`](crate::pricing::OptionPricing::calculate_price_telegraph)
/// averages the discounted payoff over, and a sensible `no_paths` for
/// [`telegraph()`].
///
/// The standard error of the estimate shrinks as `1 / sqrt(no_paths)`: with
/// 10 000 paths it is 1% of the payoff's standard deviation (for an
/// at-the-money one-year call with 20% volatility, about 0.15 on a price
/// near 10.45). The cost is `no_paths * no_steps` steps of two draws each.
pub const TELEGRAPH_PATHS: NonZeroUsize = match NonZeroUsize::new(10_000) {
    Some(paths) => paths,
    None => NonZeroUsize::MIN,
};

/// Prices an option under telegraph regime-switching volatility.
///
/// A two-state telegraph process selects the volatility of a log-Euler
/// geometric Brownian motion: `sigma_plus` while the state is +1,
/// `sigma_minus` while it is -1 (see [`RegimeVolatility`]). Each step of
/// length `dt = T / no_steps` first advances the regime (flip probability
/// `1 - exp(-lambda * dt)`, `lambda` being the exit rate of the current
/// regime: `lambda_down` leaves +1, `lambda_up` leaves -1) and then updates
/// the price with the active regime's volatility `sigma_s`:
///
/// ```text
/// S <- S * exp((r - q - sigma_s^2 / 2) * dt + sigma_s * sqrt(dt) * Z),  Z ~ N(0, 1)
/// ```
///
/// where `q` is the option's continuous dividend yield (#756). The drift of
/// every step is the risk-neutral one of the regime it is taken in, so the
/// discounted, dividend-adjusted underlying is a martingale and put-call
/// parity holds whatever the rates. Every path draws its own initial regime
/// (+1 or -1 with equal probability), and the price is the average of the
/// discounted payoff over `no_paths` paths: a Monte-Carlo estimate of the
/// risk-neutral expectation.
///
/// Conditional on its regime path a price is lognormal with total variance
/// `sigma_plus^2 * T_plus + sigma_minus^2 * T_minus`, `T_plus` and `T_minus`
/// being the time spent in each regime. A European price therefore lies
/// between the Black-Scholes prices at `sigma_minus` and `sigma_plus`, and
/// rises with the share of time the rates keep the path in the higher
/// volatility regime. With `sigma_plus == sigma_minus` the estimate
/// converges to the Black-Scholes price at that volatility and the same
/// dividend yield.
///
/// # Arguments
///
/// * `option` - Reference to the Options structure containing all option parameters
/// * `no_steps` - Number of time steps of every simulated path
/// * `no_paths` - Number of simulated paths the discounted payoff is averaged
///   over; [`TELEGRAPH_PATHS`] is the count the [`OptionPricing`] trait uses.
/// * `lambda_up` - Optional transition rate from down state (-1) to up state
///   (+1); must be non-negative when given
/// * `lambda_down` - Optional transition rate from up state (+1) to down
///   state (-1); must be non-negative when given
/// * `volatility` - The volatilities of the +1 and -1 regimes. The option's
///   own `implied_volatility` is not used to diffuse the price.
/// * `rng` - The generator every draw is taken from: the returns simulated to
///   estimate a missing rate, then for each path its initial state, one
///   transition draw and one standard normal shock per step. A seeded
///   generator such as [`optionstratlib_core::utils::deterministic_rng`]
///   makes the price reproducible; pass `&mut rand::rng()` to draw from the
///   thread-local RNG.
///
/// # Returns
///
/// * `Result<Decimal, PricingError>` - The Monte-Carlo option price or an error
///
/// # Details
///
/// The function handles parameter estimation automatically if transition rates are not provided.
/// When missing, it simulates returns based on the option's implied volatility to estimate
/// appropriate telegraph parameters.
///
/// # Errors
///
/// Returns `PricingError::InvalidParameter` when a supplied transition rate
/// is negative, `PricingError::ExpirationDate` when the option's
/// expiration cannot be converted, a decimal error when parameter
/// estimation produces degenerate rates, `PricingError::NonFinite` when a
/// simulated terminal price is not representable, and
/// `PricingError::MethodError` when an intermediate is not representable
/// as `f64` or the averaging overflows.
///
/// # References
///
/// Hamilton (1989), "A New Approach to the Economic Analysis of
/// Nonstationary Time Series and the Business Cycle", Econometrica 57(2),
/// for the two-state Markov regime; Naik (1993), "Option Valuation and
/// Hedging Strategies with Jumps in the Volatility of Asset Returns",
/// Journal of Finance 48(5), for option pricing under a two-state
/// volatility.
///
/// [`OptionPricing`]: crate::pricing::OptionPricing
#[tracing::instrument(skip(option, rng), level = "debug")]
pub fn telegraph<R: Rng + ?Sized>(
    option: &Options,
    no_steps: NonZeroUsize,
    no_paths: NonZeroUsize,
    lambda_up: Option<Decimal>,
    lambda_down: Option<Decimal>,
    volatility: RegimeVolatility,
    rng: &mut R,
) -> Result<Decimal, PricingError> {
    let lambda_up = non_negative_rate("lambda_up", lambda_up)?;
    let lambda_down = non_negative_rate("lambda_down", lambda_down)?;
    let no_steps_raw = no_steps.get();
    let no_paths_raw = no_paths.get();
    let no_steps_dec = Decimal::from_usize(no_steps_raw).ok_or_else(|| {
        PricingError::method_error("telegraph", &format!("invalid no_steps: {no_steps_raw}"))
    })?;
    let no_paths_dec = Decimal::from_usize(no_paths_raw).ok_or_else(|| {
        PricingError::method_error("telegraph", &format!("invalid no_paths: {no_paths_raw}"))
    })?;
    let time_to_expiration = option.time_to_expiration()?.to_dec();
    let dt = d_div(time_to_expiration, no_steps_dec, "pricing::telegraph::dt")?;

    let one_over_252 = finite_decimal(1.0 / 252.0)
        .ok_or_else(|| PricingError::non_finite("pricing::telegraph::one_over_252", 1.0 / 252.0))?;

    let (lambda_up_temp, lambda_down_temp) = match (lambda_up, lambda_down) {
        (None, None) => {
            let returns = simulate_returns(
                Decimal::ZERO,
                option.implied_volatility,
                100,
                one_over_252,
                rng,
            )?;
            estimate_telegraph_parameters(&returns, Decimal::ZERO)?
        }
        (Some(l_up), None) => {
            let returns = simulate_returns(
                Decimal::ZERO,
                option.implied_volatility,
                100,
                one_over_252,
                rng,
            )?;
            let (_, l_down) = estimate_telegraph_parameters(&returns, Decimal::ZERO)?;
            (l_up, l_down)
        }
        (None, Some(l_down)) => {
            let returns = simulate_returns(
                Decimal::ZERO,
                option.implied_volatility,
                100,
                one_over_252,
                rng,
            )?;
            let (l_up, _) = estimate_telegraph_parameters(&returns, Decimal::ZERO)?;
            (l_up, l_down)
        }
        (Some(l_up), Some(l_down)) => (l_up, l_down),
    };
    // `dt` is the same on every step, so each regime's flip probability is
    // loop-invariant: the up regime leaves at `lambda_down`, the down regime
    // at `lambda_up` (the same law as `TelegraphProcess::next_state`).
    let flip_from_up = flip_probability(lambda_down_temp, dt);
    let flip_from_down = flip_probability(lambda_up_temp, dt);

    // Loop-invariant per-regime coefficients. `carry` is the risk-neutral
    // drift rate `r - q` of the underlying before the Ito correction each
    // regime subtracts; the dividend yield was missing until #756.
    let carry = d_sub(
        option.risk_free_rate,
        option.dividend_yield.to_dec(),
        "pricing::telegraph::carry",
    )?;
    let sqrt_dt = d_sqrt(dt, "pricing::telegraph::sqrt_dt")
        .map_err(|_| PricingError::method_error("telegraph", "non-finite dt sqrt"))?;
    let step_up = RegimeStep::new(carry, volatility.sigma_plus, dt, sqrt_dt)?;
    let step_down = RegimeStep::new(carry, volatility.sigma_minus, dt, sqrt_dt)?;

    let spot = option.underlying_price.to_dec();
    let mut payoff_sum = Decimal::ZERO;
    // f64 moments of the payoff, only for the standard error traced below.
    let mut payoff_sum_f64 = 0.0_f64;
    let mut payoff_sum_sq_f64 = 0.0_f64;
    for _ in 0..no_paths_raw {
        let mut state_up = rng.random::<f64>() < 0.5;
        let mut log_return = 0.0_f64;
        for _ in 0..no_steps_raw {
            let flip = if state_up {
                flip_from_up
            } else {
                flip_from_down
            };
            if rng.random::<f64>() < flip {
                state_up = !state_up;
            }
            let z: f64 = StandardNormal.sample(rng);
            let step = if state_up { step_up } else { step_down };
            log_return += step.drift_dt + step.diffusion * z;
        }
        let growth_f64 = log_return.exp(); // scan-banned: allow -- f64 `exp`: returns inf on overflow, it does not abort; the non-finite value is rejected by `finite_decimal` below
        let growth = finite_decimal(growth_f64)
            .ok_or_else(|| PricingError::non_finite("pricing::telegraph::growth", growth_f64))?;
        let terminal =
            Positive::new_decimal(d_mul(spot, growth, "pricing::telegraph::terminal_price")?)?;
        let payoff = option.payoff_at_price(&terminal)?;
        payoff_sum = d_add(payoff_sum, payoff, "pricing::telegraph::payoff_sum")?;
        let payoff_f64 = payoff.to_f64().unwrap_or(0.0);
        payoff_sum_f64 += payoff_f64;
        payoff_sum_sq_f64 += payoff_f64 * payoff_f64;
    }

    let n = no_paths_raw as f64;
    let mean_f64 = payoff_sum_f64 / n;
    let variance_f64 = (payoff_sum_sq_f64 / n - mean_f64 * mean_f64).max(0.0);
    let std_error_f64 = (variance_f64 / n).sqrt(); // scan-banned: allow -- f64 `sqrt` of a non-negative value, only traced
    debug!(
        paths = no_paths_raw,
        steps = no_steps_raw,
        payoff_mean = mean_f64,
        payoff_std_error = std_error_f64,
        "telegraph Monte-Carlo estimate"
    );

    let mean_payoff = d_div(payoff_sum, no_paths_dec, "pricing::telegraph::mean_payoff")?;
    // Build the discount exponent through a checked multiplication so
    // an overflow on `-risk_free_rate * time_to_expiration` is tagged
    // before `.exp()` compresses it back into a bounded range.
    let discount = discount_factor(
        option.risk_free_rate,
        time_to_expiration,
        "pricing::telegraph::discount_exponent",
        "pricing::telegraph::discount",
    )?;
    let result = d_mul(mean_payoff, discount, "pricing::telegraph::price")?;
    Ok(result)
}

/// Both regimes at the 20% volatility the test options are quoted at.
#[cfg(test)]
fn regimes_20() -> RegimeVolatility {
    RegimeVolatility::constant(optionstratlib_core::pos_or_panic!(0.2)).unwrap()
}

#[cfg(test)]
mod tests_telegraph_process_basis {
    use super::*;
    use optionstratlib_core::utils::{DETERMINISTIC_RNG_DEFAULT_SEED, deterministic_rng};
    use optionstratlib_core::{model::Positive, pos_or_panic};

    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use rust_decimal_macros::dec;

    #[test]
    fn test_telegraph_process_new() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let tp = TelegraphProcess::new(dec!(0.5), dec!(0.3), &mut rng);
        assert_eq!(tp.lambda_up, dec!(0.5));
        assert_eq!(tp.lambda_down, dec!(0.3));
        assert!(tp.current_state == 1 || tp.current_state == -1);
    }

    #[test]
    fn test_telegraph_process_next_state() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let mut tp = TelegraphProcess::new(Decimal::ONE, Decimal::ONE, &mut rng);
        let _initial_state = tp.get_current_state();
        let new_state = tp.next_state(dec!(0.1), &mut rng);
        assert!(new_state == 1 || new_state == -1);
        // There's a chance the state didn't change, so we can't assert inequality
    }

    #[test]
    fn test_next_state_empirical_flip_rate_matches_poisson() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        // Regression test for #351: prior code had an inverted underflow
        // guard that forced probability = 1.0 every step. Verify the
        // empirical flip rate now matches the Poisson transition
        // probability  P(flip in dt) = 1 - exp(-lambda * dt)  within a
        // 5 σ Monte-Carlo bound.
        let lambda_f = 0.5_f64;
        let dt_f = 0.01_f64;
        let mut tp = TelegraphProcess::new(dec!(0.5), dec!(0.5), &mut rng);

        let n = 100_000_u64;
        let mut prev = tp.get_current_state();
        let mut flips: u64 = 0;
        for _ in 0..n {
            let next = tp.next_state(dec!(0.01), &mut rng);
            if next != prev {
                flips += 1;
            }
            prev = next;
        }
        let empirical = flips as f64 / n as f64;
        let expected = 1.0 - (-lambda_f * dt_f).exp();
        let std_err = (expected * (1.0 - expected) / n as f64).sqrt();

        assert!(
            (empirical - expected).abs() < 5.0 * std_err,
            "empirical flip rate {empirical} differs from expected {expected} by more than 5σ ({})",
            5.0 * std_err
        );
        // Sanity: must be far below 1.0 (the buggy value).
        assert!(empirical < 0.05, "flip rate suspiciously high: {empirical}");
    }

    #[test]
    fn test_telegraph_process_get_current_state() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let tp = TelegraphProcess::new(dec!(0.5), dec!(0.5), &mut rng);
        let state = tp.get_current_state();
        assert!(state == 1 || state == -1);
    }

    #[test]
    fn test_estimate_telegraph_parameters() {
        let returns = vec![
            dec!(-0.01),
            dec!(0.02),
            dec!(0.01),
            dec!(-0.02),
            dec!(0.03),
            dec!(-0.01),
            dec!(0.01),
            dec!(-0.03),
        ];
        let threshold = dec!(0.01);
        let (lambda_up, lambda_down) = estimate_telegraph_parameters(&returns, threshold).unwrap();
        assert!(lambda_up > Decimal::ZERO);
        assert!(lambda_down > Decimal::ZERO);
    }

    #[test]
    fn test_telegraph() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        // Create a mock Options struct
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: Positive::ONE,
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            implied_volatility: pos_or_panic!(0.2),
            underlying_symbol: "".to_string(),
            expiration_date: Default::default(),
            quantity: Positive::ONE,
            contract_size: Positive::ONE,
            exotic_params: None,
        };

        let _price = telegraph(
            &option,
            optionstratlib_core::nz!(1000),
            optionstratlib_core::nz!(1_000),
            Some(dec!(0.7)),
            Some(dec!(0.5)),
            regimes_20(),
            &mut rng,
        );
        // price is stochastic
        // assert_relative_eq!(price, 0.0, epsilon = 0.0001);
    }
}

#[cfg(test)]
mod tests_telegraph_process_extended {
    use super::*;
    use optionstratlib_core::utils::{DETERMINISTIC_RNG_DEFAULT_SEED, deterministic_rng};
    use optionstratlib_core::{model::Positive, pos_or_panic};

    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};

    use rust_decimal_macros::dec;

    // Helper function to create a mock Options struct
    fn create_mock_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: Positive::HUNDRED,
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            implied_volatility: pos_or_panic!(0.2),
            underlying_symbol: "".to_string(),
            expiration_date: Default::default(),
            quantity: Positive::ZERO,
            contract_size: Positive::ONE,
            exotic_params: None,
        }
    }

    #[test]
    fn test_telegraph_process_new() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let tp = TelegraphProcess::new(dec!(0.5), dec!(0.3), &mut rng);
        assert_eq!(tp.lambda_up, dec!(0.5));
        assert_eq!(tp.lambda_down, dec!(0.3));
        assert!(tp.get_current_state() == 1 || tp.get_current_state() == -1);
    }

    #[test]
    fn test_telegraph_process_next_state() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let mut tp = TelegraphProcess::new(dec!(1000.0), dec!(1000.0), &mut rng); // High rates to ensure state change
        let initial_state = tp.get_current_state();
        let new_state = tp.next_state(dec!(0.1), &mut rng);
        assert_ne!(initial_state, new_state);
    }

    #[test]
    fn test_telegraph_process_get_current_state() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let tp = TelegraphProcess::new(dec!(0.5), dec!(0.5), &mut rng);
        let state = tp.get_current_state();
        assert!(state == 1 || state == -1);
    }

    #[test]
    fn test_estimate_telegraph_parameters() {
        let returns = vec![
            dec!(-0.01),
            dec!(0.02),
            dec!(0.01),
            dec!(-0.02),
            dec!(0.03),
            dec!(-0.01),
            dec!(0.01),
            dec!(-0.03),
        ];
        let threshold = Decimal::ZERO;
        let result = estimate_telegraph_parameters(&returns, threshold);
        assert!(result.is_ok());
        let (lambda_up, lambda_down) = result.unwrap();
        assert!(lambda_up > Decimal::ZERO);
        assert!(lambda_down > Decimal::ZERO);
    }

    #[test]
    fn test_estimate_telegraph_parameters_all_positive() {
        let returns = vec![
            dec!(0.01),
            dec!(0.02),
            dec!(0.01),
            dec!(0.02),
            dec!(0.03),
            dec!(0.01),
            dec!(0.01),
            dec!(0.03),
        ];
        let threshold = Decimal::ZERO;
        // All returns are positive, so all will be in state +1, no state transitions
        // This should result in an error due to empty down_durations
        assert!(estimate_telegraph_parameters(&returns, threshold).is_err());
    }

    #[test]
    fn test_estimate_telegraph_parameters_all_negative() {
        let returns = vec![
            dec!(-0.01),
            dec!(-0.02),
            dec!(-0.01),
            dec!(-0.02),
            dec!(-0.03),
            dec!(-0.01),
            dec!(-0.01),
            dec!(-0.03),
        ];
        let threshold = dec!(0.01);
        assert!(estimate_telegraph_parameters(&returns, threshold).is_err());
    }

    #[test]
    fn test_telegraph_with_provided_parameters() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let option = create_mock_option();
        let _price = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            regimes_20(),
            &mut rng,
        );
        // assert!(price > 0.0);
    }

    #[test]
    fn test_telegraph_with_estimated_parameters() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let option = create_mock_option();
        let _price = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            None,
            None,
            regimes_20(),
            &mut rng,
        );
        // assert!(price > 0.0);
    }

    #[test]
    fn test_telegraph_with_one_estimated_parameter() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let option = create_mock_option();
        let _price_up = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            Some(dec!(0.5)),
            None,
            regimes_20(),
            &mut rng,
        );
        let _price_down = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            None,
            Some(dec!(0.5)),
            regimes_20(),
            &mut rng,
        );

        // assert!(price_up > 0.0);
        // assert!(price_down > 0.0);
    }

    #[test]
    fn test_telegraph_different_no_steps() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let option = create_mock_option();
        let _price_100 = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            regimes_20(),
            &mut rng,
        );
        let _price_1000 = telegraph(
            &option,
            optionstratlib_core::nz!(1000),
            optionstratlib_core::nz!(1_000),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            regimes_20(),
            &mut rng,
        );

        // assert!(price_100 > 0.0);
        // assert!(price_1000 > 0.0);
        // assert_ne!(price_100, price_1000);
    }

    #[test]
    fn test_regime_volatility_zero_sigma_rejected() {
        for (plus, minus, name) in [
            (Positive::ZERO, pos_or_panic!(0.2), "sigma_plus"),
            (pos_or_panic!(0.2), Positive::ZERO, "sigma_minus"),
        ] {
            match RegimeVolatility::new(plus, minus) {
                Err(PricingError::InvalidParameter { parameter, .. }) => {
                    assert_eq!(parameter, name)
                }
                other => panic!("expected InvalidParameter for {name}, got {other:?}"),
            }
        }
        assert!(RegimeVolatility::constant(Positive::ZERO).is_err());
    }

    #[test]
    fn test_telegraph_negative_rate_rejected() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let option = create_mock_option();
        for (up, down, name) in [
            (Some(dec!(-0.5)), Some(dec!(0.5)), "lambda_up"),
            (Some(dec!(0.5)), Some(dec!(-0.5)), "lambda_down"),
            (None, Some(dec!(-0.1)), "lambda_down"),
        ] {
            match telegraph(
                &option,
                optionstratlib_core::nz!(10),
                optionstratlib_core::nz!(10),
                up,
                down,
                regimes_20(),
                &mut rng,
            ) {
                Err(PricingError::InvalidParameter { parameter, .. }) => {
                    assert_eq!(parameter, name)
                }
                other => panic!("expected InvalidParameter for {name}, got {other:?}"),
            }
        }
    }

    #[test]
    fn test_telegraph_zero_risk_free_rate() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let mut option = create_mock_option();
        option.risk_free_rate = Decimal::ZERO;
        let _price = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            regimes_20(),
            &mut rng,
        );
        // assert!(price > 0.0);
    }

    #[test]
    fn test_telegraph_zero_time_to_expiration() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let option = create_mock_option();
        let price = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            regimes_20(),
            &mut rng,
        )
        .unwrap();
        assert_eq!(
            price,
            option.payoff_at_price(&option.underlying_price).unwrap()
        );
    }
}

#[cfg(test)]
mod tests_telegraph_seeded {
    use super::*;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::pos_or_panic;
    use optionstratlib_core::utils::{DETERMINISTIC_RNG_DEFAULT_SEED, deterministic_rng};
    use rust_decimal_macros::dec;

    /// `telegraph(&option_30d(), 100, TELEGRAPH_PATHS, Some(0.5), Some(0.5),
    /// regimes_20())` drawn from
    /// `deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED)`. Re-baselined by
    /// #743: the kernel now averages `TELEGRAPH_PATHS` paths driven by a
    /// standard normal shock instead of returning one path driven by a
    /// positive uniform. Re-baselined by #755: the regime selects the
    /// volatility instead of the sign of the shock, so each draw enters
    /// unsigned (both regimes at 20% here).
    const PINNED_TELEGRAPH_PRICE: Decimal = dec!(40.237916421487854842265370875);

    fn option_30d() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: pos_or_panic!(60.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            implied_volatility: pos_or_panic!(0.2),
            underlying_symbol: "TEST".to_string(),
            expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
            quantity: Positive::ONE,
            contract_size: Positive::ONE,
            exotic_params: None,
        }
    }

    fn state_path(seed: u64, steps: usize) -> Vec<i8> {
        let mut rng = deterministic_rng(seed);
        let mut tp = TelegraphProcess::new(dec!(1.0), dec!(2.0), &mut rng);
        let mut path = vec![tp.get_current_state()];
        path.extend((0..steps).map(|_| tp.next_state(dec!(0.1), &mut rng)));
        path
    }

    #[test]
    fn test_telegraph_process_same_seed_identical_state_path() {
        assert_eq!(state_path(21, 500), state_path(21, 500));
        assert_ne!(state_path(21, 500), state_path(22, 500));
    }

    #[test]
    fn test_telegraph_process_new_seeded_initial_state_takes_both_values() {
        let states: Vec<i8> = (0..64)
            .map(|seed| {
                TelegraphProcess::new(Decimal::ONE, Decimal::ONE, &mut deterministic_rng(seed))
                    .get_current_state()
            })
            .collect();
        assert!(states.contains(&1));
        assert!(states.contains(&-1));
    }

    #[test]
    fn test_telegraph_process_seeded_occupancy_matches_stationary_law() {
        // Discrete chain with flip probabilities p_up = 1 - exp(-2 dt) and
        // p_down = 1 - exp(-dt): the long-run share of time in +1 is
        // p_up / (p_up + p_down) = 0.66556. The state correlation time is
        // 1 / (lambda_up + lambda_down) = 1/3, so over T = 2000 the time
        // average has a standard deviation of about 0.0086; the band is 5 of
        // them.
        let lambda_up = 2.0_f64;
        let lambda_down = 1.0_f64;
        let dt = 0.01_f64;
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let mut tp = TelegraphProcess::new(dec!(2.0), dec!(1.0), &mut rng);
        let n = 200_000_u64;
        let mut up: u64 = 0;
        for _ in 0..n {
            if tp.next_state(dec!(0.01), &mut rng) == 1 {
                up += 1;
            }
        }
        let p_up = 1.0 - (-lambda_up * dt).exp();
        let p_down = 1.0 - (-lambda_down * dt).exp();
        let expected = p_up / (p_up + p_down);
        let share = up as f64 / n as f64;
        assert!(
            (share - expected).abs() < 0.043,
            "share of time in +1 {share} differs from stationary {expected}"
        );
    }

    #[test]
    fn test_telegraph_same_seed_identical_price() {
        let option = option_30d();
        for lambdas in [
            (Some(dec!(0.5)), Some(dec!(0.5))),
            (Some(dec!(0.5)), None),
            (None, Some(dec!(0.5))),
            (None, None),
        ] {
            let price = |seed: u64| {
                telegraph(
                    &option,
                    optionstratlib_core::nz!(100),
                    TELEGRAPH_PATHS,
                    lambdas.0,
                    lambdas.1,
                    regimes_20(),
                    &mut deterministic_rng(seed),
                )
                .unwrap()
            };
            assert_eq!(price(3), price(3), "lambdas {lambdas:?}");
        }
    }

    #[test]
    fn test_telegraph_seeded_regression_pinned() {
        let price = telegraph(
            &option_30d(),
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            regimes_20(),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        assert_eq!(price, PINNED_TELEGRAPH_PRICE);
    }

    #[test]
    fn test_calculate_price_telegraph_matches_kernel_on_same_seed() {
        use crate::pricing::OptionPricing;
        let option = option_30d();
        let via_trait = option
            .calculate_price_telegraph(
                optionstratlib_core::nz!(100),
                regimes_20(),
                &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
            )
            .unwrap();
        let via_kernel = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            None,
            None,
            regimes_20(),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        assert_eq!(via_trait, via_kernel);
    }

    /// One-year at-the-money option, no dividend: the case Black-Scholes and
    /// the telegraph kernel price under the same law.
    fn option_1y_atm(style: OptionStyle) -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: Positive::HUNDRED,
            risk_free_rate: dec!(0.05),
            option_style: style,
            dividend_yield: Positive::ZERO,
            implied_volatility: pos_or_panic!(0.2),
            underlying_symbol: "TEST".to_string(),
            expiration_date: ExpirationDate::Days(pos_or_panic!(365.0)),
            quantity: Positive::ONE,
            contract_size: Positive::ONE,
            exotic_params: None,
        }
    }

    #[test]
    fn test_telegraph_switching_disabled_converges_to_black_scholes() {
        use crate::pricing::black_scholes_model::black_scholes;
        // Zero rates keep every path in its initial regime. With a standard
        // normal shock the log-Euler step is exact for GBM, so the estimate
        // is unbiased for the Black-Scholes price (10.4506 call, 5.5735
        // put). The payoff standard deviations are about 14.7 (call) and
        // 8.7 (put); over 40 000 paths the standard errors are 0.074 and
        // 0.044, and the band is 5 of them.
        for (style, band) in [
            (OptionStyle::Call, dec!(0.37)),
            (OptionStyle::Put, dec!(0.22)),
        ] {
            let option = option_1y_atm(style);
            let price = telegraph(
                &option,
                optionstratlib_core::nz!(12),
                optionstratlib_core::nz!(40_000),
                Some(Decimal::ZERO),
                Some(Decimal::ZERO),
                regimes_20(),
                &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
            )
            .unwrap();
            let reference = black_scholes(&option).unwrap();
            assert!(
                (price - reference).abs() < band,
                "{style:?}: telegraph {price} vs Black-Scholes {reference}"
            );
        }
    }

    #[test]
    fn test_telegraph_dividend_yield_switching_disabled_converges_to_black_scholes() {
        use crate::pricing::black_scholes_model::black_scholes;
        // #756: with q = 3% the drift is r - q - sigma^2/2, so the estimate
        // is unbiased for the Black-Scholes price with the same q (8.6525
        // call, 5.7148 put). The payoff standard deviations are about 13.0
        // (call) and 8.7 (put); over 40 000 paths the standard errors are
        // 0.065 and 0.044, and the band is 5 of them. Before the fix the
        // call came out near the q = 0 price 10.45, about 28 standard
        // errors away.
        for (style, band) in [
            (OptionStyle::Call, dec!(0.33)),
            (OptionStyle::Put, dec!(0.22)),
        ] {
            let mut option = option_1y_atm(style);
            option.dividend_yield = pos_or_panic!(0.03);
            let price = telegraph(
                &option,
                optionstratlib_core::nz!(12),
                optionstratlib_core::nz!(40_000),
                Some(Decimal::ZERO),
                Some(Decimal::ZERO),
                regimes_20(),
                &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
            )
            .unwrap();
            let reference = black_scholes(&option).unwrap();
            assert!(
                (price - reference).abs() < band,
                "{style:?}: telegraph {price} vs Black-Scholes {reference}"
            );
        }
    }

    #[test]
    fn test_telegraph_dispersion_across_seeds_shrinks_with_paths() {
        // The spread of the estimate across seeds scales as 1 / sqrt(paths):
        // 16 times the paths should cut it by about 4. Asserting a factor of
        // 2 leaves room for the sampling error of a 32-seed standard
        // deviation.
        let option = option_1y_atm(OptionStyle::Call);
        let spread = |paths: NonZeroUsize| {
            let prices: Vec<f64> = (0..32_u64)
                .map(|seed| {
                    telegraph(
                        &option,
                        optionstratlib_core::nz!(8),
                        paths,
                        Some(dec!(0.5)),
                        Some(dec!(0.5)),
                        regimes_20(),
                        &mut deterministic_rng(seed),
                    )
                    .unwrap()
                    .to_f64()
                    .unwrap()
                })
                .collect();
            let n = prices.len() as f64;
            let mean = prices.iter().sum::<f64>() / n;
            (prices.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
        };
        let coarse = spread(optionstratlib_core::nz!(100));
        let fine = spread(optionstratlib_core::nz!(1_600));
        assert!(
            fine < coarse / 2.0,
            "spread with 1600 paths {fine} not well below spread with 100 paths {coarse}"
        );
    }

    #[test]
    fn test_telegraph_put_call_parity_on_same_seed() {
        // Same seed, same paths: C - P is the discounted mean of S_T - K,
        // an unbiased estimate of S - K e^{-rT} = 4.877 whatever the
        // regime rates and levels, because every step carries the
        // risk-neutral drift of its regime. With volatilities 0.3 / 0.1 S_T
        // has a standard deviation below 32, so over 40 000 paths the
        // standard error is below 0.16; the band is 3 of them.
        let price = |style| {
            telegraph(
                &option_1y_atm(style),
                optionstratlib_core::nz!(12),
                optionstratlib_core::nz!(40_000),
                Some(dec!(1.0)),
                Some(dec!(2.0)),
                regimes_split(),
                &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
            )
            .unwrap()
        };
        let forward_gap = dec!(100) - dec!(100) * (-dec!(0.05)).exp();
        let parity = price(OptionStyle::Call) - price(OptionStyle::Put);
        assert!(
            (parity - forward_gap).abs() < dec!(0.5),
            "C - P = {parity}, expected about {forward_gap}"
        );
    }

    /// Regimes at 30% (+1) and 10% (-1) volatility.
    fn regimes_split() -> RegimeVolatility {
        RegimeVolatility::new(pos_or_panic!(0.3), pos_or_panic!(0.1)).unwrap()
    }

    /// Black-Scholes price of `option` at volatility `sigma`.
    fn black_scholes_at(option: &Options, sigma: Positive) -> Decimal {
        let mut option = option.clone();
        option.implied_volatility = sigma;
        crate::pricing::black_scholes_model::black_scholes(&option).unwrap()
    }

    fn price_split(style: OptionStyle, lambda_up: Decimal, lambda_down: Decimal) -> Decimal {
        telegraph(
            &option_1y_atm(style),
            optionstratlib_core::nz!(12),
            optionstratlib_core::nz!(40_000),
            Some(lambda_up),
            Some(lambda_down),
            regimes_split(),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap()
    }

    #[test]
    fn test_telegraph_equal_regimes_with_switching_converges_to_black_scholes() {
        // Equal levels: switching (rates 1 and 2) leaves the law that of
        // GBM at 20%, so the estimate is unbiased for Black-Scholes
        // (10.4506 call, 5.5735 put). Bands are 5 standard errors over
        // 40 000 paths, as in the switching-disabled test.
        for (style, band) in [
            (OptionStyle::Call, dec!(0.37)),
            (OptionStyle::Put, dec!(0.22)),
        ] {
            let option = option_1y_atm(style);
            let price = telegraph(
                &option,
                optionstratlib_core::nz!(12),
                optionstratlib_core::nz!(40_000),
                Some(dec!(1.0)),
                Some(dec!(2.0)),
                regimes_20(),
                &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
            )
            .unwrap();
            let reference = black_scholes_at(&option, pos_or_panic!(0.2));
            assert!(
                (price - reference).abs() < band,
                "{style:?}: telegraph {price} vs Black-Scholes {reference}"
            );
        }
    }

    #[test]
    fn test_telegraph_frozen_regimes_match_black_scholes_mixture() {
        // Zero rates freeze each path in its initial regime, drawn +1 or -1
        // with equal probability, so the price is the closed-form mixture
        // (BS(0.3) + BS(0.1)) / 2 = (14.2313 + 6.8051) / 2 = 10.5182 for the
        // call. The payoff standard deviation is below 20, so over 40 000
        // paths the standard error is below 0.1; the band is 5 of them.
        let option = option_1y_atm(OptionStyle::Call);
        let price = price_split(OptionStyle::Call, Decimal::ZERO, Decimal::ZERO);
        let mixture = (black_scholes_at(&option, pos_or_panic!(0.3))
            + black_scholes_at(&option, pos_or_panic!(0.1)))
            / Decimal::TWO;
        assert!(
            (price - mixture).abs() < dec!(0.5),
            "telegraph {price} vs Black-Scholes mixture {mixture}"
        );
    }

    #[test]
    fn test_telegraph_more_time_in_high_vol_regime_raises_price() {
        // Stationary share of time in the +1 (30%) regime is
        // lambda_up / (lambda_up + lambda_down): 0.95 with (19, 1), 0.5 with
        // (1, 1), 0.05 with (1, 19). The prices are then close to BS at the
        // effective volatilities sqrt(0.95 * 0.09 + 0.05 * 0.01) = 0.293,
        // 0.224 and 0.116, about 13.9, 11.4 and 7.4: gaps of several units
        // against standard errors near 0.1.
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let mostly_high = price_split(style, dec!(19), dec!(1));
            let balanced = price_split(style, dec!(1), dec!(1));
            let mostly_low = price_split(style, dec!(1), dec!(19));
            assert!(
                mostly_high > balanced + Decimal::ONE && balanced > mostly_low + Decimal::ONE,
                "{style:?}: prices {mostly_high} / {balanced} / {mostly_low} not ordered by the time in the high-volatility regime"
            );
        }
    }

    #[test]
    fn test_telegraph_split_regimes_price_between_black_scholes_bounds() {
        // Conditional on its regime path the price is Black-Scholes at an
        // integrated variance between 0.01 and 0.09, so the expectation lies
        // between BS(0.1) and BS(0.3) (call 6.81 and 14.23, put 1.93 and
        // 9.35). The rates (1, 2) keep it well inside; the margin of 0.5
        // covers 5 standard errors.
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let option = option_1y_atm(style);
            let price = price_split(style, dec!(1), dec!(2));
            let low = black_scholes_at(&option, pos_or_panic!(0.1));
            let high = black_scholes_at(&option, pos_or_panic!(0.3));
            assert!(
                low + dec!(0.5) < price && price + dec!(0.5) < high,
                "{style:?}: telegraph {price} outside [{low}, {high}]"
            );
        }
    }
}
