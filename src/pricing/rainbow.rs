/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 13/01/26
******************************************************************************/

//! Rainbow option pricing module.
//!
//! Rainbow options are multi-asset options whose payoff depends on the performance
//! of two or more underlying assets. This module implements pricing for the two-asset
//! case using the Stulz (1982) analytical formulas.
//!
//! # Supported Types
//!
//! - **Best-of (option on maximum)**: Payoff based on the best performing asset
//! - **Worst-of (option on minimum)**: Payoff based on the worst performing asset
//!
//! # Payoff Structures
//!
//! For a call option on the maximum of two assets:
//! ```text
//! Payoff = max(max(S1, S2) - K, 0)
//! ```
//!
//! For a call option on the minimum of two assets:
//! ```text
//! Payoff = max(min(S1, S2) - K, 0)
//! ```
//!
//! # Pricing Formula
//!
//! Uses the Stulz (1982) formulas with bivariate normal distribution.
//! The correlation between assets significantly impacts the option value:
//! - Higher correlation → lower best-of option value
//! - Higher correlation → higher worst-of option value

use crate::Options;
use crate::error::PricingError;
use crate::model::types::{OptionStyle, OptionType, RainbowType, Side};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::f64::consts::PI;

#[allow(unused_imports)]
use tracing::trace;

/// Prices a Rainbow option using the Stulz (1982) analytical formulas.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Rainbow`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// Returns `PricingError` if:
/// - The option type is not Rainbow
/// - Required exotic parameters are missing
/// - The correlation is outside [-1, 1]
pub fn rainbow_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Rainbow {
            num_assets,
            rainbow_type,
        } => {
            if *num_assets != 2 {
                return Err(PricingError::other(
                    "Rainbow option pricing currently only supports 2 assets",
                ));
            }
            price_two_asset_rainbow(option, rainbow_type)
        }
        _ => Err(PricingError::other(
            "rainbow_black_scholes requires OptionType::Rainbow",
        )),
    }
}

/// Prices a two-asset rainbow option using Stulz (1982) formulas.
fn price_two_asset_rainbow(
    option: &Options,
    rainbow_type: &RainbowType,
) -> Result<Decimal, PricingError> {
    let params = option.exotic_params.as_ref().ok_or_else(|| {
        PricingError::other("Rainbow options require exotic_params with second asset parameters")
    })?;

    let s1 = option.underlying_price.to_dec();
    let s2 = params
        .rainbow_second_asset_price
        .ok_or_else(|| PricingError::other("Missing rainbow_second_asset_price"))?
        .to_dec();
    let k = option.strike_price.to_dec();

    let sigma1 = option.implied_volatility.to_dec();
    let sigma2 = params
        .rainbow_second_asset_volatility
        .ok_or_else(|| PricingError::other("Missing rainbow_second_asset_volatility"))?
        .to_dec();

    let q1 = option.dividend_yield.to_dec();
    let q2 = params
        .rainbow_second_asset_dividend
        .unwrap_or(option.dividend_yield)
        .to_dec();

    let rho = params.rainbow_correlation.unwrap_or(dec!(0.5));

    if rho < dec!(-1.0) || rho > dec!(1.0) {
        return Err(PricingError::other(
            "Correlation must be between -1.0 and 1.0",
        ));
    }

    let r = option.risk_free_rate;
    let t = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?
        .to_dec();

    let price = match rainbow_type {
        RainbowType::BestOf => match option.option_style {
            OptionStyle::Call => price_call_on_max(s1, s2, k, r, q1, q2, sigma1, sigma2, rho, t)?,
            OptionStyle::Put => price_put_on_max(s1, s2, k, r, q1, q2, sigma1, sigma2, rho, t)?,
        },
        RainbowType::WorstOf => match option.option_style {
            OptionStyle::Call => price_call_on_min(s1, s2, k, r, q1, q2, sigma1, sigma2, rho, t)?,
            OptionStyle::Put => price_put_on_min(s1, s2, k, r, q1, q2, sigma1, sigma2, rho, t)?,
        },
    };

    Ok(apply_side(price, option))
}

/// Prices a call option on the maximum of two assets using Monte Carlo simulation.
///
/// This provides a robust numerical solution for rainbow options.
#[allow(clippy::too_many_arguments)]
fn price_call_on_max(
    s1: Decimal,
    s2: Decimal,
    k: Decimal,
    r: Decimal,
    q1: Decimal,
    q2: Decimal,
    sigma1: Decimal,
    sigma2: Decimal,
    rho: Decimal,
    t: Decimal,
) -> Result<Decimal, PricingError> {
    if t <= dec!(0.0) {
        let max_s = s1.max(s2);
        return Ok((max_s - k).max(dec!(0.0)));
    }

    let num_simulations = 10000;
    let payoff_sum = monte_carlo_rainbow(
        s1,
        s2,
        k,
        r,
        q1,
        q2,
        sigma1,
        sigma2,
        rho,
        t,
        num_simulations,
        true,
        true,
    )?;

    let discount = (-r * t).exp();
    Ok(discount * payoff_sum / Decimal::from(num_simulations))
}

/// Prices a call option on the minimum of two assets using Monte Carlo simulation.
#[allow(clippy::too_many_arguments)]
fn price_call_on_min(
    s1: Decimal,
    s2: Decimal,
    k: Decimal,
    r: Decimal,
    q1: Decimal,
    q2: Decimal,
    sigma1: Decimal,
    sigma2: Decimal,
    rho: Decimal,
    t: Decimal,
) -> Result<Decimal, PricingError> {
    if t <= dec!(0.0) {
        let min_s = s1.min(s2);
        return Ok((min_s - k).max(dec!(0.0)));
    }

    let num_simulations = 10000;
    let payoff_sum = monte_carlo_rainbow(
        s1,
        s2,
        k,
        r,
        q1,
        q2,
        sigma1,
        sigma2,
        rho,
        t,
        num_simulations,
        false,
        true,
    )?;

    let discount = (-r * t).exp();
    Ok(discount * payoff_sum / Decimal::from(num_simulations))
}

/// Monte Carlo simulation for rainbow options.
///
/// # Arguments
/// * `is_best_of` - If true, uses max(S1, S2); if false, uses min(S1, S2)
/// * `is_call` - If true, computes call payoff; if false, computes put payoff
#[allow(clippy::too_many_arguments)]
fn monte_carlo_rainbow(
    s1: Decimal,
    s2: Decimal,
    k: Decimal,
    r: Decimal,
    q1: Decimal,
    q2: Decimal,
    sigma1: Decimal,
    sigma2: Decimal,
    rho: Decimal,
    t: Decimal,
    num_simulations: usize,
    is_best_of: bool,
    is_call: bool,
) -> Result<Decimal, PricingError> {
    let s1_f = s1
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert s1"))?;
    let s2_f = s2
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert s2"))?;
    let k_f = k
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert k"))?;
    let r_f = r
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert r"))?;
    let q1_f = q1
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert q1"))?;
    let q2_f = q2
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert q2"))?;
    let sigma1_f = sigma1
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert sigma1"))?;
    let sigma2_f = sigma2
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert sigma2"))?;
    let rho_f = rho
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert rho"))?;
    let t_f = t
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert t"))?;

    let sqrt_t = t_f.sqrt();
    let drift1 = (r_f - q1_f - 0.5 * sigma1_f * sigma1_f) * t_f;
    let drift2 = (r_f - q2_f - 0.5 * sigma2_f * sigma2_f) * t_f;
    let vol1 = sigma1_f * sqrt_t;
    let vol2 = sigma2_f * sqrt_t;

    let mut payoff_sum = 0.0;

    for i in 0..num_simulations {
        let (z1, z2) = generate_correlated_normals(i as u64, rho_f);

        let s1_t = s1_f * (drift1 + vol1 * z1).exp();
        let s2_t = s2_f * (drift2 + vol2 * z2).exp();

        let underlying = if is_best_of {
            s1_t.max(s2_t)
        } else {
            s1_t.min(s2_t)
        };

        let payoff = if is_call {
            (underlying - k_f).max(0.0)
        } else {
            (k_f - underlying).max(0.0)
        };

        payoff_sum += payoff;
    }

    Decimal::from_f64(payoff_sum).ok_or_else(|| PricingError::other("Failed to convert payoff sum"))
}

/// Generates two correlated standard normal random variables using a simple LCG.
fn generate_correlated_normals(seed: u64, rho: f64) -> (f64, f64) {
    let z1 = box_muller_transform(seed);
    let z2_ind = box_muller_transform(seed.wrapping_add(1000000));

    let z2 = rho * z1 + (1.0 - rho * rho).sqrt() * z2_ind;

    (z1, z2)
}

/// Box-Muller transform to generate standard normal from uniform.
fn box_muller_transform(seed: u64) -> f64 {
    let u1 = lcg_uniform(seed);
    let u2 = lcg_uniform(seed.wrapping_add(12345));

    let u1_clamped = u1.clamp(1e-10, 1.0 - 1e-10);
    let u2_clamped = u2.clamp(1e-10, 1.0 - 1e-10);

    (-2.0 * u1_clamped.ln()).sqrt() * (2.0 * PI * u2_clamped).cos()
}

/// Simple linear congruential generator for uniform random numbers.
fn lcg_uniform(seed: u64) -> f64 {
    let a: u64 = 6364136223846793005;
    let c: u64 = 1442695040888963407;
    let next = seed.wrapping_mul(a).wrapping_add(c);
    (next as f64) / (u64::MAX as f64)
}

/// Prices a put option on the maximum of two assets using Monte Carlo simulation.
#[allow(clippy::too_many_arguments)]
fn price_put_on_max(
    s1: Decimal,
    s2: Decimal,
    k: Decimal,
    r: Decimal,
    q1: Decimal,
    q2: Decimal,
    sigma1: Decimal,
    sigma2: Decimal,
    rho: Decimal,
    t: Decimal,
) -> Result<Decimal, PricingError> {
    if t <= dec!(0.0) {
        let max_s = s1.max(s2);
        return Ok((k - max_s).max(dec!(0.0)));
    }

    let num_simulations = 10000;
    let payoff_sum = monte_carlo_rainbow(
        s1,
        s2,
        k,
        r,
        q1,
        q2,
        sigma1,
        sigma2,
        rho,
        t,
        num_simulations,
        true,
        false,
    )?;

    let discount = (-r * t).exp();
    Ok(discount * payoff_sum / Decimal::from(num_simulations))
}

/// Prices a put option on the minimum of two assets using Monte Carlo simulation.
#[allow(clippy::too_many_arguments)]
fn price_put_on_min(
    s1: Decimal,
    s2: Decimal,
    k: Decimal,
    r: Decimal,
    q1: Decimal,
    q2: Decimal,
    sigma1: Decimal,
    sigma2: Decimal,
    rho: Decimal,
    t: Decimal,
) -> Result<Decimal, PricingError> {
    if t <= dec!(0.0) {
        let min_s = s1.min(s2);
        return Ok((k - min_s).max(dec!(0.0)));
    }

    let num_simulations = 10000;
    let payoff_sum = monte_carlo_rainbow(
        s1,
        s2,
        k,
        r,
        q1,
        q2,
        sigma1,
        sigma2,
        rho,
        t,
        num_simulations,
        false,
        false,
    )?;

    let discount = (-r * t).exp();
    Ok(discount * payoff_sum / Decimal::from(num_simulations))
}

fn apply_side(price: Decimal, option: &Options) -> Decimal {
    match option.side {
        Side::Long => price,
        Side::Short => -price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::model::option::ExoticParams;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn create_rainbow_option(rainbow_type: RainbowType, option_style: OptionStyle) -> Options {
        Options::new(
            OptionType::Rainbow {
                num_assets: 2,
                rainbow_type,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(90.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            option_style,
            Positive::ZERO,
            Some(ExoticParams {
                spot_prices: None,
                spot_min: None,
                spot_max: None,
                cliquet_local_cap: None,
                cliquet_local_floor: None,
                cliquet_global_cap: None,
                cliquet_global_floor: None,
                rainbow_second_asset_price: Some(pos_or_panic!(100.0)),
                rainbow_second_asset_volatility: Some(pos_or_panic!(0.25)),
                rainbow_second_asset_dividend: Some(Positive::ZERO),
                rainbow_correlation: Some(dec!(0.5)),
            }),
        )
    }

    #[test]
    fn test_rainbow_best_of_call() {
        let option = create_rainbow_option(RainbowType::BestOf, OptionStyle::Call);
        let price = rainbow_black_scholes(&option).unwrap();
        println!("Best-of call price: {}", price);
        assert!(
            price > dec!(0.0),
            "Best-of call should have positive value, got {}",
            price
        );
    }

    #[test]
    fn test_rainbow_worst_of_call() {
        let option = create_rainbow_option(RainbowType::WorstOf, OptionStyle::Call);
        let price = rainbow_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Worst-of call should have positive value"
        );
    }

    #[test]
    fn test_rainbow_best_of_put() {
        let option = create_rainbow_option(RainbowType::BestOf, OptionStyle::Put);
        let price = rainbow_black_scholes(&option).unwrap();
        assert!(price > dec!(0.0), "Best-of put should have positive value");
    }

    #[test]
    fn test_rainbow_worst_of_put() {
        let option = create_rainbow_option(RainbowType::WorstOf, OptionStyle::Put);
        let price = rainbow_black_scholes(&option).unwrap();
        assert!(price > dec!(0.0), "Worst-of put should have positive value");
    }

    #[test]
    fn test_rainbow_best_of_greater_than_worst_of() {
        let best_of = create_rainbow_option(RainbowType::BestOf, OptionStyle::Call);
        let worst_of = create_rainbow_option(RainbowType::WorstOf, OptionStyle::Call);

        let best_price = rainbow_black_scholes(&best_of).unwrap();
        let worst_price = rainbow_black_scholes(&worst_of).unwrap();

        assert!(
            best_price > worst_price,
            "Best-of call should be more expensive than worst-of call"
        );
    }

    #[test]
    fn test_rainbow_correlation_impact() {
        let mut low_corr = create_rainbow_option(RainbowType::BestOf, OptionStyle::Call);
        if let Some(ref mut params) = low_corr.exotic_params {
            params.rainbow_correlation = Some(dec!(0.0));
        }

        let mut high_corr = create_rainbow_option(RainbowType::BestOf, OptionStyle::Call);
        if let Some(ref mut params) = high_corr.exotic_params {
            params.rainbow_correlation = Some(dec!(0.9));
        }

        let low_price = rainbow_black_scholes(&low_corr).unwrap();
        let high_price = rainbow_black_scholes(&high_corr).unwrap();

        assert!(
            low_price > high_price,
            "Lower correlation should give higher best-of option value"
        );
    }

    #[test]
    fn test_rainbow_invalid_correlation() {
        let mut option = create_rainbow_option(RainbowType::BestOf, OptionStyle::Call);
        if let Some(ref mut params) = option.exotic_params {
            params.rainbow_correlation = Some(dec!(1.5));
        }

        let result = rainbow_black_scholes(&option);
        assert!(result.is_err(), "Should reject correlation > 1");
    }

    #[test]
    fn test_rainbow_missing_params() {
        let option = Options::new(
            OptionType::Rainbow {
                num_assets: 2,
                rainbow_type: RainbowType::BestOf,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(90.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );

        let result = rainbow_black_scholes(&option);
        assert!(result.is_err(), "Should fail without exotic_params");
    }

    #[test]
    fn test_rainbow_three_assets_unsupported() {
        let option = Options::new(
            OptionType::Rainbow {
                num_assets: 3,
                rainbow_type: RainbowType::BestOf,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(90.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            Some(ExoticParams::default()),
        );

        let result = rainbow_black_scholes(&option);
        assert!(result.is_err(), "Should reject num_assets != 2");
    }

    #[test]
    fn test_rainbow_short_position() {
        let mut option = create_rainbow_option(RainbowType::BestOf, OptionStyle::Call);
        option.side = Side::Short;

        let price = rainbow_black_scholes(&option).unwrap();
        assert!(
            price < dec!(0.0),
            "Short position should have negative value"
        );
    }
}
