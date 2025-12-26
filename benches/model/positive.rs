use criterion::{Criterion, criterion_group, criterion_main};
use positive::Positive;
use positive::pos_or_panic;
use rust_decimal_macros::dec;
use std::hint::black_box;

pub(crate) fn benchmark_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Creation Operations");

    group.bench_function("new from f64", |bencher| {
        let value = 100.0;
        bencher.iter(|| Positive::new(black_box(value)))
    });

    group.bench_function("new from decimal", |bencher| {
        let value = dec!(100.0);
        bencher.iter(|| Positive::new_decimal(black_box(value)))
    });

    group.bench_function("pos! macro", |bencher| {
        bencher.iter(|| pos_or_panic!(black_box(100.0)))
    });

    group.finish();
}

pub(crate) fn benchmark_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("Arithmetic Operations");

    let val1 = Positive::HUNDRED;
    let val2 = pos_or_panic!(50.0);
    let decimal = dec!(25.0);

    group.bench_function("addition", |bencher| {
        let x = val1;
        let y = val2;
        bencher.iter(|| {
            let result = x + y;
            black_box(result)
        })
    });

    group.bench_function("subtraction", |bencher| {
        let x = val1;
        let y = val2;
        bencher.iter(|| {
            if x > y {
                let result = x - y;
                black_box(result)
            } else {
                let result = y - x;
                black_box(result)
            }
        })
    });

    group.bench_function("multiplication", |bencher| {
        let x = val1;
        let y = val2;
        bencher.iter(|| {
            let result = x * y;
            black_box(result)
        })
    });

    group.bench_function("division", |bencher| {
        let x = val1;
        let y = val2;
        bencher.iter(|| {
            let result = x / y;
            black_box(result)
        })
    });

    group.bench_function("decimal operations", |bencher| {
        let x = val1;
        let y = val2;
        let d = decimal;
        bencher.iter(|| {
            let result = x * d + y / d;
            black_box(result)
        })
    });

    group.finish();
}

pub(crate) fn benchmark_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Conversion Operations");

    let value = pos_or_panic!(123.456);

    group.bench_function("to_f64", |bencher| {
        let x = value;
        bencher.iter(|| black_box(x.to_f64()))
    });

    group.bench_function("to_dec", |bencher| {
        let x = value;
        bencher.iter(|| black_box(x.to_dec()))
    });

    group.bench_function("to_i64", |bencher| {
        let x = value;
        bencher.iter(|| black_box(x.to_i64()))
    });

    group.finish();
}

pub(crate) fn benchmark_math_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mathematical Operations");

    let value = pos_or_panic!(2.5);

    group.bench_function("powi", |bencher| {
        let x = value;
        bencher.iter(|| black_box(x.powi(3)))
    });

    group.bench_function("sqrt", |bencher| {
        let x = value;
        bencher.iter(|| black_box(x.sqrt()))
    });

    group.bench_function("ln", |bencher| {
        let x = value;
        bencher.iter(|| black_box(x.ln()))
    });

    group.bench_function("exp", |bencher| {
        let x = value;
        bencher.iter(|| black_box(x.exp()))
    });

    group.bench_function("round", |bencher| {
        let x = value;
        bencher.iter(|| black_box(x.round()))
    });

    group.bench_function("floor", |bencher| {
        let x = value;
        bencher.iter(|| black_box(x.floor()))
    });

    group.bench_function("round_to", |bencher| {
        let x = value;
        bencher.iter(|| black_box(x.round_to(2)))
    });

    group.finish();
}

pub(crate) fn benchmark_comparisons(c: &mut Criterion) {
    let mut group = c.benchmark_group("Comparison Operations");

    let val1 = Positive::HUNDRED;
    let val2 = pos_or_panic!(50.0);

    group.bench_function("max", |bencher| {
        let x = val1;
        let y = val2;
        bencher.iter(|| black_box(x.max(y)))
    });

    group.bench_function("min", |bencher| {
        let x = val1;
        let y = val2;
        bencher.iter(|| black_box(x.min(y)))
    });

    group.bench_function("partial_eq", |bencher| {
        let x = val1;
        let y = val2;
        bencher.iter(|| black_box(x == y))
    });

    group.bench_function("partial_ord", |bencher| {
        let x = val1;
        let y = val2;
        bencher.iter(|| black_box(x > y))
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_creation,
    benchmark_arithmetic,
    benchmark_conversions,
    benchmark_math_operations,
    benchmark_comparisons
);
criterion_main!(benches);
