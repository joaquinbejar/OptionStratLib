/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 13/01/26
******************************************************************************/

//! Chooser option pricing module.
//!
//! Chooser options (also called as-you-like-it options) allow the holder to
//! choose at a specified date whether the option becomes a call or a put.
//!
//! # Simple Chooser (Rubinstein 1991)
//!
//! At the choice date t, the holder chooses max(Call, Put).
//! The value is:
//!
//! `V = S*e^(-qT)*N(d1) - K*e^(-rT)*N(d2) + K*e^(-rt)*N(-y2) - S*e^(-qt)*N(-y1)`
//!
//! Where:
//! - T = time to final expiration
//! - t = time to choice date
//! - d1, d2 are standard BS d-values for T
//! - y1 = [ln(S/K) + b*t + (σ²/2)*t] / (σ√t)
//! - y2 = y1 - σ√t

use crate::Options;
use crate::error::PricingError;
use crate::greeks::{big_n, d1, d2};
use crate::model::decimal::{d_add, d_div, d_exp, d_ln, d_mul, d_sqrt, d_sub};
use crate::model::types::OptionType;
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::cmp::Ordering;

/// Prices a Chooser option using Rubinstein (1991) simple chooser formula.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Chooser`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// - [`PricingError::MethodError`] when `option` is not an
///   [`OptionType::Chooser`] variant, when the expiration cannot be converted
///   to a year fraction, when the `d1` / `d2` kernels reject the inputs, or
///   when both the choice-date diffusion `σ√t` and the log-moneyness collapse
///   to zero, which leaves `y1` genuinely undefined.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range: the discount factors, the zero-volatility
///   forward, `y1` / `y2`, or the four price legs.
pub fn chooser_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Chooser { choice_date } => simple_chooser_price(option, choice_date.to_f64()),
        _ => Err(PricingError::other(
            "chooser_black_scholes requires OptionType::Chooser",
        )),
    }
}

/// Prices a simple chooser option.
///
/// At the choice date, the holder chooses the maximum of call or put value.
/// Uses Rubinstein (1991) closed-form solution.
fn simple_chooser_price(option: &Options, choice_date_days: f64) -> Result<Decimal, PricingError> {
    let s = option.underlying_price;
    let k = option.strike_price;
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let t_big = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    // Convert choice_date from days to years
    let t_choice = Positive::new(choice_date_days / 365.0).unwrap_or(Positive::ZERO);

    // Validation: choice date must be before expiration
    if t_choice >= t_big {
        // If choice at or after expiration, it's just max(call, put) = straddle-like
        return price_at_choice_equals_expiry(option);
    }

    if t_big == Positive::ZERO {
        // At expiration, intrinsic value
        let call_intrinsic =
            d_sub(s.to_dec(), k.to_dec(), "pricing::chooser::intrinsic::call")?.max(Decimal::ZERO);
        let put_intrinsic =
            d_sub(k.to_dec(), s.to_dec(), "pricing::chooser::intrinsic::put")?.max(Decimal::ZERO);
        return Ok(apply_side(call_intrinsic.max(put_intrinsic), option));
    }

    if sigma == Positive::ZERO {
        // Zero vol: deterministic choice
        let discount_t = d_exp(
            d_mul(-r, t_big.to_dec(), "pricing::chooser::zero_vol::neg_rt")?,
            "pricing::chooser::zero_vol::discount",
        )?;
        let forward = d_mul(
            s.to_dec(),
            d_exp(
                d_mul(
                    d_sub(r, q, "pricing::chooser::zero_vol::carry")?,
                    t_big.to_dec(),
                    "pricing::chooser::zero_vol::carry_t",
                )?,
                "pricing::chooser::zero_vol::growth",
            )?,
            "pricing::chooser::zero_vol::forward",
        )?;
        let call_intrinsic = d_sub(
            forward,
            k.to_dec(),
            "pricing::chooser::zero_vol::call::intrinsic",
        )?
        .max(Decimal::ZERO);
        let put_intrinsic = d_sub(
            k.to_dec(),
            forward,
            "pricing::chooser::zero_vol::put::intrinsic",
        )?
        .max(Decimal::ZERO);
        let call_val = d_mul(
            call_intrinsic,
            discount_t,
            "pricing::chooser::zero_vol::call::discounted",
        )?;
        let put_val = d_mul(
            put_intrinsic,
            discount_t,
            "pricing::chooser::zero_vol::put::discounted",
        )?;
        return Ok(apply_side(call_val.max(put_val), option));
    }

    let b = d_sub(r, q, "pricing::chooser::carry")?;
    let t_big_dec = t_big.to_dec();
    let t_choice_dec = t_choice.to_dec();
    let sqrt_t_choice = d_sqrt(t_choice_dec, "pricing::chooser::sqrt_t_choice")?;

    // Standard BS d-values for the final expiration T. Both reject a zero
    // underlying or strike, so `S / K` below is a ratio of two positives.
    let d1_val = d1(s, k, b, t_big, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b, t_big, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    // d-values for the choice date t
    // y1 = [ln(S/K) + (b + σ²/2)*t] / (σ√t)
    // y2 = y1 - σ√t
    let sigma_dec = sigma.to_dec();
    let sigma_sqrt_t_choice = d_mul(
        sigma_dec,
        sqrt_t_choice,
        "pricing::chooser::sigma_sqrt_t_choice",
    )?;
    let drift = d_mul(
        d_add(
            b,
            d_div(
                d_mul(sigma_dec, sigma_dec, "pricing::chooser::variance")?,
                dec!(2),
                "pricing::chooser::half_variance",
            )?,
            "pricing::chooser::drift_rate",
        )?,
        t_choice_dec,
        "pricing::chooser::drift",
    )?;
    let moneyness = d_div(s.to_dec(), k.to_dec(), "pricing::chooser::moneyness")?;
    // `None` carries the `ln(S / K) = -∞` limit reached when the moneyness
    // rounds below the smallest representable `Decimal`.
    let y1_numerator = if moneyness.is_zero() {
        None
    } else {
        Some(d_add(
            d_ln(moneyness, "pricing::chooser::log_moneyness")?,
            drift,
            "pricing::chooser::y1_numerator",
        )?)
    };

    // `N(-y)` at the choice date. When the choice-date diffusion `σ√t`
    // collapses to zero the exponents diverge and the CDF saturates at the
    // sign of the numerator; `0 / 0` stays genuinely undefined and is
    // reported rather than guessed.
    let (n_neg_y1, n_neg_y2) = match (y1_numerator, sigma_sqrt_t_choice.is_zero()) {
        (None, _) => (Decimal::ONE, Decimal::ONE),
        (Some(numerator), true) => match numerator.cmp(&Decimal::ZERO) {
            Ordering::Greater => (Decimal::ZERO, Decimal::ZERO),
            Ordering::Less => (Decimal::ONE, Decimal::ONE),
            Ordering::Equal => {
                return Err(PricingError::method_error(
                    "simple_chooser_price",
                    "choice-date diffusion and log-moneyness both collapsed to zero",
                ));
            }
        },
        (Some(numerator), false) => {
            let y1 = d_div(numerator, sigma_sqrt_t_choice, "pricing::chooser::y1")?;
            let y2 = d_sub(y1, sigma_sqrt_t_choice, "pricing::chooser::y2")?;
            (
                big_n(-y1).unwrap_or(Decimal::ZERO),
                big_n(-y2).unwrap_or(Decimal::ZERO),
            )
        }
    };

    // Get cumulative normal values
    let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
    let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);

    // Discount factors
    let dividend_discount_t = d_exp(
        d_mul(-q, t_big_dec, "pricing::chooser::neg_qt")?,
        "pricing::chooser::dividend_discount_t",
    )?;
    let discount_t = d_exp(
        d_mul(-r, t_big_dec, "pricing::chooser::neg_rt")?,
        "pricing::chooser::discount_t",
    )?;
    let dividend_discount_choice = d_exp(
        d_mul(-q, t_choice_dec, "pricing::chooser::neg_qt_choice")?,
        "pricing::chooser::dividend_discount_choice",
    )?;
    let discount_choice = d_exp(
        d_mul(-r, t_choice_dec, "pricing::chooser::neg_rt_choice")?,
        "pricing::chooser::discount_choice",
    )?;

    // Rubinstein (1991) simple chooser formula:
    // V = S*e^(-qT)*N(d1) - K*e^(-rT)*N(d2) + K*e^(-rt)*N(-y2) - S*e^(-qt)*N(-y1)
    // This equals: Call(K, T) + Put_component_for_choice_flexibility.
    // Every leg is now built via two chained `d_mul` calls so the
    // leading monetary product (underlying * dividend discount, or
    // strike * discount) is checked and a subsequent saturation on
    // the CDF weight cannot mask the original overflow.
    let leg_s_t_discounted = d_mul(
        s.to_dec(),
        dividend_discount_t,
        "pricing::chooser::price::leg_s_t_discounted",
    )?;
    let leg_s_t = d_mul(leg_s_t_discounted, n_d1, "pricing::chooser::price::leg_s_t")?;
    let leg_k_t_discounted = d_mul(
        k.to_dec(),
        discount_t,
        "pricing::chooser::price::leg_k_t_discounted",
    )?;
    let leg_k_t = d_mul(leg_k_t_discounted, n_d2, "pricing::chooser::price::leg_k_t")?;
    let leg_k_choice_discounted = d_mul(
        k.to_dec(),
        discount_choice,
        "pricing::chooser::price::leg_k_choice_discounted",
    )?;
    let leg_k_choice = d_mul(
        leg_k_choice_discounted,
        n_neg_y2,
        "pricing::chooser::price::leg_k_choice",
    )?;
    let leg_s_choice_discounted = d_mul(
        s.to_dec(),
        dividend_discount_choice,
        "pricing::chooser::price::leg_s_choice_discounted",
    )?;
    let leg_s_choice = d_mul(
        leg_s_choice_discounted,
        n_neg_y1,
        "pricing::chooser::price::leg_s_choice",
    )?;
    let diff1 = d_sub(leg_s_t, leg_k_t, "pricing::chooser::price::diff1")?;
    let diff2 = d_add(diff1, leg_k_choice, "pricing::chooser::price::diff2")?;
    let price = d_sub(diff2, leg_s_choice, "pricing::chooser::price")?;

    Ok(apply_side(price.max(Decimal::ZERO), option))
}

/// Handles the edge case where choice date equals or exceeds expiration.
fn price_at_choice_equals_expiry(option: &Options) -> Result<Decimal, PricingError> {
    // At this point, chooser becomes max(call, put) = straddle at expiry
    // For European option at expiry, this is max(intrinsic_call, intrinsic_put)
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
        let call_intrinsic = d_sub(
            s.to_dec(),
            k.to_dec(),
            "pricing::chooser::expiry::call::intrinsic",
        )?
        .max(Decimal::ZERO);
        let put_intrinsic = d_sub(
            k.to_dec(),
            s.to_dec(),
            "pricing::chooser::expiry::put::intrinsic",
        )?
        .max(Decimal::ZERO);
        return Ok(apply_side(call_intrinsic.max(put_intrinsic), option));
    }

    // Price as call + put (straddle) since choice is at expiry
    let b = d_sub(r, q, "pricing::chooser::expiry::carry")?;
    let d1_val = d1(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
    let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
    let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
    let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);

    let t_dec = t.to_dec();
    let dividend_discount = d_exp(
        d_mul(-q, t_dec, "pricing::chooser::expiry::neg_qt")?,
        "pricing::chooser::expiry::dividend_discount",
    )?;
    let discount = d_exp(
        d_mul(-r, t_dec, "pricing::chooser::expiry::neg_rt")?,
        "pricing::chooser::expiry::discount",
    )?;

    // Call + Put = Straddle
    let s_pv = d_mul(
        s.to_dec(),
        dividend_discount,
        "pricing::chooser::expiry::s_pv",
    )?;
    let k_pv = d_mul(k.to_dec(), discount, "pricing::chooser::expiry::k_pv")?;
    let call_s_leg = d_mul(s_pv, n_d1, "pricing::chooser::expiry::call_s_leg")?;
    let call_k_leg = d_mul(k_pv, n_d2, "pricing::chooser::expiry::call_k_leg")?;
    let put_k_leg = d_mul(k_pv, n_neg_d2, "pricing::chooser::expiry::put_k_leg")?;
    let put_s_leg = d_mul(s_pv, n_neg_d1, "pricing::chooser::expiry::put_s_leg")?;
    let call_price = d_sub(call_s_leg, call_k_leg, "pricing::chooser::expiry::call")?;
    let put_price = d_sub(put_k_leg, put_s_leg, "pricing::chooser::expiry::put")?;
    let price = d_add(call_price, put_price, "pricing::chooser::expiry::price")?;

    Ok(apply_side(price.max(Decimal::ZERO), option))
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

    fn create_chooser_option(choice_date_days: f64) -> Options {
        Options::new(
            OptionType::Chooser {
                choice_date: pos_or_panic!(choice_date_days),
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,                          // strike
            ExpirationDate::Days(pos_or_panic!(182.5)), // ~0.5 years
            pos_or_panic!(0.25),                        // volatility
            Positive::ONE,                              // quantity
            Positive::HUNDRED,                          // underlying (ATM)
            dec!(0.05),                                 // risk-free rate
            OptionStyle::Call,                          // Will be ignored for chooser
            Positive::ZERO,                             // dividend yield
            None,
        )
    }

    #[test]
    fn test_simple_chooser() {
        let option = create_chooser_option(45.0); // Choice in 45 days
        let price = chooser_black_scholes(&option).unwrap();
        // Chooser should have positive value (it's always >= call or put)
        assert!(
            price > Decimal::ZERO,
            "Chooser should be positive: {}",
            price
        );
    }

    #[test]
    fn test_chooser_more_valuable_than_call() {
        let chooser = create_chooser_option(45.0);
        let chooser_price = chooser_black_scholes(&chooser).unwrap();

        // Create equivalent vanilla call
        let call = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(182.5)),
            pos_or_panic!(0.25),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        let call_price = crate::pricing::black_scholes_model::black_scholes(&call).unwrap();

        assert!(
            chooser_price >= call_price,
            "Chooser {} should be >= call {}",
            chooser_price,
            call_price
        );
    }

    #[test]
    fn test_chooser_more_valuable_than_put() {
        let chooser = create_chooser_option(45.0);
        let chooser_price = chooser_black_scholes(&chooser).unwrap();

        // Create equivalent vanilla put
        let put = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(182.5)),
            pos_or_panic!(0.25),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Put,
            Positive::ZERO,
            None,
        );
        let put_price = crate::pricing::black_scholes_model::black_scholes(&put).unwrap();

        assert!(
            chooser_price >= put_price,
            "Chooser {} should be >= put {}",
            chooser_price,
            put_price
        );
    }

    #[test]
    fn test_early_choice_date() {
        // Very early choice date (1 day)
        let option = create_chooser_option(1.0);
        let price = chooser_black_scholes(&option).unwrap();
        assert!(price > Decimal::ZERO, "Early choice date price: {}", price);
    }

    #[test]
    fn test_late_choice_date() {
        // Choice date close to expiration
        let option = create_chooser_option(180.0);
        let price = chooser_black_scholes(&option).unwrap();
        assert!(price > Decimal::ZERO, "Late choice date price: {}", price);
    }

    #[test]
    fn test_choice_at_expiry() {
        // Choice at expiration = straddle
        let option = create_chooser_option(182.5);
        let price = chooser_black_scholes(&option).unwrap();
        assert!(price > Decimal::ZERO, "Choice at expiry price: {}", price);
    }

    #[test]
    fn test_short_chooser_option() {
        let mut option = create_chooser_option(45.0);
        let long_price = chooser_black_scholes(&option).unwrap();

        option.side = Side::Short;
        let short_price = chooser_black_scholes(&option).unwrap();

        assert_decimal_eq!(long_price, -short_price, dec!(1e-10));
    }

    #[test]
    fn test_zero_time_to_expiry() {
        let mut option = create_chooser_option(0.0);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = chooser_black_scholes(&option).unwrap();
        // ATM at expiry, intrinsic is 0
        assert_decimal_eq!(price, Decimal::ZERO, dec!(0.01));
    }

    #[test]
    fn test_itm_call_at_expiry() {
        let mut option = create_chooser_option(0.0);
        option.underlying_price = pos_or_panic!(110.0);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = chooser_black_scholes(&option).unwrap();
        // ITM call intrinsic = 10
        assert_decimal_eq!(price, dec!(10.0), dec!(0.01));
    }

    #[test]
    fn test_itm_put_at_expiry() {
        let mut option = create_chooser_option(0.0);
        option.underlying_price = pos_or_panic!(90.0);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = chooser_black_scholes(&option).unwrap();
        // ITM put intrinsic = 10
        assert_decimal_eq!(price, dec!(10.0), dec!(0.01));
    }

    #[test]
    fn test_higher_vol_means_higher_chooser_value() {
        let low_vol = create_chooser_option(45.0);
        let low_vol_price = chooser_black_scholes(&low_vol).unwrap();

        let mut high_vol = low_vol.clone();
        high_vol.implied_volatility = pos_or_panic!(0.4);
        let high_vol_price = chooser_black_scholes(&high_vol).unwrap();

        assert!(
            high_vol_price > low_vol_price,
            "Higher vol {} should mean higher chooser value: {}",
            high_vol_price,
            low_vol_price
        );
    }
}
