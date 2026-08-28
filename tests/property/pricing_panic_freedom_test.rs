//! Property-based tests for panic freedom across the exotic pricing kernels.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it. Every failure must
//! therefore come back as a `Result`, including for inputs that are extreme
//! but structurally valid.
//!
//! These tests drive every `src/pricing/` entry point over the edges of the
//! `Decimal` domain — volatilities and strikes at the smallest representable
//! scale, times to expiry and prices at the largest, rates far outside
//! anything a market quotes — and over the exotic parameter surface:
//! barriers at zero and at `Positive::MAX`, empty averaging windows, caps and
//! floors at the `Decimal` extremes, correlations well outside `[-1, 1]`. The
//! assertion is deliberately weak: whatever comes back, it must come back.

use optionstratlib::model::ExpirationDate;
use optionstratlib::model::Options;
use optionstratlib::model::option::ExoticParams;
use optionstratlib::model::types::{
    AsianAveragingType, BarrierType, BinaryType, LookbackType, OptionStyle, OptionType,
    RainbowType, Side,
};
use optionstratlib::pricing::{
    BinomialPricingParams, asian_black_scholes, barone_adesi_whaley, barrier_black_scholes,
    binary_black_scholes, chooser_black_scholes, cliquet_black_scholes, compound_black_scholes,
    exchange_black_scholes, generate_binomial_tree, lookback_black_scholes,
    monte_carlo_option_pricing, power_black_scholes, price_binomial, probability_keep_under_strike,
    quanto_black_scholes, rainbow_black_scholes, simulate_returns, spread_black_scholes, telegraph,
};
use positive::Positive;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;
use std::str::FromStr;

/// The extremes that reach the arithmetic panics: the smallest representable
/// `Decimal`, the largest, and the ordinary values in between.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos("0.000000000000000000000000001")),
        Just(pos("0.0000000000001")),
        Just(pos("0.2")),
        Just(pos("100")),
        Just(pos("1000000")),
        Just(Positive::MAX),
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

/// Every exotic `OptionType`, including the shapes that have no financial
/// meaning: a zero barrier, an empty reset schedule, a rainbow with the wrong
/// number of assets, a zero exponent.
fn exotic_option_type() -> impl Strategy<Value = OptionType> {
    prop_oneof![
        Just(OptionType::European),
        Just(OptionType::American),
        Just(OptionType::Bermuda {
            exercise_dates: vec![]
        }),
        Just(OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric
        }),
        Just(OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic
        }),
        Just(OptionType::Barrier {
            barrier_type: BarrierType::UpAndIn,
            barrier_level: Positive::ZERO,
            rebate: None
        }),
        Just(OptionType::Barrier {
            barrier_type: BarrierType::UpAndOut,
            barrier_level: pos("100"),
            rebate: Some(Positive::MAX)
        }),
        Just(OptionType::Barrier {
            barrier_type: BarrierType::DownAndIn,
            barrier_level: Positive::MAX,
            rebate: Some(Positive::ZERO)
        }),
        Just(OptionType::Barrier {
            barrier_type: BarrierType::DownAndOut,
            barrier_level: pos("0.000000000000000000000000001"),
            rebate: None
        }),
        Just(OptionType::Binary {
            binary_type: BinaryType::CashOrNothing
        }),
        Just(OptionType::Binary {
            binary_type: BinaryType::AssetOrNothing
        }),
        Just(OptionType::Binary {
            binary_type: BinaryType::Gap
        }),
        Just(OptionType::Lookback {
            lookback_type: LookbackType::FixedStrike
        }),
        Just(OptionType::Lookback {
            lookback_type: LookbackType::FloatingStrike
        }),
        Just(OptionType::Compound {
            underlying_option: Box::new(OptionType::European)
        }),
        Just(OptionType::Chooser {
            choice_date: Positive::ZERO
        }),
        Just(OptionType::Chooser {
            choice_date: pos("15")
        }),
        Just(OptionType::Chooser {
            choice_date: Positive::MAX
        }),
        Just(OptionType::Cliquet {
            reset_dates: vec![]
        }),
        Just(OptionType::Cliquet {
            reset_dates: vec![Positive::MAX, Positive::ZERO, pos("30")]
        }),
        Just(OptionType::Rainbow {
            num_assets: 2,
            rainbow_type: RainbowType::BestOf
        }),
        Just(OptionType::Rainbow {
            num_assets: 0,
            rainbow_type: RainbowType::WorstOf
        }),
        Just(OptionType::Spread {
            second_asset: Positive::ZERO
        }),
        Just(OptionType::Spread {
            second_asset: Positive::MAX
        }),
        Just(OptionType::Exchange {
            second_asset: Positive::ZERO
        }),
        Just(OptionType::Exchange {
            second_asset: pos("100")
        }),
        Just(OptionType::Quanto {
            exchange_rate: Positive::ZERO
        }),
        Just(OptionType::Quanto {
            exchange_rate: Positive::MAX
        }),
        Just(OptionType::Power {
            exponent: Positive::ZERO
        }),
        Just(OptionType::Power { exponent: pos("2") }),
        Just(OptionType::Power {
            exponent: Positive::MAX
        }),
    ]
}

/// The `OptionType`s handed to the lattice and simulation engines: the three
/// exercise styles they induct over, plus `Chooser` and `Power`, whose payoff
/// arms are evaluated on every terminal node before an unsupported type is
/// rejected.
fn lattice_option_type() -> impl Strategy<Value = OptionType> {
    prop_oneof![
        Just(OptionType::European),
        Just(OptionType::American),
        Just(OptionType::Bermuda {
            exercise_dates: vec![]
        }),
        Just(OptionType::Bermuda {
            exercise_dates: vec![Positive::ZERO, pos("30"), Positive::MAX]
        }),
        Just(OptionType::Chooser {
            choice_date: Positive::ZERO
        }),
        Just(OptionType::Chooser {
            choice_date: pos("15")
        }),
        Just(OptionType::Power {
            exponent: Positive::ZERO
        }),
        Just(OptionType::Power { exponent: pos("2") }),
        Just(OptionType::Power {
            exponent: Positive::MAX
        }),
    ]
}

/// The three shapes of `exotic_params` that reach the kernels: absent, all
/// zero, and all at the `Decimal` / `Positive` extremes with correlations
/// outside `[-1, 1]`.
fn exotic_params() -> impl Strategy<Value = Option<ExoticParams>> {
    prop_oneof![
        Just(None),
        Just(Some(ExoticParams {
            spot_prices: Some(vec![]),
            spot_min: Some(Decimal::ZERO),
            spot_max: Some(Decimal::ZERO),
            cliquet_local_cap: Some(Decimal::ZERO),
            cliquet_local_floor: Some(Decimal::ZERO),
            cliquet_global_cap: Some(Decimal::ZERO),
            cliquet_global_floor: Some(Decimal::ZERO),
            rainbow_second_asset_price: Some(Positive::ZERO),
            rainbow_second_asset_volatility: Some(Positive::ZERO),
            rainbow_second_asset_dividend: Some(Positive::ZERO),
            rainbow_correlation: Some(Decimal::ZERO),
            spread_second_asset_volatility: Some(Positive::ZERO),
            spread_second_asset_dividend: Some(Positive::ZERO),
            spread_correlation: Some(Decimal::ZERO),
            quanto_fx_volatility: Some(Positive::ZERO),
            quanto_fx_correlation: Some(Decimal::ZERO),
            quanto_foreign_rate: Some(Decimal::ZERO),
            exchange_second_asset_volatility: Some(Positive::ZERO),
            exchange_second_asset_dividend: Some(Positive::ZERO),
            exchange_correlation: Some(Decimal::ZERO),
        })),
        Just(Some(ExoticParams {
            spot_prices: Some(vec![pos("100"), Positive::ZERO, Positive::MAX]),
            spot_min: Some(Decimal::ZERO),
            spot_max: Some(Decimal::MAX),
            cliquet_local_cap: Some(Decimal::ZERO),
            cliquet_local_floor: Some(Decimal::MIN),
            cliquet_global_cap: Some(Decimal::MAX),
            cliquet_global_floor: Some(Decimal::ZERO),
            rainbow_second_asset_price: Some(Positive::MAX),
            rainbow_second_asset_volatility: Some(Positive::ZERO),
            rainbow_second_asset_dividend: Some(Positive::MAX),
            rainbow_correlation: Some(dec!(5)),
            spread_second_asset_volatility: Some(Positive::MAX),
            spread_second_asset_dividend: Some(Positive::MAX),
            spread_correlation: Some(dec!(-5)),
            quanto_fx_volatility: Some(Positive::MAX),
            quanto_fx_correlation: Some(dec!(9)),
            quanto_foreign_rate: Some(Decimal::MAX),
            exchange_second_asset_volatility: Some(Positive::ZERO),
            exchange_second_asset_dividend: Some(Positive::MAX),
            exchange_correlation: Some(dec!(-3)),
        })),
    ]
}

#[allow(clippy::too_many_arguments)]
fn build_option(
    option_type: OptionType,
    spot: Positive,
    strike: Positive,
    volatility: Positive,
    days: Positive,
    quantity: Positive,
    dividend_yield: Positive,
    rate: Decimal,
    style: OptionStyle,
    side: Side,
    exotic: Option<ExoticParams>,
) -> Options {
    Options::new(
        option_type,
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
        exotic,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Every exotic closed form returns for every combination of extremes,
    /// whatever the `OptionType` it is handed: each one rejects the types it
    /// does not model, so all twelve are driven with the same option.
    #[test]
    fn test_exotic_pricing_never_panics_on_extreme_inputs(
        option_type in exotic_option_type(),
        exotic in exotic_params(),
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
            option_type, spot, strike, volatility, days, quantity, dividend_yield, rate,
            style, side, exotic,
        );

        let _ = asian_black_scholes(&option);
        let _ = barrier_black_scholes(&option);
        let _ = binary_black_scholes(&option);
        let _ = lookback_black_scholes(&option);
        let _ = compound_black_scholes(&option);
        let _ = chooser_black_scholes(&option);
        let _ = cliquet_black_scholes(&option);
        let _ = rainbow_black_scholes(&option);
        let _ = spread_black_scholes(&option);
        let _ = quanto_black_scholes(&option);
        let _ = exchange_black_scholes(&option);
        let _ = power_black_scholes(&option);
    }

    /// The Barone-Adesi-Whaley approximation returns for every combination of
    /// extremes, including the `r = 0` boundary where `1 - e^(-rT)` vanishes.
    #[test]
    fn test_barone_adesi_whaley_never_panics_on_extreme_inputs(
        spot in extreme_positive(),
        strike in extreme_positive(),
        volatility in extreme_positive(),
        years in extreme_positive(),
        dividend_yield in extreme_positive(),
        rate in extreme_rate(),
        style in option_style(),
    ) {
        let _ = barone_adesi_whaley(
            spot, strike, years, rate, dividend_yield, volatility, &style,
        );
    }

    /// The lattice and simulation engines return for every combination of
    /// extremes. They evaluate `OptionType::payoff` on every terminal node
    /// before rejecting a type they cannot induct over, so they are driven on
    /// the exercise styles they model plus `Chooser` and `Power`, whose payoff
    /// arms used to abort on any out-of-the-money contract.
    #[test]
    fn test_numerical_engines_never_panic_on_extreme_inputs(
        option_type in lattice_option_type(),
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
            option_type.clone(), spot, strike, volatility, days, quantity, dividend_yield,
            rate, style, side, None,
        );
        let steps = NonZeroUsize::new(8).expect("literal is non-zero");
        let simulations = NonZeroUsize::new(4).expect("literal is non-zero");
        let params = BinomialPricingParams {
            asset: spot,
            volatility,
            int_rate: rate,
            strike,
            expiry: days,
            no_steps: steps,
            option_type: &option_type,
            option_style: &style,
            side: &side,
        };

        let _ = price_binomial(params.clone());
        let _ = generate_binomial_tree(&params);
        let _ = monte_carlo_option_pricing(&option, steps, simulations);
        let _ = telegraph(&option, steps, Some(dec!(0.5)), Some(dec!(0.3)));
        let _ = telegraph(&option, steps, None, None);
        let _ = probability_keep_under_strike(option, Some(strike));
        let _ = simulate_returns(rate, volatility, 8, days.to_dec());
    }
}
