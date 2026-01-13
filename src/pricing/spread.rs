//! Spread Option Pricing Module
//!
//! This module implements pricing for spread options, which are multi-asset options
//! whose payoff depends on the difference between two underlying asset prices.
//!
//! # Pricing Methods
//!
//! - **Kirk's Approximation**: For spread options with non-zero strike (K ≠ 0)
//! - **Margrabe's Formula**: Closed-form solution for exchange options (K = 0)
//!
//! # Payoff Structure
//!
//! - Call: max(S1 - S2 - K, 0)
//! - Put: max(K - (S1 - S2), 0) = max(K + S2 - S1, 0)
//!
//! # Common Applications
//!
//! - Energy markets (crack spreads, spark spreads)
//! - Agricultural markets (crush spreads)
//! - Interest rate markets (yield curve spreads)

use crate::Options;
use crate::error::PricingError;
use crate::greeks::big_n;
use crate::model::types::{OptionStyle, OptionType, Side};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

/// Prices a Spread option using Kirk's approximation or Margrabe's formula.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Spread`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// Returns an error if:
/// - The option type is not `Spread`
/// - Required exotic parameters are missing
/// - Correlation is outside the valid range [-1, 1]
pub fn spread_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    let second_asset_price = match &option.option_type {
        OptionType::Spread { second_asset } => Decimal::from_f64(*second_asset)
            .ok_or_else(|| PricingError::other("Failed to convert second_asset to Decimal"))?,
        _ => {
            return Err(PricingError::other(
                "spread_black_scholes requires OptionType::Spread",
            ));
        }
    };

    let params = option
        .exotic_params
        .as_ref()
        .ok_or_else(|| PricingError::other("Spread options require exotic_params"))?;

    let sigma2 = params
        .spread_second_asset_volatility
        .ok_or_else(|| PricingError::other("Missing spread_second_asset_volatility"))?;

    let q2 = params
        .spread_second_asset_dividend
        .unwrap_or(positive::Positive::ZERO);

    let rho = params
        .spread_correlation
        .ok_or_else(|| PricingError::other("Missing spread_correlation"))?;

    if rho < dec!(-1.0) || rho > dec!(1.0) {
        return Err(PricingError::other("Correlation must be between -1 and 1"));
    }

    let s1 = Decimal::from(option.underlying_price);
    let s2 = second_asset_price;
    let k = Decimal::from(option.strike_price);
    let r = option.risk_free_rate;
    let q1 = Decimal::from(option.dividend_yield);
    let sigma1 = Decimal::from(option.implied_volatility);
    let t = Decimal::from(option.expiration_date.get_years()?);

    let price = if k.abs() < dec!(0.0001) {
        margrabe_formula(
            s1,
            s2,
            q1,
            Decimal::from(q2),
            sigma1,
            Decimal::from(sigma2),
            rho,
            t,
        )?
    } else {
        kirk_approximation(
            s1,
            s2,
            k,
            r,
            q1,
            Decimal::from(q2),
            sigma1,
            Decimal::from(sigma2),
            rho,
            t,
            &option.option_style,
        )?
    };

    Ok(apply_side(price, option))
}

/// Kirk's approximation for spread options with non-zero strike.
///
/// Treats the spread option as a call on S1 with adjusted strike (S2 + K).
///
/// # Arguments
///
/// * `s1` - Price of the first underlying asset
/// * `s2` - Price of the second underlying asset
/// * `k` - Strike price
/// * `r` - Risk-free interest rate
/// * `q1` - Dividend yield of the first asset
/// * `q2` - Dividend yield of the second asset
/// * `sigma1` - Volatility of the first asset
/// * `sigma2` - Volatility of the second asset
/// * `rho` - Correlation between the two assets
/// * `t` - Time to expiration in years
/// * `style` - Option style (Call or Put)
#[allow(clippy::too_many_arguments)]
fn kirk_approximation(
    s1: Decimal,
    s2: Decimal,
    k: Decimal,
    r: Decimal,
    q1: Decimal,
    _q2: Decimal,
    sigma1: Decimal,
    sigma2: Decimal,
    rho: Decimal,
    t: Decimal,
    style: &OptionStyle,
) -> Result<Decimal, PricingError> {
    if t <= dec!(0.0) {
        let spread = s1 - s2;
        return match style {
            OptionStyle::Call => Ok((spread - k).max(dec!(0.0))),
            OptionStyle::Put => Ok((k - spread).max(dec!(0.0))),
        };
    }

    let adjusted_strike = s2 + k;
    if adjusted_strike <= dec!(0.0) {
        return Err(PricingError::other(
            "Adjusted strike (S2 + K) must be positive",
        ));
    }

    let s2_ratio = s2 / adjusted_strike;

    let sigma_sq = sigma1 * sigma1 + s2_ratio * s2_ratio * sigma2 * sigma2
        - dec!(2.0) * rho * sigma1 * sigma2 * s2_ratio;

    let sigma = sigma_sq
        .sqrt()
        .ok_or_else(|| PricingError::other("Failed to compute adjusted volatility"))?;

    let sqrt_t = t
        .sqrt()
        .ok_or_else(|| PricingError::other("Failed to compute sqrt(t)"))?;

    let d1 =
        ((s1 / adjusted_strike).ln() + (r - q1 + sigma * sigma / dec!(2.0)) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;

    let s1_pv = s1 * (-q1 * t).exp();
    let adjusted_strike_pv = adjusted_strike * (-r * t).exp();

    match style {
        OptionStyle::Call => {
            let call = s1_pv * big_n(d1)? - adjusted_strike_pv * big_n(d2)?;
            Ok(call.max(dec!(0.0)))
        }
        OptionStyle::Put => {
            let put = adjusted_strike_pv * big_n(-d2)? - s1_pv * big_n(-d1)?;
            Ok(put.max(dec!(0.0)))
        }
    }
}

/// Margrabe's formula for exchange options (K = 0).
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
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn create_spread_option(strike: Positive, option_style: OptionStyle) -> Options {
        Options::new(
            OptionType::Spread {
                second_asset: 100.0,
            },
            Side::Long,
            "TEST".to_string(),
            strike,
            ExpirationDate::Days(pos_or_panic!(90.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(105.0),
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
                rainbow_second_asset_price: None,
                rainbow_second_asset_volatility: None,
                rainbow_second_asset_dividend: None,
                rainbow_correlation: None,
                spread_second_asset_volatility: Some(pos_or_panic!(0.25)),
                spread_second_asset_dividend: Some(Positive::ZERO),
                spread_correlation: Some(dec!(0.5)),
                quanto_fx_volatility: None,
                quanto_fx_correlation: None,
                quanto_foreign_rate: None,
                exchange_second_asset_volatility: None,
                exchange_second_asset_dividend: None,
                exchange_correlation: None,
            }),
        )
    }

    #[test]
    fn test_spread_call_positive_value() {
        let option = create_spread_option(Positive::ZERO, OptionStyle::Call);
        let price = spread_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Spread call should have positive value, got {}",
            price
        );
    }

    #[test]
    fn test_spread_put_positive_value() {
        let option = create_spread_option(pos_or_panic!(10.0), OptionStyle::Put);
        let price = spread_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Spread put should have positive value, got {}",
            price
        );
    }

    #[test]
    fn test_margrabe_exchange_option() {
        let option = create_spread_option(Positive::ZERO, OptionStyle::Call);
        let price = spread_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Exchange option (K=0) should have positive value"
        );
    }

    #[test]
    fn test_kirk_approximation_nonzero_strike() {
        let option = create_spread_option(pos_or_panic!(5.0), OptionStyle::Call);
        let price = spread_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Kirk approximation should produce positive value"
        );
    }

    #[test]
    fn test_spread_correlation_impact() {
        let mut low_corr = create_spread_option(pos_or_panic!(5.0), OptionStyle::Call);
        if let Some(ref mut params) = low_corr.exotic_params {
            params.spread_correlation = Some(dec!(0.0));
        }

        let mut high_corr = create_spread_option(pos_or_panic!(5.0), OptionStyle::Call);
        if let Some(ref mut params) = high_corr.exotic_params {
            params.spread_correlation = Some(dec!(0.9));
        }

        let low_price = spread_black_scholes(&low_corr).unwrap();
        let high_price = spread_black_scholes(&high_corr).unwrap();

        assert!(
            low_price > high_price,
            "Lower correlation should give higher spread option value (more uncertainty in spread)"
        );
    }

    #[test]
    fn test_spread_invalid_correlation() {
        let mut option = create_spread_option(pos_or_panic!(5.0), OptionStyle::Call);
        if let Some(ref mut params) = option.exotic_params {
            params.spread_correlation = Some(dec!(1.5));
        }

        let result = spread_black_scholes(&option);
        assert!(result.is_err(), "Should reject correlation > 1");
    }

    #[test]
    fn test_spread_missing_params() {
        let option = Options::new(
            OptionType::Spread {
                second_asset: 100.0,
            },
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(5.0),
            ExpirationDate::Days(pos_or_panic!(90.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );

        let result = spread_black_scholes(&option);
        assert!(result.is_err(), "Should fail without exotic_params");
    }

    #[test]
    fn test_spread_short_position() {
        let mut option = create_spread_option(pos_or_panic!(5.0), OptionStyle::Call);
        option.side = Side::Short;

        let price = spread_black_scholes(&option).unwrap();
        assert!(
            price < dec!(0.0),
            "Short position should have negative value"
        );
    }

    #[test]
    fn test_spread_put_call_parity() {
        let call = create_spread_option(pos_or_panic!(5.0), OptionStyle::Call);
        let put = create_spread_option(pos_or_panic!(5.0), OptionStyle::Put);

        let call_price = spread_black_scholes(&call).unwrap();
        let put_price = spread_black_scholes(&put).unwrap();

        let s1 = Decimal::from(call.underlying_price);
        let s2 = dec!(100.0);
        let k = dec!(5.0);
        let r = call.risk_free_rate;
        let t = Decimal::from(call.expiration_date.get_years().unwrap());

        let forward_spread = s1 - s2;
        let k_pv = k * (-r * t).exp();

        let parity_diff = (call_price - put_price - forward_spread + k_pv).abs();

        assert!(
            parity_diff < dec!(2.0),
            "Put-call parity should approximately hold, diff = {}",
            parity_diff
        );
    }

    #[test]
    fn test_spread_deep_itm_call() {
        let mut option = create_spread_option(Positive::ZERO, OptionStyle::Call);
        option.underlying_price = pos_or_panic!(150.0);

        let price = spread_black_scholes(&option).unwrap();
        let intrinsic = dec!(150.0) - dec!(100.0);

        assert!(
            price >= intrinsic * dec!(0.9),
            "Deep ITM spread call should be close to intrinsic value"
        );
    }

    #[test]
    fn test_spread_deep_otm_call() {
        let mut option = create_spread_option(pos_or_panic!(50.0), OptionStyle::Call);
        option.underlying_price = pos_or_panic!(80.0);

        let price = spread_black_scholes(&option).unwrap();

        assert!(
            price < dec!(5.0),
            "Deep OTM spread call should have small value"
        );
    }

    #[test]
    fn test_spread_negative_correlation() {
        let mut option = create_spread_option(pos_or_panic!(5.0), OptionStyle::Call);
        if let Some(ref mut params) = option.exotic_params {
            params.spread_correlation = Some(dec!(-0.5));
        }

        let price = spread_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Spread option with negative correlation should have positive value"
        );
    }
}
