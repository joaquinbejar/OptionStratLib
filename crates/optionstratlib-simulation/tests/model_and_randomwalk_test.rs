//! `WalkType`, `RandomWalk` and `Simulator` through the public API (#633).
//!
//! These tests lived in `tests/unit/simulation/model_and_randomwalk_tests.rs`,
//! which no `mod.rs` declared, so they never compiled. They are revived here
//! against the current API; the `Graph` part moved to the facade's
//! visualization tests, where `Graph` lives. One assertion was wrong from the
//! start: it expected `Simulator`'s `Display` to start with
//! `"Simulator Title: SIM"`, a string that only ever appeared in this test
//! (commit 341379aa); `Display` has always printed the bare title on the
//! first line, and that is what is asserted now.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::{Len, TimeFrame};
use optionstratlib_pricing::pricing::Profit;
use optionstratlib_simulation::simulation::randomwalk::RandomWalk;
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    WalkParams, WalkType, WalkTypeAble, generator_positive,
};

/// A walker that keeps every default: a `Historical` walk replays its prices.
#[derive(Clone)]
struct Replay;

impl WalkTypeAble<Positive, Positive> for Replay {}

fn params(prices: &[f64]) -> WalkParams<Positive, Positive> {
    WalkParams {
        size: prices.len(),
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(50.0),
        ),
        walker: Box::new(Replay),
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices: prices.iter().map(|p| pos_or_panic!(*p)).collect(),
            symbol: Some("ABC".to_string()),
        },
    }
}

#[test]
fn test_walk_type_historical_display_names_kind_symbol_and_prices() {
    let walk_type = WalkType::Historical {
        timeframe: TimeFrame::Day,
        prices: vec![Positive::ONE, Positive::TWO, pos_or_panic!(3.0)],
        symbol: Some("ABC".to_string()),
    };
    let shown = format!("{walk_type}");
    assert!(shown.contains("Historical"), "{shown}");
    assert!(shown.contains("ABC"), "{shown}");
    assert!(shown.contains("prices"), "{shown}");
}

#[test]
fn test_random_walk_profit_is_not_implemented() {
    let walk = match RandomWalk::new(
        "RW_Title".to_string(),
        &params(&[50.0, 51.0, 53.0]),
        generator_positive,
    ) {
        Ok(walk) => walk,
        Err(error) => panic!("RandomWalk::new: {error}"),
    };
    assert_eq!(walk.get_title(), "RW_Title");
    match walk.calculate_profit_at(&pos_or_panic!(101.0)) {
        Err(error) => assert!(
            error.to_string().to_lowercase().contains("not implemented"),
            "{error}"
        ),
        Ok(value) => panic!("expected the not-implemented error, got {value}"),
    }
}

#[test]
fn test_simulator_accessors_display_profit_and_indexing() {
    let mut sim = match Simulator::new(
        "SIM".to_string(),
        2,
        &params(&[50.0, 52.0, 55.0, 54.0]),
        generator_positive,
    ) {
        Ok(sim) => sim,
        Err(error) => panic!("Simulator::new: {error}"),
    };
    assert_eq!(sim.len(), 2);
    assert_eq!(sim.get_random_walks().len(), 2);
    assert_eq!(sim.get_last_steps().len(), 2);
    assert_eq!(sim.get_last_values().len(), 2);
    // Each walk replays the prices, so each ends on the last one.
    assert_eq!(
        sim.get_last_positive_values(),
        vec![pos_or_panic!(54.0), pos_or_panic!(54.0)]
    );

    let shown = format!("{sim}");
    assert_eq!(shown.lines().next(), Some("SIM"), "{shown}");
    assert!(shown.contains("\tRandomWalk Title: SIM_0"), "{shown}");
    assert!(shown.contains("\tRandomWalk Title: SIM_1"), "{shown}");

    match sim.calculate_profit_at(&pos_or_panic!(55.0)) {
        Err(error) => assert!(
            error.to_string().to_lowercase().contains("not implemented"),
            "{error}"
        ),
        Ok(value) => panic!("expected the not-implemented error, got {value}"),
    }

    assert_eq!(sim[0].get_title(), "SIM_0");
    sim[0].set_title("CHANGED".to_string());
    assert_eq!(sim[0].get_title(), "CHANGED");
}
