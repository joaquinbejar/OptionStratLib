//! Property-based tests for panic freedom
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it. Every failure must
//! therefore come back as a `Result`, including for inputs that are extreme
//! but structurally valid.
//!
//! These tests drive the Greeks, the three closed-form pricing models and the
//! chain builder across the edges of the `Decimal` domain — volatilities and
//! strikes at the smallest representable scale, times to expiry and prices at
//! the largest, rates far outside anything a market quotes. The assertion is
//! deliberately weak: whatever comes back, it must come back.

use optionstratlib::chains::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::greeks::Greeks;
use optionstratlib::model::ExpirationDate;
use optionstratlib::model::Options;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pricing::{black_76, black_scholes, garman_kohlhagen};
use positive::Positive;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

/// The extremes that reach the arithmetic panics: the smallest representable
/// `Decimal`, the largest, and the ordinary values in between.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(Positive::ONE),
        Just(Positive::MAX),
        Just(pos("0.000000000000000000000000001")),
        Just(pos("0.0000000000001")),
        Just(pos("0.2")),
        Just(pos("30")),
        Just(pos("100")),
        Just(pos("1000000")),
        Just(pos("1000000000")),
    ]
}

/// Rates well outside anything quotable, in both directions.
fn extreme_rate() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(dec!(0.05)),
        Just(dec!(-0.05)),
        Just(dec!(1000000)),
        Just(dec!(-1000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

fn pos(value: &str) -> Positive {
    let decimal = Decimal::from_str(value).expect("literal is a valid Decimal");
    Positive::new_decimal(decimal).expect("literal is non-negative")
}

fn option_style() -> impl Strategy<Value = OptionStyle> {
    prop_oneof![Just(OptionStyle::Call), Just(OptionStyle::Put)]
}

fn side() -> impl Strategy<Value = Side> {
    prop_oneof![Just(Side::Long), Just(Side::Short)]
}

#[allow(clippy::too_many_arguments)]
fn build_option(
    spot: Positive,
    strike: Positive,
    volatility: Positive,
    days: Positive,
    quantity: Positive,
    dividend_yield: Positive,
    rate: Decimal,
    style: OptionStyle,
    side: Side,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "TEST".to_string(),
        strike,
        ExpirationDate::Days(days),
        volatility,
        quantity,
        spot,
        rate,
        style,
        dividend_yield,
        None,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Every Greek returns for every combination of extremes.
    #[test]
    fn test_greeks_never_panic_on_extreme_inputs(
        spot in extreme_positive(),
        strike in extreme_positive(),
        volatility in extreme_positive(),
        days in extreme_positive(),
        quantity in extreme_positive(),
        dividend_yield in extreme_positive(),
        rate in extreme_rate(),
        style in option_style(),
        side in side(),
    ) {
        let option = build_option(
            spot, strike, volatility, days, quantity, dividend_yield, rate, style, side,
        );

        let _ = option.delta();
        let _ = option.gamma();
        let _ = option.theta();
        let _ = option.vega();
        let _ = option.rho();
        let _ = option.rho_d();
        let _ = option.alpha();
        let _ = option.vanna();
        let _ = option.vomma();
        let _ = option.veta();
        let _ = option.charm();
        let _ = option.color();
        let _ = option.greeks();
    }

    /// The closed-form pricing models return for every combination of extremes.
    #[test]
    fn test_pricing_never_panics_on_extreme_inputs(
        spot in extreme_positive(),
        strike in extreme_positive(),
        volatility in extreme_positive(),
        days in extreme_positive(),
        quantity in extreme_positive(),
        dividend_yield in extreme_positive(),
        rate in extreme_rate(),
        style in option_style(),
        side in side(),
    ) {
        let option = build_option(
            spot, strike, volatility, days, quantity, dividend_yield, rate, style, side,
        );

        let _ = black_scholes(&option);
        let _ = black_76(&option);
        let _ = garman_kohlhagen(&option);
        let _ = option.calculate_price_black_scholes();
        let _ = option.intrinsic_value(spot);
        let _ = option.time_value();
        let _ = option.payoff();
    }

    /// Chain construction returns for every combination of extremes, with and
    /// without the per-strike greek snapshots.
    #[test]
    fn test_build_chain_never_panics_on_extreme_inputs(
        spot in extreme_positive(),
        volatility in extreme_positive(),
        days in extreme_positive(),
        dividend_yield in extreme_positive(),
        rate in extreme_rate(),
        greek_snapshots in any::<bool>(),
    ) {
        let price_params = OptionDataPriceParams::new(
            Some(Box::new(spot)),
            Some(ExpirationDate::Days(days)),
            Some(rate),
            Some(dividend_yield),
            Some("TEST".to_string()),
        );
        let params = OptionChainBuildParams::new(
            "TEST".to_string(),
            Some(Positive::ONE),
            5,
            Some(pos("5")),
            dec!(-0.2),
            dec!(0.1),
            pos("0.02"),
            2,
            price_params,
            volatility,
        )
        .with_greek_snapshots(greek_snapshots);

        let _ = OptionChain::build_chain(&params);
    }
}
