/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-04-27
******************************************************************************/

//! # Garman–Kohlhagen Greeks
//!
//! Closed-form Greeks for the Garman–Kohlhagen (1983) FX option pricing
//! model. Garman–Kohlhagen prices European options on a foreign-exchange
//! spot rate `S` (units of domestic per unit of foreign), with the foreign
//! currency earning interest at rate `r_f`. Structurally GK ≡ BSM with the
//! continuous dividend yield `q = r_f`, and the carry-adjusted form of
//! `d1`/`d2` is required (see *Note on BSM Greeks* below).
//!
//! ## FX field mapping
//!
//! | Field on `Options`  | FX interpretation              |
//! |---------------------|--------------------------------|
//! | `underlying_price`  | spot rate `S`                  |
//! | `strike_price`      | strike `K`                     |
//! | `risk_free_rate`    | domestic rate `r_d`            |
//! | `dividend_yield`    | foreign rate `r_f`             |
//!
//! ## Greek units
//!
//! Mirrors the BSM module:
//!
//! * `delta_gk` — per unit move in `S`, applies long/short sign.
//! * `gamma_gk` — per unit move in `S` (second order).
//! * `vega_gk`  — per **1%** change in volatility.
//! * `theta_gk` — per **calendar day** (annual figure ÷ 365).
//! * `rho_domestic_gk` — per **1%** change in `r_d`.
//! * `rho_foreign_gk`  — per **1%** change in `r_f`.
//!
//! ## Two rhos
//!
//! Unlike equity options, FX options carry two rate sensitivities:
//!
//! * `ρ_d = ∂price/∂r_d` — long calls are positive, long puts negative.
//! * `ρ_f = ∂price/∂r_f` — long calls are negative, long puts positive.
//!
//! ## Note on BSM Greeks
//!
//! These functions implement the carry-adjusted Garman–Kohlhagen formulas
//! directly using the `b = r_d − r_f` cost-of-carry term in `d1`/`d2`. They
//! do **not** delegate to [`crate::greeks::delta`] / [`crate::greeks::gamma`] / etc.,
//! because those functions currently call the unadjusted `d1` (passing
//! only `risk_free_rate`) before multiplying by `e^(-qT)` — a mismatch
//! between the d-values and the discount factor that yields incorrect
//! results when `dividend_yield ≠ 0`. The pricing kernels are unaffected
//! (they go through `calculate_d_values`, which does use `r − q`), and
//! fixing the BSM Greeks will be tracked separately.

use crate::Options;
use crate::error::PricingError;
use crate::error::greeks::GreeksError;
use crate::greeks::utils::{big_n, d1, d2, n};
use crate::model::decimal::{d_add, d_div, d_mul, d_sub};
use crate::model::types::{OptionStyle, OptionType, Side};
use positive::Positive;
use rust_decimal::{Decimal, MathematicalOps};
use tracing::{instrument, trace};

/// Reject any option type that is not European; Garman–Kohlhagen prices
/// European FX options only.
#[cold]
fn reject_non_european(label: &str) -> GreeksError {
    GreeksError::Pricing(Box::new(PricingError::unsupported_option_type(
        label,
        "Garman-Kohlhagen",
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

/// Cost of carry `b = r_d − r_f` for Garman–Kohlhagen.
fn cost_of_carry(option: &Options) -> Decimal {
    option.risk_free_rate - option.dividend_yield.to_dec()
}

/// Returns `Some(time_in_years)` when the option has not expired yet, or
/// `None` at expiration. Mirrors the `T == 0` short-circuit used by the
/// BSM Greeks in `src/greeks/equations.rs`, where the d-values are
/// undefined and the Greeks collapse to discrete intrinsic-state values.
fn time_to_expiry(option: &Options) -> Result<Option<Positive>, GreeksError> {
    let years = option.expiration_date.get_years()?;
    if years == Positive::ZERO {
        Ok(None)
    } else {
        Ok(Some(years))
    }
}

/// Closed-form delta value at expiration. Calls pay 1 ITM / 0 OTM; puts pay
/// −1 ITM / 0 OTM. Matches the `T == 0` branch of `crate::greeks::delta`.
fn delta_at_expiry(option: &Options) -> Decimal {
    let sign = side_sign(option);
    match option.option_style {
        OptionStyle::Call => {
            if option.underlying_price > option.strike_price {
                sign
            } else {
                Decimal::ZERO
            }
        }
        OptionStyle::Put => {
            if option.underlying_price < option.strike_price {
                -sign
            } else {
                Decimal::ZERO
            }
        }
    }
}

/// Computes (`d1`, `d2`) for Garman–Kohlhagen using `b = r_d − r_f` as the
/// drift term, mirroring the helper used by the GK pricing kernel.
fn calculate_d_values_gk(option: &Options) -> Result<(Decimal, Decimal), GreeksError> {
    let years = option.expiration_date.get_years()?;
    let b = cost_of_carry(option);
    let d1_value = d1(
        option.underlying_price,
        option.strike_price,
        b,
        years,
        option.implied_volatility,
    )?;
    let d2_value = d2(
        option.underlying_price,
        option.strike_price,
        b,
        years,
        option.implied_volatility,
    )?;
    Ok((d1_value, d2_value))
}

/// Computes the spot delta of an FX option under Garman–Kohlhagen.
///
/// # Formulas
///
/// - Call: `Δ_call = e^(-r_f·T) · N(d1)`
/// - Put:  `Δ_put  = -e^(-r_f·T) · N(-d1)`
///
/// where `d1 = [ln(S/K) + (r_d − r_f + σ²/2)·T] / (σ·√T)`.
///
/// Spot delta-parity: `Δ_call − Δ_put = e^(-r_f·T)`.
///
/// `Side::Long` keeps the sign; `Side::Short` flips it. Result is multiplied
/// by `option.quantity`.
///
/// # Errors
///
/// - [`GreeksError::Pricing`] wrapping [`PricingError::UnsupportedOptionType`]
///   for non-European option types.
/// - Propagates `GreeksError` from `d1` when expiration / volatility are
///   non-positive.
#[instrument(skip(option), fields(
    strike = %option.strike_price,
    style = ?option.option_style,
    side = ?option.side,
    r_d = %option.risk_free_rate,
    r_f = %option.dividend_yield,
))]
pub fn delta_gk(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let Some(t_pos) = time_to_expiry(option)? else {
        // Mirror BSM: at expiration the option is a binary intrinsic state.
        let qty = option.quantity.to_dec();
        return Ok(delta_at_expiry(option) * qty);
    };
    let t = t_pos.to_dec();
    let (d1_v, _d2) = calculate_d_values_gk(option)?;

    let r_f = option.dividend_yield.to_dec();
    let exp_neg_rf_t = (-r_f * t).exp();

    let raw = match option.option_style {
        OptionStyle::Call => d_mul(exp_neg_rf_t, big_n(d1_v)?, "greeks::gk::delta::call")?,
        OptionStyle::Put => -d_mul(exp_neg_rf_t, big_n(-d1_v)?, "greeks::gk::delta::put")?,
    };
    let signed = d_mul(side_sign(option), raw, "greeks::gk::delta::sign")?;
    let result = d_mul(
        signed,
        option.quantity.to_dec(),
        "greeks::gk::delta::quantity",
    )?;
    trace!(
        "GK Delta: S={}, K={}, r_d={}, r_f={}, d1={}, raw={}, result={}",
        option.underlying_price, option.strike_price, option.risk_free_rate, r_f, d1_v, raw, result
    );
    Ok(result)
}

/// Computes the gamma of an FX option under Garman–Kohlhagen.
///
/// # Formula
///
/// `Γ = e^(-r_f·T) · n(d1) / (S · σ · √T)` — identical for calls and puts,
/// independent of `Side`. Result is multiplied by `option.quantity`.
///
/// # Errors
///
/// - [`GreeksError::Pricing`] wrapping [`PricingError::UnsupportedOptionType`]
///   for non-European option types.
/// - Propagates `GreeksError` from `d1` when expiration / volatility are
///   non-positive.
#[instrument(skip(option), fields(strike = %option.strike_price))]
pub fn gamma_gk(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let Some(t) = time_to_expiry(option)? else {
        return Ok(Decimal::ZERO);
    };
    let (d1_v, _d2) = calculate_d_values_gk(option)?;

    let r_f = option.dividend_yield.to_dec();
    let exp_neg_rf_t = (-r_f * t.to_dec()).exp();
    let s = option.underlying_price.to_dec();
    let sigma = option.implied_volatility.to_dec();
    let sqrt_t = t.sqrt().to_dec();

    let denom = d_mul(
        s,
        d_mul(sigma, sqrt_t, "greeks::gk::gamma::sigma_sqrt_t")?,
        "greeks::gk::gamma::denom",
    )?;
    let numer = d_mul(exp_neg_rf_t, n(d1_v)?, "greeks::gk::gamma::numer")?;
    let raw = d_div(numer, denom, "greeks::gk::gamma::raw")?;

    let result = d_mul(raw, option.quantity.to_dec(), "greeks::gk::gamma::quantity")?;
    Ok(result)
}

/// Computes the vega of an FX option under Garman–Kohlhagen, per 1 % change
/// in volatility.
///
/// # Formula
///
/// `ν = S · e^(-r_f·T) · n(d1) · √T`, divided by 100. Identical for calls
/// and puts, independent of `Side`.
///
/// # Errors
///
/// - [`GreeksError::Pricing`] wrapping [`PricingError::UnsupportedOptionType`]
///   for non-European option types.
/// - Propagates `GreeksError` from `d1` when expiration / volatility are
///   non-positive.
#[instrument(skip(option), fields(strike = %option.strike_price))]
pub fn vega_gk(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let Some(t) = time_to_expiry(option)? else {
        return Ok(Decimal::ZERO);
    };
    let (d1_v, _d2) = calculate_d_values_gk(option)?;

    let r_f = option.dividend_yield.to_dec();
    let exp_neg_rf_t = (-r_f * t.to_dec()).exp();
    let s = option.underlying_price.to_dec();
    let sqrt_t = t.sqrt().to_dec();

    let leg1 = d_mul(s, exp_neg_rf_t, "greeks::gk::vega::s_df")?;
    let leg2 = d_mul(leg1, n(d1_v)?, "greeks::gk::vega::times_n")?;
    let raw = d_mul(leg2, sqrt_t, "greeks::gk::vega::times_sqrt_t")?;

    let weighted = d_mul(raw, option.quantity.to_dec(), "greeks::gk::vega::quantity")?;
    let result = d_div(weighted, Decimal::ONE_HUNDRED, "greeks::gk::vega::per_pct")?;
    Ok(result)
}

/// Computes the theta of an FX option under Garman–Kohlhagen, per calendar
/// day.
///
/// # Formulas (annual)
///
/// - Call: `Θ_call = -S·e^(-r_f T)·n(d1)·σ/(2√T) − r_d·K·e^(-r_d T)·N(d2) + r_f·S·e^(-r_f T)·N(d1)`
/// - Put:  `Θ_put  = -S·e^(-r_f T)·n(d1)·σ/(2√T) + r_d·K·e^(-r_d T)·N(-d2) − r_f·S·e^(-r_f T)·N(-d1)`
///
/// Annual figure divided by 365 to express decay per calendar day,
/// multiplied by `option.quantity`.
///
/// # Errors
///
/// - [`GreeksError::Pricing`] wrapping [`PricingError::UnsupportedOptionType`]
///   for non-European option types.
/// - Propagates `GreeksError` from `d1` / `d2` when expiration / volatility
///   are non-positive.
#[instrument(skip(option), fields(strike = %option.strike_price))]
pub fn theta_gk(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let Some(t) = time_to_expiry(option)? else {
        return Ok(Decimal::ZERO);
    };
    let (d1_v, d2_v) = calculate_d_values_gk(option)?;

    let r_d = option.risk_free_rate;
    let r_f = option.dividend_yield.to_dec();
    let s = option.underlying_price.to_dec();
    let k = option.strike_price.to_dec();
    let sigma = option.implied_volatility.to_dec();
    let sqrt_t = t.sqrt().to_dec();
    let exp_neg_rd_t = (-r_d * t.to_dec()).exp();
    let exp_neg_rf_t = (-r_f * t.to_dec()).exp();

    // Volatility decay term: -S·e^(-r_f T)·n(d1)·σ/(2√T)
    let two_sqrt_t = d_mul(Decimal::TWO, sqrt_t, "greeks::gk::theta::two_sqrt_t")?;
    let s_df_f = d_mul(s, exp_neg_rf_t, "greeks::gk::theta::s_df_f")?;
    let s_df_f_n = d_mul(s_df_f, n(d1_v)?, "greeks::gk::theta::s_df_f_n")?;
    let s_df_f_n_sigma = d_mul(s_df_f_n, sigma, "greeks::gk::theta::s_df_f_n_sigma")?;
    let common = -d_div(s_df_f_n_sigma, two_sqrt_t, "greeks::gk::theta::common")?;

    // Rate-dependent legs (annual)
    let r_d_k_df_d = d_mul(
        r_d,
        d_mul(k, exp_neg_rd_t, "greeks::gk::theta::k_df_d")?,
        "greeks::gk::theta::r_d_k_df_d",
    )?;
    let r_f_s_df_f = d_mul(r_f, s_df_f, "greeks::gk::theta::r_f_s_df_f")?;

    let annual = match option.option_style {
        OptionStyle::Call => {
            // Θ_call = common − r_d·K·e^(-r_d T)·N(d2) + r_f·S·e^(-r_f T)·N(d1)
            let minus = d_mul(r_d_k_df_d, big_n(d2_v)?, "greeks::gk::theta::call::minus")?;
            let plus = d_mul(r_f_s_df_f, big_n(d1_v)?, "greeks::gk::theta::call::plus")?;
            let common_plus = d_add(common, plus, "greeks::gk::theta::call::common_plus")?;
            d_sub(common_plus, minus, "greeks::gk::theta::call::sum")?
        }
        OptionStyle::Put => {
            // Θ_put = common + r_d·K·e^(-r_d T)·N(-d2) − r_f·S·e^(-r_f T)·N(-d1)
            let plus = d_mul(r_d_k_df_d, big_n(-d2_v)?, "greeks::gk::theta::put::plus")?;
            let minus = d_mul(r_f_s_df_f, big_n(-d1_v)?, "greeks::gk::theta::put::minus")?;
            let common_plus = d_add(common, plus, "greeks::gk::theta::put::common_plus")?;
            d_sub(common_plus, minus, "greeks::gk::theta::put::sum")?
        }
    };

    let weighted = d_mul(
        annual,
        option.quantity.to_dec(),
        "greeks::gk::theta::quantity",
    )?;
    let result = d_div(weighted, Decimal::from(365), "greeks::gk::theta::per_day")?;
    Ok(result)
}

/// Computes the **domestic rho** of an FX option under Garman–Kohlhagen,
/// per 1 % change in the domestic risk-free rate `r_d`.
///
/// # Formulas
///
/// - Call: `ρ_d^call =  K · T · e^(-r_d·T) · N(d2)`
/// - Put:  `ρ_d^put  = -K · T · e^(-r_d·T) · N(-d2)`
///
/// Annual figure divided by 100, multiplied by `option.quantity`. Long
/// calls have positive domestic rho; long puts have negative.
///
/// # Errors
///
/// - [`GreeksError::Pricing`] wrapping [`PricingError::UnsupportedOptionType`]
///   for non-European option types.
/// - Propagates `GreeksError` from `d2` when expiration / volatility are
///   non-positive.
#[instrument(skip(option), fields(strike = %option.strike_price))]
pub fn rho_domestic_gk(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let Some(t) = time_to_expiry(option)? else {
        return Ok(Decimal::ZERO);
    };
    let (_d1, d2_v) = calculate_d_values_gk(option)?;

    let r_d = option.risk_free_rate;
    let k = option.strike_price.to_dec();
    let exp_neg_rd_t = (-r_d * t.to_dec()).exp();

    let base = d_mul(
        k,
        d_mul(t.to_dec(), exp_neg_rd_t, "greeks::gk::rho_d::t_df")?,
        "greeks::gk::rho_d::base",
    )?;
    let raw = match option.option_style {
        OptionStyle::Call => d_mul(base, big_n(d2_v)?, "greeks::gk::rho_d::call")?,
        OptionStyle::Put => -d_mul(base, big_n(-d2_v)?, "greeks::gk::rho_d::put")?,
    };
    let weighted = d_mul(raw, option.quantity.to_dec(), "greeks::gk::rho_d::quantity")?;
    let result = d_div(weighted, Decimal::ONE_HUNDRED, "greeks::gk::rho_d::per_pct")?;
    Ok(result)
}

/// Computes the **foreign rho** of an FX option under Garman–Kohlhagen,
/// per 1 % change in the foreign risk-free rate `r_f`.
///
/// # Formulas
///
/// - Call: `ρ_f^call = -S · T · e^(-r_f·T) · N(d1)`
/// - Put:  `ρ_f^put  =  S · T · e^(-r_f·T) · N(-d1)`
///
/// Annual figure divided by 100, multiplied by `option.quantity`. Long
/// calls have negative foreign rho; long puts have positive.
///
/// # Errors
///
/// - [`GreeksError::Pricing`] wrapping [`PricingError::UnsupportedOptionType`]
///   for non-European option types.
/// - Propagates `GreeksError` from `d1` when expiration / volatility are
///   non-positive.
#[instrument(skip(option), fields(strike = %option.strike_price))]
pub fn rho_foreign_gk(option: &Options) -> Result<Decimal, GreeksError> {
    ensure_european(option)?;
    let Some(t) = time_to_expiry(option)? else {
        return Ok(Decimal::ZERO);
    };
    let (d1_v, _d2) = calculate_d_values_gk(option)?;

    let r_f = option.dividend_yield.to_dec();
    let s = option.underlying_price.to_dec();
    let exp_neg_rf_t = (-r_f * t.to_dec()).exp();

    let base = d_mul(
        s,
        d_mul(t.to_dec(), exp_neg_rf_t, "greeks::gk::rho_f::t_df")?,
        "greeks::gk::rho_f::base",
    )?;
    let raw = match option.option_style {
        OptionStyle::Call => -d_mul(base, big_n(d1_v)?, "greeks::gk::rho_f::call")?,
        OptionStyle::Put => d_mul(base, big_n(-d1_v)?, "greeks::gk::rho_f::put")?,
    };
    let weighted = d_mul(raw, option.quantity.to_dec(), "greeks::gk::rho_f::quantity")?;
    let result = d_div(weighted, Decimal::ONE_HUNDRED, "greeks::gk::rho_f::per_pct")?;
    Ok(result)
}

/// Trait that exposes the Garman–Kohlhagen Greeks for any type that can
/// produce an [`Options`] reference.
///
/// Mirrors the [`crate::pricing::GarmanKohlhagen`] pricing trait.
/// Implementors only need to provide [`GarmanKohlhagenGreeks::get_option`];
/// default implementations route to the free-function Greeks above.
pub trait GarmanKohlhagenGreeks {
    /// Returns the option to compute Greeks for.
    fn get_option(&self) -> Result<&Options, GreeksError>;

    /// Spot delta — see [`delta_gk`].
    fn delta_gk(&self) -> Result<Decimal, GreeksError> {
        delta_gk(self.get_option()?)
    }

    /// Gamma — see [`gamma_gk`].
    fn gamma_gk(&self) -> Result<Decimal, GreeksError> {
        gamma_gk(self.get_option()?)
    }

    /// Vega — see [`vega_gk`].
    fn vega_gk(&self) -> Result<Decimal, GreeksError> {
        vega_gk(self.get_option()?)
    }

    /// Theta — see [`theta_gk`].
    fn theta_gk(&self) -> Result<Decimal, GreeksError> {
        theta_gk(self.get_option()?)
    }

    /// Domestic rho — see [`rho_domestic_gk`].
    fn rho_domestic_gk(&self) -> Result<Decimal, GreeksError> {
        rho_domestic_gk(self.get_option()?)
    }

    /// Foreign rho — see [`rho_foreign_gk`].
    fn rho_foreign_gk(&self) -> Result<Decimal, GreeksError> {
        rho_foreign_gk(self.get_option()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::greeks::{delta, gamma, vega};
    use crate::pricing::garman_kohlhagen;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn create_fx_option(
        s: f64,
        k: f64,
        r_d: Decimal,
        r_f: f64,
        t_days: f64,
        sigma: f64,
        style: OptionStyle,
    ) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "EURUSD".to_string(),
            pos_or_panic!(k),
            ExpirationDate::Days(pos_or_panic!(t_days)),
            pos_or_panic!(sigma),
            Positive::ONE,
            pos_or_panic!(s),
            r_d,
            style,
            pos_or_panic!(r_f),
            None,
        )
    }

    fn close(a: Decimal, b: Decimal, tol: Decimal) -> bool {
        (a - b).abs() < tol
    }

    // ---- delta range and parity ----------------------------------------

    #[test]
    fn test_delta_call_in_range() {
        let opt = create_fx_option(1.10, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Call);
        let years = opt.expiration_date.get_years().unwrap().to_dec();
        let upper = (-opt.dividend_yield.to_dec() * years).exp();
        let d = delta_gk(&opt).expect("delta call");
        assert!(
            d > Decimal::ZERO && d < upper,
            "Δ_call = {}, expected ∈ (0, {})",
            d,
            upper
        );
    }

    #[test]
    fn test_delta_put_in_range() {
        let opt = create_fx_option(1.10, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Put);
        let years = opt.expiration_date.get_years().unwrap().to_dec();
        let lower = -((-opt.dividend_yield.to_dec() * years).exp());
        let d = delta_gk(&opt).expect("delta put");
        assert!(
            d < Decimal::ZERO && d > lower,
            "Δ_put = {}, expected ∈ ({}, 0)",
            d,
            lower
        );
    }

    /// FX spot delta-parity: `Δ_call − Δ_put = e^(-r_f·T)`.
    #[test]
    fn test_spot_delta_parity() {
        let s = 1.10;
        let k = 1.10;
        let r_d = dec!(0.04);
        let r_f = 0.02;
        let t_days = 90.0;
        let sigma = 0.10;

        let call = create_fx_option(s, k, r_d, r_f, t_days, sigma, OptionStyle::Call);
        let put = create_fx_option(s, k, r_d, r_f, t_days, sigma, OptionStyle::Put);

        let dc = delta_gk(&call).unwrap();
        let dp = delta_gk(&put).unwrap();

        let years = call.expiration_date.get_years().unwrap().to_dec();
        let expected = (-call.dividend_yield.to_dec() * years).exp();
        assert!(
            close(dc - dp, expected, dec!(1e-9)),
            "Δc − Δp = {} vs e^(-r_f·T) = {}",
            dc - dp,
            expected
        );
    }

    // ---- positivity / symmetry -----------------------------------------

    #[test]
    fn test_gamma_positive_call_and_put() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for s in [0.95, 1.00, 1.10, 1.20] {
                let opt = create_fx_option(s, 1.10, dec!(0.04), 0.02, 90.0, 0.10, style);
                let g = gamma_gk(&opt).expect("gamma");
                assert!(g > Decimal::ZERO, "style={:?} S={} Γ={}", style, s, g);
            }
        }
    }

    #[test]
    fn test_vega_positive_call_and_put() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for s in [0.95, 1.00, 1.10, 1.20] {
                let opt = create_fx_option(s, 1.10, dec!(0.04), 0.02, 90.0, 0.10, style);
                let v = vega_gk(&opt).expect("vega");
                assert!(v > Decimal::ZERO, "style={:?} S={} ν={}", style, s, v);
            }
        }
    }

    #[test]
    fn test_gamma_call_equals_put() {
        let call = create_fx_option(1.12, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Call);
        let put = create_fx_option(1.12, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Put);
        assert_eq!(gamma_gk(&call).unwrap(), gamma_gk(&put).unwrap());
    }

    #[test]
    fn test_vega_call_equals_put() {
        let call = create_fx_option(1.12, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Call);
        let put = create_fx_option(1.12, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Put);
        assert_eq!(vega_gk(&call).unwrap(), vega_gk(&put).unwrap());
    }

    // ---- rho signs -----------------------------------------------------

    #[test]
    fn test_rho_signs_long_call() {
        let opt = create_fx_option(1.10, 1.10, dec!(0.04), 0.02, 180.0, 0.10, OptionStyle::Call);
        let rd = rho_domestic_gk(&opt).unwrap();
        let rf = rho_foreign_gk(&opt).unwrap();
        assert!(
            rd > Decimal::ZERO,
            "long call ρ_d should be positive: {}",
            rd
        );
        assert!(
            rf < Decimal::ZERO,
            "long call ρ_f should be negative: {}",
            rf
        );
    }

    #[test]
    fn test_rho_signs_long_put() {
        let opt = create_fx_option(1.10, 1.10, dec!(0.04), 0.02, 180.0, 0.10, OptionStyle::Put);
        let rd = rho_domestic_gk(&opt).unwrap();
        let rf = rho_foreign_gk(&opt).unwrap();
        assert!(
            rd < Decimal::ZERO,
            "long put ρ_d should be negative: {}",
            rd
        );
        assert!(
            rf > Decimal::ZERO,
            "long put ρ_f should be positive: {}",
            rf
        );
    }

    // ---- BSM equivalence (only meaningful when q = 0) ------------------

    /// When `dividend_yield == 0`, GK Greeks must agree with the existing
    /// BSM Greeks bit-exactly. For `dividend_yield > 0` the BSM Greeks have
    /// a known issue (d1 not carry-adjusted) — covered separately.
    #[test]
    fn test_matches_bsm_greeks_when_q_is_zero() {
        let opt = create_fx_option(
            100.0,
            100.0,
            dec!(0.05),
            0.0,
            180.0,
            0.20,
            OptionStyle::Call,
        );
        // delta and gamma agree exactly when q=0.
        assert_eq!(delta_gk(&opt).unwrap(), delta(&opt).unwrap());
        assert_eq!(gamma_gk(&opt).unwrap(), gamma(&opt).unwrap());
        assert_eq!(vega_gk(&opt).unwrap(), vega(&opt).unwrap());
    }

    // ---- theta vs numerical differentiation ----------------------------

    /// Compare analytic theta against a one-day price difference.
    /// Numerical Θ ≈ price(T − 1d) − price(T), which is the daily decay
    /// the analytic formula reports. Tolerance 5e-3.
    #[test]
    fn test_theta_matches_numerical_differentiation() {
        let s = 1.10;
        let k = 1.10;
        let r_d = dec!(0.04);
        let r_f = 0.02;
        let sigma = 0.10;
        let t_days = 90.0;

        let opt = create_fx_option(s, k, r_d, r_f, t_days, sigma, OptionStyle::Call);
        let opt_minus = create_fx_option(s, k, r_d, r_f, t_days - 1.0, sigma, OptionStyle::Call);

        let p_now = garman_kohlhagen(&opt).unwrap();
        let p_back = garman_kohlhagen(&opt_minus).unwrap();
        let numerical_theta = p_back - p_now;

        let analytic = theta_gk(&opt).unwrap();
        assert!(
            close(analytic, numerical_theta, dec!(5e-3)),
            "analytic Θ = {} vs numerical = {} (diff = {})",
            analytic,
            numerical_theta,
            (analytic - numerical_theta).abs()
        );
    }

    // ---- FX delta reference --------------------------------------------

    /// Reference FX-call delta: S=0.98, K=1.00, r_d=0.05, r_f=0.04, T=4/12,
    /// σ=0.10. Hand-computed:
    ///   d1 = [ln(0.98) + (0.05 − 0.04 + 0.005) · 1/3] / (0.10·√(1/3)) ≈ -0.2633
    ///   N(d1) ≈ 0.39614
    ///   Δ_call = e^(-0.04/3) · 0.39614 ≈ 0.3909
    /// Tolerance 1e-3 (Garman–Kohlhagen ≡ BSM-with-`q=r_f` carry-adjusted).
    #[test]
    fn test_fx_call_delta_reference() {
        let opt = create_fx_option(
            0.98,
            1.00,
            dec!(0.05),
            0.04,
            121.6667,
            0.10,
            OptionStyle::Call,
        );
        let d = delta_gk(&opt).unwrap();
        let expected = dec!(0.3909);
        assert!(
            close(d, expected, dec!(1e-3)),
            "FX call Δ = {} expected ≈ {}",
            d,
            expected
        );
    }

    // ---- error paths ---------------------------------------------------

    #[test]
    fn test_zero_volatility_returns_error() {
        let opt = create_fx_option(1.10, 1.10, dec!(0.04), 0.02, 90.0, 0.0, OptionStyle::Call);
        assert!(delta_gk(&opt).is_err());
        assert!(gamma_gk(&opt).is_err());
        assert!(vega_gk(&opt).is_err());
        assert!(theta_gk(&opt).is_err());
        assert!(rho_domestic_gk(&opt).is_err());
        assert!(rho_foreign_gk(&opt).is_err());
    }

    #[test]
    fn test_unsupported_american_returns_error() {
        let mut opt = create_fx_option(1.10, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Call);
        opt.option_type = OptionType::American;
        for r in [
            delta_gk(&opt),
            gamma_gk(&opt),
            vega_gk(&opt),
            theta_gk(&opt),
            rho_domestic_gk(&opt),
            rho_foreign_gk(&opt),
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
        let mut opt = create_fx_option(1.10, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Call);
        opt.option_type = OptionType::Bermuda {
            exercise_dates: vec![],
        };
        assert!(matches!(delta_gk(&opt), Err(GreeksError::Pricing(_))));
    }

    // ---- side and quantity --------------------------------------------

    #[test]
    fn test_side_short_negates_delta() {
        let long = create_fx_option(1.12, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Call);
        let mut short = long.clone();
        short.side = Side::Short;
        assert_eq!(delta_gk(&long).unwrap(), -delta_gk(&short).unwrap());
    }

    #[test]
    fn test_quantity_scales_linearly() {
        let mut opt = create_fx_option(1.12, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Call);
        let d1_value = delta_gk(&opt).unwrap();
        opt.quantity = pos_or_panic!(4.0);
        let d4 = delta_gk(&opt).unwrap();
        assert!(close(d4, d1_value * Decimal::from(4), dec!(1e-15)));
    }

    // ---- trait --------------------------------------------------------

    #[test]
    fn test_garman_kohlhagen_greeks_trait() {
        struct FxQuote(Options);
        impl GarmanKohlhagenGreeks for FxQuote {
            fn get_option(&self) -> Result<&Options, GreeksError> {
                Ok(&self.0)
            }
        }

        let opt = create_fx_option(1.12, 1.10, dec!(0.04), 0.02, 90.0, 0.10, OptionStyle::Call);
        let q = FxQuote(opt.clone());
        assert_eq!(q.delta_gk().unwrap(), delta_gk(&opt).unwrap());
        assert_eq!(q.gamma_gk().unwrap(), gamma_gk(&opt).unwrap());
        assert_eq!(q.vega_gk().unwrap(), vega_gk(&opt).unwrap());
        assert_eq!(q.theta_gk().unwrap(), theta_gk(&opt).unwrap());
        assert_eq!(q.rho_domestic_gk().unwrap(), rho_domestic_gk(&opt).unwrap());
        assert_eq!(q.rho_foreign_gk().unwrap(), rho_foreign_gk(&opt).unwrap());
    }

    // ---- T = 0 (expiration) handling, mirrors BSM Greeks --------------

    #[test]
    fn test_t_zero_delta_call_long_itm() {
        let opt = create_fx_option(1.20, 1.10, dec!(0.04), 0.02, 0.0, 0.10, OptionStyle::Call);
        assert_eq!(delta_gk(&opt).unwrap(), Decimal::ONE);
    }

    #[test]
    fn test_t_zero_delta_call_long_otm() {
        let opt = create_fx_option(1.05, 1.10, dec!(0.04), 0.02, 0.0, 0.10, OptionStyle::Call);
        assert_eq!(delta_gk(&opt).unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_t_zero_delta_put_long_itm() {
        let opt = create_fx_option(1.05, 1.10, dec!(0.04), 0.02, 0.0, 0.10, OptionStyle::Put);
        assert_eq!(delta_gk(&opt).unwrap(), Decimal::NEGATIVE_ONE);
    }

    #[test]
    fn test_t_zero_delta_short_call_itm() {
        let mut opt = create_fx_option(1.20, 1.10, dec!(0.04), 0.02, 0.0, 0.10, OptionStyle::Call);
        opt.side = Side::Short;
        assert_eq!(delta_gk(&opt).unwrap(), Decimal::NEGATIVE_ONE);
    }

    #[test]
    fn test_t_zero_other_greeks_zero() {
        let opt = create_fx_option(1.10, 1.10, dec!(0.04), 0.02, 0.0, 0.10, OptionStyle::Call);
        assert_eq!(gamma_gk(&opt).unwrap(), Decimal::ZERO);
        assert_eq!(vega_gk(&opt).unwrap(), Decimal::ZERO);
        assert_eq!(theta_gk(&opt).unwrap(), Decimal::ZERO);
        assert_eq!(rho_domestic_gk(&opt).unwrap(), Decimal::ZERO);
        assert_eq!(rho_foreign_gk(&opt).unwrap(), Decimal::ZERO);
    }
}
