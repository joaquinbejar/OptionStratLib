#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

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
//! visualization; the `optionstratlib` facade provides both.
//!
//! ## Behaviour that lives outside this crate
//!
//! Some capabilities take a math type as input but belong to a higher layer,
//! so they are implemented there, not here (#517):
//!
//! | Capability | Where | Owner |
//! | --- | --- | --- |
//! | `BasicCurves`, `BasicSurfaces`: projecting an option set onto a `Curve` or `Surface` (prices options, reads Greeks) | `optionstratlib::analytics` | analytics; final adapter in M4-01 (#529) |
//! | `Graph` and `Plottable` for `Curve`, `Vec<Curve>` and `Surface` | `optionstratlib::visualization` | visualization; graph adapters in M6-02 (#543) |
//!
//! Rendering failures are `GraphError`s of the visualization layer; the math
//! errors carry no rendering variant.
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
