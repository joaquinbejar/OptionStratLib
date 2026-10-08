//! Benchmarks of the `optionstratlib-visualization` data paths (#789).
//!
//! Chart data generation (`Graph::graph_data`) for curves, surfaces,
//! options, positions, strategies, random walks and simulators, and the
//! terminal tables rendered to a `String`. Nothing here renders through
//! Plotly or writes a file, so the benches need no feature.
//!
//! Run with `cargo bench -p optionstratlib-visualization --bench visualization`.

use chrono::Utc;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_backtest::backtesting::{Simulate, SimulationStatsResult};
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Position, Positive, Side,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_market::chains::OptionChain;
use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib_math::curves::{Curve, Point2D};
use optionstratlib_math::geometrics::GeometricObject;
use optionstratlib_math::surfaces::{Point3D, Surface};
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    ExitPolicy, WalkParams, WalkType, WalkTypeAble, generator_positive,
};
use optionstratlib_strategies::strategies::{BullCallSpread, IronCondor, LongCall};
use optionstratlib_visualization::visualization::Graph;
use optionstratlib_visualization::visualization::terminal::{ChainReport, SimulationReport};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::hint::black_box;

/// A walker that keeps every default stochastic kernel.
#[derive(Clone)]
struct Gbm;

impl WalkTypeAble<Positive, Positive> for Gbm {}

fn curve(points: usize) -> Curve {
    Curve::from_vector(
        (0..points)
            .map(|i| {
                let x = dec!(10) * Decimal::from(i) / Decimal::from(points - 1);
                Point2D::new(x, x * x / dec!(7))
            })
            .collect(),
    )
}

fn surface(side: usize) -> Surface {
    Surface::from_vector(
        (0..side)
            .flat_map(|i| {
                (0..side).map(move |j| {
                    let x = Decimal::from(i);
                    let y = Decimal::from(j);
                    Point3D::new(x, y, x * y / dec!(7))
                })
            })
            .collect(),
    )
}

fn call_option() -> Options {
    Options::new(
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
    )
}

fn bull_call_spread() -> BullCallSpread {
    BullCallSpread::new(
        "SP500".to_string(),
        pos_or_panic!(5781.88),
        pos_or_panic!(5750.0),
        pos_or_panic!(5820.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(85.04),
        pos_or_panic!(29.85),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.73),
        pos_or_panic!(0.73),
    )
    .expect("the bull call spread fixture builds")
}

fn iron_condor() -> IronCondor {
    IronCondor::new(
        "SP500".to_string(),
        pos_or_panic!(5781.88),
        pos_or_panic!(5900.0),
        pos_or_panic!(5650.0),
        pos_or_panic!(5950.0),
        pos_or_panic!(5600.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(42.0),
        pos_or_panic!(48.0),
        pos_or_panic!(28.0),
        pos_or_panic!(33.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .expect("the iron condor fixture builds")
}

fn simulator(paths: usize, steps: usize) -> Simulator<Positive, Positive> {
    let params = WalkParams {
        size: steps,
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
        seed: Some(42),
    };
    Simulator::new("bench".to_string(), paths, &params, generator_positive)
        .expect("the fixture simulator builds")
}

fn chain() -> OptionChain {
    let params = OptionChainBuildParams::new(
        "BENCH".to_string(),
        None,
        10,
        None,
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            Some(pos_or_panic!(0.01)),
            Some("BENCH".to_string()),
        ),
        pos_or_panic!(0.2),
    );
    OptionChain::build_chain(&params).expect("the fixture chain builds")
}

fn bench_geometry(c: &mut Criterion) {
    let mut group = c.benchmark_group("visualization/geometry");
    for points in [128, 2_048] {
        let curve = curve(points);
        group.throughput(Throughput::Elements(points as u64));
        group.bench_function(format!("curve_graph_data/{points}"), |b| {
            b.iter(|| black_box(&curve).graph_data())
        });
    }
    let curves: Vec<Curve> = (0..5).map(|_| curve(128)).collect();
    group.throughput(Throughput::Elements(5 * 128));
    group.bench_function("curves_graph_data/5x128", |b| {
        b.iter(|| black_box(&curves).graph_data())
    });
    for side in [16, 64] {
        let surface = surface(side);
        group.throughput(Throughput::Elements((side * side) as u64));
        group.bench_function(format!("surface_graph_data/{side}x{side}"), |b| {
            b.iter(|| black_box(&surface).graph_data())
        });
    }
    group.finish();
}

fn bench_payoff_charts(c: &mut Criterion) {
    let mut group = c.benchmark_group("visualization/payoff");
    let option = call_option();
    let position = Position::new(
        call_option(),
        pos_or_panic!(2.5),
        Utc::now(),
        Positive::ZERO,
        Positive::ZERO,
        None,
        None,
    );
    let long_call = LongCall::new(
        "BENCH".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(2.5),
        Positive::ZERO,
        Positive::ZERO,
    )
    .expect("the long call fixture builds");
    let spread = bull_call_spread();
    let condor = iron_condor();

    group.bench_function("options_graph_data", |b| {
        b.iter(|| black_box(&option).graph_data())
    });
    group.bench_function("position_graph_data", |b| {
        b.iter(|| black_box(&position).graph_data())
    });
    group.bench_function("long_call_graph_data", |b| {
        b.iter(|| black_box(&long_call).graph_data())
    });
    group.bench_function("bull_call_spread_graph_data", |b| {
        b.iter(|| black_box(&spread).graph_data())
    });
    group.bench_function("iron_condor_graph_data", |b| {
        b.iter(|| black_box(&condor).graph_data())
    });
    group.finish();
}

fn bench_simulation_charts(c: &mut Criterion) {
    let mut group = c.benchmark_group("visualization/simulation");
    for paths in [10, 100] {
        let sim = simulator(paths, 252);
        group.throughput(Throughput::Elements((paths * 252) as u64));
        group.bench_function(format!("simulator_graph_data/{paths}_paths_x_252"), |b| {
            b.iter(|| black_box(&sim).graph_data())
        });
    }
    let sim = simulator(1, 1_008);
    let walk = sim.first().expect("the simulator has a walk");
    group.throughput(Throughput::Elements(1_008));
    group.bench_function("random_walk_graph_data/1008", |b| {
        b.iter(|| black_box(walk).graph_data())
    });
    group.finish();
}

fn bench_terminal(c: &mut Criterion) {
    let mut group = c.benchmark_group("visualization/terminal");
    let chain = chain();
    group.bench_function(format!("chain_render_table/{}", chain.options.len()), |b| {
        b.iter(|| black_box(&chain).render_table())
    });
    let stats = long_call_backtest();
    group.bench_function("simulation_render_summary/100", |b| {
        b.iter(|| black_box(&stats).render_summary())
    });
    group.bench_function("simulation_render_individual_results/100", |b| {
        b.iter(|| black_box(&stats).render_individual_results())
    });
    group.finish();
}

/// A 100-path long-call backtest whose report the terminal benches render.
fn long_call_backtest() -> SimulationStatsResult {
    let long_call = LongCall::new(
        "BENCH".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(2.5),
        Positive::ZERO,
        Positive::ZERO,
    )
    .expect("the long call fixture builds");
    long_call
        .simulate(&simulator(100, 30), ExitPolicy::Expiration)
        .expect("the fixture backtest runs")
}

criterion_group!(
    benches,
    bench_geometry,
    bench_payoff_charts,
    bench_simulation_charts,
    bench_terminal
);
criterion_main!(benches);
