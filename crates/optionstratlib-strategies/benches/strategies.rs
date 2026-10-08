//! Benchmarks of the `optionstratlib-strategies` public paths (#789).
//!
//! Construction of strategies from one to four legs, break-evens, P&L at a
//! price and across a sweep, the profit metrics, the strategy Greeks, the
//! probability analysis, delta neutrality, and the chain optimisers on the
//! repository's real SP500 chain and on synthetic chains of growing size.
//!
//! Run with `cargo bench -p optionstratlib-strategies --bench strategies`.

use criterion::measurement::WallTime;
use criterion::{
    BatchSize, BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main,
};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib_market::chains::{FindOptimalSide, OptionChain};
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::pricing::Profit;
use optionstratlib_strategies::strategies::base::{BreakEvenable, Optimizable};
use optionstratlib_strategies::strategies::probabilities::ProbabilityAnalysis;
use optionstratlib_strategies::strategies::{
    BullCallSpread, DeltaNeutrality, IronButterfly, IronCondor, LongButterflySpread, LongCall,
    ShortStrangle, Strategies,
};
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;

/// The real 45-strike SP500 chain the optimiser integration tests read.
const SP500_JSON: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
);

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

fn spot() -> Positive {
    pos_or_panic!(5781.88)
}

fn expiration() -> ExpirationDate {
    ExpirationDate::Days(pos_or_panic!(30.0))
}

fn long_call() -> LongCall {
    LongCall::new(
        "SP500".to_string(),
        pos_or_panic!(5800.0),
        expiration(),
        pos_or_panic!(0.18),
        Positive::ONE,
        spot(),
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(80.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .expect("the long call fixture builds")
}

fn bull_call_spread() -> BullCallSpread {
    BullCallSpread::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5750.0),
        pos_or_panic!(5820.0),
        expiration(),
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

fn short_strangle() -> ShortStrangle {
    ShortStrangle::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5950.0),
        pos_or_panic!(5600.0),
        expiration(),
        pos_or_panic!(0.18),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(30.0),
        pos_or_panic!(35.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .expect("the short strangle fixture builds")
}

fn long_butterfly() -> LongButterflySpread {
    LongButterflySpread::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5700.0),
        pos_or_panic!(5780.0),
        pos_or_panic!(5860.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(120.0),
        pos_or_panic!(70.0),
        pos_or_panic!(35.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .expect("the long butterfly fixture builds")
}

fn iron_condor() -> IronCondor {
    IronCondor::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5900.0),
        pos_or_panic!(5650.0),
        pos_or_panic!(5950.0),
        pos_or_panic!(5600.0),
        expiration(),
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

fn iron_butterfly() -> IronButterfly {
    IronButterfly::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5780.0),
        pos_or_panic!(5880.0),
        pos_or_panic!(5680.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(95.0),
        pos_or_panic!(90.0),
        pos_or_panic!(50.0),
        pos_or_panic!(45.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .expect("the iron butterfly fixture builds")
}

fn sp500_chain() -> OptionChain {
    OptionChain::load_from_json(SP500_JSON).expect("the SP500 fixture chain loads")
}

/// A synthetic 30-day chain on the SP500 spot with `half_width` strikes
/// either side of the money.
fn synthetic_chain(half_width: usize) -> OptionChain {
    let params = OptionChainBuildParams::new(
        "SP500".to_string(),
        Some(pos_or_panic!(100.0)),
        half_width,
        None,
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(spot())),
            Some(expiration()),
            Some(dec!(0.05)),
            Some(Positive::ZERO),
            Some("SP500".to_string()),
        ),
        pos_or_panic!(0.18),
    );
    OptionChain::build_chain(&params).expect("the synthetic chain builds")
}

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/construction");
    group.bench_function("long_call/1_leg", |b| b.iter(|| black_box(long_call())));
    group.bench_function("bull_call_spread/2_legs", |b| {
        b.iter(|| black_box(bull_call_spread()))
    });
    group.bench_function("short_strangle/2_legs", |b| {
        b.iter(|| black_box(short_strangle()))
    });
    group.bench_function("long_butterfly/3_legs", |b| {
        b.iter(|| black_box(long_butterfly()))
    });
    group.bench_function("iron_condor/4_legs", |b| {
        b.iter(|| black_box(iron_condor()))
    });
    group.bench_function("iron_butterfly/4_legs", |b| {
        b.iter(|| black_box(iron_butterfly()))
    });
    group.finish();
}

/// The read-only evaluation paths, measured on one strategy.
///
/// `bounded_ratio` is false for a strategy whose maximum profit is
/// unlimited: its profit ratio is documented to return
/// `StrategyError::NumericConversion` (#788), so there is no successful
/// path to time.
fn bench_evaluation<S>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    label: &str,
    strategy: &S,
    bounded_ratio: bool,
) where
    S: Strategies + BreakEvenable + Profit + Greeks + Clone,
{
    let at = pos_or_panic!(5800.0);
    bench_ok(group, &format!("{label}/get_break_even_points"), || {
        black_box(strategy).get_break_even_points().map(|p| p.len())
    });
    group.bench_function(format!("{label}/update_break_even_points"), |b| {
        b.iter_batched(
            || strategy.clone(),
            |mut s| {
                let _ = black_box(s.update_break_even_points());
                s
            },
            BatchSize::SmallInput,
        )
    });
    bench_ok(group, &format!("{label}/calculate_profit_at"), || {
        black_box(strategy).calculate_profit_at(black_box(&at))
    });
    bench_ok(group, &format!("{label}/get_max_profit"), || {
        black_box(strategy).get_max_profit()
    });
    bench_ok(group, &format!("{label}/get_max_loss"), || {
        black_box(strategy).get_max_loss()
    });
    bench_ok(group, &format!("{label}/get_total_cost"), || {
        black_box(strategy).get_total_cost()
    });
    bench_ok(group, &format!("{label}/get_profit_area"), || {
        black_box(strategy).get_profit_area()
    });
    if bounded_ratio {
        bench_ok(group, &format!("{label}/get_profit_ratio"), || {
            black_box(strategy).get_profit_ratio()
        });
    }
    bench_ok(group, &format!("{label}/greeks"), || {
        black_box(strategy).greeks()
    });

    // 201 prices across +-10% of the spot: the P&L chart and the profit
    // area both walk a sweep like this.
    let prices: Vec<Positive> = (0..=200)
        .map(|i| pos_or_panic!(5200.0 + 6.0 * f64::from(i)))
        .collect();
    bench_ok(
        group,
        &format!("{label}/calculate_profit_sweep_201"),
        || {
            prices
                .iter()
                .map(|p| strategy.calculate_profit_at(black_box(p)))
                .collect::<Result<Vec<_>, _>>()
        },
    );
}

fn bench_evaluations(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/evaluation");
    bench_evaluation(&mut group, "long_call", &long_call(), false);
    bench_evaluation(&mut group, "bull_call_spread", &bull_call_spread(), true);
    bench_evaluation(&mut group, "short_strangle", &short_strangle(), true);
    bench_evaluation(&mut group, "long_butterfly", &long_butterfly(), true);
    bench_evaluation(&mut group, "iron_condor", &iron_condor(), true);
    bench_evaluation(&mut group, "iron_butterfly", &iron_butterfly(), true);
    group.finish();
}

fn bench_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/analysis");
    group.sample_size(20);
    let condor = iron_condor();
    let strangle = short_strangle();
    bench_ok(&mut group, "iron_condor/probability_of_profit", || {
        black_box(&condor).probability_of_profit(None, None)
    });
    bench_ok(&mut group, "iron_condor/expected_value", || {
        black_box(&condor).expected_value(None, None)
    });
    bench_ok(&mut group, "iron_condor/analyze_probabilities", || {
        black_box(&condor).analyze_probabilities(None, None)
    });
    bench_ok(&mut group, "short_strangle/delta_neutrality", || {
        black_box(&strangle).delta_neutrality()
    });
    bench_ok(&mut group, "short_strangle/delta_adjustments", || {
        black_box(&strangle).delta_adjustments()
    });
    group.finish();
}

/// Times one optimiser call: the optimiser mutates the strategy in place,
/// so each iteration starts from a fresh clone that is not timed.
///
/// As in [`bench_ok`], the search must find a candidate on the fixture
/// before it is timed: a search that returns `NoValidCandidate` at once
/// would report a misleadingly cheap number.
fn bench_optimiser<S>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    name: &str,
    strategy: &S,
    chain: &OptionChain,
    area: bool,
) where
    S: Optimizable + Clone,
{
    let search = |s: &mut S| {
        if area {
            s.get_best_area(black_box(chain), FindOptimalSide::All)
        } else {
            s.get_best_ratio(black_box(chain), FindOptimalSide::All)
        }
    };
    if let Err(e) = search(&mut strategy.clone()) {
        panic!("bench `{name}`: the search found no candidate on the fixture: {e:?}");
    }
    group.bench_function(name, |b| {
        b.iter_batched(
            || strategy.clone(),
            |mut s| {
                let result = search(&mut s);
                (s, black_box(result))
            },
            BatchSize::LargeInput,
        )
    });
}

fn bench_optimisers(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategies/optimiser");
    group.sample_size(10);
    let sp500 = sp500_chain();
    let strikes = sp500.options.len();
    let spread = bull_call_spread();
    let strangle = short_strangle();
    let butterfly = long_butterfly();
    let condor = iron_condor();

    bench_optimiser(
        &mut group,
        &format!("bull_call_spread_best_ratio/sp500_{strikes}"),
        &spread,
        &sp500,
        false,
    );
    bench_optimiser(
        &mut group,
        &format!("bull_call_spread_best_area/sp500_{strikes}"),
        &spread,
        &sp500,
        true,
    );
    bench_optimiser(
        &mut group,
        &format!("short_strangle_best_area/sp500_{strikes}"),
        &strangle,
        &sp500,
        true,
    );
    bench_optimiser(
        &mut group,
        &format!("long_butterfly_best_ratio/sp500_{strikes}"),
        &butterfly,
        &sp500,
        false,
    );
    bench_optimiser(
        &mut group,
        &format!("iron_condor_best_ratio/sp500_{strikes}"),
        &condor,
        &sp500,
        false,
    );

    // The same two- and four-leg searches on synthetic chains of growing
    // size: the combination count is n^2 and n^4 before filtering.
    for half_width in [5, 10, 20] {
        let chain = synthetic_chain(half_width);
        let n = chain.options.len();
        group.throughput(Throughput::Elements(n as u64));
        bench_optimiser(
            &mut group,
            &format!("bull_call_spread_best_ratio/synthetic_{n}"),
            &spread,
            &chain,
            false,
        );
        bench_optimiser(
            &mut group,
            &format!("iron_condor_best_ratio/synthetic_{n}"),
            &condor,
            &chain,
            false,
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_construction,
    bench_evaluations,
    bench_analysis,
    bench_optimisers
);
criterion_main!(benches);
