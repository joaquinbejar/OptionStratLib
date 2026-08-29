/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 13/01/26
******************************************************************************/

//! Lookback option pricing module.
//!
//! Lookback options are path-dependent options with payoffs that depend on
//! the maximum or minimum price of the underlying asset during the option's life.
//!
//! # Variants
//!
//! - **Floating Strike**: Strike is set at expiration based on observed extremum
//!   - Call: pays S_T - S_min (always ITM)
//!   - Put: pays S_max - S_T (always ITM)
//!
//! - **Fixed Strike**: Standard strike with payoff based on extremum
//!   - Call: pays max(S_max - K, 0)
//!   - Put: pays max(K - S_min, 0)
//!
//! # Formulas
//!
//! This implementation uses the Goldman-Sosin-Gatto (1979) and Conze-Viswanathan
//! (1991) closed-form solutions for continuous monitoring.

use crate::Options;
use crate::error::PricingError;
use crate::greeks::{big_n, d1, d2};
use crate::model::decimal::{d_add, d_div, d_exp, d_mul, d_sqrt, d_sub};
use crate::model::types::{LookbackType, OptionStyle, OptionType};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Prices a Lookback option using appropriate closed-form formula.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Lookback`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// - [`PricingError::MethodError`] when `option` is not an
///   [`OptionType::Lookback`] variant, when the lookback sub-type is an
///   unsupported `#[non_exhaustive]` variant, when the expiration cannot be
///   converted to a year fraction, or when the `d1` / `d2` kernels reject the
///   inputs.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range, or when `σ√T` collapses to zero: the
///   discount factors, the Goldman-Sosin-Gatto `a1` / `a2`, the reflection
///   weight `σ²/(2b)`, the Conze-Viswanathan `λ`, or the final legs.
pub fn lookback_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Lookback { lookback_type } => match lookback_type {
            LookbackType::FloatingStrike => floating_strike_lookback(option),
            LookbackType::FixedStrike => fixed_strike_lookback(option),
            // `LookbackType` is `#[non_exhaustive]`.
            _ => Err(PricingError::other(
                "lookback_black_scholes: unsupported LookbackType",
            )),
        },
        _ => Err(PricingError::other(
            "lookback_black_scholes requires OptionType::Lookback",
        )),
    }
}

/// Prices a floating strike lookback option.
///
/// **Floating Strike Call**: S_T - S_min (buyer gets to buy at lowest price)
/// **Floating Strike Put**: S_max - S_T (buyer gets to sell at highest price)
///
/// For new contracts, S_min = S_max = S (current spot price).
///
/// Uses Goldman-Sosin-Gatto (1979) formula.
fn floating_strike_lookback(option: &Options) -> Result<Decimal, PricingError> {
    let s = option.underlying_price;
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let t = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    if t == Positive::ZERO {
        // At expiration, value is intrinsic (but for new contract, S_min = S_max = S, so 0)
        return Ok(Decimal::ZERO);
    }

    let t_dec = t.to_dec();
    let b = d_sub(r, q, "pricing::lookback::floating::carry")?; // cost of carry
    let discount = d_exp(
        d_mul(-r, t_dec, "pricing::lookback::floating::neg_rt")?,
        "pricing::lookback::floating::discount",
    )?;
    let dividend_discount = d_exp(
        d_mul(-q, t_dec, "pricing::lookback::floating::neg_qt")?,
        "pricing::lookback::floating::dividend_discount",
    )?;

    if sigma == Positive::ZERO {
        // Zero volatility: no path variation, lookback equals vanilla
        let forward = d_mul(
            s.to_dec(),
            d_exp(
                d_mul(b, t_dec, "pricing::lookback::floating::zero_vol::carry_t")?,
                "pricing::lookback::floating::zero_vol::growth",
            )?,
            "pricing::lookback::floating::zero_vol::forward",
        )?;
        let intrinsic = match option.option_style {
            OptionStyle::Call => d_sub(
                forward,
                s.to_dec(),
                "pricing::lookback::floating::zero_vol::call",
            )?
            .max(Decimal::ZERO),
            OptionStyle::Put => d_sub(
                s.to_dec(),
                forward,
                "pricing::lookback::floating::zero_vol::put",
            )?
            .max(Decimal::ZERO),
        };
        let value = d_mul(
            intrinsic,
            discount,
            "pricing::lookback::floating::zero_vol::discounted",
        )?;
        return Ok(apply_side(value, option));
    }

    let sigma_dec = sigma.to_dec();
    let sigma_sq = d_mul(
        sigma_dec,
        sigma_dec,
        "pricing::lookback::floating::sigma_sq",
    )?;
    let sqrt_t = d_sqrt(t_dec, "pricing::lookback::floating::sqrt_t")?;
    let sigma_sqrt_t = d_mul(
        sigma_dec,
        sqrt_t,
        "pricing::lookback::floating::sigma_sqrt_t",
    )?;
    let s_dec = s.to_dec();
    // `a1 = (b + σ²/2) T / (σ√T)`, shared by both styles outside the `b ≈ 0`
    // special case.
    let a1_general = || -> Result<Decimal, PricingError> {
        Ok(d_div(
            d_mul(
                d_add(
                    b,
                    d_div(sigma_sq, dec!(2), "pricing::lookback::floating::half_var")?,
                    "pricing::lookback::floating::drift_rate",
                )?,
                t_dec,
                "pricing::lookback::floating::drift",
            )?,
            sigma_sqrt_t,
            "pricing::lookback::floating::a1",
        )?)
    };
    // `σ² / (2b)`, the Goldman-Sosin-Gatto reflection weight.
    let reflection_weight = || -> Result<Decimal, PricingError> {
        Ok(d_div(
            sigma_sq,
            d_mul(dec!(2), b, "pricing::lookback::floating::two_b")?,
            "pricing::lookback::floating::reflection_weight",
        )?)
    };

    // For a new floating strike lookback (S_min = S_max = S):
    // Use Goldman-Sosin-Gatto formulas

    let price = match option.option_style {
        OptionStyle::Call => {
            // Floating strike call: C = S*e^(-qT)*N(a1) - S*e^(-rT)*N(a1 - sigma*sqrt(T))
            //                        + S*e^(-rT)*(sigma^2/(2b))*(N(a1 - sigma*sqrt(T)) - e^(b*T)*N(-a1))
            // where a1 = (b + sigma^2/2)*T / (sigma*sqrt(T))

            if b.abs() < dec!(1e-10) {
                // Special case when b ≈ 0 (ATM forward)
                let a1 = d_div(
                    sigma_sqrt_t,
                    dec!(2),
                    "pricing::lookback::floating::call::flat::a1",
                )?;
                let n_a1 = big_n(a1).unwrap_or(Decimal::ZERO);
                let n_neg_a1 = big_n(-a1).unwrap_or(Decimal::ZERO);

                // Simplified formula for b = 0
                let centred = d_sub(
                    d_mul(
                        dec!(2),
                        n_a1,
                        "pricing::lookback::floating::call::flat::two_n",
                    )?,
                    dec!(1),
                    "pricing::lookback::floating::call::flat::centred",
                )?;
                let bracket = d_add(
                    centred,
                    d_mul(
                        d_div(
                            dec!(2),
                            dec!(2.506628274631),
                            "pricing::lookback::floating::call::flat::coefficient",
                        )?,
                        d_exp(
                            d_mul(
                                a1,
                                n_neg_a1,
                                "pricing::lookback::floating::call::flat::exponent",
                            )?,
                            "pricing::lookback::floating::call::flat::exp",
                        )?
                        .min(dec!(10)),
                        "pricing::lookback::floating::call::flat::tail",
                    )?,
                    "pricing::lookback::floating::call::flat::bracket",
                )?
                .min(s_dec);
                d_add(
                    d_mul(
                        s_dec,
                        centred,
                        "pricing::lookback::floating::call::flat::level",
                    )?,
                    d_mul(
                        d_mul(
                            s_dec,
                            sigma_sqrt_t,
                            "pricing::lookback::floating::call::flat::scale",
                        )?,
                        bracket,
                        "pricing::lookback::floating::call::flat::spread",
                    )?,
                    "pricing::lookback::floating::call::flat::price",
                )?
            } else {
                let a1 = a1_general()?;
                let a2 = d_sub(a1, sigma_sqrt_t, "pricing::lookback::floating::call::a2")?;

                let n_a1 = big_n(a1).unwrap_or(Decimal::ZERO);
                let n_a2 = big_n(a2).unwrap_or(Decimal::ZERO);
                let n_neg_a1 = big_n(-a1).unwrap_or(Decimal::ZERO);

                let term1 = d_mul(
                    d_mul(
                        s_dec,
                        dividend_discount,
                        "pricing::lookback::floating::call::s_pv",
                    )?,
                    n_a1,
                    "pricing::lookback::floating::call::term1",
                )?;
                let s_discounted = d_mul(
                    s_dec,
                    discount,
                    "pricing::lookback::floating::call::s_discounted",
                )?;
                let term2 = d_mul(
                    s_discounted,
                    n_a2,
                    "pricing::lookback::floating::call::term2",
                )?;
                let term3 = d_mul(
                    d_mul(
                        s_discounted,
                        reflection_weight()?,
                        "pricing::lookback::floating::call::reflection",
                    )?,
                    d_sub(
                        n_a2,
                        d_mul(
                            d_exp(
                                d_mul(b, t_dec, "pricing::lookback::floating::call::bt")?,
                                "pricing::lookback::floating::call::exp_bt",
                            )?,
                            n_neg_a1,
                            "pricing::lookback::floating::call::reflected_cdf",
                        )?,
                        "pricing::lookback::floating::call::bracket",
                    )?,
                    "pricing::lookback::floating::call::term3",
                )?;

                let diff = d_sub(term1, term2, "pricing::lookback::floating::call::diff")?;
                d_add(diff, term3, "pricing::lookback::floating::call::price")?
            }
        }
        OptionStyle::Put => {
            // Floating strike put: P = S*e^(-rT)*N(-a2) - S*e^(-qT)*N(-a1)
            //                        + S*e^(-rT)*(sigma^2/(2b))*(e^(b*T)*N(a1) - N(a2))

            if b.abs() < dec!(1e-10) {
                let a1 = d_div(
                    sigma_sqrt_t,
                    dec!(2),
                    "pricing::lookback::floating::put::flat::a1",
                )?;
                let n_neg_a1 = big_n(-a1).unwrap_or(Decimal::ZERO);

                // Simplified for b = 0
                d_add(
                    d_mul(
                        s_dec,
                        d_sub(
                            dec!(1),
                            d_mul(
                                dec!(2),
                                n_neg_a1,
                                "pricing::lookback::floating::put::flat::two_n",
                            )?,
                            "pricing::lookback::floating::put::flat::centred",
                        )?,
                        "pricing::lookback::floating::put::flat::level",
                    )?,
                    d_mul(
                        d_mul(
                            s_dec,
                            sigma_sqrt_t,
                            "pricing::lookback::floating::put::flat::scale",
                        )?,
                        dec!(0.5),
                        "pricing::lookback::floating::put::flat::spread",
                    )?,
                    "pricing::lookback::floating::put::flat::price",
                )?
            } else {
                let a1 = a1_general()?;
                let a2 = d_sub(a1, sigma_sqrt_t, "pricing::lookback::floating::put::a2")?;

                let n_neg_a1 = big_n(-a1).unwrap_or(Decimal::ZERO);
                let n_neg_a2 = big_n(-a2).unwrap_or(Decimal::ZERO);
                let n_a1 = big_n(a1).unwrap_or(Decimal::ZERO);
                let n_a2 = big_n(a2).unwrap_or(Decimal::ZERO);

                let s_discounted = d_mul(
                    s_dec,
                    discount,
                    "pricing::lookback::floating::put::s_discounted",
                )?;
                let term1 = d_mul(
                    s_discounted,
                    n_neg_a2,
                    "pricing::lookback::floating::put::term1",
                )?;
                let term2 = d_mul(
                    d_mul(
                        s_dec,
                        dividend_discount,
                        "pricing::lookback::floating::put::s_pv",
                    )?,
                    n_neg_a1,
                    "pricing::lookback::floating::put::term2",
                )?;
                let term3 = d_mul(
                    d_mul(
                        s_discounted,
                        reflection_weight()?,
                        "pricing::lookback::floating::put::reflection",
                    )?,
                    d_sub(
                        d_mul(
                            d_exp(
                                d_mul(b, t_dec, "pricing::lookback::floating::put::bt")?,
                                "pricing::lookback::floating::put::exp_bt",
                            )?,
                            n_a1,
                            "pricing::lookback::floating::put::reflected_cdf",
                        )?,
                        n_a2,
                        "pricing::lookback::floating::put::bracket",
                    )?,
                    "pricing::lookback::floating::put::term3",
                )?;

                let diff = d_sub(term1, term2, "pricing::lookback::floating::put::diff")?;
                d_add(diff, term3, "pricing::lookback::floating::put::price")?
            }
        }
    };

    Ok(apply_side(price.max(Decimal::ZERO), option))
}

/// Prices a fixed strike lookback option.
///
/// **Fixed Strike Call**: max(S_max - K, 0)
/// **Fixed Strike Put**: max(K - S_min, 0)
///
/// For new contracts, S_min = S_max = S.
/// Uses Conze-Viswanathan (1991) approach.
fn fixed_strike_lookback(option: &Options) -> Result<Decimal, PricingError> {
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
        // At expiration, for new contract S_max = S_min = S
        let intrinsic = match option.option_style {
            OptionStyle::Call => d_sub(
                s.to_dec(),
                k.to_dec(),
                "pricing::lookback::fixed::intrinsic::call",
            )?
            .max(Decimal::ZERO),
            OptionStyle::Put => d_sub(
                k.to_dec(),
                s.to_dec(),
                "pricing::lookback::fixed::intrinsic::put",
            )?
            .max(Decimal::ZERO),
        };
        return Ok(apply_side(intrinsic, option));
    }

    let t_dec = t.to_dec();
    let b = d_sub(r, q, "pricing::lookback::fixed::carry")?;
    let discount = d_exp(
        d_mul(-r, t_dec, "pricing::lookback::fixed::neg_rt")?,
        "pricing::lookback::fixed::discount",
    )?;
    let dividend_discount = d_exp(
        d_mul(-q, t_dec, "pricing::lookback::fixed::neg_qt")?,
        "pricing::lookback::fixed::dividend_discount",
    )?;

    if sigma == Positive::ZERO {
        let forward = d_mul(
            s.to_dec(),
            d_exp(
                d_mul(b, t_dec, "pricing::lookback::fixed::zero_vol::carry_t")?,
                "pricing::lookback::fixed::zero_vol::growth",
            )?,
            "pricing::lookback::fixed::zero_vol::forward",
        )?;
        let payoff = match option.option_style {
            OptionStyle::Call => d_sub(
                forward,
                k.to_dec(),
                "pricing::lookback::fixed::zero_vol::call",
            )?
            .max(Decimal::ZERO),
            OptionStyle::Put => d_sub(
                k.to_dec(),
                forward,
                "pricing::lookback::fixed::zero_vol::put",
            )?
            .max(Decimal::ZERO),
        };
        let intrinsic = d_mul(
            payoff,
            discount,
            "pricing::lookback::fixed::zero_vol::discounted",
        )?;
        return Ok(apply_side(intrinsic, option));
    }

    let sigma_dec = sigma.to_dec();
    let sigma_sq = d_mul(sigma_dec, sigma_dec, "pricing::lookback::fixed::sigma_sq")?;
    let sqrt_t = d_sqrt(t_dec, "pricing::lookback::fixed::sqrt_t")?;
    let sigma_sqrt_t = d_mul(sigma_dec, sqrt_t, "pricing::lookback::fixed::sigma_sqrt_t")?;
    let s_dec = s.to_dec();
    // Lookback premium shared by both styles: the `λ` cut-off and the
    // `S σ√T (N(λ) − ½) / 2` scaling of the running extremum.
    let lookback_premium = |lambda: Decimal| -> Result<Decimal, PricingError> {
        let n_lambda = big_n(lambda).unwrap_or(dec!(0.5));
        Ok(d_mul(
            d_mul(
                d_mul(
                    s_dec,
                    sigma_sqrt_t,
                    "pricing::lookback::fixed::premium_scale",
                )?,
                d_sub(n_lambda, dec!(0.5), "pricing::lookback::fixed::premium_cdf")?,
                "pricing::lookback::fixed::premium_weighted",
            )?,
            dec!(0.5),
            "pricing::lookback::fixed::premium",
        )?)
    };
    let lambda = if b.abs() < dec!(1e-10) {
        d_add(
            dec!(1),
            d_div(
                d_mul(sigma_sq, t_dec, "pricing::lookback::fixed::flat_variance")?,
                dec!(2),
                "pricing::lookback::fixed::flat_half_variance",
            )?,
            "pricing::lookback::fixed::lambda_flat",
        )?
    } else {
        d_div(
            d_mul(
                d_add(
                    b,
                    d_div(sigma_sq, dec!(2), "pricing::lookback::fixed::half_variance")?,
                    "pricing::lookback::fixed::drift_rate",
                )?,
                t_dec,
                "pricing::lookback::fixed::drift",
            )?,
            sigma_sqrt_t,
            "pricing::lookback::fixed::lambda",
        )?
    };

    // For fixed strike lookback, we use a combination of standard BS
    // plus lookback premium

    // First, get standard BS price
    let d1_val = d1(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let price = match option.option_style {
        OptionStyle::Call => {
            // Fixed strike lookback call: pays max(S_max - K, 0)
            // For a new contract: similar to standard call + lookback premium

            let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
            let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);

            // Standard BS call
            let s_leg = d_mul(
                d_mul(
                    s_dec,
                    dividend_discount,
                    "pricing::lookback::fixed::call::s_discounted",
                )?,
                n_d1,
                "pricing::lookback::fixed::call::s_leg",
            )?;
            let k_leg = d_mul(
                d_mul(
                    k.to_dec(),
                    discount,
                    "pricing::lookback::fixed::call::k_discounted",
                )?,
                n_d2,
                "pricing::lookback::fixed::call::k_leg",
            )?;
            let bs_call = d_sub(s_leg, k_leg, "pricing::lookback::fixed::call::bs")?;

            // Lookback premium (value of being able to exercise at maximum)
            // For new contract from S, use simplified formula
            d_add(
                bs_call,
                lookback_premium(lambda)?,
                "pricing::lookback::fixed::call::price",
            )?
            .max(Decimal::ZERO)
        }
        OptionStyle::Put => {
            // Fixed strike lookback put: pays max(K - S_min, 0)

            let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
            let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);

            // Standard BS put. Mirror of the call branch: build the
            // discounted strike / discounted forward with `d_mul`,
            // then fold in the CDF weight with a second `d_mul`.
            let k_discounted = d_mul(
                k.to_dec(),
                discount,
                "pricing::lookback::fixed::put::k_discounted",
            )?;
            let k_leg = d_mul(
                k_discounted,
                n_neg_d2,
                "pricing::lookback::fixed::put::k_leg",
            )?;
            let s_discounted = d_mul(
                s_dec,
                dividend_discount,
                "pricing::lookback::fixed::put::s_discounted",
            )?;
            let s_leg = d_mul(
                s_discounted,
                n_neg_d1,
                "pricing::lookback::fixed::put::s_leg",
            )?;
            let bs_put = d_sub(k_leg, s_leg, "pricing::lookback::fixed::put::bs")?;

            // Lookback premium (value of being able to exercise at minimum)
            d_add(
                bs_put,
                lookback_premium(lambda)?,
                "pricing::lookback::fixed::put::price",
            )?
            .max(Decimal::ZERO)
        }
    };

    Ok(apply_side(price, option))
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

    fn create_lookback_option(style: OptionStyle, lookback_type: LookbackType) -> Options {
        Options::new(
            OptionType::Lookback { lookback_type },
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
    fn test_floating_strike_call() {
        let option = create_lookback_option(OptionStyle::Call, LookbackType::FloatingStrike);
        let price = lookback_black_scholes(&option).unwrap();
        // Floating strike lookback should have positive value
        assert!(
            price > Decimal::ZERO,
            "Floating strike call should be positive: {}",
            price
        );
    }

    #[test]
    fn test_floating_strike_put() {
        let option = create_lookback_option(OptionStyle::Put, LookbackType::FloatingStrike);
        let price = lookback_black_scholes(&option).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Floating strike put should be positive: {}",
            price
        );
    }

    #[test]
    fn test_fixed_strike_call() {
        let option = create_lookback_option(OptionStyle::Call, LookbackType::FixedStrike);
        let price = lookback_black_scholes(&option).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Fixed strike call should be positive: {}",
            price
        );
    }

    #[test]
    fn test_fixed_strike_put() {
        let option = create_lookback_option(OptionStyle::Put, LookbackType::FixedStrike);
        let price = lookback_black_scholes(&option).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Fixed strike put should be positive: {}",
            price
        );
    }

    #[test]
    fn test_lookback_more_expensive_than_vanilla() {
        // Lookback options should be more expensive than vanilla options
        let lookback = create_lookback_option(OptionStyle::Call, LookbackType::FixedStrike);
        let lookback_price = lookback_black_scholes(&lookback).unwrap();

        // Compare with approximate vanilla Black-Scholes price for ATM call
        // ATM call with 0.5Y, 25% vol, 5% rate ≈ 7-8
        assert!(
            lookback_price > dec!(7.0),
            "Lookback should be at least as expensive as vanilla: {}",
            lookback_price
        );
    }

    #[test]
    fn test_short_lookback_option() {
        let mut option = create_lookback_option(OptionStyle::Call, LookbackType::FloatingStrike);
        let long_price = lookback_black_scholes(&option).unwrap();

        option.side = Side::Short;
        let short_price = lookback_black_scholes(&option).unwrap();

        assert_decimal_eq!(long_price, -short_price, dec!(1e-10));
    }

    #[test]
    fn test_zero_time_to_expiry() {
        let mut option = create_lookback_option(OptionStyle::Call, LookbackType::FloatingStrike);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = lookback_black_scholes(&option).unwrap();
        // For new contract at expiry, S_min = S = S_max, so floating strike = 0
        assert_decimal_eq!(price, Decimal::ZERO, dec!(1e-10));
    }

    #[test]
    fn test_fixed_strike_itm_at_expiry() {
        let mut option = create_lookback_option(OptionStyle::Call, LookbackType::FixedStrike);
        option.underlying_price = pos_or_panic!(110.0); // ITM
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = lookback_black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(10.0), dec!(1e-10));
    }

    #[test]
    fn test_higher_vol_means_higher_lookback_value() {
        let low_vol = create_lookback_option(OptionStyle::Call, LookbackType::FloatingStrike);
        let low_vol_price = lookback_black_scholes(&low_vol).unwrap();

        let mut high_vol = low_vol.clone();
        high_vol.implied_volatility = pos_or_panic!(0.4);
        let high_vol_price = lookback_black_scholes(&high_vol).unwrap();

        assert!(
            high_vol_price > low_vol_price,
            "Higher vol should mean higher lookback value: {} vs {}",
            high_vol_price,
            low_vol_price
        );
    }

    #[test]
    fn test_floating_strike_symmetry() {
        // For ATM options, floating call and put should have similar values
        let call = create_lookback_option(OptionStyle::Call, LookbackType::FloatingStrike);
        let put = create_lookback_option(OptionStyle::Put, LookbackType::FloatingStrike);

        let call_price = lookback_black_scholes(&call).unwrap();
        let put_price = lookback_black_scholes(&put).unwrap();

        // They should be in the same ballpark (within 50% of each other)
        let ratio = if call_price > put_price {
            call_price / put_price
        } else {
            put_price / call_price
        };
        assert!(
            ratio < dec!(2.0),
            "Call and put should be similar for ATM: call={}, put={}",
            call_price,
            put_price
        );
    }
}
