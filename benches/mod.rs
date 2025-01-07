use criterion::{criterion_group, criterion_main};

mod chains;
mod model;

use chains::optiondata::benchmark_option_data;
use model::positive::{
    benchmark_arithmetic,
    benchmark_comparisons,
    benchmark_conversions,
    // asumiendo que el archivo está en chains/positive.rs
    benchmark_creation,
    benchmark_math_operations,
};

use model::option::{
    benchmark_binary_tree, benchmark_greeks, benchmark_pricing, benchmark_valuations,
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
    benchmark_binary_tree
);
criterion_main!(benches);
