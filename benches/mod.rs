use criterion::{criterion_group, criterion_main};

mod chains;
mod model;

use chains::optiondata::benchmark_option_data;
use model::positive::{
    benchmark_arithmetic,
    benchmark_comparisons,
    benchmark_conversions,
    benchmark_creation,
    benchmark_math_operations,
};

use model::option::{
    benchmark_binary_tree, benchmark_greeks, benchmark_pricing, benchmark_valuations,
};

use model::position::{
    benchmark_costs_and_fees,
    benchmark_graphics,
    benchmark_profit_calculations,
    benchmark_time_calculations,
    benchmark_validations,
};

criterion_group!(
    benches,
    benchmark_option_data,
    benchmark_creation,
    benchmark_arithmetic,
    benchmark_conversions,
    benchmark_math_operations,
    benchmark_comparisons,
    benchmark_pricing,
    benchmark_greeks,
    benchmark_valuations,
    benchmark_binary_tree,
    benchmark_costs_and_fees,
    benchmark_profit_calculations,
    benchmark_time_calculations,
    benchmark_graphics,
    benchmark_validations
);
criterion_main!(benches);
