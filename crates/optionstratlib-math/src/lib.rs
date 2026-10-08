#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]
// Per rules/global_rules.md §Arithmetic, an operator that overflows, divides
// by zero or breaks the `Positive` invariant aborts the caller, so every
// operation on `Decimal`, `Positive` and the integers goes through its
// checked form (#788). `clippy.toml` exempts the unary minus on `Decimal`,
// which cannot overflow. Casts that truncate, wrap or drop the sign are
// denied for the same reason. Unit tests are exempt, as above.
#![deny(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
#![cfg_attr(
    test,
    allow(
        clippy::arithmetic_side_effects,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )
)]

//! # optionstratlib-math
//!
//! Generic numerical containers and algorithms of OptionStratLib: 2D curves,
//! 3D surfaces, the geometry, construction and interpolation machinery they
//! share, and the statistics extracted from them. Nothing here knows about
//! option chains, Greeks, pricing or plotting; it depends only on
//! `optionstratlib-core` and general numeric crates.
//!
//! - [`curves`]: [`curves::Curve`], [`curves::Point2D`] and the curve traits.
//! - [`surfaces`]: [`surfaces::Surface`], [`surfaces::Point3D`] and the
//!   surface traits.
//! - [`geometrics`]: construction, interpolation (linear, bilinear, cubic,
//!   spline), arithmetic and metric extraction shared by curves and surfaces.
//! - [`error`]: [`error::CurveError`], [`error::SurfaceError`],
//!   [`error::InterpolationError`] and [`error::MetricsError`].
//!
//! Coordinates are `rust_decimal::Decimal`; arithmetic on them is checked.
//! Option-specific projections of curves and surfaces (`BasicCurves`,
//! `BasicSurfaces`) belong to the analytics layer, and plotting to
//! `optionstratlib-visualization`; the `optionstratlib` facade provides both.
//!
//! ## Imports: no prelude
//!
//! This crate has no `prelude` module, by decision (#518). Each module root
//! re-exports what a caller needs: `curves` (`Curve`, `Point2D`, `Curvable`,
//! `StatisticalCurve`), `surfaces` (`Surface`, `Point3D`, `Surfacable`),
//! `geometrics` (construction, interpolation and the shared traits) and
//! `error`.
//!
//! Measured when the crate was extracted, counting explicit
//! `optionstratlib::{curves,surfaces,geometrics}::…` imports: the examples
//! name no math item explicitly (they use the facade's `prelude::*`), and
//! tests and benches name 20 distinct math items in 4 files, most of them in
//! one property test that imports 15 `geometrics` items. That use is narrow
//! and specialised; a crate glob would serve none of it better than the
//! module roots already do.
//!
//! ```rust
//! use optionstratlib_math::curves::{Curvable, Curve, Point2D, StatisticalCurve};
//! use optionstratlib_math::error::{CurveError, SurfaceError};
//! use optionstratlib_math::geometrics::{
//!     ConstructionMethod, ConstructionParams, GeometricObject, Interpolate, InterpolationType,
//! };
//! use optionstratlib_math::surfaces::{Point3D, Surfacable, Surface};
//! ```
//!
//! ## Behaviour that lives outside this crate
//!
//! Some capabilities take a math type as input but belong to a higher layer,
//! so they are implemented there, not here (#517):
//!
//! | Capability | Where | Owner |
//! | --- | --- | --- |
//! | `BasicCurves`, `BasicSurfaces`: projecting an option set onto a `Curve` or `Surface` (prices options, reads Greeks) | `optionstratlib::analytics` | analytics; final adapter in M4-01 (#529) |
//! | `Graph` and `Plottable` for `Curve`, `Vec<Curve>` and `Surface` | `optionstratlib_visualization::visualization`, also `optionstratlib::visualization` | visualization (#542) |
//!
//! Rendering failures are `GraphError`s of the visualization layer; the math
//! errors carry no rendering variant.
//!
//! ## Place in the workspace
//!
//! - **Depends on** `optionstratlib-core`.
//! - **Must not depend on** `optionstratlib-pricing`, `-simulation`, `-market`,
//!   `-analytics`, `-strategies`, `-backtest` and `-visualization`; `make
//!   check-graph` enforces the layering (ADR-0001 D9).
//! - **In the facade:** `optionstratlib::{curves, surfaces, geometrics}` and the
//!   math errors in `optionstratlib::error`, under the facade feature `math`. The
//!   facade paths are the same types as the paths here; the [ownership
//!   map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
//!   lists every one with its feature.
//!
//! Moving from 0.21? The [0.22 architecture and adoption
//! guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md)
//! maps each redesign to its replacement workflow; 0.22 keeps no
//! compatibility with 0.21.
//!
//! ## Minimal example
//!
//! ```rust
//! use optionstratlib_math::curves::{Curve, Point2D};
//! use std::collections::BTreeSet;
//!
//! let points: BTreeSet<Point2D> = [(0, 0), (1, 2), (2, 4)]
//!     .into_iter()
//!     .map(|(x, y)| Point2D::new(x, y))
//!     .collect();
//! let curve = Curve::new(points);
//! assert_eq!(curve.points.len(), 3);
//! ```
//!
//! ## Runnable example
//!
//! A runnable program that depends on this crate directly, with the smallest
//! dependency set and no facade, is [`osl-example-direct-math`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/math); `make tree-example-direct-math`
//! asserts its resolved graph.
//!
//! ## Features
//!
//! - `schema` (off by default): derives `utoipa::ToSchema` on the math types
//!   and enables `optionstratlib-core/schema`.

/// Geometry shared by curves and surfaces: construction, interpolation,
/// arithmetic and metric extraction.
pub mod geometrics;

/// Two-dimensional curves: `Curve`, `Point2D` and their traits.
pub mod curves;

/// Three-dimensional surfaces: `Surface`, `Point3D` and their traits.
pub mod surfaces;

/// Errors raised by the math types.
pub mod error;

/// Version of the `optionstratlib-math` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compiles the Rust examples of this crate's `README.md` as doctests, so the
/// README cannot drift from the API (#554). Exists only under `cfg(doctest)`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
