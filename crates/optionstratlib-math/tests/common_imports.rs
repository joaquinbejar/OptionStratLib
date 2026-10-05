//! The imports a math consumer writes, compiled and exercised (#518).
//!
//! `optionstratlib-math` has no prelude by decision; these module roots are
//! its intended entry points. A test here breaks when one of them moves.

use optionstratlib_math::curves::{Curvable, Curve, Point2D, StatisticalCurve};
use optionstratlib_math::error::{CurveError, SurfaceError};
use optionstratlib_math::geometrics::{
    ConstructionMethod, ConstructionParams, GeometricObject, Interpolate, InterpolationType,
};
use optionstratlib_math::surfaces::{Point3D, Surfacable, Surface};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn test_curve_from_curves_root() {
    let curve = Curve::from_vector(vec![
        Point2D::new(Decimal::ZERO, Decimal::ZERO),
        Point2D::new(Decimal::ONE, Decimal::ONE),
        Point2D::new(Decimal::TWO, dec!(4.0)),
    ]);
    match curve.interpolate(dec!(0.5), InterpolationType::Linear) {
        Ok(point) => assert_eq!(point.y, dec!(0.5)),
        Err(error) => panic!("linear interpolation failed: {error}"),
    }
}

#[test]
fn test_construction_from_geometrics_root() {
    let result: Result<Curve, CurveError> = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| Ok(Point2D::new(t, t))),
        params: ConstructionParams::D2 {
            t_start: Decimal::ZERO,
            t_end: Decimal::ONE,
            steps: 4,
        },
    });
    assert!(result.is_ok());
}

#[test]
fn test_surface_from_surfaces_root() {
    let surface = Surface::from_vector(vec![
        Point3D::new(Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
        Point3D::new(Decimal::ONE, Decimal::ZERO, Decimal::ONE),
        Point3D::new(Decimal::ZERO, Decimal::ONE, Decimal::ONE),
    ]);
    let _error_type: Option<SurfaceError> = None;
    assert_eq!(surface.points.len(), 3);
}

/// The traits are importable by name for bounds and `dyn` use.
fn _trait_paths_resolve<C: Curvable, S: Surfacable, T: StatisticalCurve>() {}
