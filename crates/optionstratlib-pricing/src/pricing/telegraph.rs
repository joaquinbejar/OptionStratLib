/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/8/24
******************************************************************************/
// Scoped allow: bulk migration of unchecked `[]` indexing to
// `.get().ok_or_else(..)` tracked as follow-ups to #341. The existing
// call sites are internal to this file and audited for invariant-bound
// indices (fixed-length buffers, just-pushed slices, etc.).
#![allow(clippy::indexing_slicing)]

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
//! In the context of financial options, the Telegraph Process can be used to model:
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

/// Prices an option using the Telegraph process simulation method.
///
/// The underlying follows a log-Euler geometric Brownian motion whose
/// diffusion term is signed by a two-state telegraph process. Each step of
/// length `dt = T / no_steps` first advances the regime `state` (flip
/// probability `1 - exp(-lambda * dt)`, `lambda` being the exit rate of the
/// current regime) and then updates the price as
///
/// ```text
/// S <- S * exp((r - sigma^2 / 2) * dt + sigma * state * sqrt(dt) * Z),  Z ~ N(0, 1)
/// ```
///
/// Every path draws its own initial regime (+1 or -1 with equal
/// probability). The price is the average of the discounted payoff over
/// `no_paths` paths, so it is a Monte-Carlo estimate of the
/// risk-neutral expectation rather than a single draw.
///
/// Because the shock is symmetric and independent of the regime path, the
/// sign the regime puts on the diffusion leaves the terminal law that of
/// geometric Brownian motion: the estimate converges to the Black-Scholes
/// price (without dividend yield) whatever the transition rates.
///
/// # Arguments
///
/// * `option` - Reference to the Options structure containing all option parameters
/// * `no_steps` - Number of time steps of every simulated path
/// * `no_paths` - Number of simulated paths the discounted payoff is averaged
///   over; [`TELEGRAPH_PATHS`] is the count the [`OptionPricing`] trait uses.
/// * `lambda_up` - Optional transition rate from down state (-1) to up state (+1)
/// * `lambda_down` - Optional transition rate from up state (+1) to down state (-1)
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
/// Returns `PricingError::ExpirationDate` when the option's
/// expiration cannot be converted, a decimal error when parameter
/// estimation produces degenerate rates, `PricingError::NonFinite` when a
/// simulated terminal price is not representable, and
/// `PricingError::MethodError` when an intermediate is not representable
/// as `f64` or the averaging overflows.
///
/// [`OptionPricing`]: crate::pricing::OptionPricing
#[tracing::instrument(skip(option, rng), level = "debug")]
pub fn telegraph<R: Rng + ?Sized>(
    option: &Options,
    no_steps: NonZeroUsize,
    no_paths: NonZeroUsize,
    lambda_up: Option<Decimal>,
    lambda_down: Option<Decimal>,
    rng: &mut R,
) -> Result<Decimal, PricingError> {
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

    // Loop-invariant risk-neutral drift `r - σ²/2`.
    let volatility = option.implied_volatility.to_dec();
    let drift: Decimal = d_sub(
        option.risk_free_rate,
        d_mul(
            dec!(0.5),
            d_powd(volatility, Decimal::TWO, "pricing::telegraph::variance")?,
            "pricing::telegraph::half_variance",
        )?,
        "pricing::telegraph::drift",
    )?;
    let drift_dt = d_mul(drift, dt, "pricing::telegraph::drift_dt")?;
    let sqrt_dt = d_sqrt(dt, "pricing::telegraph::sqrt_dt")
        .map_err(|_| PricingError::method_error("telegraph", "non-finite dt sqrt"))?;
    let diffusion = d_mul(volatility, sqrt_dt, "pricing::telegraph::diffusion")?;
    let drift_dt_f64 = drift_dt.to_f64().ok_or_else(|| {
        PricingError::method_error("telegraph", "drift * dt not representable as f64")
    })?;
    let diffusion_f64 = diffusion.to_f64().ok_or_else(|| {
        PricingError::method_error("telegraph", "sigma * sqrt(dt) not representable as f64")
    })?;

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
            let shock = if state_up { z } else { -z };
            log_return += drift_dt_f64 + diffusion_f64 * shock;
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
            &mut rng,
        );
        let _price_down = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            None,
            Some(dec!(0.5)),
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
            &mut rng,
        );
        let _price_1000 = telegraph(
            &option,
            optionstratlib_core::nz!(1000),
            optionstratlib_core::nz!(1_000),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            &mut rng,
        );

        // assert!(price_100 > 0.0);
        // assert!(price_1000 > 0.0);
        // assert_ne!(price_100, price_1000);
    }

    #[test]
    fn test_telegraph_zero_volatility() {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        let mut option = create_mock_option();
        option.implied_volatility = Positive::ZERO;
        let _price = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            &mut rng,
        );
        // assert_relative_eq!(price, 0.0, epsilon = 1e-6);
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

    /// `telegraph(&option_30d(), 100, TELEGRAPH_PATHS, Some(0.5), Some(0.5))`
    /// drawn from
    /// `deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED)`. Re-baselined by
    /// #743: the kernel now averages `TELEGRAPH_PATHS` paths driven by a
    /// standard normal shock instead of returning one path driven by a
    /// positive uniform.
    const PINNED_TELEGRAPH_PRICE: Decimal = dec!(40.208194873575235296394764785);

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
                &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
            )
            .unwrap();
        let via_kernel = telegraph(
            &option,
            optionstratlib_core::nz!(100),
            TELEGRAPH_PATHS,
            None,
            None,
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
        // regime rates. S_T has a standard deviation of about 20.3, so over
        // 40 000 paths the standard error is 0.10; the band is 5 of them.
        let price = |style| {
            telegraph(
                &option_1y_atm(style),
                optionstratlib_core::nz!(12),
                optionstratlib_core::nz!(40_000),
                Some(dec!(1.0)),
                Some(dec!(2.0)),
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
}
