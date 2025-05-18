use criterion::Criterion;
use optionstratlib::chains::OptionData;
use optionstratlib::chains::utils::OptionDataPriceParams;
use optionstratlib::{ExpirationDate, Positive, pos};
use rust_decimal_macros::dec;
use std::hint::black_box;

pub fn benchmark_option_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("OptionData Operations");

    // Basic operations benchmarks
    benchmark_basic_operations(&mut group);

    // Price calculation benchmarks with different parameters
    benchmark_price_calculations(&mut group);

    // Complex operations benchmarks
    benchmark_complex_operations(&mut group);

    group.finish();
}

fn benchmark_basic_operations(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
) {
    // Benchmark creation with minimal data
    group.bench_function("create minimal option data", |b| {
        b.iter(|| {
            let option_data = OptionData::new(
                black_box(Positive::new(100.0).unwrap()),
                None,
                None,
                None,
                None,
                Some(Positive::new(0.2).unwrap()),
                None,
                None,
                None,
                None,
                None,
            );
            black_box(option_data)
        })
    });

    // Benchmark creation with full data
    group.bench_function("create full option data", |b| {
        b.iter(|| {
            let option_data = OptionData::new(
                black_box(Positive::new(100.0).unwrap()),
                Some(Positive::new(10.0).unwrap()),
                Some(Positive::new(11.0).unwrap()),
                Some(Positive::new(9.0).unwrap()),
                Some(Positive::new(10.0).unwrap()),
                Some(Positive::new(0.2).unwrap()),
                Some(dec!(0.5)),
                Some(dec!(0.5)),
                Some(dec!(0.5)),
                Some(Positive::new(1000.0).unwrap()),
                Some(100),
            );
            black_box(option_data)
        })
    });

    let option_data = create_test_option_data();
    group.bench_function("validate option data", |b| {
        b.iter(|| black_box(option_data.validate()))
    });
}

fn benchmark_price_calculations(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
) {
    let option_data = create_test_option_data();

    // Standard price calculation
    let standard_params = create_standard_price_params();
    group.bench_function("calculate standard prices", |b| {
        b.iter(|| {
            let mut data = option_data.clone();
            black_box(data.calculate_prices(&standard_params, false))
        })
    });

    // High volatility price calculation
    let high_vol_params = create_price_params_with_volatility(0.5);
    group.bench_function("calculate high volatility prices", |b| {
        b.iter(|| {
            let mut data = option_data.clone();
            black_box(data.calculate_prices(&high_vol_params, false))
        })
    });
}

fn benchmark_complex_operations(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
) {
    let option_data = create_test_option_data();
    let params = create_standard_price_params();

    // Combined operations benchmark
    group.bench_function("complete option processing", |b| {
        b.iter(|| {
            let mut data = option_data.clone();
            black_box(data.validate());
            let _ = black_box(data.calculate_prices(&params, false));
            black_box(data)
        })
    });
}

// Helper functions to create test data
fn create_test_option_data() -> OptionData {
    OptionData::new(
        Positive::new(100.0).unwrap(),
        Some(Positive::new(10.0).unwrap()),
        Some(Positive::new(11.0).unwrap()),
        Some(Positive::new(9.0).unwrap()),
        Some(Positive::new(10.0).unwrap()),
        Some(Positive::new(0.2).unwrap()),
        Some(dec!(0.5)),
        None,
        None,
        None,
        None,
    )
}

fn create_standard_price_params() -> OptionDataPriceParams {
    OptionDataPriceParams::new(
        Positive::new(100.0).unwrap(),
        ExpirationDate::Days(pos!(30.0)),
        Some(Positive::new(0.2).unwrap()),
        dec!(0.05),
        pos!(0.01),
        None,
    )
}

fn create_price_params_with_volatility(volatility: f64) -> OptionDataPriceParams {
    OptionDataPriceParams::new(
        Positive::new(100.0).unwrap(),
        ExpirationDate::Days(pos!(30.0)),
        Some(Positive::new(volatility).unwrap()),
        dec!(0.05),
        pos!(0.01),
        None,
    )
}
