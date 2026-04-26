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
//! `Options` carries a single risk-free rate plus a `dividend_yield`. For
//! Garman–Kohlhagen we reuse those fields with the FX interpretation:
//!
//! - `Options::risk_free_rate`  — domestic risk-free rate `r_d`.
//! - `Options::dividend_yield`  — foreign risk-free rate `r_f`.
//! - `Options::underlying_price` — spot FX rate `S`.
//!
//! No schema change is required. The mapping is intentional: GK is the
//! standard textbook reduction of BSM under the FX interpretation, and
//! delegating to [`crate::pricing::black_scholes_model::black_scholes`]
//! guarantees a bit-exact equivalence to the BSM kernel.

use crate::Options;
use crate::error::PricingError;
use crate::model::types::OptionType;
use crate::pricing::black_scholes_model::black_scholes;
use rust_decimal::Decimal;
use tracing::instrument;

/// Computes the price of a European FX option using the Garman–Kohlhagen
/// (1983) closed-form model.
///
/// # Arguments
///
/// * `option` — `Options` with the FX field interpretation. The
///   `underlying_price` field carries the spot FX rate `S`,
///   `risk_free_rate` carries the domestic rate `r_d`, and
///   `dividend_yield` carries the foreign rate `r_f`.
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
/// `q = r_f`, and the implementation delegates to the existing BSM
/// kernel after validating the option type. The FX put-call parity
/// reduces to:
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
    r_f = %option.dividend_yield,
))]
pub fn garman_kohlhagen(option: &Options) -> Result<Decimal, PricingError> {
    match option.option_type {
        OptionType::European => black_scholes(option),
        OptionType::American => Err(PricingError::unsupported_option_type(
            "American",
            "Garman-Kohlhagen",
        )),
        OptionType::Bermuda { .. } => Err(PricingError::unsupported_option_type(
            "Bermuda",
            "Garman-Kohlhagen",
        )),
        OptionType::Asian { .. } => Err(PricingError::unsupported_option_type(
            "Asian",
            "Garman-Kohlhagen",
        )),
        OptionType::Barrier { .. } => Err(PricingError::unsupported_option_type(
            "Barrier",
            "Garman-Kohlhagen",
        )),
        OptionType::Binary { .. } => Err(PricingError::unsupported_option_type(
            "Binary",
            "Garman-Kohlhagen",
        )),
        OptionType::Lookback { .. } => Err(PricingError::unsupported_option_type(
            "Lookback",
            "Garman-Kohlhagen",
        )),
        OptionType::Compound { .. } => Err(PricingError::unsupported_option_type(
            "Compound",
            "Garman-Kohlhagen",
        )),
        OptionType::Chooser { .. } => Err(PricingError::unsupported_option_type(
            "Chooser",
            "Garman-Kohlhagen",
        )),
        OptionType::Cliquet { .. } => Err(PricingError::unsupported_option_type(
            "Cliquet",
            "Garman-Kohlhagen",
        )),
        OptionType::Rainbow { .. } => Err(PricingError::unsupported_option_type(
            "Rainbow",
            "Garman-Kohlhagen",
        )),
        OptionType::Spread { .. } => Err(PricingError::unsupported_option_type(
            "Spread",
            "Garman-Kohlhagen",
        )),
        OptionType::Quanto { .. } => Err(PricingError::unsupported_option_type(
            "Quanto",
            "Garman-Kohlhagen",
        )),
        OptionType::Exchange { .. } => Err(PricingError::unsupported_option_type(
            "Exchange",
            "Garman-Kohlhagen",
        )),
        OptionType::Power { .. } => Err(PricingError::unsupported_option_type(
            "Power",
            "Garman-Kohlhagen",
        )),
    }
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
    use crate::ExpirationDate;
    use crate::model::types::{OptionStyle, Side};
    use crate::pricing::unified::{PricingEngine, price_option};
    use positive::{Positive, pos_or_panic};
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
        let via_engine = price_option(&option, &PricingEngine::ClosedFormGK).unwrap();
        assert_eq!(via_engine.to_dec(), direct);
    }

    #[test]
    fn test_pricing_engine_closed_form_gk_dispatch_short_uses_abs() {
        let mut option =
            create_fx_option(1.25, 1.2, dec!(0.05), 0.03, 180.0, 0.15, OptionStyle::Call);
        option.side = Side::Short;
        let direct = garman_kohlhagen(&option).unwrap();
        let via_engine = price_option(&option, &PricingEngine::ClosedFormGK).unwrap();
        assert_eq!(via_engine.to_dec(), direct.abs());
    }
}
