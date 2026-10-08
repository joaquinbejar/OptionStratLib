/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-10-08
******************************************************************************/

//! Theta and vega of non-European options (#817).
//!
//! `greeks::theta` and `greeks::vega` used to return the European
//! Black-Scholes closed form whatever the [`OptionType`]. They now dispatch
//! to the finite differences of `greeks::numerical`, like `delta` and
//! `gamma`, so each family reports the sensitivity of its own price.
//!
//! # Reference
//!
//! Each family is checked against a reference computed here by bumping its
//! own closed-form price (`pricing::black_scholes`) directly, with steps
//! different from the library's: half a day either side for theta (the
//! library uses one day) and `0.001` either side in volatility for vega (the
//! library uses `0.01`). Both are central differences, so the two estimates
//! differ by `O(h^2)` truncation, a few parts in ten thousand of the value
//! on these 90-day, at-the-money setups; the bounds below are 1% relative,
//! with an absolute floor for values near zero (see [`THETA_FLOOR`] and
//! [`VEGA_FLOOR`]).
//!
//! Vega is quoted per volatility point for every option type, so the
//! reference divides its per-unit derivative by 100.

use optionstratlib_core::model::option::ExoticParams;
use optionstratlib_core::model::types::{
    AsianAveragingType, BarrierType, BinaryType, LookbackType, OptionStyle, OptionType,
    RainbowType, Side,
};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::greeks::{theta, vega};
use optionstratlib_pricing::pricing::black_scholes;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt::Debug;

const DAYS: Decimal = dec!(90);
const SIGMA: Decimal = dec!(0.25);

/// Half-day time step of the reference theta, in days.
const THETA_STEP_DAYS: Decimal = dec!(0.5);

/// Volatility step of the reference vega.
const VEGA_STEP: Decimal = dec!(0.001);

/// Absolute floor of the theta bound, per day.
const THETA_FLOOR: Decimal = dec!(0.0001);

/// Absolute floor of the vega bound, per vol point: 0.25% of the at-the-money
/// European vega here (`0.196`). Where vega is small after a cancellation
/// (the up-and-out call, `-0.0147`) the `O(h^2)` truncation of the library's
/// one-point step is about `3e-4`, above 1% of the value.
const VEGA_FLOOR: Decimal = dec!(0.0005);

fn ok<T, E: Debug>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("{context}: unexpected error {err:?}"),
    }
}

fn positive(value: Decimal) -> Positive {
    ok(Positive::new_decimal(value), "positive")
}

fn option(option_type: OptionType, style: OptionStyle, params: Option<ExoticParams>) -> Options {
    option_with_strike(option_type, style, params, Positive::HUNDRED)
}

fn option_with_strike(
    option_type: OptionType,
    style: OptionStyle,
    params: Option<ExoticParams>,
    strike: Positive,
) -> Options {
    Options::new(
        option_type,
        Side::Long,
        "TEST".to_string(),
        strike,
        ExpirationDate::Days(positive(DAYS)),
        positive(SIGMA),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        style,
        pos_or_panic!(0.01),
        params,
    )
}

/// One case per family whose theta and vega the dispatch moves.
fn families() -> Vec<(&'static str, Options)> {
    vec![
        (
            "barrier up-and-out call",
            option(
                OptionType::Barrier {
                    barrier_type: BarrierType::UpAndOut,
                    barrier_level: pos_or_panic!(130.0),
                    rebate: None,
                },
                OptionStyle::Call,
                None,
            ),
        ),
        (
            "barrier down-and-in put",
            option(
                OptionType::Barrier {
                    barrier_type: BarrierType::DownAndIn,
                    barrier_level: pos_or_panic!(85.0),
                    rebate: None,
                },
                OptionStyle::Put,
                None,
            ),
        ),
        (
            "asian arithmetic call",
            option(
                OptionType::Asian {
                    averaging_type: AsianAveragingType::Arithmetic,
                },
                OptionStyle::Call,
                None,
            ),
        ),
        (
            "asian geometric put",
            option(
                OptionType::Asian {
                    averaging_type: AsianAveragingType::Geometric,
                },
                OptionStyle::Put,
                None,
            ),
        ),
        (
            "lookback floating-strike call",
            option(
                OptionType::Lookback {
                    lookback_type: LookbackType::FloatingStrike,
                },
                OptionStyle::Call,
                None,
            ),
        ),
        (
            "lookback fixed-strike put",
            option(
                OptionType::Lookback {
                    lookback_type: LookbackType::FixedStrike,
                },
                OptionStyle::Put,
                None,
            ),
        ),
        (
            "binary cash-or-nothing call",
            option(
                OptionType::Binary {
                    binary_type: BinaryType::CashOrNothing,
                },
                OptionStyle::Call,
                None,
            ),
        ),
        (
            "binary asset-or-nothing put",
            option(
                OptionType::Binary {
                    binary_type: BinaryType::AssetOrNothing,
                },
                OptionStyle::Put,
                None,
            ),
        ),
        (
            "chooser",
            option(
                OptionType::Chooser {
                    choice_date: pos_or_panic!(30.0),
                },
                OptionStyle::Call,
                None,
            ),
        ),
        (
            // One strike serves both legs: at 50 the underlying call is worth
            // about 50, so the compound is near the money.
            "compound call on call",
            option_with_strike(
                OptionType::Compound {
                    underlying_option: Box::new(OptionType::European),
                },
                OptionStyle::Call,
                None,
                pos_or_panic!(50.0),
            ),
        ),
        (
            "cliquet",
            option(
                OptionType::Cliquet {
                    reset_dates: vec![pos_or_panic!(30.0), pos_or_panic!(60.0)],
                },
                OptionStyle::Call,
                Some(ExoticParams {
                    cliquet_local_cap: Some(dec!(0.05)),
                    cliquet_local_floor: Some(dec!(0.0)),
                    ..ExoticParams::default()
                }),
            ),
        ),
        (
            "rainbow best-of call",
            option(
                OptionType::Rainbow {
                    num_assets: 2,
                    rainbow_type: RainbowType::BestOf,
                },
                OptionStyle::Call,
                Some(ExoticParams {
                    rainbow_second_asset_price: Some(Positive::HUNDRED),
                    rainbow_second_asset_volatility: Some(pos_or_panic!(0.3)),
                    rainbow_second_asset_dividend: Some(Positive::ZERO),
                    rainbow_correlation: Some(dec!(0.5)),
                    ..ExoticParams::default()
                }),
            ),
        ),
        (
            // The spread `S1 - S2` is 5, so a strike of 5 is at the money.
            "spread call",
            option_with_strike(
                OptionType::Spread {
                    second_asset: pos_or_panic!(95.0),
                },
                OptionStyle::Call,
                Some(ExoticParams {
                    spread_second_asset_volatility: Some(pos_or_panic!(0.3)),
                    spread_second_asset_dividend: Some(Positive::ZERO),
                    spread_correlation: Some(dec!(0.5)),
                    ..ExoticParams::default()
                }),
                pos_or_panic!(5.0),
            ),
        ),
        (
            "quanto call",
            option(
                OptionType::Quanto {
                    exchange_rate: pos_or_panic!(1.1),
                },
                OptionStyle::Call,
                Some(ExoticParams {
                    quanto_fx_volatility: Some(pos_or_panic!(0.1)),
                    quanto_fx_correlation: Some(dec!(0.3)),
                    quanto_foreign_rate: Some(dec!(0.03)),
                    ..ExoticParams::default()
                }),
            ),
        ),
        (
            "exchange call",
            option(
                OptionType::Exchange {
                    second_asset: pos_or_panic!(95.0),
                },
                OptionStyle::Call,
                Some(ExoticParams {
                    exchange_second_asset_volatility: Some(pos_or_panic!(0.3)),
                    exchange_second_asset_dividend: Some(Positive::ZERO),
                    exchange_correlation: Some(dec!(0.5)),
                    ..ExoticParams::default()
                }),
            ),
        ),
        (
            "power call",
            option(
                OptionType::Power {
                    exponent: pos_or_panic!(1.5),
                },
                OptionStyle::Call,
                None,
            ),
        ),
    ]
}

/// The family's own price, the quantity the reference differentiates.
fn price(option: &Options, context: &str) -> Decimal {
    ok(black_scholes(option), context)
}

fn with_days(option: &Options, days: Decimal) -> Options {
    let mut bumped = option.clone();
    bumped.expiration_date = ExpirationDate::Days(positive(days));
    bumped
}

fn with_vol(option: &Options, sigma: Decimal) -> Options {
    let mut bumped = option.clone();
    bumped.implied_volatility = positive(sigma);
    bumped
}

/// Per-day theta: `(P(T - h) - P(T + h)) / 2h` with `h` half a day.
fn reference_theta(option: &Options) -> Decimal {
    let shorter = price(&with_days(option, DAYS - THETA_STEP_DAYS), "theta down");
    let longer = price(&with_days(option, DAYS + THETA_STEP_DAYS), "theta up");
    (shorter - longer) / (dec!(2) * THETA_STEP_DAYS)
}

/// Per-vol-point vega: `(P(sigma + h) - P(sigma - h)) / 2h / 100`.
fn reference_vega(option: &Options) -> Decimal {
    let up = price(&with_vol(option, SIGMA + VEGA_STEP), "vega up");
    let down = price(&with_vol(option, SIGMA - VEGA_STEP), "vega down");
    (up - down) / (dec!(2) * VEGA_STEP) / dec!(100)
}

/// `|actual - expected| <= max(1% of |expected|, floor)`.
fn assert_close(name: &str, greek: &str, actual: Decimal, expected: Decimal, floor: Decimal) {
    let bound = (expected.abs() * dec!(0.01)).max(floor);
    assert!(
        (actual - expected).abs() <= bound,
        "{name}: {greek} {actual} vs bumped-price reference {expected} (bound {bound})"
    );
}

/// The option with its type set to European, which is what `theta` and
/// `vega` priced before #817 whatever the type.
fn as_european(option: &Options) -> Options {
    let mut european = option.clone();
    european.option_type = OptionType::European;
    european.exotic_params = None;
    european
}

#[test]
fn test_exotic_theta_matches_bumped_price_reference() {
    for (name, option) in families() {
        // Per-day theta; the two steps differ by under 5e-6 on every case.
        assert_close(
            name,
            "theta",
            ok(theta(&option), name),
            reference_theta(&option),
            THETA_FLOOR,
        );
    }
}

#[test]
fn test_exotic_vega_matches_bumped_price_reference() {
    for (name, option) in families() {
        assert_close(
            name,
            "vega",
            ok(vega(&option), name),
            reference_vega(&option),
            VEGA_FLOOR,
        );
    }
}

/// Before #817 every family reported the European theta and vega; each one
/// now reports a different value, its own.
#[test]
fn test_exotic_theta_vega_differ_from_the_european_closed_form() {
    for (name, option) in families() {
        let european = as_european(&option);
        assert_ne!(
            ok(theta(&option), name),
            ok(theta(&european), name),
            "{name}: theta"
        );
        assert_ne!(
            ok(vega(&option), name),
            ok(vega(&european), name),
            "{name}: vega"
        );
    }
}

/// The European path is untouched: the values pinned here are the closed
/// forms before #817, digit for digit.
#[test]
fn test_european_theta_vega_unchanged() {
    let call = option(OptionType::European, OptionStyle::Call, None);
    let put = option(OptionType::European, OptionStyle::Put, None);
    assert_eq!(
        ok(theta(&call), "call theta"),
        dec!(-0.0325116056731876041127726879)
    );
    assert_eq!(
        ok(theta(&put), "put theta"),
        dec!(-0.021713804797145829968547607)
    );
    assert_eq!(
        ok(vega(&call), "call vega"),
        dec!(0.1956433012493177677876632744)
    );
    assert_eq!(ok(vega(&put), "put vega"), ok(vega(&call), "call vega"));
}

/// The numerical values are those of one long unit contract; the dispatch
/// signs and scales them like the closed forms.
#[test]
fn test_exotic_theta_vega_follow_the_position_sign_convention() {
    for (name, long) in families() {
        let mut short = long.clone();
        short.side = Side::Short;
        short.quantity = pos_or_panic!(3.0);
        assert_eq!(
            ok(theta(&short), name),
            -dec!(3) * ok(theta(&long), name),
            "{name}: theta"
        );
        assert_eq!(
            ok(vega(&short), name),
            -dec!(3) * ok(vega(&long), name),
            "{name}: vega"
        );
    }
}

#[test]
fn test_exotic_theta_vega_at_expiry_are_zero() {
    for (name, option) in families() {
        let expired = with_days(&option, Decimal::ZERO);
        assert_eq!(ok(theta(&expired), name), Decimal::ZERO, "{name}: theta");
        assert_eq!(ok(vega(&expired), name), Decimal::ZERO, "{name}: vega");
    }
}

/// An American option has no closed form: theta and vega now fail like
/// delta and gamma instead of returning the European value.
#[test]
fn test_american_theta_vega_fail_like_delta() {
    let american = option(OptionType::American, OptionStyle::Put, None);
    assert!(optionstratlib_pricing::greeks::delta(&american).is_err());
    assert!(theta(&american).is_err());
    assert!(vega(&american).is_err());
}
