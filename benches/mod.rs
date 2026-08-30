use criterion::{criterion_group, criterion_main};

mod chains;
mod geometrics;
mod model;

use chains::generators::benchmark_chain_generators;
use chains::optiondata::benchmark_option_data;
use geometrics::merge::{
    benchmark_curve_merge_multiply, benchmark_decimal_product_fold,
    benchmark_surface_merge_multiply,
};
use model::positive::{
    benchmark_arithmetic, benchmark_comparisons, benchmark_conversions, benchmark_creation,
    benchmark_math_operations,
};
use model::strategy::benchmark_strategies;

use model::option::{
    benchmark_binary_tree, benchmark_greeks, benchmark_maturities, benchmark_pricing,
    benchmark_valuations,
};

use model::position::{
    benchmark_costs_and_fees, benchmark_profit_calculations, benchmark_time_calculations,
    benchmark_validations,
};

criterion_group!(
    benches,
    benchmark_chain_generators,
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
    benchmark_maturities,
    benchmark_costs_and_fees,
    benchmark_profit_calculations,
    benchmark_time_calculations,
    benchmark_validations,
    benchmark_strategies,
    benchmark_curve_merge_multiply,
    benchmark_surface_merge_multiply,
    benchmark_decimal_product_fold
);
criterion_main!(benches);
