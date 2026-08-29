/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

use crate::Options;
use crate::error::PricingError;
use crate::greeks::big_n;
use crate::model::decimal::{d_add, d_div, d_exp, d_ln, d_mul, d_powd, d_sqrt, d_sub};
use crate::model::types::{BarrierType, OptionStyle, OptionType};
use positive::Positive;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;

/// Prices a barrier option using the Black-Scholes analytical extension.
/// Supports Down-And-In, Up-And-In, Down-And-Out, and Up-And-Out variants.
///
/// # Errors
///
/// - [`PricingError::UnsupportedOptionType`] when `option` is not a
///   [`OptionType::Barrier`] variant.
/// - [`PricingError::MethodError`] when the volatility is zero, when the
///   underlying, the strike or the barrier level is zero (the closed form
///   divides by all three), or when the `λ` discriminant is negative.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range, or when a log-moneyness ratio underflows
///   to zero: `μ`, the `x` / `y` / `z` arguments, the reflection powers,
///   the discount factors, or the final price composition.
/// - `PricingError::ExpirationDate` when the expiration cannot be converted.
pub fn barrier_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    let (barrier_type, barrier_level, rebate) = match &option.option_type {
        OptionType::Barrier {
            barrier_type,
            barrier_level,
            rebate,
        } => {
            // `barrier_level` and `rebate` are `Positive`, i.e. `Decimal`-backed
            // and always finite, so no non-finite check is needed.
            let bl = barrier_level.to_dec();
            let rb = rebate.unwrap_or(Positive::ZERO).to_dec();
            (barrier_type, bl, rb)
        }
        _ => {
            return Err(PricingError::unsupported_option_type(
                "Non-Barrier",
                "Barrier BS",
            ));
        }
    };

    let s = option.underlying_price.to_dec();
    let k = option.strike_price.to_dec();
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility.to_dec();
    let t = option.time_to_expiration()?.to_dec();

    if t == Decimal::ZERO {
        return option
            .payoff()
            .map_err(|e| PricingError::other(&e.to_string()));
    }

    if sigma == Decimal::ZERO {
        return Err(PricingError::other(
            "Volatility cannot be zero for barrier options pricing",
        ));
    }

    // The closed form divides by `S`, by `K` and by the barrier, and takes
    // logarithms of their ratios: a zero on any of the three has no financial
    // meaning here and is rejected before any arithmetic runs.
    if s == Decimal::ZERO {
        return Err(PricingError::other(
            "Underlying price cannot be zero for barrier options pricing",
        ));
    }
    if k == Decimal::ZERO {
        return Err(PricingError::other(
            "Strike price cannot be zero for barrier options pricing",
        ));
    }
    if barrier_level == Decimal::ZERO {
        return Err(PricingError::other(
            "Barrier level cannot be zero for barrier options pricing",
        ));
    }

    let b = d_sub(r, q, "pricing::barrier::carry")?; // Cost of carry
    let sigma2 = d_mul(sigma, sigma, "pricing::barrier::sigma2")?;
    let mu = d_div(
        d_sub(
            b,
            d_div(sigma2, dec!(2.0), "pricing::barrier::half_variance")?,
            "pricing::barrier::mu_numerator",
        )?,
        sigma2,
        "pricing::barrier::mu",
    )?;
    let lambda_discriminant = d_add(
        d_mul(mu, mu, "pricing::barrier::mu_squared")?,
        d_div(
            d_mul(dec!(2.0), r, "pricing::barrier::two_r")?,
            sigma2,
            "pricing::barrier::rate_over_variance",
        )?,
        "pricing::barrier::lambda_discriminant",
    )?;
    let lambda = lambda_discriminant.sqrt().ok_or_else(|| {
        PricingError::method_error("barrier_black_scholes", "non-finite lambda discriminant")
    })?;

    let sqrt_t = d_sqrt(t, "pricing::barrier::sqrt_t")?;
    let sigma_sqrt_t = d_mul(sigma, sqrt_t, "pricing::barrier::sigma_sqrt_t")?;
    let mu_plus_one_term = d_mul(
        d_add(mu, dec!(1.0), "pricing::barrier::mu_plus_one")?,
        sigma_sqrt_t,
        "pricing::barrier::drift_term",
    )?;
    // `H / S` drives `y2`, `z` and every reflection power below, so it is
    // computed once.
    let h_over_s = d_div(barrier_level, s, "pricing::barrier::h_over_s")?;
    let log_h_over_s = d_ln(h_over_s, "pricing::barrier::log_h_over_s")?;

    // Components used across different barrier types
    let x1 = d_add(
        d_div(
            d_ln(
                d_div(s, k, "pricing::barrier::moneyness")?,
                "pricing::barrier::log_moneyness",
            )?,
            sigma_sqrt_t,
            "pricing::barrier::x1_ratio",
        )?,
        mu_plus_one_term,
        "pricing::barrier::x1",
    )?;
    let x2 = d_add(
        d_div(
            d_ln(
                d_div(s, barrier_level, "pricing::barrier::s_over_h")?,
                "pricing::barrier::log_s_over_h",
            )?,
            sigma_sqrt_t,
            "pricing::barrier::x2_ratio",
        )?,
        mu_plus_one_term,
        "pricing::barrier::x2",
    )?;
    let y1 = d_add(
        d_div(
            d_ln(
                d_div(
                    d_mul(barrier_level, barrier_level, "pricing::barrier::h_squared")?,
                    d_mul(s, k, "pricing::barrier::s_times_k")?,
                    "pricing::barrier::h2_over_sk",
                )?,
                "pricing::barrier::log_h2_over_sk",
            )?,
            sigma_sqrt_t,
            "pricing::barrier::y1_ratio",
        )?,
        mu_plus_one_term,
        "pricing::barrier::y1",
    )?;
    let y2 = d_add(
        d_div(log_h_over_s, sigma_sqrt_t, "pricing::barrier::y2_ratio")?,
        mu_plus_one_term,
        "pricing::barrier::y2",
    )?;
    let z = d_add(
        d_div(log_h_over_s, sigma_sqrt_t, "pricing::barrier::z_ratio")?,
        d_mul(lambda, sigma_sqrt_t, "pricing::barrier::z_drift")?,
        "pricing::barrier::z",
    )?;

    // Shared discount factors: every closure below uses both.
    let discount_q = d_exp(
        d_mul(-q, t, "pricing::barrier::neg_qt")?,
        "pricing::barrier::discount_q",
    )?;
    let discount_r = d_exp(
        d_mul(-r, t, "pricing::barrier::neg_rt")?,
        "pricing::barrier::discount_r",
    )?;

    let _phi = match option.option_style {
        OptionStyle::Call => dec!(1.0),
        OptionStyle::Put => dec!(-1.0),
    };

    let _eta = match barrier_type {
        BarrierType::DownAndIn | BarrierType::DownAndOut => dec!(1.0),
        BarrierType::UpAndIn | BarrierType::UpAndOut => dec!(-1.0),
        // `BarrierType` is `#[non_exhaustive]`; unreachable here because the
        // match on `(style, barrier_type)` below rejects unknown variants.
        _ => dec!(1.0),
    };

    // `(H / S)^e`: the reflection factor shared by the `C`, `D`, `E` and `F`
    // terms of Reiner-Rubinstein.
    let reflect_pow = |exponent: Decimal, tag: &'static str| -> Result<Decimal, PricingError> {
        Ok(d_powd(h_over_s, exponent, tag)?)
    };

    // `φ S e^(-qT) N(·) − φ K e^(-rT) N(·)`, the vanilla leg of the decomposition.
    let vanilla_leg = |phi_val: Decimal, x_val: Decimal| -> Result<Decimal, PricingError> {
        let n1 = big_n(d_mul(phi_val, x_val, "pricing::barrier::vanilla::arg1")?)?;
        let n2 = big_n(d_mul(
            phi_val,
            d_sub(x_val, sigma_sqrt_t, "pricing::barrier::vanilla::shift")?,
            "pricing::barrier::vanilla::arg2",
        )?)?;
        let spot = d_mul(
            d_mul(
                d_mul(phi_val, s, "pricing::barrier::vanilla::phi_s")?,
                discount_q,
                "pricing::barrier::vanilla::spot_pv",
            )?,
            n1,
            "pricing::barrier::vanilla::spot_leg",
        )?;
        let strike = d_mul(
            d_mul(
                d_mul(phi_val, k, "pricing::barrier::vanilla::phi_k")?,
                discount_r,
                "pricing::barrier::vanilla::strike_pv",
            )?,
            n2,
            "pricing::barrier::vanilla::strike_leg",
        )?;
        Ok(d_sub(spot, strike, "pricing::barrier::vanilla")?)
    };

    let f_a = &vanilla_leg;
    let f_b = &vanilla_leg;

    // `φ S e^(-qT) (H/S)^(2(μ+1)) N(·) − φ K e^(-rT) (H/S)^(2μ) N(·)`.
    let reflected_leg =
        |phi_val: Decimal, eta_val: Decimal, y_val: Decimal| -> Result<Decimal, PricingError> {
            let n1 = big_n(d_mul(eta_val, y_val, "pricing::barrier::reflected::arg1")?)?;
            let n2 = big_n(d_mul(
                eta_val,
                d_sub(y_val, sigma_sqrt_t, "pricing::barrier::reflected::shift")?,
                "pricing::barrier::reflected::arg2",
            )?)?;
            let h_s_ratio = reflect_pow(
                d_mul(
                    dec!(2.0),
                    d_add(mu, dec!(1.0), "pricing::barrier::reflected::mu_plus_one")?,
                    "pricing::barrier::reflected::exponent_spot",
                )?,
                "pricing::barrier::reflected::pow_spot",
            )?;
            let h_s_ratio_mu = reflect_pow(
                d_mul(
                    dec!(2.0),
                    mu,
                    "pricing::barrier::reflected::exponent_strike",
                )?,
                "pricing::barrier::reflected::pow_strike",
            )?;
            let spot = d_mul(
                d_mul(
                    d_mul(
                        d_mul(phi_val, s, "pricing::barrier::reflected::phi_s")?,
                        discount_q,
                        "pricing::barrier::reflected::spot_pv",
                    )?,
                    h_s_ratio,
                    "pricing::barrier::reflected::spot_reflected",
                )?,
                n1,
                "pricing::barrier::reflected::spot_leg",
            )?;
            let strike = d_mul(
                d_mul(
                    d_mul(
                        d_mul(phi_val, k, "pricing::barrier::reflected::phi_k")?,
                        discount_r,
                        "pricing::barrier::reflected::strike_pv",
                    )?,
                    h_s_ratio_mu,
                    "pricing::barrier::reflected::strike_reflected",
                )?,
                n2,
                "pricing::barrier::reflected::strike_leg",
            )?;
            Ok(d_sub(spot, strike, "pricing::barrier::reflected")?)
        };

    let f_c = &reflected_leg;
    let f_d = &reflected_leg;

    // Rebate paid at expiry when the barrier is never hit.
    let f_e = |eta_val: Decimal| -> Result<Decimal, PricingError> {
        if rebate == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        let n1 = big_n(d_mul(
            eta_val,
            d_sub(x2, sigma_sqrt_t, "pricing::barrier::rebate_expiry::shift1")?,
            "pricing::barrier::rebate_expiry::arg1",
        )?)?;
        let h_s_ratio_mu = reflect_pow(
            d_mul(dec!(2.0), mu, "pricing::barrier::rebate_expiry::exponent")?,
            "pricing::barrier::rebate_expiry::pow",
        )?;
        let n2 = big_n(d_mul(
            eta_val,
            d_sub(y2, sigma_sqrt_t, "pricing::barrier::rebate_expiry::shift2")?,
            "pricing::barrier::rebate_expiry::arg2",
        )?)?;
        Ok(d_mul(
            d_mul(
                rebate,
                discount_r,
                "pricing::barrier::rebate_expiry::rebate_pv",
            )?,
            d_sub(
                n1,
                d_mul(
                    h_s_ratio_mu,
                    n2,
                    "pricing::barrier::rebate_expiry::reflected",
                )?,
                "pricing::barrier::rebate_expiry::bracket",
            )?,
            "pricing::barrier::rebate_expiry",
        )?)
    };

    // Rebate paid on the hit itself.
    let f_f = |eta_val: Decimal| -> Result<Decimal, PricingError> {
        if rebate == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        let h_s_ratio_mu_lambda = reflect_pow(
            d_add(mu, lambda, "pricing::barrier::rebate_hit::exponent_up")?,
            "pricing::barrier::rebate_hit::pow_up",
        )?;
        let h_s_ratio_mu_lambda_neg = reflect_pow(
            d_sub(mu, lambda, "pricing::barrier::rebate_hit::exponent_down")?,
            "pricing::barrier::rebate_hit::pow_down",
        )?;
        let n1 = big_n(d_mul(eta_val, z, "pricing::barrier::rebate_hit::arg1")?)?;
        let n2 = big_n(d_mul(
            eta_val,
            d_sub(
                z,
                d_mul(
                    d_mul(
                        dec!(2.0),
                        lambda,
                        "pricing::barrier::rebate_hit::two_lambda",
                    )?,
                    sigma_sqrt_t,
                    "pricing::barrier::rebate_hit::shift",
                )?,
                "pricing::barrier::rebate_hit::z_shifted",
            )?,
            "pricing::barrier::rebate_hit::arg2",
        )?)?;
        Ok(d_mul(
            rebate,
            d_add(
                d_mul(
                    h_s_ratio_mu_lambda,
                    n1,
                    "pricing::barrier::rebate_hit::up_leg",
                )?,
                d_mul(
                    h_s_ratio_mu_lambda_neg,
                    n2,
                    "pricing::barrier::rebate_hit::down_leg",
                )?,
                "pricing::barrier::rebate_hit::bracket",
            )?,
            "pricing::barrier::rebate_hit",
        )?)
    };

    // Each closure return is an addend of the final barrier price; the
    // composition that fuses them into the returned value goes through
    // `d_add` / `d_sub` so an overflow of the user-visible price surfaces a
    // `DecimalError::Overflow` instead of aborting.
    const OP: &str = "pricing::barrier::price";
    match (option.option_style, barrier_type) {
        // Down-and-out call
        (OptionStyle::Call, BarrierType::DownAndOut) => {
            if k >= barrier_level {
                let lhs = d_sub(f_a(dec!(1.0), x1)?, f_c(dec!(1.0), dec!(1.0), y1)?, OP)?;
                Ok(d_add(lhs, f_e(dec!(1.0))?, OP)?)
            } else {
                let lhs = d_sub(f_b(dec!(1.0), x2)?, f_d(dec!(1.0), dec!(1.0), y2)?, OP)?;
                Ok(d_add(lhs, f_e(dec!(1.0))?, OP)?)
            }
        }
        // Down-and-in call
        (OptionStyle::Call, BarrierType::DownAndIn) => {
            if k >= barrier_level {
                Ok(d_add(f_c(dec!(1.0), dec!(1.0), y1)?, f_f(dec!(1.0))?, OP)?)
            } else {
                let s1 = d_sub(f_a(dec!(1.0), x1)?, f_b(dec!(1.0), x2)?, OP)?;
                let s2 = d_add(s1, f_d(dec!(1.0), dec!(1.0), y2)?, OP)?;
                Ok(d_add(s2, f_f(dec!(1.0))?, OP)?)
            }
        }
        // Up-and-out call
        (OptionStyle::Call, BarrierType::UpAndOut) => {
            if k >= barrier_level {
                Ok(f_f(dec!(-1.0))?)
            } else {
                let s1 = d_sub(f_a(dec!(1.0), x1)?, f_b(dec!(1.0), x2)?, OP)?;
                let s2 = d_add(s1, f_d(dec!(1.0), dec!(-1.0), y2)?, OP)?;
                Ok(d_add(s2, f_f(dec!(-1.0))?, OP)?)
            }
        }
        // Up-and-in call
        (OptionStyle::Call, BarrierType::UpAndIn) => {
            if k >= barrier_level {
                Ok(d_add(f_a(dec!(1.0), x1)?, f_f(dec!(-1.0))?, OP)?)
            } else {
                let s1 = d_sub(f_b(dec!(1.0), x2)?, f_d(dec!(1.0), dec!(-1.0), y2)?, OP)?;
                Ok(d_add(s1, f_f(dec!(-1.0))?, OP)?)
            }
        }
        // Down-and-out put
        (OptionStyle::Put, BarrierType::DownAndOut) => {
            if k >= barrier_level {
                let s1 = d_sub(f_b(dec!(-1.0), x2)?, f_d(dec!(-1.0), dec!(1.0), y2)?, OP)?;
                Ok(d_add(s1, f_e(dec!(1.0))?, OP)?)
            } else {
                let s1 = d_sub(f_a(dec!(-1.0), x1)?, f_c(dec!(-1.0), dec!(1.0), y1)?, OP)?;
                Ok(d_add(s1, f_e(dec!(1.0))?, OP)?)
            }
        }
        // Down-and-in put
        (OptionStyle::Put, BarrierType::DownAndIn) => {
            if k >= barrier_level {
                let s1 = d_sub(f_a(dec!(-1.0), x1)?, f_b(dec!(-1.0), x2)?, OP)?;
                let s2 = d_add(s1, f_d(dec!(-1.0), dec!(1.0), y2)?, OP)?;
                Ok(d_add(s2, f_f(dec!(1.0))?, OP)?)
            } else {
                Ok(d_add(f_c(dec!(-1.0), dec!(1.0), y1)?, f_f(dec!(1.0))?, OP)?)
            }
        }
        // Up-and-out put
        (OptionStyle::Put, BarrierType::UpAndOut) => {
            if k >= barrier_level {
                Ok(f_e(dec!(-1.0))?)
            } else {
                let s1 = d_sub(f_a(dec!(-1.0), x1)?, f_b(dec!(-1.0), x2)?, OP)?;
                let s2 = d_add(s1, f_d(dec!(-1.0), dec!(-1.0), y2)?, OP)?;
                Ok(d_add(s2, f_e(dec!(-1.0))?, OP)?)
            }
        }
        // Up-and-in put
        (OptionStyle::Put, BarrierType::UpAndIn) => {
            if k >= barrier_level {
                Ok(d_add(f_a(dec!(-1.0), x1)?, f_f(dec!(-1.0))?, OP)?)
            } else {
                let s1 = d_sub(f_b(dec!(-1.0), x2)?, f_d(dec!(-1.0), dec!(-1.0), y2)?, OP)?;
                Ok(d_add(s1, f_f(dec!(-1.0))?, OP)?)
            }
        }
        // `BarrierType` is `#[non_exhaustive]`: a barrier added upstream has no
        // closed form here until it gets its own arm.
        (_, _) => Err(PricingError::other(
            "barrier_black_scholes: unsupported BarrierType",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::types::{BarrierType, OptionStyle, OptionType, Side};
    use crate::{ExpirationDate, Options};
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_option(style: OptionStyle, barrier_type: BarrierType, level: f64) -> Options {
        Options {
            option_type: OptionType::Barrier {
                barrier_type,
                barrier_level: pos_or_panic!(level),
                rebate: None,
            },
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: pos_or_panic!(100.0),
            expiration_date: ExpirationDate::Days(pos_or_panic!(182.5)), // ~0.5 year
            implied_volatility: pos_or_panic!(0.25),
            quantity: pos_or_panic!(1.0),
            underlying_price: pos_or_panic!(100.0),
            risk_free_rate: dec!(0.08),
            option_style: style,
            dividend_yield: pos_or_panic!(0.04),
            exotic_params: None,
        }
    }

    #[test]
    fn test_down_and_out_call() {
        let option = create_test_option(OptionStyle::Call, BarrierType::DownAndOut, 95.0);
        let price = barrier_black_scholes(&option).unwrap();
        // S=100, K=100, H=95, r=0.08, q=0.04, sigma=0.25, T=0.5
        // Price should be approx 4.5126
        assert!(
            price > dec!(4.5) && price < dec!(4.6),
            "Price was {}",
            price
        );
    }

    #[test]
    fn test_down_and_in_call() {
        let option = create_test_option(OptionStyle::Call, BarrierType::DownAndIn, 95.0);
        let price = barrier_black_scholes(&option).unwrap();
        // Price should be approx 3.3368
        assert!(
            price > dec!(3.3) && price < dec!(3.4),
            "Price was {}",
            price
        );
    }

    #[test]
    fn test_in_out_parity() {
        let out_option = create_test_option(OptionStyle::Call, BarrierType::DownAndOut, 95.0);
        let in_option = create_test_option(OptionStyle::Call, BarrierType::DownAndIn, 95.0);

        let out_price = barrier_black_scholes(&out_option).unwrap();
        let in_price = barrier_black_scholes(&in_option).unwrap();

        let total = out_price + in_price;

        // Should equal vanilla BS call
        let mut vanilla = out_option.clone();
        vanilla.option_type = OptionType::European;
        let vanilla_price = crate::pricing::black_scholes_model::black_scholes(&vanilla).unwrap();

        // Using a slightly larger tolerance for Decimal calculations
        assert!(
            (total - vanilla_price).abs() < dec!(0.001),
            "Total: {}, Vanilla: {}",
            total,
            vanilla_price
        );
    }

    #[test]
    fn test_up_in_out_parity_call() {
        let out_option = create_test_option(OptionStyle::Call, BarrierType::UpAndOut, 105.0);
        let in_option = create_test_option(OptionStyle::Call, BarrierType::UpAndIn, 105.0);

        let out_price = barrier_black_scholes(&out_option).unwrap();
        let in_price = barrier_black_scholes(&in_option).unwrap();
        let total = out_price + in_price;

        let mut vanilla = out_option.clone();
        vanilla.option_type = OptionType::European;
        let vanilla_price = crate::pricing::black_scholes_model::black_scholes(&vanilla).unwrap();

        assert!(
            (total - vanilla_price).abs() < dec!(0.001),
            "Total: {}, Vanilla: {}",
            total,
            vanilla_price
        );
    }

    #[test]
    fn test_down_in_out_parity_put() {
        let out_option = create_test_option(OptionStyle::Put, BarrierType::DownAndOut, 95.0);
        let in_option = create_test_option(OptionStyle::Put, BarrierType::DownAndIn, 95.0);

        let out_price = barrier_black_scholes(&out_option).unwrap();
        let in_price = barrier_black_scholes(&in_option).unwrap();
        let total = out_price + in_price;

        let mut vanilla = out_option.clone();
        vanilla.option_type = OptionType::European;
        let vanilla_price = crate::pricing::black_scholes_model::black_scholes(&vanilla).unwrap();

        assert!(
            (total - vanilla_price).abs() < dec!(0.001),
            "Total: {}, Vanilla: {}",
            total,
            vanilla_price
        );
    }

    #[test]
    fn test_up_in_out_parity_put() {
        let out_option = create_test_option(OptionStyle::Put, BarrierType::UpAndOut, 105.0);
        let in_option = create_test_option(OptionStyle::Put, BarrierType::UpAndIn, 105.0);

        let out_price = barrier_black_scholes(&out_option).unwrap();
        let in_price = barrier_black_scholes(&in_option).unwrap();
        let total = out_price + in_price;

        let mut vanilla = out_option.clone();
        vanilla.option_type = OptionType::European;
        let vanilla_price = crate::pricing::black_scholes_model::black_scholes(&vanilla).unwrap();

        assert!(
            (total - vanilla_price).abs() < dec!(0.001),
            "Total: {}, Vanilla: {}",
            total,
            vanilla_price
        );
    }

    #[test]
    fn test_barrier_greeks() {
        let option = create_test_option(OptionStyle::Call, BarrierType::DownAndOut, 95.0);

        let delta = crate::greeks::delta(&option).unwrap();
        let gamma = crate::greeks::gamma(&option).unwrap();
        let vega = crate::greeks::vega(&option).unwrap();
        let rho = crate::greeks::rho(&option).unwrap();
        // DIC delta can be high near barrier.
        assert!(
            delta > dec!(0.1) && delta < dec!(2.0),
            "Delta was {}",
            delta
        );
        // Barrier Greeks can be negative and have higher magnitudes than vanilla
        tracing::debug!(delta = %delta, gamma = %gamma, vega = %vega, rho = %rho, "Barrier Greeks");
    }
}
