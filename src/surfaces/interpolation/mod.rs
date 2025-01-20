


//! Surface Interpolation Module
//!
//! Advanced interpolation techniques for 3D surface estimation.
//!
mod traits;
mod types;
mod linear;
mod bilinear;
mod trilinear;

mod cubic;

pub use traits::Interpolate;
pub use types::SurfaceInterpolationType;
pub use linear::LinearInterpolation;
pub use bilinear::BilinearInterpolation;
pub use trilinear::TrilinearInterpolation;