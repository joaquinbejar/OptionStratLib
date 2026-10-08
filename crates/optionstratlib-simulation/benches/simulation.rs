//! Benchmarks of the `optionstratlib-simulation` public paths (#789).
//!
//! One seeded path of every walk model at two lengths, the simulator at
//! scaled path and step counts, the Monte-Carlo price read off a simulator,
//! the Ornstein-Uhlenbeck generator and the expanding-window volatilities.
//!
//! Run with `cargo bench -p optionstratlib-simulation --bench simulation`.

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Positive, Side,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    WalkParams, WalkType, WalkTypeAble, expanding_window_vols, generate_ou_process,
    generator_positive,
};
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;

/// Fixed seed: every iteration generates the same paths.
const SEED: u64 = 42;

/// Path lengths: a trading year and four of them.
const WALK_LENGTHS: [usize; 2] = [252, 1_008];

/// A walker that keeps every default stochastic kernel.
#[derive(Clone)]
struct Walker;

impl WalkTypeAble<Positive, Positive> for Walker {}

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

fn dt() -> Positive {
    pos_or_panic!(1.0 / 252.0)
}

/// A deterministic daily price history for the `Historical` model.
fn history(len: usize) -> Vec<Positive> {
    (0..len)
        .map(|i| {
            let wave = ((i % 13) as f64 - 6.0) / 5.0;
            pos_or_panic!(100.0 + wave + (i as f64) / 211.0)
        })
        .collect()
}

/// Every walk model with realistic daily parameters.
fn walk_models(len: usize) -> Vec<(&'static str, WalkType)> {
    let volatility = pos_or_panic!(0.2);
    vec![
        (
            "brownian",
            WalkType::Brownian {
                dt: dt(),
                drift: dec!(0.05),
                volatility,
            },
        ),
        (
            "geometric_brownian",
            WalkType::GeometricBrownian {
                dt: dt(),
                drift: dec!(0.05),
                volatility,
            },
        ),
        (
            "log_returns",
            WalkType::LogReturns {
                dt: dt(),
                expected_return: dec!(0.05),
                volatility,
                autocorrelation: Some(dec!(0.1)),
            },
        ),
        (
            "mean_reverting",
            WalkType::MeanReverting {
                dt: dt(),
                volatility,
                speed: Positive::ONE,
                mean: Positive::HUNDRED,
            },
        ),
        (
            "jump_diffusion",
            WalkType::JumpDiffusion {
                dt: dt(),
                drift: dec!(0.05),
                volatility,
                intensity: pos_or_panic!(0.1),
                jump_mean: dec!(-0.02),
                jump_volatility: pos_or_panic!(0.05),
            },
        ),
        (
            "garch",
            WalkType::Garch {
                dt: dt(),
                drift: dec!(0.05),
                volatility,
                alpha: pos_or_panic!(0.1),
                beta: pos_or_panic!(0.85),
            },
        ),
        (
            "heston",
            WalkType::Heston {
                dt: dt(),
                drift: dec!(0.05),
                volatility,
                kappa: pos_or_panic!(2.0),
                theta: pos_or_panic!(0.04),
                xi: pos_or_panic!(0.3),
                rho: dec!(-0.7),
            },
        ),
        (
            "custom",
            WalkType::Custom {
                dt: dt(),
                drift: dec!(0.05),
                volatility,
                vov: pos_or_panic!(0.2),
                vol_speed: Positive::ONE,
                vol_mean: volatility,
            },
        ),
        (
            "telegraph",
            WalkType::Telegraph {
                dt: dt(),
                drift: dec!(0.05),
                volatility,
                lambda_up: pos_or_panic!(0.5),
                lambda_down: pos_or_panic!(0.5),
                vol_multiplier_up: None,
                vol_multiplier_down: None,
            },
        ),
        (
            "historical",
            WalkType::Historical {
                timeframe: TimeFrame::Day,
                prices: history(len),
                symbol: None,
            },
        ),
    ]
}

fn walk_params(size: usize, walk_type: WalkType) -> WalkParams<Positive, Positive> {
    WalkParams {
        size,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(2000.0)),
            Positive::HUNDRED,
        ),
        walk_type,
        walker: Box::new(Walker),
        seed: Some(SEED),
    }
}

fn gbm(size: usize) -> WalkParams<Positive, Positive> {
    walk_params(
        size,
        WalkType::GeometricBrownian {
            dt: dt(),
            drift: dec!(0.05),
            volatility: pos_or_panic!(0.2),
        },
    )
}

fn bench_walks(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation/walk");
    for len in WALK_LENGTHS {
        group.throughput(Throughput::Elements(len as u64));
        for (label, walk_type) in walk_models(len) {
            let params = walk_params(len, walk_type);
            bench_ok(&mut group, &format!("{label}/{len}"), || {
                generator_positive(black_box(&params))
            });
        }
    }
    group.finish();
}

fn bench_simulator(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation/simulator");
    group.sample_size(10);
    for (paths, steps) in [(100, 30), (1_000, 30), (100, 252)] {
        let params = gbm(steps);
        group.throughput(Throughput::Elements((paths * steps) as u64));
        bench_ok(
            &mut group,
            &format!("new/{paths}_paths_x_{steps}_steps"),
            || {
                Simulator::new(
                    "bench".to_string(),
                    paths,
                    black_box(&params),
                    generator_positive,
                )
            },
        );
    }

    let simulator = Simulator::new("bench".to_string(), 1_000, &gbm(30), generator_positive)
        .expect("the fixture simulator builds");
    let option = Options::new(
        OptionType::European,
        Side::Long,
        "BENCH".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    );
    group.throughput(Throughput::Elements(1_000));
    group.bench_function("get_last_positive_values/1000_paths", |b| {
        b.iter(|| black_box(&simulator).get_last_positive_values())
    });
    bench_ok(&mut group, "get_mc_option_price/1000_paths", || {
        black_box(&simulator).get_mc_option_price(black_box(&option))
    });
    group.finish();
}

fn bench_processes(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation/process");
    for len in WALK_LENGTHS {
        group.throughput(Throughput::Elements(len as u64));
        bench_ok(&mut group, &format!("generate_ou_process/{len}"), || {
            generate_ou_process(
                Positive::HUNDRED,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(0.2),
                dt(),
                black_box(len),
            )
        });
        let prices = history(len);
        bench_ok(&mut group, &format!("expanding_window_vols/{len}"), || {
            expanding_window_vols(black_box(&prices), TimeFrame::Day)
        });
    }
    group.finish();
}

criterion_group!(benches, bench_walks, bench_simulator, bench_processes);
criterion_main!(benches);
