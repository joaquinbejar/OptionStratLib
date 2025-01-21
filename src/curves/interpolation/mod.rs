//! # Curve Interpolation Module
//!
//! Advanced interpolation techniques for precise curve estimation and data approximation.
//!
//! ## Interpolation Methods
//! - Linear Interpolation
//! - Cubic Interpolation
//! - Spline Interpolation
//! - Bilinear Interpolation
//!
//! ## Key Features
//! - High-precision decimal-based calculations
//! - Flexible interpolation strategy selection
//! - Error handling for edge cases
//! - Support for various data point distributions
//!
//! ## Usage
//! ```rust
//! use rust_decimal::Decimal;
//! use rust_decimal_macros::dec;
//! use optionstratlib::curves::{Curve, Point2D};
//! use optionstratlib::curves::interpolation::{Interpolate, InterpolationType};
//! let curve = Curve::from_vector(vec![
//!            Point2D::new(Decimal::ZERO, Decimal::ZERO), // p11
//!            Point2D::new(Decimal::ONE, Decimal::ONE),   // p12
//!            Point2D::new(Decimal::ZERO, Decimal::ONE),  // p21
//!            Point2D::new(Decimal::ONE, Decimal::TWO),   // p22
//!        ]);
//! // Interpolate a point using cubic method
//! let x = dec!(0.5);
//! let interpolated_point = curve.interpolate(x, InterpolationType::Cubic);
//! ```
//!
//! ## Interpolation Strategies
//! - Smooth curve fitting
//! - Missing value estimation
//! - Data point refinement
//!
mod bilinear;
mod cubic;
mod linear;
mod spline;
mod traits;
mod types;

pub use bilinear::BiLinearInterpolation;
pub use cubic::CubicInterpolation;
pub use linear::LinearInterpolation;
pub use spline::SplineInterpolation;
pub use interpolation::Interpolate;
pub use types::InterpolationType;
