mod bilinear;
mod cubic;
mod linear;
mod spline;
mod types;
mod traits;

pub use types::InterpolationType;
pub use traits::Interpolate;
pub use linear::LinearInterpolation;
pub use cubic::CubicInterpolation;
pub use spline::SplineInterpolation;
pub use bilinear::BiLinearInterpolation;