//! Monte Carlo Estimator for Funding Networks
//!
//! A funding network ([`crate::pricing::funding_network`]) whose weights are
//! not negative, `A_ij ≤ 0` off the diagonal and `A·1 ≥ 0`, prices each
//! contract as the expected payoff at a random maturity. That maturity is the
//! absorption time of a Markov chain on the contracts: from contract `i` the
//! chain waits an exponential time of rate `A_ii`, then stops with
//! probability `(A·1)_i/A_ii` or moves to contract `j` with probability
//! `−A_ij/A_ii`. The price of contract `i` is the mean of `φ(S_τ)` over
//! chains started at `i`.
//!
//! Both the maturity and the underlying at the maturity are sampled from
//! their exact laws under the models of [`LevyModel`], so the estimator has
//! no discretisation bias: its only error is the standard error it reports.
//! It shares no code path with the Fourier inversion of
//! [`funding_network_prices`].
//!
//! # Discrete Funding
//!
//! With [`FundingSchedule::Discrete`] funding is paid every `Δ` years, each
//! payment being `Δ·A·(F − φ(S)·1)` on the prices at the payment time. The
//! price just after a payment is then
//!
//! ```text
//! F = Σ_{k≥1} (I − Δ·A)^{k−1}·Δ·A·1·E[φ(S_{kΔ})]
//! ```
//!
//! which is the same chain observed at the payment times: it holds contract
//! `i` for a geometric number of periods, leaving at each one with
//! probability `Δ·A_ii`. That needs `Δ·A_ii ≤ 1` for every contract. For a
//! single contract of rate `κ` the weights are `κΔ·(1 − κΔ)^{k−1}`, and with
//! `κΔ = 1` the contract is a European option of maturity `Δ` that rolls at
//! every payment.
//!
//! # Example
//!
//! ```rust
//! use optionstratlib_core::model::Positive;
//! use optionstratlib_core::model::types::OptionStyle;
//! use optionstratlib_core::utils::deterministic_rng;
//! use optionstratlib_pricing::pricing::{
//!     FundingNetwork, FundingNetworkParams, FundingSchedule, LevyModel,
//!     funding_network_prices, funding_network_prices_monte_carlo,
//! };
//! use rust_decimal_macros::dec;
//! use std::num::{NonZeroU32, NonZeroUsize};
//!
//! // A ladder of two rungs with a funding rate of 12 per year.
//! let network = FundingNetwork::ladder(
//!     Positive::new_decimal(dec!(12))?,
//!     NonZeroU32::MIN.saturating_add(1),
//! )?;
//! let params = FundingNetworkParams::try_new(
//!     Positive::HUNDRED,
//!     Positive::HUNDRED,
//!     dec!(0),
//!     LevyModel::BlackScholes {
//!         volatility: Positive::new_decimal(dec!(0.5))?,
//!     },
//!     OptionStyle::Call,
//! )?;
//! let exact = funding_network_prices(&network, &params)?;
//! let estimates = funding_network_prices_monte_carlo(
//!     &network,
//!     &params,
//!     FundingSchedule::Continuous,
//!     NonZeroUsize::MIN.saturating_add(49_999),
//!     &mut deterministic_rng(7),
//! )?;
//! for (estimate, price) in estimates.iter().zip(exact.iter()) {
//!     let error = (estimate.value() - *price).abs();
//!     assert!(error < dec!(5) * estimate.standard_error());
//! }
//! # Ok::<(), optionstratlib_pricing::error::PricingError>(())
//! ```

use crate::error::PricingError;
use crate::pricing::funding_network::{
    FundingNetwork, FundingNetworkParams, FundingSchedule, ModelKernel, exp_f64, ln_f64,
    reject_zero, sqrt_f64, to_decimal,
};
use optionstratlib_core::model::decimal::decimal_to_f64;
use optionstratlib_core::model::types::OptionStyle;
use rand::Rng;
use rand_distr::{Distribution, Exp1, OpenClosed01, Poisson, StandardNormal};
use rust_decimal::Decimal;
use std::num::NonZeroUsize;

/// Method name carried by the errors of this module.
const METHOD: &str = "funding_network_monte_carlo";

/// Transitions after which a path that has not reached its maturity is
/// reported as an error. A stable network reaches it with probability one.
const MAX_TRANSITIONS: u32 = 1_000_000;

/// Slack on `Δ·A_ii ≤ 1`, for a product that is one up to rounding.
const INTERVAL_SLACK: f64 = 1e-12;

/// Monte Carlo estimate of the price of one contract of a funding network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FundingNetworkEstimate {
    value: Decimal,
    standard_error: Decimal,
}

impl FundingNetworkEstimate {
    /// Mean of the sampled payoffs: the estimate of the price, per unit of
    /// underlying.
    #[must_use]
    #[inline]
    pub const fn value(&self) -> Decimal {
        self.value
    }

    /// Standard error of [`FundingNetworkEstimate::value`]: the sample
    /// standard deviation of the payoffs over the square root of the number
    /// of paths. Zero with a single path.
    #[must_use]
    #[inline]
    pub const fn standard_error(&self) -> Decimal {
        self.standard_error
    }
}

/// Monte Carlo prices of every contract of a funding network, per unit of
/// underlying, in the order of the rows of the funding matrix.
///
/// Each contract takes `paths` independent paths. A path draws the maturity
/// of the contract and then the underlying at that maturity, both from their
/// exact laws, so the estimate is unbiased whatever the number of paths.
///
/// # Arguments
///
/// * `network` - The funding network; its weights must not be negative
///   ([`FundingNetwork::is_non_negative`]).
/// * `params` - Underlying price, strike, drift, model and payoff.
/// * `schedule` - Continuous funding, or funding paid at a fixed interval.
///   With an interval the value is the price just after a payment.
/// * `paths` - Paths per contract.
/// * `rng` - Random number generator. A seeded generator such as
///   [`optionstratlib_core::utils::deterministic_rng`] makes the estimates
///   reproducible.
///
/// # Errors
///
/// - [`PricingError::InvalidParameter`] when the network has a positive
///   entry off the diagonal or a negative row sum (its weights are not a
///   maturity law), when the drift is not below the decay rate of the
///   network, when the interval of a discrete schedule is zero, or when it
///   exceeds one over the largest diagonal entry of the funding matrix.
/// - [`PricingError::MethodError`] when the number of jumps of a path cannot
///   be sampled, or a path does not reach its maturity.
/// - [`PricingError::Decimal`] when an input has no `f64` form.
/// - [`PricingError::NonFinite`] when an estimate has no `Decimal` form.
pub fn funding_network_prices_monte_carlo<R: Rng + ?Sized>(
    network: &FundingNetwork,
    params: &FundingNetworkParams,
    schedule: FundingSchedule,
    paths: NonZeroUsize,
    rng: &mut R,
) -> Result<Vec<FundingNetworkEstimate>, PricingError> {
    require_maturity_law(network)?;
    let clock = Clock::new(network, schedule)?;
    let spot = decimal_to_f64(params.underlying_price().to_dec())?;
    let strike = decimal_to_f64(params.strike_price().to_dec())?;
    let drift = decimal_to_f64(params.drift())?;
    if drift >= network.scheduled_decay(schedule)? {
        return Err(PricingError::invalid_parameter(
            "drift",
            params.drift(),
            "must be below the decay rate of the network",
        ));
    }
    let kernel = ModelKernel::new(&params.model(), drift)?;
    let style = params.option_style();

    let mut estimates = Vec::with_capacity(network.order());
    for contract in 0..network.order() {
        let mut count = 0.0_f64;
        let mut sum = 0.0_f64;
        let mut squares = 0.0_f64;
        for _ in 0..paths.get() {
            let maturity = sample_maturity(network, contract, clock, rng)?;
            let terminal = sample_terminal(&kernel, spot, maturity, rng)?;
            let payoff = match style {
                OptionStyle::Call => (terminal - strike).max(0.0),
                OptionStyle::Put => (strike - terminal).max(0.0),
            };
            count += 1.0;
            sum += payoff;
            squares += payoff * payoff;
        }
        let mean = sum / count;
        let variance = if count > 1.0 {
            ((squares - count * mean * mean) / (count - 1.0)).max(0.0)
        } else {
            0.0
        };
        estimates.push(FundingNetworkEstimate {
            value: to_decimal("pricing::funding_network_monte_carlo::value", mean)?,
            standard_error: to_decimal(
                "pricing::funding_network_monte_carlo::standard_error",
                sqrt_f64(variance / count),
            )?,
        });
    }
    Ok(estimates)
}

/// Rejects a network whose weights are not a maturity law, naming the entry
/// or the row sum that breaks it.
fn require_maturity_law(network: &FundingNetwork) -> Result<(), PricingError> {
    for (row_index, row) in network.rows().iter().enumerate() {
        for (column_index, weight) in row.iter().enumerate() {
            if column_index != row_index && *weight > 0.0 {
                return Err(PricingError::invalid_parameter(
                    "funding_matrix",
                    network.entry(row_index, column_index).unwrap_or_default(),
                    "Monte Carlo needs no positive entry off the diagonal",
                ));
            }
        }
    }
    for exit in network.anchor() {
        if *exit < 0.0 {
            return Err(PricingError::invalid_parameter(
                "funding_matrix",
                to_decimal("pricing::funding_network_monte_carlo::row_sum", *exit)?,
                "Monte Carlo needs no negative row sum",
            ));
        }
    }
    if !network.is_non_negative() {
        return Err(PricingError::invalid_parameter(
            "funding_matrix",
            Decimal::ZERO,
            "Monte Carlo needs funding weights that are not negative",
        ));
    }
    Ok(())
}

/// How long the chain holds a contract.
#[derive(Debug, Clone, Copy)]
enum Clock {
    /// An exponential time of the rate of the contract.
    Continuous,
    /// A geometric number of periods of `interval` years.
    Discrete { interval: f64 },
}

impl Clock {
    /// Validates the schedule against the rates of the network.
    fn new(network: &FundingNetwork, schedule: FundingSchedule) -> Result<Self, PricingError> {
        let (interval, years) = match schedule {
            FundingSchedule::Continuous => return Ok(Self::Continuous),
            FundingSchedule::Discrete { interval } => {
                reject_zero("interval", interval)?;
                (interval, decimal_to_f64(interval.to_dec())?)
            }
        };
        for (index, row) in network.rows().iter().enumerate() {
            let rate = row.get(index).copied().unwrap_or(0.0);
            if years * rate > 1.0 + INTERVAL_SLACK {
                return Err(PricingError::invalid_parameter(
                    "interval",
                    interval.to_dec(),
                    "exceeds one over the largest diagonal entry of the funding matrix",
                ));
            }
        }
        Ok(Self::Discrete { interval: years })
    }

    /// Time the chain spends on a contract of rate `rate` before it leaves.
    fn holding<R: Rng + ?Sized>(self, rate: f64, rng: &mut R) -> f64 {
        match self {
            Self::Continuous => {
                let unit: f64 = Exp1.sample(rng);
                unit / rate
            }
            Self::Discrete { interval } => {
                let leave = interval * rate;
                if leave >= 1.0 {
                    return interval;
                }
                // Periods survived before the one in which the chain leaves:
                // geometric, by inversion of its distribution function.
                let uniform: f64 = OpenClosed01.sample(rng);
                let survived = (ln_f64(uniform) / (-leave).ln_1p()).floor();
                interval * (survived + 1.0)
            }
        }
    }
}

/// The error of a contract index outside the network.
fn missing_contract() -> PricingError {
    PricingError::method_error(METHOD, "the chain left the contracts of the network")
}

/// One draw of the maturity of contract `start`: the absorption time of the
/// chain started there.
fn sample_maturity<R: Rng + ?Sized>(
    network: &FundingNetwork,
    start: usize,
    clock: Clock,
    rng: &mut R,
) -> Result<f64, PricingError> {
    let mut state = start;
    let mut time = 0.0_f64;
    for _ in 0..MAX_TRANSITIONS {
        let row = network.rows().get(state).ok_or_else(missing_contract)?;
        let rate = row.get(state).copied().ok_or_else(missing_contract)?;
        let exit = network
            .anchor()
            .get(state)
            .copied()
            .ok_or_else(missing_contract)?;
        if rate <= 0.0 {
            return Err(PricingError::method_error(
                METHOD,
                "a contract of the network has no funding rate of its own",
            ));
        }
        time += clock.holding(rate, rng);
        // A point of the total rate: first the mass of the payoff, then the
        // mass of every other contract.
        let uniform: f64 = OpenClosed01.sample(rng);
        let mut mark = uniform * rate - exit;
        if mark <= 0.0 {
            return Ok(time);
        }
        let mut next = None;
        for (column, weight) in row.iter().enumerate() {
            if column == state {
                continue;
            }
            // `weight ≤ 0`: its mass is `−weight`.
            mark += weight;
            if mark <= 0.0 {
                next = Some(column);
                break;
            }
        }
        match next {
            Some(column) => state = column,
            // The masses add up to the rate except for rounding; what is
            // left belongs to the payoff.
            None => return Ok(time),
        }
    }
    Err(PricingError::method_error(
        METHOD,
        "a path did not reach its maturity",
    ))
}

/// One draw of the underlying price after `maturity` years.
fn sample_terminal<R: Rng + ?Sized>(
    kernel: &ModelKernel,
    spot: f64,
    maturity: f64,
    rng: &mut R,
) -> Result<f64, PricingError> {
    let normal: f64 = StandardNormal.sample(rng);
    let mut log_return = kernel.location * maturity + sqrt_f64(kernel.variance * maturity) * normal;
    let expected_jumps = kernel.intensity * maturity;
    if expected_jumps > 0.0 {
        let jumps: f64 = Poisson::new(expected_jumps)
            .map_err(|e| PricingError::method_error(METHOD, &format!("number of jumps: {e}")))?
            .sample(rng);
        if jumps > 0.0 {
            // The sum of `jumps` independent normal log-jumps.
            let normal: f64 = StandardNormal.sample(rng);
            log_return +=
                jumps * kernel.jump_mean + sqrt_f64(jumps * kernel.jump_variance) * normal;
        }
    }
    Ok(spot * exp_f64(log_return))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pricing::everlasting::{EverlastingParams, everlasting_price};
    use crate::pricing::funding_network::{
        LevyModel, funding_network_prices, funding_network_prices_with,
    };
    use optionstratlib_core::model::Positive;
    use optionstratlib_core::pos_or_panic;
    use optionstratlib_core::utils::deterministic_rng;
    use rust_decimal::prelude::ToPrimitive;
    use rust_decimal_macros::dec;
    use statrs::distribution::{ContinuousCDF, Normal};
    use std::num::NonZeroU32;

    const MONTH: f64 = 30.0 / 365.0;
    const SIGMA: f64 = 0.5;
    const PATHS: usize = 200_000;

    fn decimal(value: f64) -> Decimal {
        Decimal::try_from(value).unwrap()
    }

    fn network(rows: &[&[f64]]) -> FundingNetwork {
        let rows: Vec<Vec<Decimal>> = rows
            .iter()
            .map(|row| row.iter().map(|value| decimal(*value)).collect())
            .collect();
        FundingNetwork::try_new(&rows).unwrap()
    }

    fn ladder(rate: f64, rungs: u32) -> FundingNetwork {
        FundingNetwork::ladder(pos_or_panic!(rate), NonZeroU32::new(rungs).unwrap()).unwrap()
    }

    fn black_scholes() -> LevyModel {
        LevyModel::BlackScholes {
            volatility: pos_or_panic!(SIGMA),
        }
    }

    fn merton() -> LevyModel {
        LevyModel::Merton {
            volatility: pos_or_panic!(SIGMA),
            jump_intensity: pos_or_panic!(20.0),
            jump_mean: dec!(-0.05),
            jump_volatility: pos_or_panic!(0.10),
        }
    }

    fn params(strike: f64, mu: f64, model: LevyModel, style: OptionStyle) -> FundingNetworkParams {
        FundingNetworkParams::try_new(
            Positive::HUNDRED,
            pos_or_panic!(strike),
            decimal(mu),
            model,
            style,
        )
        .unwrap()
    }

    fn estimate(
        network: &FundingNetwork,
        params: &FundingNetworkParams,
        schedule: FundingSchedule,
        seed: u64,
    ) -> Vec<(f64, f64)> {
        funding_network_prices_monte_carlo(
            network,
            params,
            schedule,
            NonZeroUsize::new(PATHS).unwrap(),
            &mut deterministic_rng(seed),
        )
        .unwrap()
        .iter()
        .map(|e| {
            (
                e.value().to_f64().unwrap(),
                e.standard_error().to_f64().unwrap(),
            )
        })
        .collect()
    }

    /// Largest distance between estimate and reference, in standard errors.
    fn worst_deviation(estimates: &[(f64, f64)], reference: &[f64]) -> f64 {
        assert_eq!(estimates.len(), reference.len());
        estimates
            .iter()
            .zip(reference.iter())
            .map(|((value, error), exact)| {
                assert!(*error > 0.0);
                // A standard error of a few parts in a thousand of the price.
                assert!(*error < 0.02 * exact.abs());
                ((value - exact) / error).abs()
            })
            .fold(0.0, f64::max)
    }

    fn fourier(network: &FundingNetwork, params: &FundingNetworkParams) -> Vec<f64> {
        funding_network_prices(network, params)
            .unwrap()
            .iter()
            .map(|value| value.to_f64().unwrap())
            .collect()
    }

    /// Undiscounted Black-Scholes call at maturity `years`, with drift `mu`.
    fn european_call(strike: f64, mu: f64, years: f64) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let root = SIGMA * years.sqrt();
        let d1 = ((100.0 / strike).ln() + (mu + 0.5 * SIGMA * SIGMA) * years) / root;
        100.0 * (mu * years).exp() * normal.cdf(d1) - strike * normal.cdf(d1 - root)
    }

    #[test]
    fn test_funding_network_prices_monte_carlo_ladder_matches_closed_form() {
        let rate = 3.0 / MONTH;
        let network = ladder(rate, 3);
        for (strike, mu, style, seed) in [
            (100.0, 0.0, OptionStyle::Call, 11_u64),
            (110.0, 0.05, OptionStyle::Call, 12),
            (95.0, 0.0, OptionStyle::Put, 13),
        ] {
            let reference: Vec<f64> = (1..=3_u32)
                .map(|rung| {
                    let closed = EverlastingParams::try_new(
                        Positive::HUNDRED,
                        pos_or_panic!(strike),
                        pos_or_panic!(SIGMA),
                        decimal(mu),
                        pos_or_panic!(rate),
                        NonZeroU32::new(rung).unwrap(),
                        style,
                    )
                    .unwrap();
                    everlasting_price(&closed).unwrap().to_f64().unwrap()
                })
                .collect();
            let estimates = estimate(
                &network,
                &params(strike, mu, black_scholes(), style),
                FundingSchedule::Continuous,
                seed,
            );
            let worst = worst_deviation(&estimates, &reference);
            assert!(worst < 4.0, "strike {strike}: {worst} standard errors");
        }
    }

    #[test]
    fn test_funding_network_prices_monte_carlo_merton_matches_fourier() {
        let network = ladder(2.0 / MONTH, 2);
        let params = params(105.0, 0.0, merton(), OptionStyle::Call);
        let estimates = estimate(&network, &params, FundingSchedule::Continuous, 21);
        let worst = worst_deviation(&estimates, &fourier(&network, &params));
        assert!(worst < 4.0, "{worst} standard errors");
    }

    #[test]
    fn test_funding_network_prices_monte_carlo_cross_funded_network_matches_fourier() {
        // Every contract is funded against the payoff and against the others.
        let network = network(&[
            &[30.0, -10.0, -5.0],
            &[-8.0, 20.0, -4.0],
            &[-2.0, -6.0, 12.0],
        ]);
        assert!(network.is_non_negative());
        let params = params(100.0, 0.02, black_scholes(), OptionStyle::Put);
        let estimates = estimate(&network, &params, FundingSchedule::Continuous, 31);
        let worst = worst_deviation(&estimates, &fourier(&network, &params));
        assert!(worst < 4.0, "{worst} standard errors");
    }

    #[test]
    fn test_funding_network_prices_monte_carlo_discrete_matches_geometric_sum() {
        // Daily funding on a contract with a mean maturity of 30 days.
        let rate = 1.0 / MONTH;
        let interval = 1.0 / 365.0;
        let leave = rate * interval;
        for (strike, mu, seed) in [(100.0, 0.0, 41_u64), (108.0, 0.05, 42)] {
            let mut reference = 0.0;
            let mut weight = leave;
            for period in 1..=2_000_u32 {
                reference += weight * european_call(strike, mu, f64::from(period) * interval);
                weight *= 1.0 - leave;
            }
            let estimates = estimate(
                &ladder(rate, 1),
                &params(strike, mu, black_scholes(), OptionStyle::Call),
                FundingSchedule::Discrete {
                    interval: pos_or_panic!(interval),
                },
                seed,
            );
            let worst = worst_deviation(&estimates, &[reference]);
            assert!(worst < 4.0, "strike {strike}: {worst} standard errors");
        }
    }

    #[test]
    fn test_funding_network_prices_monte_carlo_discrete_second_rung_matches_negative_binomial_sum()
    {
        // Rung 2 leaves after two successes: `(k − 1)·p²·(1 − p)^{k−2}`.
        let rate = 2.0 / MONTH;
        let interval = 3.0 / 365.0;
        let leave = rate * interval;
        let mut reference = 0.0;
        let mut power = 1.0;
        for period in 2..=1_000_u32 {
            let weight = f64::from(period - 1) * leave * leave * power;
            reference += weight * european_call(100.0, 0.0, f64::from(period) * interval);
            power *= 1.0 - leave;
        }
        let estimates = estimate(
            &ladder(rate, 2),
            &params(100.0, 0.0, black_scholes(), OptionStyle::Call),
            FundingSchedule::Discrete {
                interval: pos_or_panic!(interval),
            },
            51,
        );
        let second = estimates.get(1).copied().unwrap();
        let worst = worst_deviation(&[second], &[reference]);
        assert!(worst < 4.0, "{worst} standard errors");
    }

    #[test]
    fn test_funding_network_prices_monte_carlo_discrete_merton_matches_fourier() {
        let network = network(&[&[30.0, -10.0], &[-15.0, 25.0]]);
        let schedule = FundingSchedule::Discrete {
            interval: pos_or_panic!(1.0 / 365.0),
        };
        let params = params(103.0, 0.0, merton(), OptionStyle::Call);
        let reference: Vec<f64> = funding_network_prices_with(&network, &params, schedule)
            .unwrap()
            .iter()
            .map(|value| value.to_f64().unwrap())
            .collect();
        let estimates = estimate(&network, &params, schedule, 71);
        let worst = worst_deviation(&estimates, &reference);
        assert!(worst < 4.0, "{worst} standard errors");
    }

    #[test]
    fn test_funding_network_prices_monte_carlo_full_funding_is_a_european_option() {
        // `interval·rate = 1`: the whole gap is paid at every payment, so
        // the contract is the option that expires at the next one.
        let estimates = estimate(
            &ladder(12.0, 1),
            &params(102.0, 0.0, black_scholes(), OptionStyle::Call),
            FundingSchedule::Discrete {
                interval: Positive::ONE / pos_or_panic!(12.0),
            },
            61,
        );
        let worst = worst_deviation(&estimates, &[european_call(102.0, 0.0, 1.0 / 12.0)]);
        assert!(worst < 4.0, "{worst} standard errors");
    }

    #[test]
    fn test_funding_network_prices_monte_carlo_same_seed_gives_identical_estimates() {
        let network = ladder(24.0, 2);
        let params = params(100.0, 0.0, merton(), OptionStyle::Call);
        let run = |seed: u64| {
            funding_network_prices_monte_carlo(
                &network,
                &params,
                FundingSchedule::Continuous,
                NonZeroUsize::new(2_000).unwrap(),
                &mut deterministic_rng(seed),
            )
            .unwrap()
        };
        assert_eq!(run(5), run(5));
        assert_ne!(run(5), run(6));
    }

    #[test]
    fn test_funding_network_prices_monte_carlo_single_path_has_zero_standard_error() {
        let estimates = funding_network_prices_monte_carlo(
            &ladder(12.0, 1),
            &params(100.0, 0.0, black_scholes(), OptionStyle::Call),
            FundingSchedule::Continuous,
            NonZeroUsize::MIN,
            &mut deterministic_rng(1),
        )
        .unwrap();
        assert_eq!(estimates.len(), 1);
        assert_eq!(estimates.first().unwrap().standard_error(), Decimal::ZERO);
        assert!(estimates.first().unwrap().value() >= Decimal::ZERO);
    }

    #[test]
    fn test_funding_network_prices_monte_carlo_rejects_inputs_outside_the_domain() {
        let run = |network: &FundingNetwork, mu: f64, schedule: FundingSchedule| {
            funding_network_prices_monte_carlo(
                network,
                &params(100.0, mu, black_scholes(), OptionStyle::Call),
                schedule,
                NonZeroUsize::new(10).unwrap(),
                &mut deterministic_rng(1),
            )
        };
        let plain = ladder(12.0, 2);
        // A positive entry off the diagonal.
        let signed = network(&[&[10.0, 2.0], &[-3.0, 10.0]]);
        assert!(matches!(
            run(&signed, 0.0, FundingSchedule::Continuous),
            Err(PricingError::InvalidParameter {
                parameter: "funding_matrix",
                ..
            })
        ));
        // A negative row sum.
        let overdrawn = network(&[&[10.0, -12.0], &[0.0, 10.0]]);
        assert!(matches!(
            run(&overdrawn, 0.0, FundingSchedule::Continuous),
            Err(PricingError::InvalidParameter {
                parameter: "funding_matrix",
                ..
            })
        ));
        // A drift at the decay rate.
        assert!(matches!(
            run(&plain, 12.0, FundingSchedule::Continuous),
            Err(PricingError::InvalidParameter {
                parameter: "drift",
                ..
            })
        ));
        // An interval of zero, and one longer than the funding rate allows.
        assert!(matches!(
            run(
                &plain,
                0.0,
                FundingSchedule::Discrete {
                    interval: Positive::ZERO
                }
            ),
            Err(PricingError::InvalidParameter {
                parameter: "interval",
                ..
            })
        ));
        assert!(matches!(
            run(
                &plain,
                0.0,
                FundingSchedule::Discrete {
                    interval: pos_or_panic!(0.1)
                }
            ),
            Err(PricingError::InvalidParameter {
                parameter: "interval",
                ..
            })
        ));
    }
}
