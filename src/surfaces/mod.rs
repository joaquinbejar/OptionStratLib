//! This module provides tools for working with 3D surfaces.
//!
//! It includes functionalities for defining surfaces, performing operations on them,
//! and visualizing them.  The core components are:
//!
//! * `Surface`: Represents a 3D surface.  See the `surface` module for more details.
//! * `Point3D`: Represents a point in 3D space.  See the `types` module for more details.
//! * `utils`: Contains utility functions for working with surfaces.  See the `utils` module for more details.
//! * `visualization`: Provides tools for visualizing surfaces.  See the `visualization` module for more details.
//!

mod basic;
mod surface;
mod types;
mod utils;
mod visualization;

pub use basic::BasicSurfaces;
pub use surface::Surface;
pub use types::Point3D;
