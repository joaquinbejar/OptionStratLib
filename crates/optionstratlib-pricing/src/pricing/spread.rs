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

use crate::error::PricingError;
use crate::kernels::{big_n, discount_factor};
use optionstratlib_core::model::Options;
use optionstratlib_core::model::decimal::{d_add, d_div, d_ln, d_mul, d_sqrt, d_sub};
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use rust_decimal::Decimal;
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
/// `ExoticParams::spread_second_asset_dividend` defaults to zero when unset.
///
/// # Errors
///
/// - [`PricingError::MethodError`] when the option type is not `Spread`, when
///   the required exotic parameters are missing, when the correlation is
///   outside `[-1, 1]`, when the Kirk adjusted strike `S2 + K` is
///   non-positive, or when the combined variance is negative.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range: the adjusted strike, the combined
///   variance, the present values, either logarithm of the log-moneyness,
///   `d1` / `d2`, or the final legs.
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

    // An unset second dividend yield is zero: a parameter default, not a
    // fallback on a failed step (#639).
    let q2 = params
        .spread_second_asset_dividend
        .unwrap_or(optionstratlib_core::model::Positive::ZERO);

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
/// Treats the spread option as a call on `S1` struck at the second asset's
/// forward plus the strike (Haug, *The Complete Guide to Option Pricing
/// Formulas*, §5.4.2), written on present values so no forward is formed:
///
/// ```text
/// P1 = S1 e^(-q1 T)                    (e^(-rT) F1)
/// P2 = S2 e^(-q2 T) + K e^(-rT)        (e^(-rT) (F2 + K))
/// w  = S2 e^(-q2 T) / P2               (F2 / (F2 + K))
/// σ² = σ1² + (w σ2)² - 2 ρ σ1 σ2 w
/// d1 = (ln(P1 / P2) + σ² T / 2) / (σ √T),   d2 = d1 - σ √T
/// c  = P1 N(d1) - P2 N(d2),   p = P2 N(-d2) - P1 N(-d1)
/// ```
///
/// With `K = 0` this is Margrabe's formula, so the two branches of
/// [`spread_black_scholes`] meet as the strike vanishes. Each asset carries
/// its own dividend yield; the second asset used to be discounted at `r`
/// with `q2` ignored, which is right only when `q2 = r` (#650).
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
    q2: Decimal,
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

    if d_add(s2, k, "pricing::spread::kirk::adjusted_strike")? <= dec!(0.0) {
        return Err(PricingError::other(
            "Adjusted strike (S2 + K) must be positive",
        ));
    }

    let s1_pv = d_mul(
        s1,
        discount_factor(
            q1,
            t,
            "pricing::spread::kirk::neg_q1t",
            "pricing::spread::kirk::dividend_discount",
        )?,
        "pricing::spread::kirk::s1_pv",
    )?;
    let s2_pv = d_mul(
        s2,
        discount_factor(
            q2,
            t,
            "pricing::spread::kirk::neg_q2t",
            "pricing::spread::kirk::dividend_discount2",
        )?,
        "pricing::spread::kirk::s2_pv",
    )?;
    let strike_pv = d_mul(
        k,
        discount_factor(
            r,
            t,
            "pricing::spread::kirk::neg_rt",
            "pricing::spread::kirk::discount",
        )?,
        "pricing::spread::kirk::strike_pv",
    )?;
    let adjusted_strike_pv = d_add(
        s2_pv,
        strike_pv,
        "pricing::spread::kirk::adjusted_strike_pv",
    )?;
    if adjusted_strike_pv.is_zero() {
        // `S2 + K > 0` but both discount factors flushed below the
        // representable scale: `ln(P1 / P2)` diverges to `+∞`, the CDFs
        // saturate, and the call is worth `S1`'s present value. That is the
        // limit of the formula, not a substitute for it.
        return Ok(match style {
            OptionStyle::Call => s1_pv,
            OptionStyle::Put => dec!(0.0),
        });
    }

    let s2_ratio = d_div(s2_pv, adjusted_strike_pv, "pricing::spread::kirk::s2_ratio")?;

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

    let sigma = d_sqrt(sigma_sq, "pricing::spread::kirk::sigma")
        .map_err(|_| PricingError::other("Failed to compute adjusted volatility"))?;

    let sqrt_t = d_sqrt(t, "pricing::spread::kirk::sqrt_t")?;
    let denominator = d_mul(sigma, sqrt_t, "pricing::spread::kirk::denominator")?;

    // `N(d1)`, `N(d2)`, `N(-d1)`, `N(-d2)`. A collapsed `σ√T` or a vanished
    // `S1` drives the normal arguments to `±∞`, where the CDFs saturate:
    // those are the limits of the formula, not substitutes for it.
    let (n_d1, n_d2, n_neg_d1, n_neg_d2) = if denominator.is_zero() {
        // σ√T → 0: the option is worth its discounted intrinsic, i.e. the step
        // function at the forwards, which compares the two present values.
        // `sigma` is exactly zero at `rho = 1, sigma1 = w * sigma2`, so this
        // branch is reachable from well-formed inputs.
        if s1_pv >= adjusted_strike_pv {
            (dec!(1.0), dec!(1.0), dec!(0.0), dec!(0.0))
        } else {
            (dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0))
        }
    } else if s1_pv.is_zero() {
        // `S1 e^(-q1 T) = 0` (a zero spot, or a discount factor flushed to
        // zero): `ln(P1 / P2)` diverges to `-∞` and the CDFs saturate.
        (dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0))
    } else {
        // Log-moneyness as `ln(P1) - ln(P2)`, never as `ln(P1 / P2)`: the
        // quotient underflows to zero below `1e-28`, so a tiny-but-nonzero
        // `S1` against a large adjusted strike would price at zero instead of
        // at `S1`'s present value. The difference of the two logarithms is
        // exact where the quotient is not, and both exist: `P2` was rejected
        // above unless positive and `P1 = 0` is the branch just above.
        let log_moneyness = d_sub(
            d_ln(s1_pv, "pricing::spread::kirk::log_s1_pv")?,
            d_ln(
                adjusted_strike_pv,
                "pricing::spread::kirk::log_adjusted_strike_pv",
            )?,
            "pricing::spread::kirk::log_moneyness",
        )?;
        let d1 = d_div(
            d_add(
                log_moneyness,
                d_mul(
                    d_div(
                        d_mul(sigma, sigma, "pricing::spread::kirk::variance")?,
                        dec!(2.0),
                        "pricing::spread::kirk::half_variance",
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

    let sigma = d_sqrt(sigma_sq, "pricing::spread::margrabe::sigma")
        .map_err(|_| PricingError::other("Failed to compute combined volatility"))?;

    let sqrt_t = d_sqrt(t, "pricing::spread::margrabe::sqrt_t")?;
    let denominator = d_mul(sigma, sqrt_t, "pricing::spread::margrabe::denominator")?;

    let s1_pv = d_mul(
        s1,
        discount_factor(
            q1,
            t,
            "pricing::spread::margrabe::neg_q1t",
            "pricing::spread::margrabe::discount1",
        )?,
        "pricing::spread::margrabe::s1_pv",
    )?;
    let s2_pv = d_mul(
        s2,
        discount_factor(
            q2,
            t,
            "pricing::spread::margrabe::neg_q2t",
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
    // present value.
    if s2.is_zero() {
        return Ok(s1_pv.max(dec!(0.0)));
    }
    // `S1 = 0`: the payoff `max(S1 - S2, 0)` is identically zero.
    if s1.is_zero() {
        return Ok(dec!(0.0));
    }

    // Log-moneyness as `ln(S1) - ln(S2)`, never as `ln(S1 / S2)`: the quotient
    // underflows to zero below `1e-28` and drags the `(q2 - q1)T` carry down
    // with it, so a tiny-but-nonzero `S1` against a `q2` large enough to wipe
    // out `S2`'s present value would price at zero instead of at `S1`'s present
    // value. Both logarithms exist: the two spot branches above are the only
    // non-positive inputs `d_ln` could see.
    let log_ratio = d_sub(
        d_ln(s1, "pricing::spread::margrabe::log_s1")?,
        d_ln(s2, "pricing::spread::margrabe::log_s2")?,
        "pricing::spread::margrabe::log_ratio",
    )?;

    let d1 = d_div(
        d_add(
            log_ratio,
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
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::option::ExoticParams;
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal::MathematicalOps;
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
                foreign_rate: None,
                exchange_second_asset_volatility: None,
                exchange_second_asset_dividend: None,
                exchange_correlation: None,
            }),
        )
    }

    /// Margrabe leg (`K = 0`). `S1 / S2` rounds below the representable
    /// `Decimal` scale, but a `q2` large enough to wipe out `S2`'s present
    /// value offsets the log-moneyness through the `(q2 - q1)T` carry, so the
    /// option is worth `S1`'s present value.
    #[test]
    fn test_margrabe_underflowing_ratio_with_offsetting_carry_prices_first_pv() {
        let mut option = create_spread_option(Positive::ZERO, OptionStyle::Call);
        option.underlying_price = Positive::new_decimal(Decimal::new(1, 27)).unwrap();
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(365.0));
        option.dividend_yield = Positive::ZERO;
        if let Some(ref mut params) = option.exotic_params {
            params.spread_second_asset_volatility = Some(pos_or_panic!(0.2));
            params.spread_second_asset_dividend = Some(pos_or_panic!(100.0));
        }

        let price = spread_black_scholes(&option).unwrap();

        assert_eq!(
            price,
            Decimal::new(1, 27),
            "vanished S2 present value should leave S1's present value, got {}",
            price
        );
    }

    /// Margrabe leg, mirror case: without the carry the same underflowing
    /// ratio really does drive both CDFs to zero.
    #[test]
    fn test_margrabe_underflowing_ratio_without_carry_is_zero() {
        let mut option = create_spread_option(Positive::ZERO, OptionStyle::Call);
        option.underlying_price = Positive::new_decimal(Decimal::new(1, 27)).unwrap();
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(365.0));
        option.dividend_yield = Positive::ZERO;
        if let Some(ref mut params) = option.exotic_params {
            params.spread_second_asset_volatility = Some(pos_or_panic!(0.2));
        }

        let price = spread_black_scholes(&option).unwrap();

        assert_eq!(
            price,
            Decimal::ZERO,
            "a ratio that vanishes in the limit should still price at zero, got {}",
            price
        );
    }

    /// Kirk leg (`K != 0`). `S1 / (S2 + K)` rounds below the representable
    /// `Decimal` scale, but an `r` and a `q2` large enough to wipe out the
    /// adjusted strike's present value `S2 e^(-q2 T) + K e^(-rT)` offset the
    /// log-moneyness, so the call is worth `S1`'s present value. Since #650
    /// `S2` is discounted at its own `q2`, not at `r`, so the test sets both.
    #[test]
    fn test_kirk_underflowing_moneyness_with_offsetting_carry_prices_first_pv() {
        let mut option = create_spread_option(pos_or_panic!(50.0), OptionStyle::Call);
        option.option_type = OptionType::Spread {
            second_asset: pos_or_panic!(50.0),
        };
        option.underlying_price = Positive::new_decimal(Decimal::new(1, 27)).unwrap();
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(365.0));
        option.risk_free_rate = dec!(100.0);
        option.dividend_yield = Positive::ZERO;
        if let Some(ref mut params) = option.exotic_params {
            params.spread_second_asset_volatility = Some(pos_or_panic!(0.2));
            params.spread_second_asset_dividend = Some(pos_or_panic!(100.0));
        }

        let price = spread_black_scholes(&option).unwrap();

        assert_eq!(
            price,
            Decimal::new(1, 27),
            "vanished adjusted-strike present value should leave S1's present value, got {}",
            price
        );
    }

    /// Kirk leg, mirror case: at `r = 0` the same underflowing moneyness is a
    /// genuine limit, so the call is worthless and the put is the full
    /// adjusted strike.
    #[test]
    fn test_kirk_underflowing_moneyness_without_carry_is_zero() {
        let mut option = create_spread_option(pos_or_panic!(50.0), OptionStyle::Call);
        option.option_type = OptionType::Spread {
            second_asset: pos_or_panic!(50.0),
        };
        option.underlying_price = Positive::new_decimal(Decimal::new(1, 27)).unwrap();
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(365.0));
        option.risk_free_rate = dec!(0.0);
        option.dividend_yield = Positive::ZERO;
        if let Some(ref mut params) = option.exotic_params {
            params.spread_second_asset_volatility = Some(pos_or_panic!(0.2));
        }

        let call = spread_black_scholes(&option).unwrap();
        assert_eq!(
            call,
            Decimal::ZERO,
            "a moneyness that vanishes in the limit should still price at zero, got {}",
            call
        );

        option.option_style = OptionStyle::Put;
        let put = spread_black_scholes(&option).unwrap();
        optionstratlib_core::assert_decimal_eq!(put, dec!(100.0), dec!(1e-20));
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

        // `q1 = q2 = 0`: `C - P = S1 - S2 - K e^(-rT)` holds exactly for Kirk
        // on present values (#650). Discounting `S2` at `r` used to leave a
        // gap of `S2 (1 - e^(-rT)) ≈ 1.23`, which the old `2.0` bound hid.
        assert!(
            parity_diff < dec!(0.000000001),
            "Put-call parity should hold, diff = {}",
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
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::option::ExoticParams;
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    /// Kirk's adjusted volatility is
    /// `sigma^2 = (sigma1 - w*sigma2)^2 + 2*w*sigma1*sigma2*(1 - rho)` with
    /// `w = S2 e^(-q2 T) / (S2 e^(-q2 T) + K e^(-rT))`, which is exactly zero
    /// at `rho = 1` and `sigma1 = w * sigma2`. These are well-formed inputs,
    /// so the zero-vol branch is reachable and its step test has to be right.
    ///
    /// `S2 = 100, K = 25, r = q2 = 0` gives `w = 0.8` exactly, so
    /// `sigma1 = 0.2` against `sigma2 = 0.25` collapses the adjusted
    /// volatility to exactly zero — the weight has to divide exactly or the
    /// branch is never reached. Since #650 `w` uses the present values, so
    /// the rate has to be zero for the weight to stay exact; the carry that
    /// separates spot from present-value moneyness comes from `q1` instead
    /// (these tests used `r = 25 %` and `r = -10 %` before).
    ///
    /// Spot moneyness `S1 - (S2 + K) = 130 - 125` says in-the-money, but with
    /// `q1 = 10 %` the present values say the opposite:
    /// `S1 e^(-q1 T) = 130 e^(-0.1) = 117.63` against `S2 + K = 125`.
    fn zero_vol_spread(option_style: OptionStyle, dividend_yield: Positive) -> Options {
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
            pos_or_panic!(130.0),
            Decimal::ZERO,
            option_style,
            dividend_yield,
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
                foreign_rate: None,
                exchange_second_asset_volatility: None,
                exchange_second_asset_dividend: None,
                exchange_correlation: None,
            }),
        )
    }

    fn price(option: &Options) -> Decimal {
        match spread_black_scholes(option) {
            Ok(price) => price,
            Err(e) => panic!("the spread should price: {e}"),
        }
    }

    #[test]
    fn test_zero_volatility_call_uses_present_values_not_spot_moneyness() {
        // The discounted intrinsic is `max(S1 e^(-q1 T) - (S2 + K), 0) = 0`.
        // Testing the spot moneyness instead returns `130 - 125 = 5`.
        let call = price(&zero_vol_spread(OptionStyle::Call, pos_or_panic!(0.10)));
        assert_eq!(call, Decimal::ZERO, "zero-vol call priced at {call}");
        let put = price(&zero_vol_spread(OptionStyle::Put, pos_or_panic!(0.10)));
        // `125 - 130 e^(-0.1) = 7.3709`.
        assert!(
            put > dec!(7.37) && put < dec!(7.38),
            "zero-vol put priced at {put}, expected the discounted intrinsic near 7.371"
        );
    }

    #[test]
    fn test_zero_volatility_put_is_worthless_when_the_call_is_in_the_money() {
        let put = price(&zero_vol_spread(OptionStyle::Put, Positive::ZERO));
        assert_eq!(put, Decimal::ZERO, "put priced at {put}");
        let call = price(&zero_vol_spread(OptionStyle::Call, Positive::ZERO));
        assert_eq!(call, dec!(5), "call priced at {call}");
    }

    #[test]
    fn test_zero_volatility_step_flips_with_the_carry() {
        // A dividend yield on the first asset pushes its present value below
        // the adjusted strike's, so the same contract flips: the call is
        // worthless and the put carries the intrinsic.
        let call = price(&zero_vol_spread(OptionStyle::Call, Positive::ZERO));
        let put = price(&zero_vol_spread(OptionStyle::Put, Positive::ZERO));
        assert!(call > Decimal::ZERO && put.is_zero(), "{call} / {put}");
        let call = price(&zero_vol_spread(OptionStyle::Call, pos_or_panic!(0.10)));
        let put = price(&zero_vol_spread(OptionStyle::Put, pos_or_panic!(0.10)));
        assert!(call.is_zero() && put > Decimal::ZERO, "{call} / {put}");
    }
}
