/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 13/01/26
******************************************************************************/

//! Chooser option pricing module.
//!
//! Chooser options (also called as-you-like-it options) allow the holder to
//! choose at a specified date whether the option becomes a call or a put.
//!
//! # Simple Chooser (Rubinstein 1991)
//!
//! At the choice date t, the holder chooses max(Call, Put). Both branches
//! share one strike K and one expiry T, which is what makes the chooser
//! *simple*. A complex chooser carries a separate strike and expiry per
//! branch, needs a bivariate normal and a root solve for the indifference
//! spot, and is not implemented here.
//!
//! The value is:
//!
//! `V = S*e^(-qT)*N(d1) - K*e^(-rT)*N(d2) + K*e^(-rT)*N(-y2) - S*e^(-qT)*N(-y1)`
//!
//! Where:
//! - T = time to final expiration
//! - t = time to choice date
//! - d1, d2 are standard BS d-values for T
//! - y1 = [ln(S/K) + b*T + (σ²/2)*t] / (σ√t)
//! - y2 = y1 - σ√t
//!
//! Two details are easy to get wrong, and both follow from the same
//! decomposition. Put-call parity at the choice date gives
//!
//! `max(C_t, P_t) = C_t + max(0, K*e^(-r(T-t)) - S_t*e^(-q(T-t)))`
//!
//! so a chooser is a T-expiry call plus `e^(-q(T-t))` puts expiring at t,
//! struck at the discounted forward `K*e^(-b(T-t))`. The d-values of that
//! embedded put are exactly y1 and y2. Hence:
//!
//! - the `y` legs discount over the **full** life T, not over t: the
//!   embedded put's own `e^(-rt)` discount combines with the strike
//!   adjustment `e^(-b(T-t))` back into `e^(-rT)`;
//! - the `y` drift mixes horizons — the carry runs to T, the variance only
//!   to t — because the choice is made at t but settles at T.
//!
//! Reference: Haug, *The Complete Guide to Option Pricing Formulas*, 2nd ed.,
//! §2.5.1 "Simple Chooser Options"; original result Rubinstein (1991),
//! "Options for the Undecided". Haug's worked example (S = K = 50, t = 0.25,
//! T = 0.5, r = b = 0.08, σ = 0.25) prices at 6.1071.

use crate::Options;
use crate::error::PricingError;
use crate::greeks::{big_n, d1, d2};
use crate::model::decimal::{d_add, d_div, d_exp, d_ln, d_mul, d_sqrt, d_sub};
use crate::model::types::OptionType;
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Prices a Chooser option using Rubinstein (1991) simple chooser formula.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Chooser`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// - [`PricingError::MethodError`] when `option` is not an
///   [`OptionType::Chooser`] variant, when the expiration cannot be converted
///   to a year fraction, or when the `d1` / `d2` kernels reject the inputs.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range: the discount factors, the zero-volatility
///   forward, `y1` / `y2`, or the four price legs.
pub fn chooser_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Chooser { choice_date } => simple_chooser_price(option, choice_date.to_f64()),
        _ => Err(PricingError::other(
            "chooser_black_scholes requires OptionType::Chooser",
        )),
    }
}

/// Prices a simple chooser option.
///
/// At the choice date, the holder chooses the maximum of call or put value.
/// Uses Rubinstein (1991) closed-form solution.
fn simple_chooser_price(option: &Options, choice_date_days: f64) -> Result<Decimal, PricingError> {
    let s = option.underlying_price;
    let k = option.strike_price;
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let t_big = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    // Convert choice_date from days to years
    let t_choice = Positive::new(choice_date_days / 365.0).unwrap_or(Positive::ZERO);

    // Validation: choice date must be before expiration
    if t_choice >= t_big {
        // If choice at or after expiration, it's just max(call, put) = straddle-like
        return price_at_choice_equals_expiry(option);
    }

    if t_big == Positive::ZERO {
        // At expiration, intrinsic value
        let call_intrinsic =
            d_sub(s.to_dec(), k.to_dec(), "pricing::chooser::intrinsic::call")?.max(Decimal::ZERO);
        let put_intrinsic =
            d_sub(k.to_dec(), s.to_dec(), "pricing::chooser::intrinsic::put")?.max(Decimal::ZERO);
        return Ok(apply_side(call_intrinsic.max(put_intrinsic), option));
    }

    if sigma == Positive::ZERO {
        // Zero vol: deterministic choice
        let discount_t = d_exp(
            d_mul(-r, t_big.to_dec(), "pricing::chooser::zero_vol::neg_rt")?,
            "pricing::chooser::zero_vol::discount",
        )?;
        let forward = d_mul(
            s.to_dec(),
            d_exp(
                d_mul(
                    d_sub(r, q, "pricing::chooser::zero_vol::carry")?,
                    t_big.to_dec(),
                    "pricing::chooser::zero_vol::carry_t",
                )?,
                "pricing::chooser::zero_vol::growth",
            )?,
            "pricing::chooser::zero_vol::forward",
        )?;
        let call_intrinsic = d_sub(
            forward,
            k.to_dec(),
            "pricing::chooser::zero_vol::call::intrinsic",
        )?
        .max(Decimal::ZERO);
        let put_intrinsic = d_sub(
            k.to_dec(),
            forward,
            "pricing::chooser::zero_vol::put::intrinsic",
        )?
        .max(Decimal::ZERO);
        let call_val = d_mul(
            call_intrinsic,
            discount_t,
            "pricing::chooser::zero_vol::call::discounted",
        )?;
        let put_val = d_mul(
            put_intrinsic,
            discount_t,
            "pricing::chooser::zero_vol::put::discounted",
        )?;
        return Ok(apply_side(call_val.max(put_val), option));
    }

    let b = d_sub(r, q, "pricing::chooser::carry")?;
    let t_big_dec = t_big.to_dec();
    let t_choice_dec = t_choice.to_dec();
    let sqrt_t_choice = d_sqrt(t_choice_dec, "pricing::chooser::sqrt_t_choice")?;

    // Standard BS d-values for the final expiration T. Both reject a zero
    // underlying or strike, so `S / K` below is a ratio of two positives.
    let d1_val = d1(s, k, b, t_big, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b, t_big, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    // d-values for the choice date t
    // y1 = [ln(S/K) + b*T + (σ²/2)*t] / (σ√t)
    // y2 = y1 - σ√t
    //
    // The two horizons do not collapse into one: the carry accrues over the
    // full life T because the chooser settles at T, while only the variance
    // realised up to the choice date t is still uncertain when the holder
    // decides.
    let sigma_dec = sigma.to_dec();
    let sigma_sqrt_t_choice = d_mul(
        sigma_dec,
        sqrt_t_choice,
        "pricing::chooser::sigma_sqrt_t_choice",
    )?;
    let carry_to_expiry = d_mul(b, t_big_dec, "pricing::chooser::carry_to_expiry")?;
    let half_variance_to_choice = d_mul(
        d_div(
            d_mul(sigma_dec, sigma_dec, "pricing::chooser::variance")?,
            dec!(2),
            "pricing::chooser::half_variance",
        )?,
        t_choice_dec,
        "pricing::chooser::half_variance_to_choice",
    )?;
    let drift = d_add(
        carry_to_expiry,
        half_variance_to_choice,
        "pricing::chooser::drift",
    )?;
    let moneyness = d_div(s.to_dec(), k.to_dec(), "pricing::chooser::moneyness")?;
    // `None` carries the `ln(S / K) = -∞` limit reached when the moneyness
    // rounds below the smallest representable `Decimal`.
    let y1_numerator = if moneyness.is_zero() {
        None
    } else {
        Some(d_add(
            d_ln(moneyness, "pricing::chooser::log_moneyness")?,
            drift,
            "pricing::chooser::y1_numerator",
        )?)
    };

    // `N(-y)` at the choice date. When the choice-date diffusion `σ√t`
    // collapses to zero the exponents diverge and both CDFs saturate at the
    // sign of the numerator, which then reads `ln(S/K) + b*T`. That is the
    // sign of `C - P = e^(-rT) * (S*e^(bT) - K)`: with no diffusion left the
    // holder must choose now, and takes the branch the forward favours.
    // Saturating to zero keeps the call legs and drops the `y` legs;
    // saturating to one turns the sum into the put by parity.
    //
    // A zero numerator leaves `y1` at `0 / 0`, but the *price* is no longer
    // ambiguous: it says `S*e^(bT) = K`, where call and put are worth the
    // same, so both saturations return the same number. The call branch is
    // taken rather than reporting an error.
    let (n_neg_y1, n_neg_y2) = match (y1_numerator, sigma_sqrt_t_choice.is_zero()) {
        (None, _) => (Decimal::ONE, Decimal::ONE),
        (Some(numerator), true) => {
            if numerator < Decimal::ZERO {
                (Decimal::ONE, Decimal::ONE)
            } else {
                (Decimal::ZERO, Decimal::ZERO)
            }
        }
        (Some(numerator), false) => {
            let y1 = d_div(numerator, sigma_sqrt_t_choice, "pricing::chooser::y1")?;
            let y2 = d_sub(y1, sigma_sqrt_t_choice, "pricing::chooser::y2")?;
            (
                big_n(-y1).unwrap_or(Decimal::ZERO),
                big_n(-y2).unwrap_or(Decimal::ZERO),
            )
        }
    };

    // Get cumulative normal values
    let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
    let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);

    // Discount factors. Every leg settles at T, the `y` legs included: the
    // choice date only fixes which branch survives, not when it pays.
    let dividend_discount_t = d_exp(
        d_mul(-q, t_big_dec, "pricing::chooser::neg_qt")?,
        "pricing::chooser::dividend_discount_t",
    )?;
    let discount_t = d_exp(
        d_mul(-r, t_big_dec, "pricing::chooser::neg_rt")?,
        "pricing::chooser::discount_t",
    )?;

    // Rubinstein (1991) simple chooser formula, Haug 2nd ed. §2.5.1:
    // V = S*e^(-qT)*N(d1) - K*e^(-rT)*N(d2) + K*e^(-rT)*N(-y2) - S*e^(-qT)*N(-y1)
    // This equals: Call(K, T) + Put_component_for_choice_flexibility.
    // The two monetary products are formed once and checked before any CDF
    // weight multiplies in, so a saturating weight cannot mask an overflow
    // in `S * e^(-qT)` or `K * e^(-rT)`.
    let s_pv = d_mul(
        s.to_dec(),
        dividend_discount_t,
        "pricing::chooser::price::s_pv",
    )?;
    let k_pv = d_mul(k.to_dec(), discount_t, "pricing::chooser::price::k_pv")?;
    let leg_s_t = d_mul(s_pv, n_d1, "pricing::chooser::price::leg_s_t")?;
    let leg_k_t = d_mul(k_pv, n_d2, "pricing::chooser::price::leg_k_t")?;
    let leg_k_choice = d_mul(k_pv, n_neg_y2, "pricing::chooser::price::leg_k_choice")?;
    let leg_s_choice = d_mul(s_pv, n_neg_y1, "pricing::chooser::price::leg_s_choice")?;
    let diff1 = d_sub(leg_s_t, leg_k_t, "pricing::chooser::price::diff1")?;
    let diff2 = d_add(diff1, leg_k_choice, "pricing::chooser::price::diff2")?;
    let price = d_sub(diff2, leg_s_choice, "pricing::chooser::price")?;

    Ok(apply_side(price.max(Decimal::ZERO), option))
}

/// Handles the edge case where choice date equals or exceeds expiration.
fn price_at_choice_equals_expiry(option: &Options) -> Result<Decimal, PricingError> {
    // At this point, chooser becomes max(call, put) = straddle at expiry
    // For European option at expiry, this is max(intrinsic_call, intrinsic_put)
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
        let call_intrinsic = d_sub(
            s.to_dec(),
            k.to_dec(),
            "pricing::chooser::expiry::call::intrinsic",
        )?
        .max(Decimal::ZERO);
        let put_intrinsic = d_sub(
            k.to_dec(),
            s.to_dec(),
            "pricing::chooser::expiry::put::intrinsic",
        )?
        .max(Decimal::ZERO);
        return Ok(apply_side(call_intrinsic.max(put_intrinsic), option));
    }

    // Price as call + put (straddle) since choice is at expiry
    let b = d_sub(r, q, "pricing::chooser::expiry::carry")?;
    let d1_val = d1(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
    let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
    let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
    let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);

    let t_dec = t.to_dec();
    let dividend_discount = d_exp(
        d_mul(-q, t_dec, "pricing::chooser::expiry::neg_qt")?,
        "pricing::chooser::expiry::dividend_discount",
    )?;
    let discount = d_exp(
        d_mul(-r, t_dec, "pricing::chooser::expiry::neg_rt")?,
        "pricing::chooser::expiry::discount",
    )?;

    // Call + Put = Straddle
    let s_pv = d_mul(
        s.to_dec(),
        dividend_discount,
        "pricing::chooser::expiry::s_pv",
    )?;
    let k_pv = d_mul(k.to_dec(), discount, "pricing::chooser::expiry::k_pv")?;
    let call_s_leg = d_mul(s_pv, n_d1, "pricing::chooser::expiry::call_s_leg")?;
    let call_k_leg = d_mul(k_pv, n_d2, "pricing::chooser::expiry::call_k_leg")?;
    let put_k_leg = d_mul(k_pv, n_neg_d2, "pricing::chooser::expiry::put_k_leg")?;
    let put_s_leg = d_mul(s_pv, n_neg_d1, "pricing::chooser::expiry::put_s_leg")?;
    let call_price = d_sub(call_s_leg, call_k_leg, "pricing::chooser::expiry::call")?;
    let put_price = d_sub(put_k_leg, put_s_leg, "pricing::chooser::expiry::put")?;
    let price = d_add(call_price, put_price, "pricing::chooser::expiry::price")?;

    Ok(apply_side(price.max(Decimal::ZERO), option))
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
    use crate::model::decimal::f64_to_decimal;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_chooser_option(choice_date_days: f64) -> Options {
        Options::new(
            OptionType::Chooser {
                choice_date: pos_or_panic!(choice_date_days),
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,                          // strike
            ExpirationDate::Days(pos_or_panic!(182.5)), // ~0.5 years
            pos_or_panic!(0.25),                        // volatility
            Positive::ONE,                              // quantity
            Positive::HUNDRED,                          // underlying (ATM)
            dec!(0.05),                                 // risk-free rate
            OptionStyle::Call,                          // Will be ignored for chooser
            Positive::ZERO,                             // dividend yield
            None,
        )
    }

    #[test]
    fn test_simple_chooser() {
        let option = create_chooser_option(45.0); // Choice in 45 days
        let price = chooser_black_scholes(&option).unwrap();
        // Chooser should have positive value (it's always >= call or put)
        assert!(
            price > Decimal::ZERO,
            "Chooser should be positive: {}",
            price
        );
    }

    #[test]
    fn test_chooser_more_valuable_than_call() {
        let chooser = create_chooser_option(45.0);
        let chooser_price = chooser_black_scholes(&chooser).unwrap();

        // Create equivalent vanilla call
        let call = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(182.5)),
            pos_or_panic!(0.25),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        let call_price = crate::pricing::black_scholes_model::black_scholes(&call).unwrap();

        assert!(
            chooser_price >= call_price,
            "Chooser {} should be >= call {}",
            chooser_price,
            call_price
        );
    }

    #[test]
    fn test_chooser_more_valuable_than_put() {
        let chooser = create_chooser_option(45.0);
        let chooser_price = chooser_black_scholes(&chooser).unwrap();

        // Create equivalent vanilla put
        let put = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(182.5)),
            pos_or_panic!(0.25),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Put,
            Positive::ZERO,
            None,
        );
        let put_price = crate::pricing::black_scholes_model::black_scholes(&put).unwrap();

        assert!(
            chooser_price >= put_price,
            "Chooser {} should be >= put {}",
            chooser_price,
            put_price
        );
    }

    #[test]
    fn test_early_choice_date() {
        // Very early choice date (1 day)
        let option = create_chooser_option(1.0);
        let price = chooser_black_scholes(&option).unwrap();
        assert!(price > Decimal::ZERO, "Early choice date price: {}", price);
    }

    #[test]
    fn test_late_choice_date() {
        // Choice date close to expiration
        let option = create_chooser_option(180.0);
        let price = chooser_black_scholes(&option).unwrap();
        assert!(price > Decimal::ZERO, "Late choice date price: {}", price);
    }

    #[test]
    fn test_choice_at_expiry() {
        // Choice at expiration = straddle
        let option = create_chooser_option(182.5);
        let price = chooser_black_scholes(&option).unwrap();
        assert!(price > Decimal::ZERO, "Choice at expiry price: {}", price);
    }

    #[test]
    fn test_short_chooser_option() {
        let mut option = create_chooser_option(45.0);
        let long_price = chooser_black_scholes(&option).unwrap();

        option.side = Side::Short;
        let short_price = chooser_black_scholes(&option).unwrap();

        assert_decimal_eq!(long_price, -short_price, dec!(1e-10));
    }

    #[test]
    fn test_zero_time_to_expiry() {
        let mut option = create_chooser_option(0.0);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = chooser_black_scholes(&option).unwrap();
        // ATM at expiry, intrinsic is 0
        assert_decimal_eq!(price, Decimal::ZERO, dec!(0.01));
    }

    #[test]
    fn test_itm_call_at_expiry() {
        let mut option = create_chooser_option(0.0);
        option.underlying_price = pos_or_panic!(110.0);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = chooser_black_scholes(&option).unwrap();
        // ITM call intrinsic = 10
        assert_decimal_eq!(price, dec!(10.0), dec!(0.01));
    }

    #[test]
    fn test_itm_put_at_expiry() {
        let mut option = create_chooser_option(0.0);
        option.underlying_price = pos_or_panic!(90.0);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = chooser_black_scholes(&option).unwrap();
        // ITM put intrinsic = 10
        assert_decimal_eq!(price, dec!(10.0), dec!(0.01));
    }

    /// Builds a chooser with every parameter spelled out, for the reference
    /// regressions below. `expiry_days` and `choice_days` are calendar days
    /// on the Actual/365 convention the crate uses everywhere.
    #[allow(clippy::too_many_arguments)]
    fn chooser_with(
        underlying: f64,
        strike: f64,
        expiry_days: f64,
        choice_days: f64,
        sigma: f64,
        rate: Decimal,
        dividend: f64,
    ) -> Options {
        Options::new(
            OptionType::Chooser {
                choice_date: pos_or_panic!(choice_days),
            },
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(strike),
            ExpirationDate::Days(pos_or_panic!(expiry_days)),
            pos_or_panic!(sigma),
            Positive::ONE,
            pos_or_panic!(underlying),
            rate,
            OptionStyle::Call,
            pos_or_panic!(dividend),
            None,
        )
    }

    /// Builds the European leg used to cross-check the chooser decomposition.
    fn european(
        underlying: f64,
        strike: f64,
        expiry_days: f64,
        sigma: f64,
        rate: Decimal,
        dividend: f64,
        style: OptionStyle,
    ) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(strike),
            ExpirationDate::Days(pos_or_panic!(expiry_days)),
            pos_or_panic!(sigma),
            Positive::ONE,
            pos_or_panic!(underlying),
            rate,
            style,
            pos_or_panic!(dividend),
            None,
        )
    }

    /// Haug, *The Complete Guide to Option Pricing Formulas*, 2nd ed.,
    /// §2.5.1 "Simple Chooser Options", worked example: S = X = 50,
    /// t = 0.25, T = 0.5, r = b = 0.08, σ = 0.25, value 6.1071.
    ///
    /// This is the acceptance test for #448. Before the fix the function
    /// discounted the `y` legs at the choice date instead of at expiry and
    /// used `b*t` where the formula has `b*T + σ²*t/2`, returning 6.5241.
    #[test]
    fn test_simple_chooser_haug_example_matches_reference() {
        // 182.5 / 365 = 0.5 years to expiry, 91.25 / 365 = 0.25 to the choice.
        let option = chooser_with(50.0, 50.0, 182.5, 91.25, 0.25, dec!(0.08), 0.0);
        let price = chooser_black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(6.1071), dec!(1e-4));
    }

    /// `max(C_t, P_t) = C_t + max(0, P_t - C_t)` at the choice date. By
    /// put-call parity the second term is `e^(-q(T-t))` puts expiring at t
    /// struck at the discounted forward `K*e^(-b(T-t))`, so the chooser must
    /// equal a T-expiry call plus that scaled t-expiry put. Checked without
    /// dividends, where the scale factor is one.
    #[test]
    fn test_simple_chooser_equals_call_plus_forward_struck_put() {
        let (s, k, expiry_days, choice_days, sigma, r) =
            (50.0, 50.0, 182.5, 91.25, 0.25, dec!(0.08));
        let chooser = chooser_with(s, k, expiry_days, choice_days, sigma, r, 0.0);
        let chooser_price = chooser_black_scholes(&chooser).unwrap();

        let call = european(s, k, expiry_days, sigma, r, 0.0, OptionStyle::Call);
        let call_price = crate::pricing::black_scholes_model::black_scholes(&call).unwrap();

        // K * e^(-b(T-t)) with b = r (no dividends).
        let carry_gap = 0.08 * (expiry_days - choice_days) / 365.0;
        let adjusted_strike = k * (-carry_gap).exp();
        let put = european(
            s,
            adjusted_strike,
            choice_days,
            sigma,
            r,
            0.0,
            OptionStyle::Put,
        );
        let put_price = crate::pricing::black_scholes_model::black_scholes(&put).unwrap();

        assert_decimal_eq!(chooser_price, call_price + put_price, dec!(1e-6));
    }

    /// Same identity with a dividend yield, so the `e^(-q(T-t))` scale on the
    /// embedded put is exercised rather than collapsing to one.
    #[test]
    fn test_simple_chooser_equals_call_plus_forward_struck_put_with_dividend() {
        // S = 100, K = 95, t = 0.25, T = 0.75, r = 0.06, q = 0.03, σ = 0.30.
        let (s, k, expiry_days, choice_days, sigma, r, q) =
            (100.0, 95.0, 273.75, 91.25, 0.30, dec!(0.06), 0.03);
        let chooser = chooser_with(s, k, expiry_days, choice_days, sigma, r, q);
        let chooser_price = chooser_black_scholes(&chooser).unwrap();

        let call = european(s, k, expiry_days, sigma, r, q, OptionStyle::Call);
        let call_price = crate::pricing::black_scholes_model::black_scholes(&call).unwrap();

        let gap_years = (expiry_days - choice_days) / 365.0;
        let adjusted_strike = k * (-(0.06 - q) * gap_years).exp();
        let put = european(
            s,
            adjusted_strike,
            choice_days,
            sigma,
            r,
            q,
            OptionStyle::Put,
        );
        let put_price = crate::pricing::black_scholes_model::black_scholes(&put).unwrap();
        let scale = f64_to_decimal((-q * gap_years).exp()).unwrap();

        assert_decimal_eq!(chooser_price, call_price + put_price * scale, dec!(1e-6));
    }

    /// A choice horizon of zero leaves no diffusion, so `σ√t` collapses and
    /// both CDFs saturate on the sign of `ln(S/K) + b*T`. Spot is *below* the
    /// strike here but the forward is above it, so the call must win: the
    /// pre-#448 numerator reduced to `ln(S/K)` and would have returned the put.
    #[test]
    fn test_simple_chooser_zero_choice_horizon_follows_the_forward_not_the_spot() {
        let (s, k, expiry_days, sigma, r) = (99.0, 100.0, 182.5, 0.25, dec!(0.08));
        let option = chooser_with(s, k, expiry_days, 0.0, sigma, r, 0.0);
        let price = chooser_black_scholes(&option).unwrap();

        let call = european(s, k, expiry_days, sigma, r, 0.0, OptionStyle::Call);
        let call_price = crate::pricing::black_scholes_model::black_scholes(&call).unwrap();
        let put = european(s, k, expiry_days, sigma, r, 0.0, OptionStyle::Put);
        let put_price = crate::pricing::black_scholes_model::black_scholes(&put).unwrap();

        assert!(
            call_price > put_price,
            "forward is above the strike, so the call must be richer: {call_price} vs {put_price}"
        );
        assert_decimal_eq!(price, call_price, dec!(1e-8));
    }

    /// The other side of the same limit: a forward far below the strike makes
    /// the numerator negative, both CDFs saturate at one, and the sum collapses
    /// to the put by parity.
    #[test]
    fn test_simple_chooser_zero_choice_horizon_deep_otm_call_returns_the_put() {
        let (s, k, expiry_days, sigma, r) = (80.0, 100.0, 182.5, 0.25, dec!(0.08));
        let option = chooser_with(s, k, expiry_days, 0.0, sigma, r, 0.0);
        let price = chooser_black_scholes(&option).unwrap();

        let put = european(s, k, expiry_days, sigma, r, 0.0, OptionStyle::Put);
        let put_price = crate::pricing::black_scholes_model::black_scholes(&put).unwrap();

        assert_decimal_eq!(price, put_price, dec!(1e-8));
    }

    /// At the forward-parity spot `S*e^(bT) = K` the `y1` numerator is zero,
    /// which used to be reported as an undefined `0 / 0`. With the corrected
    /// numerator the price is no longer ambiguous — call and put are worth the
    /// same there — so the function returns that common value.
    #[test]
    fn test_simple_chooser_zero_choice_horizon_at_forward_parity_prices_the_tie() {
        // S = 100 * e^(-0.08 * 0.5) puts the forward exactly on the strike.
        let (k, expiry_days, sigma, r) = (100.0, 182.5, 0.25, dec!(0.08));
        let s = k * (-0.04f64).exp();
        let option = chooser_with(s, k, expiry_days, 0.0, sigma, r, 0.0);
        let price = chooser_black_scholes(&option).unwrap();

        let call = european(s, k, expiry_days, sigma, r, 0.0, OptionStyle::Call);
        let call_price = crate::pricing::black_scholes_model::black_scholes(&call).unwrap();
        let put = european(s, k, expiry_days, sigma, r, 0.0, OptionStyle::Put);
        let put_price = crate::pricing::black_scholes_model::black_scholes(&put).unwrap();

        assert_decimal_eq!(call_price, put_price, dec!(1e-8));
        assert_decimal_eq!(price, call_price, dec!(1e-8));
    }

    /// Choosing at expiry is choosing after the fact, which is a straddle:
    /// `max((S-K)+, (K-S)+)` pays whichever leg finished in the money.
    #[test]
    fn test_choice_at_expiry_equals_straddle() {
        let (s, k, expiry_days, sigma, r) = (100.0, 100.0, 182.5, 0.25, dec!(0.05));
        let option = chooser_with(s, k, expiry_days, expiry_days, sigma, r, 0.0);
        let price = chooser_black_scholes(&option).unwrap();

        let call = european(s, k, expiry_days, sigma, r, 0.0, OptionStyle::Call);
        let call_price = crate::pricing::black_scholes_model::black_scholes(&call).unwrap();
        let put = european(s, k, expiry_days, sigma, r, 0.0, OptionStyle::Put);
        let put_price = crate::pricing::black_scholes_model::black_scholes(&put).unwrap();

        assert_decimal_eq!(price, call_price + put_price, dec!(1e-8));
    }

    /// The closed form and the `price_at_choice_equals_expiry` branch have to
    /// meet: as t approaches T, `y1` tends to `d1` and `y2` to `d2`, and the
    /// formula collapses to the straddle. A tenth of a day short of expiry the
    /// gap is already under a cent, and the chooser stays below the straddle.
    #[test]
    fn test_simple_chooser_converges_to_the_straddle_as_choice_nears_expiry() {
        let (s, k, expiry_days, sigma, r) = (100.0, 100.0, 182.5, 0.25, dec!(0.05));
        let price =
            chooser_black_scholes(&chooser_with(s, k, expiry_days, 182.4, sigma, r, 0.0)).unwrap();

        let straddle =
            chooser_black_scholes(&chooser_with(s, k, expiry_days, expiry_days, sigma, r, 0.0))
                .unwrap();

        assert!(
            price < straddle,
            "an earlier choice cannot be worth more than choosing at expiry: {price} vs {straddle}"
        );
        assert_decimal_eq!(price, straddle, dec!(0.01));
    }

    #[test]
    fn test_higher_vol_means_higher_chooser_value() {
        let low_vol = create_chooser_option(45.0);
        let low_vol_price = chooser_black_scholes(&low_vol).unwrap();

        let mut high_vol = low_vol.clone();
        high_vol.implied_volatility = pos_or_panic!(0.4);
        let high_vol_price = chooser_black_scholes(&high_vol).unwrap();

        assert!(
            high_vol_price > low_vol_price,
            "Higher vol {} should mean higher chooser value: {}",
            high_vol_price,
            low_vol_price
        );
    }
}
