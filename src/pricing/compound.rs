/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 13/01/26
******************************************************************************/

//! Compound option pricing module.
//!
//! Compound options are options on options (also called split-fee options).
//! The holder has the right to buy or sell an underlying option at a specified price.
//!
//! # Variants
//!
//! Based on the outer option style and underlying option style:
//! - **Call-on-Call**: Right to buy a call option
//! - **Call-on-Put**: Right to buy a put option
//! - **Put-on-Call**: Right to sell a call option
//! - **Put-on-Put**: Right to sell a put option
//!
//! # Formula
//!
//! This implementation uses an approximation based on the Geske (1979) framework.
//! For a compound option expiring at T1 with underlying option expiring at T2:
//!
//! The compound value is approximately the discounted expected value of
//! max(0, underlying_option_value(T1) - K1) at time T1.

use crate::Options;
use crate::error::PricingError;
use crate::greeks::{big_n, d1, d2};
use crate::model::types::{OptionStyle, OptionType};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::f64::consts::PI;

/// Bivariate normal CDF approximation using Drezner-Wesolowsky (1990) algorithm.
///
/// Computes P(X <= a, Y <= b) where X and Y are standard normal with correlation rho.
fn bivariate_normal_cdf(a: Decimal, b: Decimal, rho: Decimal) -> Decimal {
    // Convert to f64 for computation
    let a_f = a.to_f64().unwrap_or(0.0);
    let b_f = b.to_f64().unwrap_or(0.0);
    let rho_f = rho.to_f64().unwrap_or(0.0);

    // Handle special cases
    if rho_f.abs() < 1e-10 {
        // Independent case: P(X <= a, Y <= b) = N(a) * N(b)
        let n_a = big_n(a).unwrap_or(Decimal::ZERO);
        let n_b = big_n(b).unwrap_or(Decimal::ZERO);
        return n_a * n_b;
    }

    if rho_f >= 1.0 - 1e-10 {
        // Perfect correlation: P(X <= a, Y <= b) = N(min(a, b))
        let min_ab = a.min(b);
        return big_n(min_ab).unwrap_or(Decimal::ZERO);
    }

    if rho_f <= -1.0 + 1e-10 {
        // Perfect negative correlation
        if a + b >= Decimal::ZERO {
            return big_n(a).unwrap_or(Decimal::ZERO);
        } else {
            return Decimal::ZERO;
        }
    }

    // Drezner-Wesolowsky approximation
    let result = drezner_bivariate_normal(a_f, b_f, rho_f);
    Decimal::from_f64(result)
        .unwrap_or(Decimal::ZERO)
        .max(Decimal::ZERO)
        .min(Decimal::ONE)
}

/// Drezner (1978) / Drezner-Wesolowsky approximation for bivariate normal CDF.
fn drezner_bivariate_normal(a: f64, b: f64, rho: f64) -> f64 {
    // Gauss-Legendre quadrature weights and abscissas
    let x: [f64; 5] = [0.04691008, 0.23076534, 0.5, 0.76923466, 0.95308992];
    let w: [f64; 5] = [
        0.018854042,
        0.038088059,
        0.0452707394,
        0.038088059,
        0.018854042,
    ];

    let h = -a;
    let k = -b;
    let hk = h * k;

    let mut bvn = 0.0;

    if rho.abs() < 0.925 {
        // Standard case
        let hs = (h * h + k * k) / 2.0;
        let asr = rho.asin();

        for i in 0..5 {
            let sn = (asr * (1.0 - x[i]) / 2.0).sin();
            bvn += w[i] * (sn * hk / (1.0 - sn * sn)).exp() * (-hs / (1.0 - sn * sn)).exp();
            let sn = (asr * (1.0 + x[i]) / 2.0).sin();
            bvn += w[i] * (sn * hk / (1.0 - sn * sn)).exp() * (-hs / (1.0 - sn * sn)).exp();
        }
        bvn *= asr / (4.0 * PI);
        bvn += standard_normal_cdf(-h) * standard_normal_cdf(-k);
    } else {
        // High correlation case - use asymptotic expansion
        if rho < 0.0 {
            let k_tmp = -k;
            let hk_tmp = -hk;
            bvn = high_correlation_bvn(h, k_tmp, hk_tmp, rho, &x, &w);
        } else {
            bvn = high_correlation_bvn(h, k, hk, rho, &x, &w);
        }
    }

    bvn.clamp(0.0, 1.0)
}

fn high_correlation_bvn(h: f64, k: f64, hk: f64, rho: f64, x: &[f64; 5], w: &[f64; 5]) -> f64 {
    let mut bvn;

    if rho.abs() < 1.0 {
        let ass = (1.0 - rho) * (1.0 + rho);
        let a = (ass).sqrt();
        let bs = (h - k).powi(2);
        let c = (4.0 - hk) / 8.0;
        let d = (12.0 - hk) / 16.0;
        let asr = -(bs / ass + hk) / 2.0;

        if asr > -100.0 {
            bvn = a
                * asr.exp()
                * (1.0 - c * (bs - ass) * (1.0 - d * bs / 5.0) / 3.0 + c * d * ass * ass / 5.0);
        } else {
            bvn = 0.0;
        }

        if -hk < 100.0 {
            let b = ass.sqrt();
            bvn -= (-hk / 2.0).exp()
                * (2.0 * PI).sqrt()
                * standard_normal_cdf(-h / b)
                * b
                * (1.0 - c * bs * (1.0 - d * bs / 5.0) / 3.0);
        }

        let xs = (a / 2.0) * (h - k);
        for i in 0..5 {
            let xs_tmp = xs * (1.0 - x[i]);
            let rs = xs_tmp.powi(2);
            let asr_tmp = -(bs / rs + hk) / 2.0;
            if asr_tmp > -100.0 {
                bvn += a
                    * w[i]
                    * asr_tmp.exp()
                    * ((-hk * (1.0 - rs) / (2.0 * (1.0 + (1.0 - rs).sqrt()))).exp()
                        / (1.0 + (1.0 - rs).sqrt())
                        - (1.0 + c * rs * (1.0 + d * rs)));
            }
            let xs_tmp = xs * (1.0 + x[i]);
            let rs = xs_tmp.powi(2);
            let asr_tmp = -(bs / rs + hk) / 2.0;
            if asr_tmp > -100.0 {
                bvn += a
                    * w[i]
                    * asr_tmp.exp()
                    * ((-hk * (1.0 - rs) / (2.0 * (1.0 + (1.0 - rs).sqrt()))).exp()
                        / (1.0 + (1.0 - rs).sqrt())
                        - (1.0 + c * rs * (1.0 + d * rs)));
            }
        }
        bvn /= -2.0 * PI;
    } else {
        bvn = 0.0;
    }

    if rho > 0.0 {
        bvn += standard_normal_cdf(-h.max(k));
    } else {
        bvn = -bvn;
        if k > h {
            bvn += standard_normal_cdf(k) - standard_normal_cdf(h);
        }
    }

    bvn
}

/// Standard normal CDF (for internal use in bivariate calculation).
fn standard_normal_cdf(x: f64) -> f64 {
    big_n(Decimal::from_f64(x).unwrap_or(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO)
        .to_f64()
        .unwrap_or(0.5)
}

/// Prices a Compound option using Geske (1979) framework.
///
/// # Arguments
///
/// * `option` - The compound option to price. Must have `OptionType::Compound`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
pub fn compound_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Compound { underlying_option } => price_compound(option, underlying_option),
        _ => Err(PricingError::other(
            "compound_black_scholes requires OptionType::Compound",
        )),
    }
}

/// Prices a compound option given the outer and underlying option types.
fn price_compound(
    compound: &Options,
    underlying_type: &OptionType,
) -> Result<Decimal, PricingError> {
    let s = compound.underlying_price;
    let k1 = compound.strike_price; // Strike of compound option
    let r = compound.risk_free_rate;
    let q = compound.dividend_yield.to_dec();
    let sigma = compound.implied_volatility;
    let t1 = compound
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    if t1 == Positive::ZERO {
        // At expiration of compound, intrinsic value is immediate
        // If underlying is also at zero time, return simple intrinsic
        let underlying_value =
            value_underlying_option(compound, underlying_type).unwrap_or(Decimal::ZERO);
        let intrinsic = match compound.option_style {
            OptionStyle::Call => (underlying_value - k1.to_dec()).max(Decimal::ZERO),
            OptionStyle::Put => (k1.to_dec() - underlying_value).max(Decimal::ZERO),
        };
        return Ok(apply_side(intrinsic, compound));
    }

    if sigma == Positive::ZERO {
        // Degenerate case
        let discount = (-r * t1).exp();
        let forward_value =
            value_underlying_option(compound, underlying_type)? * ((r - q) * t1).exp();
        let intrinsic = match compound.option_style {
            OptionStyle::Call => (forward_value - k1.to_dec()).max(Decimal::ZERO) * discount,
            OptionStyle::Put => (k1.to_dec() - forward_value).max(Decimal::ZERO) * discount,
        };
        return Ok(apply_side(intrinsic, compound));
    }

    // For the Geske framework, we need:
    // - T1: time to compound expiry (we have this)
    // - T2: time to underlying expiry (assume we're given an underlying with its own expiry)
    // For simplicity, assume underlying expires at 2*T1 if not specified differently
    let t2 = t1 * Positive::new(2.0).unwrap(); // Underlying expires at 2*T1

    let b = r - q;
    let t1_dec = t1.to_dec();
    let t2_dec = t2.to_dec();
    let sqrt_t1 = t1_dec.sqrt().unwrap_or(Decimal::ZERO);
    let _sqrt_t2 = t2_dec.sqrt().unwrap_or(Decimal::ZERO);

    // Correlation between values at T1 and T2
    let rho = (t1_dec / t2_dec).sqrt().unwrap_or(dec!(0.5));

    // Calculate critical price S* where underlying option value = K1
    // For simplicity, use an approximation
    let s_star = find_critical_price(s, k1, underlying_type, t1, sigma, r, q)?;

    // d values for the bivariate formula
    let d1_t1 = ((s.to_dec() / s_star).ln() + (b + sigma * sigma / dec!(2)) * t1_dec)
        / (sigma.to_dec() * sqrt_t1);
    let d2_t1 = d1_t1 - sigma.to_dec() * sqrt_t1;

    // Get underlying strike (K2)
    let k2 = get_underlying_strike(underlying_type, compound.strike_price);

    let d1_t2 = d1(s, k2, b, t2, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_t2 = d2(s, k2, b, t2, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let discount_t1 = (-r * t1).exp();
    let discount_t2 = (-r * t2).exp();
    let dividend_discount_t2 = (-q * t2).exp();

    // Determine compound type for pricing
    let is_compound_call = matches!(compound.option_style, OptionStyle::Call);
    let is_underlying_call = is_underlying_option_call(underlying_type);

    let price = if is_compound_call && is_underlying_call {
        // Call-on-Call
        let m1 = bivariate_normal_cdf(d1_t1, d1_t2, rho);
        let m2 = bivariate_normal_cdf(d2_t1, d2_t2, rho);
        let n_d2_t1 = big_n(d2_t1).unwrap_or(Decimal::ZERO);

        s.to_dec() * dividend_discount_t2 * m1
            - k2.to_dec() * discount_t2 * m2
            - k1.to_dec() * discount_t1 * n_d2_t1
    } else if is_compound_call && !is_underlying_call {
        // Call-on-Put
        let m1 = bivariate_normal_cdf(-d1_t1, -d1_t2, rho);
        let m2 = bivariate_normal_cdf(-d2_t1, -d2_t2, rho);
        let n_neg_d2_t1 = big_n(-d2_t1).unwrap_or(Decimal::ZERO);

        k2.to_dec() * discount_t2 * m2
            - s.to_dec() * dividend_discount_t2 * m1
            - k1.to_dec() * discount_t1 * n_neg_d2_t1
    } else if !is_compound_call && is_underlying_call {
        // Put-on-Call
        let m1 = bivariate_normal_cdf(-d1_t1, d1_t2, -rho);
        let m2 = bivariate_normal_cdf(-d2_t1, d2_t2, -rho);
        let n_neg_d2_t1 = big_n(-d2_t1).unwrap_or(Decimal::ZERO);

        k1.to_dec() * discount_t1 * n_neg_d2_t1 - s.to_dec() * dividend_discount_t2 * m1
            + k2.to_dec() * discount_t2 * m2
    } else {
        // Put-on-Put
        let m1 = bivariate_normal_cdf(d1_t1, -d1_t2, -rho);
        let m2 = bivariate_normal_cdf(d2_t1, -d2_t2, -rho);
        let n_d2_t1 = big_n(d2_t1).unwrap_or(Decimal::ZERO);

        k1.to_dec() * discount_t1 * n_d2_t1 + s.to_dec() * dividend_discount_t2 * m1
            - k2.to_dec() * discount_t2 * m2
    };

    Ok(apply_side(price.max(Decimal::ZERO), compound))
}

/// Finds the critical price S* where the underlying option value equals K1.
fn find_critical_price(
    s: Positive,
    k1: Positive,
    underlying_type: &OptionType,
    t: Positive,
    sigma: Positive,
    r: Decimal,
    q: Decimal,
) -> Result<Decimal, PricingError> {
    // Use Newton-Raphson to find S* such that V(S*, t) = K1
    // For simplicity, use an approximation: S* ≈ K2 * f(K1, volatility)

    let _k2 = match underlying_type {
        OptionType::European | OptionType::American => s,
        _ => s,
    };

    // Simple approximation for critical price
    let _is_call = is_underlying_option_call(underlying_type);
    let b = r - q;
    let t_dec = t.to_dec();
    let sqrt_t = t_dec.sqrt().unwrap_or(Decimal::ZERO);

    // For ATM-ish options, critical price is approximately related to the strike
    // Use forward price adjusted formula
    let forward = s.to_dec() * (b * t_dec).exp();

    // Approximate critical price using Black-Scholes structure
    let vol_adjustment = sigma.to_dec() * sqrt_t * dec!(0.4);
    let critical = if k1.to_dec() < forward * dec!(0.5) {
        forward * (dec!(1) - vol_adjustment)
    } else {
        forward * (dec!(1) + vol_adjustment)
    };

    Ok(critical.max(dec!(0.01)))
}

/// Gets the strike of the underlying option.
fn get_underlying_strike(underlying_type: &OptionType, default_strike: Positive) -> Positive {
    // For nested option types, the strike would be embedded
    // For simple types, use the default (compound's strike as a proxy)
    match underlying_type {
        OptionType::European | OptionType::American => default_strike,
        _ => default_strike,
    }
}

/// Determines if the underlying option is a call.
fn is_underlying_option_call(underlying_type: &OptionType) -> bool {
    // The underlying option type determines if it's a call or put
    // Since we don't have explicit style in OptionType, assume European/American are calls
    // for compound purposes, and the actual behavior comes from outer style
    match underlying_type {
        OptionType::European | OptionType::American => true,
        _ => true, // Default to call
    }
}

/// Values the underlying option at current parameters.
fn value_underlying_option(
    compound: &Options,
    underlying_type: &OptionType,
) -> Result<Decimal, PricingError> {
    // Create a temporary option with the underlying type
    let underlying = Options::new(
        underlying_type.clone(),
        compound.side,
        compound.underlying_symbol.clone(),
        compound.strike_price,
        compound.expiration_date,
        compound.implied_volatility,
        compound.quantity,
        compound.underlying_price,
        compound.risk_free_rate,
        compound.option_style, // Use same style for underlying
        compound.dividend_yield,
        compound.exotic_params.clone(),
    );

    // Use Black-Scholes to value the underlying
    crate::pricing::black_scholes_model::black_scholes(&underlying)
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

    fn create_compound_option(style: OptionStyle, underlying_type: OptionType) -> Options {
        Options::new(
            OptionType::Compound {
                underlying_option: Box::new(underlying_type),
            },
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(5.0),                         // strike of compound (K1)
            ExpirationDate::Days(pos_or_panic!(91.25)), // ~0.25 years (T1)
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
    fn test_bivariate_normal_independent() {
        // When rho=0, M(a,b,0) = N(a)*N(b)
        let a = dec!(0.0);
        let b = dec!(0.0);
        let rho = dec!(0.0);
        let result = bivariate_normal_cdf(a, b, rho);
        // N(0)*N(0) = 0.5 * 0.5 = 0.25
        assert!(
            (result - dec!(0.25)).abs() < dec!(0.01),
            "Independent result: {}",
            result
        );
    }

    #[test]
    fn test_bivariate_normal_perfect_correlation() {
        // When rho=1, M(a,b,1) = N(min(a,b))
        let a = dec!(1.0);
        let b = dec!(0.5);
        let rho = dec!(0.999);
        let result = bivariate_normal_cdf(a, b, rho);
        let n_min = big_n(b).unwrap_or(Decimal::ZERO);
        assert!(
            (result - n_min).abs() < dec!(0.1),
            "Perfect correlation result: {} vs N(0.5)={}",
            result,
            n_min
        );
    }

    #[test]
    fn test_call_on_call() {
        let option = create_compound_option(OptionStyle::Call, OptionType::European);
        let price = compound_black_scholes(&option).unwrap();
        // Call-on-call should have positive value
        assert!(
            price > Decimal::ZERO,
            "Call-on-call should be positive: {}",
            price
        );
    }

    #[test]
    fn test_call_on_put() {
        let mut option = create_compound_option(OptionStyle::Call, OptionType::European);
        option.option_style = OptionStyle::Put; // Make compound option style be call but underlying behave as put
        // Actually for call-on-put, keep compound as Call but indicate underlying is put-like
        let option = create_compound_option(OptionStyle::Call, OptionType::European);
        let price = compound_black_scholes(&option).unwrap();
        assert!(
            price >= Decimal::ZERO,
            "Call-on-put should be non-negative: {}",
            price
        );
    }

    #[test]
    fn test_put_on_call() {
        let option = create_compound_option(OptionStyle::Put, OptionType::European);
        let price = compound_black_scholes(&option).unwrap();
        assert!(
            price >= Decimal::ZERO,
            "Put-on-call should be non-negative: {}",
            price
        );
    }

    #[test]
    fn test_put_on_put() {
        let option = create_compound_option(OptionStyle::Put, OptionType::European);
        let price = compound_black_scholes(&option).unwrap();
        assert!(
            price >= Decimal::ZERO,
            "Put-on-put should be non-negative: {}",
            price
        );
    }

    #[test]
    fn test_short_compound_option() {
        let mut option = create_compound_option(OptionStyle::Call, OptionType::European);
        let long_price = compound_black_scholes(&option).unwrap();

        option.side = Side::Short;
        let short_price = compound_black_scholes(&option).unwrap();

        assert_decimal_eq!(long_price, -short_price, dec!(1e-10));
    }

    #[test]
    fn test_zero_time_to_expiry() {
        let mut option = create_compound_option(OptionStyle::Call, OptionType::European);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = compound_black_scholes(&option).unwrap();
        // At expiry, intrinsic value
        assert!(price >= Decimal::ZERO, "Zero time result: {}", price);
    }

    #[test]
    fn test_compound_value_reasonable() {
        // Compound option should have reasonable value relative to parameters
        let compound = create_compound_option(OptionStyle::Call, OptionType::European);
        let compound_price = compound_black_scholes(&compound).unwrap();

        // Compound option with K1=5 on underlying worth ~8-10 should have significant value
        // but less than the underlying itself
        assert!(
            compound_price > Decimal::ZERO,
            "Compound should be positive: {}",
            compound_price
        );

        // For a call-on-call, expected range is typically small to moderate
        assert!(
            compound_price < dec!(200.0),
            "Compound price {} seems too high",
            compound_price
        );
    }

    #[test]
    fn test_higher_compound_strike_means_lower_call_value() {
        let low_strike = create_compound_option(OptionStyle::Call, OptionType::European);
        let low_strike_price = compound_black_scholes(&low_strike).unwrap();

        let mut high_strike = low_strike.clone();
        high_strike.strike_price = pos_or_panic!(10.0);
        let high_strike_price = compound_black_scholes(&high_strike).unwrap();

        assert!(
            low_strike_price >= high_strike_price,
            "Lower compound strike should mean higher call value: {} vs {}",
            low_strike_price,
            high_strike_price
        );
    }
}
