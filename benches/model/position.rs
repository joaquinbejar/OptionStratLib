/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 7/1/25
******************************************************************************/

use chrono::Utc;
use criterion::{Criterion, criterion_group, criterion_main};
use optionstratlib::model::Position;
use optionstratlib::pnl::utils::PnLCalculator;
use optionstratlib::{ExpirationDate, OptionStyle, OptionType, Options, Side, pos_or_panic};
use positive::Positive;
use rust_decimal_macros::dec;
use std::hint::black_box;

fn create_test_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        Positive::HUNDRED,
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

fn create_test_position() -> Position {
    Position::new(
        create_test_option(),
        pos_or_panic!(5.0), // premium
        Utc::now(),         // date
        pos_or_panic!(0.5), // open_fee
        pos_or_panic!(0.5), // close_fee
        None,
        None,
    )
}

pub(crate) fn benchmark_costs_and_fees(c: &mut Criterion) {
    let mut group = c.benchmark_group("Position Costs and Fees");
    let position = create_test_position();

    group.bench_function("total_cost", |bencher| {
        bencher.iter(|| black_box(position.total_cost()))
    });

    group.bench_function("premium_received", |bencher| {
        bencher.iter(|| black_box(position.premium_received()))
    });

    group.bench_function("net_premium_received", |bencher| {
        bencher.iter(|| black_box(position.net_premium_received()))
    });

    group.bench_function("fees", |bencher| {
        bencher.iter(|| black_box(position.fees()))
    });

    group.bench_function("net_cost", |bencher| {
        bencher.iter(|| black_box(position.net_cost()))
    });

    group.finish();
}

pub(crate) fn benchmark_profit_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Position Profit Calculations");
    let position = create_test_position();
    let test_price = pos_or_panic!(110.0);

    group.bench_function("break_even", |bencher| {
        bencher.iter(|| black_box(position.break_even()))
    });

    group.bench_function("pnl_at_expiration", |bencher| {
        bencher.iter(|| black_box(position.pnl_at_expiration(&Some(&test_price))))
    });

    group.bench_function("unrealized_pnl", |bencher| {
        bencher.iter(|| black_box(position.unrealized_pnl(test_price)))
    });

    group.bench_function("calculate_pnl", |bencher| {
        bencher.iter(|| {
            black_box(position.calculate_pnl(
                &test_price,
                ExpirationDate::Days(pos_or_panic!(15.0)),
                &pos_or_panic!(0.25),
            ))
        })
    });

    group.finish();
}

pub(crate) fn benchmark_time_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Time Calculations");
    let position = create_test_position();

    group.bench_function("days_held", |bencher| {
        bencher.iter(|| black_box(position.days_held()))
    });

    group.bench_function("days_to_expiration", |bencher| {
        bencher.iter(|| black_box(position.days_to_expiration()))
    });

    group.finish();
}

pub(crate) fn benchmark_validations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Validation Operations");
    let position = create_test_position();

    group.bench_function("validate", |bencher| {
        bencher.iter(|| black_box(position.validate()))
    });

    group.bench_function("is_long", |bencher| {
        bencher.iter(|| black_box(position.is_long()))
    });

    group.bench_function("is_short", |bencher| {
        bencher.iter(|| black_box(position.is_short()))
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_costs_and_fees,
    benchmark_profit_calculations,
    benchmark_time_calculations,
    benchmark_validations
);
criterion_main!(benches);
