//! Exchange Option Pricing Module
//!
//! This module implements pricing for Exchange options (also known as Margrabe options),
//! which give the holder the right to exchange one asset for another.
//!
//! # Margrabe's Formula (1978)
//!
//! Exchange options have a closed-form solution:
//! - Combined volatility: σ = sqrt(σ1² + σ2² - 2ρσ1σ2)
//! - d1 = (ln(S1/S2) + (q2 - q1 + σ²/2)T) / (σ√T)
//! - d2 = d1 - σ√T
//! - C = S1 × e^(-q1×T) × N(d1) - S2 × e^(-q2×T) × N(d2)
//!
//! # Key Properties
//!
//! - No strike price (K = 0 effectively)
//! - Payoff: max(S1 - S2, 0)
//! - Symmetric: exchange(S1, S2) + exchange(S2, S1) = S1 + S2
//!
//! # Common Applications
//!
//! - Stock-for-stock mergers
//! - Switching options
//! - Outperformance options

use crate::Options;
use crate::error::PricingError;
use crate::greeks::big_n;
use crate::model::types::{OptionType, Side};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

/// Prices an Exchange option using Margrabe's formula.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Exchange`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// Returns an error if:
/// - The option type is not `Exchange`
/// - Required exotic parameters are missing
/// - Correlation is outside the valid range [-1, 1]
pub fn exchange_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    let second_asset_price = match &option.option_type {
        OptionType::Exchange { second_asset } => Decimal::from_f64(*second_asset)
            .ok_or_else(|| PricingError::other("Failed to convert second_asset to Decimal"))?,
        _ => {
            return Err(PricingError::other(
                "exchange_black_scholes requires OptionType::Exchange",
            ));
        }
    };

    let params = option
        .exotic_params
        .as_ref()
        .ok_or_else(|| PricingError::other("Exchange options require exotic_params"))?;

    let sigma2 = params
        .exchange_second_asset_volatility
        .ok_or_else(|| PricingError::other("Missing exchange_second_asset_volatility"))?;

    let q2 = params
        .exchange_second_asset_dividend
        .unwrap_or(positive::Positive::ZERO);

    let rho = params
        .exchange_correlation
        .ok_or_else(|| PricingError::other("Missing exchange_correlation"))?;

    if rho < dec!(-1.0) || rho > dec!(1.0) {
        return Err(PricingError::other("Correlation must be between -1 and 1"));
    }

    let s1 = Decimal::from(option.underlying_price);
    let s2 = second_asset_price;
    let q1 = Decimal::from(option.dividend_yield);
    let sigma1 = Decimal::from(option.implied_volatility);
    let t = Decimal::from(option.expiration_date.get_years()?);

    let price = margrabe_formula(
        s1,
        s2,
        q1,
        Decimal::from(q2),
        sigma1,
        Decimal::from(sigma2),
        rho,
        t,
    )?;

    Ok(apply_side(price, option))
}

/// Margrabe's formula for exchange options.
///
/// Provides a closed-form solution for the option to exchange one asset for another.
///
/// # Arguments
///
/// * `s1` - Price of the first underlying asset
/// * `s2` - Price of the second underlying asset
/// * `q1` - Dividend yield of the first asset
/// * `q2` - Dividend yield of the second asset
/// * `sigma1` - Volatility of the first asset
/// * `sigma2` - Volatility of the second asset
/// * `rho` - Correlation between the two assets
/// * `t` - Time to expiration in years
#[allow(clippy::too_many_arguments)]
fn margrabe_formula(
    s1: Decimal,
    s2: Decimal,
    q1: Decimal,
    q2: Decimal,
    sigma1: Decimal,
    sigma2: Decimal,
    rho: Decimal,
    t: Decimal,
) -> Result<Decimal, PricingError> {
    if t <= dec!(0.0) {
        return Ok((s1 - s2).max(dec!(0.0)));
    }

    let sigma_sq = sigma1 * sigma1 + sigma2 * sigma2 - dec!(2.0) * rho * sigma1 * sigma2;

    let sigma = sigma_sq
        .sqrt()
        .ok_or_else(|| PricingError::other("Failed to compute combined volatility"))?;

    if sigma <= dec!(0.0) {
        return Ok((s1 * (-q1 * t).exp() - s2 * (-q2 * t).exp()).max(dec!(0.0)));
    }

    let sqrt_t = t
        .sqrt()
        .ok_or_else(|| PricingError::other("Failed to compute sqrt(t)"))?;

    let d1 = ((s1 / s2).ln() + (q2 - q1 + sigma * sigma / dec!(2.0)) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;

    let s1_pv = s1 * (-q1 * t).exp();
    let s2_pv = s2 * (-q2 * t).exp();

    let price = s1_pv * big_n(d1)? - s2_pv * big_n(d2)?;

    Ok(price.max(dec!(0.0)))
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
    use crate::model::types::OptionStyle;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn create_exchange_option() -> Options {
        Options::new(
            OptionType::Exchange {
                second_asset: 100.0,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(90.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            pos_or_panic!(0.02),
            Some(ExoticParams {
                spot_prices: None,
                spot_min: None,
                spot_max: None,
                cliquet_local_cap: None,
                cliquet_local_floor: None,
                cliquet_global_cap: None,
                cliquet_global_floor: None,
                rainbow_second_asset_price: None,
                rainbow_second_asset_volatility: None,
                rainbow_second_asset_dividend: None,
                rainbow_correlation: None,
                spread_second_asset_volatility: None,
                spread_second_asset_dividend: None,
                spread_correlation: None,
                quanto_fx_volatility: None,
                quanto_fx_correlation: None,
                quanto_foreign_rate: None,
                exchange_second_asset_volatility: Some(pos_or_panic!(0.25)),
                exchange_second_asset_dividend: Some(pos_or_panic!(0.01)),
                exchange_correlation: Some(dec!(0.5)),
            }),
        )
    }

    #[test]
    fn test_exchange_option_positive_value() {
        let option = create_exchange_option();
        let price = exchange_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Exchange option should have positive value, got {}",
            price
        );
    }

    #[test]
    fn test_exchange_option_atm() {
        let mut option = create_exchange_option();
        option.underlying_price = pos_or_panic!(100.0);
        option.option_type = OptionType::Exchange {
            second_asset: 100.0,
        };

        let price = exchange_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "ATM exchange option should have positive time value"
        );
    }

    #[test]
    fn test_exchange_correlation_impact() {
        let mut low_corr = create_exchange_option();
        if let Some(ref mut params) = low_corr.exotic_params {
            params.exchange_correlation = Some(dec!(0.0));
        }

        let mut high_corr = create_exchange_option();
        if let Some(ref mut params) = high_corr.exotic_params {
            params.exchange_correlation = Some(dec!(0.9));
        }

        let low_price = exchange_black_scholes(&low_corr).unwrap();
        let high_price = exchange_black_scholes(&high_corr).unwrap();

        assert!(
            low_price > high_price,
            "Lower correlation should give higher exchange option value"
        );
    }

    #[test]
    fn test_exchange_invalid_correlation() {
        let mut option = create_exchange_option();
        if let Some(ref mut params) = option.exotic_params {
            params.exchange_correlation = Some(dec!(1.5));
        }

        let result = exchange_black_scholes(&option);
        assert!(result.is_err(), "Should reject correlation > 1");
    }

    #[test]
    fn test_exchange_missing_params() {
        let option = Options::new(
            OptionType::Exchange {
                second_asset: 100.0,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(90.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );

        let result = exchange_black_scholes(&option);
        assert!(result.is_err(), "Should fail without exotic_params");
    }

    #[test]
    fn test_exchange_short_position() {
        let mut option = create_exchange_option();
        option.side = Side::Short;

        let price = exchange_black_scholes(&option).unwrap();
        assert!(
            price < dec!(0.0),
            "Short position should have negative value"
        );
    }

    #[test]
    fn test_exchange_deep_itm() {
        let mut option = create_exchange_option();
        option.underlying_price = pos_or_panic!(150.0);
        option.option_type = OptionType::Exchange {
            second_asset: 100.0,
        };

        let price = exchange_black_scholes(&option).unwrap();
        let intrinsic = dec!(150.0) - dec!(100.0);

        assert!(
            price >= intrinsic * dec!(0.8),
            "Deep ITM exchange option should be close to intrinsic value"
        );
    }

    #[test]
    fn test_exchange_deep_otm() {
        let mut option = create_exchange_option();
        option.underlying_price = pos_or_panic!(50.0);
        option.option_type = OptionType::Exchange {
            second_asset: 100.0,
        };

        let price = exchange_black_scholes(&option).unwrap();

        assert!(
            price < dec!(5.0),
            "Deep OTM exchange option should have small value"
        );
    }

    #[test]
    fn test_exchange_negative_correlation() {
        let mut option = create_exchange_option();
        if let Some(ref mut params) = option.exotic_params {
            params.exchange_correlation = Some(dec!(-0.5));
        }

        let price = exchange_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Exchange option with negative correlation should have positive value"
        );
    }

    #[test]
    fn test_exchange_zero_dividend() {
        let mut option = create_exchange_option();
        option.dividend_yield = Positive::ZERO;
        if let Some(ref mut params) = option.exotic_params {
            params.exchange_second_asset_dividend = Some(Positive::ZERO);
        }

        let price = exchange_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Exchange option with zero dividends should have positive value"
        );
    }

    #[test]
    fn test_exchange_perfect_correlation() {
        let mut option = create_exchange_option();
        if let Some(ref mut params) = option.exotic_params {
            params.exchange_correlation = Some(dec!(1.0));
        }

        let price = exchange_black_scholes(&option).unwrap();
        assert!(
            price >= dec!(0.0),
            "Exchange option with perfect correlation should have non-negative value"
        );
    }
}
