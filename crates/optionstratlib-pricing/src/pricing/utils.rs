/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/8/24
******************************************************************************/

use crate::error::PricingError;
use crate::kernels::{big_n, d2, discount_factor};
use crate::pricing::binomial_model::BinomialPricingParams;
use crate::pricing::constants::{CLAMP_MAX, CLAMP_MIN};
use optionstratlib_core::error::DecimalError;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{
    d_add, d_div, d_exp, d_ln, d_mul, d_powd, d_sqrt, d_sub, finite_decimal,
};
use optionstratlib_core::model::payoff::{Payoff, PayoffInfo};
use optionstratlib_core::model::types::Side;
use optionstratlib_core::utils::random_decimal;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;

/// Simulates stock returns based on a normal distribution using pure decimal arithmetic.
///
/// # Arguments
///
/// * `mean` - The mean return (annualized)
/// * `std_dev` - The standard deviation of returns (annualized)
/// * `length` - The number of returns to simulate
/// * `time_step` - The time step for each return (e.g., 1/252 for daily returns assuming 252 trading days)
/// * `rng` - The generator every uniform draw of the Box-Muller transform is
///   taken from. A seeded generator such as
///   [`optionstratlib_core::utils::deterministic_rng`] makes the returns
///   reproducible; pass `&mut rand::rng()` to draw from the thread-local RNG.
///
/// # Returns
///
/// A Result containing either:
/// - Ok(`Vec<Decimal>`): A vector of simulated returns as Decimal numbers
/// - Err(DecimalError): If there's an error in decimal calculations
///
/// # Errors
///
/// Returns [`DecimalError::ConversionError`] when the sampled normal
/// variate cannot be represented as a `Decimal` (e.g. NaN or out-of-range
/// float), and [`DecimalError::ArithmeticError`] when the
/// `mean + std_dev * z` combination overflows the `Decimal` range or
/// `length` returns cannot be allocated.
pub fn simulate_returns<R: Rng + ?Sized>(
    mean: Decimal,
    std_dev: Positive,
    length: usize,
    time_step: Decimal,
    rng: &mut R,
) -> Result<Vec<Decimal>, DecimalError> {
    /// Generates a pair of normally distributed random numbers using Box-Muller transform
    fn generate_normal_pair<R: Rng + ?Sized>(
        rng: &mut R,
    ) -> Result<(Decimal, Decimal), DecimalError> {
        // Generate two uniform random numbers between 0 and 1
        let u1 = random_decimal(rng)?;
        let u2 = random_decimal(rng)?;

        // Convert to normal distribution using Box-Muller transform
        let r = d_sqrt(
            d_mul(
                -Decimal::TWO,
                d_ln(u1, "pricing::utils::box_muller::log_u1")?,
                "pricing::utils::box_muller::radius_squared",
            )?,
            "pricing::utils::box_muller::radius",
        )
        .map_err(|_| {
            DecimalError::arithmetic_error("sqrt", "non-finite operand in Box-Muller r")
        })?;
        let theta = d_mul(
            d_mul(
                Decimal::TWO,
                Decimal::PI,
                "pricing::utils::box_muller::two_pi",
            )?,
            u2,
            "pricing::utils::box_muller::theta",
        )?;

        // `theta` lies in `[0, 2π)`, where the series converge; the checked
        // forms report rather than abort if that ever stops holding (#788).
        let trig = || {
            DecimalError::arithmetic_error(
                "pricing::utils::box_muller",
                "trigonometric series overflowed",
            )
        };
        let x1 = d_mul(
            r,
            theta.checked_cos().ok_or_else(trig)?,
            "pricing::utils::box_muller::x1",
        )?;
        let x2 = d_mul(
            r,
            theta.checked_sin().ok_or_else(trig)?,
            "pricing::utils::box_muller::x2",
        )?;

        Ok((x1, x2))
    }

    if std_dev < Decimal::ZERO {
        return Err(DecimalError::InvalidValue {
            value: std_dev.to_f64(),
            reason: "Standard deviation cannot be negative".to_string(),
        });
    }

    // Adjust mean and standard deviation for the time step
    let adjusted_mean = d_mul(mean, time_step, "pricing::utils::simulate::adjusted_mean")?;
    let adjusted_std = d_mul(
        std_dev.to_dec(),
        d_sqrt(time_step, "pricing::utils::simulate::sqrt_time_step").map_err(|_| {
            DecimalError::arithmetic_error("sqrt", "invalid (negative or non-finite) time_step")
        })?,
        "pricing::utils::simulate::adjusted_std",
    )?;

    // Reserved fallibly: `vec![..; length]` and `Vec::with_capacity(length)`
    // aborted with `capacity overflow` on a length wider than the address
    // space (#788), and a refused allocation is reported the same way.
    let mut returns: Vec<Decimal> = Vec::new();
    returns.try_reserve_exact(length).map_err(|_| {
        DecimalError::arithmetic_error(
            "pricing::utils::simulate_returns",
            &format!("cannot allocate {length} returns"),
        )
    })?;

    // Special case: if std_dev is 0, return a vector of constant values
    if adjusted_std == Decimal::ZERO {
        returns.resize(length, adjusted_mean);
        return Ok(returns);
    }

    // Generate pairs of normally distributed random numbers using Box-Muller transform
    for _ in 0..length.div_ceil(2) {
        let (n1, n2) = generate_normal_pair(rng)?;

        // Scale the random numbers by mean and std_dev
        let r1 = d_add(
            d_mul(n1, adjusted_std, "pricing::utils::simulate::scale_1")?,
            adjusted_mean,
            "pricing::utils::simulate::shift_1",
        )?;
        returns.push(r1);

        if returns.len() < length {
            let r2 = d_add(
                d_mul(n2, adjusted_std, "pricing::utils::simulate::scale_2")?,
                adjusted_mean,
                "pricing::utils::simulate::shift_2",
            )?;
            returns.push(r2);
        }
    }

    Ok(returns)
}

/// Calculates the up factor for an asset's price movement model.
///
/// # Arguments
///
/// * `volatility` - The volatility of the asset, represented as a floating point number.
/// * `dt` - The time increment for the model, typically represented in years as a floating point number.
///
/// # Returns
///
/// * A floating point number representing the up factor calculated based on the given volatility and time increment.
///
#[inline]
pub(crate) fn calculate_up_factor(
    volatility: Positive,
    dt: Decimal,
) -> Result<Decimal, DecimalError> {
    let sqrt_dt = d_sqrt(dt, "pricing::utils::up_factor::sqrt_dt")
        .map_err(|_| DecimalError::arithmetic_error("sqrt", "non-finite dt in up factor"))?;
    d_exp(
        d_mul(
            sqrt_dt,
            volatility.to_dec(),
            "pricing::binomial::up_factor::exponent",
        )?,
        "pricing::binomial::up_factor",
    )
}

/// Calculates the down factor for a given volatility and time step.
///
/// # Parameters
/// - `volatility`: The volatility of the asset, typically represented by a
///   non-negative floating-point number.
/// - `dt`: The time step size, given as a floating-point number, representing
///   the discrete length of time over which the calculation is to be performed.
///
/// # Returns
/// A floating-point number representing the down factor, calculated using the
/// given volatility and time step.
///
#[inline]
pub(crate) fn calculate_down_factor(
    volatility: Positive,
    dt: Decimal,
) -> Result<Decimal, DecimalError> {
    let sqrt_dt = d_sqrt(dt, "pricing::utils::down_factor::sqrt_dt")
        .map_err(|_| DecimalError::arithmetic_error("sqrt", "non-finite dt in down factor"))?;
    d_exp(
        d_mul(
            d_mul(
                dec!(-1.0),
                sqrt_dt,
                "pricing::binomial::down_factor::sqrt_dt",
            )?,
            volatility.to_dec(),
            "pricing::binomial::down_factor::exponent",
        )?,
        "pricing::binomial::down_factor",
    )
}

/// Calculates the probability using a given interest rate, time interval,
/// down factor, and up factor.
///
/// # Arguments
///
/// * `int_rate` - The interest rate as a floating-point number.
/// * `dt` - The time interval as a floating-point number.
/// * `down_factor` - The down factor as a floating-point number.
/// * `up_factor` - The up factor as a floating-point number.
///
/// # Returns
///
/// Returns the calculated probability which is clamped between `CLAMP_MIN` and `CLAMP_MAX`.
#[inline]
pub(crate) fn calculate_probability(
    int_rate: Decimal,
    dt: Decimal,
    down_factor: Decimal,
    up_factor: Decimal,
) -> Result<Decimal, DecimalError> {
    let spread = d_sub(
        up_factor,
        down_factor,
        "pricing::binomial::probability::spread",
    )?;
    if spread.is_zero() {
        // A lattice with no spread carries no risk-neutral probability: the
        // callers detect the collapse and take the deterministic path instead
        // of consuming a fabricated weight.
        return Err(DecimalError::arithmetic_error(
            "pricing::binomial::probability",
            "up and down factors coincide, risk-neutral probability is undefined",
        ));
    }
    let growth = d_exp(
        d_mul(int_rate, dt, "pricing::binomial::probability::rate_dt")?,
        "pricing::binomial::probability::growth",
    )?;
    Ok(d_div(
        d_sub(
            growth,
            down_factor,
            "pricing::binomial::probability::numerator",
        )?,
        spread,
        "pricing::binomial::probability",
    )?
    .clamp(CLAMP_MIN, CLAMP_MAX))
}

/// Calculates the discount factor given an interest rate and time period.
///
/// This function computes the discount factor using the formula:
/// `exp(-int_rate * dt)`, where `exp` is the exponential function.
///
/// # Parameters
/// - `int_rate`: The interest rate (as a floating-point number).
/// - `dt`: The time period (as a floating-point number).
///
/// # Returns
/// A floating-point number representing the discount factor.
///
#[inline]
pub(crate) fn calculate_discount_factor(
    int_rate: Decimal,
    dt: Decimal,
) -> Result<Decimal, DecimalError> {
    discount_factor(
        int_rate,
        dt,
        "pricing::binomial::discount_factor::exponent",
        "pricing::binomial::discount_factor",
    )
}

/// Calculates the value of an option node in a binomial options pricing model.
///
/// This function computes the value of a node by weighing the possible
/// future values at the next time step by the given probability of moving up.
/// The result is then discounted by a given discount factor to account for the
/// time value of money.
///
/// # Arguments
///
/// * `probability` - A `f64` representing the probability of moving to the next state.
/// * `next` - A mutable reference to a 2D vector containing the future values of the option.
/// * `node` - A `usize` indicating the current node's position.
/// * `discount_factor` - A `f64` used to discount the future values back to the present value.
///
/// # Returns
///
/// * A `f64` representing the calculated value of the current option node.
#[inline]
pub(crate) fn option_node_value_wrapper(
    probability: Decimal,
    next: &mut [Vec<Decimal>],
    node: usize,
    discount_factor: Decimal,
) -> Result<Decimal, DecimalError> {
    let next_step = next
        .first()
        .ok_or_else(|| DecimalError::arithmetic_error("pricing::binomial::node", "missing step"))?;
    let price_up = *next_step.get(node).ok_or_else(|| {
        DecimalError::arithmetic_error("pricing::binomial::node", "missing up node")
    })?;
    let price_down = *node
        .checked_add(1)
        .and_then(|down| next_step.get(down))
        .ok_or_else(|| {
            DecimalError::arithmetic_error("pricing::binomial::node", "missing down node")
        })?;
    option_node_value(probability, price_up, price_down, discount_factor)
}

/// Calculates the value of an option node in a binomial tree model.
///
/// # Parameters
/// - `probability`: The probability of the price moving up.
/// - `price_up`: The price if the market moves up.
/// - `price_down`: The price if the market moves down.
/// - `discount_factor`: The factor to discount the future value.
///
/// # Returns
/// The discounted expected value of the option node.
#[inline]
pub(crate) fn option_node_value(
    probability: Decimal,
    price_up: Decimal,
    price_down: Decimal,
    discount_factor: Decimal,
) -> Result<Decimal, DecimalError> {
    let up_branch = d_mul(probability, price_up, "pricing::binomial::node::up_branch")?;
    let down_branch = d_mul(
        d_sub(
            Decimal::ONE,
            probability,
            "pricing::binomial::node::down_weight",
        )?,
        price_down,
        "pricing::binomial::node::down_branch",
    )?;
    let expected = d_add(up_branch, down_branch, "pricing::binomial::node::expected")?;
    d_mul(
        expected,
        discount_factor,
        "pricing::binomial::node::discounted",
    )
}

/// Calculates the option price using the Binomial Pricing Model.
///
/// # Parameters
///
/// * `params`: An instance of `BinomialPricingParams` containing the necessary parameters
///   such as the asset price, strike price, option type, and number of steps.
/// * `u`: A `Decimal` representing the up factor in the binomial tree.
/// * `d`: A `Decimal` representing the down factor in the binomial tree.
/// * `i`: A `usize` representing the current step in the binomial tree.
///
/// # Returns
///
/// `Result<Decimal, PricingError>` — the option price at the given step,
/// or `PricingError::Options` if the payoff is not representable.
///
/// # Errors
///
/// Returns [`PricingError::Options`] wrapping
/// [`OptionsError::PayoffError`](optionstratlib_core::error::OptionsError::PayoffError)
/// when `params.option_type.payoff(...)` has no `Decimal` representation.
///
pub(crate) fn calculate_option_price(
    params: BinomialPricingParams,
    u: Decimal,
    d: Decimal,
    i: usize,
) -> Result<Decimal, PricingError> {
    // `i` is bounded by `no_steps` at every call site, but the subtraction is
    // checked so a caller that walks past the last step reports instead of
    // wrapping.
    let down_steps = params.no_steps.get().checked_sub(i).ok_or_else(|| {
        PricingError::method_error(
            "calculate_option_price",
            "step index exceeds the number of lattice steps",
        )
    })?;
    let up_power = d_powd(
        u,
        Decimal::from(i as u64),
        "pricing::binomial::option_price::up_power",
    )?;
    let down_power = d_powd(
        d,
        Decimal::from(down_steps as u64),
        "pricing::binomial::option_price::down_power",
    )?;
    let spot = d_mul(
        d_mul(
            params.asset.to_dec(),
            up_power,
            "pricing::binomial::option_price::spot_up",
        )?,
        down_power,
        "pricing::binomial::option_price::spot",
    )?;
    let info = PayoffInfo {
        spot: Positive::new_decimal(spot)?,
        strike: params.strike,
        style: *params.option_style,
        side: *params.side,
        spot_prices: None,
        spot_min: None,
        spot_max: None,
        exotic_params: None,
    };
    let payoff = params.option_type.payoff(&info)?;

    Ok(payoff)
}

/// Calculates the discounted payoff for an option based on the binomial pricing model.
///
/// # Parameters
///
/// * `params`: A structure containing parameters needed for the binomial pricing calculation.
///
/// # Returns
///
/// `Result<Decimal, PricingError>` — the discounted payoff (sign-adjusted for
/// `Side::Long` / `Side::Short`), or `PricingError::Options` if the
/// payoff is not representable.
///
/// The function takes into account the future asset price, the interest rate, the expiry time,
/// the type of option (call or put), and the style of the option (European or American).
///
/// It adjusts the future asset price with the provided interest rate and expiry time,
/// calculates the payoff, discounts it by the interest rate, and then adjusts for the side
/// of the trade (long or short).
///
/// # Errors
///
/// - [`PricingError::Options`] wrapping
///   [`OptionsError::PayoffError`](optionstratlib_core::error::OptionsError::PayoffError)
///   when `params.option_type.payoff(...)` has no `Decimal` representation.
/// - [`PricingError::Decimal`] (via `#[from]`) when the checked multiplications
///   `-rate * expiry` or `discount * payoff` overflow.
///
pub(crate) fn calculate_discounted_payoff(
    params: BinomialPricingParams,
) -> Result<Decimal, PricingError> {
    let growth = d_exp(
        d_mul(
            params.int_rate,
            params.expiry.to_dec(),
            "pricing::binomial::discounted_payoff::growth_exponent",
        )?,
        "pricing::binomial::discounted_payoff::growth",
    )?;
    let forward = d_mul(
        params.asset.to_dec(),
        growth,
        "pricing::binomial::discounted_payoff::forward",
    )?;
    let info = PayoffInfo {
        spot: Positive::new_decimal(forward)?,
        strike: params.strike,
        style: *params.option_style,
        side: *params.side,
        spot_prices: None,
        spot_min: None,
        spot_max: None,
        exotic_params: None,
    };

    let payoff = params.option_type.payoff(&info)?;
    // Build the discount exponent through a checked multiplication so
    // that an overflow on `-rate * expiry` is tagged rather than
    // saturating silently before `.exp()` compresses it back into a
    // bounded range.
    let discount = discount_factor(
        params.int_rate,
        params.expiry.to_dec(),
        "pricing::binomial::discounted_payoff::discount_exponent",
        "pricing::binomial::discounted_payoff::discount",
    )?;
    let discounted_payoff = d_mul(
        discount,
        payoff,
        "pricing::binomial::discounted_payoff::discounted",
    )?;
    match params.side {
        Side::Long => Ok(discounted_payoff),
        Side::Short => Ok(-discounted_payoff),
    }
}

/// Calculates a Wiener process (Brownian motion) increment over a small-time step `dt`.
///
/// This function uses the standard normal distribution to sample a value and scales it
/// by the square root of `dt` to produce the Wiener increment. The Wiener increment is a
/// random variable with a normal distribution, which is essential for simulating Brownian motion
/// in continuous time.
///
/// # Arguments
///
/// * `sqrt_dt` - The square root of the time step, [`wiener_sqrt_dt`] of it.
///   It is the same on every step of a path, so the caller computes it once
///   instead of once per increment (#859).
/// * `rng` - The generator the standard normal sample is drawn from.
///
/// # Returns
///
/// `Result<Decimal, PricingError>` — the Wiener process increment for the
/// given time step.
///
/// # Errors
///
/// - [`PricingError::Decimal`] (via `#[from]`) if `Normal::new(0.0, 1.0)` fails
///   (effectively never; parameters are constants) or if the scaled sample
///   leaves the `Decimal` range.
/// - [`PricingError::NonFinite`] if the sampled normal value is non-finite,
///   tagged `"pricing::monte_carlo::wiener_increment::sample"`.
///
pub(crate) fn wiener_increment<R: Rng + ?Sized>(
    sqrt_dt: Decimal,
    rng: &mut R,
) -> Result<Decimal, PricingError> {
    let normal = Normal::new(0.0, 1.0)
        .map_err(|e| DecimalError::arithmetic_error("Normal::new(0.0, 1.0)", &e.to_string()))?;

    let sample_f64 = normal.sample(rng);
    let sample = finite_decimal(sample_f64).ok_or_else(|| {
        PricingError::non_finite("pricing::monte_carlo::wiener_increment::sample", sample_f64)
    })?;

    Ok(d_mul(
        sample,
        sqrt_dt,
        "pricing::monte_carlo::wiener_increment::scaled",
    )?)
}

/// The `sqrt(dt)` a path passes to every [`wiener_increment`] of it.
///
/// It was computed inside `wiener_increment`, once per step of every path
/// (#859); the operation and its error are the same.
///
/// # Errors
///
/// Returns [`DecimalError`] when the square root of `dt` is undefined
/// (a negative or unrepresentable `dt`).
pub(crate) fn wiener_sqrt_dt(dt: Decimal) -> Result<Decimal, DecimalError> {
    d_sqrt(dt, "pricing::monte_carlo::wiener_increment::sqrt_dt")
        .map_err(|_| DecimalError::arithmetic_error("sqrt", "non-finite dt in wiener_increment"))
}

/// Calculates the probability that the option will remain under the strike price.
///
/// # Parameters
/// - `option`: An `Options` struct that contains various attributes necessary for the calculation,
///   such as underlying price, strike price, risk-free rate, expiration date, and implied volatility.
/// - `strike`: An optional `Positive` strike price. If `None`, the function uses
///   the `strike_price` from the `Options` struct.
///
/// # Returns
/// `Result<Decimal, DecimalError>` — the probability `N(-d2)` that the
/// underlying remains under the strike at expiration.
///
/// # Errors
///
/// - `DecimalError::ArithmeticError` if `d2(...)` cannot be evaluated for the
///   given option (e.g., zero volatility / non-positive time to expiration).
/// - Whatever `expiration_date.get_years()?` propagates through
///   `From<ExpirationDateError>`.
pub fn probability_keep_under_strike(
    option: Options,
    strike: Option<Positive>,
) -> Result<Decimal, DecimalError> {
    let strike_price = match strike {
        Some(strike) => strike,
        None => option.strike_price,
    };
    let years = option.expiration_date.get_years()?;
    let d2_val = d2(
        option.underlying_price,
        strike_price,
        d_sub(
            option.risk_free_rate,
            option.dividend_yield.to_dec(),
            "pricing::utils::probability_keep_under_strike::carry",
        )?, // carry b = r - q
        years,
        option.implied_volatility,
    )
    .map_err(|e| DecimalError::arithmetic_error("d2", &e.to_string()))?;
    big_n(-d2_val)
}

#[cfg(test)]
mod tests_simulate_returns {
    use super::*;
    use num_traits::FromPrimitive;
    use optionstratlib_core::pos_or_panic;
    use optionstratlib_core::utils::{DETERMINISTIC_RNG_DEFAULT_SEED, deterministic_rng};

    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::decimal::DecimalStats;
    use rust_decimal_macros::dec;

    #[test]
    fn test_simulate_returns() {
        let mean = dec!(0.05); // 5% annual return
        let std_dev = pos_or_panic!(0.2); // 20% annual volatility
        let length = 252; // One year of daily returns
        let time_step = Decimal::from_f64(1.0 / 252.0).unwrap(); // Daily time step

        let returns = simulate_returns(
            mean,
            std_dev,
            length,
            time_step,
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();

        assert_eq!(returns.len(), length);

        // Check that the mean and standard deviation are reasonably close to expected values
        let simulated_mean = returns.clone().mean().unwrap();
        let simulated_std_dev = returns.std_dev().unwrap();

        assert_decimal_eq!(simulated_mean, mean * time_step, dec!(0.01));
        assert_decimal_eq!(
            simulated_std_dev,
            std_dev * time_step.sqrt().unwrap(),
            dec!(0.01)
        );
    }
}

#[cfg(test)]
mod tests_simulate_returns_bis {
    use super::*;

    /// First three returns of `N(0, 0.2^2 * 0.004)` drawn from
    /// `deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED)`.
    const PINNED_RETURNS: [Decimal; 3] = [
        dec!(-0.0069930797843514744440741021),
        dec!(0.0115367336920983749405395342),
        dec!(-0.0018475187016137067853641022),
    ];
    use optionstratlib_core::pos_or_panic;
    use optionstratlib_core::utils::{DETERMINISTIC_RNG_DEFAULT_SEED, deterministic_rng};

    use num_traits::FromPrimitive;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::decimal::DecimalStats;
    use rust_decimal_macros::dec;

    #[test]
    fn test_simulate_returns_length() {
        let length = 1000;
        let returns = simulate_returns(
            dec!(0.05),
            pos_or_panic!(0.2),
            length,
            Decimal::from_f64(1.0 / 252.0).unwrap(),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        assert_eq!(returns.len(), length);
    }

    #[test]
    fn test_simulate_returns_zero_mean() {
        let returns = simulate_returns(
            dec!(0.0),
            pos_or_panic!(0.2),
            1000,
            Decimal::from_f64(1.0 / 252.0).unwrap(),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        let mean = returns.mean().unwrap();
        assert!(mean.abs() < dec!(0.01));
    }

    #[test]
    fn test_simulate_returns_zero_volatility() {
        let mean = dec!(0.05);
        let time_step = Decimal::from_f64(1.0 / 252.0).unwrap();
        let returns = simulate_returns(
            mean,
            Positive::ZERO,
            100,
            time_step,
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();

        let expected = mean * time_step;
        for r in returns {
            assert_decimal_eq!(r, expected, dec!(1e-10));
        }
    }

    #[test]
    fn test_simulate_returns_single_value() {
        let returns = simulate_returns(
            dec!(0.05),
            pos_or_panic!(0.2),
            1,
            Decimal::from_f64(1.0 / 252.0).unwrap(),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        assert_eq!(returns.len(), 1);
    }

    #[test]
    fn test_simulate_returns_yearly_step() {
        let returns = simulate_returns(
            dec!(0.05),
            pos_or_panic!(0.2),
            100,
            dec!(1.0),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        assert_eq!(returns.len(), 100);
        for r in returns {
            assert!(r > dec!(-1.0));
        }
    }

    #[test]
    #[should_panic]
    fn test_simulate_returns_invalid_std_dev() {
        assert!(
            simulate_returns(
                dec!(0.05),
                pos_or_panic!(-0.2),
                100,
                Decimal::from_f64(1.0 / 252.0).unwrap(),
                &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
            )
            .is_err()
        );
    }

    #[test]
    fn test_simulate_returns_same_seed_identical_returns() {
        let draw = |seed: u64| {
            simulate_returns(
                dec!(0.05),
                pos_or_panic!(0.2),
                9,
                dec!(0.004),
                &mut deterministic_rng(seed),
            )
            .unwrap()
        };
        assert_eq!(draw(5), draw(5));
        assert_ne!(draw(5), draw(6));
    }

    #[test]
    fn test_simulate_returns_seeded_regression_pinned() {
        let returns = simulate_returns(
            Decimal::ZERO,
            pos_or_panic!(0.2),
            3,
            dec!(0.004),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        assert_eq!(returns, PINNED_RETURNS.to_vec());
    }

    #[test]
    fn test_simulate_returns_seeded_moments_match_parameters() {
        // 20_000 N(mu dt, sigma^2 dt) draws: the sample mean has standard
        // error sigma sqrt(dt / n) ~ 8.9e-5 and the sample standard deviation
        // sigma sqrt(dt / (2 n)) ~ 6.3e-5; both bands are 5 of them.
        let mean = dec!(0.05);
        let time_step = dec!(0.004);
        let returns = simulate_returns(
            mean,
            pos_or_panic!(0.2),
            20_000,
            time_step,
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        let sample_mean = returns.clone().mean().unwrap();
        let sample_std = returns.std_dev().unwrap();
        assert_decimal_eq!(sample_mean, mean * time_step, dec!(0.000447));
        assert_decimal_eq!(
            sample_std,
            dec!(0.2) * time_step.sqrt().unwrap(),
            dec!(0.000317)
        );
    }
}

#[cfg(test)]
mod tests_utils {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use optionstratlib_core::assert_decimal_eq;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    fn test_calculate_up_factor() {
        let volatility = pos_or_panic!(0.09531018);
        let dt = dec!(1.0);
        let up_factor = calculate_up_factor(volatility, dt).unwrap();
        let expected_up_factor = (volatility * dt.sqrt().unwrap()).exp();
        assert!(
            (up_factor - expected_up_factor).abs() < EPSILON,
            "Expected {expected_up_factor}, got {up_factor}"
        );
    }

    #[test]
    fn test_calculate_up_factor_2() {
        let volatility = pos_or_panic!(0.17);
        let dt = dec!(1.0);
        let up_factor = calculate_up_factor(volatility, dt).unwrap();
        let expected_up_factor = dec!(1.1853048504885680);
        assert_decimal_eq!(up_factor, expected_up_factor, EPSILON);
    }

    #[test]
    fn test_calculate_down_factor() {
        let volatility = pos_or_panic!(0.09531018);
        let dt = dec!(1.0);
        let down_factor = calculate_down_factor(volatility, dt).unwrap();
        let expected_down_factor = (-dt.sqrt().unwrap() * volatility).exp();
        assert!(
            (down_factor - expected_down_factor).abs() < EPSILON,
            "Expected {expected_down_factor}, got {down_factor}"
        );
    }

    #[test]
    fn test_calculate_down_factor_2() {
        let volatility = pos_or_panic!(0.17);
        let dt = dec!(1.0);
        let up_factor = calculate_down_factor(volatility, dt).unwrap();
        let expected_up_factor = dec!(0.843664817188432427);
        assert_decimal_eq!(up_factor, expected_up_factor, EPSILON);
    }

    #[test]
    fn test_calculate_probability() {
        let int_rate = dec!(0.05);
        let dt = Decimal::ONE;
        let down_factor = dec!(0.909090909);
        let up_factor = dec!(1.1);
        let probability = calculate_probability(int_rate, dt, down_factor, up_factor).unwrap();
        let expected_probability = (((int_rate * dt).exp() - down_factor)
            / (up_factor - down_factor))
            .clamp(CLAMP_MIN, CLAMP_MAX);
        assert!(
            (probability - expected_probability).abs() < EPSILON,
            "Expected {expected_probability}, got {probability}"
        );
    }

    #[test]
    fn test_calculate_probability_ii() {
        let int_rate = dec!(0.05);
        let dt = Decimal::ONE;
        let down_factor = dec!(0.8);
        let up_factor = dec!(1.2);
        let probability = calculate_probability(int_rate, dt, down_factor, up_factor).unwrap();
        assert_decimal_eq!(probability, dec!(0.62817774088541), EPSILON);
    }

    #[test]
    fn test_calculate_discount_factor() {
        let int_rate = dec!(0.05);
        let dt = Decimal::ONE;
        let discount_factor = calculate_discount_factor(int_rate, dt).unwrap();
        let expected_discount_factor = (-int_rate * dt).exp();
        assert!(
            (discount_factor - expected_discount_factor).abs() < EPSILON,
            "Expected {expected_discount_factor}, got {discount_factor}"
        );
    }
}

/// #859: `sqrt(dt)` is computed once per path instead of inside every
/// increment. The increments must be the ones the per-step version drew,
/// digit for digit, from the same seeded generator.
#[cfg(test)]
mod tests_wiener_increment_hoisted_sqrt {
    use super::*;
    use optionstratlib_core::utils::deterministic_rng;
    use rust_decimal_macros::dec;

    /// `wiener_increment` as it was before #859: the square root inside.
    fn per_step_increment<R: Rng + ?Sized>(
        dt: Decimal,
        rng: &mut R,
    ) -> Result<Decimal, PricingError> {
        let normal = Normal::new(0.0, 1.0)
            .map_err(|e| DecimalError::arithmetic_error("Normal::new(0.0, 1.0)", &e.to_string()))?;
        let sample_f64 = normal.sample(rng);
        let sample = finite_decimal(sample_f64).ok_or_else(|| {
            PricingError::non_finite("pricing::monte_carlo::wiener_increment::sample", sample_f64)
        })?;
        let sqrt_dt =
            d_sqrt(dt, "pricing::monte_carlo::wiener_increment::sqrt_dt").map_err(|_| {
                DecimalError::arithmetic_error("sqrt", "non-finite dt in wiener_increment")
            })?;
        Ok(d_mul(
            sample,
            sqrt_dt,
            "pricing::monte_carlo::wiener_increment::scaled",
        )?)
    }

    #[test]
    fn test_wiener_increment_hoisted_sqrt_is_bit_identical() {
        for dt in [
            dec!(0.00396825396825),
            dec!(0.0833333333333),
            Decimal::ONE,
            dec!(2.5),
        ] {
            let mut hoisted_rng = deterministic_rng(17);
            let mut per_step_rng = deterministic_rng(17);
            let sqrt_dt = wiener_sqrt_dt(dt).unwrap();
            for step in 0..5_000 {
                let hoisted = wiener_increment(sqrt_dt, &mut hoisted_rng).unwrap();
                let per_step = per_step_increment(dt, &mut per_step_rng).unwrap();
                assert_eq!(hoisted, per_step, "dt {dt}, step {step}");
            }
        }
    }

    #[test]
    fn test_wiener_sqrt_dt_rejects_a_negative_dt() {
        assert!(wiener_sqrt_dt(dec!(-0.01)).is_err());
        assert_eq!(wiener_sqrt_dt(dec!(0.25)).unwrap(), dec!(0.5));
    }
}

#[cfg(test)]
mod tests_probability_keep_under_strike {
    use super::*;
    use optionstratlib_core::{model::Positive, pos_or_panic, spos};

    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::constants::DAYS_IN_A_YEAR;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::{OptionStyle, OptionType};
    use rust_decimal_macros::dec;
    use tracing::info;

    #[test]
    fn test_probability_keep_under_strike_with_given_strike() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: Positive::HUNDRED,
            risk_free_rate: Decimal::ZERO,
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            implied_volatility: pos_or_panic!(0.001),
            underlying_symbol: "".to_string(),
            quantity: Positive::ONE,
            contract_size: Positive::ONE,
            exotic_params: None,
        };
        let strike = spos!(100.0);
        let probability = probability_keep_under_strike(option, strike).unwrap();
        info!("{:?} {}", strike, probability);
        assert_decimal_eq!(probability, dec!(0.5), dec!(0.001));
    }

    #[test]
    fn test_probability_keep_under_strike_with_default_strike() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: pos_or_panic!(110.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            implied_volatility: pos_or_panic!(0.2),
            underlying_symbol: "".to_string(),
            quantity: Positive::ZERO,
            contract_size: Positive::ONE,
            exotic_params: None,
        };
        let strike = None;
        let probability = probability_keep_under_strike(option, strike).unwrap();
        assert!(
            probability > Decimal::ZERO && probability < Decimal::ONE,
            "Probability should be between 0 and 1"
        );
    }

    #[test]
    fn test_probability_keep_under_strike_zero_volatility() {
        // Zero implied volatility makes d2 ill-defined (division by zero in the
        // analytical form). Post panic-free refactor this surfaces as a typed Err
        // instead of a panic.
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: Positive::HUNDRED,
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            implied_volatility: Positive::ZERO,
            underlying_symbol: "".to_string(),
            quantity: Positive::ZERO,
            contract_size: Positive::ONE,
            exotic_params: None,
        };
        let strike = None;
        assert!(
            probability_keep_under_strike(option, strike).is_err(),
            "zero volatility should produce a DecimalError, not a panic"
        );
    }

    #[test]
    fn test_probability_keep_under_strike_high_volatility() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: pos_or_panic!(110.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            implied_volatility: pos_or_panic!(5.0), // Alta volatilidad
            underlying_symbol: "".to_string(),
            quantity: Positive::ZERO,
            contract_size: Positive::ONE,
            exotic_params: None,
        };
        let strike = None;
        let probability = probability_keep_under_strike(option, strike).unwrap();
        assert!(
            probability > Decimal::ZERO && probability < Decimal::ONE,
            "Probability should still be valid even with high volatility"
        );
    }

    #[test]
    fn test_probability_keep_under_strike_expired_option() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: pos_or_panic!(110.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            expiration_date: ExpirationDate::Days(Positive::ONE),
            implied_volatility: pos_or_panic!(0.2),
            underlying_symbol: "".to_string(),
            quantity: Positive::ZERO,
            contract_size: Positive::ONE,
            exotic_params: None,
        };
        let strike = None;
        let probability = probability_keep_under_strike(option, strike).unwrap();
        assert_eq!(
            probability,
            Decimal::ONE,
            "Expired option should have zero probability of being ITM"
        );
    }
}

#[cfg(test)]
mod tests_calculate_up_down_factor {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::decimal::ONE_DAY;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    fn test_factors_standard_case() {
        let volatility = pos_or_panic!(0.2); // 20% volatility
        let dt = ONE_DAY; // One trading day

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // Verify that up and down factors are reciprocals
        assert_decimal_eq!(up * down, dec!(1.0), EPSILON);
        // Verify values are in expected range
        assert!(up > Decimal::ONE);
        assert!(down < Decimal::ONE);
    }

    #[test]
    fn test_factors_zero_volatility() {
        let volatility = Positive::ZERO;
        let dt = ONE_DAY;

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // With zero volatility, both factors should be 1.0
        assert_decimal_eq!(up, Decimal::ONE, dec!(1e-10));
        assert_decimal_eq!(down, Decimal::ONE, dec!(1e-10));
    }

    #[test]
    fn test_factors_zero_dt() {
        let volatility = pos_or_panic!(0.2);
        let dt = Decimal::ZERO;

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // With zero dt, both factors should be 1.0
        assert_decimal_eq!(up, Decimal::ONE, EPSILON);
        assert_decimal_eq!(down, Decimal::ONE, EPSILON);
    }

    #[test]
    fn test_factors_high_volatility() {
        let volatility = Positive::ONE; // 100% volatility
        let dt = Decimal::ONE; // One year

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // Verify expected behavior for extreme values
        assert!(up > dec!(1.0));
        assert!(down < dec!(1.0));
        assert_decimal_eq!(up * down, Decimal::ONE, dec!(1e-10));
    }

    #[test]
    fn test_factors_small_dt() {
        let volatility = pos_or_panic!(0.2);
        let dt = ONE_DAY / dec!(24.0); // One hour (assuming 24-hour trading day)

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // Verify behavior with very small time steps
        assert!(up > Decimal::ONE);
        assert!(down < Decimal::ONE);
        assert_decimal_eq!(up * down, Decimal::ONE, dec!(1e-10));
    }

    #[test]
    fn test_factors_different_time_periods() {
        let volatility = pos_or_panic!(0.2);
        let daily_dt = ONE_DAY;
        let weekly_dt = dec!(5.0) / dec!(252.0);
        let monthly_dt = dec!(21.0) / dec!(252.0);

        let daily_up = calculate_up_factor(volatility, daily_dt).unwrap();
        let weekly_up = calculate_up_factor(volatility, weekly_dt).unwrap();
        let monthly_up = calculate_up_factor(volatility, monthly_dt).unwrap();

        // Longer periods should have larger factors
        assert!(daily_up < weekly_up);
        assert!(weekly_up < monthly_up);
    }

    #[test]
    fn test_factors_extreme_volatility() {
        let volatility = pos_or_panic!(5.0); // 500% volatility
        let dt = Decimal::ONE; // One year

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // Verify behavior with extreme volatility
        assert!(up > Decimal::ONE);
        assert!(down < Decimal::ONE);
        assert_decimal_eq!(up * down, Decimal::ONE, EPSILON);
    }

    #[test]
    fn test_factors_symmetry() {
        let volatility = pos_or_panic!(0.3);
        let dt = dec!(1.0) / dec!(12.0); // One month

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // Up move should be multiplicative inverse of down move
        assert_decimal_eq!(up, Decimal::ONE / down, dec!(1e-10));
    }

    #[test]
    fn test_factors_consistency() {
        let volatility = pos_or_panic!(0.2);
        let dt1 = ONE_DAY;
        let dt2 = dt1 / dec!(2.0);

        let up1 = calculate_up_factor(volatility, dt1).unwrap();
        let up2 = calculate_up_factor(volatility, dt2).unwrap();

        // Factor for larger dt should be greater
        assert!(up1 > up2);
    }
}
