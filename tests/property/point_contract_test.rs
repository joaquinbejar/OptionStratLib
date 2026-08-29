//! Property-based tests for the `Eq` / `Ord` / `Hash` contract on `Point2D`
//! and `Point3D`.
//!
//! `Ord` requires that `a == b` hold exactly when `a.cmp(&b)` is
//! `Ordering::Equal`, and `Hash` requires that equal values hash alike. Both
//! types used to break the first: they compared equal on a prefix of their
//! coordinates while ordering on all of them, so two points could be equal
//! and yet order strictly. `Curve` stores its points in a
//! `BTreeSet<Point2D>`, which dispatches on `Ord`, while `Surface` indexes
//! itself by `Point2D` through a `HashSet`, which dispatches on `Eq` and
//! `Hash` — so the disagreement was not academic: it let a curve hold two
//! ordinates for one abscissa and made half of a surface grid disappear on
//! merge.
//!
//! These properties are the invariant, driven over coordinates chosen to
//! stress `Decimal` equality: values that are numerically equal at different
//! scales (`1.0` versus `1.00`) hash differently if the implementation reads
//! the raw mantissa and scale rather than normalizing, and there are enough
//! repeats in the strategy that pairs actually collide.

use optionstratlib::curves::Point2D;
use optionstratlib::surfaces::Point3D;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Hash of a single value, for comparing two hashes directly.
fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// A small pool of coordinates, deliberately narrow so that independently
/// generated points collide often enough to exercise the equal case, and
/// containing values that are numerically equal at different scales.
fn coordinate() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(dec!(1)),
        Just(dec!(1.0)),
        Just(dec!(1.00)),
        Just(dec!(-1.0)),
        Just(dec!(2.5)),
        Just(dec!(2.50)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
        (-1000i64..1000i64).prop_map(Decimal::from),
    ]
}

fn point2d() -> impl Strategy<Value = Point2D> {
    (coordinate(), coordinate()).prop_map(|(x, y)| Point2D::new(x, y))
}

fn point3d() -> impl Strategy<Value = Point3D> {
    (coordinate(), coordinate(), coordinate()).prop_map(|(x, y, z)| Point3D::new(x, y, z))
}

proptest! {
    /// The `Ord` contract: equality and `cmp` are the same relation.
    #[test]
    fn point2d_eq_iff_cmp_equal(a in point2d(), b in point2d()) {
        prop_assert_eq!(a == b, a.cmp(&b) == Ordering::Equal);
    }

    /// The `Hash` contract: equal points hash alike. Unequal points may
    /// collide, so only this direction is asserted.
    #[test]
    fn point2d_hash_agrees_with_eq(a in point2d(), b in point2d()) {
        if a == b {
            prop_assert_eq!(hash_of(&a), hash_of(&b));
        }
    }

    /// `PartialOrd` agrees with `Ord`, as its contract requires for a type
    /// that implements both.
    #[test]
    fn point2d_partial_cmp_agrees_with_cmp(a in point2d(), b in point2d()) {
        prop_assert_eq!(a.partial_cmp(&b), Some(a.cmp(&b)));
    }

    /// Ordering is antisymmetric: reversing the arguments reverses the result.
    #[test]
    fn point2d_cmp_is_antisymmetric(a in point2d(), b in point2d()) {
        prop_assert_eq!(a.cmp(&b), b.cmp(&a).reverse());
    }

    /// Ordering is transitive.
    #[test]
    fn point2d_cmp_is_transitive(a in point2d(), b in point2d(), c in point2d()) {
        if a <= b && b <= c {
            prop_assert!(a <= c);
        }
    }

    /// Equality is reflexive, and a point hashes to itself.
    #[test]
    fn point2d_is_reflexive(a in point2d()) {
        prop_assert_eq!(a, a);
        prop_assert_eq!(a.cmp(&a), Ordering::Equal);
        prop_assert_eq!(hash_of(&a), hash_of(&a));
    }

    /// Equal points are indistinguishable by coordinate, which is what makes
    /// `Point2D` usable as a surface index: an index that compared on `x`
    /// alone would identify two different points in the plane.
    #[test]
    fn point2d_equality_reads_both_coordinates(a in point2d(), b in point2d()) {
        prop_assert_eq!(a == b, a.x == b.x && a.y == b.y);
    }

    /// The `Ord` contract for `Point3D`.
    #[test]
    fn point3d_eq_iff_cmp_equal(a in point3d(), b in point3d()) {
        prop_assert_eq!(a == b, a.cmp(&b) == Ordering::Equal);
    }

    /// The `Hash` contract for `Point3D`.
    #[test]
    fn point3d_hash_agrees_with_eq(a in point3d(), b in point3d()) {
        if a == b {
            prop_assert_eq!(hash_of(&a), hash_of(&b));
        }
    }

    /// `PartialOrd` agrees with `Ord` for `Point3D`.
    #[test]
    fn point3d_partial_cmp_agrees_with_cmp(a in point3d(), b in point3d()) {
        prop_assert_eq!(a.partial_cmp(&b), Some(a.cmp(&b)));
    }

    /// Ordering is antisymmetric for `Point3D`.
    #[test]
    fn point3d_cmp_is_antisymmetric(a in point3d(), b in point3d()) {
        prop_assert_eq!(a.cmp(&b), b.cmp(&a).reverse());
    }

    /// Ordering is transitive for `Point3D`.
    #[test]
    fn point3d_cmp_is_transitive(a in point3d(), b in point3d(), c in point3d()) {
        if a <= b && b <= c {
            prop_assert!(a <= c);
        }
    }

    /// Equality is reflexive for `Point3D`, and a point hashes to itself.
    #[test]
    fn point3d_is_reflexive(a in point3d()) {
        prop_assert_eq!(a, a);
        prop_assert_eq!(a.cmp(&a), Ordering::Equal);
        prop_assert_eq!(hash_of(&a), hash_of(&a));
    }

    /// Equality reads all three coordinates for `Point3D`.
    #[test]
    fn point3d_equality_reads_all_coordinates(a in point3d(), b in point3d()) {
        prop_assert_eq!(a == b, a.x == b.x && a.y == b.y && a.z == b.z);
    }
}

/// A `Decimal` compares equal across scales, so the points do too and must
/// hash alike. Pinned as an explicit case because it is the shape the
/// previous `Point2D::hash` got wrong: it hashed the raw mantissa and scale.
#[test]
fn equal_across_decimal_scales_hashes_alike() {
    let a = Point2D::new(dec!(1.0), dec!(2.0));
    let b = Point2D::new(dec!(1.000), dec!(2.0000));
    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));

    let c = Point3D::new(dec!(1.0), dec!(2.0), dec!(3.0));
    let d = Point3D::new(dec!(1.000), dec!(2.0000), dec!(3.00000));
    assert_eq!(c, d);
    assert_eq!(hash_of(&c), hash_of(&d));
}
