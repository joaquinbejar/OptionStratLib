//! Every strategy belongs to exactly one family, and every family is
//! available with the crate's default features (#532).
//!
//! `family_of` matches `StrategyType` exhaustively, so a new strategy that is
//! not assigned a family fails to compile here, and the family table in the
//! crate docs has to be updated with it.

use optionstratlib_strategies::strategies::base::StrategyType;
use optionstratlib_strategies::strategies::custom::CustomStrategy;
use optionstratlib_strategies::strategies::{
    BearCallSpread, BearPutSpread, BullCallLadder, BullCallSpread, BullPutSpread, Collar,
    CoveredCall, IronButterfly, IronCondor, LongButterflySpread, LongCall, LongPut, LongStraddle,
    LongStrangle, PoorMansCoveredCall, ProtectivePut, ShortButterflySpread, ShortCall, ShortPut,
    ShortStraddle, ShortStrangle, Strategable,
};
use static_assertions::assert_impl_all;
use std::collections::BTreeMap;

/// The families of the crate docs' "Strategy families" table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Family {
    SingleLeg,
    VerticalSpread,
    Ladder,
    Butterfly,
    Condor,
    StraddleStrangle,
    CoveredProtective,
    Custom,
}

fn family_of(strategy: StrategyType) -> Family {
    match strategy {
        StrategyType::LongCall
        | StrategyType::LongPut
        | StrategyType::ShortCall
        | StrategyType::ShortPut => Family::SingleLeg,
        StrategyType::BullCallSpread
        | StrategyType::BullPutSpread
        | StrategyType::BearCallSpread
        | StrategyType::BearPutSpread => Family::VerticalSpread,
        StrategyType::BullCallLadder => Family::Ladder,
        StrategyType::LongButterflySpread
        | StrategyType::ShortButterflySpread
        | StrategyType::IronButterfly => Family::Butterfly,
        StrategyType::IronCondor => Family::Condor,
        StrategyType::LongStraddle
        | StrategyType::ShortStraddle
        | StrategyType::LongStrangle
        | StrategyType::ShortStrangle => Family::StraddleStrangle,
        StrategyType::CoveredCall
        | StrategyType::ProtectivePut
        | StrategyType::Collar
        | StrategyType::PoorMansCoveredCall => Family::CoveredProtective,
        StrategyType::Custom => Family::Custom,
    }
}

const ALL: [StrategyType; 22] = [
    StrategyType::LongCall,
    StrategyType::LongPut,
    StrategyType::ShortCall,
    StrategyType::ShortPut,
    StrategyType::BullCallSpread,
    StrategyType::BullPutSpread,
    StrategyType::BearCallSpread,
    StrategyType::BearPutSpread,
    StrategyType::BullCallLadder,
    StrategyType::LongButterflySpread,
    StrategyType::ShortButterflySpread,
    StrategyType::IronButterfly,
    StrategyType::IronCondor,
    StrategyType::LongStraddle,
    StrategyType::ShortStraddle,
    StrategyType::LongStrangle,
    StrategyType::ShortStrangle,
    StrategyType::CoveredCall,
    StrategyType::ProtectivePut,
    StrategyType::Collar,
    StrategyType::PoorMansCoveredCall,
    StrategyType::Custom,
];

// Every family's strategies are full strategies in the default build.
assert_impl_all!(LongCall: Strategable);
assert_impl_all!(LongPut: Strategable);
assert_impl_all!(ShortCall: Strategable);
assert_impl_all!(ShortPut: Strategable);
assert_impl_all!(BullCallSpread: Strategable);
assert_impl_all!(BullPutSpread: Strategable);
assert_impl_all!(BearCallSpread: Strategable);
assert_impl_all!(BearPutSpread: Strategable);
assert_impl_all!(BullCallLadder: Strategable);
assert_impl_all!(LongButterflySpread: Strategable);
assert_impl_all!(ShortButterflySpread: Strategable);
assert_impl_all!(IronButterfly: Strategable);
assert_impl_all!(IronCondor: Strategable);
assert_impl_all!(LongStraddle: Strategable);
assert_impl_all!(ShortStraddle: Strategable);
assert_impl_all!(LongStrangle: Strategable);
assert_impl_all!(ShortStrangle: Strategable);
assert_impl_all!(CoveredCall: Strategable);
assert_impl_all!(ProtectivePut: Strategable);
assert_impl_all!(Collar: Strategable);
assert_impl_all!(PoorMansCoveredCall: Strategable);
assert_impl_all!(CustomStrategy: Strategable);

#[test]
fn test_every_strategy_type_has_one_family_and_the_table_sizes_hold() {
    let mut sizes: BTreeMap<Family, usize> = BTreeMap::new();
    for strategy in ALL {
        *sizes.entry(family_of(strategy)).or_default() += 1;
    }
    let expected = BTreeMap::from([
        (Family::SingleLeg, 4),
        (Family::VerticalSpread, 4),
        (Family::Ladder, 1),
        (Family::Butterfly, 3),
        (Family::Condor, 1),
        (Family::StraddleStrangle, 4),
        (Family::CoveredProtective, 4),
        (Family::Custom, 1),
    ]);
    assert_eq!(sizes, expected);
}

#[test]
fn test_strategy_type_names_round_trip_for_every_family() {
    for strategy in ALL {
        let name = format!("{strategy:?}");
        assert_eq!(name.parse::<StrategyType>(), Ok(strategy), "{name}");
    }
}
