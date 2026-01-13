//! Power Option Pricing Module
//!
//! This module implements pricing for Power options, which have payoffs that are
//! a power function of the underlying asset price.
//!
//! # Payoff Structure
//!
//! - **Standard Power Call**: max(S^n - K, 0)
//! - **Standard Power Put**: max(K - S^n, 0)
//!
//! # Pricing Formula
//!
//! For a power option, S^n is log-normal with adjusted parameters:
//! - E[S^n] = S^n × e^(n×(r-q)×T + n×(n-1)×σ²×T/2)
//!
//! # Common Applications
//!
//! - Leveraged exposure to underlying asset
//! - Speculative trading on large price movements
//! - Structured products with non-linear payoffs

use crate::Options;
use crate::error::PricingError;
use crate::greeks::big_n;
use crate::model::types::{OptionStyle, OptionType, Side};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

/// Prices a Power option using the adjusted Black-Scholes formula.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Power`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// Returns an error if:
/// - The option type is not `Power`
/// - The exponent is less than or equal to 0
pub fn power_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    let exponent = match &option.option_type {
        OptionType::Power { exponent } => *exponent,
        _ => {
            return Err(PricingError::other(
                "power_black_scholes requires OptionType::Power",
            ));
        }
    };

    if exponent <= 0.0 {
        return Err(PricingError::other(
            "Power option exponent must be greater than 0",
        ));
    }

    let n = Decimal::from_f64(exponent)
        .ok_or_else(|| PricingError::other("Failed to convert exponent to Decimal"))?;

    let s = Decimal::from(option.underlying_price);
    let k = Decimal::from(option.strike_price);
    let r = option.risk_free_rate;
    let q = Decimal::from(option.dividend_yield);
    let sigma = Decimal::from(option.implied_volatility);
    let t = Decimal::from(option.expiration_date.get_years()?);

    let price = power_price(s, k, r, q, sigma, t, n, &option.option_style)?;

    Ok(apply_side(price, option))
}

/// Computes the power option price using adjusted Black-Scholes.
///
/// # Arguments
///
/// * `s` - Spot price of the underlying asset
/// * `k` - Strike price
/// * `r` - Risk-free interest rate
/// * `q` - Dividend yield
/// * `sigma` - Volatility of the underlying asset
/// * `t` - Time to expiration in years
/// * `n` - Power exponent
/// * `style` - Option style (Call or Put)
#[allow(clippy::too_many_arguments)]
fn power_price(
    s: Decimal,
    k: Decimal,
    r: Decimal,
    q: Decimal,
    sigma: Decimal,
    t: Decimal,
    n: Decimal,
    style: &OptionStyle,
) -> Result<Decimal, PricingError> {
    if t <= dec!(0.0) {
        let s_f64 = s
            .to_f64()
            .ok_or_else(|| PricingError::other("Failed to convert s"))?;
        let n_f64 = n
            .to_f64()
            .ok_or_else(|| PricingError::other("Failed to convert n"))?;
        let s_power = Decimal::from_f64(s_f64.powf(n_f64))
            .ok_or_else(|| PricingError::other("Failed to compute S^n"))?;

        return match style {
            OptionStyle::Call => Ok((s_power - k).max(dec!(0.0))),
            OptionStyle::Put => Ok((k - s_power).max(dec!(0.0))),
        };
    }

    let s_f64 = s
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert s"))?;
    let n_f64 = n
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert n"))?;
    let r_f64 = r
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert r"))?;
    let q_f64 = q
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert q"))?;
    let sigma_f64 = sigma
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert sigma"))?;
    let t_f64 = t
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert t"))?;
    let k_f64 = k
        .to_f64()
        .ok_or_else(|| PricingError::other("Failed to convert k"))?;

    let s_power = s_f64.powf(n_f64);

    let drift_adjustment =
        n_f64 * (r_f64 - q_f64) + n_f64 * (n_f64 - 1.0) * sigma_f64 * sigma_f64 / 2.0;
    let forward = s_power * (drift_adjustment * t_f64).exp();

    let sigma_adj = n_f64 * sigma_f64;

    let sqrt_t = t_f64.sqrt();
    let d1 = ((forward / k_f64).ln() + sigma_adj * sigma_adj * t_f64 / 2.0) / (sigma_adj * sqrt_t);
    let d2 = d1 - sigma_adj * sqrt_t;

    let discount = (-r_f64 * t_f64).exp();

    let d1_dec =
        Decimal::from_f64(d1).ok_or_else(|| PricingError::other("Failed to convert d1"))?;
    let d2_dec =
        Decimal::from_f64(d2).ok_or_else(|| PricingError::other("Failed to convert d2"))?;
    let forward_dec = Decimal::from_f64(forward)
        .ok_or_else(|| PricingError::other("Failed to convert forward"))?;
    let discount_dec = Decimal::from_f64(discount)
        .ok_or_else(|| PricingError::other("Failed to convert discount"))?;

    let price = match style {
        OptionStyle::Call => discount_dec * (forward_dec * big_n(d1_dec)? - k * big_n(d2_dec)?),
        OptionStyle::Put => discount_dec * (k * big_n(-d2_dec)? - forward_dec * big_n(-d1_dec)?),
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

    fn create_power_option(exponent: f64, option_style: OptionStyle) -> Options {
        Options::new(
            OptionType::Power { exponent },
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(100.0),
            ExpirationDate::Days(pos_or_panic!(90.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(10.0),
            dec!(0.05),
            option_style,
            pos_or_panic!(0.02),
            Some(ExoticParams::default()),
        )
    }

    #[test]
    fn test_power_call_squared() {
        let option = create_power_option(2.0, OptionStyle::Call);
        let price = power_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Power call (n=2) should have positive value, got {}",
            price
        );
    }

    #[test]
    fn test_power_put_squared() {
        let mut option = create_power_option(2.0, OptionStyle::Put);
        option.strike_price = pos_or_panic!(150.0);

        let price = power_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Power put (n=2) should have positive value, got {}",
            price
        );
    }

    #[test]
    fn test_power_exponent_one() {
        let mut option = create_power_option(1.0, OptionStyle::Call);
        option.underlying_price = pos_or_panic!(110.0);
        option.strike_price = pos_or_panic!(100.0);

        let price = power_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Power option with n=1 should have positive value"
        );
    }

    #[test]
    fn test_power_exponent_three() {
        let option = create_power_option(3.0, OptionStyle::Call);
        let price = power_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Power call (n=3) should have positive value"
        );
    }

    #[test]
    fn test_power_invalid_exponent_zero() {
        let option = create_power_option(0.0, OptionStyle::Call);
        let result = power_black_scholes(&option);
        assert!(result.is_err(), "Should reject exponent = 0");
    }

    #[test]
    fn test_power_invalid_exponent_negative() {
        let option = create_power_option(-1.0, OptionStyle::Call);
        let result = power_black_scholes(&option);
        assert!(result.is_err(), "Should reject negative exponent");
    }

    #[test]
    fn test_power_short_position() {
        let mut option = create_power_option(2.0, OptionStyle::Call);
        option.side = Side::Short;

        let price = power_black_scholes(&option).unwrap();
        assert!(
            price < dec!(0.0),
            "Short position should have negative value"
        );
    }

    #[test]
    fn test_power_deep_itm_call() {
        let mut option = create_power_option(2.0, OptionStyle::Call);
        option.underlying_price = pos_or_panic!(15.0);
        option.strike_price = pos_or_panic!(100.0);

        let price = power_black_scholes(&option).unwrap();
        let intrinsic = dec!(225.0) - dec!(100.0);

        assert!(
            price >= intrinsic * dec!(0.8),
            "Deep ITM power call should be close to intrinsic value"
        );
    }

    #[test]
    fn test_power_deep_otm_call() {
        let mut option = create_power_option(2.0, OptionStyle::Call);
        option.underlying_price = pos_or_panic!(5.0);
        option.strike_price = pos_or_panic!(100.0);

        let price = power_black_scholes(&option).unwrap();

        assert!(
            price < dec!(10.0),
            "Deep OTM power call should have small value"
        );
    }

    #[test]
    fn test_power_higher_exponent_higher_value() {
        let option_n2 = create_power_option(2.0, OptionStyle::Call);
        let option_n3 = create_power_option(3.0, OptionStyle::Call);

        let price_n2 = power_black_scholes(&option_n2).unwrap();
        let price_n3 = power_black_scholes(&option_n3).unwrap();

        assert!(
            price_n3 > price_n2,
            "Higher exponent should give higher option value for ITM options"
        );
    }

    #[test]
    fn test_power_fractional_exponent() {
        let mut option = create_power_option(0.5, OptionStyle::Call);
        option.underlying_price = pos_or_panic!(100.0);
        option.strike_price = pos_or_panic!(5.0);

        let price = power_black_scholes(&option).unwrap();
        assert!(
            price > dec!(0.0),
            "Power option with fractional exponent should have positive value"
        );
    }
}
