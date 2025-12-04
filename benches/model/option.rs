/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 7/1/25
******************************************************************************/

use criterion::{Criterion, criterion_group, criterion_main};
use optionstratlib::greeks::Greeks;
use optionstratlib::pnl::utils::PnLCalculator;
use optionstratlib::{ExpirationDate, OptionStyle, OptionType, Options, Side, pos};
use rust_decimal_macros::dec;
use std::hint::black_box;

fn create_test_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        pos!(100.0),
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.2),
        pos!(1.0),
        pos!(100.0),
        dec!(0.05),
        OptionStyle::Call,
        pos!(0.01),
        None,
    )
}

pub(crate) fn benchmark_pricing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Pricing Methods");
    let option = create_test_option();

    group.bench_function("black_scholes", |bencher| {
        bencher.iter(|| black_box(option.calculate_price_black_scholes()))
    });

    group.bench_function("binomial_50_steps", |bencher| {
        bencher.iter(|| black_box(option.calculate_price_binomial(50)))
    });

    group.bench_function("telegraph_50_steps", |bencher| {
        bencher.iter(|| black_box(option.calculate_price_telegraph(50)))
    });

    group.finish();
}

pub(crate) fn benchmark_greeks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Greeks Calculations");
    let option = create_test_option();

    group.bench_function("delta", |bencher| {
        bencher.iter(|| black_box(option.delta().unwrap()))
    });

    group.bench_function("gamma", |bencher| {
        bencher.iter(|| black_box(option.gamma().unwrap()))
    });

    group.bench_function("theta", |bencher| {
        bencher.iter(|| black_box(option.theta().unwrap()))
    });

    group.bench_function("vega", |bencher| {
        bencher.iter(|| black_box(option.vega().unwrap()))
    });

    group.bench_function("rho", |bencher| {
        bencher.iter(|| black_box(option.rho().unwrap()))
    });

    group.bench_function("vanna", |bencher| {
        bencher.iter(|| black_box(option.vanna().unwrap()))
    });    

    group.bench_function("vomma", |bencher| {
        bencher.iter(|| black_box(option.vomma().unwrap()))
    });     

    group.bench_function("veta", |bencher| {
        bencher.iter(|| black_box(option.veta().unwrap()))
    }); 

    group.bench_function("all_greeks", |bencher| {
        bencher.iter(|| black_box(option.greeks().unwrap()))
    });

    group.finish();
}

pub(crate) fn benchmark_valuations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Valuations");
    let option = create_test_option();

    group.bench_function("payoff", |bencher| {
        bencher.iter(|| black_box(option.payoff()))
    });

    group.bench_function("intrinsic_value", |bencher| {
        bencher.iter(|| black_box(option.intrinsic_value(pos!(110.0))))
    });

    group.bench_function("time_value", |bencher| {
        bencher.iter(|| black_box(option.time_value()))
    });

    group.bench_function("pnl_calculation", |bencher| {
        bencher.iter(|| {
            black_box(option.calculate_pnl(
                &pos!(110.0),
                ExpirationDate::Days(pos!(15.0)),
                &pos!(0.25),
            ))
        })
    });

    group.finish();
}

pub(crate) fn benchmark_binary_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("Binary Tree Operations");
    // Configure more time for samples
    group
        .sample_size(50)
        .warm_up_time(std::time::Duration::from_secs(5))
        .measurement_time(std::time::Duration::from_secs(10));

    let option = create_test_option();

    for steps in [10, 50, 100].iter() {
        group.bench_function(format!("binomial_tree_{steps}_steps"), |bencher| {
            bencher.iter(|| black_box(option.calculate_price_binomial_tree(*steps)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_pricing,
    benchmark_greeks,
    benchmark_valuations,
    benchmark_binary_tree
);
criterion_main!(benches);
