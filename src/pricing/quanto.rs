//! Quanto Option Pricing Module
//!
//! This module implements pricing for Quanto options, which are derivatives where
//! the underlying asset is denominated in one currency (foreign) but the payoff
//! is settled in another currency (domestic) at a fixed exchange rate.
//!
//! # Quanto Adjustment
//!
//! The key insight is that the drift of the underlying asset must be adjusted
//! for the correlation between the asset and the exchange rate:
//!
//! Adjusted drift = r_d - q - ρ × σ_S × σ_FX
//!
//! Where:
//! - r_d: Domestic risk-free rate
//! - q: Dividend yield of the underlying
//! - ρ: Correlation between asset and FX rate
//! - σ_S: Volatility of the underlying asset
//! - σ_FX: Volatility of the exchange rate
//!
//! # Common Applications
//!
//! - Foreign equity investments with currency protection
//! - Commodity options settled in a different currency
//! - Cross-border structured products

use crate::Options;
use crate::error::PricingError;
use crate::greeks::big_n;
use crate::model::types::{OptionStyle, OptionType, Side};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

/// Prices a Quanto option using the quanto-adjusted Black-Scholes formula.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Quanto`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// Returns an error if:
/// - The option type is not `Quanto`
/// - Required exotic parameters are missing
/// - Correlation is outside the valid range [-1, 1]
pub fn quanto_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    let exchange_rate = match &option.option_type {
        OptionType::Quanto { exchange_rate } => Decimal::from_f64(*exchange_rate)
            .ok_or_else(|| PricingError::other("Failed to convert exchange_rate to Decimal"))?,
        _ => {
            return Err(PricingError::other(
                "quanto_black_scholes requires OptionType::Quanto",
            ));
        }
    };

    let params = option
        .exotic_params
        .as_ref()
        .ok_or_else(|| PricingError::other("Quanto options require exotic_params"))?;

    let sigma_fx = params
        .quanto_fx_volatility
        .ok_or_else(|| PricingError::other("Missing quanto_fx_volatility"))?;

    let rho = params
        .quanto_fx_correlation
        .ok_or_else(|| PricingError::other("Missing quanto_fx_correlation"))?;

    if rho < dec!(-1.0) || rho > dec!(1.0) {
        return Err(PricingError::other("Correlation must be between -1 and 1"));
    }

    let s = Decimal::from(option.underlying_price);
    let k = Decimal::from(option.strike_price);
    let r_d = option.risk_free_rate;
    let q = Decimal::from(option.dividend_yield);
    let sigma_s = Decimal::from(option.implied_volatility);
    let t = Decimal::from(option.expiration_date.get_years()?);

    if t <= dec!(0.0) {
        let intrinsic = match option.option_style {
            OptionStyle::Call => (s - k).max(dec!(0.0)),
            OptionStyle::Put => (k - s).max(dec!(0.0)),
        };
        return Ok(apply_side(intrinsic * exchange_rate, option));
    }

    let price = quanto_price(
        s,
        k,
        r_d,
        q,
        sigma_s,
        Decimal::from(sigma_fx),
        rho,
        t,
        exchange_rate,
        &option.option_style,
    )?;

    Ok(apply_side(price, option))
}

/// Computes the quanto-adjusted Black-Scholes price.
///
/// # Arguments
///
/// * `s` - Spot price of the underlying asset (in foreign currency)
/// * `k` - Strike price (in foreign currency)
/// * `r_d` - Domestic risk-free interest rate
/// * `q` - Dividend yield of the underlying
/// * `sigma_s` - Volatility of the underlying asset
/// * `sigma_fx` - Volatility of the exchange rate
/// * `rho` - Correlation between asset and FX rate
/// * `t` - Time to expiration in years
/// * `x` - Fixed exchange rate (domestic/foreign)
/// * `style` - Option style (Call or Put)
#[allow(clippy::too_many_arguments)]
fn quanto_price(
    s: Decimal,
    k: Decimal,
    r_d: Decimal,
    q: Decimal,
    sigma_s: Decimal,
    sigma_fx: Decimal,
    rho: Decimal,
    t: Decimal,
    x: Decimal,
    style: &OptionStyle,
) -> Result<Decimal, PricingError> {
    let quanto_adjustment = rho * sigma_s * sigma_fx;
    let adjusted_drift = r_d - q - quanto_adjustment;

    let forward = s * (adjusted_drift * t).exp();

    let sqrt_t = t
        .sqrt()
        .ok_or_else(|| PricingError::other("Failed to compute sqrt(t)"))?;

    let d1 = ((forward / k).ln() + (sigma_s * sigma_s / dec!(2.0)) * t) / (sigma_s * sqrt_t);
    let d2 = d1 - sigma_s * sqrt_t;

    let discount = (-r_d * t).exp();

    let price = match style {
        OptionStyle::Call => x * discount * (forward * big_n(d1)? - k * big_n(d2)?),
        OptionStyle::Put => x * discount * (k * big_n(-d2)? - forward * big_n(-d1)?),
    };

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
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn create_quanto_option(option_style: OptionStyle) -> Options {
        Options::new(
            OptionType::Quanto {
                exchange_rate: 1.25,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(90.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(105.0),
            dec!(0.05),
            option_style,
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
                quanto_fx_volatility: Some(pos_or_panic!(0.1)),
                quanto_fx_correlation: Some(dec!(0.3)),
                quanto_foreign_rate: Some(dec!(0.03)),
                exchange_second_asset_volatility: None,
                exchange_second_asset_dividend: None,
                exchange_correlation: None,
            }),
        )
    }

    #[test]
    fn test_quanto_call_positive_value() {
        let option = create_quanto_option(OptionStyle::Call);
        let price = quanto_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Quanto call should have positive value, got {}",
            price
        );
    }

    #[test]
    fn test_quanto_put_positive_value() {
        let option = create_quanto_option(OptionStyle::Put);
        let price = quanto_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Quanto put should have positive value, got {}",
            price
        );
    }

    #[test]
    fn test_quanto_zero_correlation() {
        let mut option = create_quanto_option(OptionStyle::Call);
        if let Some(ref mut params) = option.exotic_params {
            params.quanto_fx_correlation = Some(dec!(0.0));
        }

        let price = quanto_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Quanto with zero correlation should have positive value"
        );
    }

    #[test]
    fn test_quanto_positive_correlation_reduces_call() {
        let mut low_corr = create_quanto_option(OptionStyle::Call);
        if let Some(ref mut params) = low_corr.exotic_params {
            params.quanto_fx_correlation = Some(dec!(0.0));
        }

        let mut high_corr = create_quanto_option(OptionStyle::Call);
        if let Some(ref mut params) = high_corr.exotic_params {
            params.quanto_fx_correlation = Some(dec!(0.8));
        }

        let low_price = quanto_black_scholes(&low_corr).unwrap();
        let high_price = quanto_black_scholes(&high_corr).unwrap();

        assert!(
            low_price > high_price,
            "Positive correlation should reduce quanto call value"
        );
    }

    #[test]
    fn test_quanto_negative_correlation_increases_call() {
        let mut zero_corr = create_quanto_option(OptionStyle::Call);
        if let Some(ref mut params) = zero_corr.exotic_params {
            params.quanto_fx_correlation = Some(dec!(0.0));
        }

        let mut neg_corr = create_quanto_option(OptionStyle::Call);
        if let Some(ref mut params) = neg_corr.exotic_params {
            params.quanto_fx_correlation = Some(dec!(-0.5));
        }

        let zero_price = quanto_black_scholes(&zero_corr).unwrap();
        let neg_price = quanto_black_scholes(&neg_corr).unwrap();

        assert!(
            neg_price > zero_price,
            "Negative correlation should increase quanto call value"
        );
    }

    #[test]
    fn test_quanto_invalid_correlation() {
        let mut option = create_quanto_option(OptionStyle::Call);
        if let Some(ref mut params) = option.exotic_params {
            params.quanto_fx_correlation = Some(dec!(1.5));
        }

        let result = quanto_black_scholes(&option);
        assert!(result.is_err(), "Should reject correlation > 1");
    }

    #[test]
    fn test_quanto_missing_params() {
        let option = Options::new(
            OptionType::Quanto {
                exchange_rate: 1.25,
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

        let result = quanto_black_scholes(&option);
        assert!(result.is_err(), "Should fail without exotic_params");
    }

    #[test]
    fn test_quanto_short_position() {
        let mut option = create_quanto_option(OptionStyle::Call);
        option.side = Side::Short;

        let price = quanto_black_scholes(&option).unwrap();
        assert!(
            price < dec!(0.0),
            "Short position should have negative value"
        );
    }

    #[test]
    fn test_quanto_exchange_rate_scaling() {
        let mut option1 = create_quanto_option(OptionStyle::Call);
        option1.option_type = OptionType::Quanto { exchange_rate: 1.0 };

        let mut option2 = create_quanto_option(OptionStyle::Call);
        option2.option_type = OptionType::Quanto { exchange_rate: 2.0 };

        let price1 = quanto_black_scholes(&option1).unwrap();
        let price2 = quanto_black_scholes(&option2).unwrap();

        let ratio = price2 / price1;
        assert!(
            (ratio - dec!(2.0)).abs() < dec!(0.01),
            "Doubling exchange rate should double the price, ratio = {}",
            ratio
        );
    }

    #[test]
    fn test_quanto_zero_fx_volatility() {
        let mut option = create_quanto_option(OptionStyle::Call);
        if let Some(ref mut params) = option.exotic_params {
            params.quanto_fx_volatility = Some(pos_or_panic!(0.0001));
        }

        let price = quanto_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Quanto with near-zero FX volatility should still price correctly"
        );
    }

    #[test]
    fn test_quanto_deep_itm_call() {
        let mut option = create_quanto_option(OptionStyle::Call);
        option.underlying_price = pos_or_panic!(150.0);

        let price = quanto_black_scholes(&option).unwrap();
        let exchange_rate = dec!(1.25);
        let intrinsic = (dec!(150.0) - dec!(100.0)) * exchange_rate;

        assert!(
            price >= intrinsic * dec!(0.9),
            "Deep ITM quanto call should be close to intrinsic value"
        );
    }

    #[test]
    fn test_quanto_deep_otm_call() {
        let mut option = create_quanto_option(OptionStyle::Call);
        option.underlying_price = pos_or_panic!(50.0);

        let price = quanto_black_scholes(&option).unwrap();

        assert!(
            price < dec!(5.0),
            "Deep OTM quanto call should have small value"
        );
    }
}
