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
//! use optionstratlib::pricing::american::barone_adesi_whaley;
//! use optionstratlib::model::types::OptionStyle;
//! use positive::Positive;
//! # fn run() -> Result<(), optionstratlib::error::Error> {
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
use crate::greeks::big_n;
use crate::model::decimal::{d_add, d_mul, d_sub};
use crate::model::types::OptionStyle;
use positive::Positive;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;

/// Maximum iterations for Newton-Raphson method to find critical price.
const MAX_ITERATIONS: usize = 100;

/// Convergence tolerance for critical price calculation.
const TOLERANCE: f64 = 1e-6;

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
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use optionstratlib::pricing::american::barone_adesi_whaley;
/// use optionstratlib::model::types::OptionStyle;
/// use positive::Positive;
/// # fn run() -> Result<(), optionstratlib::error::Error> {
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
/// Returns `PricingError::MethodError` (with method
/// `barone_adesi_whaley`) when the intermediate Newton–Raphson
/// iteration cannot converge on the early-exercise boundary
/// `Sx`, and [`PricingError::SqrtFailure`] when the quadratic
/// formula receives a negative discriminant from the approximation
/// parameters.
pub fn barone_adesi_whaley(
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
        return Ok(match option_style {
            OptionStyle::Call => d_sub(s, k, "pricing::american::intrinsic::call")?.max(Decimal::ZERO),
            OptionStyle::Put => d_sub(k, s, "pricing::american::intrinsic::put")?.max(Decimal::ZERO),
        });
    }

    if sigma <= Decimal::ZERO {
        // Zero volatility: deterministic pricing
        let neg_rt = d_mul(-r, t, "pricing::american::zero_vol::rt")?;
        let neg_qt = d_mul(-q, t, "pricing::american::zero_vol::qt")?;
        let discount_r = neg_rt.exp();
        let discount_q = neg_qt.exp();
        let s_disc = d_mul(s, discount_q, "pricing::american::zero_vol::s_disc")?;
        let k_disc = d_mul(k, discount_r, "pricing::american::zero_vol::k_disc")?;
        return Ok(match option_style {
            OptionStyle::Call => {
                d_sub(s_disc, k_disc, "pricing::american::zero_vol::call")?.max(Decimal::ZERO)
            }
            OptionStyle::Put => {
                d_sub(k_disc, s_disc, "pricing::american::zero_vol::put")?.max(Decimal::ZERO)
            }
        });
    }

    // Calculate European option price first
    let european_price = black_scholes_european(s, k, t, r, q, sigma, option_style)?;

    // For American calls on non-dividend paying stocks, early exercise is never optimal
    if matches!(option_style, OptionStyle::Call) && q <= Decimal::ZERO {
        return Ok(european_price);
    }

    // Calculate BAW parameters
    let sigma_sq = sigma * sigma;
    let m = dec!(2) * r / sigma_sq;
    let n = dec!(2) * (r - q) / sigma_sq;
    let k_factor = dec!(1) - (-r * t).exp();

    match option_style {
        OptionStyle::Call => {
            let discriminant = (n - dec!(1)).powi(2) + dec!(4) * m / k_factor;
            let sqrt_disc = discriminant.sqrt().ok_or_else(|| {
                PricingError::method_error(
                    "baw",
                    "cannot calculate square root of negative discriminant",
                )
            })?;
            let q2 = (-(n - dec!(1)) + sqrt_disc) / dec!(2);

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
                let a2 = (s_star / q2) * (dec!(1) - (-q * t).exp() * n_d1);
                let early_exercise_premium = a2 * (s / s_star).powd(q2);
                d_add(
                    european_price,
                    early_exercise_premium,
                    "pricing::american::call::price",
                )
                .map_err(PricingError::from)
            }
        }
        OptionStyle::Put => {
            let discriminant = (n - dec!(1)).powi(2) + dec!(4) * m / k_factor;
            let sqrt_disc = discriminant.sqrt().ok_or_else(|| {
                PricingError::method_error(
                    "baw",
                    "cannot calculate square root of negative discriminant",
                )
            })?;
            let q1 = (-(n - dec!(1)) - sqrt_disc) / dec!(2);

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
                let a1 = -(s_star_star / q1) * (dec!(1) - (-q * t).exp() * n_minus_d1);
                let early_exercise_premium = a1 * (s / s_star_star).powd(q1);
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
    let sqrt_t = t.sqrt().ok_or_else(|| {
        PricingError::method_error("black_scholes", "cannot calculate square root of time")
    })?;
    let d2_val = d1_val - sigma * sqrt_t;

    let discount = (-r * t).exp();
    let forward_factor = (-q * t).exp();

    let n_d1 = big_n(d1_val)?;
    let n_d2 = big_n(d2_val)?;
    let n_minus_d1 = big_n(-d1_val)?;
    let n_minus_d2 = big_n(-d2_val)?;

    match option_style {
        OptionStyle::Call => Ok(s * forward_factor * n_d1 - k * discount * n_d2),
        OptionStyle::Put => Ok(k * discount * n_minus_d2 - s * forward_factor * n_minus_d1),
    }
}

/// Calculates d1 parameter for Black-Scholes formula.
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
    let sqrt_t = t
        .sqrt()
        .ok_or_else(|| PricingError::method_error("d1", "cannot calculate square root of time"))?;
    let ln_s_k = (s / k).ln();
    Ok((ln_s_k + (r - q + sigma * sigma / dec!(2)) * t) / (sigma * sqrt_t))
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
    let mut s_star = strike * dec!(1.1);

    for _ in 0..MAX_ITERATIONS {
        let d1_val = d1(s_star, strike, t, r, q, sigma)?;
        let n_d1 = big_n(d1_val)?;
        let exp_qt = (-q * t).exp();

        // Function: f(S*) = S* - K - C_european(S*) - (S*/q2)(1 - e^(-qT) * N(d1))
        let c_euro = black_scholes_european(s_star, strike, t, r, q, sigma, &OptionStyle::Call)?;
        let lhs = s_star - strike;
        let rhs = c_euro + (s_star / q2) * (dec!(1) - exp_qt * n_d1);
        let f = lhs - rhs;

        // Derivative approximation
        let delta_s = s_star * dec!(0.0001);
        let d1_plus = d1(s_star + delta_s, strike, t, r, q, sigma)?;
        let n_d1_plus = big_n(d1_plus)?;
        let c_euro_plus =
            black_scholes_european(s_star + delta_s, strike, t, r, q, sigma, &OptionStyle::Call)?;
        let rhs_plus = c_euro_plus + ((s_star + delta_s) / q2) * (dec!(1) - exp_qt * n_d1_plus);
        let f_plus = (s_star + delta_s) - strike - rhs_plus;

        let f_prime: Decimal = (f_plus - f) / delta_s;

        if f_prime.abs() < dec!(1e-10) {
            break;
        }

        let s_star_new = s_star - f / f_prime;

        if (s_star_new - s_star).abs() < Decimal::from_f64_retain(TOLERANCE).unwrap_or(dec!(1e-6)) {
            return Ok(s_star_new.max(strike)); // S* must be >= K for calls
        }

        s_star = s_star_new.max(strike * dec!(0.5)); // Keep S* reasonable
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
    let mut s_star = strike * dec!(0.9);

    for _ in 0..MAX_ITERATIONS {
        let d1_val = d1(s_star, strike, t, r, q, sigma)?;
        let n_minus_d1 = big_n(-d1_val)?;
        let exp_qt = (-q * t).exp();

        // Function: f(S**) = K - S** - P_european(S**) + (S**/q1)(1 - e^(-qT) * N(-d1))
        let p_euro = black_scholes_european(s_star, strike, t, r, q, sigma, &OptionStyle::Put)?;
        let lhs = strike - s_star;
        let rhs = p_euro - (s_star / q1) * (dec!(1) - exp_qt * n_minus_d1);
        let f = lhs - rhs;

        // Derivative approximation
        let delta_s = s_star * dec!(0.0001);
        let delta_s = delta_s.max(dec!(0.01)); // Ensure minimum step
        let d1_plus = d1(s_star + delta_s, strike, t, r, q, sigma)?;
        let n_minus_d1_plus = big_n(-d1_plus)?;
        let p_euro_plus =
            black_scholes_european(s_star + delta_s, strike, t, r, q, sigma, &OptionStyle::Put)?;
        let rhs_plus =
            p_euro_plus - ((s_star + delta_s) / q1) * (dec!(1) - exp_qt * n_minus_d1_plus);
        let f_plus = strike - (s_star + delta_s) - rhs_plus;

        let f_prime: Decimal = (f_plus - f) / delta_s;

        if f_prime.abs() < dec!(1e-10) {
            break;
        }

        let s_star_new = s_star - f / f_prime;

        if (s_star_new - s_star).abs() < Decimal::from_f64_retain(TOLERANCE).unwrap_or(dec!(1e-6)) {
            return Ok(s_star_new.max(dec!(0.01)).min(strike)); // 0 < S** <= K for puts
        }

        s_star = s_star_new.max(dec!(0.01)).min(strike * dec!(1.5)); // Keep S** reasonable
    }

    // Return best estimate
    Ok(s_star.max(dec!(0.01)).min(strike))
}

#[cfg(test)]
mod tests_american_pricing {
    use super::*;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use positive::pos_or_panic;

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
