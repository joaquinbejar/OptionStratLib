/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-04-27
******************************************************************************/

//! # Black-76 Greeks
//!
//! Closed-form Greeks (delta, gamma, vega, theta, rho) for the Black-76 (Black 1976)
//! pricing model on options on futures, forwards, swaptions, caps/floors, and
//! commodity futures.
//!
//! Black-76 differs from Black–Scholes–Merton in that the input is the forward
//! price `F` (with the cost of carry already baked in) rather than the spot
//! price `S`. The drift in `d1`/`d2` is therefore zero and the dividend yield is
//! irrelevant. Both legs of every Greek share the single discount factor
//! `e^(-rT)`.
//!
//! Greek units mirror the BSM module:
//!
//! * `delta_b76` — per unit move in `F`, applies long/short sign.
//! * `gamma_b76` — per unit move in `F` (second order).
//! * `vega_b76`  — per **1%** change in volatility.
//! * `theta_b76` — per **calendar day** (annual figure divided by 365).
//! * `rho_b76`   — per **1%** change in the risk-free rate.
//!
//! All quantities are returned as `Decimal` and scale linearly with
//! `option.quantity`.
//!
//! See `src/pricing/black_76.rs` for the matching pricing kernel.

use crate::Options;
use crate::error::PricingError;
use crate::error::greeks::GreeksError;
use crate::greeks::utils::{big_n, calculate_d_values_black_76, n};
use crate::model::decimal::{d_div, d_mul, d_sub};
use crate::model::types::{OptionStyle, OptionType, Side};
use rust_decimal::{Decimal, MathematicalOps};
use tracing::{instrument, trace};

/// Reject any option type that is not European; Black-76 only prices European
/// futures/forward options.
#[cold]
fn reject_non_european(label: &str) -> GreeksError {
    GreeksError::Pricing(Box::new(PricingError::unsupported_option_type(
        label, "Black-76",
    )))
}

fn ensure_european(option: &Options) -> Result<(), GreeksError> {
    match option.option_type {
        OptionType::European => Ok(()),
        OptionType::American => Err(reject_non_european("American")),
        OptionType::Bermuda { .. } => Err(reject_non_european("Bermuda")),
        _ => Err(reject_non_european("exotic")),
    }
}

fn side_sign(option: &Options) -> Decimal {
    if matches!(option.side, Side::Long) {
        Decimal::ONE
    } else {
        Decimal::NEGATIVE_ONE
    }
}

fn discount_factor(option: &Options, t: Decimal) -> Decimal {
    (-option.risk_free_rate * t).exp()
}

/// Computes the delta of an option under the Black-76 model.
///
/// # Formulas
///
/// - Call: `Δ_call = e^(-rT) · N(d1)`
/// - Put:  `Δ_put  = -e^(-rT) · N(-d1)`
///
/// The result is multiplied by `+1` for `Side::Long` and `-1` for `Side::Short`,
/// then by `option.quantity`.
///
/// # Errors
///
/// - `GreeksError::Pricing(UnsupportedOptionType)` for non-European options.
/// - Propagates `GreeksError` from `calculate_d_values_black_76` when expiration
///   or volatility are non-positive.
#[instrument(skip(option), fields(
    strike = %option.strike_price,
    style = ?option.option_style,
    side = ?option.side,
))]
pub fn delta_b76(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let t = option.expiration_date.get_years()?.to_dec();
    let (d1, _d2) = calculate_d_values_black_76(option)?;

    let df = discount_factor(option, t);
    let raw = match option.option_style {
        OptionStyle::Call => d_mul(df, big_n(d1)?, "greeks::black_76::delta::call")?,
        OptionStyle::Put => -d_mul(df, big_n(-d1)?, "greeks::black_76::delta::put")?,
    };

    let signed = d_mul(side_sign(option), raw, "greeks::black_76::delta::sign")?;
    let result = d_mul(
        signed,
        option.quantity.to_dec(),
        "greeks::black_76::delta::quantity",
    )?;

    trace!(
        "Black-76 Delta: F={}, K={}, e^(-rT)={}, d1={}, raw={}, result={}",
        option.underlying_price, option.strike_price, df, d1, raw, result
    );
    Ok(result)
}

/// Computes the gamma of an option under the Black-76 model.
///
/// # Formula
///
/// `Γ = e^(-rT) · n(d1) / (F · σ · √T)`
///
/// Gamma is identical for calls and puts and does not flip with `Side`. Result
/// is multiplied by `option.quantity`.
///
/// # Errors
///
/// - `GreeksError::Pricing(UnsupportedOptionType)` for non-European options.
/// - Propagates `GreeksError` from `calculate_d_values_black_76` when expiration
///   or volatility are non-positive.
#[instrument(skip(option), fields(strike = %option.strike_price))]
pub fn gamma_b76(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let t = option.expiration_date.get_years()?;
    let (d1, _d2) = calculate_d_values_black_76(option)?;

    let df = discount_factor(option, t.to_dec());
    let f = option.underlying_price.to_dec();
    let sigma = option.implied_volatility.to_dec();
    let sqrt_t = t.sqrt().to_dec();

    let denom = d_mul(
        f,
        d_mul(sigma, sqrt_t, "greeks::black_76::gamma::sigma_sqrt_t")?,
        "greeks::black_76::gamma::denom",
    )?;
    let numer = d_mul(df, n(d1)?, "greeks::black_76::gamma::numer")?;
    let raw = d_div(numer, denom, "greeks::black_76::gamma::raw")?;

    let result = d_mul(
        raw,
        option.quantity.to_dec(),
        "greeks::black_76::gamma::quantity",
    )?;
    trace!(
        "Black-76 Gamma: F={}, K={}, d1={}, df={}, result={}",
        option.underlying_price, option.strike_price, d1, df, result
    );
    Ok(result)
}

/// Computes the vega of an option under the Black-76 model, per 1% change in
/// volatility.
///
/// # Formula
///
/// `ν = F · e^(-rT) · n(d1) · √T`, divided by 100 to express the sensitivity per
/// 1 percentage point of volatility, then multiplied by `option.quantity`.
///
/// Vega is identical for calls and puts and does not flip with `Side`.
///
/// # Errors
///
/// - `GreeksError::Pricing(UnsupportedOptionType)` for non-European options.
/// - Propagates `GreeksError` from `calculate_d_values_black_76` when expiration
///   or volatility are non-positive.
#[instrument(skip(option), fields(strike = %option.strike_price))]
pub fn vega_b76(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let t = option.expiration_date.get_years()?;
    let (d1, _d2) = calculate_d_values_black_76(option)?;

    let df = discount_factor(option, t.to_dec());
    let f = option.underlying_price.to_dec();
    let sqrt_t = t.sqrt().to_dec();

    // F * e^(-rT) * n(d1) * √T
    let leg1 = d_mul(f, df, "greeks::black_76::vega::f_df")?;
    let leg2 = d_mul(leg1, n(d1)?, "greeks::black_76::vega::times_n")?;
    let raw = d_mul(leg2, sqrt_t, "greeks::black_76::vega::times_sqrt_t")?;

    let weighted = d_mul(
        raw,
        option.quantity.to_dec(),
        "greeks::black_76::vega::quantity",
    )?;
    let result = d_div(
        weighted,
        Decimal::ONE_HUNDRED,
        "greeks::black_76::vega::per_pct",
    )?;
    trace!(
        "Black-76 Vega: F={}, K={}, d1={}, df={}, result={}",
        option.underlying_price, option.strike_price, d1, df, result
    );
    Ok(result)
}

/// Computes the theta of an option under the Black-76 model, per calendar day.
///
/// # Formulas (annual)
///
/// - `Θ_call_year = -F·e^(-rT)·n(d1)·σ/(2√T) + r·F·e^(-rT)·N(d1) - r·K·e^(-rT)·N(d2)`
/// - `Θ_put_year  = -F·e^(-rT)·n(d1)·σ/(2√T) - r·F·e^(-rT)·N(-d1) + r·K·e^(-rT)·N(-d2)`
///
/// The annual figure is divided by 365 to express decay per calendar day, then
/// multiplied by `option.quantity`.
///
/// Theta does not flip with `Side`; the sign is governed by the formula itself.
///
/// # Errors
///
/// - `GreeksError::Pricing(UnsupportedOptionType)` for non-European options.
/// - Propagates `GreeksError` from `calculate_d_values_black_76` when expiration
///   or volatility are non-positive.
#[instrument(skip(option), fields(strike = %option.strike_price))]
pub fn theta_b76(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let t = option.expiration_date.get_years()?;
    let (d1, d2) = calculate_d_values_black_76(option)?;

    let df = discount_factor(option, t.to_dec());
    let f = option.underlying_price.to_dec();
    let k = option.strike_price.to_dec();
    let r = option.risk_free_rate;
    let sigma = option.implied_volatility.to_dec();
    let sqrt_t = t.sqrt().to_dec();

    // Common (volatility-decay) term, negative: -F·e^(-rT)·n(d1)·σ/(2·√T)
    let two_sqrt_t = d_mul(Decimal::TWO, sqrt_t, "greeks::black_76::theta::two_sqrt_t")?;
    let f_df = d_mul(f, df, "greeks::black_76::theta::f_df")?;
    let f_df_n = d_mul(f_df, n(d1)?, "greeks::black_76::theta::f_df_n")?;
    let f_df_n_sigma = d_mul(f_df_n, sigma, "greeks::black_76::theta::f_df_n_sigma")?;
    let f_df_n_sigma_over_2sqrt_t =
        d_div(f_df_n_sigma, two_sqrt_t, "greeks::black_76::theta::div")?;
    let common = -f_df_n_sigma_over_2sqrt_t;

    // r-dependent legs (annual, undiscounted view)
    let r_f_df = d_mul(r, f_df, "greeks::black_76::theta::r_f_df")?;
    let k_df = d_mul(k, df, "greeks::black_76::theta::k_df")?;
    let r_k_df = d_mul(r, k_df, "greeks::black_76::theta::r_k_df")?;

    let annual = match option.option_style {
        OptionStyle::Call => {
            // Θ_call = common + r·F·e^(-rT)·N(d1) − r·K·e^(-rT)·N(d2)
            let plus = d_mul(r_f_df, big_n(d1)?, "greeks::black_76::theta::call::plus")?;
            let minus = d_mul(r_k_df, big_n(d2)?, "greeks::black_76::theta::call::minus")?;
            d_sub(common + plus, minus, "greeks::black_76::theta::call::sum")?
        }
        OptionStyle::Put => {
            // Θ_put = common − r·F·e^(-rT)·N(−d1) + r·K·e^(-rT)·N(−d2)
            let minus = d_mul(r_f_df, big_n(-d1)?, "greeks::black_76::theta::put::minus")?;
            let plus = d_mul(r_k_df, big_n(-d2)?, "greeks::black_76::theta::put::plus")?;
            d_sub(common + plus, minus, "greeks::black_76::theta::put::sum")?
        }
    };

    let weighted = d_mul(
        annual,
        option.quantity.to_dec(),
        "greeks::black_76::theta::quantity",
    )?;
    let result = d_div(
        weighted,
        Decimal::from(365),
        "greeks::black_76::theta::per_day",
    )?;
    trace!(
        "Black-76 Theta: F={}, K={}, d1={}, d2={}, annual={}, result={}",
        option.underlying_price, option.strike_price, d1, d2, annual, result
    );
    Ok(result)
}

/// Computes the rho of an option under the Black-76 model, per 1% change in the
/// risk-free rate.
///
/// # Formulas
///
/// In Black-76 the rate `r` only appears in the discount factor `e^(-rT)`, so
/// the rho of the long option is simply `-T · price`. For a call:
///
/// `ρ_call = K · T · e^(-rT) · N(d2) - F · T · e^(-rT) · N(d1) = -T · C`
///
/// And symmetrically `ρ_put = -T · P`.
///
/// The annual figure is divided by 100 to express the sensitivity per 1
/// percentage point of rate, then multiplied by `option.quantity`. Rho does not
/// flip with `Side`.
///
/// # Errors
///
/// - `GreeksError::Pricing(UnsupportedOptionType)` for non-European options.
/// - Propagates `GreeksError` from `calculate_d_values_black_76` when expiration
///   or volatility are non-positive.
#[instrument(skip(option), fields(strike = %option.strike_price))]
pub fn rho_b76(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let t = option.expiration_date.get_years()?;
    let (d1, d2) = calculate_d_values_black_76(option)?;

    let df = discount_factor(option, t.to_dec());
    let f = option.underlying_price.to_dec();
    let k = option.strike_price.to_dec();
    let t_dec = t.to_dec();

    // Long-side, undiscounted rho:
    //   call: -T · e^(-rT) · [F N(d1) - K N(d2)]
    //   put : -T · e^(-rT) · [K N(-d2) - F N(-d1)]
    let bracket = match option.option_style {
        OptionStyle::Call => d_sub(
            d_mul(f, big_n(d1)?, "greeks::black_76::rho::call::f_leg")?,
            d_mul(k, big_n(d2)?, "greeks::black_76::rho::call::k_leg")?,
            "greeks::black_76::rho::call::bracket",
        )?,
        OptionStyle::Put => d_sub(
            d_mul(k, big_n(-d2)?, "greeks::black_76::rho::put::k_leg")?,
            d_mul(f, big_n(-d1)?, "greeks::black_76::rho::put::f_leg")?,
            "greeks::black_76::rho::put::bracket",
        )?,
    };

    let neg_t_df = d_mul(-t_dec, df, "greeks::black_76::rho::neg_t_df")?;
    let annual = d_mul(neg_t_df, bracket, "greeks::black_76::rho::annual")?;

    let weighted = d_mul(
        annual,
        option.quantity.to_dec(),
        "greeks::black_76::rho::quantity",
    )?;
    let result = d_div(
        weighted,
        Decimal::ONE_HUNDRED,
        "greeks::black_76::rho::per_pct",
    )?;
    trace!(
        "Black-76 Rho: F={}, K={}, d1={}, d2={}, annual={}, result={}",
        option.underlying_price, option.strike_price, d1, d2, annual, result
    );
    Ok(result)
}

/// Trait that exposes Black-76 Greeks for any type that can produce an
/// [`Options`] reference.
///
/// Mirrors the `Black76` pricing trait. Implementors only need to provide
/// [`Black76Greeks::get_option`]; default implementations route to the
/// free-function Greeks above.
pub trait Black76Greeks {
    /// Returns the option to price.
    fn get_option(&self) -> Result<&Options, GreeksError>;

    /// Black-76 delta — see [`delta_b76`].
    fn delta_b76(&self) -> Result<Decimal, GreeksError> {
        delta_b76(self.get_option()?)
    }

    /// Black-76 gamma — see [`gamma_b76`].
    fn gamma_b76(&self) -> Result<Decimal, GreeksError> {
        gamma_b76(self.get_option()?)
    }

    /// Black-76 vega — see [`vega_b76`].
    fn vega_b76(&self) -> Result<Decimal, GreeksError> {
        vega_b76(self.get_option()?)
    }

    /// Black-76 theta — see [`theta_b76`].
    fn theta_b76(&self) -> Result<Decimal, GreeksError> {
        theta_b76(self.get_option()?)
    }

    /// Black-76 rho — see [`rho_b76`].
    fn rho_b76(&self) -> Result<Decimal, GreeksError> {
        rho_b76(self.get_option()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::greeks::{delta, gamma, vega};
    use crate::pricing::black_76;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn create_option(
        f: f64,
        k: f64,
        r: Decimal,
        t_days: f64,
        sigma: f64,
        style: OptionStyle,
    ) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "FUT".to_string(),
            pos_or_panic!(k),
            ExpirationDate::Days(pos_or_panic!(t_days)),
            pos_or_panic!(sigma),
            Positive::ONE,
            pos_or_panic!(f),
            r,
            style,
            pos_or_panic!(0.0),
            None,
        )
    }

    fn close(a: Decimal, b: Decimal, tol: Decimal) -> bool {
        (a - b).abs() < tol
    }

    // ---- delta ----------------------------------------------------------

    #[test]
    fn test_delta_call_in_unit_interval() {
        for f in [80.0, 95.0, 100.0, 105.0, 120.0] {
            let opt = create_option(f, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
            let d = delta_b76(&opt).expect("call delta");
            assert!(d > Decimal::ZERO && d < Decimal::ONE, "F={} -> Δ={}", f, d);
        }
    }

    #[test]
    fn test_delta_put_in_negative_unit_interval() {
        for f in [80.0, 95.0, 100.0, 105.0, 120.0] {
            let opt = create_option(f, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Put);
            let d = delta_b76(&opt).expect("put delta");
            assert!(
                d < Decimal::ZERO && d > Decimal::NEGATIVE_ONE,
                "F={} -> Δ={}",
                f,
                d
            );
        }
    }

    /// Identity: `Δ_call - Δ_put = e^(-rT)` for long-side, qty=1.
    #[test]
    fn test_delta_call_minus_put_equals_discount_factor() {
        let r = dec!(0.05);
        let t_days = 180.0;
        let call = create_option(100.0, 100.0, r, t_days, 0.2, OptionStyle::Call);
        let put = create_option(100.0, 100.0, r, t_days, 0.2, OptionStyle::Put);

        let dc = delta_b76(&call).unwrap();
        let dp = delta_b76(&put).unwrap();

        let years = call.expiration_date.get_years().unwrap().to_dec();
        let expected = (-r * years).exp();
        assert!(
            close(dc - dp, expected, dec!(1e-9)),
            "Δc - Δp = {} expected {}",
            dc - dp,
            expected
        );
    }

    // ---- gamma / vega positivity ---------------------------------------

    #[test]
    fn test_gamma_positive_call_and_put() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for f in [80.0, 100.0, 120.0] {
                let opt = create_option(f, 100.0, dec!(0.05), 180.0, 0.2, style);
                let g = gamma_b76(&opt).expect("gamma");
                assert!(g > Decimal::ZERO, "style={:?} F={} Γ={}", style, f, g);
            }
        }
    }

    #[test]
    fn test_vega_positive_call_and_put() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for f in [80.0, 100.0, 120.0] {
                let opt = create_option(f, 100.0, dec!(0.05), 180.0, 0.2, style);
                let v = vega_b76(&opt).expect("vega");
                assert!(v > Decimal::ZERO, "style={:?} F={} ν={}", style, f, v);
            }
        }
    }

    /// Calls and puts share the same gamma in Black-76.
    #[test]
    fn test_gamma_call_equals_put() {
        let call = create_option(105.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        let put = create_option(105.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Put);
        let gc = gamma_b76(&call).unwrap();
        let gp = gamma_b76(&put).unwrap();
        assert!(close(gc, gp, dec!(1e-15)), "Γc={} Γp={}", gc, gp);
    }

    /// Calls and puts share the same vega in Black-76.
    #[test]
    fn test_vega_call_equals_put() {
        let call = create_option(105.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        let put = create_option(105.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Put);
        let vc = vega_b76(&call).unwrap();
        let vp = vega_b76(&put).unwrap();
        assert!(close(vc, vp, dec!(1e-15)), "νc={} νp={}", vc, vp);
    }

    // ---- theta sanity ---------------------------------------------------

    #[test]
    fn test_theta_call_atm_long_negative() {
        let opt = create_option(100.0, 100.0, dec!(0.05), 30.0, 0.2, OptionStyle::Call);
        let th = theta_b76(&opt).expect("theta");
        assert!(th < Decimal::ZERO, "ATM long call should decay: Θ={}", th);
    }

    #[test]
    fn test_theta_put_atm_long_negative() {
        let opt = create_option(100.0, 100.0, dec!(0.05), 30.0, 0.2, OptionStyle::Put);
        let th = theta_b76(&opt).expect("theta");
        assert!(th < Decimal::ZERO, "ATM long put should decay: Θ={}", th);
    }

    // ---- rho identity ---------------------------------------------------

    /// In Black-76, `ρ_call = -T · price_call` (per pct of rate, before qty).
    #[test]
    fn test_rho_equals_minus_t_times_price_call() {
        let opt = create_option(100.0, 95.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        let rho = rho_b76(&opt).unwrap();
        let price = black_76(&opt).unwrap();
        let years = opt.expiration_date.get_years().unwrap().to_dec();
        let expected = -years * price / Decimal::ONE_HUNDRED;
        assert!(
            close(rho, expected, dec!(1e-15)),
            "ρ={} expected={}",
            rho,
            expected
        );
    }

    /// In Black-76, `ρ_put = -T · price_put` (per pct of rate, before qty).
    #[test]
    fn test_rho_equals_minus_t_times_price_put() {
        let opt = create_option(100.0, 105.0, dec!(0.05), 180.0, 0.2, OptionStyle::Put);
        let rho = rho_b76(&opt).unwrap();
        let price = black_76(&opt).unwrap();
        let years = opt.expiration_date.get_years().unwrap().to_dec();
        let expected = -years * price / Decimal::ONE_HUNDRED;
        assert!(
            close(rho, expected, dec!(1e-15)),
            "ρ={} expected={}",
            rho,
            expected
        );
    }

    // ---- BSM cross-check (S = F·e^(-rT), q = 0) -------------------------

    fn make_bsm_match(opt_b76: &Options) -> Options {
        let years = opt_b76.expiration_date.get_years().unwrap().to_dec();
        let df = (-opt_b76.risk_free_rate * years).exp();
        let s = Positive::new_decimal(opt_b76.underlying_price.to_dec() * df).unwrap();
        let mut bsm = opt_b76.clone();
        bsm.underlying_price = s;
        // dividend_yield already 0 from create_option
        bsm
    }

    /// `Δ_b76 = e^(-rT) · Δ_bsm` under the discounted-spot transform, q=0.
    #[test]
    fn test_delta_matches_bsm_with_discounted_spot() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for f in [90.0, 100.0, 110.0] {
                let opt = create_option(f, 100.0, dec!(0.05), 180.0, 0.2, style);
                let bsm = make_bsm_match(&opt);
                let years = opt.expiration_date.get_years().unwrap().to_dec();
                let df = (-opt.risk_free_rate * years).exp();
                let lhs = delta_b76(&opt).unwrap();
                let rhs = df * delta(&bsm).unwrap();
                assert!(
                    close(lhs, rhs, dec!(1e-9)),
                    "style={:?} F={} Δ_b76={} vs e^(-rT)·Δ_bsm={}",
                    style,
                    f,
                    lhs,
                    rhs
                );
            }
        }
    }

    /// `ν_b76 = ν_bsm` under the discounted-spot transform, q=0.
    #[test]
    fn test_vega_matches_bsm_with_discounted_spot() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for f in [90.0, 100.0, 110.0] {
                let opt = create_option(f, 100.0, dec!(0.05), 180.0, 0.2, style);
                let bsm = make_bsm_match(&opt);
                let lhs = vega_b76(&opt).unwrap();
                let rhs = vega(&bsm).unwrap();
                assert!(
                    close(lhs, rhs, dec!(1e-9)),
                    "style={:?} F={} ν_b76={} vs ν_bsm={}",
                    style,
                    f,
                    lhs,
                    rhs
                );
            }
        }
    }

    /// `Γ_b76 = e^(-2rT) · Γ_bsm` under the discounted-spot transform, q=0.
    #[test]
    fn test_gamma_matches_bsm_with_discounted_spot() {
        for f in [90.0, 100.0, 110.0] {
            let opt = create_option(f, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
            let bsm = make_bsm_match(&opt);
            let years = opt.expiration_date.get_years().unwrap().to_dec();
            let df2 = (-Decimal::TWO * opt.risk_free_rate * years).exp();
            let lhs = gamma_b76(&opt).unwrap();
            let rhs = df2 * gamma(&bsm).unwrap();
            assert!(
                close(lhs, rhs, dec!(1e-9)),
                "F={} Γ_b76={} vs e^(-2rT)·Γ_bsm={}",
                f,
                lhs,
                rhs
            );
        }
    }

    // ---- error paths ----------------------------------------------------

    #[test]
    fn test_zero_volatility_returns_error() {
        let opt = create_option(100.0, 100.0, dec!(0.05), 180.0, 0.0, OptionStyle::Call);
        assert!(delta_b76(&opt).is_err());
        assert!(gamma_b76(&opt).is_err());
        assert!(vega_b76(&opt).is_err());
        assert!(theta_b76(&opt).is_err());
        assert!(rho_b76(&opt).is_err());
    }

    #[test]
    fn test_unsupported_american_returns_error() {
        let mut opt = create_option(100.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        opt.option_type = OptionType::American;
        for r in [
            delta_b76(&opt),
            gamma_b76(&opt),
            vega_b76(&opt),
            theta_b76(&opt),
            rho_b76(&opt),
        ] {
            match r {
                Err(GreeksError::Pricing(boxed)) => match *boxed {
                    PricingError::UnsupportedOptionType { .. } => {}
                    other => panic!("wrong PricingError variant: {:?}", other),
                },
                Err(other) => panic!("wrong error: {:?}", other),
                Ok(_) => panic!("expected error for American"),
            }
        }
    }

    #[test]
    fn test_unsupported_bermuda_returns_error() {
        let mut opt = create_option(100.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        opt.option_type = OptionType::Bermuda {
            exercise_dates: vec![],
        };
        assert!(matches!(delta_b76(&opt), Err(GreeksError::Pricing(_))));
    }

    // ---- side and quantity ---------------------------------------------

    #[test]
    fn test_side_short_negates_delta() {
        let long = create_option(100.0, 95.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        let mut short = long.clone();
        short.side = Side::Short;
        let dl = delta_b76(&long).unwrap();
        let ds = delta_b76(&short).unwrap();
        assert_eq!(dl, -ds);
    }

    #[test]
    fn test_quantity_scales_linearly() {
        let mut opt = create_option(100.0, 95.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        let d1 = delta_b76(&opt).unwrap();
        opt.quantity = pos_or_panic!(3.0);
        let d3 = delta_b76(&opt).unwrap();
        assert!(close(d3, d1 * Decimal::from(3), dec!(1e-15)));
    }

    // ---- trait ----------------------------------------------------------

    #[test]
    fn test_black76_greeks_trait() {
        struct Wrap(Options);
        impl Black76Greeks for Wrap {
            fn get_option(&self) -> Result<&Options, GreeksError> {
                Ok(&self.0)
            }
        }
        let opt = create_option(100.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        let w = Wrap(opt.clone());
        assert_eq!(w.delta_b76().unwrap(), delta_b76(&opt).unwrap());
        assert_eq!(w.gamma_b76().unwrap(), gamma_b76(&opt).unwrap());
        assert_eq!(w.vega_b76().unwrap(), vega_b76(&opt).unwrap());
        assert_eq!(w.theta_b76().unwrap(), theta_b76(&opt).unwrap());
        assert_eq!(w.rho_b76().unwrap(), rho_b76(&opt).unwrap());
    }

    // ---- Hull reference -------------------------------------------------

    /// Hull (10th ed., Ch. 18, ATM call on a futures contract):
    /// F=20, K=20, T≈4/12, r=0.09, σ=0.25 → call delta ≈ e^(-rT) · N(d1).
    /// d1 = σ√T / 2 ≈ 0.5·0.25·√(1/3) ≈ 0.0722, so N(d1) ≈ 0.5288 and
    /// Δ ≈ e^(-0.03) · 0.5288 ≈ 0.5132. Tolerance 1e-3.
    #[test]
    fn test_hull_atm_call_delta() {
        let opt = create_option(20.0, 20.0, dec!(0.09), 121.67, 0.25, OptionStyle::Call);
        let d = delta_b76(&opt).unwrap();
        let expected = dec!(0.5132);
        assert!(
            close(d, expected, dec!(1e-3)),
            "Hull ATM call Δ = {} expected ≈ {}",
            d,
            expected
        );
    }
}
