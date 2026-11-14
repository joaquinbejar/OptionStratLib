//! Everlasting Option Pricing Module
//!
//! An everlasting option is a perpetual contract on an option payoff. It has
//! no expiry: the long side pays the short side funding at the rate
//! `κ·(F − φ(S))` per year, where `F` is the contract price, `φ(S)` the payoff
//! at the current underlying price and `κ` the funding rate. With that rule
//! the price is the expected payoff at an independent random time,
//! `F = E[φ(S_τ)]` with `τ` exponential of rate `κ`, so the contract behaves
//! like an option whose maturity is random with mean `1/κ` years.
//!
//! # Ladders
//!
//! A ladder chains such contracts: rung 1 is funded against the payoff, rung
//! `j` against the price of rung `j − 1`, all at the same rate, so the long
//! side of rung `j` pays `κ·(F_j − F_{j−1})` per year. The maturity of rung
//! `j` is then the sum of `j` independent exponential times, an Erlang law of
//! shape `j` and rate `κ`: its mean is `j/κ` years and its dispersion
//! (standard deviation over mean) is `1/√j`, so a higher rung is closer to a
//! dated option.
//!
//! # Model
//!
//! The underlying is a geometric Brownian motion with volatility `σ` (per
//! square root of a year) and drift `μ` (per year) under the pricing measure.
//! The contract is margined and settled through funding, so prices are not
//! discounted: use `μ = 0` when the underlying is a futures or a perpetual
//! mark price, and the cost of carry otherwise.
//!
//! # Pricing Formula
//!
//! Let `a > 1` and `−b < 0` be the roots of `½σ²β(β − 1) + μβ = κ`. The
//! log-return at the random maturity of rung `j` is the difference of two
//! independent Erlang variables of shape `j` and rates `a` and `b`. With
//! `k = ln(K/S) ≥ 0`, `E_p(z) = e^{−z}·Σ_{i<p} z^i/i!` and
//! `c_m = C(j−1+m, m)·a^j·b^j/(a+b)^{j+m}`, the call is
//!
//! ```text
//! C_j = S·Δ_j − K·Σ_{m=0}^{j−1} c_m·E_{j−m}(a·k)/a^{j−m}
//! Δ_j = Σ_{m=0}^{j−1} c_m·E_{j−m}((a−1)·k)/(a−1)^{j−m}
//! Γ_j = e^{−(a−1)k}/S · Σ_{m=0}^{j−1} c_m·k^{j−m−1}/(j−m−1)!
//! ```
//!
//! For `K < S` the same expressions with `b` in place of `a` give the put,
//! and the other side follows from parity,
//! `C_j − P_j = S·(κ/(κ − μ))^j − K`. For `j = 1` and `μ = 0` the price at
//! the money is `K/√(1 + 8κ/σ²)`.
//!
//! The sensitivities use three identities of the ladder, exact under this
//! model:
//!
//! - funding is the generator applied to the price,
//!   `κ·(F_j − F_{j−1}) = ½σ²S²·Γ_j + μ·S·Δ_j`;
//! - `κ·∂F_j/∂κ = j·(F_j − F_{j+1})`;
//! - `σ·∂F_j/∂σ = 2j·[(F_{j+1} − F_j) − (μ/κ)·S·Δ_{j+1}]`.
//!
//! # Numerical Method
//!
//! The kernel runs in `f64` on logarithms of the binomial and power factors,
//! in `O(j)` operations and without allocation. Against an independent
//! quadrature of the Erlang mixture of Black-Scholes values, the relative
//! error is below `1e-13` for ordinary parameters and below `1e-11` up to
//! rung [`EVERLASTING_MAX_RUNG`] with `a` in the thousands.
//!
//! # Example
//!
//! ```rust
//! use optionstratlib_core::model::Positive;
//! use optionstratlib_core::model::types::OptionStyle;
//! use optionstratlib_pricing::pricing::{
//!     EverlastingParams, everlasting_funding, everlasting_price,
//! };
//! use rust_decimal_macros::dec;
//! use std::num::NonZeroU32;
//!
//! // A call at the money with 50% volatility and a funding rate of 12 per
//! // year: a mean maturity of one month.
//! let params = EverlastingParams::try_new(
//!     Positive::HUNDRED,
//!     Positive::HUNDRED,
//!     Positive::new_decimal(dec!(0.5))?,
//!     dec!(0),
//!     Positive::new_decimal(dec!(12))?,
//!     NonZeroU32::MIN,
//!     OptionStyle::Call,
//! )?;
//! let price = everlasting_price(&params)?;
//! let funding = everlasting_funding(&params)?;
//! // Rung 1 is funded against the payoff, which is zero at the money.
//! assert!((funding - dec!(12) * price).abs() < dec!(0.000001));
//! # Ok::<(), optionstratlib_pricing::error::PricingError>(())
//! ```

use crate::error::PricingError;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{decimal_to_f64, finite_decimal};
use optionstratlib_core::model::types::OptionStyle;
use rust_decimal::Decimal;
use statrs::distribution::{ContinuousCDF, Normal};
use std::num::NonZeroU32;

/// Highest ladder rung the closed form accepts. The kernel is validated
/// against quadrature up to this rung.
pub const EVERLASTING_MAX_RUNG: u32 = 64;

/// Method name carried by the errors of this module.
const METHOD: &str = "everlasting";

/// Inputs of an everlasting option on rung `rung` of a ladder, under
/// geometric Brownian motion.
///
/// Built with [`EverlastingParams::try_new`], which checks the domain of
/// every field; the fields are private so a value of this type is always
/// priceable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EverlastingParams {
    underlying_price: Positive,
    strike_price: Positive,
    volatility: Positive,
    drift: Decimal,
    funding_rate: Positive,
    rung: NonZeroU32,
    option_style: OptionStyle,
}

impl EverlastingParams {
    /// Validates and stores the inputs of an everlasting option.
    ///
    /// # Arguments
    ///
    /// * `underlying_price` - Current price `S` of the underlying.
    /// * `strike_price` - Strike `K` of the payoff.
    /// * `volatility` - Volatility `σ`, per square root of a year.
    /// * `drift` - Drift `μ` of the underlying under the pricing measure, per
    ///   year. Zero for a futures or a perpetual mark price.
    /// * `funding_rate` - Funding rate `κ`, per year. Rung `j` has a mean
    ///   maturity of `j/κ` years.
    /// * `rung` - Rung of the ladder; `1` is the plain everlasting option.
    /// * `option_style` - Call or put.
    ///
    /// # Errors
    ///
    /// [`PricingError::InvalidParameter`] when the underlying price, the
    /// strike, the volatility or the funding rate is zero, when the funding
    /// rate does not exceed the drift (the expected underlying price at the
    /// random maturity is then infinite), or when the rung exceeds
    /// [`EVERLASTING_MAX_RUNG`].
    pub fn try_new(
        underlying_price: Positive,
        strike_price: Positive,
        volatility: Positive,
        drift: Decimal,
        funding_rate: Positive,
        rung: NonZeroU32,
        option_style: OptionStyle,
    ) -> Result<Self, PricingError> {
        reject_zero("underlying_price", underlying_price)?;
        reject_zero("strike_price", strike_price)?;
        reject_zero("volatility", volatility)?;
        reject_zero("funding_rate", funding_rate)?;
        if funding_rate.to_dec() <= drift {
            return Err(PricingError::invalid_parameter(
                "funding_rate",
                funding_rate.to_dec(),
                "must exceed the drift",
            ));
        }
        if rung.get() > EVERLASTING_MAX_RUNG {
            return Err(PricingError::invalid_parameter(
                "rung",
                Decimal::from(rung.get()),
                "exceeds EVERLASTING_MAX_RUNG",
            ));
        }
        Ok(Self {
            underlying_price,
            strike_price,
            volatility,
            drift,
            funding_rate,
            rung,
            option_style,
        })
    }

    /// Current price `S` of the underlying.
    #[must_use]
    #[inline]
    pub const fn underlying_price(&self) -> Positive {
        self.underlying_price
    }

    /// Strike `K` of the payoff.
    #[must_use]
    #[inline]
    pub const fn strike_price(&self) -> Positive {
        self.strike_price
    }

    /// Volatility `σ`, per square root of a year.
    #[must_use]
    #[inline]
    pub const fn volatility(&self) -> Positive {
        self.volatility
    }

    /// Drift `μ` of the underlying under the pricing measure, per year.
    #[must_use]
    #[inline]
    pub const fn drift(&self) -> Decimal {
        self.drift
    }

    /// Funding rate `κ`, per year.
    #[must_use]
    #[inline]
    pub const fn funding_rate(&self) -> Positive {
        self.funding_rate
    }

    /// Rung of the ladder; `1` is the plain everlasting option.
    #[must_use]
    #[inline]
    pub const fn rung(&self) -> NonZeroU32 {
        self.rung
    }

    /// Call or put.
    #[must_use]
    #[inline]
    pub const fn option_style(&self) -> OptionStyle {
        self.option_style
    }

    /// The `f64` inputs of the kernel.
    fn kernel_inputs(&self) -> Result<KernelInputs, PricingError> {
        Ok(KernelInputs {
            spot: decimal_to_f64(self.underlying_price.to_dec())?,
            strike: decimal_to_f64(self.strike_price.to_dec())?,
            sigma: decimal_to_f64(self.volatility.to_dec())?,
            mu: decimal_to_f64(self.drift)?,
            kappa: decimal_to_f64(self.funding_rate.to_dec())?,
            rung: self.rung.get(),
            style: self.option_style,
        })
    }
}

/// Price of an everlasting option, per unit of underlying, for the long side.
///
/// # Errors
///
/// - [`PricingError::Decimal`] when an input has no `f64` form.
/// - [`PricingError::NonFinite`] when the `f64` kernel leaves its range, for
///   example a funding rate so close to the drift that the price overflows.
pub fn everlasting_price(params: &EverlastingParams) -> Result<Decimal, PricingError> {
    let values = ladder_values(&params.kernel_inputs()?);
    // Far out of the money the two terms of the price cancel to rounding
    // error; a price is never negative.
    to_decimal("pricing::everlasting::price", values.price.max(0.0))
}

/// Delta of an everlasting option: `∂F/∂S`, in units of underlying.
///
/// # Errors
///
/// The errors of [`everlasting_price`].
pub fn everlasting_delta(params: &EverlastingParams) -> Result<Decimal, PricingError> {
    let values = ladder_values(&params.kernel_inputs()?);
    to_decimal("pricing::everlasting::delta", values.delta)
}

/// Gamma of an everlasting option: `∂²F/∂S²`, per unit of underlying price.
///
/// # Errors
///
/// The errors of [`everlasting_price`].
pub fn everlasting_gamma(params: &EverlastingParams) -> Result<Decimal, PricingError> {
    let values = ladder_values(&params.kernel_inputs()?);
    to_decimal("pricing::everlasting::gamma", values.gamma)
}

/// Funding the long side pays the short side, per unit of underlying and per
/// year: `κ·(F_j − F_{j−1})`, with the payoff in place of `F_0`.
///
/// It is computed as the generator applied to the price,
/// `½σ²S²·Γ + μ·S·Δ`, which is the same quantity without the cancellation of
/// two close prices. With zero drift it is never negative.
///
/// # Errors
///
/// The errors of [`everlasting_price`].
pub fn everlasting_funding(params: &EverlastingParams) -> Result<Decimal, PricingError> {
    let inputs = params.kernel_inputs()?;
    let values = ladder_values(&inputs);
    let funding = 0.5 * inputs.sigma * inputs.sigma * inputs.spot * inputs.spot * values.gamma
        + inputs.mu * inputs.spot * values.delta;
    to_decimal("pricing::everlasting::funding", funding)
}

/// Vega of an everlasting option: `∂F/∂σ`, per unit of volatility (a change
/// of volatility from 0.50 to 0.51 moves the price by about `0.01·vega`).
///
/// It is the ladder identity
/// `σ·∂F_j/∂σ = 2j·[(F_{j+1} − F_j) − (μ/κ)·S·Δ_{j+1}]`, so no derivative is
/// taken numerically.
///
/// # Errors
///
/// The errors of [`everlasting_price`].
pub fn everlasting_vega(params: &EverlastingParams) -> Result<Decimal, PricingError> {
    let inputs = params.kernel_inputs()?;
    let here = ladder_values(&inputs);
    let next = ladder_values(&inputs.next_rung()?);
    let rung = f64::from(inputs.rung);
    let vega = 2.0 * rung / inputs.sigma
        * ((next.price - here.price) - inputs.mu / inputs.kappa * inputs.spot * next.delta);
    to_decimal("pricing::everlasting::vega", vega)
}

/// Sensitivity of an everlasting option to its funding rate: `∂F/∂κ`, per
/// unit of rate (per year).
///
/// It is the ladder identity `κ·∂F_j/∂κ = j·(F_j − F_{j+1})`. A higher
/// funding rate shortens the maturity, so the value is negative whenever the
/// price grows with maturity.
///
/// # Errors
///
/// The errors of [`everlasting_price`].
pub fn everlasting_funding_rate_sensitivity(
    params: &EverlastingParams,
) -> Result<Decimal, PricingError> {
    let inputs = params.kernel_inputs()?;
    let here = ladder_values(&inputs);
    let next = ladder_values(&inputs.next_rung()?);
    let sensitivity = f64::from(inputs.rung) * (here.price - next.price) / inputs.kappa;
    to_decimal(
        "pricing::everlasting::funding_rate_sensitivity",
        sensitivity,
    )
}

/// Mean maturity of rung `rung` of a ladder with funding rate
/// `funding_rate`: `j/κ`, in years.
///
/// # Errors
///
/// [`PricingError::InvalidParameter`] when the funding rate is zero, and
/// [`PricingError::Positive`] when the quotient leaves the `Decimal` range.
pub fn everlasting_mean_maturity(
    funding_rate: Positive,
    rung: NonZeroU32,
) -> Result<Positive, PricingError> {
    reject_zero("funding_rate", funding_rate)?;
    Ok(Positive::new_decimal(Decimal::from(rung.get()))?.checked_div(&funding_rate)?)
}

/// Equivalent maturity of rung `rung`, in years: the maturity `T` at which a
/// European option with the same volatility has the same price at the money,
/// with zero drift.
///
/// At the money and without drift the European price is
/// `K·(2Φ(σ√T/2) − 1)`, so `σ√T = 2Φ⁻¹(½ + ½·F/K)`. The equivalent maturity
/// is shorter than the mean maturity `j/κ`: for the plain everlasting option
/// it tends to `π/(4κ)` when `σ²/κ` is small, and the ratio to the mean tends
/// to one as the rung grows.
///
/// # Errors
///
/// - [`PricingError::InvalidParameter`] when the volatility or the funding
///   rate is zero, or when the rung exceeds [`EVERLASTING_MAX_RUNG`].
/// - [`PricingError::MethodError`] when the standard normal law cannot be
///   built.
/// - [`PricingError::NonFinite`] when the `f64` kernel leaves its range.
pub fn everlasting_equivalent_maturity(
    volatility: Positive,
    funding_rate: Positive,
    rung: NonZeroU32,
) -> Result<Positive, PricingError> {
    let params = EverlastingParams::try_new(
        Positive::ONE,
        Positive::ONE,
        volatility,
        Decimal::ZERO,
        funding_rate,
        rung,
        OptionStyle::Call,
    )?;
    let inputs = params.kernel_inputs()?;
    let at_the_money = ladder_values(&inputs).price;
    let normal = Normal::new(0.0, 1.0)
        .map_err(|e| PricingError::method_error(METHOD, &format!("standard normal: {e}")))?;
    let root_total_variance = 2.0 * normal.inverse_cdf(0.5 + 0.5 * at_the_money);
    let maturity = (root_total_variance / inputs.sigma) * (root_total_variance / inputs.sigma);
    let maturity = to_decimal("pricing::everlasting::equivalent_maturity", maturity)?;
    Ok(Positive::new_decimal(maturity)?)
}

/// Rejects a zero value of a parameter that must be strictly positive.
fn reject_zero(parameter: &'static str, value: Positive) -> Result<(), PricingError> {
    if value == Positive::ZERO {
        return Err(PricingError::invalid_parameter(
            parameter,
            value.to_dec(),
            "must be strictly positive",
        ));
    }
    Ok(())
}

/// Converts a kernel output to `Decimal`, rejecting `NaN` and infinities.
fn to_decimal(context: &'static str, value: f64) -> Result<Decimal, PricingError> {
    finite_decimal(value).ok_or_else(|| PricingError::non_finite(context, value))
}

/// `e^x` in `f64`.
#[inline]
fn exp_f64(x: f64) -> f64 {
    x.exp() // scan-banned: allow -- f64 `exp`: returns inf on overflow, it does not abort; the non-finite value is rejected at the `Decimal` boundary
}

/// `ln x` in `f64`.
#[inline]
fn ln_f64(x: f64) -> f64 {
    x.ln() // scan-banned: allow -- f64 `ln`: returns -inf or NaN outside its domain, it does not abort; the non-finite value is rejected at the `Decimal` boundary
}

/// `√x` in `f64`.
#[inline]
fn sqrt_f64(x: f64) -> f64 {
    x.sqrt() // scan-banned: allow -- f64 `sqrt`: returns NaN for negative input, it does not abort; the non-finite value is rejected at the `Decimal` boundary
}

/// The inputs of the kernel in `f64`.
#[derive(Debug, Clone, Copy)]
struct KernelInputs {
    spot: f64,
    strike: f64,
    sigma: f64,
    mu: f64,
    kappa: f64,
    rung: u32,
    style: OptionStyle,
}

impl KernelInputs {
    /// The same contract one rung higher. The kernel has no rung limit of
    /// its own, so the top rung has a next one for the identities.
    fn next_rung(&self) -> Result<Self, PricingError> {
        let rung = self
            .rung
            .checked_add(1)
            .ok_or_else(|| PricingError::method_error(METHOD, "rung overflows u32"))?;
        Ok(Self { rung, ..*self })
    }
}

/// Price, delta and gamma of one rung.
#[derive(Debug, Clone, Copy)]
struct LadderValues {
    price: f64,
    delta: f64,
    gamma: f64,
}

/// The three sums of the closed form on the out-of-the-money side.
#[derive(Debug, Clone, Copy)]
struct TailSums {
    /// `Σ c_m·E_{j−m}(r_s·d)/r_s^{j−m}`: the coefficient of the underlying.
    spot: f64,
    /// `Σ c_m·E_{j−m}(r_k·d)/r_k^{j−m}`: the coefficient of the strike.
    strike: f64,
    /// `e^{−r_s·d}·Σ c_m·d^{j−m−1}/(j−m−1)!`: gamma times the underlying.
    gamma: f64,
}

/// Sums of the closed form for rung `rung`, at log-distance `distance ≥ 0`
/// from the strike, with decay rates `rate_spot` and `rate_strike` and
/// `c_m = C(j−1+m, m)·a^j·b^j/(a+b)^{j+m}`.
///
/// One pass over `p = j − m = 1..=j`, on logarithms: `E_p` grows by one term
/// of its sum per step and the binomial coefficient is updated by its ratio.
fn tail_sums(
    rung: u32,
    ln_a: f64,
    ln_b: f64,
    ln_a_plus_b: f64,
    rate_spot: f64,
    rate_strike: f64,
    distance: f64,
) -> TailSums {
    let j = f64::from(rung);
    let ln_rate_spot = ln_f64(rate_spot);
    let ln_rate_strike = ln_f64(rate_strike);
    let z_spot = rate_spot * distance;
    let z_strike = rate_strike * distance;
    let ln_distance = ln_f64(distance);
    let ln_z_spot = ln_f64(z_spot);
    let ln_z_strike = ln_f64(z_strike);

    // ln C(2j−2, j−1), the coefficient of m = j − 1 (p = 1).
    let mut ln_binomial = 0.0_f64;
    for i in 1..rung {
        let i = f64::from(i);
        ln_binomial += ln_f64((j - 1.0 + i) / i);
    }

    let mut ln_factorial = 0.0_f64; // ln (p − 1)!
    let mut e_spot = 0.0_f64; // E_p(z_spot)
    let mut e_strike = 0.0_f64; // E_p(z_strike)
    let mut sums = TailSums {
        spot: 0.0,
        strike: 0.0,
        gamma: 0.0,
    };

    for index in 0..rung {
        // p = index + 1 = j − m.
        let i = f64::from(index);
        let p_f = i + 1.0;
        let m = j - p_f;
        if index > 0 {
            ln_factorial += ln_f64(i);
        }
        // The term i of each sum; for i = 0 the power is 1 even at distance 0.
        let (pow_spot, pow_strike, pow_distance) = if index == 0 {
            (0.0, 0.0, 0.0)
        } else {
            (i * ln_z_spot, i * ln_z_strike, i * ln_distance)
        };
        e_spot += exp_f64(pow_spot - ln_factorial - z_spot);
        e_strike += exp_f64(pow_strike - ln_factorial - z_strike);

        let ln_c = ln_binomial + j * (ln_a + ln_b) - (j + m) * ln_a_plus_b;
        sums.spot += exp_f64(ln_c - p_f * ln_rate_spot) * e_spot;
        sums.strike += exp_f64(ln_c - p_f * ln_rate_strike) * e_strike;
        sums.gamma += exp_f64(ln_c + pow_distance - ln_factorial - z_spot);

        // C(j−1+m, m) → C(j−2+m, m−1) for the next p.
        if m > 0.0 {
            ln_binomial += ln_f64(m / (j - 1.0 + m));
        }
    }
    sums
}

/// Price, delta and gamma of one rung under geometric Brownian motion.
///
/// The inputs come from a validated [`EverlastingParams`]: prices,
/// volatility and funding rate strictly positive, funding rate above the
/// drift. Outside that domain the result is non-finite and is rejected at
/// the `Decimal` boundary.
fn ladder_values(inputs: &KernelInputs) -> LadderValues {
    let variance = inputs.sigma * inputs.sigma;
    let nu = inputs.mu - 0.5 * variance;
    let discriminant = sqrt_f64(nu * nu + 2.0 * variance * inputs.kappa);
    // a·b = 2κ/σ²: the root that would cancel is taken from the product.
    let (a, b) = if nu >= 0.0 {
        let b = (nu + discriminant) / variance;
        (2.0 * inputs.kappa / (variance * b), b)
    } else {
        let a = (discriminant - nu) / variance;
        (a, 2.0 * inputs.kappa / (variance * a))
    };
    // a − 1 from the value of the quadratic at 1, exact when κ is near μ.
    let a_minus_one = (inputs.kappa - inputs.mu) / (0.5 * variance * (1.0 + b));
    // E[S_τ]/S = (κ/(κ − μ))^j.
    let forward_factor = exp_f64(-f64::from(inputs.rung) * (-inputs.mu / inputs.kappa).ln_1p());

    let ln_a = ln_f64(a);
    let ln_b = ln_f64(b);
    let ln_a_plus_b = ln_f64(a + b);
    let log_moneyness = ln_f64(inputs.strike / inputs.spot);

    let (call, put, call_delta, put_delta, gamma_spot) = if log_moneyness >= 0.0 {
        let sums = tail_sums(
            inputs.rung,
            ln_a,
            ln_b,
            ln_a_plus_b,
            a_minus_one,
            a,
            log_moneyness,
        );
        let call = inputs.spot * sums.spot - inputs.strike * sums.strike;
        let put = call - inputs.spot * forward_factor + inputs.strike;
        (call, put, sums.spot, sums.spot - forward_factor, sums.gamma)
    } else {
        let sums = tail_sums(
            inputs.rung,
            ln_a,
            ln_b,
            ln_a_plus_b,
            b + 1.0,
            b,
            -log_moneyness,
        );
        let put = inputs.strike * sums.strike - inputs.spot * sums.spot;
        let call = put + inputs.spot * forward_factor - inputs.strike;
        (
            call,
            put,
            forward_factor - sums.spot,
            -sums.spot,
            sums.gamma,
        )
    };

    let gamma = gamma_spot / inputs.spot;
    match inputs.style {
        OptionStyle::Call => LadderValues {
            price: call,
            delta: call_delta,
            gamma,
        },
        OptionStyle::Put => LadderValues {
            price: put,
            delta: put_delta,
            gamma,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal::prelude::ToPrimitive;
    use rust_decimal_macros::dec;

    const MONTH: f64 = 30.0 / 365.0;

    fn rung(j: u32) -> NonZeroU32 {
        NonZeroU32::new(j).unwrap()
    }

    fn params(
        spot: f64,
        strike: f64,
        mu: f64,
        sigma: f64,
        kappa: f64,
        j: u32,
        style: OptionStyle,
    ) -> EverlastingParams {
        EverlastingParams::try_new(
            pos_or_panic!(spot),
            pos_or_panic!(strike),
            pos_or_panic!(sigma),
            Decimal::try_from(mu).unwrap(),
            pos_or_panic!(kappa),
            rung(j),
            style,
        )
        .unwrap()
    }

    fn inputs(
        spot: f64,
        strike: f64,
        mu: f64,
        sigma: f64,
        kappa: f64,
        j: u32,
        style: OptionStyle,
    ) -> KernelInputs {
        KernelInputs {
            spot,
            strike,
            sigma,
            mu,
            kappa,
            rung: j,
            style,
        }
    }

    fn price(spot: f64, strike: f64, mu: f64, sigma: f64, kappa: f64, j: u32) -> f64 {
        ladder_values(&inputs(
            spot,
            strike,
            mu,
            sigma,
            kappa,
            j,
            OptionStyle::Call,
        ))
        .price
    }

    fn relative(actual: f64, expected: f64) -> f64 {
        ((actual - expected) / expected).abs()
    }

    /// Undiscounted Black-Scholes call at maturity `s`, with drift `mu`.
    fn european_call(spot: f64, strike: f64, mu: f64, sigma: f64, s: f64) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let root = sigma * s.sqrt();
        let d1 = ((spot / strike).ln() + (mu + 0.5 * sigma * sigma) * s) / root;
        spot * (mu * s).exp() * normal.cdf(d1) - strike * normal.cdf(d1 - root)
    }

    /// Erlang mixture of European calls by composite Simpson on the variable
    /// `u = √s`, which removes the square-root behaviour at zero.
    fn erlang_mixture(spot: f64, strike: f64, mu: f64, sigma: f64, kappa: f64, j: u32) -> f64 {
        let upper = ((f64::from(j) + 40.0 + 12.0 * f64::from(j).sqrt()) / kappa).sqrt();
        let steps = 40_000_usize;
        let h = upper / steps as f64;
        let ln_factorial: f64 = (1..j).map(|i| f64::from(i).ln()).sum();
        let integrand = |u: f64| {
            if u == 0.0 {
                return 0.0;
            }
            let s = u * u;
            let ln_density =
                f64::from(j) * kappa.ln() + f64::from(j - 1) * s.ln() - kappa * s - ln_factorial;
            ln_density.exp() * european_call(spot, strike, mu, sigma, s) * 2.0 * u
        };
        let mut total = integrand(0.0) + integrand(upper);
        for i in 1..steps {
            let weight = if i % 2 == 1 { 4.0 } else { 2.0 };
            total += weight * integrand(i as f64 * h);
        }
        total * h / 3.0
    }

    #[test]
    fn test_everlasting_price_reference_grid_matches_quadrature() {
        // (S, K, μ, σ, κ, j, price): the price is a 40-digit quadrature of
        // the Erlang mixture of Black-Scholes values, computed independently.
        let references = [
            (
                100.0,
                100.0,
                0.0,
                0.5,
                365.0 / 30.0,
                1,
                5.061_534_248_586_659,
            ),
            (
                100.0,
                120.0,
                0.0,
                0.5,
                365.0 / 30.0,
                1,
                0.915_559_536_329_961,
            ),
            (
                100.0,
                80.0,
                0.0,
                0.5,
                365.0 / 30.0,
                1,
                20.499_469_092_357_974,
            ),
            (
                100.0,
                110.0,
                0.10,
                0.5,
                4.0 * 365.0 / 30.0,
                3,
                1.769_310_696_361_171,
            ),
            (
                100.0,
                90.0,
                -0.20,
                0.5,
                8.0 * 365.0 / 30.0,
                8,
                10.549_218_892_751_894,
            ),
            (
                100.0,
                100.0,
                0.05,
                0.8,
                4.0 * 365.0 / 7.0,
                4,
                4.332_161_284_275_916,
            ),
            (
                100.0,
                130.0,
                0.0,
                0.3,
                16.0 * 365.0 / 30.0,
                16,
                0.005_636_227_278_154_376,
            ),
            (
                100.0,
                105.0,
                0.03,
                0.6,
                64.0 * 365.0 / 30.0,
                64,
                4.897_996_045_174_277,
            ),
            (
                100.0,
                95.0,
                0.0,
                0.2,
                64.0 * 365.0 / 90.0,
                64,
                6.859_857_544_220_178,
            ),
            (
                100.0,
                400.0,
                0.0,
                0.5,
                365.0 / 30.0,
                1,
                1.142_631_243_615_998e-5,
            ),
        ];
        for (spot, strike, mu, sigma, kappa, j, expected) in references {
            let actual = price(spot, strike, mu, sigma, kappa, j);
            assert!(
                relative(actual, expected) < 1e-12,
                "S={spot} K={strike} mu={mu} j={j}: {actual} vs {expected}"
            );
        }
    }

    #[test]
    fn test_everlasting_price_extreme_decay_rate_matches_quadrature() {
        // a ≈ 4300 at rung 64: the two terms of the price agree to three
        // digits, so the tolerance is looser.
        let actual = price(100.0, 101.0, 0.0, 0.05, 64.0 * 365.0, 64);
        assert!(relative(actual, 6.610_884_996_811_611e-6) < 1e-10);
    }

    #[test]
    fn test_everlasting_price_grid_matches_erlang_mixture() {
        for mu in [0.0, 0.10, -0.20] {
            for (ladder, j) in [(1, 1), (2, 1), (2, 2), (4, 3), (8, 8)] {
                let kappa = f64::from(ladder) / MONTH;
                for strike in [80.0, 95.0, 100.0, 105.0, 125.0] {
                    let actual = price(100.0, strike, mu, 0.5, kappa, j);
                    let expected = erlang_mixture(100.0, strike, mu, 0.5, kappa, j);
                    assert!(
                        (actual - expected).abs() < 1e-6,
                        "mu={mu} j={j} K={strike}: {actual} vs {expected}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_everlasting_price_first_rung_matches_closed_form() {
        let (sigma, kappa, strike) = (0.5_f64, 365.0 / 30.0, 100.0_f64);
        // At the money, no drift: K/√(1 + 8κ/σ²).
        let atm = price(strike, strike, 0.0, sigma, kappa, 1);
        let expected = strike / (1.0 + 8.0 * kappa / (sigma * sigma)).sqrt();
        assert!(relative(atm, expected) < 1e-14);
        // Out of the money, with drift: K·b·e^{−ak}/((a + b)(a − 1)).
        let mu = 0.07_f64;
        let nu = mu - 0.5 * sigma * sigma;
        let disc = (nu * nu + 2.0 * sigma * sigma * kappa).sqrt();
        let (a, b) = ((disc - nu) / (sigma * sigma), (disc + nu) / (sigma * sigma));
        let k = (115.0_f64 / 100.0).ln();
        let expected = 115.0 * b * (-a * k).exp() / ((a + b) * (a - 1.0));
        assert!(relative(price(100.0, 115.0, mu, sigma, kappa, 1), expected) < 1e-13);
    }

    #[test]
    fn test_everlasting_price_put_call_parity_holds() {
        for (mu, j, strike) in [(0.0, 1, 90.0), (0.08, 3, 110.0), (-0.15, 6, 100.0)] {
            let kappa = f64::from(j) / MONTH;
            let call =
                everlasting_price(&params(100.0, strike, mu, 0.5, kappa, j, OptionStyle::Call))
                    .unwrap()
                    .to_f64()
                    .unwrap();
            let put =
                everlasting_price(&params(100.0, strike, mu, 0.5, kappa, j, OptionStyle::Put))
                    .unwrap()
                    .to_f64()
                    .unwrap();
            let forward = 100.0 * (kappa / (kappa - mu)).powi(j as i32);
            assert!((call - put - (forward - strike)).abs() < 1e-10);
        }
    }

    #[test]
    fn test_everlasting_price_is_continuous_at_the_strike() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for j in [1, 4] {
                let kappa = f64::from(j) / MONTH;
                let below = ladder_values(&inputs(100.0 - 1e-9, 100.0, 0.05, 0.5, kappa, j, style));
                let above = ladder_values(&inputs(100.0 + 1e-9, 100.0, 0.05, 0.5, kappa, j, style));
                assert!((below.price - above.price).abs() < 1e-7);
                assert!((below.delta - above.delta).abs() < 1e-7);
                assert!((below.gamma - above.gamma).abs() < 1e-7);
            }
        }
    }

    #[test]
    fn test_everlasting_delta_and_gamma_match_finite_differences() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for (mu, j, strike) in [(0.0, 1, 110.0), (0.10, 3, 92.0), (-0.20, 8, 104.0)] {
                let kappa = f64::from(j) / MONTH;
                let at = |spot: f64| ladder_values(&inputs(spot, strike, mu, 0.5, kappa, j, style));
                let h = 1e-3;
                let (down, mid, up) = (at(100.0 - h), at(100.0), at(100.0 + h));
                let delta = (up.price - down.price) / (2.0 * h);
                let gamma = (up.delta - down.delta) / (2.0 * h);
                assert!((mid.delta - delta).abs() < 1e-8, "delta {style:?} j={j}");
                assert!((mid.gamma - gamma).abs() < 1e-8, "gamma {style:?} j={j}");
            }
        }
    }

    #[test]
    fn test_everlasting_funding_equals_rate_times_spread_to_previous_rung() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for (mu, strike) in [(0.0, 107.0), (0.10, 96.0)] {
                let kappa = 4.0 / MONTH;
                let payoff = match style {
                    OptionStyle::Call => (100.0_f64 - strike).max(0.0),
                    OptionStyle::Put => (strike - 100.0_f64).max(0.0),
                };
                let mut previous = payoff;
                for j in 1..=4 {
                    let p = params(100.0, strike, mu, 0.5, kappa, j, style);
                    let here = everlasting_price(&p).unwrap().to_f64().unwrap();
                    let funding = everlasting_funding(&p).unwrap().to_f64().unwrap();
                    assert!(
                        (funding - kappa * (here - previous)).abs() < 1e-9,
                        "{style:?} mu={mu} j={j}"
                    );
                    previous = here;
                }
            }
        }
    }

    #[test]
    fn test_everlasting_funding_without_drift_is_not_negative() {
        for strike in [70.0, 100.0, 140.0] {
            let p = params(100.0, strike, 0.0, 0.5, 2.0 / MONTH, 2, OptionStyle::Put);
            assert!(everlasting_funding(&p).unwrap() >= Decimal::ZERO);
        }
    }

    #[test]
    fn test_everlasting_vega_matches_finite_difference() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for (mu, j, strike) in [(0.0, 1, 100.0), (0.10, 3, 112.0), (-0.20, 5, 93.0)] {
                let kappa = f64::from(j) / MONTH;
                let at = |sigma: f64| {
                    ladder_values(&inputs(100.0, strike, mu, sigma, kappa, j, style)).price
                };
                let h = 1e-4;
                let expected = (at(0.5 + h) - at(0.5 - h)) / (2.0 * h);
                let vega = everlasting_vega(&params(100.0, strike, mu, 0.5, kappa, j, style))
                    .unwrap()
                    .to_f64()
                    .unwrap();
                assert!(
                    (vega - expected).abs() < 1e-6,
                    "{style:?} mu={mu} j={j}: {vega} vs {expected}"
                );
            }
        }
    }

    #[test]
    fn test_everlasting_funding_rate_sensitivity_matches_finite_difference() {
        for (mu, j, strike) in [(0.0, 1, 100.0), (0.10, 3, 112.0), (-0.20, 5, 93.0)] {
            let kappa = f64::from(j) / MONTH;
            let h = 1e-4 * kappa;
            let expected = (price(100.0, strike, mu, 0.5, kappa + h, j)
                - price(100.0, strike, mu, 0.5, kappa - h, j))
                / (2.0 * h);
            let actual = everlasting_funding_rate_sensitivity(&params(
                100.0,
                strike,
                mu,
                0.5,
                kappa,
                j,
                OptionStyle::Call,
            ))
            .unwrap()
            .to_f64()
            .unwrap();
            assert!((actual - expected).abs() < 1e-8, "mu={mu} j={j}");
            assert!(actual < 0.0);
        }
    }

    #[test]
    fn test_everlasting_mean_maturity_is_rung_over_rate() {
        let maturity = everlasting_mean_maturity(pos_or_panic!(8.0), rung(4)).unwrap();
        assert_eq!(maturity.to_dec(), dec!(0.5));
        assert!(everlasting_mean_maturity(Positive::ZERO, rung(1)).is_err());
    }

    #[test]
    fn test_everlasting_equivalent_maturity_first_rung_matches_closed_form() {
        let (sigma, kappa) = (0.5_f64, 365.0 / 30.0);
        let theta = sigma * sigma / kappa;
        let normal = Normal::new(0.0, 1.0).unwrap();
        let root = 2.0 * normal.inverse_cdf(0.5 + 0.5 * (theta / (8.0 + theta)).sqrt());
        let expected = root * root / (sigma * sigma);
        let actual =
            everlasting_equivalent_maturity(pos_or_panic!(sigma), pos_or_panic!(kappa), rung(1))
                .unwrap()
                .to_f64();
        assert!(relative(actual, expected) < 1e-12);
        // Close to π/(4κ), and below it.
        let rule = std::f64::consts::FRAC_PI_4 / kappa;
        assert!(actual < rule && relative(actual, rule) < 2e-3);
    }

    #[test]
    fn test_everlasting_equivalent_maturity_approaches_mean_with_the_rung() {
        // Small total variance: T_eq/T → Γ(j + ½)²/(j·Γ(j)²).
        let limits = [
            (1, std::f64::consts::FRAC_PI_4),
            (2, 0.883_572_933_822_129),
            (8, 0.969_252_862_415_066_5),
        ];
        let mut previous = 0.0;
        for (j, limit) in limits {
            let kappa = f64::from(j) / MONTH;
            let ratio =
                everlasting_equivalent_maturity(pos_or_panic!(0.02), pos_or_panic!(kappa), rung(j))
                    .unwrap()
                    .to_f64()
                    * kappa
                    / f64::from(j);
            assert!((ratio - limit).abs() < 1e-5, "j={j}: {ratio} vs {limit}");
            assert!(ratio > previous && ratio < 1.0);
            previous = ratio;
        }
    }

    #[test]
    fn test_everlasting_params_rejects_values_outside_the_domain() {
        let build = |spot: Positive, strike: Positive, sigma: Positive, mu, kappa: Positive, j| {
            EverlastingParams::try_new(spot, strike, sigma, mu, kappa, rung(j), OptionStyle::Call)
        };
        let (hundred, half, ten) = (Positive::HUNDRED, pos_or_panic!(0.5), Positive::TEN);
        assert!(build(hundred, hundred, half, dec!(0), ten, 1).is_ok());
        for (result, parameter) in [
            (
                build(Positive::ZERO, hundred, half, dec!(0), ten, 1),
                "underlying_price",
            ),
            (
                build(hundred, Positive::ZERO, half, dec!(0), ten, 1),
                "strike_price",
            ),
            (
                build(hundred, hundred, Positive::ZERO, dec!(0), ten, 1),
                "volatility",
            ),
            (
                build(hundred, hundred, half, dec!(0), Positive::ZERO, 1),
                "funding_rate",
            ),
            (
                build(hundred, hundred, half, dec!(10), ten, 1),
                "funding_rate",
            ),
            (
                build(
                    hundred,
                    hundred,
                    half,
                    dec!(0),
                    ten,
                    EVERLASTING_MAX_RUNG + 1,
                ),
                "rung",
            ),
        ] {
            assert!(matches!(
                result,
                Err(PricingError::InvalidParameter { parameter: p, .. }) if p == parameter
            ));
        }
    }

    #[test]
    fn test_everlasting_params_accessors_return_the_inputs() {
        let p = params(100.0, 110.0, 0.05, 0.5, 12.0, 3, OptionStyle::Put);
        assert_eq!(p.underlying_price(), Positive::HUNDRED);
        assert_eq!(p.strike_price(), pos_or_panic!(110.0));
        assert_eq!(p.volatility(), pos_or_panic!(0.5));
        assert_eq!(p.drift(), dec!(0.05));
        assert_eq!(p.funding_rate(), pos_or_panic!(12.0));
        assert_eq!(p.rung(), rung(3));
        assert_eq!(p.option_style(), OptionStyle::Put);
    }

    #[test]
    fn test_everlasting_price_far_out_of_the_money_is_zero_not_negative() {
        let p = params(100.0, 1e6, 0.0, 0.2, 64.0 * 365.0, 64, OptionStyle::Call);
        assert_eq!(everlasting_price(&p).unwrap(), Decimal::ZERO);
        assert_eq!(everlasting_gamma(&p).unwrap(), Decimal::ZERO);
    }
}
