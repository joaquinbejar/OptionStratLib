//! Settlement obligations through the public `optionstratlib_core` paths.
//!
//! The unit tests in `model/settlement.rs` hold the rule matrix. Here: the two
//! sides of one contract settle against each other, whatever the inputs, and
//! nothing panics at the ends of the `Positive` range.

use chrono::{DateTime, TimeDelta, Utc};
use optionstratlib_core::error::SettlementError;
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{
    CashRounding, ExpirationDate, Options, Position, Positive, SettlementEvent,
    SettlementObligation, SettlementPrice, SettlementTerms, SettlementType, settlement_obligation,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

fn cutoff() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH + TimeDelta::days(20_000)
}

fn contract(
    side: Side,
    style: OptionStyle,
    strike: Positive,
    quantity: Positive,
    multiplier: Positive,
) -> Options {
    Options::new(
        OptionType::American,
        side,
        "TEST".to_string(),
        strike,
        ExpirationDate::Days(Positive::ONE),
        pos(dec!(0.2)),
        quantity,
        strike,
        dec!(0.05),
        style,
        Positive::ZERO,
        None,
    )
    .with_contract_size(multiplier)
}

fn terms(
    settlement_type: SettlementType,
    event: SettlementEvent,
    price: Positive,
    scale: u32,
) -> SettlementTerms {
    SettlementTerms {
        settlement_type,
        event,
        price: Some(SettlementPrice {
            price,
            source: "test".to_string(),
            observed_at: cutoff(),
        }),
        cutoff: cutoff(),
        rounding: CashRounding { scale },
    }
}

/// An ordinary positive decimal, with up to four decimal places.
fn ordinary_positive() -> impl Strategy<Value = Positive> {
    (0i64..=10_000_000, 0u32..=4).prop_map(|(mantissa, scale)| pos(Decimal::new(mantissa, scale)))
}

/// The whole `Positive` range, including the ends that break the arithmetic.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        ordinary_positive(),
        Just(Positive::ZERO),
        Just(pos(Decimal::from_parts(1, 0, 0, false, 28))),
        Just(pos(dec!(1000000000000000))),
        Just(Positive::MAX),
    ]
}

fn style_strategy() -> impl Strategy<Value = OptionStyle> {
    prop_oneof![Just(OptionStyle::Call), Just(OptionStyle::Put)]
}

fn settlement_type_strategy() -> impl Strategy<Value = SettlementType> {
    prop_oneof![Just(SettlementType::Cash), Just(SettlementType::Physical)]
}

/// `(long's event, short's event)`: the same execution seen from each side.
fn events_strategy() -> impl Strategy<Value = (SettlementEvent, SettlementEvent)> {
    prop_oneof![
        Just((SettlementEvent::Expiry, SettlementEvent::Expiry)),
        Just((SettlementEvent::Exercise, SettlementEvent::Assignment)),
    ]
}

/// Asserts that `long` and `short` are the two sides of one settlement.
fn assert_offsets(long: &SettlementObligation, short: &SettlementObligation) {
    assert_eq!(long.cash.checked_add(short.cash), Some(Decimal::ZERO));
    assert_eq!(
        long.underlying.checked_add(short.underlying),
        Some(Decimal::ZERO)
    );
    assert_eq!(long.extinguished_quantity, short.extinguished_quantity);
}

proptest! {
    #[test]
    fn test_settlement_long_and_short_offset_to_zero(
        strike in ordinary_positive(),
        price in ordinary_positive(),
        quantity in ordinary_positive(),
        multiplier in ordinary_positive(),
        scale in 0u32..=6,
        style in style_strategy(),
        settlement_type in settlement_type_strategy(),
        (long_event, short_event) in events_strategy(),
    ) {
        let long = settlement_obligation(
            &contract(Side::Long, style, strike, quantity, multiplier),
            &terms(settlement_type, long_event, price, scale),
        );
        let short = settlement_obligation(
            &contract(Side::Short, style, strike, quantity, multiplier),
            &terms(settlement_type, short_event, price, scale),
        );
        prop_assert!(long.is_ok() && short.is_ok(), "{long:?} {short:?}");
        if let (Ok(long), Ok(short)) = (&long, &short) {
            assert_offsets(long, short);
        }
    }

    #[test]
    fn test_settlement_extreme_inputs_offset_or_fail_on_both_sides(
        strike in extreme_positive(),
        price in extreme_positive(),
        quantity in extreme_positive(),
        multiplier in extreme_positive(),
        scale in prop_oneof![Just(0u32), Just(2u32), Just(28u32), Just(u32::MAX)],
        style in style_strategy(),
        settlement_type in settlement_type_strategy(),
        (long_event, short_event) in events_strategy(),
    ) {
        let long = settlement_obligation(
            &contract(Side::Long, style, strike, quantity, multiplier),
            &terms(settlement_type, long_event, price, scale),
        );
        let short = settlement_obligation(
            &contract(Side::Short, style, strike, quantity, multiplier),
            &terms(settlement_type, short_event, price, scale),
        );
        match (&long, &short) {
            (Ok(long), Ok(short)) => assert_offsets(long, short),
            (
                Err(SettlementError::ArithmeticOverflow { .. }),
                Err(SettlementError::ArithmeticOverflow { .. }),
            ) => {}
            _ => prop_assert!(false, "the sides disagree: {long:?} {short:?}"),
        }
    }
}

#[test]
fn test_settlement_option_held_by_a_position_settles() {
    let option = contract(
        Side::Short,
        OptionStyle::Put,
        pos(dec!(50)),
        pos(dec!(3)),
        Positive::HUNDRED,
    );
    let position = Position::new(
        option,
        pos(dec!(1.5)),
        cutoff(),
        Positive::ZERO,
        Positive::ZERO,
        None,
        None,
    );
    let result = settlement_obligation(
        &position.option,
        &terms(
            SettlementType::Physical,
            SettlementEvent::Assignment,
            pos(dec!(40)),
            2,
        ),
    );
    assert_eq!(
        result,
        Ok(SettlementObligation {
            extinguished_quantity: pos(dec!(3)),
            cash: dec!(-15000),
            underlying: dec!(300),
        })
    );
}

#[test]
fn test_settlement_obligation_round_trips_through_json() {
    let obligation = SettlementObligation {
        extinguished_quantity: pos(dec!(2)),
        cash: dec!(-20000),
        underlying: dec!(200),
    };
    let parsed = serde_json::to_string(&obligation)
        .and_then(|text| serde_json::from_str::<SettlementObligation>(&text));
    assert_eq!(parsed.ok(), Some(obligation));
}
