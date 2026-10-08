//! Benchmarks of the simulation-backed chain and series generators of
//! `optionstratlib-market` (#789), behind the `synthetic` feature.
//!
//! Every step of a generated walk rebuilds a full chain (or one chain per
//! expiration for a series), so the cost scales with steps times strikes.
//! The walks are seeded, so the work per iteration is identical.
//!
//! Run with `cargo bench -p optionstratlib-market --features synthetic --bench synthetic`.

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_market::chains::utils::OptionDataPriceParams;
use optionstratlib_market::chains::{OptionChain, OptionChainBuildParams, generator_optionchain};
use optionstratlib_market::series::{
    OptionSeries, OptionSeriesBuildParams, generator_optionseries,
};
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{WalkParams, WalkType, WalkTypeAble};
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;

/// Fixed seed: every iteration generates the same path.
const SEED: u64 = 42;

/// A walker that keeps every default stochastic kernel.
#[derive(Clone)]
struct Gbm;

impl WalkTypeAble<Positive, OptionChain> for Gbm {}
impl WalkTypeAble<Positive, OptionSeries> for Gbm {}

/// Checks that `f` succeeds on the fixture, then measures it.
///
/// A path that fails fast would otherwise be timed as its error branch and
/// report a misleadingly cheap number.
fn bench_ok<T, E: Debug>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    name: &str,
    mut f: impl FnMut() -> Result<T, E>,
) {
    if let Err(e) = f() {
        panic!("bench `{name}`: the fixture returned an error: {e:?}");
    }
    group.bench_function(name, |bench| bench.iter(|| black_box(f())));
}

fn build_params(half_width: usize) -> OptionChainBuildParams {
    OptionChainBuildParams::new(
        "BENCH".to_string(),
        None,
        half_width,
        None,
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(60.0))),
            Some(dec!(0.05)),
            Some(pos_or_panic!(0.01)),
            Some("BENCH".to_string()),
        ),
        pos_or_panic!(0.2),
    )
}

fn walk_type() -> WalkType {
    WalkType::GeometricBrownian {
        dt: pos_or_panic!(1.0 / 252.0),
        drift: dec!(0.0),
        volatility: pos_or_panic!(0.2),
    }
}

fn chain_walk(steps: usize, half_width: usize) -> WalkParams<Positive, OptionChain> {
    let chain =
        OptionChain::build_chain(&build_params(half_width)).expect("the fixture chain builds");
    WalkParams {
        size: steps,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(60.0)),
            chain,
        ),
        walk_type: walk_type(),
        walker: Box::new(Gbm),
        seed: Some(SEED),
    }
}

fn series_walk(steps: usize, expirations: usize) -> WalkParams<Positive, OptionSeries> {
    let days: Vec<Positive> = (1..=expirations)
        .map(|i| pos_or_panic!(14.0 * i as f64))
        .collect();
    let series = OptionSeries::build_series(&OptionSeriesBuildParams::new(build_params(10), days))
        .expect("the fixture series builds");
    WalkParams {
        size: steps,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(60.0)),
            series,
        ),
        walk_type: walk_type(),
        walker: Box::new(Gbm),
        seed: Some(SEED),
    }
}

fn bench_generators(c: &mut Criterion) {
    let mut group = c.benchmark_group("market/synthetic");
    group.sample_size(10);
    for (steps, half_width) in [(10, 10), (30, 10), (30, 25)] {
        let params = chain_walk(steps, half_width);
        group.throughput(Throughput::Elements(steps as u64));
        bench_ok(
            &mut group,
            &format!("generator_optionchain/{steps}_steps_half_width_{half_width}"),
            || generator_optionchain(black_box(&params)),
        );
    }
    for (steps, expirations) in [(10, 3), (30, 3)] {
        let params = series_walk(steps, expirations);
        group.throughput(Throughput::Elements(steps as u64));
        bench_ok(
            &mut group,
            &format!("generator_optionseries/{steps}_steps_{expirations}_expirations"),
            || generator_optionseries(black_box(&params)),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_generators);
criterion_main!(benches);
