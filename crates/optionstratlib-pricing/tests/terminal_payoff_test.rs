/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-10-08
******************************************************************************/

//! The core payoff of every option family is the contract's terminal payoff:
//! `Options::payoff`, which backs P&L, equals what the family's pricing
//! kernel returns at `T = 0` (#844).
//!
//! Each family is checked for calls and puts, long and short, at a spot
//! above, at and below the strike, so a payoff that ignored the side, the
//! strike or the second asset shows up as a mismatch.

use optionstratlib_core::model::option::ExoticParams;
use optionstratlib_core::model::types::{
    AsianAveragingType, BarrierType, BinaryType, LookbackType, OptionStyle, OptionType,
    RainbowType, Side,
};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::error::PricingError;
use optionstratlib_pricing::pricing::{
    asian_black_scholes, barrier_black_scholes, binary_black_scholes, chooser_black_scholes,
    cliquet_black_scholes, compound_black_scholes, exchange_black_scholes, lookback_black_scholes,
    power_black_scholes, quanto_black_scholes, rainbow_black_scholes, spread_black_scholes,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

type Kernel = fn(&Options) -> Result<Decimal, PricingError>;

/// Spots above, at and below the strike of 100.
const SPOTS: [f64; 3] = [105.0, 100.0, 95.0];

fn expired(
    option_type: OptionType,
    spot: f64,
    style: OptionStyle,
    side: Side,
    exotic_params: Option<ExoticParams>,
) -> Options {
    Options::new(
        option_type,
        side,
        "TERM".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(Positive::ZERO),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(spot),
        dec!(0.05),
        style,
        Positive::ZERO,
        exotic_params,
    )
}

/// `Options::payoff` against `kernel` at `T = 0` over every spot, style and
/// side.
fn assert_terminal_payoff(
    option_type: OptionType,
    exotic_params: Option<ExoticParams>,
    kernel: Kernel,
) {
    for spot in SPOTS {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for side in [Side::Long, Side::Short] {
                let option = expired(
                    option_type.clone(),
                    spot,
                    style,
                    side,
                    exotic_params.clone(),
                );
                let payoff = match option.payoff() {
                    Ok(value) => value,
                    Err(e) => panic!("{option_type:?} {style:?} {side:?} S={spot}: payoff {e}"),
                };
                let terminal = match kernel(&option) {
                    Ok(value) => value,
                    Err(e) => panic!("{option_type:?} {style:?} {side:?} S={spot}: kernel {e}"),
                };
                assert_eq!(
                    payoff, terminal,
                    "{option_type:?} {style:?} {side:?} S={spot}"
                );
            }
        }
    }
}

#[test]
fn test_binary_payoff_is_the_terminal_value() {
    for binary_type in [
        BinaryType::CashOrNothing,
        BinaryType::AssetOrNothing,
        BinaryType::Gap,
    ] {
        assert_terminal_payoff(
            OptionType::Binary { binary_type },
            None,
            binary_black_scholes,
        );
    }
}

#[test]
fn test_chooser_payoff_is_the_terminal_value() {
    assert_terminal_payoff(
        OptionType::Chooser {
            choice_date: pos_or_panic!(30.0),
        },
        None,
        chooser_black_scholes,
    );
}

#[test]
fn test_power_payoff_is_the_terminal_value() {
    for exponent in [Positive::ONE, Positive::TWO, pos_or_panic!(1.5)] {
        assert_terminal_payoff(OptionType::Power { exponent }, None, power_black_scholes);
    }
}

#[test]
fn test_lookback_payoff_is_the_terminal_value() {
    for lookback_type in [LookbackType::FixedStrike, LookbackType::FloatingStrike] {
        assert_terminal_payoff(
            OptionType::Lookback { lookback_type },
            None,
            lookback_black_scholes,
        );
    }
}

#[test]
fn test_asian_payoff_is_the_terminal_value() {
    for averaging_type in [
        AsianAveragingType::Arithmetic,
        AsianAveragingType::Geometric,
    ] {
        assert_terminal_payoff(
            OptionType::Asian { averaging_type },
            None,
            asian_black_scholes,
        );
    }
}

#[test]
fn test_compound_payoff_is_the_terminal_value() {
    for underlying in [
        OptionType::European,
        OptionType::Binary {
            binary_type: BinaryType::AssetOrNothing,
        },
    ] {
        assert_terminal_payoff(
            OptionType::Compound {
                underlying_option: Box::new(underlying),
            },
            None,
            compound_black_scholes,
        );
    }
}

#[test]
fn test_barrier_payoff_is_the_terminal_value() {
    for barrier_type in [
        BarrierType::UpAndIn,
        BarrierType::UpAndOut,
        BarrierType::DownAndIn,
        BarrierType::DownAndOut,
    ] {
        for rebate in [None, Some(pos_or_panic!(2.0))] {
            assert_terminal_payoff(
                OptionType::Barrier {
                    barrier_type,
                    barrier_level: pos_or_panic!(102.0),
                    rebate,
                },
                None,
                barrier_black_scholes,
            );
        }
    }
}

#[test]
fn test_cliquet_payoff_is_the_terminal_value() {
    let cliquet = OptionType::Cliquet {
        reset_dates: vec![pos_or_panic!(30.0), pos_or_panic!(60.0)],
    };
    assert_terminal_payoff(cliquet.clone(), None, cliquet_black_scholes);
    assert_terminal_payoff(
        cliquet,
        Some(ExoticParams {
            cliquet_global_floor: Some(dec!(1.5)),
            cliquet_global_cap: Some(dec!(10)),
            ..ExoticParams::default()
        }),
        cliquet_black_scholes,
    );
}

#[test]
fn test_rainbow_payoff_is_the_terminal_value() {
    for rainbow_type in [RainbowType::BestOf, RainbowType::WorstOf] {
        for second in [98.0, 103.0] {
            assert_terminal_payoff(
                OptionType::Rainbow {
                    num_assets: 2,
                    rainbow_type,
                },
                Some(ExoticParams {
                    rainbow_second_asset_price: Some(pos_or_panic!(second)),
                    rainbow_second_asset_volatility: Some(pos_or_panic!(0.25)),
                    ..ExoticParams::default()
                }),
                rainbow_black_scholes,
            );
        }
    }
}

#[test]
fn test_spread_payoff_is_the_terminal_value() {
    let params = Some(ExoticParams {
        spread_second_asset_volatility: Some(pos_or_panic!(0.25)),
        spread_correlation: Some(dec!(0.3)),
        ..ExoticParams::default()
    });
    for second in [98.0, 3.0] {
        assert_terminal_payoff(
            OptionType::Spread {
                second_asset: pos_or_panic!(second),
            },
            params.clone(),
            spread_black_scholes,
        );
    }
}

/// The example of #844: a spread put with `S1 = 105`, `S2 = 98`, `K = 100`
/// pays `100 - 7 = 93`; it paid 5 as a vanilla put on `S1`.
#[test]
fn test_spread_put_pays_on_the_spread() {
    let option = expired(
        OptionType::Spread {
            second_asset: pos_or_panic!(98.0),
        },
        105.0,
        OptionStyle::Put,
        Side::Long,
        None,
    );
    assert_eq!(option.payoff().ok(), Some(dec!(93)));
}

#[test]
fn test_exchange_payoff_is_the_terminal_value() {
    let params = Some(ExoticParams {
        exchange_second_asset_volatility: Some(pos_or_panic!(0.25)),
        exchange_correlation: Some(dec!(0.3)),
        ..ExoticParams::default()
    });
    for second in [98.0, 103.0] {
        assert_terminal_payoff(
            OptionType::Exchange {
                second_asset: pos_or_panic!(second),
            },
            params.clone(),
            exchange_black_scholes,
        );
    }
}

#[test]
fn test_quanto_payoff_is_the_terminal_value() {
    assert_terminal_payoff(
        OptionType::Quanto {
            exchange_rate: pos_or_panic!(1.25),
        },
        Some(ExoticParams {
            quanto_fx_volatility: Some(pos_or_panic!(0.1)),
            quanto_fx_correlation: Some(dec!(0.2)),
            ..ExoticParams::default()
        }),
        quanto_black_scholes,
    );
}
