use criterion::Criterion;
use optionstratlib::ExpirationDate;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::chains::{OptionChain, generator_optionchain};
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble, generator_positive};
use optionstratlib::utils::TimeFrame;
use positive::{Positive, pos_or_panic, spos};
use rust_decimal_macros::dec;
use std::hint::black_box;

#[derive(Clone)]
struct BenchWalker {}

impl WalkTypeAble<Positive, OptionChain> for BenchWalker {}
impl WalkTypeAble<Positive, Positive> for BenchWalker {}

fn build_initial_chain() -> OptionChain {
    let price_params = OptionDataPriceParams::new(
        Some(Box::new(Positive::HUNDRED)),
        Some(ExpirationDate::Days(pos_or_panic!(30.0))),
        Some(dec!(0.05)),
        spos!(0.02),
        Some("BENCH".to_string()),
    );

    let chain_params = OptionChainBuildParams::new(
        "BENCH".to_string(),
        None,
        10,
        spos!(5.0),
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        price_params,
        pos_or_panic!(0.2),
    );

    match OptionChain::build_chain(&chain_params) {
        Ok(chain) => chain,
        Err(e) => panic!("bench fixture chain failed to build: {e}"),
    }
}

fn chain_walk_params(size: usize) -> WalkParams<Positive, OptionChain> {
    WalkParams {
        size,
        init_step: Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Day,
                ExpirationDate::Days(pos_or_panic!(60.0)),
            ),
            y: Ystep::new(0, build_initial_chain()),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: pos_or_panic!(1.0 / 252.0),
            drift: dec!(0.0),
            volatility: pos_or_panic!(0.2),
        },
        walker: Box::new(BenchWalker {}),
    }
}

fn positive_walk_params(size: usize) -> WalkParams<Positive, Positive> {
    WalkParams {
        size,
        init_step: Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Day,
                ExpirationDate::Days(pos_or_panic!(2000.0)),
            ),
            y: Ystep::new(0, Positive::HUNDRED),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: pos_or_panic!(1.0 / 252.0),
            drift: dec!(0.0),
            volatility: pos_or_panic!(0.2),
        },
        walker: Box::new(BenchWalker {}),
    }
}

pub fn benchmark_chain_generators(c: &mut Criterion) {
    let mut group = c.benchmark_group("Chain Generators");
    group.sample_size(20);

    let params_10 = chain_walk_params(10);
    group.bench_function("generator_optionchain 10 steps", |b| {
        b.iter(|| {
            let steps = generator_optionchain(black_box(&params_10));
            black_box(steps)
        })
    });

    let params_25 = chain_walk_params(25);
    group.bench_function("generator_optionchain 25 steps", |b| {
        b.iter(|| {
            let steps = generator_optionchain(black_box(&params_25));
            black_box(steps)
        })
    });

    let positive_params = positive_walk_params(1_000);
    group.bench_function("generator_positive 1000 steps", |b| {
        b.iter(|| {
            let steps = generator_positive(black_box(&positive_params));
            black_box(steps)
        })
    });

    group.finish();
}
