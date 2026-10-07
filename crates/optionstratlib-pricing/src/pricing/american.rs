/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/1/26
******************************************************************************/

//! # American Option Pricing Module
//!
//! This module provides analytical approximation methods for pricing American options.
//! American options can be exercised at any time before expiration, which makes them
//! more valuable than European options but also more complex to price.
//!
//! ## Implemented Methods
//!
//! ### Barone-Adesi-Whaley (BAW) Approximation
//!
//! The BAW model provides a fast analytical approximation for American options with
//! O(1) complexity, making it suitable for real-time pricing applications.
//!
//! ## Usage Example
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use optionstratlib_pricing::pricing::american::barone_adesi_whaley;
//! use optionstratlib_core::model::types::OptionStyle;
//! use optionstratlib_core::model::Positive;
//! # fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let price = barone_adesi_whaley(
//!     Positive::HUNDRED,      // underlying price
//!     Positive::HUNDRED,      // strike price
//!     Positive::ONE,          // time to expiration (years)
//!     dec!(0.05),             // risk-free rate
//!     Positive::ZERO,         // dividend yield
//!     Positive::new(0.2)?,    // volatility
//!     &OptionStyle::Call,
//! );
//! # Ok(())
//! # }
//! ```
//!
//! ## References
//!
//! - Barone-Adesi, G., & Whaley, R. E. (1987). "Efficient Analytic Approximation
//!   of American Option Values". Journal of Finance, 42(2), 301-320.

use crate::error::PricingError;
use crate::kernels::{big_n, discount_factor};
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_add, d_div, d_ln, d_mul, d_powd, d_sqrt, d_sub};
use optionstratlib_core::model::types::OptionStyle;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Maximum iterations for Newton-Raphson method to find critical price.
const MAX_ITERATIONS: usize = 100;

/// Convergence tolerance for critical price calculation, in price units.
/// A `Decimal` constant, so no `f64` conversion (and no fallback for a
/// failed one) sits in front of the iteration (#639).
const TOLERANCE: Decimal = dec!(0.000001);

/// Prices an American option using the Barone-Adesi-Whaley (BAW) approximation.
///
/// This method provides a fast analytical approximation for American options,
/// offering O(1) complexity compared to O(n²) for binomial tree methods.
///
/// # Parameters
///
/// * `spot` - Current price of the underlying asset
/// * `strike` - Strike price of the option
/// * `time_to_expiry` - Time to expiration in years
/// * `risk_free_rate` - Annualized risk-free interest rate
/// * `dividend_yield` - Annualized dividend yield
/// * `volatility` - Annualized volatility (standard deviation of returns)
/// * `option_style` - Whether the option is a Call or Put
///
/// # Returns
///
/// * `Result<Decimal, PricingError>` - The estimated American option price
///
/// # Algorithm
///
/// For American calls:
/// - If S >= S*, return S - K (immediate exercise)
/// - Otherwise, return C_european + A2 * (S/S*)^q2
///
/// For American puts:
/// - If S <= S**, return K - S (immediate exercise)
/// - Otherwise, return P_european + A1 * (S/S**)^q1
///
/// Where S* and S** are the critical (early exercise) prices.
///
/// Limits and floor:
/// - `T = 0` returns the intrinsic value.
/// - `σ = 0` returns the exact value on the deterministic forward path: the
///   best of exercising now, at expiry, or at the interior optimum when the
///   carry makes one exist (see `zero_volatility_american`).
/// - The result is never below the intrinsic value `max(S - K, 0)` /
///   `max(K - S, 0)`: an American option can be exercised immediately.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use optionstratlib_pricing::pricing::american::barone_adesi_whaley;
/// use optionstratlib_core::model::types::OptionStyle;
/// use optionstratlib_core::model::Positive;
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
/// // Price an American call option
/// let call_price = barone_adesi_whaley(
///     Positive::HUNDRED,           // spot = 100
///     Positive::HUNDRED,           // strike = 100
///     Positive::ONE,               // 1 year to expiry
///     dec!(0.05),                  // 5% risk-free rate
///     Positive::ZERO,              // no dividends
///     Positive::new(0.2)?,         // 20% volatility
///     &OptionStyle::Call,
/// )?;
///
/// // Price an American put option
/// let put_price = barone_adesi_whaley(
///     Positive::HUNDRED,
///     Positive::HUNDRED,
///     Positive::ONE,
///     dec!(0.05),
///     Positive::ZERO,
///     Positive::new(0.2)?,
///     &OptionStyle::Put,
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// - [`PricingError::MethodError`] when the quadratic formula receives a
///   negative discriminant, or when the internal `d1` rejects its inputs (a
///   non-positive time, volatility, underlying or strike).
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range, or when a moneyness ratio underflows to
///   zero: the BAW parameters `M`, `N`, `q1` / `q2`, the Newton-Raphson
///   boundary iteration, or the early-exercise premium.
pub fn barone_adesi_whaley(
    spot: Positive,
    strike: Positive,
    time_to_expiry: Positive,
    risk_free_rate: Decimal,
    dividend_yield: Positive,
    volatility: Positive,
    option_style: &OptionStyle,
) -> Result<Decimal, PricingError> {
    let approximation = baw_approximation(
        spot,
        strike,
        time_to_expiry,
        risk_free_rate,
        dividend_yield,
        volatility,
        option_style,
    )?;
    // Domain floor (#648): an American option can be exercised now, so it is
    // worth at least its intrinsic value (Hull, *Options, Futures and Other
    // Derivatives*, early exercise). The quadratic approximation respects it
    // above the critical price, but the `q = 0` call returns the European
    // value, which falls below `S - K` at a negative rate.
    Ok(approximation.max(intrinsic_value(
        spot.to_dec(),
        strike.to_dec(),
        option_style,
    )?))
}

/// Intrinsic value `max(S - K, 0)` (call) or `max(K - S, 0)` (put).
#[inline]
fn intrinsic_value(
    s: Decimal,
    k: Decimal,
    option_style: &OptionStyle,
) -> Result<Decimal, PricingError> {
    Ok(match option_style {
        OptionStyle::Call => d_sub(s, k, "pricing::american::intrinsic::call")?,
        OptionStyle::Put => d_sub(k, s, "pricing::american::intrinsic::put")?,
    }
    .max(Decimal::ZERO))
}

/// Exact American value when the underlying is deterministic (`σ = 0`).
///
/// The spot follows its forward, `S(τ) = S e^((r - q)τ)`, so exercising a put
/// at `τ` is worth `f(τ) = K e^(-rτ) - S e^(-qτ)` today and a call `-f(τ)`.
/// The holder picks the best `τ ∈ [0, T]` or lets the option lapse:
///
/// ```text
/// V = max(0, f(0), f(T), f(τ*))      (put; -f for a call)
/// τ* = ln(rK / (qS)) / (r - q)       (f'(τ*) = 0, only when 0 < τ* < T)
/// ```
///
/// `f''(τ*) = r K e^(-rτ*) (r - q)`, so the stationary point is the
/// interior maximum of the put when `r < q` and of the call when `r > q`;
/// evaluating it whenever it lies inside `(0, T)` and keeping the larger
/// value covers both without branching on the sign. It exists only for
/// `r > 0`, `q > 0`, `r ≠ q`. `f(0)` is the intrinsic value, so the result
/// never falls below it (#648).
///
/// # Errors
///
/// Returns [`PricingError::Decimal`] when a discount factor, the logarithm
/// or a leg leaves the representable `Decimal` range.
fn zero_volatility_american(
    s: Decimal,
    k: Decimal,
    t: Decimal,
    r: Decimal,
    q: Decimal,
    option_style: &OptionStyle,
) -> Result<Decimal, PricingError> {
    // Value today of exercising at `tau`, from the holder's side.
    let exercise_at = |tau: Decimal| -> Result<Decimal, PricingError> {
        let k_pv = d_mul(
            k,
            discount_factor(
                r,
                tau,
                "pricing::american::zero_vol::rt",
                "pricing::american::zero_vol::discount_r",
            )?,
            "pricing::american::zero_vol::k_pv",
        )?;
        let s_pv = d_mul(
            s,
            discount_factor(
                q,
                tau,
                "pricing::american::zero_vol::qt",
                "pricing::american::zero_vol::discount_q",
            )?,
            "pricing::american::zero_vol::s_pv",
        )?;
        Ok(match option_style {
            OptionStyle::Call => d_sub(s_pv, k_pv, "pricing::american::zero_vol::call")?,
            OptionStyle::Put => d_sub(k_pv, s_pv, "pricing::american::zero_vol::put")?,
        })
    };

    let mut best = exercise_at(Decimal::ZERO)?
        .max(exercise_at(t)?)
        .max(Decimal::ZERO);
    if r > Decimal::ZERO && q > Decimal::ZERO && r != q {
        let ratio = d_div(
            d_mul(r, k, "pricing::american::zero_vol::rk")?,
            d_mul(q, s, "pricing::american::zero_vol::qs")?,
            "pricing::american::zero_vol::ratio",
        )?;
        let tau_star = d_div(
            d_ln(ratio, "pricing::american::zero_vol::ln_ratio")?,
            d_sub(r, q, "pricing::american::zero_vol::carry")?,
            "pricing::american::zero_vol::tau_star",
        )?;
        if tau_star > Decimal::ZERO && tau_star < t {
            best = best.max(exercise_at(tau_star)?);
        }
    }
    Ok(best)
}

/// The Barone-Adesi-Whaley quadratic approximation before the intrinsic
/// floor of [`barone_adesi_whaley`]; same parameters and errors.
fn baw_approximation(
    spot: Positive,
    strike: Positive,
    time_to_expiry: Positive,
    risk_free_rate: Decimal,
    dividend_yield: Positive,
    volatility: Positive,
    option_style: &OptionStyle,
) -> Result<Decimal, PricingError> {
    let s = spot.to_dec();
    let k = strike.to_dec();
    let t = time_to_expiry.to_dec();
    let r = risk_free_rate;
    let q = dividend_yield.to_dec();
    let sigma = volatility.to_dec();

    // Handle edge cases
    if t <= Decimal::ZERO {
        // At expiration, return intrinsic value
        return intrinsic_value(s, k, option_style);
    }

    if sigma <= Decimal::ZERO {
        // Zero volatility: the holder chooses the best exercise time on the
        // deterministic forward path, not just expiry (#648).
        return zero_volatility_american(s, k, t, r, q, option_style);
    }

    // Calculate European option price first
    let european_price = black_scholes_european(s, k, t, r, q, sigma, option_style)?;

    // For American calls on non-dividend paying stocks, early exercise is never optimal
    if matches!(option_style, OptionStyle::Call) && q <= Decimal::ZERO {
        return Ok(european_price);
    }

    // Calculate BAW parameters
    let sigma_sq = d_mul(sigma, sigma, "pricing::american::sigma_sq")?;
    let m = d_div(
        d_mul(dec!(2), r, "pricing::american::two_r")?,
        sigma_sq,
        "pricing::american::m",
    )?;
    let n = d_div(
        d_mul(
            dec!(2),
            d_sub(r, q, "pricing::american::carry")?,
            "pricing::american::two_carry",
        )?,
        sigma_sq,
        "pricing::american::n",
    )?;
    let k_factor = d_sub(
        dec!(1),
        discount_factor(
            r,
            t,
            "pricing::american::neg_rt",
            "pricing::american::discount",
        )?,
        "pricing::american::k_factor",
    )?;

    // `4M / K` with `K = 1 - e^(-rT)`. `K` is exactly zero at `r = 0` (and
    // whenever `rT` rounds `e^(-rT)` back to one), where `M` is zero too. The
    // limit of `4M / K = (8r/σ²) / (1 - e^(-rT))` as `rT → 0` is `8 / (σ² T)`,
    // which is what the ratio is replaced by there.
    let four_m_over_k = if k_factor.is_zero() {
        d_div(
            dec!(8),
            d_mul(sigma_sq, t, "pricing::american::variance_time")?,
            "pricing::american::four_m_over_k_limit",
        )?
    } else {
        d_div(
            d_mul(dec!(4), m, "pricing::american::four_m")?,
            k_factor,
            "pricing::american::four_m_over_k",
        )?
    };

    let n_minus_one = d_sub(n, dec!(1), "pricing::american::n_minus_one")?;
    let discriminant = d_add(
        d_powd(n_minus_one, Decimal::TWO, "pricing::american::n_squared")?,
        four_m_over_k,
        "pricing::american::discriminant",
    )?;
    let sqrt_disc = d_sqrt(discriminant, "pricing::american::sqrt_disc").map_err(|_| {
        PricingError::method_error(
            "baw",
            "cannot calculate square root of negative discriminant",
        )
    })?;

    match option_style {
        OptionStyle::Call => {
            let q2 = d_div(
                d_add(-n_minus_one, sqrt_disc, "pricing::american::q2_numerator")?,
                dec!(2),
                "pricing::american::q2",
            )?;

            // Find critical price S*
            let s_star = find_critical_price_call(s, k, t, r, q, sigma, q2)?;

            if s >= s_star {
                // Immediate exercise is optimal
                d_sub(s, k, "pricing::american::call::immediate_exercise")
                    .map_err(PricingError::from)
            } else {
                // Early exercise premium
                let d1_val = d1(s_star, k, t, r, q, sigma)?;
                let n_d1 = big_n(d1_val)?;
                let a2 = d_mul(
                    d_div(s_star, q2, "pricing::american::call::a2_scale")?,
                    d_sub(
                        dec!(1),
                        d_mul(
                            discount_factor(
                                q,
                                t,
                                "pricing::american::call::neg_qt",
                                "pricing::american::call::dividend_discount",
                            )?,
                            n_d1,
                            "pricing::american::call::a2_weight",
                        )?,
                        "pricing::american::call::a2_bracket",
                    )?,
                    "pricing::american::call::a2",
                )?;
                let early_exercise_premium = d_mul(
                    a2,
                    d_powd(
                        d_div(s, s_star, "pricing::american::call::moneyness")?,
                        q2,
                        "pricing::american::call::power",
                    )?,
                    "pricing::american::call::premium",
                )?;
                d_add(
                    european_price,
                    early_exercise_premium,
                    "pricing::american::call::price",
                )
                .map_err(PricingError::from)
            }
        }
        OptionStyle::Put => {
            let q1 = d_div(
                d_sub(-n_minus_one, sqrt_disc, "pricing::american::q1_numerator")?,
                dec!(2),
                "pricing::american::q1",
            )?;

            // Find critical price S**
            let s_star_star = find_critical_price_put(s, k, t, r, q, sigma, q1)?;

            if s <= s_star_star {
                // Immediate exercise is optimal
                d_sub(k, s, "pricing::american::put::immediate_exercise")
                    .map_err(PricingError::from)
            } else {
                // Early exercise premium
                let d1_val = d1(s_star_star, k, t, r, q, sigma)?;
                let n_minus_d1 = big_n(-d1_val)?;
                let a1 = d_mul(
                    -d_div(s_star_star, q1, "pricing::american::put::a1_scale")?,
                    d_sub(
                        dec!(1),
                        d_mul(
                            discount_factor(
                                q,
                                t,
                                "pricing::american::put::neg_qt",
                                "pricing::american::put::dividend_discount",
                            )?,
                            n_minus_d1,
                            "pricing::american::put::a1_weight",
                        )?,
                        "pricing::american::put::a1_bracket",
                    )?,
                    "pricing::american::put::a1",
                )?;
                let early_exercise_premium = d_mul(
                    a1,
                    d_powd(
                        d_div(s, s_star_star, "pricing::american::put::moneyness")?,
                        q1,
                        "pricing::american::put::power",
                    )?,
                    "pricing::american::put::premium",
                )?;
                d_add(
                    european_price,
                    early_exercise_premium,
                    "pricing::american::put::price",
                )
                .map_err(PricingError::from)
            }
        }
    }
}

/// Calculates the Black-Scholes price for a European option.
///
/// This is a helper function used internally by the BAW approximation.
fn black_scholes_european(
    s: Decimal,
    k: Decimal,
    t: Decimal,
    r: Decimal,
    q: Decimal,
    sigma: Decimal,
    option_style: &OptionStyle,
) -> Result<Decimal, PricingError> {
    let d1_val = d1(s, k, t, r, q, sigma)?;
    let sqrt_t = d_sqrt(t, "pricing::american::european::sqrt_t")?;
    let d2_val = d_sub(
        d1_val,
        d_mul(sigma, sqrt_t, "pricing::american::european::sigma_sqrt_t")?,
        "pricing::american::european::d2",
    )?;

    let discount = discount_factor(
        r,
        t,
        "pricing::american::european::neg_rt",
        "pricing::american::european::discount",
    )?;
    let forward_factor = discount_factor(
        q,
        t,
        "pricing::american::european::neg_qt",
        "pricing::american::european::forward_factor",
    )?;

    let n_d1 = big_n(d1_val)?;
    let n_d2 = big_n(d2_val)?;
    let n_minus_d1 = big_n(-d1_val)?;
    let n_minus_d2 = big_n(-d2_val)?;

    let s_pv = d_mul(s, forward_factor, "pricing::american::european::s_pv")?;
    let k_pv = d_mul(k, discount, "pricing::american::european::k_pv")?;

    match option_style {
        OptionStyle::Call => Ok(d_sub(
            d_mul(s_pv, n_d1, "pricing::american::european::call::spot")?,
            d_mul(k_pv, n_d2, "pricing::american::european::call::strike")?,
            "pricing::american::european::call",
        )?),
        OptionStyle::Put => Ok(d_sub(
            d_mul(k_pv, n_minus_d2, "pricing::american::european::put::strike")?,
            d_mul(s_pv, n_minus_d1, "pricing::american::european::put::spot")?,
            "pricing::american::european::put",
        )?),
    }
}

/// Calculates d1 parameter for Black-Scholes formula.
// Not `kernels::d1`: BAW evaluates it on raw `Decimal`s at trial prices with
// `sigma * sigma` and `PricingError` validation, a different step sequence.
fn d1(
    s: Decimal,
    k: Decimal,
    t: Decimal,
    r: Decimal,
    q: Decimal,
    sigma: Decimal,
) -> Result<Decimal, PricingError> {
    if t <= Decimal::ZERO || sigma <= Decimal::ZERO {
        return Err(PricingError::method_error(
            "d1",
            "time and volatility must be positive",
        ));
    }
    // `ln(S / K)` is undefined for a zero underlying or a zero strike, so both
    // are rejected before the ratio is formed.
    if s <= Decimal::ZERO || k <= Decimal::ZERO {
        return Err(PricingError::method_error(
            "d1",
            "underlying price and strike must be positive",
        ));
    }
    let sqrt_t = d_sqrt(t, "pricing::american::d1::sqrt_t")?;
    let ln_s_k = d_ln(
        d_div(s, k, "pricing::american::d1::moneyness")?,
        "pricing::american::d1::log_moneyness",
    )?;
    Ok(d_div(
        d_add(
            ln_s_k,
            d_mul(
                d_add(
                    d_sub(r, q, "pricing::american::d1::carry")?,
                    d_div(
                        d_mul(sigma, sigma, "pricing::american::d1::variance")?,
                        dec!(2),
                        "pricing::american::d1::half_variance",
                    )?,
                    "pricing::american::d1::drift_rate",
                )?,
                t,
                "pricing::american::d1::drift",
            )?,
            "pricing::american::d1::numerator",
        )?,
        d_mul(sigma, sqrt_t, "pricing::american::d1::denominator")?,
        "pricing::american::d1",
    )?)
}

/// Finds the critical price S* for American calls using Newton-Raphson.
///
/// The critical price is where immediate exercise becomes optimal.
fn find_critical_price_call(
    _spot: Decimal,
    strike: Decimal,
    t: Decimal,
    r: Decimal,
    q: Decimal,
    sigma: Decimal,
    q2: Decimal,
) -> Result<Decimal, PricingError> {
    // Initial guess: use strike as starting point
    let mut s_star = d_mul(strike, dec!(1.1), "pricing::american::call::s_star_seed")?;
    let exp_qt = discount_factor(
        q,
        t,
        "pricing::american::call::neg_qt_seed",
        "pricing::american::call::dividend_discount_seed",
    )?;
    let tolerance = TOLERANCE;

    // `C_euro(S) + (S / q2)(1 - e^(-qT) N(d1(S)))`: the right-hand side of the
    // BAW boundary condition, evaluated at `S` and at the bumped `S + ΔS`.
    let boundary_rhs = |spot: Decimal| -> Result<Decimal, PricingError> {
        let d1_val = d1(spot, strike, t, r, q, sigma)?;
        let n_d1 = big_n(d1_val)?;
        let c_euro = black_scholes_european(spot, strike, t, r, q, sigma, &OptionStyle::Call)?;
        Ok(d_add(
            c_euro,
            d_mul(
                d_div(spot, q2, "pricing::american::call::boundary_scale")?,
                d_sub(
                    dec!(1),
                    d_mul(exp_qt, n_d1, "pricing::american::call::boundary_weight")?,
                    "pricing::american::call::boundary_bracket",
                )?,
                "pricing::american::call::boundary_premium",
            )?,
            "pricing::american::call::boundary_rhs",
        )?)
    };

    for _ in 0..MAX_ITERATIONS {
        // Function: f(S*) = S* - K - C_european(S*) - (S*/q2)(1 - e^(-qT) * N(d1))
        let f = d_sub(
            d_sub(s_star, strike, "pricing::american::call::lhs")?,
            boundary_rhs(s_star)?,
            "pricing::american::call::f",
        )?;

        // Derivative approximation
        let delta_s = d_mul(s_star, dec!(0.0001), "pricing::american::call::delta_s")?;
        if delta_s.is_zero() {
            // The bump underflowed below the representable scale, so no
            // derivative is available: fall back to the current estimate.
            break;
        }
        let bumped = d_add(s_star, delta_s, "pricing::american::call::bumped")?;
        let f_plus = d_sub(
            d_sub(bumped, strike, "pricing::american::call::lhs_plus")?,
            boundary_rhs(bumped)?,
            "pricing::american::call::f_plus",
        )?;

        let f_prime: Decimal = d_div(
            d_sub(f_plus, f, "pricing::american::call::df")?,
            delta_s,
            "pricing::american::call::f_prime",
        )?;

        if f_prime.abs() < dec!(1e-10) {
            break;
        }

        let s_star_new = d_sub(
            s_star,
            d_div(f, f_prime, "pricing::american::call::newton_step")?,
            "pricing::american::call::s_star_new",
        )?;

        if d_sub(s_star_new, s_star, "pricing::american::call::convergence")?.abs() < tolerance {
            return Ok(s_star_new.max(strike)); // S* must be >= K for calls
        }

        // Keep S* reasonable
        s_star = s_star_new.max(d_mul(strike, dec!(0.5), "pricing::american::call::floor")?);
    }

    // Return best estimate
    Ok(s_star.max(strike))
}

/// Finds the critical price S** for American puts using Newton-Raphson.
///
/// The critical price is where immediate exercise becomes optimal.
fn find_critical_price_put(
    _spot: Decimal,
    strike: Decimal,
    t: Decimal,
    r: Decimal,
    q: Decimal,
    sigma: Decimal,
    q1: Decimal,
) -> Result<Decimal, PricingError> {
    // Initial guess: use strike as starting point
    let mut s_star = d_mul(strike, dec!(0.9), "pricing::american::put::s_star_seed")?;
    let exp_qt = discount_factor(
        q,
        t,
        "pricing::american::put::neg_qt_seed",
        "pricing::american::put::dividend_discount_seed",
    )?;
    let tolerance = TOLERANCE;

    // `P_euro(S) - (S / q1)(1 - e^(-qT) N(-d1(S)))`: the right-hand side of the
    // BAW boundary condition, evaluated at `S` and at the bumped `S + ΔS`.
    let boundary_rhs = |spot: Decimal| -> Result<Decimal, PricingError> {
        let d1_val = d1(spot, strike, t, r, q, sigma)?;
        let n_minus_d1 = big_n(-d1_val)?;
        let p_euro = black_scholes_european(spot, strike, t, r, q, sigma, &OptionStyle::Put)?;
        Ok(d_sub(
            p_euro,
            d_mul(
                d_div(spot, q1, "pricing::american::put::boundary_scale")?,
                d_sub(
                    dec!(1),
                    d_mul(
                        exp_qt,
                        n_minus_d1,
                        "pricing::american::put::boundary_weight",
                    )?,
                    "pricing::american::put::boundary_bracket",
                )?,
                "pricing::american::put::boundary_premium",
            )?,
            "pricing::american::put::boundary_rhs",
        )?)
    };

    for _ in 0..MAX_ITERATIONS {
        // Function: f(S**) = K - S** - P_european(S**) + (S**/q1)(1 - e^(-qT) * N(-d1))
        let f = d_sub(
            d_sub(strike, s_star, "pricing::american::put::lhs")?,
            boundary_rhs(s_star)?,
            "pricing::american::put::f",
        )?;

        // Derivative approximation
        let delta_s =
            d_mul(s_star, dec!(0.0001), "pricing::american::put::delta_s")?.max(dec!(0.01)); // Ensure minimum step
        let bumped = d_add(s_star, delta_s, "pricing::american::put::bumped")?;
        let f_plus = d_sub(
            d_sub(strike, bumped, "pricing::american::put::lhs_plus")?,
            boundary_rhs(bumped)?,
            "pricing::american::put::f_plus",
        )?;

        let f_prime: Decimal = d_div(
            d_sub(f_plus, f, "pricing::american::put::df")?,
            delta_s,
            "pricing::american::put::f_prime",
        )?;

        if f_prime.abs() < dec!(1e-10) {
            break;
        }

        let s_star_new = d_sub(
            s_star,
            d_div(f, f_prime, "pricing::american::put::newton_step")?,
            "pricing::american::put::s_star_new",
        )?;

        if d_sub(s_star_new, s_star, "pricing::american::put::convergence")?.abs() < tolerance {
            return Ok(s_star_new.max(dec!(0.01)).min(strike)); // 0 < S** <= K for puts
        }

        // Keep S** reasonable
        s_star = s_star_new.max(dec!(0.01)).min(d_mul(
            strike,
            dec!(1.5),
            "pricing::american::put::ceiling",
        )?);
    }

    // Return best estimate
    Ok(s_star.max(dec!(0.01)).min(strike))
}

#[cfg(test)]
mod tests_american_pricing {
    use super::*;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use optionstratlib_core::pos_or_panic;

    #[test]
    fn test_baw_call_at_expiry() {
        // At expiration, should return intrinsic value
        let price = barone_adesi_whaley(
            pos_or_panic!(110.0),
            Positive::HUNDRED,
            Positive::ZERO,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(0.2),
            &OptionStyle::Call,
        )
        .unwrap();

        assert_relative_eq!(price.to_f64().unwrap(), 10.0, epsilon = 0.01);
    }

    #[test]
    fn test_baw_put_at_expiry() {
        // At expiration, should return intrinsic value
        let price = barone_adesi_whaley(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            Positive::ZERO,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(0.2),
            &OptionStyle::Put,
        )
        .unwrap();

        assert_relative_eq!(price.to_f64().unwrap(), 10.0, epsilon = 0.01);
    }

    #[test]
    fn test_baw_call_no_dividend() {
        // American call on non-dividend stock equals European call
        let price = barone_adesi_whaley(
            Positive::HUNDRED,
            Positive::HUNDRED,
            Positive::ONE,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(0.2),
            &OptionStyle::Call,
        )
        .unwrap();

        // Should be close to Black-Scholes European call (~10.45)
        assert!(price.to_f64().unwrap() > 9.0);
        assert!(price.to_f64().unwrap() < 12.0);
    }

    #[test]
    fn test_baw_put_has_early_exercise_premium() {
        // American put should be worth more than European put
        let american_put = barone_adesi_whaley(
            Positive::HUNDRED,
            Positive::HUNDRED,
            Positive::ONE,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(0.2),
            &OptionStyle::Put,
        )
        .unwrap();

        let european_put = black_scholes_european(
            dec!(100),
            dec!(100),
            dec!(1),
            dec!(0.05),
            dec!(0),
            dec!(0.2),
            &OptionStyle::Put,
        )
        .unwrap();

        // American put >= European put
        assert!(american_put >= european_put);
    }

    #[test]
    fn test_baw_deep_itm_put() {
        // Deep ITM put should be close to intrinsic value
        let price = barone_adesi_whaley(
            pos_or_panic!(50.0),
            Positive::HUNDRED,
            Positive::ONE,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(0.2),
            &OptionStyle::Put,
        )
        .unwrap();

        // Should be at least intrinsic value (50)
        assert!(price.to_f64().unwrap() >= 49.0);
    }

    #[test]
    fn test_baw_call_with_dividend() {
        // American call with dividend should have early exercise premium
        let american_call = barone_adesi_whaley(
            Positive::HUNDRED,
            Positive::HUNDRED,
            Positive::ONE,
            dec!(0.05),
            pos_or_panic!(0.03), // 3% dividend yield
            pos_or_panic!(0.2),
            &OptionStyle::Call,
        )
        .unwrap();

        // Should be positive
        assert!(american_call.to_f64().unwrap() > 0.0);
    }

    #[test]
    fn test_baw_zero_volatility() {
        // Zero volatility should give deterministic price
        let price = barone_adesi_whaley(
            pos_or_panic!(110.0),
            Positive::HUNDRED,
            Positive::ONE,
            dec!(0.05),
            Positive::ZERO,
            Positive::ZERO,
            &OptionStyle::Call,
        )
        .unwrap();

        // Should be positive for ITM call
        assert!(price.to_f64().unwrap() > 0.0);
    }

    #[test]
    fn test_baw_otm_call() {
        // OTM call should have positive time value
        let price = barone_adesi_whaley(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            Positive::ONE,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(0.2),
            &OptionStyle::Call,
        )
        .unwrap();

        // Should be positive (time value)
        assert!(price.to_f64().unwrap() > 0.0);
        // But less than ATM
        assert!(price.to_f64().unwrap() < 10.0);
    }

    #[test]
    fn test_baw_otm_put() {
        // OTM put should have positive time value
        let price = barone_adesi_whaley(
            pos_or_panic!(110.0),
            Positive::HUNDRED,
            Positive::ONE,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(0.2),
            &OptionStyle::Put,
        )
        .unwrap();

        // Should be positive (time value)
        assert!(price.to_f64().unwrap() > 0.0);
    }
}
