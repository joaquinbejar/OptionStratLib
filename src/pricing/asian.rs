/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

//! Asian option pricing module.
//!
//! Asian options are path-dependent options where the payoff depends on the
//! average price of the underlying asset over a specified period. This module
//! implements pricing for both geometric and arithmetic averaging.
//!
//! # Averaging Types
//!
//! - **Geometric Average**: Uses geometric mean of prices. Has a closed-form
//!   Black-Scholes solution with adjusted volatility and drift.
//! - **Arithmetic Average**: Uses arithmetic mean of prices. No closed-form
//!   solution exists; uses Turnbull-Wakeman approximation.
//!
//! # Formula Sources
//!
//! - Kemna & Vorst (1990) for geometric average Asian options
//! - Turnbull & Wakeman (1991) for arithmetic average approximation

use crate::Options;
use crate::error::PricingError;
use crate::greeks::{big_n, d1, d2};
use crate::model::decimal::{d_add, d_div, d_exp, d_ln, d_mul, d_powd, d_sqrt, d_sub};
use crate::model::types::{AsianAveragingType, OptionStyle, OptionType};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

/// Prices an Asian option using the appropriate method based on averaging type.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Asian`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// - [`PricingError::MethodError`] when the option type is not
///   `OptionType::Asian`, when the averaging type is an unsupported
///   `#[non_exhaustive]` variant, or when the expiration cannot be converted
///   to a year fraction.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range: the discount factor, the forward, either
///   Turnbull-Wakeman moment, the moment-matched variance, or the final
///   Black legs.
/// - [`PricingError::Positive`] when the `σ / √3` geometric adjustment is not
///   representable as a `Positive`.
pub fn asian_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Asian { averaging_type } => match averaging_type {
            AsianAveragingType::Geometric => geometric_asian_price(option),
            AsianAveragingType::Arithmetic => arithmetic_asian_price(option),
            // `AsianAveragingType` is `#[non_exhaustive]`.
            _ => Err(PricingError::other(
                "asian_black_scholes: unsupported AsianAveragingType",
            )),
        },
        _ => Err(PricingError::other(
            "asian_black_scholes requires OptionType::Asian",
        )),
    }
}

/// Prices a geometric average Asian option using closed-form Black-Scholes.
///
/// Uses the Kemna-Vorst (1990) closed-form solution. The geometric average
/// of a lognormal process is also lognormal, allowing for an analytical solution.
///
/// # Adjustments
///
/// For geometric averaging:
/// - Adjusted volatility: `σ_adj = σ / √3`
/// - Adjusted cost-of-carry: `b_adj = (r - q - σ²/6) / 2`
fn geometric_asian_price(option: &Options) -> Result<Decimal, PricingError> {
    let s = option.underlying_price;
    let k = option.strike_price;
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let t = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    if t == Positive::ZERO {
        return intrinsic_value(option);
    }

    if sigma == Positive::ZERO {
        // Deterministic case: the average of a driftless-variance path is the
        // forward itself, so the option collapses to a discounted intrinsic.
        let t_dec = t.to_dec();
        let discount = d_exp(
            d_mul(-r, t_dec, "pricing::asian::geometric::det::neg_rt")?,
            "pricing::asian::geometric::det::discount",
        )?;
        let carry = d_sub(r, q, "pricing::asian::geometric::det::carry")?;
        let forward = d_mul(
            s.to_dec(),
            d_exp(
                d_mul(carry, t_dec, "pricing::asian::geometric::det::carry_t")?,
                "pricing::asian::geometric::det::growth",
            )?,
            "pricing::asian::geometric::det::forward",
        )?;
        let intrinsic = match option.option_style {
            OptionStyle::Call => {
                d_sub(forward, k.to_dec(), "pricing::asian::geometric::det::call")?
                    .max(Decimal::ZERO)
            }
            OptionStyle::Put => d_sub(k.to_dec(), forward, "pricing::asian::geometric::det::put")?
                .max(Decimal::ZERO),
        };
        let price = d_mul(intrinsic, discount, "pricing::asian::geometric::det::price")?;
        return Ok(apply_side(price, option));
    }

    // Geometric average adjustments (Kemna-Vorst)
    let sigma_sq = d_mul(
        sigma.to_dec(),
        sigma.to_dec(),
        "pricing::asian::geometric::sigma_sq",
    )?;
    let sqrt_three = Positive::new(3.0_f64.sqrt())
        .map_err(|e| PricingError::method_error("geometric_asian_price", &e.to_string()))?;
    let sigma_adj = Positive::new_decimal(d_div(
        sigma.to_dec(),
        sqrt_three.to_dec(),
        "pricing::asian::geometric::sigma_adj",
    )?)?;
    let b_adj = d_div(
        d_sub(
            d_sub(r, q, "pricing::asian::geometric::carry")?,
            d_div(
                sigma_sq,
                dec!(6),
                "pricing::asian::geometric::variance_drag",
            )?,
            "pricing::asian::geometric::b_numerator",
        )?,
        dec!(2),
        "pricing::asian::geometric::b_adj",
    )?;

    // Calculate d1 and d2 with adjusted parameters
    let d1_val = d1(s, k, b_adj, t, sigma_adj)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b_adj, t, sigma_adj)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let t_dec = t.to_dec();
    let discount = d_exp(
        d_mul(-r, t_dec, "pricing::asian::geometric::neg_rt")?,
        "pricing::asian::geometric::discount",
    )?;
    // e^((b_adj - r) T): the geometric-average carry replaces the spot drift.
    let carry_discount = d_exp(
        d_mul(
            d_sub(b_adj, r, "pricing::asian::geometric::carry_spread")?,
            t_dec,
            "pricing::asian::geometric::carry_spread_t",
        )?,
        "pricing::asian::geometric::carry_discount",
    )?;
    let s_leg = d_mul(
        s.to_dec(),
        carry_discount,
        "pricing::asian::geometric::spot_leg",
    )?;
    let k_leg = d_mul(
        k.to_dec(),
        discount,
        "pricing::asian::geometric::strike_leg",
    )?;

    let price = match option.option_style {
        OptionStyle::Call => {
            let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
            let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
            d_sub(
                d_mul(s_leg, n_d1, "pricing::asian::geometric::call::spot")?,
                d_mul(k_leg, n_d2, "pricing::asian::geometric::call::strike")?,
                "pricing::asian::geometric::call",
            )?
        }
        OptionStyle::Put => {
            let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
            let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);
            d_sub(
                d_mul(k_leg, n_neg_d2, "pricing::asian::geometric::put::strike")?,
                d_mul(s_leg, n_neg_d1, "pricing::asian::geometric::put::spot")?,
                "pricing::asian::geometric::put",
            )?
        }
    };

    Ok(apply_side(price, option))
}

/// Prices an arithmetic average Asian option using Turnbull-Wakeman approximation.
///
/// The arithmetic average of a lognormal process is not lognormal, so no
/// closed-form solution exists. This implementation uses the Turnbull-Wakeman
/// (1991) approximation which matches the first two moments of the arithmetic
/// average to a lognormal distribution.
fn arithmetic_asian_price(option: &Options) -> Result<Decimal, PricingError> {
    let s = option.underlying_price;
    let k = option.strike_price;
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let t = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    if t == Positive::ZERO {
        return intrinsic_value(option);
    }

    let t_dec = t.to_dec();
    let discount = d_exp(
        d_mul(-r, t_dec, "pricing::asian::arithmetic::neg_rt")?,
        "pricing::asian::arithmetic::discount",
    )?;

    if sigma == Positive::ZERO {
        let carry = d_sub(r, q, "pricing::asian::arithmetic::det::carry")?;
        let forward = d_mul(
            s.to_dec(),
            d_exp(
                d_mul(carry, t_dec, "pricing::asian::arithmetic::det::carry_t")?,
                "pricing::asian::arithmetic::det::growth",
            )?,
            "pricing::asian::arithmetic::det::forward",
        )?;
        return deterministic_price(forward, k.to_dec(), discount, option);
    }

    // Turnbull-Wakeman approximation
    let b = d_sub(r, q, "pricing::asian::arithmetic::carry")?; // cost of carry
    let sigma_dec = sigma.to_dec();
    let sigma_sq = d_mul(sigma_dec, sigma_dec, "pricing::asian::arithmetic::sigma_sq")?;
    let s_dec = s.to_dec();
    let s_sq = d_powd(s_dec, Decimal::TWO, "pricing::asian::arithmetic::s_sq")?;
    let t_sq = d_powd(t_dec, Decimal::TWO, "pricing::asian::arithmetic::t_sq")?;
    let bt = d_mul(b, t_dec, "pricing::asian::arithmetic::bt")?;
    let two_b_plus_var = d_add(
        d_mul(dec!(2), b, "pricing::asian::arithmetic::two_b")?,
        sigma_sq,
        "pricing::asian::arithmetic::two_b_plus_var",
    )?;
    let b_plus_var = d_add(b, sigma_sq, "pricing::asian::arithmetic::b_plus_var")?;

    // First moment of arithmetic average (M1)
    let m1 = if b.abs() < dec!(1e-10) {
        s_dec
    } else {
        d_mul(
            s_dec,
            d_div(
                d_sub(
                    d_exp(bt, "pricing::asian::arithmetic::m1::exp_bt")?,
                    dec!(1),
                    "pricing::asian::arithmetic::m1::numerator",
                )?,
                bt,
                "pricing::asian::arithmetic::m1::ratio",
            )?,
            "pricing::asian::arithmetic::m1",
        )?
    };

    // Second moment of arithmetic average (M2)
    let m2 = if b.abs() < dec!(1e-10) {
        let term = d_exp(
            d_mul(sigma_sq, t_dec, "pricing::asian::arithmetic::m2::var_t")?,
            "pricing::asian::arithmetic::m2::exp_var_t",
        )?;
        d_mul(s_sq, term, "pricing::asian::arithmetic::m2::driftless")?
    } else {
        let term1_exp = d_exp(
            d_mul(
                two_b_plus_var,
                t_dec,
                "pricing::asian::arithmetic::m2::term1_exponent",
            )?,
            "pricing::asian::arithmetic::m2::term1_exp",
        )?;
        let term1 = d_div(
            d_mul(
                d_mul(dec!(2), s_sq, "pricing::asian::arithmetic::m2::two_s_sq")?,
                term1_exp,
                "pricing::asian::arithmetic::m2::term1_numerator",
            )?,
            d_mul(
                d_mul(
                    b_plus_var,
                    two_b_plus_var,
                    "pricing::asian::arithmetic::m2::term1_roots",
                )?,
                t_sq,
                "pricing::asian::arithmetic::m2::term1_denominator",
            )?,
            "pricing::asian::arithmetic::m2::term1",
        )?;

        let term2 = d_mul(
            d_div(
                d_mul(dec!(2), s_sq, "pricing::asian::arithmetic::m2::two_s_sq2")?,
                d_mul(b, t_sq, "pricing::asian::arithmetic::m2::b_t_sq")?,
                "pricing::asian::arithmetic::m2::term2_scale",
            )?,
            d_sub(
                d_div(
                    dec!(1),
                    two_b_plus_var,
                    "pricing::asian::arithmetic::m2::term2_first",
                )?,
                d_div(
                    d_exp(bt, "pricing::asian::arithmetic::m2::exp_bt")?,
                    b_plus_var,
                    "pricing::asian::arithmetic::m2::term2_second",
                )?,
                "pricing::asian::arithmetic::m2::term2_bracket",
            )?,
            "pricing::asian::arithmetic::m2::term2",
        )?;

        d_add(term1, term2, "pricing::asian::arithmetic::m2")?
    };

    // Forward price of the average
    let f_adj = m1;

    // A non-positive first moment leaves the moment matching undefined: the
    // average is known to be `f_adj`, so the option is worth its discounted
    // intrinsic (the σ → 0 limit of the Black formula below).
    if f_adj <= Decimal::ZERO {
        return deterministic_price(f_adj, k.to_dec(), discount, option);
    }

    // Adjusted volatility from moment matching
    let m1_sq = d_powd(m1, Decimal::TWO, "pricing::asian::arithmetic::m1_sq")?;
    let moment_ratio = d_div(m2, m1_sq, "pricing::asian::arithmetic::moment_ratio")?;
    let variance = if moment_ratio > Decimal::ZERO {
        d_div(
            d_ln(moment_ratio, "pricing::asian::arithmetic::log_moment_ratio")?,
            t_dec,
            "pricing::asian::arithmetic::variance",
        )?
    } else {
        // `M2 / M1²` collapsed below the representable scale: the matched
        // lognormal has no spread left, which the `sqrt` fallback below maps
        // back onto the input volatility.
        Decimal::ZERO
    };
    let sigma_adj = variance.sqrt().unwrap_or(sigma_dec);

    // Use Black-Scholes with adjusted parameters
    let sqrt_t = d_sqrt(t_dec, "pricing::asian::arithmetic::sqrt_t")?;
    let denominator = d_mul(sigma_adj, sqrt_t, "pricing::asian::arithmetic::denominator")?;
    if denominator.is_zero() {
        // Zero matched volatility (or zero maturity in the limit): the average
        // is deterministic and the price is the discounted intrinsic.
        return deterministic_price(f_adj, k.to_dec(), discount, option);
    }

    let moneyness = d_div(f_adj, k.to_dec(), "pricing::asian::arithmetic::moneyness")?;
    let log_moneyness = if moneyness.is_zero() {
        // `F / K` rounded below the smallest representable `Decimal`, so the
        // logarithm diverges; `Decimal::MIN` is the representable stand-in and
        // drives `big_n` to the same 0 / 1 saturation the limit produces.
        Decimal::MIN
    } else {
        d_ln(moneyness, "pricing::asian::arithmetic::log_moneyness")?
    };
    let half_variance_t = d_div(
        d_mul(
            d_mul(
                sigma_adj,
                sigma_adj,
                "pricing::asian::arithmetic::sigma_adj_sq",
            )?,
            t_dec,
            "pricing::asian::arithmetic::variance_t",
        )?,
        dec!(2),
        "pricing::asian::arithmetic::half_variance_t",
    )?;
    let d1_val = d_div(
        d_add(
            log_moneyness,
            half_variance_t,
            "pricing::asian::arithmetic::d1_numerator",
        )?,
        denominator,
        "pricing::asian::arithmetic::d1",
    )?;
    let d2_val = d_sub(d1_val, denominator, "pricing::asian::arithmetic::d2")?;

    let price = match option.option_style {
        OptionStyle::Call => {
            let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
            let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
            d_mul(
                discount,
                d_sub(
                    d_mul(f_adj, n_d1, "pricing::asian::arithmetic::call::forward")?,
                    d_mul(k.to_dec(), n_d2, "pricing::asian::arithmetic::call::strike")?,
                    "pricing::asian::arithmetic::call::intrinsic",
                )?,
                "pricing::asian::arithmetic::call",
            )?
        }
        OptionStyle::Put => {
            let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
            let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);
            d_mul(
                discount,
                d_sub(
                    d_mul(
                        k.to_dec(),
                        n_neg_d2,
                        "pricing::asian::arithmetic::put::strike",
                    )?,
                    d_mul(f_adj, n_neg_d1, "pricing::asian::arithmetic::put::forward")?,
                    "pricing::asian::arithmetic::put::intrinsic",
                )?,
                "pricing::asian::arithmetic::put",
            )?
        }
    };

    Ok(apply_side(price, option))
}

/// Discounted intrinsic on a known average, i.e. the `σ → 0` limit of the
/// Black formula used by both Asian kernels.
fn deterministic_price(
    forward: Decimal,
    strike: Decimal,
    discount: Decimal,
    option: &Options,
) -> Result<Decimal, PricingError> {
    let intrinsic = match option.option_style {
        OptionStyle::Call => {
            d_sub(forward, strike, "pricing::asian::deterministic::call")?.max(Decimal::ZERO)
        }
        OptionStyle::Put => {
            d_sub(strike, forward, "pricing::asian::deterministic::put")?.max(Decimal::ZERO)
        }
    };
    let price = d_mul(intrinsic, discount, "pricing::asian::deterministic::price")?;
    Ok(apply_side(price, option))
}

/// Calculates intrinsic value at expiration.
fn intrinsic_value(option: &Options) -> Result<Decimal, PricingError> {
    let s = option.underlying_price.to_dec();
    let k = option.strike_price.to_dec();
    let value = match option.option_style {
        OptionStyle::Call => d_sub(s, k, "pricing::asian::intrinsic::call")?.max(Decimal::ZERO),
        OptionStyle::Put => d_sub(k, s, "pricing::asian::intrinsic::put")?.max(Decimal::ZERO),
    };
    Ok(apply_side(value, option))
}

/// Applies the side (long/short) multiplier to the price.
fn apply_side(price: Decimal, option: &Options) -> Decimal {
    match option.side {
        crate::model::types::Side::Long => price,
        crate::model::types::Side::Short => -price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::assert_decimal_eq;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_asian_option(style: OptionStyle, averaging_type: AsianAveragingType) -> Options {
        Options::new(
            OptionType::Asian { averaging_type },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,                          // strike
            ExpirationDate::Days(pos_or_panic!(182.5)), // ~0.5 years
            pos_or_panic!(0.25),                        // volatility
            Positive::ONE,                              // quantity
            Positive::HUNDRED,                          // underlying
            dec!(0.05),                                 // risk-free rate
            style,
            Positive::ZERO, // dividend yield
            None,
        )
    }

    #[test]
    fn test_geometric_asian_call() {
        let option = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        let price = asian_black_scholes(&option).unwrap();
        // Price should be positive and less than vanilla BS price
        assert!(
            price > Decimal::ZERO,
            "Geometric Asian call should be positive: {}",
            price
        );
        assert!(
            price < dec!(15.0),
            "Geometric Asian call should be less than vanilla"
        );
    }

    #[test]
    fn test_geometric_asian_put() {
        let option = create_asian_option(OptionStyle::Put, AsianAveragingType::Geometric);
        let price = asian_black_scholes(&option).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Geometric Asian put should be positive: {}",
            price
        );
    }

    #[test]
    fn test_arithmetic_asian_call() {
        let option = create_asian_option(OptionStyle::Call, AsianAveragingType::Arithmetic);
        let price = asian_black_scholes(&option).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Arithmetic Asian call should be positive: {}",
            price
        );
    }

    #[test]
    fn test_arithmetic_asian_put() {
        let option = create_asian_option(OptionStyle::Put, AsianAveragingType::Arithmetic);
        let price = asian_black_scholes(&option).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Arithmetic Asian put should be positive: {}",
            price
        );
    }

    #[test]
    fn test_geometric_less_than_arithmetic() {
        // For standard cases, geometric average <= arithmetic average
        // So geometric Asian call <= arithmetic Asian call
        let geometric = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        let arithmetic = create_asian_option(OptionStyle::Call, AsianAveragingType::Arithmetic);

        let geo_price = asian_black_scholes(&geometric).unwrap();
        let arith_price = asian_black_scholes(&arithmetic).unwrap();

        // Allow some tolerance for approximation errors
        assert!(
            geo_price <= arith_price + dec!(0.5),
            "Geometric {} should be <= Arithmetic {}",
            geo_price,
            arith_price
        );
    }

    #[test]
    fn test_short_asian_option() {
        let mut option = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        let long_price = asian_black_scholes(&option).unwrap();

        option.side = Side::Short;
        let short_price = asian_black_scholes(&option).unwrap();

        assert_decimal_eq!(long_price, -short_price, dec!(1e-10));
    }

    #[test]
    fn test_zero_time_to_expiry() {
        let mut option = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = asian_black_scholes(&option).unwrap();
        assert_decimal_eq!(price, Decimal::ZERO, dec!(1e-10));
    }

    #[test]
    fn test_itm_asian_call() {
        let mut option = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        option.underlying_price = pos_or_panic!(120.0); // ITM
        let price = asian_black_scholes(&option).unwrap();
        assert!(
            price > dec!(10.0),
            "ITM Asian call should have significant value: {}",
            price
        );
    }

    #[test]
    fn test_otm_asian_call() {
        let mut option = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        option.underlying_price = pos_or_panic!(80.0); // OTM
        let price = asian_black_scholes(&option).unwrap();
        assert!(
            price < dec!(5.0),
            "OTM Asian call should have low value: {}",
            price
        );
    }
}
