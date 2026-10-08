//! Benchmarks of the `optionstratlib-pricing` IV solvers and volatility
//! utilities (#789).
//!
//! The implied-volatility solvers at the money, out of the money and deep
//! out of the money, and the historical, EWMA, GARCH and Heston estimators
//! over scaled return series.
//!
//! Run with `cargo bench -p optionstratlib-pricing --bench volatility`.

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Positive, Side,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::{DETERMINISTIC_RNG_DEFAULT_SEED, TimeFrame, deterministic_rng};
use optionstratlib_pricing::pricing::{OptionPricing, black_scholes};
use optionstratlib_pricing::volatility::{
    adjust_volatility, calculate_iv, constant_volatility, ewma_volatility, garch_volatility,
    historical_volatility, implied_volatility, simulate_heston_volatility,
    uncertain_volatility_bounds,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;

/// Return-series lengths: a quarter, a year and four years of daily returns.
const SERIES_LENGTHS: [usize; 3] = [63, 252, 1_008];

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

/// A call struck at `strike` on a spot of 100, 30 days, priced at 25%
/// volatility so the solvers have a non-trivial target.
fn call_at(strike: f64) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "BENCH".to_string(),
        pos_or_panic!(strike),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.25),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}

/// Daily returns with a deterministic, non-trivial shape and long mantissas.
fn returns(len: usize) -> Vec<Decimal> {
    (0..len)
        .map(|i| (Decimal::from(i % 23) - dec!(11)) / dec!(997))
        .collect()
}

fn bench_implied_volatility(c: &mut Criterion) {
    let mut group = c.benchmark_group("volatility/implied");
    for (label, strike) in [("atm", 100.0), ("otm", 110.0), ("deep_otm", 130.0)] {
        let option = call_at(strike);
        let price = black_scholes(&option).expect("the fixture prices");
        let market = Positive::new_decimal(price).expect("a call price is non-negative");

        bench_ok(
            &mut group,
            &format!("calculate_implied_volatility/{label}"),
            || black_box(&option).calculate_implied_volatility(black_box(price)),
        );
        let mut solver_option = option.clone();
        bench_ok(&mut group, &format!("implied_volatility/{label}"), || {
            implied_volatility(black_box(market), &mut solver_option, 100)
        });
        bench_ok(&mut group, &format!("calculate_iv/{label}"), || {
            calculate_iv(
                black_box(market),
                pos_or_panic!(strike),
                OptionStyle::Call,
                Positive::HUNDRED,
                pos_or_panic!(30.0),
                "BENCH".to_string(),
            )
        });
    }

    // Solving the smile of a 50-strike chain, one solve per strike.
    let chain: Vec<(Options, Decimal)> = (0..50)
        .map(|i| {
            let option = call_at(80.0 + f64::from(i));
            let price = black_scholes(&option).expect("the fixture prices");
            (option, price)
        })
        .collect();
    group.throughput(Throughput::Elements(chain.len() as u64));
    bench_ok(&mut group, "calculate_implied_volatility_chain/50", || {
        chain
            .iter()
            .map(|(o, p)| black_box(o).calculate_implied_volatility(black_box(*p)))
            .collect::<Result<Vec<_>, _>>()
    });
    group.finish();
}

fn bench_estimators(c: &mut Criterion) {
    let mut group = c.benchmark_group("volatility/estimators");
    for len in SERIES_LENGTHS {
        let series = returns(len);
        group.throughput(Throughput::Elements(len as u64));
        bench_ok(&mut group, &format!("constant/{len}"), || {
            constant_volatility(black_box(&series))
        });
        bench_ok(&mut group, &format!("historical_window_21/{len}"), || {
            historical_volatility(black_box(&series), 21)
        });
        bench_ok(&mut group, &format!("ewma/{len}"), || {
            ewma_volatility(black_box(&series), dec!(0.94))
        });
        bench_ok(&mut group, &format!("garch/{len}"), || {
            garch_volatility(black_box(&series), dec!(0.000002), dec!(0.1), dec!(0.85))
        });
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        bench_ok(&mut group, &format!("heston_simulation/{len}"), || {
            simulate_heston_volatility(
                dec!(2.0),
                dec!(0.04),
                dec!(0.3),
                dec!(0.04),
                dec!(0.004),
                len,
                &mut rng,
            )
        });
    }
    group.finish();
}

fn bench_utilities(c: &mut Criterion) {
    let mut group = c.benchmark_group("volatility/utilities");
    let option = call_at(105.0);
    bench_ok(&mut group, "uncertain_volatility_bounds", || {
        uncertain_volatility_bounds(black_box(&option), pos_or_panic!(0.15), pos_or_panic!(0.35))
    });
    bench_ok(&mut group, "adjust_volatility_day_to_year", || {
        adjust_volatility(
            black_box(pos_or_panic!(0.0126)),
            TimeFrame::Day,
            TimeFrame::Year,
        )
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_implied_volatility,
    bench_estimators,
    bench_utilities
);
criterion_main!(benches);
