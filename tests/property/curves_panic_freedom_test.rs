//! Property-based tests for panic freedom across curves, surfaces and the
//! shared geometry helpers.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it. Every failure must
//! therefore come back as a `Result`, including for inputs that are extreme
//! but structurally valid.
//!
//! Geometry fails in two directions at once, so both are driven here. The
//! numeric axis is the usual `Decimal` domain: coordinates at the smallest
//! representable scale, at `Decimal::MAX` and `Decimal::MIN`, and the
//! ordinary values in between. The structural axis is what makes a point set
//! degenerate rather than merely large: an empty set, a single point, two
//! points sharing an abscissa (every interpolator divides by `x2 - x1`),
//! collinear neighbours (barycentric weights divide by a zero triangle area),
//! a surface that is one row or one column, and a grid with a repeated
//! `(x, y)`. The assertion is deliberately weak: whatever comes back, it must
//! come back.

use optionstratlib::curves::{Curve, Point2D, StatisticalCurve};
use optionstratlib::geometrics::{
    Arithmetic, AxisOperations, BasicMetrics, ConstructionMethod, ConstructionParams,
    GeometricObject, GeometricTransformations, Interpolate, InterpolationType,
    MergeAxisInterpolate, MergeOperation, MetricsExtractor, RangeMetrics, ShapeMetrics,
    TrendMetrics,
};
use optionstratlib::surfaces::{Point3D, Surface};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeSet;

/// The smallest representable `Decimal`. Squaring it underflows to zero,
/// which is what collapses a regression denominator on a curve whose
/// abscissas are all tiny.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

/// Coordinates that reach the arithmetic panics: the edges of the `Decimal`
/// range, the smallest representable scale, and the ordinary values between.
fn extreme_coordinate() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(TINY),
        Just(-TINY),
        Just(dec!(0.5)),
        Just(dec!(1)),
        Just(dec!(100)),
        Just(dec!(-100)),
        Just(dec!(1000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

fn interpolation_type() -> impl Strategy<Value = InterpolationType> {
    prop_oneof![
        Just(InterpolationType::Linear),
        Just(InterpolationType::Bilinear),
        Just(InterpolationType::Cubic),
        Just(InterpolationType::Spline),
    ]
}

fn merge_operation() -> impl Strategy<Value = MergeOperation> {
    prop_oneof![
        Just(MergeOperation::Add),
        Just(MergeOperation::Subtract),
        Just(MergeOperation::Multiply),
        Just(MergeOperation::Divide),
        Just(MergeOperation::Max),
        Just(MergeOperation::Min),
    ]
}

fn curve_from(points: &[(Decimal, Decimal)]) -> Curve {
    Curve::new(
        points
            .iter()
            .map(|&(x, y)| Point2D::new(x, y))
            .collect::<BTreeSet<_>>(),
    )
}

fn surface_from(points: &[(Decimal, Decimal, Decimal)]) -> Surface {
    Surface::new(
        points
            .iter()
            .map(|&(x, y, z)| Point3D::new(x, y, z))
            .collect::<BTreeSet<_>>(),
    )
}

/// The structural degeneracies a curve can carry, each paired with the
/// numeric extreme the strategy draws.
fn degenerate_curve() -> impl Strategy<Value = Curve> {
    (extreme_coordinate(), extreme_coordinate(), 0usize..8).prop_map(|(a, b, shape)| {
        let points: Vec<(Decimal, Decimal)> = match shape {
            // Empty: nothing to interpolate, index or average over.
            0 => vec![],
            // A single point: `len - 1` underflows, the variance has no spread.
            1 => vec![(a, b)],
            // Two points: below the window every algorithm but linear needs.
            2 => vec![(a, b), (dec!(1), dec!(1))],
            // `Point2D` orders on (x, y) but compares equal on x alone, so a
            // `BTreeSet` legitimately holds a repeated abscissa.
            3 => vec![(a, b), (a, dec!(3)), (dec!(2), dec!(4)), (dec!(3), dec!(5))],
            // Four points, the minimum for cubic and bilinear.
            4 => vec![(dec!(0), a), (dec!(1), b), (dec!(2), a), (dec!(3), b)],
            // Flat: zero variance, zero standard deviation.
            5 => vec![(dec!(0), a), (dec!(1), a), (dec!(2), a), (dec!(3), a)],
            // Abscissas at the smallest scale: their squares underflow, which
            // collapses the ordinary-least-squares denominator.
            6 => vec![
                (Decimal::ZERO, a),
                (TINY, b),
                (TINY + TINY, a),
                (TINY + TINY + TINY, b),
            ],
            // Coordinates spanning the whole `Decimal` range in both axes.
            _ => vec![
                (Decimal::MIN, a),
                (Decimal::ZERO, b),
                (dec!(1), a),
                (Decimal::MAX, b),
            ],
        };
        curve_from(&points)
    })
}

/// The structural degeneracies a surface can carry.
fn degenerate_surface() -> impl Strategy<Value = Surface> {
    (extreme_coordinate(), extreme_coordinate(), 0usize..8).prop_map(|(a, b, shape)| {
        let points: Vec<(Decimal, Decimal, Decimal)> = match shape {
            // Empty: every mean divides by zero.
            0 => vec![],
            // A single point: no triangle, no spread.
            1 => vec![(a, b, a)],
            // Two points: below the three a barycentric weight needs.
            2 => vec![(dec!(0), dec!(0), a), (dec!(1), dec!(1), b)],
            // One row: every y is the same.
            3 => (0..4i64).map(|i| (Decimal::from(i), dec!(0), a)).collect(),
            // One column: every x is the same, so the trend denominator is
            // zero.
            4 => (0..4i64).map(|j| (dec!(0), Decimal::from(j), b)).collect(),
            // Collinear: the three nearest points span no triangle area.
            5 => (0..4i64)
                .map(|i| (Decimal::from(i), Decimal::from(i), a))
                .collect(),
            // A repeated `(x, y)` with different z values.
            6 => vec![
                (dec!(1), dec!(1), a),
                (dec!(1), dec!(1), b),
                (dec!(2), dec!(2), a),
                (dec!(3), dec!(3), b),
            ],
            // A full grid, with z at the numeric extreme.
            _ => {
                let mut grid = Vec::new();
                for i in 0..4i64 {
                    for j in 0..4i64 {
                        grid.push((
                            Decimal::from(i),
                            Decimal::from(j),
                            if (i + j) % 2 == 0 { a } else { b },
                        ));
                    }
                }
                grid
            }
        };
        surface_from(&points)
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Every interpolator returns for every combination of a degenerate point
    /// set and an extreme query abscissa. This covers the two failures the
    /// geometry adds to the numeric ones: an out-of-range window index, and a
    /// zero-length knot interval from a repeated abscissa.
    #[test]
    fn test_curve_interpolation_never_panics(
        curve in degenerate_curve(),
        kind in interpolation_type(),
        x in extreme_coordinate(),
    ) {
        let _ = curve.interpolate(x, kind);
        let _ = curve.find_bracket_points(x);
        let _ = curve.get_closest_point(&x);
        let _ = curve.get_values(x);
        let _ = curve.contains_point(&x);
        let _ = curve.get_point(&x);
    }

    /// Every metric returns for every degenerate curve, including the empty
    /// one, the flat one whose standard deviation is zero, and the one whose
    /// abscissas collapse the regression denominator.
    #[test]
    fn test_curve_metrics_never_panic(curve in degenerate_curve()) {
        let _ = curve.compute_basic_metrics();
        let _ = curve.compute_shape_metrics();
        let _ = curve.compute_range_metrics();
        let _ = curve.compute_trend_metrics();
        let _ = curve.compute_risk_metrics();
        let _ = curve.compute_curve_metrics();
        let _ = curve.get_x_values();
        let _ = curve.get_index_values();
        let _ = curve.to_string();
    }

    /// Every transformation returns, including the ones whose factor or delta
    /// pushes a coordinate out of the representable range, and the wrong-arity
    /// argument lists.
    #[test]
    fn test_curve_transformations_never_panic(
        curve in degenerate_curve(),
        a in extreme_coordinate(),
        b in extreme_coordinate(),
    ) {
        let _ = curve.translate(vec![&a, &b]);
        let _ = curve.translate(vec![&a]);
        let _ = curve.translate(vec![]);
        let _ = curve.scale(vec![&a, &b]);
        let _ = curve.scale(vec![&a]);
        let _ = curve.extrema();
        let _ = curve.measure_under(&a);
        let _ = curve.derivative_at(&Point2D::new(a, b));
    }

    /// Merging, intersecting and axis-aligning two independently degenerate
    /// curves returns, whichever operation is asked for.
    #[test]
    fn test_curve_binary_operations_never_panic(
        left in degenerate_curve(),
        right in degenerate_curve(),
        operation in merge_operation(),
        kind in interpolation_type(),
    ) {
        let _ = left.merge_with(&right, operation);
        let _ = Curve::merge(&[&left, &right], operation);
        let _ = Curve::merge(&[], operation);
        let _ = Curve::merge(&[&left], operation);
        let _ = left.intersect_with(&right);
        let _ = left.merge_axis_interpolate(&right, kind);
    }

    /// Parametric construction returns for a zero step count, which divides
    /// the span by zero, and for a span that overflows the `Decimal` range.
    #[test]
    fn test_curve_construction_never_panics(
        start in extreme_coordinate(),
        end in extreme_coordinate(),
        steps in 0usize..4,
    ) {
        let _ = Curve::construct(ConstructionMethod::Parametric {
            f: Box::new(|t: Decimal| Ok(Point2D::new(t, t))),
            params: ConstructionParams::D2 { t_start: start, t_end: end, steps },
        });
        let _ = Curve::construct(ConstructionMethod::<Point2D, Decimal>::FromData {
            points: BTreeSet::new(),
        });
    }

    /// Statistical generation returns for every point count, including counts
    /// that exceed the abscissas the curve can supply and a target
    /// distribution with no spread.
    #[test]
    fn test_statistical_curve_generation_never_panics(
        curve in degenerate_curve(),
        num_points in 0usize..12,
        std_dev in extreme_coordinate(),
        mean in extreme_coordinate(),
    ) {
        let basic = BasicMetrics { mean, median: mean, mode: mean, std_dev };
        let shape = ShapeMetrics {
            skewness: Decimal::ZERO,
            kurtosis: dec!(3),
            peaks: vec![],
            valleys: vec![],
            inflection_points: vec![],
        };
        let range = RangeMetrics {
            min: Point2D::new(Decimal::ZERO, dec!(5)),
            max: Point2D::new(dec!(10), dec!(15)),
            range: dec!(10),
            quartiles: (dec!(7), dec!(10), dec!(13)),
            interquartile_range: dec!(6),
        };
        let trend = TrendMetrics {
            slope: dec!(1),
            intercept: Decimal::ZERO,
            moving_average: vec![],
            r_squared: dec!(1),
        };

        let _ = curve.generate_statistical_curve(
            &basic, &shape, &range, &trend, num_points, Some(42),
        );
        let _ = curve.generate_refined_statistical_curve(
            &basic, &shape, &range, &trend, num_points, 2, dec!(0.1), Some(42),
        );
        let _ = curve.verify_curve_metrics(&curve, &basic, dec!(0.1));
    }

    /// Every surface interpolator returns for every combination of a
    /// degenerate point set and an extreme query point, including the
    /// two-point surface that used to index a third neighbour and the
    /// collinear one whose barycentric denominator is zero.
    #[test]
    fn test_surface_interpolation_never_panics(
        surface in degenerate_surface(),
        kind in interpolation_type(),
        x in extreme_coordinate(),
        y in extreme_coordinate(),
    ) {
        let xy = Point2D::new(x, y);
        let _ = surface.interpolate(xy, kind);
        let _ = surface.get_closest_point(&xy);
        let _ = surface.get_values(xy);
        let _ = surface.contains_point(&xy);
        let _ = surface.get_point(&xy);
        let _ = surface.get_f64_points();
    }

    /// Every surface metric returns, including on the empty surface where
    /// every mean divides by zero and the quartile index reads past the end.
    #[test]
    fn test_surface_metrics_never_panic(surface in degenerate_surface()) {
        let _ = surface.compute_basic_metrics();
        let _ = surface.compute_shape_metrics();
        let _ = surface.compute_range_metrics();
        let _ = surface.compute_trend_metrics();
        let _ = surface.compute_risk_metrics();
        let _ = surface.get_index_values();
    }

    /// Every surface transformation returns, at every arity and every extreme.
    #[test]
    fn test_surface_transformations_never_panic(
        surface in degenerate_surface(),
        a in extreme_coordinate(),
        b in extreme_coordinate(),
        c in extreme_coordinate(),
    ) {
        let _ = surface.translate(vec![&a, &b, &c]);
        let _ = surface.translate(vec![&a]);
        let _ = surface.scale(vec![&a, &b, &c]);
        let _ = surface.scale(vec![&a]);
        let _ = surface.extrema();
        let _ = surface.measure_under(&a);
        let _ = surface.derivative_at(&Point3D::new(a, b, c));
    }

    /// Merging, intersecting and axis-aligning two independently degenerate
    /// surfaces returns, whichever operation is asked for.
    #[test]
    fn test_surface_binary_operations_never_panic(
        left in degenerate_surface(),
        right in degenerate_surface(),
        operation in merge_operation(),
        kind in interpolation_type(),
    ) {
        let _ = left.merge_with(&right, operation);
        let _ = Surface::merge(&[&left, &right], operation);
        let _ = Surface::merge(&[], operation);
        let _ = Surface::merge(&[&left], operation);
        let _ = left.intersect_with(&right);
        let _ = left.merge_axis_interpolate(&right, kind);
    }

    /// Parametric surface construction returns for a zero step count on
    /// either axis and for spans that overflow the `Decimal` range.
    #[test]
    fn test_surface_construction_never_panics(
        x_start in extreme_coordinate(),
        x_end in extreme_coordinate(),
        y_start in extreme_coordinate(),
        y_end in extreme_coordinate(),
        x_steps in 0usize..3,
        y_steps in 0usize..3,
    ) {
        let _ = Surface::construct(ConstructionMethod::Parametric {
            f: Box::new(|p: Point2D| Ok(Point3D::new(p.x, p.y, p.x))),
            params: ConstructionParams::D3 {
                x_start, x_end, y_start, y_end, x_steps, y_steps,
            },
        });
        let _ = Surface::construct(ConstructionMethod::<Point3D, Point2D>::FromData {
            points: BTreeSet::new(),
        });
    }
}
