use criterion::Criterion;
use optionstratlib::ExpirationDate;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::chains::{OptionChain, generator_optionchain};
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble, generator_positive};
use optionstratlib::utils::{Len, TimeFrame};
use positive::{Positive, pos_or_panic, spos};
use rust_decimal_macros::dec;
use std::hint::black_box;

#[derive(Clone)]
struct BenchWalker {}

impl WalkTypeAble<Positive, OptionChain> for BenchWalker {}
impl WalkTypeAble<Positive, Positive> for BenchWalker {}

/// Build parameters for a synthetic chain of `chain_size` strikes either side
/// of the money.
///
/// The expiration is deliberately in the future: at `T == 0` ten of the twelve
/// greeks early-return zero, so an expired fixture would make the greek work
/// look almost free and the measurement meaningless.
fn chain_build_params(chain_size: usize) -> OptionChainBuildParams {
    let price_params = OptionDataPriceParams::new(
        Some(Box::new(Positive::HUNDRED)),
        Some(ExpirationDate::Days(pos_or_panic!(30.0))),
        Some(dec!(0.05)),
        spos!(0.02),
        Some("BENCH".to_string()),
    );

    OptionChainBuildParams::new(
        "BENCH".to_string(),
        None,
        chain_size,
        spos!(1.0),
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        price_params,
        pos_or_panic!(0.2),
    )
}

fn build_initial_chain() -> OptionChain {
    match OptionChain::build_chain(&chain_build_params(10)) {
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

    // `build_chain` caps the generated grid at 31 strikes, so half-widths above
    // 15 all collapse onto the same chain. Label each case by the strike count
    // it actually produces, so a future cap change cannot silently turn this
    // into three measurements of the same work.
    for half_width in [5_usize, 10, 15] {
        let build_params = chain_build_params(half_width);
        let strikes = match OptionChain::build_chain(&build_params) {
            Ok(chain) => chain.len(),
            Err(e) => panic!("bench fixture chain failed to build: {e}"),
        };
        group.bench_function(format!("build_chain {strikes} strikes"), |b| {
            b.iter(|| {
                let chain = OptionChain::build_chain(black_box(&build_params));
                black_box(chain)
            })
        });
    }

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
