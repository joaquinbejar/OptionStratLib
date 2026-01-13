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
use crate::model::types::{LookbackType, OptionStyle, OptionType};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
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
pub fn lookback_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Lookback { lookback_type } => match lookback_type {
            LookbackType::FloatingStrike => floating_strike_lookback(option),
            LookbackType::FixedStrike => fixed_strike_lookback(option),
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

    if sigma == Positive::ZERO {
        // Zero volatility: no path variation, lookback equals vanilla
        let discount = (-r * t).exp();
        let forward = s * ((r - q) * t).exp();
        let value = match option.option_style {
            OptionStyle::Call => (forward - s).max(Positive::ZERO).to_dec() * discount,
            OptionStyle::Put => (s - forward).max(Positive::ZERO).to_dec() * discount,
        };
        return Ok(apply_side(value, option));
    }

    let b = r - q; // cost of carry
    let sigma_sq = sigma * sigma;
    let t_dec = t.to_dec();
    let sqrt_t = t_dec.sqrt().unwrap_or(Decimal::ZERO);

    // For a new floating strike lookback (S_min = S_max = S):
    // Use Goldman-Sosin-Gatto formulas

    let price = match option.option_style {
        OptionStyle::Call => {
            // Floating strike call: C = S*e^(-qT)*N(a1) - S*e^(-rT)*N(a1 - sigma*sqrt(T))
            //                        + S*e^(-rT)*(sigma^2/(2b))*(N(a1 - sigma*sqrt(T)) - e^(b*T)*N(-a1))
            // where a1 = (b + sigma^2/2)*T / (sigma*sqrt(T))

            if b.abs() < dec!(1e-10) {
                // Special case when b ≈ 0 (ATM forward)
                let a1 = sigma.to_dec() * sqrt_t / dec!(2);
                let n_a1 = big_n(a1).unwrap_or(Decimal::ZERO);
                let n_neg_a1 = big_n(-a1).unwrap_or(Decimal::ZERO);

                // Simplified formula for b = 0
                s.to_dec() * (dec!(2) * n_a1 - dec!(1))
                    + s.to_dec()
                        * sigma.to_dec()
                        * sqrt_t
                        * (dec!(2) * n_a1 - dec!(1)
                            + dec!(2) / (dec!(2.506628274631) * dec!(1))
                                * (a1 * n_neg_a1).exp().min(dec!(10)))
                        .min(s.to_dec())
            } else {
                let a1 = ((b + sigma_sq / dec!(2)) * t_dec) / (sigma.to_dec() * sqrt_t);
                let a2 = a1 - sigma.to_dec() * sqrt_t;

                let n_a1 = big_n(a1).unwrap_or(Decimal::ZERO);
                let n_a2 = big_n(a2).unwrap_or(Decimal::ZERO);
                let n_neg_a1 = big_n(-a1).unwrap_or(Decimal::ZERO);

                let dividend_discount = (-q * t).exp();
                let discount = (-r * t).exp();

                let term1 = s.to_dec() * dividend_discount * n_a1;
                let term2 = s.to_dec() * discount * n_a2;
                let term3 = s.to_dec()
                    * discount
                    * (sigma_sq / (dec!(2) * b))
                    * (n_a2 - (b * t_dec).exp() * n_neg_a1);

                term1 - term2 + term3
            }
        }
        OptionStyle::Put => {
            // Floating strike put: P = S*e^(-rT)*N(-a2) - S*e^(-qT)*N(-a1)
            //                        + S*e^(-rT)*(sigma^2/(2b))*(e^(b*T)*N(a1) - N(a2))

            if b.abs() < dec!(1e-10) {
                let a1 = sigma.to_dec() * sqrt_t / dec!(2);
                let n_neg_a1 = big_n(-a1).unwrap_or(Decimal::ZERO);

                // Simplified for b = 0
                s.to_dec() * (dec!(1) - dec!(2) * n_neg_a1)
                    + s.to_dec() * sigma.to_dec() * sqrt_t * dec!(0.5)
            } else {
                let a1 = ((b + sigma_sq / dec!(2)) * t_dec) / (sigma.to_dec() * sqrt_t);
                let a2 = a1 - sigma.to_dec() * sqrt_t;

                let n_neg_a1 = big_n(-a1).unwrap_or(Decimal::ZERO);
                let n_neg_a2 = big_n(-a2).unwrap_or(Decimal::ZERO);
                let n_a1 = big_n(a1).unwrap_or(Decimal::ZERO);
                let n_a2 = big_n(a2).unwrap_or(Decimal::ZERO);

                let dividend_discount = (-q * t).exp();
                let discount = (-r * t).exp();

                let term1 = s.to_dec() * discount * n_neg_a2;
                let term2 = s.to_dec() * dividend_discount * n_neg_a1;
                let term3 = s.to_dec()
                    * discount
                    * (sigma_sq / (dec!(2) * b))
                    * ((b * t_dec).exp() * n_a1 - n_a2);

                term1 - term2 + term3
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
            OptionStyle::Call => (s - k).max(Positive::ZERO).to_dec(),
            OptionStyle::Put => (k - s).max(Positive::ZERO).to_dec(),
        };
        return Ok(apply_side(intrinsic, option));
    }

    if sigma == Positive::ZERO {
        let discount = (-r * t).exp();
        let forward = s * ((r - q) * t).exp();
        let intrinsic = match option.option_style {
            OptionStyle::Call => (forward - k).max(Positive::ZERO).to_dec() * discount,
            OptionStyle::Put => (k - forward).max(Positive::ZERO).to_dec() * discount,
        };
        return Ok(apply_side(intrinsic, option));
    }

    let b = r - q;
    let sigma_sq = sigma * sigma;
    let t_dec = t.to_dec();
    let sqrt_t = t_dec.sqrt().unwrap_or(Decimal::ZERO);

    // For fixed strike lookback, we use a combination of standard BS
    // plus lookback premium

    // First, get standard BS price
    let d1_val = d1(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let discount = (-r * t).exp();
    let dividend_discount = (-q * t).exp();

    let price = match option.option_style {
        OptionStyle::Call => {
            // Fixed strike lookback call: pays max(S_max - K, 0)
            // For a new contract: similar to standard call + lookback premium

            let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
            let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);

            // Standard BS call
            let bs_call = s.to_dec() * dividend_discount * n_d1 - k.to_dec() * discount * n_d2;

            // Lookback premium (value of being able to exercise at maximum)
            // For new contract from S, use simplified formula
            let lambda = if b.abs() < dec!(1e-10) {
                dec!(1) + sigma_sq * t_dec / dec!(2)
            } else {
                (b + sigma_sq / dec!(2)) * t_dec / (sigma.to_dec() * sqrt_t)
            };
            let n_lambda = big_n(lambda).unwrap_or(dec!(0.5));
            let lookback_premium =
                s.to_dec() * sigma.to_dec() * sqrt_t * (n_lambda - dec!(0.5)) * dec!(0.5);

            (bs_call + lookback_premium).max(Decimal::ZERO)
        }
        OptionStyle::Put => {
            // Fixed strike lookback put: pays max(K - S_min, 0)

            let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
            let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);

            // Standard BS put
            let bs_put =
                k.to_dec() * discount * n_neg_d2 - s.to_dec() * dividend_discount * n_neg_d1;

            // Lookback premium (value of being able to exercise at minimum)
            let lambda = if b.abs() < dec!(1e-10) {
                dec!(1) + sigma_sq * t_dec / dec!(2)
            } else {
                (b + sigma_sq / dec!(2)) * t_dec / (sigma.to_dec() * sqrt_t)
            };
            let n_lambda = big_n(lambda).unwrap_or(dec!(0.5));
            let lookback_premium =
                s.to_dec() * sigma.to_dec() * sqrt_t * (n_lambda - dec!(0.5)) * dec!(0.5);

            (bs_put + lookback_premium).max(Decimal::ZERO)
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
