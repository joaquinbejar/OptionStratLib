/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-04-26
******************************************************************************/

//! Garman–Kohlhagen (1983) closed-form pricing for European FX options.
//!
//! The Garman–Kohlhagen model prices European options on a foreign-exchange
//! spot rate `S` (units of domestic currency per unit of foreign). The
//! foreign currency earns interest at rate `r_f`, which acts exactly like
//! a continuous dividend yield in Black–Scholes–Merton, so structurally
//! GK ≡ BSM with `q = r_f`.
//!
//! ## Field mapping
//!
//! `Options` carries the FX inputs as follows:
//!
//! - `Options::risk_free_rate`  — domestic risk-free rate `r_d`
//!   (signed `Decimal`, may be negative).
//! - `ExoticParams::foreign_rate` — foreign risk-free rate `r_f` (signed
//!   `Decimal`, may be negative, #720). When `exotic_params` or its
//!   `foreign_rate` is `None`, `r_f` falls back to
//!   `Options::dividend_yield`, which is `Positive` and so cannot carry a
//!   negative rate; when it is set, it takes precedence and
//!   `dividend_yield` is ignored.
//! - `Options::underlying_price` — spot FX rate `S`.
//!
//! GK is the standard textbook reduction of BSM under the FX
//! interpretation: the price is the European Black–Scholes–Merton kernel
//! evaluated at `q = r_f`, so with `r_f = dividend_yield` it is bit-exact
//! with [`crate::pricing::black_scholes_model::black_scholes`].

use crate::error::PricingError;
use crate::pricing::black_scholes_model::black_scholes_european_with_yield;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::types::OptionType;
use rust_decimal::Decimal;
use tracing::instrument;

/// Computes the price of a European FX option using the Garman–Kohlhagen
/// (1983) closed-form model.
///
/// # Arguments
///
/// * `option` — `Options` with the FX field interpretation. The
///   `underlying_price` field carries the spot FX rate `S`,
///   `risk_free_rate` carries the domestic rate `r_d`, and the foreign rate
///   `r_f` is `ExoticParams::foreign_rate`, falling back to
///   `dividend_yield` when that is not set.
///
/// # Returns
///
/// * `Ok(Decimal)` — the calculated option price, with `Side::Short`
///   returning the negation of the long price.
/// * `Err(PricingError)` — see *Errors* below.
///
/// # Supported Option Types
///
/// Only [`OptionType::European`] is supported. `American`, `Bermuda`, and
/// every exotic variant return [`PricingError::UnsupportedOptionType`]
/// tagged with `method = "Garman-Kohlhagen"`.
///
/// # Description
///
/// The Garman–Kohlhagen formula prices a European FX option on a spot
/// rate `S` quoted as domestic per unit of foreign:
///
/// ```text
/// d1 = [ln(S / K) + (r_d - r_f + sigma^2 / 2) * T] / (sigma * sqrt(T))
/// d2 = d1 - sigma * sqrt(T)
///
/// Call:  C = S * e^(-r_f T) * N(d1) - K * e^(-r_d T) * N(d2)
/// Put:   P = K * e^(-r_d T) * N(-d2) - S * e^(-r_f T) * N(-d1)
/// ```
///
/// This is structurally identical to Black–Scholes–Merton with
/// `q = r_f`, and the implementation delegates to the European BSM
/// kernel at that signed `q` after validating the option type. The FX
/// put-call parity reduces to:
///
/// ```text
/// C - P = S * e^(-r_f T) - K * e^(-r_d T)
/// ```
///
/// # Errors
///
/// Returns [`PricingError::UnsupportedOptionType`] for non-European
/// option types. Forwards [`PricingError::ExpirationDate`] when the
/// expiration cannot be converted to a positive year fraction, and
/// [`PricingError::MethodError`] when the underlying BSM kernel hits a
/// numerical wall (e.g. zero volatility, non-finite intermediate value).
#[instrument(skip(option), fields(
    strike = %option.strike_price,
    style = ?option.option_style,
    side = ?option.side,
    r_d = %option.risk_free_rate,
    r_f = %foreign_rate(option),
))]
pub fn garman_kohlhagen(option: &Options) -> Result<Decimal, PricingError> {
    match option.option_type {
        OptionType::European => black_scholes_european_with_yield(option, foreign_rate(option)),
        _ => Err(PricingError::unsupported_option_type(
            "Non-European",
            "Garman-Kohlhagen",
        )),
    }
}

/// The foreign risk-free rate `r_f` Garman–Kohlhagen prices `option` with,
/// per year, continuously compounded.
///
/// `ExoticParams::foreign_rate` when it is set, which may be negative
/// (#720); otherwise `Options::dividend_yield`, the mapping this model used
/// before that field existed, which keeps every option built without it on
/// the same price. The pricer and the Garman–Kohlhagen Greeks both read the
/// rate here, so they always agree on it.
#[must_use]
#[inline]
pub(crate) fn foreign_rate(option: &Options) -> Decimal {
    option
        .exotic_params
        .as_ref()
        .and_then(|params| params.foreign_rate)
        .unwrap_or_else(|| option.dividend_yield.to_dec())
}

/// Trait for types that can be priced using the Garman–Kohlhagen model.
///
/// Mirrors the [`crate::pricing::BlackScholes`] trait pattern. Implementors
/// expose their underlying [`Options`] via [`GarmanKohlhagen::get_option`]
/// and inherit a default
/// [`GarmanKohlhagen::calculate_price_garman_kohlhagen`] implementation.
pub trait GarmanKohlhagen {
    /// Returns a reference to the option data backing this instrument.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError::MethodError`] when the implementor cannot
    /// resolve the current option (e.g. a placeholder wrapper before the
    /// option has been bound to a trade or position).
    fn get_option(&self) -> Result<&Options, PricingError>;

    /// Calculates the FX option price using Garman–Kohlhagen.
    ///
    /// # Errors
    ///
    /// Propagates any [`PricingError`] returned by
    /// [`GarmanKohlhagen::get_option`] or [`garman_kohlhagen`].
    fn calculate_price_garman_kohlhagen(&self) -> Result<Decimal, PricingError> {
        garman_kohlhagen(self.get_option()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pricing::black_scholes;
    use crate::pricing::{ClosedFormEngine, price_option_with};
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::{OptionStyle, Side};
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal::MathematicalOps;
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

    /// Hull canonical FX example: S = K = 1.6 USD/GBP, r_d = 0.08,
    /// r_f = 0.11, sigma = 0.2, T = 4/12 -> call ~= 0.0639.
    /// Encoded as `365 / 3` days so the library's `days / 365` conversion
    /// lands on T = 1/3.
    const HULL_T_DAYS: f64 = 365.0 / 3.0;

    #[test]
    fn test_garman_kohlhagen_call_hull_reference() {
        let option = create_fx_option(
            1.6,
            1.6,
            dec!(0.08),
            0.11,
            HULL_T_DAYS,
            0.2,
            OptionStyle::Call,
        );
        let price = garman_kohlhagen(&option).unwrap();
        let expected = dec!(0.0639);
        let tolerance = dec!(0.001);
        assert!(
            (price - expected).abs() < tolerance,
            "GK call price {} outside tolerance of {} from expected {}",
            price,
            tolerance,
            expected
        );
    }

    #[test]
    fn test_garman_kohlhagen_put_hull_reference() {
        // FX put-call parity: C - P = S * e^(-r_f T) - K * e^(-r_d T)
        let call = create_fx_option(
            1.6,
            1.6,
            dec!(0.08),
            0.11,
            HULL_T_DAYS,
            0.2,
            OptionStyle::Call,
        );
        let put = create_fx_option(
            1.6,
            1.6,
            dec!(0.08),
            0.11,
            HULL_T_DAYS,
            0.2,
            OptionStyle::Put,
        );
        let call_price = garman_kohlhagen(&call).unwrap();
        let put_price = garman_kohlhagen(&put).unwrap();

        let years = call.expiration_date.get_years().unwrap().to_dec();
        let discount_d = (-call.risk_free_rate * years).exp();
        let discount_f = (-call.dividend_yield.to_dec() * years).exp();
        let parity =
            call.underlying_price.to_dec() * discount_f - call.strike_price.to_dec() * discount_d;
        let actual = call_price - put_price;
        assert!(
            (actual - parity).abs() < dec!(1e-6),
            "FX parity at Hull params: C-P={}, expected={}",
            actual,
            parity
        );
    }

    /// GK must equal BSM with `q = r_f` exactly. This is the structural
    /// guarantee that grounds the wrapper.
    fn assert_matches_bsm(
        s: f64,
        k: f64,
        r_d: Decimal,
        r_f: f64,
        t_days: f64,
        sigma: f64,
        style: OptionStyle,
    ) {
        let option = create_fx_option(s, k, r_d, r_f, t_days, sigma, style);
        let gk_price = garman_kohlhagen(&option).unwrap();
        let bs_price = black_scholes(&option).unwrap();
        let tolerance = dec!(1e-9);
        assert!(
            (gk_price - bs_price).abs() < tolerance,
            "GK vs BSM mismatch (style={:?}, S={}, K={}, r_d={}, r_f={}, T_days={}, sigma={}): \
             GK={}, BSM={}, diff={}",
            style,
            s,
            k,
            r_d,
            r_f,
            t_days,
            sigma,
            gk_price,
            bs_price,
            (gk_price - bs_price).abs()
        );
    }

    #[test]
    fn test_garman_kohlhagen_matches_bsm_call_atm() {
        assert_matches_bsm(1.2, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
    }

    #[test]
    fn test_garman_kohlhagen_matches_bsm_call_itm() {
        assert_matches_bsm(1.3, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
    }

    #[test]
    fn test_garman_kohlhagen_matches_bsm_call_otm() {
        assert_matches_bsm(1.1, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
    }

    #[test]
    fn test_garman_kohlhagen_matches_bsm_put_atm() {
        assert_matches_bsm(1.2, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Put);
    }

    #[test]
    fn test_garman_kohlhagen_matches_bsm_put_otm() {
        assert_matches_bsm(1.3, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Put);
    }

    fn assert_fx_parity(
        s: f64,
        k: f64,
        r_d: Decimal,
        r_f: f64,
        t_days: f64,
        sigma: f64,
        tolerance: Decimal,
    ) {
        let call = create_fx_option(s, k, r_d, r_f, t_days, sigma, OptionStyle::Call);
        let put = create_fx_option(s, k, r_d, r_f, t_days, sigma, OptionStyle::Put);
        let call_price = garman_kohlhagen(&call).unwrap();
        let put_price = garman_kohlhagen(&put).unwrap();
        let years = call.expiration_date.get_years().unwrap().to_dec();
        let discount_d = (-call.risk_free_rate * years).exp();
        let discount_f = (-call.dividend_yield.to_dec() * years).exp();
        let expected =
            call.underlying_price.to_dec() * discount_f - call.strike_price.to_dec() * discount_d;
        let actual = call_price - put_price;
        assert!(
            (actual - expected).abs() < tolerance,
            "FX parity violation: S={}, K={}, r_d={}, r_f={}: C-P={}, expected={}",
            s,
            k,
            r_d,
            r_f,
            actual,
            expected
        );
    }

    #[test]
    fn test_garman_kohlhagen_fx_put_call_parity_atm() {
        assert_fx_parity(1.2, 1.2, dec!(0.05), 0.03, 180.0, 0.15, dec!(1e-6));
    }

    #[test]
    fn test_garman_kohlhagen_fx_put_call_parity_itm() {
        assert_fx_parity(1.3, 1.2, dec!(0.05), 0.03, 180.0, 0.15, dec!(1e-6));
    }

    #[test]
    fn test_garman_kohlhagen_fx_put_call_parity_otm() {
        assert_fx_parity(1.1, 1.2, dec!(0.05), 0.03, 180.0, 0.15, dec!(1e-6));
    }

    #[test]
    fn test_garman_kohlhagen_symmetric_rates_collapse_to_forward_parity() {
        // r_d == r_f -> C - P = e^(-r T) * (S - K)
        let r = dec!(0.05);
        let r_f = 0.05;
        let s = 1.3;
        let k = 1.2;
        let t_days = 180.0;
        let sigma = 0.15;
        let call = create_fx_option(s, k, r, r_f, t_days, sigma, OptionStyle::Call);
        let put = create_fx_option(s, k, r, r_f, t_days, sigma, OptionStyle::Put);
        let call_price = garman_kohlhagen(&call).unwrap();
        let put_price = garman_kohlhagen(&put).unwrap();
        let years = call.expiration_date.get_years().unwrap().to_dec();
        let discount = (-r * years).exp();
        let expected = discount
            * (Decimal::from_f64_retain(s).unwrap() - Decimal::from_f64_retain(k).unwrap());
        let actual = call_price - put_price;
        assert!(
            (actual - expected).abs() < dec!(1e-6),
            "Symmetric-rate parity: C-P={}, expected={}",
            actual,
            expected
        );
    }

    #[test]
    fn test_garman_kohlhagen_zero_volatility_returns_error() {
        let option = create_fx_option(1.2, 1.2, dec!(0.05), 0.03, 180.0, 0.0, OptionStyle::Call);
        let result = garman_kohlhagen(&option);
        assert!(result.is_err(), "zero vol should propagate BSM error");
    }

    #[test]
    fn test_garman_kohlhagen_monotonicity_call_in_spot() {
        let r_d = dec!(0.05);
        let r_f = 0.03;
        let k = 1.2;
        let t_days = 180.0;
        let sigma = 0.15;
        let p_low = garman_kohlhagen(&create_fx_option(
            1.1,
            k,
            r_d,
            r_f,
            t_days,
            sigma,
            OptionStyle::Call,
        ))
        .unwrap();
        let p_mid = garman_kohlhagen(&create_fx_option(
            1.2,
            k,
            r_d,
            r_f,
            t_days,
            sigma,
            OptionStyle::Call,
        ))
        .unwrap();
        let p_high = garman_kohlhagen(&create_fx_option(
            1.3,
            k,
            r_d,
            r_f,
            t_days,
            sigma,
            OptionStyle::Call,
        ))
        .unwrap();
        assert!(p_low < p_mid, "call must increase with S");
        assert!(p_mid < p_high, "call must increase with S");
    }

    #[test]
    fn test_garman_kohlhagen_monotonicity_put_in_spot() {
        let r_d = dec!(0.05);
        let r_f = 0.03;
        let k = 1.2;
        let t_days = 180.0;
        let sigma = 0.15;
        let p_low = garman_kohlhagen(&create_fx_option(
            1.1,
            k,
            r_d,
            r_f,
            t_days,
            sigma,
            OptionStyle::Put,
        ))
        .unwrap();
        let p_mid = garman_kohlhagen(&create_fx_option(
            1.2,
            k,
            r_d,
            r_f,
            t_days,
            sigma,
            OptionStyle::Put,
        ))
        .unwrap();
        let p_high = garman_kohlhagen(&create_fx_option(
            1.3,
            k,
            r_d,
            r_f,
            t_days,
            sigma,
            OptionStyle::Put,
        ))
        .unwrap();
        assert!(p_low > p_mid, "put must decrease with S");
        assert!(p_mid > p_high, "put must decrease with S");
    }

    #[test]
    fn test_garman_kohlhagen_short_side_is_negation() {
        let long = create_fx_option(1.25, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
        let mut short = long.clone();
        short.side = Side::Short;
        let p_long = garman_kohlhagen(&long).unwrap();
        let p_short = garman_kohlhagen(&short).unwrap();
        assert_eq!(p_long, -p_short);
    }

    #[test]
    fn test_garman_kohlhagen_quantity_invariance() {
        let mut option =
            create_fx_option(1.25, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
        let p1 = garman_kohlhagen(&option).unwrap();
        option.quantity = pos_or_panic!(5.0);
        let p2 = garman_kohlhagen(&option).unwrap();
        assert_eq!(p1, p2, "per-contract price must be quantity-invariant");
    }

    #[test]
    fn test_garman_kohlhagen_unsupported_american() {
        let mut option =
            create_fx_option(1.2, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
        option.option_type = OptionType::American;
        let result = garman_kohlhagen(&option);
        assert!(result.is_err());
    }

    #[test]
    fn test_garman_kohlhagen_unsupported_bermuda() {
        let mut option =
            create_fx_option(1.2, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
        option.option_type = OptionType::Bermuda {
            exercise_dates: vec![],
        };
        let result = garman_kohlhagen(&option);
        assert!(result.is_err());
    }

    #[test]
    fn test_garman_kohlhagen_trait_default_method() {
        struct FxOption {
            option: Options,
        }
        impl GarmanKohlhagen for FxOption {
            fn get_option(&self) -> Result<&Options, PricingError> {
                Ok(&self.option)
            }
        }

        let option = create_fx_option(1.25, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
        let wrapped = FxOption { option };
        let direct = garman_kohlhagen(wrapped.get_option().unwrap()).unwrap();
        let via_trait = wrapped.calculate_price_garman_kohlhagen().unwrap();
        assert_eq!(direct, via_trait);
    }

    #[test]
    fn test_pricing_engine_closed_form_gk_dispatch_long() {
        let option = create_fx_option(1.25, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
        let direct = garman_kohlhagen(&option).unwrap();
        let via_engine = price_option_with(&option, &ClosedFormEngine::ClosedFormGK).unwrap();
        assert_eq!(via_engine.to_dec(), direct);
    }

    #[test]
    fn test_pricing_engine_closed_form_gk_dispatch_short_uses_abs() {
        let mut option =
            create_fx_option(1.25, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
        option.side = Side::Short;
        let direct = garman_kohlhagen(&option).unwrap();
        let via_engine = price_option_with(&option, &ClosedFormEngine::ClosedFormGK).unwrap();
        assert_eq!(via_engine.to_dec(), direct.abs());
    }
}

/// The signed foreign rate of #720: `ExoticParams::foreign_rate` prices FX
/// options at a negative `r_f`, which `Options::dividend_yield` (a
/// `Positive`) cannot carry, and falls back to `dividend_yield` when unset.
#[cfg(test)]
mod tests_foreign_rate {
    use super::*;
    use crate::pricing::black_scholes;
    use optionstratlib_core::model::option::ExoticParams;
    use optionstratlib_core::model::types::{OptionStyle, Side};
    use optionstratlib_core::model::{ExpirationDate, Positive};
    use optionstratlib_core::pos_or_panic;
    use rust_decimal::MathematicalOps;
    use rust_decimal_macros::dec;

    /// Largest gap allowed against a closed form evaluated in `f64` and
    /// against parity: the kernel's normal CDF runs in `f64`, so agreement
    /// is to about `1e-15`; the bound leaves three orders of headroom.
    const CLOSED_FORM_TOLERANCE: Decimal = dec!(1e-12);

    /// Spot, strike, domestic rate, days to expiry and volatility of a case.
    struct Market {
        s: Decimal,
        k: Decimal,
        r_d: Decimal,
        t_days: Decimal,
        sigma: Decimal,
    }

    /// The `S = 1.2`, `K = 1.15`, `r_d = 5%`, 180-day, `σ = 15%` case.
    const EURUSD: Market = Market {
        s: dec!(1.2),
        k: dec!(1.15),
        r_d: dec!(0.05),
        t_days: dec!(180),
        sigma: dec!(0.15),
    };

    fn fx_option(
        market: &Market,
        dividend_yield: Positive,
        foreign_rate: Option<Decimal>,
        style: OptionStyle,
    ) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "USDCHF".to_string(),
            Positive::new_decimal(market.k).unwrap(),
            ExpirationDate::Days(Positive::new_decimal(market.t_days).unwrap()),
            Positive::new_decimal(market.sigma).unwrap(),
            Positive::ONE,
            Positive::new_decimal(market.s).unwrap(),
            market.r_d,
            style,
            dividend_yield,
            foreign_rate.map(|rate| ExoticParams {
                foreign_rate: Some(rate),
                ..ExoticParams::default()
            }),
        )
    }

    fn signed_rate_option(s: Decimal, r_f: Decimal, style: OptionStyle) -> Options {
        let market = Market {
            s,
            k: dec!(0.98),
            r_d: dec!(0.02),
            t_days: dec!(182.5),
            sigma: dec!(0.10),
        };
        fx_option(&market, Positive::ZERO, Some(r_f), style)
    }

    /// `S = 1.00`, `K = 0.98`, `r_d = 2%`, `r_f = -0.75%` (the SNB policy
    /// rate of 2015–2022), `σ = 10%`, `T = 0.5`. Reference values from the
    /// Garman–Kohlhagen formula evaluated independently in `f64` with
    /// Python's `statistics.NormalDist`:
    /// `d1 = [ln(S/K) + (r_d - r_f + σ²/2) T] / (σ √T)`, `d2 = d1 - σ √T`,
    /// `C = S e^(-r_f T) N(d1) - K e^(-r_d T) N(d2) = 0.047738205471098`,
    /// `P = K e^(-r_d T) N(-d2) - S e^(-r_f T) N(-d1) = 0.014230002497975`.
    #[test]
    fn test_garman_kohlhagen_negative_foreign_rate_matches_closed_form() {
        let call = signed_rate_option(dec!(1.00), dec!(-0.0075), OptionStyle::Call);
        let put = signed_rate_option(dec!(1.00), dec!(-0.0075), OptionStyle::Put);
        let call_price = garman_kohlhagen(&call).unwrap();
        let put_price = garman_kohlhagen(&put).unwrap();
        assert!(
            (call_price - dec!(0.047738205471098)).abs() < CLOSED_FORM_TOLERANCE,
            "call {call_price}"
        );
        assert!(
            (put_price - dec!(0.014230002497975)).abs() < CLOSED_FORM_TOLERANCE,
            "put {put_price}"
        );
    }

    /// FX put-call parity `C - P = S e^(-r_f T) - K e^(-r_d T)` at a
    /// negative, a zero and a positive foreign rate, in and out of the money.
    #[test]
    fn test_garman_kohlhagen_fx_parity_holds_at_signed_foreign_rates() {
        for r_f in [dec!(-0.01), Decimal::ZERO, dec!(0.03)] {
            for s in [dec!(0.90), dec!(1.00), dec!(1.10)] {
                let call = signed_rate_option(s, r_f, OptionStyle::Call);
                let put = signed_rate_option(s, r_f, OptionStyle::Put);
                let years = call.expiration_date.get_years().unwrap().to_dec();
                let parity = s * (-r_f * years).exp()
                    - call.strike_price.to_dec() * (-call.risk_free_rate * years).exp();
                let actual = garman_kohlhagen(&call).unwrap() - garman_kohlhagen(&put).unwrap();
                assert!(
                    (actual - parity).abs() < CLOSED_FORM_TOLERANCE,
                    "r_f={r_f} S={s}: C-P={actual}, parity={parity}"
                );
            }
        }
    }

    /// A lower foreign rate makes the foreign currency cheaper to hold, so
    /// the call rises and the put falls as `r_f` goes from 3% to -1%.
    #[test]
    fn test_garman_kohlhagen_call_rises_and_put_falls_as_foreign_rate_drops() {
        let price = |r_f: Decimal, style| {
            garman_kohlhagen(&signed_rate_option(dec!(1.00), r_f, style)).unwrap()
        };
        let rates = [dec!(0.03), Decimal::ZERO, dec!(-0.01)];
        for pair in rates.windows(2) {
            assert!(price(pair[1], OptionStyle::Call) > price(pair[0], OptionStyle::Call));
            assert!(price(pair[1], OptionStyle::Put) < price(pair[0], OptionStyle::Put));
        }
    }

    /// Unset, `foreign_rate` falls back to `dividend_yield`: no exotic
    /// params, exotic params without a foreign rate, and a foreign rate
    /// equal to the yield all price bit for bit as Black–Scholes–Merton
    /// with `q = dividend_yield`, as before #720.
    #[test]
    fn test_garman_kohlhagen_unset_foreign_rate_falls_back_to_dividend_yield() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let base = fx_option(&EURUSD, pos_or_panic!(0.03), None, style);
            let reference = black_scholes(&base).unwrap();
            assert_eq!(garman_kohlhagen(&base).unwrap(), reference);

            let mut empty = base.clone();
            empty.exotic_params = Some(ExoticParams::default());
            assert_eq!(garman_kohlhagen(&empty).unwrap(), reference);

            let mut same = base.clone();
            same.exotic_params = Some(ExoticParams {
                foreign_rate: Some(dec!(0.03)),
                ..ExoticParams::default()
            });
            assert_eq!(garman_kohlhagen(&same).unwrap(), reference);
        }
    }

    /// A set `foreign_rate` takes precedence: with `dividend_yield = 5%` and
    /// `foreign_rate = 3%` the price is the one at `r_f = 3%`.
    #[test]
    fn test_garman_kohlhagen_foreign_rate_takes_precedence_over_dividend_yield() {
        let both = fx_option(
            &EURUSD,
            pos_or_panic!(0.05),
            Some(dec!(0.03)),
            OptionStyle::Call,
        );
        let yield_only = fx_option(&EURUSD, pos_or_panic!(0.03), None, OptionStyle::Call);
        assert_eq!(
            garman_kohlhagen(&both).unwrap(),
            garman_kohlhagen(&yield_only).unwrap()
        );
    }

    /// The short side of a negative-rate option is the negated long price.
    #[test]
    fn test_garman_kohlhagen_negative_foreign_rate_short_is_negated_long() {
        let long = signed_rate_option(dec!(1.00), dec!(-0.0075), OptionStyle::Call);
        let mut short = long.clone();
        short.side = Side::Short;
        assert_eq!(
            garman_kohlhagen(&short).unwrap(),
            -garman_kohlhagen(&long).unwrap()
        );
    }
}
