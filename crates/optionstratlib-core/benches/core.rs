//! Benchmarks of the `optionstratlib-core` public paths (#789).
//!
//! Covers the checked `Decimal` helpers every higher layer calls in its inner
//! loops, the `Positive` conversions, payoffs and position P&L, and the
//! construction of the option and position value objects.
//!
//! Run with `cargo bench -p optionstratlib-core --bench core`.

use chrono::Utc;
use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_core::model::decimal::{
    DecimalStats, d_add, d_div, d_exp, d_ln, d_mul, d_powd, d_sqrt, d_sum, decimal_to_f64,
    f64_to_decimal,
};
use optionstratlib_core::model::payoff::{Payoff, PayoffInfo};
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Position, Positive, Side,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::calculate_log_returns;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;

/// Series lengths for the helpers that fold a slice: a month, a trading
/// year, and four years of daily closes.
const SERIES_LENGTHS: [usize; 3] = [21, 252, 1_008];

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

/// A European call 5% out of the money, 30 days out.
fn call_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "BENCH".to_string(),
        pos_or_panic!(105.0),
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

fn call_position() -> Position {
    Position::new(
        call_option(),
        pos_or_panic!(2.5),
        Utc::now(),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
        None,
        None,
    )
}

/// A deterministic price path around 100 with a long mantissa on every
/// point, so the arithmetic is not measured on short, exact operands.
fn price_path(len: usize) -> Vec<Positive> {
    (0..len)
        .map(|i| {
            let wave = ((i % 17) as f64 - 8.0) / 7.0;
            pos_or_panic!(100.0 + wave + (i as f64) / 97.0)
        })
        .collect()
}

fn decimal_series(len: usize) -> Vec<Decimal> {
    (0..len)
        .map(|i| Decimal::from(i % 89 + 11) / dec!(970))
        .collect()
}

fn bench_decimal_helpers(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/decimal");
    let a = dec!(101.123456789);
    let b = dec!(0.987654321);

    bench_ok(&mut group, "d_add", || {
        d_add(black_box(a), black_box(b), "bench")
    });
    bench_ok(&mut group, "d_mul", || {
        d_mul(black_box(a), black_box(b), "bench")
    });
    bench_ok(&mut group, "d_div", || {
        d_div(black_box(a), black_box(b), "bench")
    });
    bench_ok(&mut group, "d_exp", || {
        d_exp(black_box(dec!(-0.0041)), "bench")
    });
    bench_ok(&mut group, "d_ln", || d_ln(black_box(a), "bench"));
    bench_ok(&mut group, "d_sqrt", || d_sqrt(black_box(a), "bench"));
    bench_ok(&mut group, "d_powd", || {
        d_powd(black_box(a), black_box(dec!(0.5)), "bench")
    });
    bench_ok(&mut group, "decimal_to_f64", || {
        decimal_to_f64(black_box(a))
    });
    bench_ok(&mut group, "f64_to_decimal", || {
        f64_to_decimal(black_box(101.123_456_789))
    });

    for len in SERIES_LENGTHS {
        let values = decimal_series(len);
        group.throughput(Throughput::Elements(len as u64));
        bench_ok(&mut group, &format!("d_sum/{len}"), || {
            d_sum(black_box(&values), "bench")
        });
        bench_ok(&mut group, &format!("mean/{len}"), || {
            black_box(&values).mean()
        });
        bench_ok(&mut group, &format!("std_dev/{len}"), || {
            black_box(&values).std_dev()
        });
    }
    group.finish();
}

fn bench_positive(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/positive");
    let a = pos_or_panic!(101.25);
    let b = pos_or_panic!(0.75);

    bench_ok(&mut group, "new_from_f64", || {
        Positive::new(black_box(101.25))
    });
    group.bench_function("add", |bench| bench.iter(|| black_box(a) + black_box(b)));
    group.bench_function("mul", |bench| bench.iter(|| black_box(a) * black_box(b)));
    bench_ok(&mut group, "checked_div", || {
        black_box(a).checked_div(&black_box(b))
    });
    group.bench_function("to_f64", |bench| bench.iter(|| black_box(a).to_f64()));
    group.bench_function("to_dec", |bench| bench.iter(|| black_box(a).to_dec()));

    for len in SERIES_LENGTHS {
        let prices = price_path(len);
        group.throughput(Throughput::Elements(len as u64));
        bench_ok(&mut group, &format!("calculate_log_returns/{len}"), || {
            calculate_log_returns(black_box(&prices))
        });
    }
    group.finish();
}

fn bench_payoff(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/payoff");
    let option = call_option();
    let position = call_position();
    let info = PayoffInfo {
        spot: pos_or_panic!(110.0),
        strike: pos_or_panic!(105.0),
        style: OptionStyle::Call,
        side: Side::Long,
        spot_prices: None,
        spot_min: None,
        spot_max: None,
        exotic_params: None,
    };
    let spot = pos_or_panic!(110.0);

    bench_ok(&mut group, "option_type_payoff", || {
        OptionType::European.payoff(black_box(&info))
    });
    bench_ok(&mut group, "options_payoff", || black_box(&option).payoff());
    bench_ok(&mut group, "options_payoff_at_price", || {
        black_box(&option).payoff_at_price(black_box(&spot))
    });
    bench_ok(&mut group, "options_intrinsic_value", || {
        black_box(&option).intrinsic_value(black_box(spot))
    });
    bench_ok(&mut group, "position_pnl_at_expiration", || {
        black_box(&position).pnl_at_expiration(&Some(black_box(&spot)))
    });
    bench_ok(&mut group, "position_unrealized_pnl", || {
        black_box(&position).unrealized_pnl(black_box(spot))
    });
    group.bench_function("position_break_even", |bench| {
        bench.iter(|| black_box(&position).break_even())
    });
    bench_ok(&mut group, "position_total_cost", || {
        black_box(&position).total_cost()
    });

    // A payoff sweep over 201 spot prices: the shape every P&L chart and
    // break-even search evaluates.
    let spots: Vec<Positive> = (0..=200)
        .map(|i| pos_or_panic!(50.0 + f64::from(i) * 0.5))
        .collect();
    group.throughput(Throughput::Elements(spots.len() as u64));
    bench_ok(&mut group, "position_pnl_at_expiration_sweep/201", || {
        spots
            .iter()
            .map(|price| position.pnl_at_expiration(&Some(black_box(price))))
            .collect::<Result<Vec<_>, _>>()
    });
    group.finish();
}

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/construction");
    let option = call_option();

    group.bench_function("options_new", |bench| {
        bench.iter(|| black_box(call_option()))
    });
    group.bench_function("position_new", |bench| {
        bench.iter(|| black_box(call_position()))
    });
    group.bench_function("options_clone", |bench| {
        bench.iter(|| black_box(&option).clone())
    });
    bench_ok(&mut group, "options_time_to_expiration", || {
        black_box(&option).time_to_expiration()
    });
    let expiration = ExpirationDate::Days(pos_or_panic!(30.0));
    bench_ok(&mut group, "expiration_days_get_years", || {
        black_box(&expiration).get_years()
    });
    let json = serde_json::to_string(&option).expect("an option serializes to JSON");
    bench_ok(&mut group, "options_to_json", || {
        serde_json::to_string(black_box(&option))
    });
    bench_ok(&mut group, "options_from_json", || {
        serde_json::from_str::<Options>(black_box(&json))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_decimal_helpers,
    bench_positive,
    bench_payoff,
    bench_construction
);
criterion_main!(benches);
