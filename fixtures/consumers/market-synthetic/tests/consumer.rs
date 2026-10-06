//! Simulation-backed option chains from `optionstratlib-market` with only
//! its `synthetic` feature (#537): the generator replays a price history over
//! a seed chain, and a simulation failure reaches the caller as
//! `ChainError::Generator`, whose source downcasts to `SimulationError`.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_core::{pos_or_panic, spos};
use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib_market::chains::{OptionChain, generator_optionchain};
use optionstratlib_market::error::ChainError;
use optionstratlib_simulation::error::SimulationError;
use optionstratlib_simulation::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib_simulation::simulation::{WalkParams, WalkType, WalkTypeAble};
use rust_decimal_macros::dec;

/// A walker that keeps every default: a `Historical` walk replays its prices.
#[derive(Clone)]
struct Replay;

impl WalkTypeAble<Positive, OptionChain> for Replay {}

fn seed_chain() -> OptionChain {
    let params = OptionChainBuildParams::new(
        "XYZ".to_string(),
        None,
        10,
        spos!(5.0),
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(60.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("XYZ".to_string()),
        ),
        pos_or_panic!(0.25),
    );
    match OptionChain::build_chain(&params) {
        Ok(chain) => chain,
        Err(error) => panic!("seed chain: {error}"),
    }
}

fn walk(size: usize, prices: Vec<Positive>) -> WalkParams<Positive, OptionChain> {
    WalkParams {
        size,
        init_step: Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Day,
                ExpirationDate::Days(pos_or_panic!(60.0)),
            ),
            y: Ystep::new(0, seed_chain()),
        },
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices,
            symbol: None,
        },
        walker: Box::new(Replay),
    }
}

#[test]
fn test_generated_chains_follow_the_replayed_prices() {
    let prices = vec![
        Positive::HUNDRED,
        pos_or_panic!(104.0),
        pos_or_panic!(98.0),
        pos_or_panic!(103.0),
    ];
    let steps = match generator_optionchain(&walk(4, prices.clone())) {
        Ok(steps) => steps,
        Err(error) => panic!("generator_optionchain: {error}"),
    };
    assert_eq!(steps.len(), 4);
    for (step, price) in steps.iter().zip(&prices).skip(1) {
        assert_eq!(step.y.value().underlying_price, *price);
    }
}

#[test]
fn test_a_simulation_failure_arrives_as_a_generator_error() {
    // A historical walk with fewer prices than steps cannot be replayed.
    match generator_optionchain(&walk(5, vec![Positive::HUNDRED, pos_or_panic!(101.0)])) {
        Err(ChainError::Generator(source)) => {
            assert!(
                source.downcast_ref::<SimulationError>().is_some(),
                "{source}"
            );
        }
        other => panic!("expected ChainError::Generator, got {other:?}"),
    }
}
