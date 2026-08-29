//! Property-based tests for the invariants of a widened quote
//!
//! `OptionData::apply_spread` widens a quote around its mid and floors both
//! sides at one tick, where the tick is `10^-decimal_places`. Whatever the
//! mid and the spread, three things must hold: the bid is at least a tick,
//! the ask is strictly above the bid, and a mid that was supplied survives
//! the call. The first two are what a market maker's quote means; the third
//! is information the caller gave us and that we have no business discarding.
//!
//! The strict `ask > bid` is the rule chosen in #459. Both sides are floored
//! at the tick, so a quote that sits entirely below the tick would otherwise
//! land on it twice and come out locked; the ask is lifted a tick instead, so
//! the thinnest market this generator emits is one tick wide.

use optionstratlib::ExpirationDate;
use optionstratlib::chains::OptionData;
use positive::Positive;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// A quote carrying only a mid, which is the shape `build_chain` produces
/// before `apply_spread` runs.
fn quote_with_mid(mid: Positive) -> OptionData {
    let mut option_data = OptionData::new(
        Positive::HUNDRED,
        None,
        None,
        None,
        None,
        Positive::new(0.2).expect("literal is positive"),
        None,
        None,
        None,
        None,
        None,
        Some("TEST".to_string()),
        Some(ExpirationDate::Days(
            Positive::new(30.0).expect("literal is positive"),
        )),
        Some(Box::new(Positive::HUNDRED)),
        Some(dec!(0.05)),
        Some(Positive::new(0.02).expect("literal is positive")),
        None,
        None,
    );
    option_data.call_middle = Some(mid);
    option_data.put_middle = Some(mid);
    option_data
}

/// A two-sided book with no mid, which takes the other branch of the match.
fn quote_with_book(bid: Positive, ask: Positive) -> OptionData {
    let mut option_data = quote_with_mid(Positive::ONE);
    option_data.call_middle = None;
    option_data.put_middle = None;
    option_data.call_bid = Some(bid);
    option_data.call_ask = Some(ask);
    option_data.put_bid = Some(bid);
    option_data.put_ask = Some(ask);
    option_data
}

fn tick(decimal_places: u32) -> Positive {
    let tick = Decimal::try_new(1, decimal_places).expect("scale is within range");
    Positive::new_decimal(tick).expect("a power of ten is positive")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// A quote widened around a mid is ordered, floored at a tick, and keeps
    /// its mid.
    #[test]
    fn test_widened_quote_is_ordered_floored_and_keeps_its_mid(
        mid in 0.0f64..1_000_000.0,
        spread in 0.0f64..1_000_000.0,
        decimal_places in 0u32..=6,
    ) {
        let mid = Positive::new(mid).expect("range is non-negative");
        let spread = Positive::new(spread).expect("range is non-negative");
        let floor = tick(decimal_places);

        let mut option_data = quote_with_mid(mid);
        option_data.apply_spread(spread, decimal_places);

        let call_bid = option_data.call_bid.expect("call bid is quoted");
        let call_ask = option_data.call_ask.expect("call ask is quoted");
        prop_assert!(call_bid >= floor, "bid {call_bid} below the tick {floor}");
        prop_assert!(call_bid < call_ask, "locked or crossed quote {call_bid}/{call_ask}");
        prop_assert!(
            call_ask.to_dec() - call_bid.to_dec() >= floor.to_dec(),
            "quote {call_bid}/{call_ask} thinner than the tick {floor}"
        );

        // The mid is never cleared, and never sits outside its own book: a row
        // carrying a mid below its bid is incoherent on the wire.
        let call_middle = option_data.call_middle.expect("mid was cleared");
        prop_assert!(
            call_middle >= call_bid && call_middle <= call_ask,
            "mid {call_middle} outside the quote {call_bid}/{call_ask}"
        );
        // A mid already inside the widened quote is preserved exactly.
        if mid >= call_bid && mid <= call_ask {
            prop_assert_eq!(call_middle, mid, "mid inside the quote was moved");
        }

        let put_bid = option_data.put_bid.expect("put bid is quoted");
        let put_ask = option_data.put_ask.expect("put ask is quoted");
        prop_assert!(put_bid >= floor);
        prop_assert!(put_bid < put_ask, "locked or crossed quote {put_bid}/{put_ask}");
        let put_middle = option_data.put_middle.expect("mid was cleared");
        prop_assert!(put_middle >= put_bid && put_middle <= put_ask);
    }

    /// The same invariants hold for a book that had no mid, whose mid is
    /// recomputed from the widened sides.
    #[test]
    fn test_widened_book_is_ordered_and_floored(
        bid in 0.0f64..1_000_000.0,
        width in 0.0f64..1_000.0,
        spread in 0.0f64..1_000_000.0,
        decimal_places in 0u32..=6,
    ) {
        let bid_value = Positive::new(bid).expect("range is non-negative");
        let ask_value = Positive::new(bid + width).expect("range is non-negative");
        let spread = Positive::new(spread).expect("range is non-negative");
        let floor = tick(decimal_places);

        let mut option_data = quote_with_book(bid_value, ask_value);
        option_data.apply_spread(spread, decimal_places);

        let call_bid = option_data.call_bid.expect("call bid is quoted");
        let call_ask = option_data.call_ask.expect("call ask is quoted");
        prop_assert!(call_bid >= floor, "bid {call_bid} below the tick {floor}");
        prop_assert!(call_bid < call_ask, "locked or crossed quote {call_bid}/{call_ask}");
        prop_assert!(
            call_ask.to_dec() - call_bid.to_dec() >= floor.to_dec(),
            "quote {call_bid}/{call_ask} thinner than the tick {floor}"
        );

        let middle = option_data.call_middle.expect("mid is recomputed");
        prop_assert!(middle >= call_bid && middle <= call_ask, "mid outside the quote");
    }

    /// The regime the lock lives in: a spread whose half is no wider than one
    /// tick, against an anchor small enough that both sides floor onto the
    /// tick. This is where `0.02` at two decimals produced `0.01 / 0.01`.
    #[test]
    fn test_sub_tick_quote_is_one_tick_wide_not_locked(
        decimal_places in 0u32..=6,
        mid_ticks in 0.0f64..1.0,
        spread_ticks in 0.0f64..2.0,
    ) {
        let floor = tick(decimal_places);
        // A mid strictly below one tick and a spread of at most two ticks:
        // `mid + half` rounds down onto the tick and `mid - half` floors onto
        // it, so both sides collide there.
        let mid = Positive::new_decimal(floor.to_dec() * Decimal::try_from(mid_ticks)
            .expect("range is representable"))
            .expect("a non-negative product is positive");
        let spread = Positive::new_decimal(floor.to_dec() * Decimal::try_from(spread_ticks)
            .expect("range is representable"))
            .expect("a non-negative product is positive");

        let mut option_data = quote_with_mid(mid);
        option_data.apply_spread(spread, decimal_places);

        let call_bid = option_data.call_bid.expect("call bid is quoted");
        let call_ask = option_data.call_ask.expect("call ask is quoted");
        prop_assert_eq!(call_bid, floor, "bid held at the tick");
        prop_assert!(
            call_ask > call_bid,
            "locked quote {} / {} at {} decimals",
            call_bid,
            call_ask,
            decimal_places
        );
    }
}
