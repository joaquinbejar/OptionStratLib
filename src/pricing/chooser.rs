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
use crate::model::types::OptionType;
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

/// Prices a Chooser option using Rubinstein (1991) simple chooser formula.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Chooser`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
pub fn chooser_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Chooser { choice_date } => simple_chooser_price(option, *choice_date),
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
        let call_intrinsic = (s.to_dec() - k.to_dec()).max(Decimal::ZERO);
        let put_intrinsic = (k.to_dec() - s.to_dec()).max(Decimal::ZERO);
        return Ok(apply_side(call_intrinsic.max(put_intrinsic), option));
    }

    if sigma == Positive::ZERO {
        // Zero vol: deterministic choice
        let discount_t = (-r * t_big).exp();
        let forward = s.to_dec() * ((r - q) * t_big.to_dec()).exp();
        let call_val = (forward - k.to_dec()).max(Decimal::ZERO) * discount_t;
        let put_val = (k.to_dec() - forward).max(Decimal::ZERO) * discount_t;
        return Ok(apply_side(call_val.max(put_val), option));
    }

    let b = r - q;
    let t_big_dec = t_big.to_dec();
    let t_choice_dec = t_choice.to_dec();
    let _sqrt_t_big = t_big_dec.sqrt().unwrap_or(Decimal::ZERO);
    let sqrt_t_choice = t_choice_dec.sqrt().unwrap_or(dec!(0.001));

    // Standard BS d-values for the final expiration T
    let d1_val = d1(s, k, b, t_big, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b, t_big, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    // d-values for the choice date t
    // y1 = [ln(S/K) + (b + σ²/2)*t] / (σ√t)
    // y2 = y1 - σ√t
    let y1 = ((s.to_dec() / k.to_dec()).ln() + (b + sigma * sigma / dec!(2)) * t_choice_dec)
        / (sigma.to_dec() * sqrt_t_choice);
    let y2 = y1 - sigma.to_dec() * sqrt_t_choice;

    // Get cumulative normal values
    let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
    let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
    let n_neg_y1 = big_n(-y1).unwrap_or(Decimal::ZERO);
    let n_neg_y2 = big_n(-y2).unwrap_or(Decimal::ZERO);

    // Discount factors
    let dividend_discount_t = (-q * t_big_dec).exp();
    let discount_t = (-r * t_big_dec).exp();
    let dividend_discount_choice = (-q * t_choice_dec).exp();
    let discount_choice = (-r * t_choice_dec).exp();

    // Rubinstein (1991) simple chooser formula:
    // V = S*e^(-qT)*N(d1) - K*e^(-rT)*N(d2) + K*e^(-rt)*N(-y2) - S*e^(-qt)*N(-y1)
    // This equals: Call(K, T) + Put_component_for_choice_flexibility
    let price = s.to_dec() * dividend_discount_t * n_d1 - k.to_dec() * discount_t * n_d2
        + k.to_dec() * discount_choice * n_neg_y2
        - s.to_dec() * dividend_discount_choice * n_neg_y1;

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
        let call_intrinsic = (s.to_dec() - k.to_dec()).max(Decimal::ZERO);
        let put_intrinsic = (k.to_dec() - s.to_dec()).max(Decimal::ZERO);
        return Ok(apply_side(call_intrinsic.max(put_intrinsic), option));
    }

    // Price as call + put (straddle) since choice is at expiry
    let b = r - q;
    let d1_val = d1(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
    let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
    let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
    let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);

    let dividend_discount = (-q * t).exp();
    let discount = (-r * t).exp();

    // Call + Put = Straddle
    let call_price = s.to_dec() * dividend_discount * n_d1 - k.to_dec() * discount * n_d2;
    let put_price = k.to_dec() * discount * n_neg_d2 - s.to_dec() * dividend_discount * n_neg_d1;

    Ok(apply_side(
        (call_price + put_price).max(Decimal::ZERO),
        option,
    ))
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
                choice_date: choice_date_days,
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
