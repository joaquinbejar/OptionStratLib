//! `black_scholes()` at expiry (#843).
//!
//! The dispatcher computed `d1` / `d2` before it looked at the option type,
//! and both divide by `σ√T`, so every exotic at `T = 0` failed with
//! `Greeks(InputError(InvalidTime))` although its kernel defines the value
//! at expiry. Each family below is priced through `black_scholes()` at
//! `T = 0` and checked against the contract's payoff at the spot, per unit
//! and signed by the side, against its own kernel, and, where the price is
//! continuous there, against the price one minute before expiry. A
//! European option at `T = 0` is its intrinsic value. For `T > 0` the
//! dispatcher must return exactly what the kernel returns.
//!
//! Inputs: `S = 100 + 5 = 105`, `K = 100`, `σ = 25 %`, `r = 5 %`, `q = 0`;
//! a second asset at 98 (spread, exchange) or 110 (rainbow), an exchange
//! rate of 1.5 (quanto), an exponent of 2 (power).

use optionstratlib_core::model::option::ExoticParams;
use optionstratlib_core::model::types::{
    AsianAveragingType, BarrierType, BinaryType, LookbackType, OptionStyle, OptionType,
    RainbowType, Side,
};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::error::PricingError;
use optionstratlib_pricing::pricing::{
    asian_black_scholes, barrier_black_scholes, binary_black_scholes, black_scholes,
    chooser_black_scholes, cliquet_black_scholes, compound_black_scholes, exchange_black_scholes,
    lookback_black_scholes, power_black_scholes, quanto_black_scholes, rainbow_black_scholes,
    spread_black_scholes,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// One minute, in days.
const ONE_MINUTE: f64 = 1.0 / 1440.0;

/// Half a unit in the third decimal.
const TOL_3DP: Decimal = dec!(0.0005);

/// One part in a million, for payoffs far above one.
const RELATIVE_TOL: Decimal = dec!(0.000001);

fn exotic_params() -> ExoticParams {
    ExoticParams {
        rainbow_second_asset_price: Some(pos_or_panic!(110.0)),
        rainbow_second_asset_volatility: Some(pos_or_panic!(0.3)),
        rainbow_second_asset_dividend: Some(Positive::ZERO),
        rainbow_correlation: Some(dec!(0.5)),
        spread_second_asset_volatility: Some(pos_or_panic!(0.3)),
        spread_second_asset_dividend: Some(Positive::ZERO),
        spread_correlation: Some(dec!(0.5)),
        quanto_fx_volatility: Some(pos_or_panic!(0.1)),
        quanto_fx_correlation: Some(dec!(0.3)),
        quanto_foreign_rate: Some(dec!(0.02)),
        exchange_second_asset_volatility: Some(pos_or_panic!(0.3)),
        exchange_second_asset_dividend: Some(Positive::ZERO),
        exchange_correlation: Some(dec!(0.5)),
        ..ExoticParams::default()
    }
}

fn option(option_type: OptionType, style: OptionStyle, side: Side, days: f64) -> Options {
    Options::new(
        option_type,
        side,
        "T0".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(days)),
        pos_or_panic!(0.25),
        Positive::ONE,
        pos_or_panic!(105.0),
        dec!(0.05),
        style,
        Positive::ZERO,
        Some(exotic_params()),
    )
}

/// The family's own kernel.
fn kernel(option: &Options) -> Result<Decimal, PricingError> {
    match option.option_type {
        OptionType::Asian { .. } => asian_black_scholes(option),
        OptionType::Barrier { .. } => barrier_black_scholes(option),
        OptionType::Binary { .. } => binary_black_scholes(option),
        OptionType::Lookback { .. } => lookback_black_scholes(option),
        OptionType::Compound { .. } => compound_black_scholes(option),
        OptionType::Chooser { .. } => chooser_black_scholes(option),
        OptionType::Cliquet { .. } => cliquet_black_scholes(option),
        OptionType::Rainbow { .. } => rainbow_black_scholes(option),
        OptionType::Spread { .. } => spread_black_scholes(option),
        OptionType::Quanto { .. } => quanto_black_scholes(option),
        OptionType::Exchange { .. } => exchange_black_scholes(option),
        OptionType::Power { .. } => power_black_scholes(option),
        _ => black_scholes(option),
    }
}

/// Whether the price one minute out is within three places of the value at
/// expiry. It is not for the lookbacks and the cliquet, whose value at a
/// fresh start carries a time value of order `σ√T` (0.01 to 0.03 one minute
/// out; the path extreme or the period return is still open), nor for the
/// compound, whose Geske price does not tend to the kernel's value at
/// expiry (reported separately).
fn continuous(option_type: &OptionType) -> bool {
    !matches!(
        option_type,
        OptionType::Lookback { .. } | OptionType::Cliquet { .. } | OptionType::Compound { .. }
    )
}

/// The long payoff at the spot of every family, `(type, call, put)`.
fn families() -> Vec<(OptionType, Decimal, Decimal)> {
    vec![
        // A new averaging window: the average is the spot.
        (
            OptionType::Asian {
                averaging_type: AsianAveragingType::Arithmetic,
            },
            dec!(5),
            dec!(0),
        ),
        (
            OptionType::Asian {
                averaging_type: AsianAveragingType::Geometric,
            },
            dec!(5),
            dec!(0),
        ),
        // Alive above 90: the vanilla.
        (
            OptionType::Barrier {
                barrier_type: BarrierType::DownAndOut,
                barrier_level: pos_or_panic!(90.0),
                rebate: None,
            },
            dec!(5),
            dec!(0),
        ),
        // Never knocked in above 90: the rebate (#826).
        (
            OptionType::Barrier {
                barrier_type: BarrierType::DownAndIn,
                barrier_level: pos_or_panic!(90.0),
                rebate: Some(pos_or_panic!(3.0)),
            },
            dec!(3),
            dec!(3),
        ),
        (
            OptionType::Binary {
                binary_type: BinaryType::CashOrNothing,
            },
            dec!(1),
            dec!(0),
        ),
        (
            OptionType::Binary {
                binary_type: BinaryType::AssetOrNothing,
            },
            dec!(105),
            dec!(0),
        ),
        (
            OptionType::Binary {
                binary_type: BinaryType::Gap,
            },
            dec!(5),
            dec!(0),
        ),
        // A new contract: the extremes are the spot.
        (
            OptionType::Lookback {
                lookback_type: LookbackType::FixedStrike,
            },
            dec!(5),
            dec!(0),
        ),
        (
            OptionType::Lookback {
                lookback_type: LookbackType::FloatingStrike,
            },
            dec!(0),
            dec!(0),
        ),
        // The kernel's convention: the underlying has the compound's style
        // and strike, so a call on a call is `max(5 - 100, 0)` and a put on
        // a put `max(100 - 0, 0)`.
        (
            OptionType::Compound {
                underlying_option: Box::new(OptionType::European),
            },
            dec!(0),
            dec!(100),
        ),
        // The better of the two intrinsics, whatever the style.
        (
            OptionType::Chooser {
                choice_date: Positive::ZERO,
            },
            dec!(5),
            dec!(5),
        ),
        // No reset period has elapsed: no return accrued.
        (
            OptionType::Cliquet {
                reset_dates: vec![pos_or_panic!(30.0)],
            },
            dec!(0),
            dec!(0),
        ),
        // Best of 105 and 110 is 110; worst is 105.
        (
            OptionType::Rainbow {
                num_assets: 2,
                rainbow_type: RainbowType::BestOf,
            },
            dec!(10),
            dec!(0),
        ),
        (
            OptionType::Rainbow {
                num_assets: 2,
                rainbow_type: RainbowType::WorstOf,
            },
            dec!(5),
            dec!(0),
        ),
        // `S1 - S2 = 7` against `K = 100`.
        (
            OptionType::Spread {
                second_asset: pos_or_panic!(98.0),
            },
            dec!(0),
            dec!(93),
        ),
        // `max(S1 - S2, 0)`; the exchange kernel has no style.
        (
            OptionType::Exchange {
                second_asset: pos_or_panic!(98.0),
            },
            dec!(7),
            dec!(7),
        ),
        // The vanilla payoff converted at 1.5.
        (
            OptionType::Quanto {
                exchange_rate: pos_or_panic!(1.5),
            },
            dec!(7.5),
            dec!(0),
        ),
        // `105² - 100`.
        (
            OptionType::Power {
                exponent: pos_or_panic!(2.0),
            },
            dec!(10925),
            dec!(0),
        ),
    ]
}

/// At expiry every exotic family prices through `black_scholes()` at its
/// payoff; it failed with `InvalidTime` before #843.
#[test]
fn test_black_scholes_at_expiry_is_the_payoff_for_every_exotic() {
    for (option_type, call, put) in families() {
        for (style, long_payoff) in [(OptionStyle::Call, call), (OptionStyle::Put, put)] {
            for (side, payoff) in [(Side::Long, long_payoff), (Side::Short, -long_payoff)] {
                let context = format!("{option_type:?} {style:?} {side:?}");
                let at_expiry = option(option_type.clone(), style, side, 0.0);
                let price = match black_scholes(&at_expiry) {
                    Ok(price) => price,
                    Err(error) => panic!("{context}: black_scholes at T = 0 failed: {error}"),
                };
                assert_eq!(price, payoff, "{context}: payoff at expiry");
                assert_eq!(
                    kernel(&at_expiry).ok(),
                    Some(price),
                    "{context}: kernel at expiry"
                );
                if continuous(&option_type) {
                    let near = option(option_type.clone(), style, side, ONE_MINUTE);
                    let near_price = match black_scholes(&near) {
                        Ok(price) => price,
                        Err(error) => panic!("{context}: one minute out failed: {error}"),
                    };
                    // The carry moves the forward by `O(T)`: about 0.002 on
                    // the 10925 of the power call, a relative `2e-7`.
                    let gap = (near_price - price).abs();
                    let tolerance = TOL_3DP.max(price.abs() * RELATIVE_TOL);
                    assert!(
                        gap <= tolerance,
                        "{context}: one minute out {near_price}, at expiry {price}"
                    );
                }
            }
        }
    }
}

/// Before expiry the dispatcher returns exactly the kernel's price, as it
/// did before #843.
#[test]
fn test_black_scholes_before_expiry_is_the_kernel_price() {
    for (option_type, _, _) in families() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for side in [Side::Long, Side::Short] {
                for days in [ONE_MINUTE, 30.0, 182.5] {
                    let contract = option(option_type.clone(), style, side, days);
                    let context = format!("{option_type:?} {style:?} {side:?} {days}d");
                    assert_eq!(
                        black_scholes(&contract).ok(),
                        kernel(&contract).ok(),
                        "{context}"
                    );
                }
            }
        }
    }
}

/// A European option at expiry is worth its intrinsic value, per unit and
/// signed by the side; it returned `InvalidTime` before #843. One minute out
/// the price agrees with it to three places away from the money.
#[test]
fn test_black_scholes_european_at_expiry_is_the_intrinsic_value() {
    for (style, long_intrinsic) in [(OptionStyle::Call, dec!(5)), (OptionStyle::Put, dec!(0))] {
        for (side, intrinsic) in [(Side::Long, long_intrinsic), (Side::Short, -long_intrinsic)] {
            let context = format!("European {style:?} {side:?}");
            let at_expiry = option(OptionType::European, style, side, 0.0);
            assert_eq!(
                black_scholes(&at_expiry).ok(),
                Some(intrinsic),
                "{context} at expiry"
            );
            let near = option(OptionType::European, style, side, ONE_MINUTE);
            let near_price = match black_scholes(&near) {
                Ok(price) => price,
                Err(error) => panic!("{context}: one minute out failed: {error}"),
            };
            assert!(
                (near_price - intrinsic).abs() <= TOL_3DP,
                "{context}: one minute out {near_price}, at expiry {intrinsic}"
            );
        }
    }
    // American and Bermuda have no closed form here; at expiry they keep
    // the error they returned before #843.
    for option_type in [
        OptionType::American,
        OptionType::Bermuda {
            exercise_dates: vec![],
        },
    ] {
        let at_expiry = option(option_type, OptionStyle::Call, Side::Long, 0.0);
        assert!(black_scholes(&at_expiry).is_err());
    }
}
