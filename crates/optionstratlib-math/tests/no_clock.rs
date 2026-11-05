//! The math containers without the wall clock (#944). This crate has no `clock` feature:
//! the closure of a parametric curve or surface is the caller's, and may price, so a
//! parallel construction has to carry the caller's reference datetime to its workers.

use chrono::{DateTime, TimeZone, Utc};
use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::utils::time::now;
use optionstratlib_math::curves::{Curve, Point2D};
use optionstratlib_math::error::{CurveError, SurfaceError};
use optionstratlib_math::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject};
use optionstratlib_math::surfaces::{Point3D, Surface};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn base() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()
}

fn in_a_pool<R: Send>(work: impl FnOnce() -> R + Send) -> R {
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap()
        .install(work)
}

#[test]
fn test_parametric_curve_carries_the_reference_to_its_workers() {
    // With a clock the carrier does nothing, as in a default build.
    if now().is_ok() {
        return;
    }
    let curve: Result<Curve, CurveError> = in_a_pool(|| {
        ExpirationDate::set_reference_datetime(Some(base()));
        let curve = Curve::construct(ConstructionMethod::Parametric {
            f: Box::new(|t: Decimal| {
                if ExpirationDate::get_reference_datetime() == Some(base()) {
                    Ok(Point2D::new(t, t))
                } else {
                    Err(CurveError::ConstructionError("no reference".to_string()))
                }
            }),
            params: ConstructionParams::D2 {
                t_start: dec!(0),
                t_end: dec!(1),
                steps: 64,
            },
        });
        ExpirationDate::set_reference_datetime(None);
        curve
    });
    assert!(curve.is_ok(), "{curve:?}");
}

#[test]
fn test_parametric_surface_carries_the_reference_to_its_workers() {
    if now().is_ok() {
        return;
    }
    let surface: Result<Surface, SurfaceError> = in_a_pool(|| {
        ExpirationDate::set_reference_datetime(Some(base()));
        let surface = Surface::construct(ConstructionMethod::Parametric {
            f: Box::new(|point: Point2D| {
                if ExpirationDate::get_reference_datetime() == Some(base()) {
                    Ok(Point3D::new(point.x, point.y, dec!(0)))
                } else {
                    Err(SurfaceError::ConstructionError("no reference".to_string()))
                }
            }),
            params: ConstructionParams::D3 {
                x_start: dec!(0),
                x_end: dec!(1),
                y_start: dec!(0),
                y_end: dec!(1),
                x_steps: 8,
                y_steps: 8,
            },
        });
        ExpirationDate::set_reference_datetime(None);
        surface
    });
    assert!(surface.is_ok(), "{surface:?}");
}
