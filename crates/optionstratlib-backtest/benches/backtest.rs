//! Benchmarks of the `optionstratlib-backtest` public paths (#789).
//!
//! A single-leg strategy backtested over seeded simulator paths, held to
//! expiration and with a profit-or-loss exit, at scaled path counts; and the
//! run statistics folded from a run's results.
//!
//! Run with `cargo bench -p optionstratlib-backtest --bench backtest`.

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_backtest::backtesting::{Simulate, SimulationStatsResult};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    ExitPolicy, WalkParams, WalkType, WalkTypeAble, generator_positive,
};
use optionstratlib_strategies::strategies::{LongCall, ShortPut};
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;

/// Fixed seed: every iteration evaluates the same paths.
const SEED: u64 = 42;

/// Daily steps per path: one 30-day expiry cycle.
const STEPS: usize = 30;

/// A walker that keeps every default stochastic kernel.
#[derive(Clone)]
struct Gbm;

impl WalkTypeAble<Positive, Positive> for Gbm {}

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

fn simulator(paths: usize) -> Simulator<Positive, Positive> {
    let params = WalkParams {
        size: STEPS,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::HUNDRED,
        ),
        walker: Box::new(Gbm),
        walk_type: WalkType::GeometricBrownian {
            dt: pos_or_panic!(1.0 / 365.0),
            drift: dec!(0.05),
            volatility: pos_or_panic!(0.2),
        },
        seed: Some(SEED),
    };
    Simulator::new("bench".to_string(), paths, &params, generator_positive)
        .expect("the fixture simulator builds")
}

fn long_call() -> LongCall {
    LongCall::new(
        "BENCH".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(2.5),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    )
    .expect("the long call fixture builds")
}

fn short_put() -> ShortPut {
    ShortPut::new(
        "BENCH".to_string(),
        pos_or_panic!(95.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(1.5),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    )
    .expect("the short put fixture builds")
}

fn bench_runs(c: &mut Criterion) {
    let mut group = c.benchmark_group("backtest/run");
    group.sample_size(10);
    let call = long_call();
    let put = short_put();
    for paths in [100, 1_000] {
        let sim = simulator(paths);
        group.throughput(Throughput::Elements((paths * STEPS) as u64));
        bench_ok(
            &mut group,
            &format!("long_call_expiration/{paths}_paths"),
            || call.simulate(black_box(&sim), ExitPolicy::Expiration),
        );
        bench_ok(
            &mut group,
            &format!("short_put_profit_or_loss/{paths}_paths"),
            || {
                put.simulate(
                    black_box(&sim),
                    ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0)),
                )
            },
        );
    }
    group.finish();
}

fn bench_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("backtest/statistics");
    for paths in [100, 1_000] {
        let results = long_call()
            .simulate(&simulator(paths), ExitPolicy::Expiration)
            .expect("the fixture run succeeds")
            .results;
        group.throughput(Throughput::Elements(paths as u64));
        bench_ok(&mut group, &format!("from_results/{paths}"), || {
            SimulationStatsResult::from_results(black_box(results.clone()))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_runs, bench_statistics);
criterion_main!(benches);
