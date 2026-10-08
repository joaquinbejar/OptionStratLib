//! Benchmarks of the `optionstratlib-pricing` Greeks (#789).
//!
//! Every analytic Greek one at a time, the full snapshot that shares the
//! Black-Scholes kernels, the finite-difference Greeks, the Black-76 and
//! Garman-Kohlhagen variants, the `d1` / `N(x)` kernels, and the per-chain
//! sweep that multiplies all of it by the strike count.
//!
//! Run with `cargo bench -p optionstratlib-pricing --bench greeks`.

use chrono::Utc;
use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_core::model::option::ExoticParams;
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Position, Positive, Side,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::greeks::numerical::{
    numerical_delta, numerical_gamma, numerical_rho, numerical_vega,
};
use optionstratlib_pricing::greeks::{
    Greeks, big_n, d1, delta_b76, delta_gk, gamma_b76, gamma_gk, n, vega_b76, vega_gk,
};
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;

/// Strike counts of the chain sweep: a weekly, a monthly, a full chain.
const CHAIN_SIZES: [usize; 3] = [10, 50, 200];

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

/// Strike `strike` on a spot of 100, 30 days, 20% volatility, 5% rate, 1%
/// yield. Well before expiry, so no Greek takes its `T == 0` early return.
fn option_at(strike: Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "BENCH".to_string(),
        strike,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        OptionStyle::Call,
        pos_or_panic!(0.01),
        None,
    )
}

/// The same contract as an FX option: Garman-Kohlhagen reads the foreign
/// rate from the exotic parameters.
fn fx_option_at(strike: Positive) -> Options {
    let mut option = option_at(strike);
    option.exotic_params = Some(ExoticParams {
        foreign_rate: Some(dec!(0.03)),
        ..ExoticParams::default()
    });
    option
}

fn bench_single_greeks(c: &mut Criterion) {
    let mut group = c.benchmark_group("greeks/analytic");
    let option = option_at(pos_or_panic!(105.0));

    bench_ok(&mut group, "delta", || black_box(&option).delta());
    bench_ok(&mut group, "gamma", || black_box(&option).gamma());
    bench_ok(&mut group, "theta", || black_box(&option).theta());
    bench_ok(&mut group, "vega", || black_box(&option).vega());
    bench_ok(&mut group, "rho", || black_box(&option).rho());
    bench_ok(&mut group, "rho_d", || black_box(&option).rho_d());
    bench_ok(&mut group, "alpha", || black_box(&option).alpha());
    bench_ok(&mut group, "vanna", || black_box(&option).vanna());
    bench_ok(&mut group, "vomma", || black_box(&option).vomma());
    bench_ok(&mut group, "veta", || black_box(&option).veta());
    bench_ok(&mut group, "charm", || black_box(&option).charm());
    bench_ok(&mut group, "color", || black_box(&option).color());
    // All twelve through the shared kernels: compare with the sum of the
    // single Greeks above to see what the memoisation saves.
    bench_ok(&mut group, "greeks_snapshot", || {
        black_box(&option).greeks()
    });

    let position = Position::new(
        option.clone(),
        pos_or_panic!(1.2),
        Utc::now(),
        Positive::ZERO,
        Positive::ZERO,
        None,
        None,
    );
    bench_ok(&mut group, "position_greeks_snapshot", || {
        black_box(&position).greeks()
    });
    group.finish();
}

fn bench_model_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("greeks/variants");
    let option = option_at(pos_or_panic!(105.0));

    bench_ok(&mut group, "numerical_delta", || {
        numerical_delta(black_box(&option))
    });
    bench_ok(&mut group, "numerical_gamma", || {
        numerical_gamma(black_box(&option))
    });
    bench_ok(&mut group, "numerical_vega", || {
        numerical_vega(black_box(&option))
    });
    // `numerical_theta` is not benchmarked: it returns an error for every
    // input (reported under #789).
    bench_ok(&mut group, "numerical_rho", || {
        numerical_rho(black_box(&option))
    });
    bench_ok(&mut group, "delta_b76", || delta_b76(black_box(&option)));
    bench_ok(&mut group, "gamma_b76", || gamma_b76(black_box(&option)));
    bench_ok(&mut group, "vega_b76", || vega_b76(black_box(&option)));
    let fx = fx_option_at(pos_or_panic!(105.0));
    bench_ok(&mut group, "delta_gk", || delta_gk(black_box(&fx)));
    bench_ok(&mut group, "gamma_gk", || gamma_gk(black_box(&fx)));
    bench_ok(&mut group, "vega_gk", || vega_gk(black_box(&fx)));
    group.finish();
}

fn bench_kernels(c: &mut Criterion) {
    let mut group = c.benchmark_group("greeks/kernels");
    let x = dec!(0.3172);
    bench_ok(&mut group, "d1", || {
        d1(
            black_box(Positive::HUNDRED),
            pos_or_panic!(105.0),
            dec!(0.04),
            pos_or_panic!(0.0822),
            pos_or_panic!(0.2),
        )
    });
    bench_ok(&mut group, "n_pdf", || n(black_box(x)));
    bench_ok(&mut group, "big_n_cdf", || big_n(black_box(x)));
    group.finish();
}

fn bench_chain_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("greeks/chain");
    for size in CHAIN_SIZES {
        // Strikes from 70% to 130% of the spot.
        let options: Vec<Options> = (0..size)
            .map(|i| option_at(pos_or_panic!(70.0 + 60.0 * (i as f64) / (size as f64))))
            .collect();
        group.throughput(Throughput::Elements(size as u64));
        bench_ok(&mut group, &format!("delta/{size}"), || {
            options
                .iter()
                .map(|o| black_box(o).delta())
                .collect::<Result<Vec<_>, _>>()
        });
        bench_ok(&mut group, &format!("greeks_snapshot/{size}"), || {
            options
                .iter()
                .map(|o| black_box(o).greeks())
                .collect::<Result<Vec<_>, _>>()
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_single_greeks,
    bench_model_variants,
    bench_kernels,
    bench_chain_sweep
);
criterion_main!(benches);
