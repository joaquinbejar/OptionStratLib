//! Benchmarks of the `optionstratlib-math` public paths (#789).
//!
//! Curve and surface construction, every interpolation method at scaled
//! sizes, the metrics extractors, and the geometric transformations. The
//! N-ary merge is measured by the facade's `geometrics/merge` group.
//!
//! Run with `cargo bench -p optionstratlib-math --bench math`.

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_math::curves::{Curve, CurveSpline, Point2D};
use optionstratlib_math::error::{CurveError, SurfaceError};
use optionstratlib_math::geometrics::{
    Arithmetic, ConstructionMethod, ConstructionParams, GeometricObject, GeometricTransformations,
    Interpolate, InterpolationType, MergeOperation, MetricsExtractor,
};
use optionstratlib_math::surfaces::{Point3D, Surface};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::hint::black_box;

/// Point counts per curve: a weekly chain, a monthly chain, a full-term
/// chain, and a dense resampling grid.
const CURVE_SIZES: [usize; 4] = [32, 128, 512, 2_048];

/// Side lengths of the square surface grids (64, 256 and 1 024 points).
const SURFACE_SIDES: [usize; 3] = [8, 16, 32];

/// Every interpolation method, with the label used in the benchmark id.
const METHODS: [(&str, InterpolationType); 4] = [
    ("linear", InterpolationType::Linear),
    ("bilinear", InterpolationType::Bilinear),
    ("cubic", InterpolationType::Cubic),
    ("spline", InterpolationType::Spline),
];

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

/// A smooth smile-shaped ordinate with a long mantissa, so the arithmetic is
/// not measured on short, exact operands.
fn smile(x: Decimal) -> Decimal {
    let centred = (x - dec!(5)) / dec!(7);
    dec!(0.2) + centred * centred / dec!(3)
}

/// The `i`-th of `n` evenly spaced abscissas over `[0, 10]`, as a caller
/// writes them: `step * i` with `step = 10 / (n - 1)`.
///
/// For most `n` the last one is off 10 in the 28th place, which the merge
/// grid handles since #795, so the merges below run on the natural grid.
fn abscissa(i: usize, n: usize) -> Decimal {
    dec!(10) / Decimal::from(n - 1) * Decimal::from(i)
}

/// `points` samples of the smile spanning `[0, 10]`.
fn curve_points(points: usize) -> Vec<Point2D> {
    (0..points)
        .map(|i| {
            let x = abscissa(i, points);
            Point2D::new(x, smile(x))
        })
        .collect()
}

fn curve_fixture(points: usize) -> Curve {
    Curve::from_vector(curve_points(points))
}

/// A `side * side` surface over `[0, 10] x [0, 10]`.
fn surface_points(side: usize) -> Vec<Point3D> {
    (0..side)
        .flat_map(|i| {
            (0..side).map(move |j| {
                let x = abscissa(i, side);
                let y = abscissa(j, side);
                Point3D::new(x, y, smile(x) + smile(y))
            })
        })
        .collect()
}

fn surface_fixture(side: usize) -> Surface {
    Surface::from_vector(surface_points(side))
}

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("math/construction");
    for size in CURVE_SIZES {
        let points = curve_points(size);
        let set: BTreeSet<Point2D> = points.iter().cloned().collect();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(format!("curve_new/{size}"), |bench| {
            bench.iter(|| Curve::new(black_box(set.clone())))
        });
        group.bench_function(format!("curve_from_vector/{size}"), |bench| {
            bench.iter(|| Curve::from_vector(black_box(points.clone())))
        });
        bench_ok(&mut group, &format!("curve_parametric/{size}"), || {
            Curve::construct(ConstructionMethod::Parametric {
                f: Box::new(|t: Decimal| -> Result<Point2D, CurveError> {
                    Ok(Point2D::new(t, smile(t)))
                }),
                params: ConstructionParams::D2 {
                    t_start: Decimal::ZERO,
                    t_end: dec!(10),
                    steps: size,
                },
            })
        });
    }
    for side in SURFACE_SIDES {
        let points = surface_points(side);
        group.throughput(Throughput::Elements((side * side) as u64));
        group.bench_function(format!("surface_from_vector/{side}x{side}"), |bench| {
            bench.iter(|| Surface::from_vector(black_box(points.clone())))
        });
        bench_ok(
            &mut group,
            &format!("surface_parametric/{side}x{side}"),
            || {
                Surface::construct(ConstructionMethod::Parametric {
                    f: Box::new(|p: Point2D| -> Result<Point3D, SurfaceError> {
                        Ok(Point3D::new(p.x, p.y, smile(p.x) + smile(p.y)))
                    }),
                    params: ConstructionParams::D3 {
                        x_start: Decimal::ZERO,
                        x_end: dec!(10),
                        y_start: Decimal::ZERO,
                        y_end: dec!(10),
                        x_steps: side,
                        y_steps: side,
                    },
                })
            },
        );
    }
    group.finish();
}

fn bench_curve_interpolation(c: &mut Criterion) {
    let mut group = c.benchmark_group("math/curve_interpolation");
    // Between two nodes, so no method can short-circuit on an exact match.
    let x = dec!(3.3333);
    for size in CURVE_SIZES {
        let curve = curve_fixture(size);
        for (label, method) in METHODS {
            bench_ok(&mut group, &format!("{label}/{size}"), || {
                black_box(&curve).interpolate(black_box(x), method)
            });
        }
        // One read of a spline solved beforehand (#858).
        let spline = CurveSpline::new(&curve);
        bench_ok(&mut group, &format!("spline_prepared/{size}"), || {
            black_box(&spline).interpolate(black_box(x))
        });
    }

    // A sweep of 100 reads on one curve: the access pattern of a resampling
    // (merge, chart, smile lookup across a chain).
    let curve = curve_fixture(128);
    let xs: Vec<Decimal> = (0..100)
        .map(|i| dec!(0.05) + Decimal::from(i) * dec!(0.099))
        .collect();
    group.throughput(Throughput::Elements(xs.len() as u64));
    for (label, method) in METHODS {
        bench_ok(&mut group, &format!("{label}_sweep_100/128"), || {
            xs.iter()
                .map(|x| curve.interpolate(black_box(*x), method))
                .collect::<Result<Vec<_>, _>>()
        });
    }
    // The same sweep through a spline solved once per sweep (#858): the
    // solve is inside the timed closure.
    bench_ok(&mut group, "spline_prepared_sweep_100/128", || {
        let spline = CurveSpline::new(black_box(&curve));
        xs.iter()
            .map(|x| spline.interpolate(black_box(*x)))
            .collect::<Result<Vec<_>, _>>()
    });
    group.finish();
}

fn bench_surface_interpolation(c: &mut Criterion) {
    let mut group = c.benchmark_group("math/surface_interpolation");
    let at = Point2D::new(dec!(3.3333), dec!(6.6667));
    for side in SURFACE_SIDES {
        let surface = surface_fixture(side);
        for (label, method) in METHODS {
            bench_ok(&mut group, &format!("{label}/{side}x{side}"), || {
                black_box(&surface).interpolate(black_box(at), method)
            });
        }
    }
    group.finish();
}

fn bench_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("math/metrics");
    for size in [128, 512] {
        let curve = curve_fixture(size);
        group.throughput(Throughput::Elements(size as u64));
        bench_ok(&mut group, &format!("curve_metrics/{size}"), || {
            black_box(&curve).compute_curve_metrics()
        });
    }
    for side in [8, 16] {
        let surface = surface_fixture(side);
        group.throughput(Throughput::Elements((side * side) as u64));
        bench_ok(
            &mut group,
            &format!("surface_metrics/{side}x{side}"),
            || black_box(&surface).compute_surface_metrics(),
        );
    }
    group.finish();
}

fn bench_transformations(c: &mut Criterion) {
    let mut group = c.benchmark_group("math/transformations");
    let size = 512;
    let curve = curve_fixture(size);
    let other = Curve::from_vector(
        curve_points(size)
            .into_iter()
            .map(|p| Point2D::new(p.x, dec!(0.4) - p.y / dec!(2)))
            .collect(),
    );
    let shift = dec!(0.25);
    let factor = dec!(1.5);
    bench_ok(&mut group, "curve_translate/512", || {
        black_box(&curve).translate(vec![&shift, &shift])
    });
    bench_ok(&mut group, "curve_scale/512", || {
        black_box(&curve).scale(vec![&factor, &factor])
    });
    bench_ok(&mut group, "curve_extrema/512", || {
        black_box(&curve).extrema()
    });
    bench_ok(&mut group, "curve_measure_under/512", || {
        black_box(&curve).measure_under(&Decimal::ZERO)
    });
    let at = Point2D::new(dec!(3.3333), Decimal::ZERO);
    bench_ok(&mut group, "curve_derivative_at/512", || {
        black_box(&curve).derivative_at(black_box(&at))
    });
    bench_ok(&mut group, "curve_intersect_with/512", || {
        black_box(&curve).intersect_with(black_box(&other))
    });
    bench_ok(&mut group, "curve_merge_with_add/512", || {
        black_box(&curve).merge_with(black_box(&other), MergeOperation::Add)
    });

    let surface = surface_fixture(16);
    bench_ok(&mut group, "surface_translate/16x16", || {
        black_box(&surface).translate(vec![&shift, &shift, &shift])
    });
    bench_ok(&mut group, "surface_extrema/16x16", || {
        black_box(&surface).extrema()
    });
    bench_ok(&mut group, "surface_merge_with_add/16x16", || {
        black_box(&surface).merge_with(black_box(&surface), MergeOperation::Add)
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_construction,
    bench_curve_interpolation,
    bench_surface_interpolation,
    bench_metrics,
    bench_transformations
);
criterion_main!(benches);
