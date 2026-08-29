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
use crate::model::decimal::{d_add, d_div, d_exp, d_ln, d_mul, d_sqrt, d_sub};
use crate::model::types::{OptionStyle, OptionType, Side};
use rust_decimal::Decimal;
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
/// - [`PricingError::MethodError`] when the option type is not `Quanto`, when
///   the required exotic parameters are missing, or when the correlation is
///   outside `[-1, 1]`.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range: the quanto drift adjustment, the forward,
///   the discount factor, `d1` / `d2`, or the converted price legs.
/// - `PricingError::ExpirationDate` when the expiration cannot be converted.
pub fn quanto_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    let exchange_rate = match &option.option_type {
        OptionType::Quanto { exchange_rate } => exchange_rate.to_dec(),
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
            OptionStyle::Call => d_sub(s, k, "pricing::quanto::intrinsic::call")?.max(dec!(0.0)),
            OptionStyle::Put => d_sub(k, s, "pricing::quanto::intrinsic::put")?.max(dec!(0.0)),
        };
        return Ok(apply_side(
            d_mul(
                intrinsic,
                exchange_rate,
                "pricing::quanto::intrinsic::converted",
            )?,
            option,
        ));
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
    let quanto_adjustment = d_mul(
        d_mul(rho, sigma_s, "pricing::quanto::rho_sigma_s")?,
        sigma_fx,
        "pricing::quanto::adjustment",
    )?;
    let adjusted_drift = d_sub(
        d_sub(r_d, q, "pricing::quanto::carry")?,
        quanto_adjustment,
        "pricing::quanto::adjusted_drift",
    )?;

    let forward = d_mul(
        s,
        d_exp(
            d_mul(adjusted_drift, t, "pricing::quanto::drift_t")?,
            "pricing::quanto::growth",
        )?,
        "pricing::quanto::forward",
    )?;

    let sqrt_t = d_sqrt(t, "pricing::quanto::sqrt_t")?;
    let denominator = d_mul(sigma_s, sqrt_t, "pricing::quanto::denominator")?;
    let discount = d_exp(
        d_mul(-r_d, t, "pricing::quanto::neg_rt")?,
        "pricing::quanto::discount",
    )?;

    // `K = 0`: the call is certain to be exercised, the put is worthless; a
    // collapsed `σ√T` freezes the underlying at its forward. Both are the
    // limits of the formula below, where the normal arguments diverge and the
    // CDFs saturate.
    let moneyness = if k.is_zero() {
        None
    } else {
        Some(d_div(forward, k, "pricing::quanto::moneyness")?)
    };
    let (n_d1, n_d2, n_neg_d1, n_neg_d2) = match moneyness {
        None => (dec!(1.0), dec!(1.0), dec!(0.0), dec!(0.0)),
        Some(ratio) if ratio.is_zero() => (dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0)),
        Some(ratio) if denominator.is_zero() => {
            if ratio >= dec!(1.0) {
                (dec!(1.0), dec!(1.0), dec!(0.0), dec!(0.0))
            } else {
                (dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0))
            }
        }
        Some(ratio) => {
            let d1 = d_div(
                d_add(
                    d_ln(ratio, "pricing::quanto::log_moneyness")?,
                    d_mul(
                        d_div(
                            d_mul(sigma_s, sigma_s, "pricing::quanto::variance")?,
                            dec!(2.0),
                            "pricing::quanto::half_variance",
                        )?,
                        t,
                        "pricing::quanto::variance_t",
                    )?,
                    "pricing::quanto::d1_numerator",
                )?,
                denominator,
                "pricing::quanto::d1",
            )?;
            let d2 = d_sub(d1, denominator, "pricing::quanto::d2")?;
            (big_n(d1)?, big_n(d2)?, big_n(-d1)?, big_n(-d2)?)
        }
    };

    let converted_discount = d_mul(x, discount, "pricing::quanto::converted_discount")?;
    let price = match style {
        OptionStyle::Call => d_mul(
            converted_discount,
            d_sub(
                d_mul(forward, n_d1, "pricing::quanto::call::forward")?,
                d_mul(k, n_d2, "pricing::quanto::call::strike")?,
                "pricing::quanto::call::intrinsic",
            )?,
            "pricing::quanto::call",
        )?,
        OptionStyle::Put => d_mul(
            converted_discount,
            d_sub(
                d_mul(k, n_neg_d2, "pricing::quanto::put::strike")?,
                d_mul(forward, n_neg_d1, "pricing::quanto::put::forward")?,
                "pricing::quanto::put::intrinsic",
            )?,
            "pricing::quanto::put",
        )?,
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
                exchange_rate: pos_or_panic!(1.25),
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
                exchange_rate: pos_or_panic!(1.25),
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
        option1.option_type = OptionType::Quanto {
            exchange_rate: pos_or_panic!(1.0),
        };

        let mut option2 = create_quanto_option(OptionStyle::Call);
        option2.option_type = OptionType::Quanto {
            exchange_rate: pos_or_panic!(2.0),
        };

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
