//! Benchmarks of the `optionstratlib-market` chain and series paths that
//! need no feature (#789).
//!
//! Chain construction at scaled strike counts, the lookups and leg
//! iterators strategies drive, Greek refreshes, exposures, JSON
//! serialization, and series construction. File I/O lives in `chains_io`
//! (`io`), the simulation-backed generators in `synthetic` (`synthetic`).
//!
//! Run with `cargo bench -p optionstratlib-market --bench chains`.

use criterion::measurement::WallTime;
use criterion::{
    BatchSize, BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main,
};
use optionstratlib_core::model::{ExpirationDate, OptionStyle, Positive, Side};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::utils::OptionDataPriceParams;
use optionstratlib_market::chains::{FindOptimalSide, OptionChain, OptionChainBuildParams};
use optionstratlib_market::series::{OptionSeries, OptionSeriesBuildParams};
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;

/// Per-side half-widths handed to `build_chain`. With no explicit strike
/// interval the builder derives one from the half-width, so the chain grows
/// with it; each case is labelled by the strike count it actually produces.
const HALF_WIDTHS: [usize; 4] = [10, 25, 50, 100];

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

/// A 30-day chain on a spot of 100 with a negative skew and a mild smile.
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
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            Some(pos_or_panic!(0.01)),
            Some("BENCH".to_string()),
        ),
        pos_or_panic!(0.2),
    )
}

fn chain(half_width: usize) -> OptionChain {
    OptionChain::build_chain(&build_params(half_width)).expect("the fixture chain builds")
}

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("market/chain_build");
    group.sample_size(20);
    for half_width in HALF_WIDTHS {
        let params = build_params(half_width);
        let strikes = chain(half_width).options.len();
        group.throughput(Throughput::Elements(strikes as u64));
        bench_ok(&mut group, &format!("build_chain/{strikes}"), || {
            OptionChain::build_chain(black_box(&params))
        });
        let with_greeks = params.clone().with_greek_snapshots(true);
        bench_ok(
            &mut group,
            &format!("build_chain_with_greeks/{strikes}"),
            || OptionChain::build_chain(black_box(&with_greeks)),
        );
        let built = chain(half_width);
        bench_ok(&mut group, &format!("to_build_params/{strikes}"), || {
            black_box(&built).to_build_params()
        });
    }
    group.finish();
}

fn bench_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("market/chain_lookup");
    for half_width in [10, 50] {
        let chain = chain(half_width);
        let strikes = chain.options.len();
        let probe = chain
            .atm_strike()
            .cloned()
            .expect("the chain has an ATM strike");
        bench_ok(&mut group, &format!("atm_option_data/{strikes}"), || {
            black_box(&chain).atm_option_data()
        });
        bench_ok(&mut group, &format!("get_strikes/{strikes}"), || {
            black_box(&chain).get_strikes()
        });
        bench_ok(
            &mut group,
            &format!("get_optiondata_with_strike/{strikes}"),
            || black_box(&chain).get_optiondata_with_strike(black_box(&probe)),
        );
        group.bench_function(format!("get_call_price/{strikes}"), |bench| {
            bench.iter(|| black_box(&chain).get_call_price(black_box(probe)))
        });
        group.bench_function(format!("filter_option_data_upper/{strikes}"), |bench| {
            bench.iter(|| black_box(&chain).filter_option_data(FindOptimalSide::Upper))
        });
        bench_ok(
            &mut group,
            &format!("get_atm_implied_volatility/{strikes}"),
            || black_box(&chain).get_atm_implied_volatility(),
        );
        bench_ok(
            &mut group,
            &format!("get_position_with_delta/{strikes}"),
            || black_box(&chain).get_position_with_delta(dec!(0.3), Side::Long, OptionStyle::Call),
        );
    }
    group.finish();
}

fn bench_leg_iterators(c: &mut Criterion) {
    let mut group = c.benchmark_group("market/chain_iterators");
    // The combination counts grow as n^2, n^3 and n^4: the optimisers walk
    // exactly these.
    for half_width in [10, 25] {
        let chain = chain(half_width);
        let strikes = chain.options.len();
        group.bench_function(format!("double_iter_count/{strikes}"), |bench| {
            bench.iter(|| black_box(&chain).get_double_iter().count())
        });
        group.bench_function(format!("triple_iter_count/{strikes}"), |bench| {
            bench.iter(|| black_box(&chain).get_triple_iter().count())
        });
        group.bench_function(format!("quad_iter_count/{strikes}"), |bench| {
            bench.iter(|| black_box(&chain).get_quad_iter().count())
        });
    }
    group.finish();
}

fn bench_refresh(c: &mut Criterion) {
    let mut group = c.benchmark_group("market/chain_refresh");
    for half_width in [10, 50] {
        let chain = chain(half_width);
        let strikes = chain.options.len();
        group.throughput(Throughput::Elements(strikes as u64));
        group.bench_function(format!("update_greeks/{strikes}"), |bench| {
            bench.iter_batched(
                || chain.clone(),
                |mut c| {
                    c.update_greeks();
                    c
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_function(format!("update_mid_prices/{strikes}"), |bench| {
            bench.iter_batched(
                || chain.clone(),
                |mut c| {
                    c.update_mid_prices();
                    c
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_function(format!("clone/{strikes}"), |bench| {
            bench.iter(|| black_box(&chain).clone())
        });
        bench_ok(&mut group, &format!("gamma_exposure/{strikes}"), || {
            black_box(&chain).gamma_exposure()
        });
        bench_ok(&mut group, &format!("delta_exposure/{strikes}"), || {
            black_box(&chain).delta_exposure()
        });
        bench_ok(&mut group, &format!("vega_exposure/{strikes}"), || {
            black_box(&chain).vega_exposure()
        });
    }
    group.finish();
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("market/chain_serde");
    for half_width in [10, 50] {
        let chain = chain(half_width);
        let strikes = chain.options.len();
        let json = serde_json::to_string(&chain).expect("a chain serializes to JSON");
        group.throughput(Throughput::Bytes(json.len() as u64));
        bench_ok(&mut group, &format!("to_json/{strikes}"), || {
            serde_json::to_string(black_box(&chain))
        });
        bench_ok(&mut group, &format!("from_json/{strikes}"), || {
            serde_json::from_str::<OptionChain>(black_box(&json))
        });
    }
    group.finish();
}

fn bench_series(c: &mut Criterion) {
    let mut group = c.benchmark_group("market/series_build");
    group.sample_size(20);
    for expirations in [3_usize, 12] {
        let days: Vec<Positive> = (1..=expirations)
            .map(|i| pos_or_panic!(7.0 * i as f64))
            .collect();
        let params = OptionSeriesBuildParams::new(build_params(10), days);
        group.throughput(Throughput::Elements(expirations as u64));
        bench_ok(&mut group, &format!("build_series/{expirations}"), || {
            OptionSeries::build_series(black_box(&params))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_construction,
    bench_lookups,
    bench_leg_iterators,
    bench_refresh,
    bench_serialization,
    bench_series
);
criterion_main!(benches);
