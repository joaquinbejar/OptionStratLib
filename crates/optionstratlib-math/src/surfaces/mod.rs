//! This module provides tools for working with 3D surfaces.
//!
//! It includes functionalities for defining surfaces, performing operations on them,
//! and visualizing them.  The core components are:
//!
//! * `Surface`: Represents a 3D surface.  See the `surface` module for more details.
//! * `Point3D`: Represents a point in 3D space.  See the `types` module for more details.
//! * `utils` (test-only): planar, constant and paraboloid surface fixtures for the unit tests.
//! * Plotting lives in `optionstratlib-visualization` (`Plottable` and the `Graph` impl for `Surface`);
//!   this module keeps only the geometry.
//!

mod surface;
mod traits;
mod types;
// Test fixtures: nothing outside the unit tests builds these surfaces, and
// their unchecked grid arithmetic has no place in the library (#788).
#[cfg(test)]
mod utils;

pub use surface::Surface;
pub use traits::Surfacable;
pub use types::Point3D;
