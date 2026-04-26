/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-04-26
******************************************************************************/
use crate::Options;
use crate::error::PricingError;
use crate::greeks::{big_n, calculate_d_values_black_76};
use crate::model::decimal::{d_mul, d_sub};
use crate::model::types::{OptionStyle, OptionType, Side};
use rust_decimal::{Decimal, MathematicalOps};
use tracing::{instrument, trace};

/// Computes the price of an option on a futures contract using the Black-76 model.
///
/// # Arguments
///
/// * `option` - An `Options` struct where `underlying_price` contains the forward/futures price F
///   (not the spot price S). All other parameters have their standard meanings.
///
/// # Returns
///
/// * `Ok(Decimal)` - The calculated price of the option.
/// * `Err(PricingError)` - If the option type is not supported or calculation fails.
///
/// # Supported Option Types
///
/// Only **European** options are supported. American, Bermuda, exotic variants return
/// `PricingError::UnsupportedOptionType`.
///
/// # Description
///
/// The Black-76 model (Black 1976) is the standard closed-form for pricing options on:
/// - Futures contracts
/// - Forward contracts
/// - Swaptions
/// - Caps/floors
/// - Commodity futures options
///
/// The key difference vs Black-Scholes is that the input is forward price F (not spot S),
/// and there is no carry term because F already incorporates all carry. Both legs are
/// discounted by the same factor e^(−rT).
///
/// ## Black-76 Formula
///
/// For a **Call** option:
/// ```text
/// C = e^(-rT) * [F * N(d1) - K * N(d2)]
/// ```
///
/// For a **Put** option:
/// ```text
/// P = e^(-rT) * [K * N(-d2) - F * N(-d1)]
/// ```
///
/// Where:
/// - `F` = Forward/futures price (passed as `underlying_price`)
/// - `K` = Strike price
/// - `r` = Risk-free interest rate
/// - `T` = Time to expiration (in years)
/// - `N()` = Cumulative standard normal distribution
/// - `d1 = [ln(F/K) + σ²T/2] / (σ√T)`
/// - `d2 = d1 − σ√T`
///
/// Note: Unlike Black-Scholes, `d1` and `d2` do NOT include the risk-free rate term
/// because the drift is zero (forward pricing).
///
/// # Errors
///
/// Returns `PricingError::ExpirationDate` when expiration cannot be converted to a
/// positive year fraction, `PricingError::MethodError` when d1/d2 evaluation fails
/// (e.g., zero volatility), and `PricingError::UnsupportedOptionType` for non-European types.
///
/// # Examples
///
/// ```rust
/// use optionstratlib::{Options, ExpirationDate};
/// use optionstratlib::model::types::{OptionStyle, OptionType, Side};
/// use optionstratlib::pricing::black_76;
/// use positive::{Positive, pos_or_panic};
/// use rust_decimal_macros::dec;
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_symbol: "ES".to_string(),
///     strike_price: pos_or_panic!(4000.0),
///     underlying_price: pos_or_panic!(4100.0),  // Forward price F, not spot
///     expiration_date: ExpirationDate::Days(pos_or_panic!(90.0)),
///     implied_volatility: pos_or_panic!(0.15),
///     quantity: Positive::ONE,
///     risk_free_rate: dec!(0.05),
///     option_style: OptionStyle::Call,
///     dividend_yield: pos_or_panic!(0.0),
///     exotic_params: None,
/// };
/// let price = black_76(&option)?;
/// # Ok::<(), optionstratlib::error::PricingError>(())
/// ```
#[instrument(skip(option), fields(
    strike = %option.strike_price,
    style = ?option.option_style,
    side = ?option.side,
))]
pub fn black_76(option: &Options) -> Result<Decimal, PricingError> {
    let (d1, d2, expiry_time) = calculate_d1_d2_and_time(option)?;
    match option.option_type {
        OptionType::European => calculate_european_option_price(option, d1, d2, expiry_time),
        OptionType::American => Err(PricingError::unsupported_option_type(
            "American", "Black-76",
        )),
        OptionType::Bermuda { .. } => {
            Err(PricingError::unsupported_option_type("Bermuda", "Black-76"))
        }
        _ => Err(PricingError::unsupported_option_type("exotic", "Black-76")),
    }
}

fn calculate_d1_d2_and_time(option: &Options) -> Result<(Decimal, Decimal, Decimal), PricingError> {
    let calculated_time_to_expiry: Decimal = option.time_to_expiration()?.to_dec();
    let (d1, d2) = calculate_d_values_black_76(option)?;
    Ok((d1, d2, calculated_time_to_expiry))
}

fn calculate_european_option_price(
    option: &Options,
    d1: Decimal,
    d2: Decimal,
    expiry_time: Decimal,
) -> Result<Decimal, PricingError> {
    match option.side {
        Side::Long => calculate_long_position(option, d1, d2, expiry_time),
        Side::Short => Ok(-calculate_long_position(option, d1, d2, expiry_time)?),
    }
}

fn calculate_long_position(
    option: &Options,
    d1: Decimal,
    d2: Decimal,
    expiry_time: Decimal,
) -> Result<Decimal, PricingError> {
    match option.option_style {
        OptionStyle::Call => calculate_call_option_price(option, d1, d2, expiry_time),
        OptionStyle::Put => calculate_put_option_price(option, d1, d2, expiry_time),
    }
}

fn calculate_call_option_price(
    option: &Options,
    d1: Decimal,
    d2: Decimal,
    t: Decimal,
) -> Result<Decimal, PricingError> {
    let big_n_d1 = big_n(d1)?;
    let big_n_d2 = big_n(d2)?;

    // e^(-rT) * [F * N(d1) - K * N(d2)]
    let rt = d_mul(-option.risk_free_rate, t, "pricing::black_76::call::rt")?;
    let discount_factor = rt.exp();

    let f_leg = d_mul(
        option.underlying_price.to_dec(),
        big_n_d1,
        "pricing::black_76::call::f_leg",
    )?;
    let k_leg = d_mul(
        option.strike_price.to_dec(),
        big_n_d2,
        "pricing::black_76::call::k_leg",
    )?;
    let undiscounted = d_sub(f_leg, k_leg, "pricing::black_76::call::undiscounted")?;
    let result = d_mul(
        discount_factor,
        undiscounted,
        "pricing::black_76::call::price",
    )?;

    trace!(
        "Black-76 Call: F={}, K={}, e^(-rT)={}, N(d1)={}, N(d2)={}, price={}",
        option.underlying_price, option.strike_price, discount_factor, big_n_d1, big_n_d2, result
    );
    Ok(result)
}

fn calculate_put_option_price(
    option: &Options,
    d1: Decimal,
    d2: Decimal,
    t: Decimal,
) -> Result<Decimal, PricingError> {
    let big_n_neg_d1 = big_n(-d1)?;
    let big_n_neg_d2 = big_n(-d2)?;

    // e^(-rT) * [K * N(-d2) - F * N(-d1)]
    let rt = d_mul(-option.risk_free_rate, t, "pricing::black_76::put::rt")?;
    let discount_factor = rt.exp();

    let k_leg = d_mul(
        option.strike_price.to_dec(),
        big_n_neg_d2,
        "pricing::black_76::put::k_leg",
    )?;
    let f_leg = d_mul(
        option.underlying_price.to_dec(),
        big_n_neg_d1,
        "pricing::black_76::put::f_leg",
    )?;
    let undiscounted = d_sub(k_leg, f_leg, "pricing::black_76::put::undiscounted")?;
    let result = d_mul(
        discount_factor,
        undiscounted,
        "pricing::black_76::put::price",
    )?;

    trace!(
        "Black-76 Put: F={}, K={}, e^(-rT)={}, N(-d1)={}, N(-d2)={}, price={}",
        option.underlying_price,
        option.strike_price,
        discount_factor,
        big_n_neg_d1,
        big_n_neg_d2,
        result
    );
    Ok(result)
}

/// Trait for types that can be priced using the Black-76 model.
///
/// This trait provides a unified interface for pricing options on futures/forwards
/// via Black-76. Implementors provide access to their `Options` data.
pub trait Black76 {
    /// Retrieves the option data for Black-76 pricing.
    fn get_option(&self) -> Result<&Options, PricingError>;

    /// Calculates the Black-76 price using the default implementation.
    fn calculate_price_black_76(&self) -> Result<Decimal, PricingError> {
        black_76(self.get_option()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::pricing::black_scholes_model::black_scholes;
    use crate::pricing::unified::{PricingEngine, price_option};
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

    #[test]
    fn test_black_76_call_hull_reference() {
        // Hull "Options, Futures and Other Derivatives" example reference
        // F=20, K=20, r=0.09, T≈4/12 years, sigma=0.25 -> call ≈ 1.1166
        // Note: Hull uses year fractions; ExpirationDate::Days(121.67) ≈ 4/12 years
        let option = create_option(20.0, 20.0, dec!(0.09), 121.67, 0.25, OptionStyle::Call);
        let price = black_76(&option).unwrap();
        // Should be close to 1.1166 but allow tolerance for rounding differences
        let expected = dec!(1.1166);
        let tolerance = dec!(0.008);
        assert!(
            (price - expected).abs() <= tolerance,
            "Black-76 call price {} outside tolerance of {} from expected {}",
            price,
            tolerance,
            expected
        );
    }

    #[test]
    fn test_black_76_put_hull_reference() {
        // At-the-money: F=K -> C=P (both approximately 1.1166)
        let call_option = create_option(20.0, 20.0, dec!(0.09), 120.0, 0.25, OptionStyle::Call);
        let put_option = create_option(20.0, 20.0, dec!(0.09), 120.0, 0.25, OptionStyle::Put);

        let call_price = black_76(&call_option).unwrap();
        let put_price = black_76(&put_option).unwrap();

        // At-the-money, call and put should be approximately equal
        let diff = (call_price - put_price).abs();
        assert!(diff < dec!(0.01), "ATM call-put diff too large: {}", diff);
    }

    #[test]
    fn test_black_76_put_call_parity_atm() {
        // C - P = e^(-rT) * (F - K)
        let f = 100.0;
        let k = 100.0;
        let r = dec!(0.05);
        let t_days = 180.0;
        let sigma = 0.2;

        let call_option = create_option(f, k, r, t_days, sigma, OptionStyle::Call);
        let put_option = create_option(f, k, r, t_days, sigma, OptionStyle::Put);

        let call_price = black_76(&call_option).unwrap();
        let put_price = black_76(&put_option).unwrap();

        let t = dec!(180) / dec!(365);
        let rt = -r * t;
        let expected_diff = rt.exp() * (dec!(100) - dec!(100));

        let actual_diff = call_price - put_price;
        let tolerance = dec!(0.000001);
        assert!(
            (actual_diff - expected_diff).abs() < tolerance,
            "Parity failed: C-P={}, expected={}",
            actual_diff,
            expected_diff
        );
    }

    #[test]
    fn test_black_76_put_call_parity_itm() {
        // C - P = e^(-rT) * (F - K) for ITM
        let f = 110.0;
        let k = 100.0;
        let r = dec!(0.05);
        let t_days = 180.0;
        let sigma = 0.2;

        let call_option = create_option(f, k, r, t_days, sigma, OptionStyle::Call);
        let put_option = create_option(f, k, r, t_days, sigma, OptionStyle::Put);

        let call_price = black_76(&call_option).unwrap();
        let put_price = black_76(&put_option).unwrap();

        let t = dec!(180) / dec!(365);
        let rt = -r * t;
        let expected_diff = rt.exp() * (dec!(110) - dec!(100));

        let actual_diff = call_price - put_price;
        let tolerance = dec!(0.000001);
        assert!(
            (actual_diff - expected_diff).abs() < tolerance,
            "ITM parity failed: C-P={}, expected={}",
            actual_diff,
            expected_diff
        );
    }

    #[test]
    fn test_black_76_put_call_parity_otm() {
        // C - P = e^(-rT) * (F - K) for OTM
        let f = 90.0;
        let k = 100.0;
        let r = dec!(0.05);
        let t_days = 180.0;
        let sigma = 0.2;

        let call_option = create_option(f, k, r, t_days, sigma, OptionStyle::Call);
        let put_option = create_option(f, k, r, t_days, sigma, OptionStyle::Put);

        let call_price = black_76(&call_option).unwrap();
        let put_price = black_76(&put_option).unwrap();

        let t = dec!(180) / dec!(365);
        let rt = -r * t;
        let expected_diff = rt.exp() * (dec!(90) - dec!(100));

        let actual_diff = call_price - put_price;
        let tolerance = dec!(0.000001);
        assert!(
            (actual_diff - expected_diff).abs() < tolerance,
            "OTM parity failed: C-P={}, expected={}",
            actual_diff,
            expected_diff
        );
    }

    #[test]
    fn test_black_76_zero_volatility_returns_error() {
        let option = create_option(100.0, 100.0, dec!(0.05), 180.0, 0.0, OptionStyle::Call);
        let result = black_76(&option);
        assert!(result.is_err(), "Zero volatility should return error");
    }

    #[test]
    fn test_black_76_monotonicity_call_in_forward() {
        let r = dec!(0.05);
        let t_days = 180.0;
        let k = 100.0;
        let sigma = 0.2;

        let opt_f_low = create_option(95.0, k, r, t_days, sigma, OptionStyle::Call);
        let opt_f_mid = create_option(100.0, k, r, t_days, sigma, OptionStyle::Call);
        let opt_f_high = create_option(105.0, k, r, t_days, sigma, OptionStyle::Call);

        let p_low = black_76(&opt_f_low).unwrap();
        let p_mid = black_76(&opt_f_mid).unwrap();
        let p_high = black_76(&opt_f_high).unwrap();

        assert!(p_low < p_mid, "Call price should increase as F increases");
        assert!(p_mid < p_high, "Call price should increase as F increases");
    }

    #[test]
    fn test_black_76_monotonicity_put_in_forward() {
        let r = dec!(0.05);
        let t_days = 180.0;
        let k = 100.0;
        let sigma = 0.2;

        let opt_f_low = create_option(95.0, k, r, t_days, sigma, OptionStyle::Put);
        let opt_f_mid = create_option(100.0, k, r, t_days, sigma, OptionStyle::Put);
        let opt_f_high = create_option(105.0, k, r, t_days, sigma, OptionStyle::Put);

        let p_low = black_76(&opt_f_low).unwrap();
        let p_mid = black_76(&opt_f_mid).unwrap();
        let p_high = black_76(&opt_f_high).unwrap();

        assert!(p_low > p_mid, "Put price should decrease as F increases");
        assert!(p_mid > p_high, "Put price should decrease as F increases");
    }

    #[test]
    fn test_black_76_short_side_is_negation() {
        let option_long = create_option(100.0, 95.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        let mut option_short = option_long.clone();
        option_short.side = Side::Short;

        let price_long = black_76(&option_long).unwrap();
        let price_short = black_76(&option_short).unwrap();

        assert_eq!(price_long, -price_short, "Short should negate long price");
    }

    #[test]
    fn test_black_76_quantity_invariance() {
        // Black-76 price is per-contract; quantity is separate concern
        let mut option = create_option(100.0, 95.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        let price1 = black_76(&option).unwrap();

        option.quantity = pos_or_panic!(2.0);
        let price2 = black_76(&option).unwrap();

        assert_eq!(
            price1, price2,
            "Per-contract price should be quantity-invariant"
        );
    }

    #[test]
    fn test_black_76_unsupported_american() {
        let mut option = create_option(100.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        option.option_type = OptionType::American;
        let result = black_76(&option);
        assert!(result.is_err(), "American options should return error");
    }

    #[test]
    fn test_black_76_unsupported_bermuda() {
        let mut option = create_option(100.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        option.option_type = OptionType::Bermuda {
            exercise_dates: vec![],
        };
        let result = black_76(&option);
        assert!(result.is_err(), "Bermuda options should return error");
    }

    #[test]
    fn test_black_76_trait_impl() {
        struct FutureOption {
            option: Options,
        }
        impl Black76 for FutureOption {
            fn get_option(&self) -> Result<&Options, PricingError> {
                Ok(&self.option)
            }
        }

        let option = create_option(100.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
        let fut_opt = FutureOption { option };
        let price_direct = black_76(fut_opt.get_option().unwrap()).unwrap();
        let price_via_trait = fut_opt.calculate_price_black_76().unwrap();
        assert_eq!(price_direct, price_via_trait);
    }

    /// Black-76 must equal Black–Scholes–Merton with `S = F · e^(-rT)` and `q = 0`.
    /// This is the structural equivalence that grounds Black-76 in the BSM framework.
    fn assert_matches_bsm(f: f64, k: f64, r: Decimal, t_days: f64, sigma: f64, style: OptionStyle) {
        let opt_b76 = create_option(f, k, r, t_days, sigma, style);
        let years = opt_b76.expiration_date.get_years().unwrap().to_dec();
        let discount_factor = (-r * years).exp();
        let s_decimal = Decimal::from_f64_retain(f).unwrap() * discount_factor;
        let s = Positive::new_decimal(s_decimal).unwrap();
        let mut opt_bsm = opt_b76.clone();
        opt_bsm.underlying_price = s;
        // q = 0 already from create_option

        let price_b76 = black_76(&opt_b76).unwrap();
        let price_bsm = black_scholes(&opt_bsm).unwrap();
        let tolerance = dec!(1e-9);
        assert!(
            (price_b76 - price_bsm).abs() < tolerance,
            "Black-76 vs BSM mismatch (style={:?}, F={}, K={}, r={}, T_days={}, sigma={}): \
             B76={}, BSM={}, diff={}",
            style,
            f,
            k,
            r,
            t_days,
            sigma,
            price_b76,
            price_bsm,
            (price_b76 - price_bsm).abs()
        );
    }

    #[test]
    fn test_black_76_matches_bsm_with_discounted_spot_call_atm() {
        assert_matches_bsm(100.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
    }

    #[test]
    fn test_black_76_matches_bsm_with_discounted_spot_call_itm() {
        assert_matches_bsm(110.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
    }

    #[test]
    fn test_black_76_matches_bsm_with_discounted_spot_call_otm() {
        assert_matches_bsm(90.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Call);
    }

    #[test]
    fn test_black_76_matches_bsm_with_discounted_spot_put_atm() {
        assert_matches_bsm(100.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Put);
    }

    #[test]
    fn test_black_76_matches_bsm_with_discounted_spot_put_otm() {
        assert_matches_bsm(110.0, 100.0, dec!(0.05), 180.0, 0.2, OptionStyle::Put);
    }

    #[test]
    fn test_pricing_engine_closed_form_black_76_dispatch() {
        let option = create_option(20.0, 20.0, dec!(0.09), 122.4, 0.25, OptionStyle::Call);
        let price_via_engine = price_option(&option, &PricingEngine::ClosedFormBlack76).unwrap();
        let price_direct = black_76(&option).unwrap();
        assert_eq!(price_via_engine.to_dec(), price_direct);
    }

    #[test]
    fn test_pricing_engine_closed_form_black_76_short_uses_abs() {
        // unified::price_option converts via Positive::new_decimal(price.abs()),
        // so a short position must surface as the magnitude.
        let mut option = create_option(20.0, 20.0, dec!(0.09), 122.4, 0.25, OptionStyle::Call);
        option.side = Side::Short;
        let price_via_engine = price_option(&option, &PricingEngine::ClosedFormBlack76).unwrap();
        let price_direct = black_76(&option).unwrap();
        assert_eq!(price_via_engine.to_dec(), price_direct.abs());
    }
}
