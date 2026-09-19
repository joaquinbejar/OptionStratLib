//! Option projections onto the math containers: `BasicCurves` (an option set
//! onto a [`crate::curves::Curve`]) and `BasicSurfaces` (onto a
//! [`crate::surfaces::Surface`]), plus their `OptionChain` implementations.
//!
//! These traits price `Options` and read Greeks, so they are analytics, not
//! geometry (ADR-0001 D4). `curves::BasicCurves` and `surfaces::BasicSurfaces`
//! remain as compatibility re-exports.

/// `impl BasicCurves for OptionChain`, `impl BasicSurfaces for OptionChain`
/// and the inherent projection wrappers on `OptionChain`.
mod chain;
/// `BasicCurves` trait.
mod curves;
/// `BasicSurfaces` trait.
mod surfaces;

pub use curves::BasicCurves;
pub use surfaces::BasicSurfaces;
