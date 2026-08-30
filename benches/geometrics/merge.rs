use criterion::{BenchmarkId, Criterion, Throughput};
use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::geometrics::{Arithmetic, GeometricObject, MergeOperation};
use optionstratlib::surfaces::{Point3D, Surface};
use rayon::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::hint::black_box;

/// Operand counts the merge is measured at.
///
/// Two operands is the `merge_with` case and cannot expose an ordering
/// difference; three is the smallest count that can. Five and ten stand for
/// the multi-leg strategy surfaces this crate actually multiplies together.
const OPERAND_COUNTS: [usize; 4] = [2, 3, 5, 10];

/// Point counts per curve, in the range an option chain produces: a few tens
/// of strikes for a weekly, a few hundred for a full-term underlying.
const CURVE_SIZES: [usize; 3] = [32, 128, 512];

/// Side length of the square grid backing each surface fixture, so the
/// surfaces carry 64 and 256 points respectively.
///
/// `Surface::merge` resamples onto a fixed 51 x 51 grid whatever the operands
/// hold, so a denser fixture buys more interpolation cost per call and not one
/// extra invocation of the fold under test. Two sizes are enough to show how
/// far interpolation dilutes it.
const SURFACE_SIDES: [usize; 2] = [8, 16];

/// Builds a `Decimal` with a long mantissa from two small integers.
///
/// The quotient by 970 is a repeating decimal, so every fixture ordinate
/// fills the 28-place scale. That is what makes a product of three or more of
/// them round, which is the whole subject of the measurement: on short
/// mantissas the fold would be exact and the ordering of the reduction would
/// not matter.
fn long_mantissa(n: u64, offset: u64) -> Decimal {
    let numerator = Decimal::from(n.wrapping_mul(7).wrapping_add(offset) % 89 + 11);
    // Kept near 1 so a ten-operand product neither overflows nor collapses
    // to zero, which would make the benchmark measure a degenerate case.
    dec!(0.5) + numerator / dec!(970)
}

/// Builds a curve of `points` samples spanning `[0, 10]`.
///
/// `index` shifts the ordinates so the operands of a merge are distinct;
/// merging a curve with itself would let the compiler share interpolation
/// work that a real merge cannot.
fn curve_fixture(points: usize, index: u64) -> Curve {
    let step = dec!(10) / Decimal::from(points.max(2) - 1);
    let samples: Vec<Point2D> = (0..points)
        .map(|i| {
            let x = step * Decimal::from(i);
            Point2D::new(x, long_mantissa(i as u64, index * 31))
        })
        .collect();
    Curve::from_vector(samples)
}

/// Builds a `side * side` surface spanning `[0, 10] x [0, 10]`.
fn surface_fixture(side: usize, index: u64) -> Surface {
    let step = dec!(10) / Decimal::from(side.max(2) - 1);
    let samples: Vec<Point3D> = (0..side)
        .flat_map(|i| {
            (0..side).map(move |j| {
                let x = step * Decimal::from(i);
                let y = step * Decimal::from(j);
                Point3D::new(x, y, long_mantissa((i * side + j) as u64, index * 31))
            })
        })
        .collect();
    Surface::from_vector(samples)
}

/// End-to-end cost of `Curve::merge` under `Multiply`, per operand count and
/// curve size.
///
/// This is the number a caller sees. It includes the cubic interpolation of
/// every operand onto the shared grid, which dominates; the fold under test
/// is a small share of it, so the isolated fold is measured separately by
/// [`benchmark_decimal_product_fold`].
pub fn benchmark_curve_merge_multiply(c: &mut Criterion) {
    let mut group = c.benchmark_group("curve_merge_multiply");

    for &size in CURVE_SIZES.iter() {
        for &operands in OPERAND_COUNTS.iter() {
            let curves: Vec<Curve> = (0..operands)
                .map(|i| curve_fixture(size, i as u64))
                .collect();
            let refs: Vec<&Curve> = curves.iter().collect();

            group.throughput(Throughput::Elements(operands as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("size_{size}"), operands),
                &refs,
                |b, refs| {
                    b.iter(|| {
                        let merged = Curve::merge(black_box(refs), MergeOperation::Multiply);
                        black_box(merged.is_ok())
                    })
                },
            );
        }
    }

    group.finish();
}

/// End-to-end cost of `Surface::merge` under `Multiply`.
///
/// `Surface::merge` samples a 51 x 51 grid, so it runs the fold 2601 times
/// per call against `Curve::merge`'s 101: if the reduction strategy were
/// ever going to show up end to end, it would show up here first.
pub fn benchmark_surface_merge_multiply(c: &mut Criterion) {
    let mut group = c.benchmark_group("surface_merge_multiply");
    group.sample_size(10);

    for &side in SURFACE_SIDES.iter() {
        for &operands in OPERAND_COUNTS.iter() {
            let surfaces: Vec<Surface> = (0..operands)
                .map(|i| surface_fixture(side, i as u64))
                .collect();
            let refs: Vec<&Surface> = surfaces.iter().collect();

            group.throughput(Throughput::Elements(operands as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("side_{side}"), operands),
                &refs,
                |b, refs| {
                    b.iter(|| {
                        let merged = Surface::merge(black_box(refs), MergeOperation::Multiply);
                        black_box(merged.is_ok())
                    })
                },
            );
        }
    }

    group.finish();
}

/// Isolated cost of the two candidate reductions, on exactly the input the
/// merge hands them: one `Vec<Decimal>` holding one interpolated ordinate per
/// operand.
///
/// The end-to-end merge benchmarks bury this behind interpolation. Here the
/// rayon `reduce` and the sequential `try_fold` are measured against each
/// other with nothing else in the frame, so the price of determinism is
/// visible rather than inferred.
pub fn benchmark_decimal_product_fold(c: &mut Criterion) {
    let mut group = c.benchmark_group("decimal_product_fold");

    for &operands in OPERAND_COUNTS.iter() {
        let values: Vec<Decimal> = (0..operands)
            .map(|i| long_mantissa(i as u64, i as u64 * 31))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("parallel_reduce", operands),
            &values,
            |b, values| {
                b.iter(|| {
                    let product: Option<Decimal> =
                        black_box(values).par_iter().copied().map(Some).reduce(
                            || Some(Decimal::ONE),
                            |a, b| match (a, b) {
                                (Some(a), Some(b)) => a.checked_mul(b),
                                _ => None,
                            },
                        );
                    black_box(product)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sequential_fold", operands),
            &values,
            |b, values| {
                b.iter(|| {
                    let product = black_box(values)
                        .iter()
                        .copied()
                        .try_fold(Decimal::ONE, |acc, v| acc.checked_mul(v));
                    black_box(product)
                })
            },
        );
    }

    group.finish();
}
