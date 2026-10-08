/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-10-05
******************************************************************************/

//! Formulas shared by the pricing models and the Greeks (#523).
//!
//! This module is the neutral bottom of the crate's internal graph: it
//! depends only on `optionstratlib_core` and `crate::error`, never on
//! `crate::pricing`, `crate::greeks` or `crate::volatility`. Both
//! `pricing` and `greeks` import from here, so `pricing` no longer reaches
//! into `greeks` and the only edge between the two is `greeks -> pricing`
//! (the bump-and-reprice numerical Greeks).
//!
//! The module is private. The items that were already public keep their
//! public path through re-exports in `crate::greeks` (`d1`, `d2`,
//! `big_n`, `calculate_d_values_black_76`); everything else is
//! `pub(crate)`.
//!
//! Only formulas used by a pricing model and a Greek, or by several pricing
//! models, live here:
//!
//! | Kernel | Used by |
//! |---|---|
//! | [`big_n`] (normal cdf) | every closed-form pricer and Greek |
//! | [`d1`], [`d2`] | Black-Scholes family pricers and Greeks |
//! | [`calculate_d_values`] | Black-Scholes and Garman-Kohlhagen pricers |
//! | [`calculate_d_values_black_76`] | Black-76 pricer and Black-76 Greeks |
//! | [`d_values_and_time`] | Black-Scholes and Black-76 pricers |
//! | [`discount_factor`] | Black-76, binomial, Monte Carlo, telegraph and exotic pricers; Black-Scholes, Black-76 and Garman-Kohlhagen Greeks |
//!
//! The normal pdf `n` is used by the Greeks only, so it stays in
//! `greeks::utils`. Model-specific variants stay with their model, each with
//! a comment saying why: the Barone-Adesi-Whaley `d1` (`pricing::american`), the
//! Garman-Kohlhagen Greek d-values (`greeks::garman_kohlhagen`) and the
//! memoised Black-Scholes Greek kernels (`greeks::equations`).
//!
//! Every kernel here was moved or factored out without changing a single
//! arithmetic step, rounding, scale or error label: results are
//! bit-identical to the inline code they replace.

use crate::error::PricingError;
use crate::error::greeks::{GreeksError, InputErrorKind, MathErrorKind};
use num_traits::ToPrimitive;
use optionstratlib_core::error::DecimalError;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{
    d_add, d_div, d_exp, d_ln, d_mul, d_powd, d_sqrt, d_sub, f64_to_decimal,
};
use rust_decimal::Decimal;
use statrs::distribution::{ContinuousCDF, Normal};

/// Calculates the `d1` parameter used in the Black-Scholes options pricing model.
///
/// The `d1` value is an intermediary result used to determine option greeks and prices.
/// It is computed using the formula:
///
/// ```math
/// d1 = (ln(S / K) + (b + σ² / 2) * T) / (σ * sqrt(T))
/// ```
///
/// Where:
/// - `S`: Underlying price
/// - `K`: Strike price
/// - `b`: Cost of carry, `b = r - q` (risk-free rate minus dividend yield)
/// - `T`: Time to expiration (in years)
/// - `σ`: Implied volatility
///
/// # Parameters
///
/// - `underlying_price`: The current price of the underlying asset. Must be positive.
/// - `strike_price`: The strike price of the option. Must be greater than zero.
/// - `carry_rate`: The cost of carry `b = r - q` (annual risk-free rate minus
///   the continuous dividend yield; equals `r` when the underlying pays no dividends).
/// - `expiration_date`: The time to expiration of the option, in years. Must be greater than zero.
/// - `implied_volatility`: The implied volatility of the option, expressed as a decimal. Must be greater than zero.
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed `d1` value.
/// - `Err(GreeksError)`: Returns an error if input validation fails. Possible errors include:
///   - Invalid underlying price (must be greater than zero).
///   - Invalid strike price (must be greater than zero).
///   - Invalid implied volatility (must be greater than zero).
///   - Invalid expiration time (must be greater than zero).
///
/// # Errors
///
/// Returns a `GreeksError::InputError` in the following cases:
/// - **InvalidPrice**: Triggered when `underlying_price` is zero or less.
/// - **InvalidStrike**: Triggered when `strike_price` is zero or less.
/// - **InvalidVolatility**: Triggered when `implied_volatility` is zero.
/// - **InvalidTime**: Triggered when `expiration_date` is zero or less.
///
/// Returns `GreeksError::MathError` or a `DecimalError` when an intermediate
/// step leaves the representable `Decimal` range: `sigma²`, the moneyness
/// `S / K`, the drift, or the final quotient.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib_pricing::greeks::d1;
/// use optionstratlib_core::{pos_or_panic, model::Positive};
///
/// let underlying_price = Positive::HUNDRED;
/// let strike_price = pos_or_panic!(95.0);
/// let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
/// let expiration_date = pos_or_panic!(0.5); // 6 months
/// let implied_volatility = pos_or_panic!(0.2);
///
/// match d1(
///     underlying_price,
///     strike_price,
///     carry_rate,
///     expiration_date,
///     implied_volatility,
/// ) {
///     Ok(result) => info!("d1: {}", result),
///     Err(e) => error!("Error: {:?}", e),
/// }
/// ```
#[inline]
pub fn d1(
    underlying_price: Positive,
    strike_price: Positive,
    carry_rate: Decimal,
    expiration_date: Positive,
    implied_volatility: Positive,
) -> Result<Decimal, GreeksError> {
    if underlying_price == Positive::ZERO {
        return Err(GreeksError::InputError(InputErrorKind::InvalidPrice {
            value: underlying_price.to_f64(),
            reason: "Underlying price price cannot be zero".to_string(),
        }));
    }

    if strike_price == Positive::ZERO {
        return Err(GreeksError::InputError(InputErrorKind::InvalidStrike {
            value: strike_price.to_string(),
            reason: "Strike price cannot be zero".to_string(),
        }));
    }

    if implied_volatility == Decimal::ZERO {
        return Err(GreeksError::InputError(InputErrorKind::InvalidVolatility {
            value: implied_volatility.to_f64(),
            reason: "Implied volatility cannot be zero".to_string(),
        }));
    }

    if expiration_date == Decimal::ZERO {
        return Err(GreeksError::InputError(InputErrorKind::InvalidTime {
            value: expiration_date,
            reason: "Expiration date cannot be zero".to_string(),
        }));
    }

    // d1 = (ln(S / K) + (b + σ² / 2) * T) / (σ * sqrt(T))
    // where b = carry_rate is the cost of carry: b = r − q for a
    // dividend-paying underlying (r − dividend_yield), r otherwise.
    // Every step below goes through the checked helpers: the raw `Decimal`
    // and `Positive` operators panic on overflow, and `ln` panics on zero,
    // both of which are reachable from ordinary-looking inputs (a tiny
    // strike, a huge underlying, a volatility at the edge of the scale).
    let underlying_price: Decimal = underlying_price.to_dec();
    let strike: Decimal = strike_price.to_dec();
    let sigma: Decimal = implied_volatility.to_dec();
    let time_to_expiry: Decimal = expiration_date.to_dec();

    let implied_volatility_squared = d_powd(sigma, Decimal::TWO, "greeks::d1::sigma_squared")?;
    let ln_price_ratio = match strike_price {
        value if value == Positive::MAX => Decimal::MIN,
        _ => {
            let moneyness = d_div(underlying_price, strike, "greeks::d1::moneyness")?;
            if moneyness.is_zero() {
                // `S / K` fell below the smallest representable `Decimal`, so
                // `ln` is `-inf`. Represented the same way as the
                // `Positive::MAX` strike branch above.
                Decimal::MIN
            } else {
                d_ln(moneyness, "greeks::d1::log_moneyness")?
            }
        }
    };

    let half_variance = d_div(
        implied_volatility_squared,
        Decimal::TWO,
        "greeks::d1::half_variance",
    )?;
    let rate_vol_term = d_add(carry_rate, half_variance, "greeks::d1::carry_term")?;
    let drift = d_mul(rate_vol_term, time_to_expiry, "greeks::d1::drift")?;
    let numerator = d_add(ln_price_ratio, drift, "greeks::d1::numerator")?;
    let sqrt_time = d_sqrt(time_to_expiry, "greeks::d1::sqrt_time")?;
    let denominator = d_mul(sigma, sqrt_time, "greeks::d1::denominator")?;

    if denominator.is_zero() {
        return Err(GreeksError::MathError(MathErrorKind::DivisionByZero));
    }

    match numerator.checked_div(denominator) {
        Some(result) => Ok(result),
        None => Err(GreeksError::MathError(MathErrorKind::Overflow)),
    }
}

/// Calculates the `d2` parameter used in the Black-Scholes options pricing model.
///
/// The `d2` value is an intermediary result derived from the `d1` value and is used
/// to determine option greeks and prices. It is computed using the formula:
///
/// ```math
/// d2 = d1 - σ * sqrt(T)
/// ```
///
/// Where:
/// - `d1`: The `d1` value calculated using the `d1` function.
/// - `σ`: Implied volatility.
/// - `T`: Time to expiration (in years).
///
/// # Parameters
///
/// - `underlying_price`: The current price of the underlying asset. Must be positive.
/// - `strike_price`: The strike price of the option. Must be greater than zero.
/// - `carry_rate`: The cost of carry `b = r - q` (annual risk-free rate minus
///   the continuous dividend yield; equals `r` when the underlying pays no dividends).
/// - `expiration_date`: The time to expiration of the option, in years. Must be greater than zero.
/// - `implied_volatility`: The implied volatility of the option, expressed as a decimal. Must be greater than zero.
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed `d2` value.
/// - `Err(GreeksError)`: Returns an error if input validation fails or if the `d1` computation fails.
///
/// # Errors
///
/// Returns a `GreeksError::InputError` in the following cases:
/// - **InvalidVolatility**: Triggered when `implied_volatility` is zero.
/// - **InvalidTime**: Triggered when `expiration_date` is zero.
///
/// # Notes
///
/// This function depends on the `d1` function to compute the `d1` value. Any errors from
/// the `d1` function will propagate to this function.
///
/// # Example
///
/// ```rust
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib_pricing::greeks::d2;
/// use optionstratlib_core::{pos_or_panic, model::Positive};
/// let underlying_price = Positive::new(100.0)?;
/// let strike_price = Positive::new(95.0)?;
/// let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
/// let expiration_date = pos_or_panic!(0.5); // 6 months
/// let implied_volatility = pos_or_panic!(0.2);
///
/// match d2(
///     underlying_price,
///     strike_price,
///     carry_rate,
///     expiration_date,
///     implied_volatility,
/// ) {
///     Ok(result) => info!("d2: {}", result),
///     Err(e) => error!("Error: {:?}", e),
/// }
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn d2(
    underlying_price: Positive,
    strike_price: Positive,
    carry_rate: Decimal,
    expiration_date: Positive,
    implied_volatility: Positive,
) -> Result<Decimal, GreeksError> {
    //    function d1 is already checking for validity of implied_volatility and
    //    expiration_date with error propagation. Can we comment out the two if blocks below?
    //    if implied_volatility == Decimal::ZERO {
    //        return Err(GreeksError::InputError(InputErrorKind::InvalidVolatility {
    //            value: implied_volatility.to_f64(),
    //            reason: "Implied volatility cannot be zero".to_string(),
    //        }));
    //    }
    //
    //    if expiration_date == Decimal::ZERO {
    //        return Err(GreeksError::InputError(InputErrorKind::InvalidTime {
    //            value: expiration_date,
    //            reason: "Expiration date cannot be zero".to_string(),
    //        }));
    //    }

    let d1_value = d1(
        underlying_price,
        strike_price,
        carry_rate,
        expiration_date,
        implied_volatility,
    )?;
    d2_from_d1(d1_value, expiration_date, implied_volatility)
}

/// `d2 = d1 - sigma * sqrt(T)` from a `d1` already computed.
///
/// The single implementation of the step [`d2`] performs after its own
/// [`d1`], so a caller that holds `d1` gets the same digits as `d2` without
/// paying for `d1` (its logarithm and square root) a second time (#859).
///
/// # Errors
///
/// Returns [`GreeksError`] when `sqrt(T)`, `sigma * sqrt(T)` or the
/// difference leaves the `Decimal` range.
#[inline]
fn d2_from_d1(
    d1_value: Decimal,
    expiration_date: Positive,
    implied_volatility: Positive,
) -> Result<Decimal, GreeksError> {
    let sqrt_time = d_sqrt(expiration_date.to_dec(), "greeks::d2::sqrt_time")?;
    let vol_time = d_mul(
        implied_volatility.to_dec(),
        sqrt_time,
        "greeks::d2::vol_time",
    )?;
    Ok(d_sub(d1_value, vol_time, "greeks::d2::adjustment")?)
}

/// Computes the cumulative distribution function (CDF) of the standard normal distribution
/// for a given input `x`.
///
/// The function uses the standard normal distribution (mean = 0, standard deviation = 1)
/// to calculate the probability that a normally distributed random variable is less than or
/// equal to `x`. This is commonly referred to as `N(x)` in financial and statistical contexts.
///
/// # Parameters
///
/// - `x: Decimal`
///   The input value for which the CDF is computed. Must be convertible to `f64`.
///
/// # Returns
///
/// - `Ok(Decimal)`: The CDF value corresponding to the input `x`.
/// - `Err(DecimalError)`: Returns an error if the conversion from `Decimal` to `f64` fails.
///
/// # Errors
///
/// Returns a `DecimalError::ConversionError` if:
/// - The input `x` cannot be converted to an `f64`.
///
/// # Notes
///
/// This function uses the [`statrs`](https://docs.rs/statrs/latest/statrs/) crate to model the
/// standard normal distribution and compute the CDF. The result is returned as a `Decimal`
/// for precision.
///
/// # Example
///
/// ```rust
/// use rust_decimal::Decimal;
/// use tracing::{error, info};
/// use optionstratlib_pricing::greeks::big_n;
///
/// let x = Decimal::new(100, 2);
///
/// match big_n(x) {
///     Ok(result) => info!("N(x): {}", result),
///     Err(e) => error!("Error: {:?}", e),
/// }
/// ```
#[inline]
pub fn big_n(x: Decimal) -> Result<Decimal, DecimalError> {
    let Some(x_f64) = x.to_f64() else {
        return Err(DecimalError::ConversionError {
            from_type: "Decimal".to_string(),
            to_type: "f64".to_string(),
            reason: "Conversion failed".to_string(),
        });
    };

    // Guard the `Decimal` → `f64` boundary: if `x` is outside the
    // representable `f64` range the conversion returns `±∞` rather
    // than `None`, and feeding that into `statrs::cdf` yields `NaN`
    // which would later collapse silently in `f64_to_decimal`.
    if !x_f64.is_finite() {
        return Err(DecimalError::invalid_value(
            x_f64,
            "big_n: Decimal -> f64 produced a non-finite value",
        ));
    }

    const MEAN: f64 = 0.0;
    const STD_DEV: f64 = 1.0;

    // Normal::new(0.0, 1.0) is infallible by construction (parameters
    // hardcoded), but surface the error type via map_err to satisfy
    // the panic-free rule.
    let normal_distribution =
        Normal::new(MEAN, STD_DEV).map_err(|e| DecimalError::ConversionError {
            from_type: "(f64, f64)".to_string(),
            to_type: "Normal".to_string(),
            reason: format!("invalid Normal parameters: {e}"),
        })?;
    let cdf = normal_distribution.cdf(x_f64);
    if !cdf.is_finite() {
        return Err(DecimalError::invalid_value(
            cdf,
            "big_n: CDF produced a non-finite value",
        ));
    }
    f64_to_decimal(cdf)
}

/// Calculates the d1 and d2 values used in financial option pricing models such as the Black-Scholes model.
///
/// # Arguments
///
/// * `option` - A reference to an `Options` struct containing the underlying price,
///   the risk-free rate, and the implied volatility of the option.
///
/// # Returns
///
/// * A tuple containing two `Decimal` values:
///     - `d1_value`: The calculated d1 value.
///     - `d2_value`: The calculated d2 value.
///
/// # Errors
///
/// Returns [`GreeksError`] when the cost of carry `r - q` overflows, when
/// the expiration cannot be converted to a year fraction, or when [`d1`] or
/// [`d2`] rejects the inputs.
#[inline]
pub(crate) fn calculate_d_values(option: &Options) -> Result<(Decimal, Decimal), GreeksError> {
    calculate_d_values_with_yield(option, option.dividend_yield.to_dec())
}

/// [`calculate_d_values`] with the continuous yield `q` given explicitly
/// and signed, instead of read from `Options::dividend_yield`: the
/// Garman–Kohlhagen pricer passes its foreign rate `r_f`, which may be
/// negative (#720). `calculate_d_values` is this function at
/// `q = dividend_yield`, so both produce the same digits for the same `q`.
///
/// # Errors
///
/// Same as [`calculate_d_values`].
#[inline]
pub(crate) fn calculate_d_values_with_yield(
    option: &Options,
    q: Decimal,
) -> Result<(Decimal, Decimal), GreeksError> {
    // `Decimal`'s `-` panics on overflow, which a rate at the edge of the
    // range reaches (`Decimal::MIN` minus any positive yield).
    let b = d_sub(option.risk_free_rate, q, "greeks::carry_rate")?;
    let years = option.expiration_date.get_years()?;
    // `d2` is derived from this `d1`: calling [`d2`] recomputed `d1`, so every
    // price paid for its logarithm twice (#859). Same digits either way.
    let d1_value = d1(
        option.underlying_price,
        option.strike_price,
        b,
        years,
        option.implied_volatility,
    )?;
    let d2_value = d2_from_d1(d1_value, years, option.implied_volatility)?;
    Ok((d1_value, d2_value))
}

/// Calculates d1 and d2 for the Black-76 model (options on futures/forwards).
///
/// For Black-76, the cost of carry `b = 0` because the forward price F already
/// incorporates all carry. This function specializes `calculate_d_values` for
/// that case.
///
/// # Arguments
/// * `option` - The option data; `underlying_price` holds the forward price F.
///
/// # Returns
/// * `Ok((d1, d2))` - The Black-76 d1 and d2 parameters.
/// * `Err(GreeksError)` - If validation or computation fails.
///
/// # Errors
///
/// Returns [`GreeksError`] when the expiration cannot be converted to a year
/// fraction, or when [`d1`] or [`d2`] rejects the inputs.
pub fn calculate_d_values_black_76(option: &Options) -> Result<(Decimal, Decimal), GreeksError> {
    // Black-76: cost of carry b = 0 (forward pricing, no carry term in d1/d2)
    let b = Decimal::ZERO;
    let years = option.expiration_date.get_years()?;
    // `d1` once, `d2` from it (#859), as in `calculate_d_values_with_yield`.
    let d1_value = d1(
        option.underlying_price,
        option.strike_price,
        b,
        years,
        option.implied_volatility,
    )?;
    let d2_value = d2_from_d1(d1_value, years, option.implied_volatility)?;
    Ok((d1_value, d2_value))
}

/// Continuous discount factor `e^(-rate * t)`.
///
/// The one implementation of the discounting step shared by the Black-76,
/// binomial, Monte Carlo, telegraph and exotic pricers and by the
/// Black-Scholes, Black-76 and Garman-Kohlhagen Greeks. It performs exactly
/// the two checked steps every caller used to inline, in the same order:
/// `exponent = -rate * t`, then `exp(exponent)`.
///
/// `exponent_op` and `exp_op` label the two steps, so each caller keeps the
/// error context it reported before.
///
/// # Arguments
///
/// * `rate` - The continuously compounded rate (risk-free, dividend or
///   foreign rate), annualised.
/// * `t` - The time in years.
/// * `exponent_op` - Label of the `-rate * t` multiplication.
/// * `exp_op` - Label of the exponential.
///
/// # Errors
///
/// Returns [`DecimalError`] when `-rate * t` leaves the `Decimal` range or
/// when a positive exponent overflows `exp`. An exponent so negative that
/// the factor is below the representable scale flushes to zero, which is the
/// discount factor's limit.
#[inline]
pub(crate) fn discount_factor(
    rate: Decimal,
    t: Decimal,
    exponent_op: &'static str,
    exp_op: &'static str,
) -> Result<Decimal, DecimalError> {
    let exponent = d_mul(-rate, t, exponent_op)?;
    d_exp(exponent, exp_op)
}

/// Returns `(d1, d2, T)` for a closed-form pricer: the time to expiry in
/// years read from the option and the pair produced by `d_values`.
///
/// Shared by the Black-Scholes pricer (with [`calculate_d_values`],
/// `b = r - q`) and the Black-76 pricer (with
/// [`calculate_d_values_black_76`], `b = 0`), which used to carry two
/// copies of this helper that differed only in the pair they pulled.
///
/// # Errors
///
/// Returns [`PricingError`] when the expiration cannot be converted to a
/// year fraction, or wraps the [`GreeksError`] raised by `d_values`.
#[inline]
pub(crate) fn d_values_and_time<F>(
    option: &Options,
    d_values: F,
) -> Result<(Decimal, Decimal, Decimal), PricingError>
where
    F: FnOnce(&Options) -> Result<(Decimal, Decimal), GreeksError>,
{
    let calculated_time_to_expiry: Decimal = option.time_to_expiration()?.to_dec();
    let (d1, d2) = d_values(option)?;
    Ok((d1, d2, calculated_time_to_expiry))
}

#[cfg(test)]
mod tests_calculate_d_values {
    use super::*;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};

    use approx::assert_relative_eq;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_d_values() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "".to_string(),
            strike_price: pos_or_panic!(110.0),
            underlying_price: Positive::HUNDRED,
            risk_free_rate: dec!(0.05),
            implied_volatility: pos_or_panic!(10.12),
            expiration_date: Default::default(),
            quantity: Positive::ONE,
            contract_size: Positive::ONE,
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        };
        let (d1_value, d2_value) = calculate_d_values(&option).unwrap();

        assert_relative_eq!(
            d1_value.to_f64().unwrap(),
            5.055522709505501,
            epsilon = 0.001
        );
        assert_relative_eq!(
            d2_value.to_f64().unwrap(),
            -5.064477290494499,
            epsilon = 0.001
        );
    }
}

#[cfg(test)]
mod tests_src_greeks_utils {
    use super::*;

    use approx::assert_relative_eq;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;
    use statrs::distribution::ContinuousCDF;
    use statrs::distribution::Normal;

    #[test]
    fn test_d1_zero_sigma() {
        let s = Positive::HUNDRED;
        let k = Positive::HUNDRED;
        let r = dec!(0.05);
        let t = Positive::ONE;
        let sigma = Positive::ZERO;
        let _ = d1(s, k, r, t, sigma).is_err();
    }

    #[test]
    fn test_d1_zero_t() {
        let s = Positive::HUNDRED;
        let k = Positive::HUNDRED;
        let r = dec!(0.05);
        let t = Positive::ZERO;
        let sigma = pos_or_panic!(0.01);
        let _ = d1(s, k, r, t, sigma).is_err();
    }

    #[test]
    fn test_d2_bis_i() {
        let s = Positive::HUNDRED;
        let k = pos_or_panic!(110.0);
        let r = dec!(0.05);
        let t = Positive::TWO;
        let sigma = pos_or_panic!(0.2);
        let computed_d2 = d2(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        let computed_d1 = d1(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        assert_relative_eq!(computed_d1, 0.15800237455184707, epsilon = 0.001);
        assert_relative_eq!(computed_d2, -0.12484033792277195, epsilon = 0.001);
    }

    #[test]
    fn test_d2_bis_ii() {
        let s = Positive::HUNDRED;
        let k = pos_or_panic!(95.0);
        let r = dec!(0.15);
        let t = Positive::ONE;
        let sigma = pos_or_panic!(0.2);
        let computed_d2 = d2(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        let computed_d1 = d1(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        assert_relative_eq!(computed_d1, 1.1064664719377526, epsilon = 0.001);
        assert_relative_eq!(computed_d2, 0.9064664719377528, epsilon = 0.001);
    }

    #[test]
    fn test_d2_zero_sigma() {
        let s = Positive::HUNDRED;
        let k = Positive::HUNDRED;
        let r = Decimal::ZERO;
        let t = Positive::ONE;
        let sigma = Positive::ZERO;
        let _ = d2(s, k, r, t, sigma).is_err();
    }

    #[test]
    fn test_d2_zero_t() {
        let s = Positive::HUNDRED;
        let k = Positive::HUNDRED;
        let r = dec!(0.02);
        let t = Positive::ZERO;
        let sigma = pos_or_panic!(0.01);
        let _ = d2(s, k, r, t, sigma).is_err();
    }

    #[test]
    fn test_big_n() {
        let x = Decimal::ZERO;
        let normal_distribution = Normal::new(0.0, 1.0).unwrap();
        let expected_big_n = normal_distribution.cdf(x.to_f64().unwrap());
        let computed_big_n = big_n(x).unwrap().to_f64().unwrap();
        assert!(
            (computed_big_n - expected_big_n).abs() < 1e-10,
            "big_n function failed"
        );

        let x = Decimal::ONE;
        let expected_big_n = normal_distribution.cdf(1.0);
        let computed_big_n = big_n(x).unwrap().to_f64().unwrap();
        assert!(
            (computed_big_n - expected_big_n).abs() < 1e-10,
            "big_n function failed"
        );
    }
}

#[cfg(test)]
mod calculate_d1_values {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use rust_decimal_macros::dec;

    #[test]
    fn test_d1_zero_volatility() {
        // Case where volatility (sigma) is zero
        let underlying_price = Positive::HUNDRED;
        let strike_price = Positive::HUNDRED;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ONE;
        let implied_volatility = Positive::ZERO;

        // When volatility is zero, d1 should handle the case correctly
        assert!(
            d1(
                underlying_price,
                strike_price,
                carry_rate,
                expiration_date,
                implied_volatility,
            )
            .is_err()
        );
    }

    #[test]
    fn test_d1_zero_time_to_expiry() {
        // Case where time to expiry is zero
        let underlying_price = Positive::HUNDRED;
        let strike_price = Positive::HUNDRED;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ZERO;
        let implied_volatility = pos_or_panic!(0.2);

        // When time to expiry is zero, d1 should handle the case correctly
        assert!(
            d1(
                underlying_price,
                strike_price,
                carry_rate,
                expiration_date,
                implied_volatility,
            )
            .is_err()
        );
    }

    #[test]
    fn test_d1_high_volatility() {
        // Case with extremely high volatility
        let underlying_price = Positive::HUNDRED;
        let strike_price = Positive::HUNDRED;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ONE;
        let implied_volatility = Positive::HUNDRED; // Very high volatility

        // High volatility should result in a small or large value for d1
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            carry_rate,
            expiration_date,
            implied_volatility,
        )
        .unwrap()
        .to_f64()
        .unwrap();

        // Assert the result should be finite and non-infinite
        assert!(
            calculated_d1.is_finite(),
            "d1 should not be infinite for high volatility"
        );
    }

    #[test]
    fn test_d1_high_underlying_price() {
        // Case with extremely high underlying price
        let underlying_price = Positive::MAX; // Very high stock price
        let strike_price = Positive::HUNDRED;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ONE;
        let implied_volatility = pos_or_panic!(0.2);

        // Very high underlying price should result in a large d1 value
        assert!(
            d1(
                underlying_price,
                strike_price,
                carry_rate,
                expiration_date,
                implied_volatility,
            )
            .is_ok()
        );
    }

    #[test]
    fn test_d1_low_underlying_price() {
        // Case with extremely low underlying price (near zero)
        let underlying_price = pos_or_panic!(0.01); // Very low stock price
        let strike_price = Positive::HUNDRED;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ONE;
        let implied_volatility = pos_or_panic!(0.2);

        // Very low underlying price should result in a small or negative d1 value
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            carry_rate,
            expiration_date,
            implied_volatility,
        )
        .unwrap()
        .to_f64()
        .unwrap();

        // Assert the result should be finite and not infinite
        assert!(
            calculated_d1.is_finite(),
            "d1 should not be infinite for low underlying price"
        );
    }

    #[test]
    fn test_d1_zero_strike_price() {
        // Case where strike price is zero
        let underlying_price = Positive::HUNDRED;
        let strike_price = Positive::ZERO;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ONE;
        let implied_volatility = pos_or_panic!(0.2);

        // Since strike price is zero, the function should call handle_zero and return positive infinity
        assert!(
            d1(
                underlying_price,
                strike_price,
                carry_rate,
                expiration_date,
                implied_volatility,
            )
            .is_err()
        );
    }

    #[test]
    fn test_d1_infinite_risk_free_rate() {
        // Case where risk-free rate is very high (infinite-like)
        let underlying_price = Positive::HUNDRED;
        let strike_price = Positive::HUNDRED;
        let carry_rate = Decimal::MAX; // Very high cost of carry
        let expiration_date = Positive::ONE;
        let implied_volatility = pos_or_panic!(0.2);

        // High risk-free rate should result in a large d1 value, potentially infinite
        assert!(
            d1(
                underlying_price,
                strike_price,
                carry_rate,
                expiration_date,
                implied_volatility,
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod calculate_d1_values_bis {
    use super::*;
    use crate::error::greeks::{GreeksError, InputErrorKind};

    use approx::assert_relative_eq;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    // Helper function to convert Decimal to f64 for testing
    fn decimal_to_f64_test(d: Decimal) -> f64 {
        d.to_f64().unwrap()
    }

    #[test]
    fn test_d1_basic_calculation() {
        let result = d1(
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.8768025782891316, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_in_the_money() {
        let result = d1(
            pos_or_panic!(110.0),
            pos_or_panic!(90.0),
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 1.3533534773107558, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_out_of_the_money() {
        let result = d1(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, -0.1768025782891315, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_zero_strike_error() {
        let result = d1(
            Positive::HUNDRED,
            Positive::ZERO,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        );

        assert!(matches!(
            result,
            Err(GreeksError::InputError(
                InputErrorKind::InvalidStrike { .. }
            ))
        ));
    }

    #[test]
    fn test_d1_zero_volatility_error() {
        let result = d1(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            Positive::ZERO,
        );

        assert!(matches!(
            result,
            Err(GreeksError::InputError(
                InputErrorKind::InvalidVolatility { .. }
            ))
        ));
    }

    #[test]
    fn test_d1_zero_time_error() {
        let result = d1(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(0.2),
        );

        assert!(matches!(
            result,
            Err(GreeksError::InputError(InputErrorKind::InvalidTime { .. }))
        ));
    }

    #[test]
    fn test_d1_short_expiry() {
        let result = d1(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            pos_or_panic!(0.0833), // approximately one month
            pos_or_panic!(0.05),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.29583282863806715, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_high_volatility() {
        let result = d1(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.5), // 50% volatility
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.35, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_zero_interest_rate() {
        let result = d1(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.0),
            Positive::ONE,
            pos_or_panic!(0.5),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.25, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_negative_interest_rate() {
        let result = d1(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(-0.02), // negative interest rate
            Positive::ONE,
            pos_or_panic!(0.5),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.21, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_negative_interest_rate_bis() {
        let result = d1(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(-0.02), // negative interest rate
            Positive::ONE,
            pos_or_panic!(0.5),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.21, epsilon = 0.0001);
    }
}

#[cfg(test)]
mod calculate_d2_values {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use rust_decimal_macros::dec;

    #[test]
    fn test_d2_zero_volatility() {
        // Case where volatility (implied_volatility) is zero
        let underlying_price = Positive::HUNDRED;
        let strike_price = Positive::HUNDRED;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ONE;
        let implied_volatility = Positive::ZERO;

        // When volatility is zero, d2 should handle the case correctly using handle_zero
        assert!(
            d2(
                underlying_price,
                strike_price,
                carry_rate,
                expiration_date,
                implied_volatility,
            )
            .is_err()
        );
    }

    #[test]
    fn test_d2_zero_time_to_expiry() {
        // Case where time to expiration is zero
        let underlying_price = Positive::HUNDRED;
        let strike_price = Positive::HUNDRED;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ZERO;
        let implied_volatility = pos_or_panic!(0.2);

        // When time to expiration is zero, handle_zero should be called
        assert!(
            d2(
                underlying_price,
                strike_price,
                carry_rate,
                expiration_date,
                implied_volatility,
            )
            .is_err()
        );
    }

    #[test]
    fn test_d2_high_volatility() {
        // Case with extremely high volatility
        let underlying_price = Positive::HUNDRED;
        let strike_price = Positive::HUNDRED;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ONE;
        let implied_volatility = Positive::HUNDRED; // Very high volatility

        // High volatility should result in a significant negative shift in d2
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            carry_rate,
            expiration_date,
            implied_volatility,
        )
        .unwrap()
        .to_f64()
        .unwrap();

        // d2 should be finite and not infinite
        assert!(
            calculated_d2.is_finite(),
            "d2 should not be infinite for high volatility"
        );
    }

    #[test]
    fn test_d2_high_underlying_price() {
        // Case with extremely high underlying price
        let underlying_price = Positive::MAX;
        let strike_price = Positive::HUNDRED;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ONE;
        let implied_volatility = pos_or_panic!(0.2);

        // Very high underlying price should result in a large d2 value
        assert!(
            d2(
                underlying_price,
                strike_price,
                carry_rate,
                expiration_date,
                implied_volatility,
            )
            .is_ok()
        );
    }

    #[test]
    fn test_d2_low_underlying_price() {
        // Case with extremely low underlying price (near zero)
        let underlying_price = pos_or_panic!(0.01);
        let strike_price = Positive::HUNDRED;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ONE;
        let implied_volatility = pos_or_panic!(0.2);

        // Very low underlying price should result in a small or negative d2 value
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            carry_rate,
            expiration_date,
            implied_volatility,
        )
        .unwrap()
        .to_f64()
        .unwrap();

        // Assert the result should be finite and not infinite
        assert!(
            calculated_d2.is_finite(),
            "d2 should not be infinite for low underlying price"
        );
    }

    #[test]
    fn test_d2_zero_strike_price() {
        // Case where strike price is zero
        let underlying_price = Positive::HUNDRED;
        let strike_price = Positive::ZERO;
        let carry_rate = dec!(0.05); // cost of carry b = r - q (r if no dividends)
        let expiration_date = Positive::ONE;
        let implied_volatility = pos_or_panic!(0.2);

        // Since strike price is zero, the function should call handle_zero and return positive infinity
        assert!(
            d2(
                underlying_price,
                strike_price,
                carry_rate,
                expiration_date,
                implied_volatility,
            )
            .is_err()
        );
    }

    #[test]
    fn test_d2_infinite_risk_free_rate() {
        // Case where risk-free rate is very high (infinite-like)
        let underlying_price = Positive::HUNDRED;
        let strike_price = Positive::HUNDRED;
        let carry_rate = Decimal::MAX; // Very high cost of carry
        let expiration_date = Positive::ONE;
        let implied_volatility = pos_or_panic!(0.2);

        // High risk-free rate should result in a large d2 value, potentially infinite
        assert!(
            d2(
                underlying_price,
                strike_price,
                carry_rate,
                expiration_date,
                implied_volatility,
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod calculate_d2_values_bis {
    use super::*;
    use approx::assert_relative_eq;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(0.0001);
    // Normal test cases
    #[test]
    fn test_d2_atm_option() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), 0.15, epsilon = 0.0001);
    }

    #[test]
    fn test_d2_itm_call() {
        let result = d2(
            pos_or_panic!(110.0),
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_decimal_eq!(result, dec!(0.6265508990216243), EPSILON);
    }

    #[test]
    fn test_d2_otm_call() {
        let result = d2(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            -0.3768025782891315,
            epsilon = 0.0001
        );
    }

    // Time to expiration variations
    #[test]
    fn test_d2_short_expiry() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            pos_or_panic!(0.0833), // 1 month
            pos_or_panic!(0.5),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            -0.04329260906898544,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_d2_long_expiry() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::TWO,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            0.21213203435596426,
            epsilon = 0.0001
        );
    }

    // Volatility variations
    #[test]
    fn test_d2_low_volatility() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.1),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), 0.45, epsilon = 0.0001);
    }

    #[test]
    fn test_d2_high_volatility() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.5),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), -0.15, epsilon = 0.0001);
    }

    // Interest rate variations
    #[test]
    fn test_d2_zero_interest() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            Decimal::ZERO,
            Positive::ONE,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), -0.1, epsilon = 0.0001);
    }

    #[test]
    fn test_d2_high_interest() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.10),
            Positive::ONE,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), 0.4, epsilon = 0.0001);
    }

    // Extreme price differences
    #[test]
    fn test_d2_deep_itm() {
        let result = d2(
            pos_or_panic!(200.0),
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            3.6157359027997265,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_d2_deep_otm() {
        let result = d2(
            pos_or_panic!(50.0),
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            -3.3157359027997266,
            epsilon = 0.0001
        );
    }

    // Very small values
    #[test]
    fn test_d2_small_price() {
        let result = d2(
            pos_or_panic!(0.01),
            pos_or_panic!(0.01),
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), 0.15, epsilon = 0.0001);
    }

    #[test]
    fn test_d2_small_time() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            pos_or_panic!(0.001),
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            0.004743416490252569,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_d2_small_volatility() {
        let result = d2(
            pos_or_panic!(200.0),
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.01),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            74.30971805599454,
            epsilon = 0.0001
        );
    }

    // Error cases
    #[test]
    fn test_d2_zero_volatility() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            Positive::ZERO,
        );
        assert!(matches!(
            result,
            Err(GreeksError::InputError(
                InputErrorKind::InvalidVolatility { .. }
            ))
        ));
    }

    #[test]
    fn test_d2_zero_time() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(0.2),
        );
        assert!(matches!(
            result,
            Err(GreeksError::InputError(InputErrorKind::InvalidTime { .. }))
        ));
    }

    // Negative interest rate
    #[test]
    fn test_d2_negative_interest() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            -dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_decimal_eq!(result, dec!(-0.35), EPSILON);
    }

    // Combined extreme cases
    #[test]
    fn test_d2_combined_extremes_high() {
        let result = d2(
            pos_or_panic!(1000.0),
            Positive::HUNDRED,
            dec!(0.15),
            pos_or_panic!(5.0),
            pos_or_panic!(0.8),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            0.812019752759385,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_d2_combined_extremes_low() {
        let result = d2(
            pos_or_panic!(10.0),
            Positive::HUNDRED,
            dec!(0.01),
            pos_or_panic!(0.1),
            pos_or_panic!(0.05),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            -145.57292814518308,
            epsilon = 0.0001
        );
    }

    // Edge cases with very large numbers
    #[test]
    fn test_d2_large_price_ratio() {
        let result = d2(
            pos_or_panic!(1_000_000.0),
            Positive::ONE,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            69.22755278982137,
            epsilon = 0.0001
        );
    }

    // Special case: ATM LEAPS (Long-term equity anticipation securities)
    #[test]
    fn test_d2_leaps() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            pos_or_panic!(2.5), // 2.5 years
            pos_or_panic!(0.15),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            0.40846086443841567,
            epsilon = 0.0001
        );
    }

    // Near-zero but valid cases
    #[test]
    fn test_d2_near_zero_valid_values() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.0001),
            pos_or_panic!(0.01),
            pos_or_panic!(0.001),
        )
        .unwrap();
        assert!(result.to_f64().unwrap().abs() < 1.0);
    }

    // Test with maximum realistic market values
    #[test]
    fn test_d2_max_realistic_values() {
        let result = d2(
            pos_or_panic!(10000.0),
            pos_or_panic!(5000.0),
            dec!(0.20),
            pos_or_panic!(3.0),
            pos_or_panic!(1.5),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            -0.8013055238112647,
            epsilon = 0.0001
        );
    }
}

#[cfg(test)]
mod calculate_big_n_values {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;
    use statrs::distribution::Normal;

    #[test]
    fn test_big_n_zero() {
        // Case where x = 0.0
        let x = Decimal::ZERO;

        // The CDF of the standard normal distribution at x = 0 is 0.5
        let expected_big_n = 0.5;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_one() {
        // Case where x = 0.0
        let x = Decimal::ONE;

        // The CDF of the standard normal distribution at x = 1 is 0.841344746054943
        let expected_big_n = 0.841344746054943;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_two() {
        // Case where x = 0.0
        let x = Decimal::TWO;

        // The CDF of the standard normal distribution at x = 2 is 0.977249868052837
        let expected_big_n = 0.977249868052837;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_positive_small_value() {
        // Case where x is a small positive value
        let x = dec!(0.5);

        // The expected CDF for the standard normal distribution at x = 0.5 can be precomputed
        let normal_distribution = Normal::new(0.0, 1.0).unwrap();
        let expected_big_n = normal_distribution.cdf(x.to_f64().unwrap());

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_negative_small_value() {
        // Case where x is a small negative value
        let x = -dec!(0.5);

        // The expected CDF for the standard normal distribution at x = -0.5 can be precomputed
        let normal_distribution = Normal::new(0.0, 1.0).unwrap();
        let expected_big_n = normal_distribution.cdf(x.to_f64().unwrap());

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_large_positive_value() {
        // Case where x is a large positive value
        let x = dec!(5.0);

        // The CDF for large positive x should be very close to 1
        let expected_big_n = 1.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-6); // if lower epsilon fail
    }

    #[test]
    fn test_big_n_large_negative_value() {
        // Case where x is a large negative value
        let x = -dec!(5.0);

        // The CDF for large negative x should be very close to 0
        let expected_big_n = 0.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-6); // if lower epsilon fail
    }

    #[test]
    fn test_big_n_extreme_positive_value() {
        // Case where x is an extremely large positive value
        let x = dec!(100.0);

        // The CDF for an extremely large positive x should be effectively 1
        let expected_big_n = 1.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that big_n(x) is effectively 1 for such a large positive input
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-12);
    }

    #[test]
    fn test_big_n_extreme_negative_value() {
        // Case where x is an extremely large negative value
        let x = -dec!(100.0);

        // The CDF for an extremely large negative x should be effectively 0
        let expected_big_n = 0.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that big_n(x) is effectively 0 for such a large negative input
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-12);
    }
}

#[cfg(test)]
mod tests_d1_d2_edge_cases {
    use super::*;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    #[test]
    fn test_d1_zero_underlying_price() {
        let result = d1(
            Positive::ZERO,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(0.2),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_d2_negative_rates_and_high_volatility() {
        let result = d2(
            Positive::HUNDRED,
            Positive::HUNDRED,
            -dec!(0.05), // tasa negativa
            Positive::ONE,
            pos_or_panic!(0.8), // alta volatilidad
        )
        .unwrap();
        assert_decimal_eq!(result, dec!(-0.4625), dec!(0.000001));
    }

    #[test]
    fn test_d1_d2_combination_extreme_values() {
        let result_d1 = d1(
            pos_or_panic!(1000.0),
            pos_or_panic!(10.0),
            dec!(0.15),
            Positive::TEN,
            pos_or_panic!(0.9),
        )
        .unwrap();
        let result_d2 = d2(
            pos_or_panic!(1000.0),
            pos_or_panic!(10.0),
            dec!(0.15),
            Positive::TEN,
            pos_or_panic!(0.9),
        )
        .unwrap();
        assert_decimal_eq!(result_d1, dec!(3.5681), dec!(0.0001));
        assert_decimal_eq!(result_d2, dec!(0.7221), dec!(0.0001));
    }
}

#[cfg(test)]
mod tests_cumulative_distribution {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_big_n_continuity() {
        let x1 = dec!(0.001);
        let x2 = dec!(-0.001);
        let result1 = big_n(x1).unwrap();
        let result2 = big_n(x2).unwrap();
        assert!((result1 - result2).abs() < dec!(0.001));
    }

    #[test]
    fn test_big_n_conversion_error() {
        let x = Decimal::MAX;
        let result = big_n(x);
        assert!(result.is_ok());
    }

    #[test]
    fn test_big_n_boundary_values() {
        let result_zero = big_n(dec!(0.0)).unwrap();
        assert_eq!(result_zero, dec!(0.5));
    }
}

#[cfg(test)]
mod tests_calculate_d_values_bis {
    use super::*;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_d_values_with_expiration() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: Positive::HUNDRED,
            underlying_price: Positive::HUNDRED,
            risk_free_rate: dec!(0.05),
            implied_volatility: pos_or_panic!(0.5),
            expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
            quantity: Positive::ONE,
            contract_size: Positive::ONE,
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        };
        let (d1, d2) = calculate_d_values(&option).unwrap();
        assert_decimal_eq!(d1, dec!(0.1003), dec!(0.0001));
        assert_decimal_eq!(d2, dec!(-0.0430), dec!(0.0001));
    }
}

#[cfg(test)]
mod tests_edge_cases_and_errors {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use rust_decimal_macros::dec;

    #[test]
    fn test_extreme_volatility_values() {
        let result = d1(
            Positive::HUNDRED,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ONE,
            pos_or_panic!(1000.0),
        );
        assert!(result.is_ok());
    }
}

/// Cross-checks that the price and Greek paths read the same shared kernels
/// with the same parameters (#523).
#[cfg(test)]
mod tests_price_greek_consistency {
    use super::*;
    use crate::greeks;
    use crate::pricing;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    /// A live European option with a non-zero dividend yield, so the
    /// Black-Scholes carry `b = r - q` differs from the Black-76 `b = 0`.
    fn option(style: OptionStyle) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "XCHK".to_string(),
            pos_or_panic!(95.0),
            ExpirationDate::Days(pos_or_panic!(120.0)),
            pos_or_panic!(0.27),
            Positive::ONE,
            pos_or_panic!(102.5),
            dec!(0.045),
            style,
            pos_or_panic!(0.018),
            None,
        )
    }

    fn ok<T, E: std::fmt::Debug>(label: &str, result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{label} failed: {error:?}"),
        }
    }

    fn years(option: &Options) -> Positive {
        ok("years", option.expiration_date.get_years())
    }

    /// The Greek-side `d1`/`d2`, called through the public `greeks` path
    /// with the carry the Greeks use, `b = r - q`.
    fn greek_d_values(option: &Options, carry: Decimal) -> (Decimal, Decimal) {
        let t = years(option);
        let d1_value = ok(
            "greeks::d1",
            greeks::d1(
                option.underlying_price,
                option.strike_price,
                carry,
                t,
                option.implied_volatility,
            ),
        );
        let d2_value = ok(
            "greeks::d2",
            greeks::d2(
                option.underlying_price,
                option.strike_price,
                carry,
                t,
                option.implied_volatility,
            ),
        );
        (d1_value, d2_value)
    }

    #[test]
    fn test_black_scholes_pricer_d_values_equal_greek_d1_d2_exactly() {
        let call = option(OptionStyle::Call);
        let (price_d1, price_d2, price_t) =
            ok("pricer d", d_values_and_time(&call, calculate_d_values));
        let carry = call.risk_free_rate - call.dividend_yield.to_dec();
        let (greek_d1, greek_d2) = greek_d_values(&call, carry);
        assert_eq!(price_d1, greek_d1);
        assert_eq!(price_d2, greek_d2);
        assert_eq!(price_t, years(&call).to_dec());
    }

    #[test]
    fn test_black_76_pricer_d_values_equal_greek_d1_d2_with_zero_carry_exactly() {
        let call = option(OptionStyle::Call);
        let (price_d1, price_d2, _) = ok(
            "pricer d",
            d_values_and_time(&call, calculate_d_values_black_76),
        );
        let (greek_d1, greek_d2) = greek_d_values(&call, Decimal::ZERO);
        assert_eq!(price_d1, greek_d1);
        assert_eq!(price_d2, greek_d2);
        // The public Black-76 path returns the same pair.
        let public = greeks::calculate_d_values_black_76(&call);
        assert!(matches!(public, Ok((d1v, d2v)) if d1v == greek_d1 && d2v == greek_d2));
    }

    /// Rebuilds `S e^(-qT) N(d1) - K e^(-rT) N(d2)` (and the put) from the
    /// Greek-side `d1`/`d2` and the shared kernels, step for step as the
    /// pricer does, and requires the pricer's result to the last digit.
    #[test]
    fn test_black_scholes_price_rebuilt_from_greek_d_values_equals_pricer() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let opt = option(style);
            let t = years(&opt).to_dec();
            let carry = opt.risk_free_rate - opt.dividend_yield.to_dec();
            let (d1v, d2v) = greek_d_values(&opt, carry);
            let discount_q = ok(
                "exp(-qT)",
                discount_factor(opt.dividend_yield.to_dec(), t, "test::qt", "test::exp_qt"),
            );
            let discount_r = ok(
                "exp(-rT)",
                discount_factor(opt.risk_free_rate, t, "test::rt", "test::exp_rt"),
            );
            let s_pv = ok(
                "s_pv",
                d_mul(opt.underlying_price.to_dec(), discount_q, "t"),
            );
            let expected = match style {
                OptionStyle::Call => {
                    let k_pv = ok("k_pv", d_mul(discount_r, opt.strike_price.to_dec(), "t"));
                    let s_leg = ok("s_leg", d_mul(s_pv, ok("N(d1)", big_n(d1v)), "t"));
                    let k_leg = ok("k_leg", d_mul(k_pv, ok("N(d2)", big_n(d2v)), "t"));
                    ok("call", d_sub(s_leg, k_leg, "t"))
                }
                OptionStyle::Put => {
                    let k_pv = ok("k_pv", d_mul(opt.strike_price.to_dec(), discount_r, "t"));
                    let k_leg = ok("k_leg", d_mul(k_pv, ok("N(-d2)", big_n(-d2v)), "t"));
                    let s_leg = ok("s_leg", d_mul(s_pv, ok("N(-d1)", big_n(-d1v)), "t"));
                    ok("put", d_sub(k_leg, s_leg, "t"))
                }
            };
            let priced = pricing::black_scholes(&opt);
            assert!(
                matches!(priced, Ok(value) if value == expected),
                "{style:?}: pricer {priced:?} != rebuilt {expected}"
            );
        }
    }

    /// `e^(-rT) [F N(d1) - K N(d2)]` (and the put) from the Greek-side
    /// zero-carry `d1`/`d2` and the shared discount kernel.
    #[test]
    fn test_black_76_price_rebuilt_from_greek_d_values_equals_pricer() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let opt = option(style);
            let t = years(&opt).to_dec();
            let (d1v, d2v) = greek_d_values(&opt, Decimal::ZERO);
            let df = ok(
                "exp(-rT)",
                discount_factor(opt.risk_free_rate, t, "test::rt", "test::df"),
            );
            let f = opt.underlying_price.to_dec();
            let k = opt.strike_price.to_dec();
            let undiscounted = match style {
                OptionStyle::Call => ok(
                    "call",
                    d_sub(
                        ok("f_leg", d_mul(f, ok("N(d1)", big_n(d1v)), "t")),
                        ok("k_leg", d_mul(k, ok("N(d2)", big_n(d2v)), "t")),
                        "t",
                    ),
                ),
                OptionStyle::Put => ok(
                    "put",
                    d_sub(
                        ok("k_leg", d_mul(k, ok("N(-d2)", big_n(-d2v)), "t")),
                        ok("f_leg", d_mul(f, ok("N(-d1)", big_n(-d1v)), "t")),
                        "t",
                    ),
                ),
            };
            let expected = ok("price", d_mul(df, undiscounted, "t"));
            let priced = pricing::black_76(&opt);
            assert!(
                matches!(priced, Ok(value) if value == expected),
                "{style:?}: pricer {priced:?} != rebuilt {expected}"
            );
        }
    }

    /// Black-Scholes put-call parity on the pricer, against the shared
    /// discount kernel: `C - P = S e^(-qT) - K e^(-rT)`.
    #[test]
    fn test_black_scholes_put_call_parity_uses_shared_discount_factors() {
        let call = option(OptionStyle::Call);
        let put = option(OptionStyle::Put);
        let t = years(&call).to_dec();
        let c = ok("call", pricing::black_scholes(&call));
        let p = ok("put", pricing::black_scholes(&put));
        let dq = ok(
            "dq",
            discount_factor(call.dividend_yield.to_dec(), t, "t", "t"),
        );
        let dr = ok("dr", discount_factor(call.risk_free_rate, t, "t", "t"));
        let forward_gap = call.underlying_price.to_dec() * dq - call.strike_price.to_dec() * dr;
        assert!(
            (c - p - forward_gap).abs() < dec!(1e-12),
            "C - P = {} expected {forward_gap}",
            c - p
        );
    }

    /// Black-Scholes delta parity: `Δ_call - Δ_put = e^(-qT)`.
    #[test]
    fn test_black_scholes_delta_parity_equals_dividend_discount_factor() {
        let call = option(OptionStyle::Call);
        let put = option(OptionStyle::Put);
        let t = years(&call).to_dec();
        let dc = ok("delta call", greeks::delta(&call));
        let dp = ok("delta put", greeks::delta(&put));
        let dq = ok(
            "dq",
            discount_factor(call.dividend_yield.to_dec(), t, "t", "t"),
        );
        assert!(
            (dc - dp - dq).abs() < dec!(1e-20),
            "Δc - Δp = {} expected {dq}",
            dc - dp
        );
    }

    /// Black-76 delta parity: `Δ_call - Δ_put = e^(-rT)`.
    #[test]
    fn test_black_76_delta_parity_equals_shared_discount_factor() {
        let call = option(OptionStyle::Call);
        let put = option(OptionStyle::Put);
        let t = years(&call).to_dec();
        let dc = ok("delta call", greeks::delta_b76(&call));
        let dp = ok("delta put", greeks::delta_b76(&put));
        let dr = ok("dr", discount_factor(call.risk_free_rate, t, "t", "t"));
        assert!(
            (dc - dp - dr).abs() < dec!(1e-12),
            "Δc - Δp = {} expected {dr}",
            dc - dp
        );
    }

    /// Garman-Kohlhagen spot-delta parity: `Δ_call - Δ_put = e^(-r_f T)`,
    /// with `r_f` carried in `dividend_yield`.
    #[test]
    fn test_garman_kohlhagen_delta_parity_equals_foreign_discount_factor() {
        let call = option(OptionStyle::Call);
        let put = option(OptionStyle::Put);
        let t = years(&call).to_dec();
        let dc = ok("delta call", greeks::delta_gk(&call));
        let dp = ok("delta put", greeks::delta_gk(&put));
        let df = ok(
            "df",
            discount_factor(call.dividend_yield.to_dec(), t, "t", "t"),
        );
        assert!(
            (dc - dp - df).abs() < dec!(1e-12),
            "Δc - Δp = {} expected {df}",
            dc - dp
        );
    }

    /// The Garman-Kohlhagen pricer and the Black-Scholes pricer agree to the
    /// last digit, and the GK and BSM deltas agree within `N` round-off: both
    /// read `d1` with `b = r_d - r_f` and discount by `e^(-r_f T)`.
    #[test]
    fn test_garman_kohlhagen_price_and_delta_match_black_scholes() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let opt = option(style);
            let gk = pricing::garman_kohlhagen(&opt);
            let bs = pricing::black_scholes(&opt);
            assert!(
                matches!((&gk, &bs), (Ok(a), Ok(b)) if a == b),
                "{gk:?} vs {bs:?}"
            );
            let gk_delta = ok("delta_gk", greeks::delta_gk(&opt));
            let bs_delta = ok("delta", greeks::delta(&opt));
            assert!(
                (gk_delta - bs_delta).abs() < dec!(1e-12),
                "{style:?}: {gk_delta} vs {bs_delta}"
            );
        }
    }

    /// Bump-and-reprice delta (through the Black-Scholes pricer) agrees with
    /// the closed-form delta (through the Greek kernels): the two paths see
    /// the same `d1` and carry.
    #[test]
    fn test_numerical_delta_through_pricer_matches_closed_form_delta() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let opt = option(style);
            let numerical = ok("numerical", greeks::numerical::numerical_delta(&opt));
            let analytic = ok("delta", greeks::delta(&opt));
            assert!(
                (numerical - analytic).abs() < dec!(1e-5),
                "{style:?}: numerical {numerical} vs analytic {analytic}"
            );
        }
    }

    #[test]
    fn test_discount_factor_matches_inline_steps_and_reports_overflow_label() {
        let inline = d_exp(ok("exponent", d_mul(-dec!(0.045), dec!(0.75), "a")), "b");
        let kernel = discount_factor(dec!(0.045), dec!(0.75), "a", "b");
        assert!(matches!((&inline, &kernel), (Ok(x), Ok(y)) if x == y));
        let overflow = discount_factor(Decimal::MIN, Decimal::MAX, "test::exponent", "test::exp");
        assert!(
            matches!(&overflow, Err(error) if error.to_string().contains("test::exponent")),
            "{overflow:?}"
        );
        // A negative exponent below the representable scale flushes to zero.
        let flushed = discount_factor(dec!(1000), Decimal::ONE, "a", "b");
        assert!(matches!(flushed, Ok(value) if value.is_zero()));
    }
}

/// #859: the d-value helpers compute `d1` once and derive `d2` from it. The
/// public [`d2`] still recomputes `d1`, so it is the reference the helpers
/// must reproduce digit for digit on every point of the grid, including the
/// points where the inputs are rejected.
#[cfg(test)]
mod tests_d_values_single_d1 {
    use super::*;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    fn option(spot: Positive, sigma: Positive, days: Positive, rate: Decimal) -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "GRID".to_string(),
            strike_price: Positive::HUNDRED,
            expiration_date: ExpirationDate::Days(days),
            implied_volatility: sigma,
            quantity: Positive::ONE,
            contract_size: Positive::ONE,
            underlying_price: spot,
            risk_free_rate: rate,
            option_style: OptionStyle::Call,
            dividend_yield: pos_or_panic!(0.02),
            exotic_params: None,
        }
    }

    /// `(d1, d2)` through the public `d1` and `d2`, as the helpers computed
    /// them before #859; an error is kept as its message.
    fn reference(option: &Options, carry: Decimal) -> Result<(Decimal, Decimal), String> {
        let years = option
            .expiration_date
            .get_years()
            .map_err(|e| e.to_string())?;
        let args = (
            option.underlying_price,
            option.strike_price,
            carry,
            years,
            option.implied_volatility,
        );
        let d1_value = d1(args.0, args.1, args.2, args.3, args.4);
        let d2_value = d2(args.0, args.1, args.2, args.3, args.4);
        match (d1_value, d2_value) {
            (Ok(a), Ok(b)) => Ok((a, b)),
            (Err(e), _) | (_, Err(e)) => Err(e.to_string()),
        }
    }

    #[test]
    fn test_d_values_bit_identical_to_the_d2_path_on_a_grid() {
        let spots = [
            pos_or_panic!(60.0),
            pos_or_panic!(95.5),
            Positive::HUNDRED,
            pos_or_panic!(137.25),
        ];
        let sigmas = [
            Positive::ZERO,
            pos_or_panic!(0.05),
            pos_or_panic!(0.2),
            pos_or_panic!(0.85),
        ];
        let days = [
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(30.0),
            pos_or_panic!(365.0),
            pos_or_panic!(1825.0),
        ];
        let rates = [dec!(-0.01), Decimal::ZERO, dec!(0.05)];
        let mut checked = 0_u32;
        for &spot in &spots {
            for &sigma in &sigmas {
                for &day in &days {
                    for &rate in &rates {
                        let option = option(spot, sigma, day, rate);
                        let q = option.dividend_yield.to_dec();
                        let carry = rate - q;
                        let with_yield =
                            calculate_d_values_with_yield(&option, q).map_err(|e| e.to_string());
                        assert_eq!(
                            with_yield,
                            reference(&option, carry),
                            "{spot} {sigma} {day} {rate}"
                        );
                        let plain = calculate_d_values(&option).map_err(|e| e.to_string());
                        assert_eq!(plain, with_yield, "{spot} {sigma} {day} {rate}");
                        let black_76 =
                            calculate_d_values_black_76(&option).map_err(|e| e.to_string());
                        assert_eq!(
                            black_76,
                            reference(&option, Decimal::ZERO),
                            "{spot} {sigma} {day} {rate}"
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(checked, 240);
    }
}
