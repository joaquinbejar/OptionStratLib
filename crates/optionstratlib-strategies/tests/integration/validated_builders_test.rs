/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 6/10/26
******************************************************************************/

//! The strategy builders that used to call `validate()` and drop the answer
//! return `StrategyError::InvalidStrategy` now (#696).
//!
//! Each builder gets a valid case, which still builds, and an invalid case,
//! which used to come back as `Ok` with legs that fail the strategy's own
//! validation. The butterfly builders that still only warn (#706) are not
//! covered here. The put verticals are covered next to their code, in
//! `bull_put_spread.rs` and `bear_put_spread.rs`.

use optionstratlib_core::model::types::{OptionStyle, Side};
use optionstratlib_core::model::utils::create_sample_position;
use optionstratlib_core::model::{ExpirationDate, Position, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_strategies::error::strategies::StrategyError;
use optionstratlib_strategies::strategies::base::{StrategyType, Validable};
use optionstratlib_strategies::strategies::custom::CustomStrategy;
use optionstratlib_strategies::strategies::{
    BearCallSpread, BullCallSpread, Collar, CoveredCall, IronButterfly, IronCondor,
    LongButterflySpread, LongCall, LongStraddle, LongStrangle, PoorMansCoveredCall, ProtectivePut,
    ShortButterflySpread, ShortPut, ShortStraddle, ShortStrangle, StrategyConstructor,
};
use rust_decimal_macros::dec;

/// One contract on an underlying at 100, 30 days out, premium 5.00.
fn leg(side: Side, style: OptionStyle, strike: Positive) -> Position {
    create_sample_position(
        style,
        side,
        Positive::HUNDRED,
        Positive::ONE,
        strike,
        pos_or_panic!(0.2),
    )
}

/// A short leg with no premium fails `Position::validate`.
fn without_premium(mut position: Position) -> Position {
    position.premium = Positive::ZERO;
    position
}

/// An option with no underlying symbol fails `Options::validate`.
fn without_symbol(mut position: Position) -> Position {
    position.option.underlying_symbol = String::new();
    position
}

fn days(n: f64) -> ExpirationDate {
    ExpirationDate::Days(pos_or_panic!(n))
}

fn assert_valid<T: Validable>(result: Result<T, StrategyError>) {
    match result {
        Ok(strategy) => assert!(strategy.validate()),
        Err(error) => panic!("expected a valid strategy, got {error}"),
    }
}

fn assert_rejected<T>(result: Result<T, StrategyError>, expected: StrategyType) {
    match result {
        Err(StrategyError::InvalidStrategy { strategy, .. }) => assert_eq!(strategy, expected),
        Err(other) => panic!("expected InvalidStrategy for {expected}, got {other}"),
        Ok(_) => panic!("expected InvalidStrategy for {expected}, got Ok"),
    }
}

// ---------------------------------------------------------------------------
// Constructors (`new`)
// ---------------------------------------------------------------------------

fn bull_call_spread(
    long_strike: Positive,
    short_strike: Positive,
) -> Result<BullCallSpread, StrategyError> {
    BullCallSpread::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        long_strike,
        short_strike,
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(7.5),
        pos_or_panic!(2.5),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_bull_call_spread_new_textbook_strikes_valid() {
    assert_valid(bull_call_spread(pos_or_panic!(95.0), pos_or_panic!(105.0)));
}

#[test]
fn test_bull_call_spread_new_inverted_strikes_rejected() {
    assert_rejected(
        bull_call_spread(pos_or_panic!(105.0), pos_or_panic!(95.0)),
        StrategyType::BullCallSpread,
    );
}

fn bear_call_spread(
    short_strike: Positive,
    long_strike: Positive,
) -> Result<BearCallSpread, StrategyError> {
    BearCallSpread::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        short_strike,
        long_strike,
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(7.5),
        pos_or_panic!(2.5),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_bear_call_spread_new_textbook_strikes_valid() {
    assert_valid(bear_call_spread(pos_or_panic!(95.0), pos_or_panic!(105.0)));
}

#[test]
fn test_bear_call_spread_new_inverted_strikes_rejected() {
    assert_rejected(
        bear_call_spread(pos_or_panic!(105.0), pos_or_panic!(95.0)),
        StrategyType::BearCallSpread,
    );
}

fn long_butterfly(
    low: Positive,
    middle: Positive,
    high: Positive,
) -> Result<LongButterflySpread, StrategyError> {
    LongButterflySpread::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        low,
        middle,
        high,
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(11.5),
        pos_or_panic!(4.5),
        pos_or_panic!(1.2),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_long_butterfly_spread_new_ordered_strikes_valid() {
    assert_valid(long_butterfly(
        pos_or_panic!(90.0),
        Positive::HUNDRED,
        pos_or_panic!(110.0),
    ));
}

#[test]
fn test_long_butterfly_spread_new_unordered_strikes_rejected() {
    assert_rejected(
        long_butterfly(Positive::HUNDRED, pos_or_panic!(90.0), pos_or_panic!(110.0)),
        StrategyType::LongButterflySpread,
    );
}

fn short_butterfly(
    low: Positive,
    middle: Positive,
    high: Positive,
) -> Result<ShortButterflySpread, StrategyError> {
    ShortButterflySpread::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        low,
        middle,
        high,
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(11.5),
        pos_or_panic!(4.5),
        pos_or_panic!(1.2),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_short_butterfly_spread_new_ordered_strikes_valid() {
    assert_valid(short_butterfly(
        pos_or_panic!(90.0),
        Positive::HUNDRED,
        pos_or_panic!(110.0),
    ));
}

#[test]
fn test_short_butterfly_spread_new_unordered_strikes_rejected() {
    assert_rejected(
        short_butterfly(Positive::HUNDRED, pos_or_panic!(90.0), pos_or_panic!(110.0)),
        StrategyType::ShortButterflySpread,
    );
}

fn collar(put_strike: Positive, call_strike: Positive) -> Result<Collar, StrategyError> {
    Collar::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        put_strike,
        call_strike,
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(1.5),
        pos_or_panic!(1.8),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_collar_new_put_below_call_valid() {
    assert_valid(collar(pos_or_panic!(95.0), pos_or_panic!(105.0)));
}

#[test]
fn test_collar_new_put_above_call_rejected() {
    assert_rejected(
        collar(pos_or_panic!(105.0), pos_or_panic!(95.0)),
        StrategyType::Collar,
    );
}

fn protective_put(symbol: &str) -> Result<ProtectivePut, StrategyError> {
    ProtectivePut::new(
        symbol.to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(1.5),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_protective_put_new_valid() {
    assert_valid(protective_put("TEST"));
}

#[test]
fn test_protective_put_new_invalid_put_rejected() {
    // An option with no underlying symbol fails `Options::validate`.
    assert_rejected(protective_put(""), StrategyType::ProtectivePut);
}

/// `CoveredCall::validate` checks a non-zero, long spot leg and a short call,
/// which `new` always builds after rejecting a zero quantity itself, so no
/// input reaches its `InvalidStrategy` branch; only the valid case is shown.
#[test]
fn test_covered_call_new_valid() {
    assert_valid(CoveredCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(105.0),
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(2.4),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    ));
}

fn iron_condor(
    short_call: Positive,
    short_put: Positive,
    long_call: Positive,
    long_put: Positive,
) -> Result<IronCondor, StrategyError> {
    IronCondor::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        short_call,
        short_put,
        long_call,
        long_put,
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(2.4),
        pos_or_panic!(2.1),
        pos_or_panic!(0.7),
        pos_or_panic!(0.6),
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_iron_condor_new_ordered_strikes_valid() {
    assert_valid(iron_condor(
        pos_or_panic!(105.0),
        pos_or_panic!(95.0),
        pos_or_panic!(115.0),
        pos_or_panic!(85.0),
    ));
}

#[test]
fn test_iron_condor_new_short_put_above_short_call_rejected() {
    assert_rejected(
        iron_condor(
            pos_or_panic!(95.0),
            pos_or_panic!(105.0),
            pos_or_panic!(115.0),
            pos_or_panic!(85.0),
        ),
        StrategyType::IronCondor,
    );
}

fn iron_butterfly(long_call: Positive, long_put: Positive) -> Result<IronButterfly, StrategyError> {
    IronButterfly::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        Positive::HUNDRED,
        long_call,
        long_put,
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(4.5),
        pos_or_panic!(4.1),
        pos_or_panic!(1.2),
        pos_or_panic!(1.1),
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_iron_butterfly_new_wings_outside_body_valid() {
    assert_valid(iron_butterfly(pos_or_panic!(110.0), pos_or_panic!(90.0)));
}

#[test]
fn test_iron_butterfly_new_inverted_wings_rejected() {
    assert_rejected(
        iron_butterfly(pos_or_panic!(90.0), pos_or_panic!(110.0)),
        StrategyType::IronButterfly,
    );
}

fn long_straddle(symbol: &str) -> Result<LongStraddle, StrategyError> {
    LongStraddle::new(
        symbol.to_string(),
        Positive::HUNDRED,
        Positive::HUNDRED,
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(4.5),
        pos_or_panic!(4.1),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_long_straddle_new_valid() {
    assert_valid(long_straddle("TEST"));
}

#[test]
fn test_long_straddle_new_empty_symbol_rejected() {
    assert_rejected(long_straddle(""), StrategyType::LongStraddle);
}

fn short_straddle(premium_short_call: Positive) -> Result<ShortStraddle, StrategyError> {
    ShortStraddle::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        Positive::HUNDRED,
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        premium_short_call,
        pos_or_panic!(4.1),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_short_straddle_new_valid() {
    assert_valid(short_straddle(pos_or_panic!(4.5)));
}

#[test]
fn test_short_straddle_new_short_leg_without_premium_rejected() {
    assert_rejected(short_straddle(Positive::ZERO), StrategyType::ShortStraddle);
}

fn long_strangle(
    call_strike: Positive,
    put_strike: Positive,
) -> Result<LongStrangle, StrategyError> {
    LongStrangle::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        call_strike,
        put_strike,
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(2.4),
        pos_or_panic!(2.1),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_long_strangle_new_call_above_put_valid() {
    assert_valid(long_strangle(pos_or_panic!(105.0), pos_or_panic!(95.0)));
}

#[test]
fn test_long_strangle_new_call_below_put_rejected() {
    assert_rejected(
        long_strangle(pos_or_panic!(95.0), pos_or_panic!(105.0)),
        StrategyType::LongStrangle,
    );
}

fn short_strangle(
    call_strike: Positive,
    put_strike: Positive,
) -> Result<ShortStrangle, StrategyError> {
    ShortStrangle::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        call_strike,
        put_strike,
        days(30.0),
        pos_or_panic!(0.2),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(2.4),
        pos_or_panic!(2.1),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_short_strangle_new_call_above_put_valid() {
    assert_valid(short_strangle(pos_or_panic!(105.0), pos_or_panic!(95.0)));
}

#[test]
fn test_short_strangle_new_call_below_put_rejected() {
    assert_rejected(
        short_strangle(pos_or_panic!(95.0), pos_or_panic!(105.0)),
        StrategyType::ShortStrangle,
    );
}

fn poor_mans_covered_call(
    premium_short_call: Positive,
) -> Result<PoorMansCoveredCall, StrategyError> {
    PoorMansCoveredCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(90.0),
        pos_or_panic!(105.0),
        days(365.0),
        days(30.0),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(15.8),
        premium_short_call,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_poor_mans_covered_call_new_valid() {
    assert_valid(poor_mans_covered_call(pos_or_panic!(2.4)));
}

#[test]
fn test_poor_mans_covered_call_new_short_leg_without_premium_rejected() {
    assert_rejected(
        poor_mans_covered_call(Positive::ZERO),
        StrategyType::PoorMansCoveredCall,
    );
}

fn long_call(symbol: &str) -> Result<LongCall, StrategyError> {
    LongCall::new(
        symbol.to_string(),
        Positive::HUNDRED,
        days(30.0),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(4.5),
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_long_call_new_valid() {
    assert_valid(long_call("TEST"));
}

#[test]
fn test_long_call_new_empty_symbol_rejected() {
    assert_rejected(long_call(""), StrategyType::LongCall);
}

fn short_put(premium: Positive) -> Result<ShortPut, StrategyError> {
    ShortPut::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        days(30.0),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        premium,
        Positive::ZERO,
        Positive::ZERO,
    )
}

#[test]
fn test_short_put_new_valid() {
    assert_valid(short_put(pos_or_panic!(4.1)));
}

#[test]
fn test_short_put_new_without_premium_rejected() {
    assert_rejected(short_put(Positive::ZERO), StrategyType::ShortPut);
}

fn custom(positions: Vec<Position>) -> Result<CustomStrategy, StrategyError> {
    CustomStrategy::new(
        "Custom".to_string(),
        "TEST".to_string(),
        "test".to_string(),
        Positive::HUNDRED,
        positions,
        pos_or_panic!(0.01),
        100,
        Positive::ONE,
    )
}

#[test]
fn test_custom_new_valid() {
    assert_valid(custom(vec![
        leg(Side::Long, OptionStyle::Call, Positive::HUNDRED),
        leg(Side::Short, OptionStyle::Call, pos_or_panic!(110.0)),
    ]));
}

#[test]
fn test_custom_new_short_leg_without_premium_rejected() {
    assert_rejected(
        custom(vec![
            leg(Side::Long, OptionStyle::Call, Positive::HUNDRED),
            without_premium(leg(Side::Short, OptionStyle::Call, pos_or_panic!(110.0))),
        ]),
        StrategyType::Custom,
    );
}

// ---------------------------------------------------------------------------
// Position-based builders (`StrategyConstructor::get_strategy`)
// ---------------------------------------------------------------------------

#[test]
fn test_bull_call_spread_get_strategy_valid() {
    assert_valid(BullCallSpread::get_strategy(&[
        leg(Side::Long, OptionStyle::Call, pos_or_panic!(95.0)),
        leg(Side::Short, OptionStyle::Call, pos_or_panic!(105.0)),
    ]));
}

#[test]
fn test_bull_call_spread_get_strategy_short_leg_without_premium_rejected() {
    assert_rejected(
        BullCallSpread::get_strategy(&[
            leg(Side::Long, OptionStyle::Call, pos_or_panic!(95.0)),
            without_premium(leg(Side::Short, OptionStyle::Call, pos_or_panic!(105.0))),
        ]),
        StrategyType::BullCallSpread,
    );
}

#[test]
fn test_bear_call_spread_get_strategy_valid() {
    assert_valid(BearCallSpread::get_strategy(&[
        leg(Side::Short, OptionStyle::Call, pos_or_panic!(95.0)),
        leg(Side::Long, OptionStyle::Call, pos_or_panic!(105.0)),
    ]));
}

#[test]
fn test_bear_call_spread_get_strategy_short_leg_without_premium_rejected() {
    assert_rejected(
        BearCallSpread::get_strategy(&[
            without_premium(leg(Side::Short, OptionStyle::Call, pos_or_panic!(95.0))),
            leg(Side::Long, OptionStyle::Call, pos_or_panic!(105.0)),
        ]),
        StrategyType::BearCallSpread,
    );
}

fn iron_condor_legs() -> Vec<Position> {
    vec![
        leg(Side::Long, OptionStyle::Put, pos_or_panic!(85.0)),
        leg(Side::Short, OptionStyle::Put, pos_or_panic!(95.0)),
        leg(Side::Short, OptionStyle::Call, pos_or_panic!(105.0)),
        leg(Side::Long, OptionStyle::Call, pos_or_panic!(115.0)),
    ]
}

#[test]
fn test_iron_condor_get_strategy_valid() {
    assert_valid(IronCondor::get_strategy(&iron_condor_legs()));
}

#[test]
fn test_iron_condor_get_strategy_short_leg_without_premium_rejected() {
    let mut legs = iron_condor_legs();
    legs[2] = without_premium(legs[2].clone());
    assert_rejected(IronCondor::get_strategy(&legs), StrategyType::IronCondor);
}

fn iron_butterfly_legs() -> Vec<Position> {
    vec![
        leg(Side::Long, OptionStyle::Put, pos_or_panic!(90.0)),
        leg(Side::Short, OptionStyle::Put, Positive::HUNDRED),
        leg(Side::Short, OptionStyle::Call, Positive::HUNDRED),
        leg(Side::Long, OptionStyle::Call, pos_or_panic!(110.0)),
    ]
}

#[test]
fn test_iron_butterfly_get_strategy_valid() {
    assert_valid(IronButterfly::get_strategy(&iron_butterfly_legs()));
}

#[test]
fn test_iron_butterfly_get_strategy_short_leg_without_premium_rejected() {
    let mut legs = iron_butterfly_legs();
    legs[1] = without_premium(legs[1].clone());
    assert_rejected(
        IronButterfly::get_strategy(&legs),
        StrategyType::IronButterfly,
    );
}

#[test]
fn test_long_straddle_get_strategy_valid() {
    assert_valid(LongStraddle::get_strategy(&[
        leg(Side::Long, OptionStyle::Call, Positive::HUNDRED),
        leg(Side::Long, OptionStyle::Put, Positive::HUNDRED),
    ]));
}

#[test]
fn test_long_straddle_get_strategy_invalid_option_rejected() {
    assert_rejected(
        LongStraddle::get_strategy(&[
            without_symbol(leg(Side::Long, OptionStyle::Call, Positive::HUNDRED)),
            leg(Side::Long, OptionStyle::Put, Positive::HUNDRED),
        ]),
        StrategyType::LongStraddle,
    );
}

#[test]
fn test_short_straddle_get_strategy_valid() {
    assert_valid(ShortStraddle::get_strategy(&[
        leg(Side::Short, OptionStyle::Call, Positive::HUNDRED),
        leg(Side::Short, OptionStyle::Put, Positive::HUNDRED),
    ]));
}

#[test]
fn test_short_straddle_get_strategy_short_leg_without_premium_rejected() {
    assert_rejected(
        ShortStraddle::get_strategy(&[
            without_premium(leg(Side::Short, OptionStyle::Call, Positive::HUNDRED)),
            leg(Side::Short, OptionStyle::Put, Positive::HUNDRED),
        ]),
        StrategyType::ShortStraddle,
    );
}

#[test]
fn test_long_strangle_get_strategy_valid() {
    assert_valid(LongStrangle::get_strategy(&[
        leg(Side::Long, OptionStyle::Put, pos_or_panic!(95.0)),
        leg(Side::Long, OptionStyle::Call, pos_or_panic!(105.0)),
    ]));
}

#[test]
fn test_long_strangle_get_strategy_invalid_option_rejected() {
    assert_rejected(
        LongStrangle::get_strategy(&[
            leg(Side::Long, OptionStyle::Put, pos_or_panic!(95.0)),
            without_symbol(leg(Side::Long, OptionStyle::Call, pos_or_panic!(105.0))),
        ]),
        StrategyType::LongStrangle,
    );
}

#[test]
fn test_short_strangle_get_strategy_valid() {
    assert_valid(ShortStrangle::get_strategy(&[
        leg(Side::Short, OptionStyle::Put, pos_or_panic!(95.0)),
        leg(Side::Short, OptionStyle::Call, pos_or_panic!(105.0)),
    ]));
}

#[test]
fn test_short_strangle_get_strategy_short_leg_without_premium_rejected() {
    assert_rejected(
        ShortStrangle::get_strategy(&[
            without_premium(leg(Side::Short, OptionStyle::Put, pos_or_panic!(95.0))),
            leg(Side::Short, OptionStyle::Call, pos_or_panic!(105.0)),
        ]),
        StrategyType::ShortStrangle,
    );
}

fn poor_mans_covered_call_legs() -> Vec<Position> {
    let mut long_call = leg(Side::Long, OptionStyle::Call, pos_or_panic!(90.0));
    long_call.option.expiration_date = days(365.0);
    long_call.premium = pos_or_panic!(15.8);
    vec![
        long_call,
        leg(Side::Short, OptionStyle::Call, pos_or_panic!(105.0)),
    ]
}

#[test]
fn test_poor_mans_covered_call_get_strategy_valid() {
    assert_valid(PoorMansCoveredCall::get_strategy(
        &poor_mans_covered_call_legs(),
    ));
}

#[test]
fn test_poor_mans_covered_call_get_strategy_short_leg_without_premium_rejected() {
    let mut legs = poor_mans_covered_call_legs();
    legs[1] = without_premium(legs[1].clone());
    assert_rejected(
        PoorMansCoveredCall::get_strategy(&legs),
        StrategyType::PoorMansCoveredCall,
    );
}

/// The error names the strategy and the builder that rejected it.
#[test]
fn test_invalid_strategy_error_message_names_strategy() {
    let error = bull_call_spread(pos_or_panic!(105.0), pos_or_panic!(95.0))
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default();
    assert_eq!(
        error,
        "invalid BullCallSpread strategy: the legs built by `new` fail validation"
    );
    // The probability layer keeps the reason when the error crosses into it.
    let probability = optionstratlib_analytics::error::probability::ProbabilityError::from(
        StrategyError::invalid_strategy(StrategyType::IronCondor, "reason"),
    );
    assert!(
        probability
            .to_string()
            .contains("invalid IronCondor strategy: reason")
    );
}
