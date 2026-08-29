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
use crate::model::decimal::{d_add, d_div, d_exp, d_ln, d_mul, d_sqrt, d_sub};
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
/// - [`PricingError::MethodError`] when the option type is not `Spread`, when
///   the required exotic parameters are missing, when the correlation is
///   outside `[-1, 1]`, when the Kirk adjusted strike `S2 + K` is
///   non-positive, or when the combined variance is negative.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range: the adjusted strike, the combined
///   variance, the present values, `d1` / `d2`, or the final legs.
/// - `PricingError::ExpirationDate` when the expiration cannot be converted.
pub fn spread_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    let second_asset_price = match &option.option_type {
        OptionType::Spread { second_asset } => second_asset.to_dec(),
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
        let spread = d_sub(s1, s2, "pricing::spread::kirk::intrinsic::spread")?;
        return match style {
            OptionStyle::Call => {
                Ok(d_sub(spread, k, "pricing::spread::kirk::intrinsic::call")?.max(dec!(0.0)))
            }
            OptionStyle::Put => {
                Ok(d_sub(k, spread, "pricing::spread::kirk::intrinsic::put")?.max(dec!(0.0)))
            }
        };
    }

    let adjusted_strike = d_add(s2, k, "pricing::spread::kirk::adjusted_strike")?;
    if adjusted_strike <= dec!(0.0) {
        return Err(PricingError::other(
            "Adjusted strike (S2 + K) must be positive",
        ));
    }

    let s2_ratio = d_div(s2, adjusted_strike, "pricing::spread::kirk::s2_ratio")?;

    let sigma_sq = d_sub(
        d_add(
            d_mul(sigma1, sigma1, "pricing::spread::kirk::var1")?,
            d_mul(
                d_mul(s2_ratio, s2_ratio, "pricing::spread::kirk::s2_ratio_sq")?,
                d_mul(sigma2, sigma2, "pricing::spread::kirk::var2")?,
                "pricing::spread::kirk::weighted_var2",
            )?,
            "pricing::spread::kirk::variance_sum",
        )?,
        d_mul(
            d_mul(
                d_mul(
                    d_mul(dec!(2.0), rho, "pricing::spread::kirk::two_rho")?,
                    sigma1,
                    "pricing::spread::kirk::two_rho_sigma1",
                )?,
                sigma2,
                "pricing::spread::kirk::two_rho_sigma1_sigma2",
            )?,
            s2_ratio,
            "pricing::spread::kirk::covariance",
        )?,
        "pricing::spread::kirk::sigma_sq",
    )?;

    let sigma = sigma_sq
        .sqrt()
        .ok_or_else(|| PricingError::other("Failed to compute adjusted volatility"))?;

    let sqrt_t = d_sqrt(t, "pricing::spread::kirk::sqrt_t")?;
    let denominator = d_mul(sigma, sqrt_t, "pricing::spread::kirk::denominator")?;

    let s1_pv = d_mul(
        s1,
        d_exp(
            d_mul(-q1, t, "pricing::spread::kirk::neg_q1t")?,
            "pricing::spread::kirk::dividend_discount",
        )?,
        "pricing::spread::kirk::s1_pv",
    )?;
    let adjusted_strike_pv = d_mul(
        adjusted_strike,
        d_exp(
            d_mul(-r, t, "pricing::spread::kirk::neg_rt")?,
            "pricing::spread::kirk::discount",
        )?,
        "pricing::spread::kirk::adjusted_strike_pv",
    )?;

    let ratio = d_div(s1, adjusted_strike, "pricing::spread::kirk::moneyness")?;

    // `N(d1)`, `N(d2)`, `N(-d1)`, `N(-d2)`. A collapsed `σ√T` or a moneyness
    // that underflowed below the representable scale drives the normal
    // arguments to `±∞`, where the CDFs saturate: those are the limits of the
    // formula, not substitutes for it.
    let (n_d1, n_d2, n_neg_d1, n_neg_d2) = if denominator.is_zero() {
        // σ√T → 0: the option is worth its discounted intrinsic, i.e. the step
        // function at the forward. The test has to compare the two present
        // values, not the spot moneyness `S1 / (S2 + K)`: the carry `(r - q1)T`
        // lives in the discounting, and dropping it flips the step whenever
        // spot and forward straddle the adjusted strike. `sigma` is exactly
        // zero at `rho = 1, sigma1 = w * sigma2`, so this branch is reachable
        // from well-formed inputs.
        if s1_pv >= adjusted_strike_pv {
            (dec!(1.0), dec!(1.0), dec!(0.0), dec!(0.0))
        } else {
            (dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0))
        }
    } else if ratio.is_zero() {
        (dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0))
    } else {
        let d1 = d_div(
            d_add(
                d_ln(ratio, "pricing::spread::kirk::log_moneyness")?,
                d_mul(
                    d_add(
                        d_sub(r, q1, "pricing::spread::kirk::carry")?,
                        d_div(
                            d_mul(sigma, sigma, "pricing::spread::kirk::variance")?,
                            dec!(2.0),
                            "pricing::spread::kirk::half_variance",
                        )?,
                        "pricing::spread::kirk::drift_rate",
                    )?,
                    t,
                    "pricing::spread::kirk::drift",
                )?,
                "pricing::spread::kirk::d1_numerator",
            )?,
            denominator,
            "pricing::spread::kirk::d1",
        )?;
        let d2 = d_sub(d1, denominator, "pricing::spread::kirk::d2")?;
        (big_n(d1)?, big_n(d2)?, big_n(-d1)?, big_n(-d2)?)
    };

    match style {
        OptionStyle::Call => {
            let call = d_sub(
                d_mul(s1_pv, n_d1, "pricing::spread::kirk::call::spot")?,
                d_mul(
                    adjusted_strike_pv,
                    n_d2,
                    "pricing::spread::kirk::call::strike",
                )?,
                "pricing::spread::kirk::call",
            )?;
            Ok(call.max(dec!(0.0)))
        }
        OptionStyle::Put => {
            let put = d_sub(
                d_mul(
                    adjusted_strike_pv,
                    n_neg_d2,
                    "pricing::spread::kirk::put::strike",
                )?,
                d_mul(s1_pv, n_neg_d1, "pricing::spread::kirk::put::spot")?,
                "pricing::spread::kirk::put",
            )?;
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
        return Ok(d_sub(s1, s2, "pricing::spread::margrabe::intrinsic")?.max(dec!(0.0)));
    }

    let sigma_sq = d_sub(
        d_add(
            d_mul(sigma1, sigma1, "pricing::spread::margrabe::var1")?,
            d_mul(sigma2, sigma2, "pricing::spread::margrabe::var2")?,
            "pricing::spread::margrabe::variance_sum",
        )?,
        d_mul(
            d_mul(
                d_mul(dec!(2.0), rho, "pricing::spread::margrabe::two_rho")?,
                sigma1,
                "pricing::spread::margrabe::two_rho_sigma1",
            )?,
            sigma2,
            "pricing::spread::margrabe::covariance",
        )?,
        "pricing::spread::margrabe::sigma_sq",
    )?;

    let sigma = sigma_sq
        .sqrt()
        .ok_or_else(|| PricingError::other("Failed to compute combined volatility"))?;

    let sqrt_t = d_sqrt(t, "pricing::spread::margrabe::sqrt_t")?;
    let denominator = d_mul(sigma, sqrt_t, "pricing::spread::margrabe::denominator")?;

    let s1_pv = d_mul(
        s1,
        d_exp(
            d_mul(-q1, t, "pricing::spread::margrabe::neg_q1t")?,
            "pricing::spread::margrabe::discount1",
        )?,
        "pricing::spread::margrabe::s1_pv",
    )?;
    let s2_pv = d_mul(
        s2,
        d_exp(
            d_mul(-q2, t, "pricing::spread::margrabe::neg_q2t")?,
            "pricing::spread::margrabe::discount2",
        )?,
        "pricing::spread::margrabe::s2_pv",
    )?;

    // Zero combined volatility, or a `σ√T` that underflowed below the
    // representable scale: the exchange ratio is deterministic and the option
    // is worth the difference of the two present values.
    if denominator.is_zero() {
        return Ok(d_sub(s1_pv, s2_pv, "pricing::spread::margrabe::deterministic")?.max(dec!(0.0)));
    }

    // `S2 = 0`: `N(d1) = N(d2) = 1` and the option collapses to `S1`'s
    // present value; a ratio that rounds to zero is the mirror limit.
    if s2.is_zero() {
        return Ok(s1_pv.max(dec!(0.0)));
    }
    let ratio = d_div(s1, s2, "pricing::spread::margrabe::ratio")?;
    if ratio.is_zero() {
        return Ok(dec!(0.0));
    }

    let d1 = d_div(
        d_add(
            d_ln(ratio, "pricing::spread::margrabe::log_ratio")?,
            d_mul(
                d_add(
                    d_sub(q2, q1, "pricing::spread::margrabe::carry")?,
                    d_div(
                        d_mul(sigma, sigma, "pricing::spread::margrabe::variance")?,
                        dec!(2.0),
                        "pricing::spread::margrabe::half_variance",
                    )?,
                    "pricing::spread::margrabe::drift_rate",
                )?,
                t,
                "pricing::spread::margrabe::drift",
            )?,
            "pricing::spread::margrabe::d1_numerator",
        )?,
        denominator,
        "pricing::spread::margrabe::d1",
    )?;
    let d2 = d_sub(d1, denominator, "pricing::spread::margrabe::d2")?;

    let price = d_sub(
        d_mul(s1_pv, big_n(d1)?, "pricing::spread::margrabe::leg1")?,
        d_mul(s2_pv, big_n(d2)?, "pricing::spread::margrabe::leg2")?,
        "pricing::spread::margrabe::price",
    )?;

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
                second_asset: pos_or_panic!(100.0),
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
                second_asset: pos_or_panic!(100.0),
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

#[cfg(test)]
mod tests_kirk_zero_volatility {
    use super::*;
    use crate::ExpirationDate;
    use crate::model::option::ExoticParams;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    /// Kirk's adjusted volatility is
    /// `sigma^2 = (sigma1 - w*sigma2)^2 + 2*w*sigma1*sigma2*(1 - rho)` with
    /// `w = S2 / (S2 + K)`, which is exactly zero at `rho = 1` and
    /// `sigma1 = w * sigma2`. These are well-formed inputs, so the zero-vol
    /// branch is reachable and its step test has to be right.
    ///
    /// `S2 = 100, K = 25` gives `w = 0.8` exactly, so `sigma1 = 0.2` against
    /// `sigma2 = 0.25` collapses the adjusted volatility to exactly zero — the
    /// weight has to divide exactly or the branch is never reached.
    ///
    /// Spot moneyness `S1 / (S2 + K) = 0.8` says out-of-the-money, but with
    /// `r = 25%` and `q1 = 0` the present values say the opposite:
    /// `S1*e^{-q1*T} = 100` against `(S2 + K)*e^{-r*T} = 125*e^{-0.25} = 97.35`.
    fn zero_vol_spread(option_style: OptionStyle, risk_free_rate: Decimal) -> Options {
        Options::new(
            OptionType::Spread {
                second_asset: Positive::HUNDRED,
            },
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(25.0),
            ExpirationDate::Days(pos_or_panic!(365.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            risk_free_rate,
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
                spread_correlation: Some(Decimal::ONE),
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
    fn test_zero_volatility_call_uses_present_values_not_spot_moneyness() {
        let option = zero_vol_spread(OptionStyle::Call, dec!(0.25));
        let price = match spread_black_scholes(&option) {
            Ok(price) => price,
            Err(e) => panic!("the spread should price: {e}"),
        };

        // The discounted intrinsic is `S1*e^{-q1*T} - (S2 + K)*e^{-r*T}`,
        // about 2.65. Testing the spot moneyness instead returns zero here.
        assert!(
            price > dec!(2.5) && price < dec!(2.8),
            "zero-vol call priced at {price}, expected the discounted intrinsic near 2.65"
        );
    }

    #[test]
    fn test_zero_volatility_put_is_worthless_when_the_call_is_in_the_money() {
        let option = zero_vol_spread(OptionStyle::Put, dec!(0.25));
        let price = match spread_black_scholes(&option) {
            Ok(price) => price,
            Err(e) => panic!("the spread should price: {e}"),
        };
        assert_eq!(price, Decimal::ZERO, "put priced at {price}");
    }

    #[test]
    fn test_zero_volatility_step_flips_with_the_carry() {
        // A negative rate pushes the adjusted strike's present value above the
        // first asset's, so the same contract flips: the call is worthless and
        // the put carries the intrinsic.
        let call = zero_vol_spread(OptionStyle::Call, dec!(-0.10));
        let put = zero_vol_spread(OptionStyle::Put, dec!(-0.10));

        let call_price = match spread_black_scholes(&call) {
            Ok(price) => price,
            Err(e) => panic!("the call should price: {e}"),
        };
        let put_price = match spread_black_scholes(&put) {
            Ok(price) => price,
            Err(e) => panic!("the put should price: {e}"),
        };

        assert_eq!(call_price, Decimal::ZERO, "call priced at {call_price}");
        assert!(
            put_price > dec!(38.0),
            "put priced at {put_price}, expected the discounted intrinsic near 38.1"
        );
    }
}
