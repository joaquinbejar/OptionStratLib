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
/// Returns `PricingError` if:
/// - The option type is not Asian
/// - Required parameters are invalid (zero volatility, etc.)
pub fn asian_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Asian { averaging_type } => match averaging_type {
            AsianAveragingType::Geometric => geometric_asian_price(option),
            AsianAveragingType::Arithmetic => arithmetic_asian_price(option),
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
        return Ok(intrinsic_value(option));
    }

    if sigma == Positive::ZERO {
        // Deterministic case
        let discount = (-r * t).exp();
        let forward = s * ((r - q) * t).exp();
        let intrinsic = match option.option_style {
            OptionStyle::Call => (forward - k).max(Positive::ZERO).to_dec(),
            OptionStyle::Put => (k - forward).max(Positive::ZERO).to_dec(),
        };
        return Ok(apply_side(intrinsic * discount, option));
    }

    // Geometric average adjustments (Kemna-Vorst)
    let sigma_sq = sigma * sigma;
    let sigma_adj = sigma / Positive::new(3.0_f64.sqrt()).unwrap();
    let b_adj = (r - q - sigma_sq / dec!(6)) / dec!(2);

    // Calculate d1 and d2 with adjusted parameters
    let d1_val = d1(s, k, b_adj, t, sigma_adj)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b_adj, t, sigma_adj)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let discount = (-r * t).exp();

    let price = match option.option_style {
        OptionStyle::Call => {
            let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
            let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
            s.to_dec() * ((b_adj - r) * t).exp() * n_d1 - k.to_dec() * discount * n_d2
        }
        OptionStyle::Put => {
            let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
            let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);
            k.to_dec() * discount * n_neg_d2 - s.to_dec() * ((b_adj - r) * t).exp() * n_neg_d1
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
        return Ok(intrinsic_value(option));
    }

    if sigma == Positive::ZERO {
        let discount = (-r * t).exp();
        let forward = s * ((r - q) * t).exp();
        let intrinsic = match option.option_style {
            OptionStyle::Call => (forward - k).max(Positive::ZERO).to_dec(),
            OptionStyle::Put => (k - forward).max(Positive::ZERO).to_dec(),
        };
        return Ok(apply_side(intrinsic * discount, option));
    }

    // Turnbull-Wakeman approximation
    let b = r - q; // cost of carry
    let sigma_sq = sigma * sigma;
    let t_dec = t.to_dec();

    // First moment of arithmetic average (M1)
    let m1 = if b.abs() < dec!(1e-10) {
        s.to_dec()
    } else {
        s.to_dec() * (((b * t).exp() - dec!(1)) / (b * t_dec))
    };

    // Second moment of arithmetic average (M2)
    let m2 = if b.abs() < dec!(1e-10) {
        let term = (sigma_sq * t_dec).exp();
        s.to_dec().powi(2) * term
    } else {
        let term1_exp = ((dec!(2) * b + sigma_sq) * t_dec).exp();
        let term1 = (dec!(2) * s.to_dec().powi(2) * term1_exp)
            / ((b + sigma_sq) * (dec!(2) * b + sigma_sq) * t_dec.powi(2));

        let term2 = (dec!(2) * s.to_dec().powi(2)) / (b * t_dec.powi(2))
            * (dec!(1) / (dec!(2) * b + sigma_sq) - (b * t_dec).exp() / (b + sigma_sq));

        term1 + term2
    };

    // Adjusted volatility from moment matching
    let variance = (m2 / m1.powi(2)).ln() / t_dec;
    let sigma_adj = variance.sqrt().unwrap_or(sigma.to_dec());
    let sigma_adj_pos = Positive::new_decimal(sigma_adj.max(dec!(0.0001)))
        .unwrap_or(Positive::new(0.0001).unwrap());

    // Forward price of the average
    let f_adj = m1;

    // Use Black-Scholes with adjusted parameters
    let d1_val = ((f_adj / k).ln() + sigma_adj * sigma_adj * t_dec / dec!(2))
        / (sigma_adj * t_dec.sqrt().unwrap());
    let d2_val = d1_val - sigma_adj * t_dec.sqrt().unwrap();

    let discount = (-r * t).exp();

    let price = match option.option_style {
        OptionStyle::Call => {
            let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
            let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
            discount * (f_adj * n_d1 - k.to_dec() * n_d2)
        }
        OptionStyle::Put => {
            let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
            let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);
            discount * (k.to_dec() * n_neg_d2 - f_adj * n_neg_d1)
        }
    };

    // Suppress unused variable warning
    let _ = sigma_adj_pos;

    Ok(apply_side(price, option))
}

/// Calculates intrinsic value at expiration.
fn intrinsic_value(option: &Options) -> Decimal {
    let s = option.underlying_price;
    let k = option.strike_price;
    let value = match option.option_style {
        OptionStyle::Call => (s - k).max(Positive::ZERO).to_dec(),
        OptionStyle::Put => (k - s).max(Positive::ZERO).to_dec(),
    };
    apply_side(value, option)
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
