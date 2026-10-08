//! Benchmarks of the `optionstratlib-analytics` public paths (#789).
//!
//! The probability kernels, option and position P&L, SPAN margin, the
//! risk-neutral density, the chain projections onto curves and surfaces, and
//! the chain metrics (curves, and surfaces at scaled grid sizes).
//!
//! Run with `cargo bench -p optionstratlib-analytics --bench analytics`.

use chrono::Utc;
use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_analytics::analytics::{
    OptionChainProjections, PriceTrend, RNDAnalysis, RNDParameters, VolatilityAdjustment,
    calculate_price_probability, calculate_single_point_probability,
};
use optionstratlib_analytics::metrics::{
    DeltaGammaProfileCurve, DollarGammaCurve, ImpliedVolatilityCurve, ImpliedVolatilitySurface,
    PriceShockCurve, PutCallRatioCurve, RiskReversalCurve, ThetaCurve, ThetaSurface,
    TimeDecaySurface, VannaVolgaSurface, VolatilitySensitivitySurface, VolatilitySkewCurve,
};
use optionstratlib_analytics::pnl::PnLCalculator;
use optionstratlib_analytics::risk::SPANMargin;
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Position, Positive, Side,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::OptionChain;
use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;

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

/// A short 95 put on a spot of 100, 30 days, sold for 1.5.
fn short_put() -> Position {
    let option = Options::new(
        OptionType::European,
        Side::Short,
        "BENCH".to_string(),
        pos_or_panic!(95.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        OptionStyle::Put,
        Positive::ZERO,
        None,
    );
    Position::new(
        option,
        pos_or_panic!(1.5),
        Utc::now(),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
        None,
        None,
    )
}

/// A 30-day chain on a spot of 100; `half_width` strikes either side.
fn chain(half_width: usize) -> OptionChain {
    let params = OptionChainBuildParams::new(
        "BENCH".to_string(),
        Some(pos_or_panic!(1000.0)),
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
    .with_greek_snapshots(true);
    OptionChain::build_chain(&params).expect("the fixture chain builds")
}

fn bench_probability(c: &mut Criterion) {
    let mut group = c.benchmark_group("analytics/probability");
    let spot = Positive::HUNDRED;
    let expiration = ExpirationDate::Days(pos_or_panic!(30.0));
    let volatility = || VolatilityAdjustment {
        base_volatility: pos_or_panic!(0.2),
        std_dev_adjustment: pos_or_panic!(0.05),
    };
    let trend = PriceTrend::new(dec!(0.05), dec!(0.9)).expect("a valid trend");

    bench_ok(&mut group, "single_point", || {
        calculate_single_point_probability(
            black_box(&spot),
            &pos_or_panic!(105.0),
            volatility(),
            None,
            &expiration,
            Some(dec!(0.05)),
        )
    });
    bench_ok(&mut group, "single_point_with_trend", || {
        calculate_single_point_probability(
            black_box(&spot),
            &pos_or_panic!(105.0),
            volatility(),
            Some(trend.clone()),
            &expiration,
            Some(dec!(0.05)),
        )
    });
    bench_ok(&mut group, "price_range", || {
        calculate_price_probability(
            black_box(&spot),
            &pos_or_panic!(95.0),
            &pos_or_panic!(105.0),
            volatility(),
            None,
            &expiration,
            Some(dec!(0.05)),
        )
    });
    group.finish();
}

fn bench_pnl_and_risk(c: &mut Criterion) {
    let mut group = c.benchmark_group("analytics/pnl_risk");
    let position = short_put();
    let spot = pos_or_panic!(98.0);
    let expiration = ExpirationDate::Days(pos_or_panic!(15.0));
    let iv = pos_or_panic!(0.22);

    bench_ok(&mut group, "position_calculate_pnl", || {
        black_box(&position).calculate_pnl(black_box(&spot), expiration, &iv)
    });
    bench_ok(&mut group, "position_calculate_pnl_at_expiration", || {
        black_box(&position).calculate_pnl_at_expiration(black_box(&spot))
    });
    bench_ok(&mut group, "options_calculate_pnl", || {
        black_box(&position.option).calculate_pnl(black_box(&spot), expiration, &iv)
    });
    let span = SPANMargin::new(dec!(0.1), dec!(0.05), dec!(0.1));
    bench_ok(&mut group, "span_margin", || {
        span.calculate_margin(black_box(&position))
    });

    // A 101-point mark-to-model sweep, the shape of a P&L chart.
    let spots: Vec<Positive> = (0..=100)
        .map(|i| pos_or_panic!(80.0 + 0.4 * f64::from(i)))
        .collect();
    group.throughput(Throughput::Elements(spots.len() as u64));
    bench_ok(&mut group, "position_calculate_pnl_sweep/101", || {
        spots
            .iter()
            .map(|s| position.calculate_pnl(black_box(s), expiration, &iv))
            .collect::<Result<Vec<_>, _>>()
    });
    group.finish();
}

fn bench_rnd(c: &mut Criterion) {
    let mut group = c.benchmark_group("analytics/rnd");
    group.sample_size(20);
    for half_width in [10, 25] {
        let chain = chain(half_width);
        let strikes = chain.options.len();
        for points in [50, 200] {
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: points,
                derivative_tolerance: pos_or_panic!(0.1),
            };
            bench_ok(
                &mut group,
                &format!("calculate_rnd/{strikes}_strikes_{points}_points"),
                || black_box(&chain).calculate_rnd(black_box(&params)),
            );
        }
        bench_ok(&mut group, &format!("calculate_skew/{strikes}"), || {
            black_box(&chain).calculate_skew()
        });
    }
    group.finish();
}

fn bench_projections(c: &mut Criterion) {
    let mut group = c.benchmark_group("analytics/projections");
    group.sample_size(20);
    let chain = chain(10);
    let strikes = chain.options.len();
    let vols: Vec<Positive> = [0.1, 0.15, 0.2, 0.25, 0.3]
        .into_iter()
        .map(|v| pos_or_panic!(v))
        .collect();
    let days: Vec<Positive> = [7.0, 14.0, 30.0, 60.0, 90.0]
        .into_iter()
        .map(|d| pos_or_panic!(d))
        .collect();

    bench_ok(&mut group, &format!("delta_curve/{strikes}"), || {
        black_box(&chain).delta_curve()
    });
    bench_ok(&mut group, &format!("gamma_curve/{strikes}"), || {
        black_box(&chain).gamma_curve()
    });
    bench_ok(&mut group, &format!("vanna_surface/{strikes}x5"), || {
        black_box(&chain).vanna_surface(vols.clone())
    });
    bench_ok(
        &mut group,
        &format!("theta_time_surface/{strikes}x5"),
        || black_box(&chain).theta_time_surface(days.clone()),
    );
    group.finish();
}

fn bench_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("analytics/metrics");
    group.sample_size(20);
    let chain = chain(10);
    let strikes = chain.options.len();
    let days: Vec<Positive> = [7.0, 14.0, 30.0, 60.0, 90.0]
        .into_iter()
        .map(|d| pos_or_panic!(d))
        .collect();
    let price_range = (pos_or_panic!(80.0), pos_or_panic!(120.0));
    let vol_range = (pos_or_panic!(0.1), pos_or_panic!(0.4));

    bench_ok(&mut group, &format!("iv_curve/{strikes}"), || {
        black_box(&chain).iv_curve()
    });
    bench_ok(&mut group, &format!("volatility_skew/{strikes}"), || {
        black_box(&chain).volatility_skew()
    });
    bench_ok(
        &mut group,
        &format!("premium_weighted_pcr/{strikes}"),
        || black_box(&chain).premium_weighted_pcr(),
    );
    bench_ok(
        &mut group,
        &format!("risk_reversal_curve/{strikes}"),
        || black_box(&chain).risk_reversal_curve(),
    );
    bench_ok(&mut group, &format!("dollar_gamma_curve/{strikes}"), || {
        black_box(&chain).dollar_gamma_curve(&OptionStyle::Call)
    });
    bench_ok(&mut group, &format!("delta_gamma_curve/{strikes}"), || {
        black_box(&chain).delta_gamma_curve()
    });
    bench_ok(&mut group, &format!("theta_curve/{strikes}"), || {
        ThetaCurve::theta_curve(black_box(&chain))
    });
    bench_ok(&mut group, &format!("price_shock_curve/{strikes}"), || {
        black_box(&chain).price_shock_curve(dec!(-0.1))
    });
    bench_ok(&mut group, &format!("iv_surface/{strikes}x5"), || {
        black_box(&chain).iv_surface(days.clone())
    });

    // The stress surfaces re-price across a grid: grid size is the knob.
    for steps in [10_usize, 20] {
        group.throughput(Throughput::Elements((steps * steps) as u64));
        bench_ok(
            &mut group,
            &format!("vanna_volga_surface/{steps}x{steps}"),
            || black_box(&chain).vanna_volga_surface(price_range, vol_range, steps, steps),
        );
        bench_ok(
            &mut group,
            &format!("volatility_sensitivity_surface/{steps}x{steps}"),
            || {
                black_box(&chain).volatility_sensitivity_surface(
                    price_range,
                    vol_range,
                    steps,
                    steps,
                )
            },
        );
        bench_ok(&mut group, &format!("time_decay_surface/{steps}x5"), || {
            black_box(&chain).time_decay_surface(price_range, days.clone(), steps)
        });
        bench_ok(&mut group, &format!("theta_surface/{steps}x5"), || {
            black_box(&chain).theta_surface(price_range, days.clone(), steps)
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_probability,
    bench_pnl_and_risk,
    bench_rnd,
    bench_projections,
    bench_metrics
);
criterion_main!(benches);
