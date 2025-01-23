//! # Curve Arithmetic Module
//!
//! Provides arithmetic operations for mathematical curves, enabling complex transformations
//! and computations on curve data.
//!
//! ## Features
//! - Support for basic arithmetic operations between curves
//! - Merge strategies for combining multiple curves
//! - Flexible operation selection with `MergeOperation` enum
//!
//! ## Supported Operations
//! - Addition
//! - Subtraction
//! - Multiplication
//! - Division
//! - Maximum value selection
//! - Minimum value selection
//!
//! ## Usage
//! ```rust
//! use rust_decimal::Decimal;
//! use optionstratlib::geometrics::{Arithmetic, GeometricObject, MergeOperation};
//! use optionstratlib::curves::{Curve, Point2D};
//! let curve1 = Curve::from_vector(vec![
//!            Point2D::new(Decimal::ZERO, Decimal::ZERO), // p11
//!            Point2D::new(Decimal::ONE, Decimal::ONE),   // p12
//!            Point2D::new(Decimal::ZERO, Decimal::ONE),  // p21
//!            Point2D::new(Decimal::ONE, Decimal::TWO),   // p22
//!        ]);
//! let curve2 = curve1.clone();
//! // Merge two curves by adding their values
//! let result_curve = Curve::merge(&[&curve1, &curve2], MergeOperation::Add);
//! ```
//!
//! ## Design Principles
//! - High-precision arithmetic using Decimal type
//! - Error handling for robust curve operations
//! - Parallel processing for efficient computations
//!
pub mod arithmetic;
mod traits;
mod transformations;
mod axis;

pub use arithmetic::MergeOperation;
pub use traits::Arithmetic;
pub use axis::{AxisOperations, MergeAxisInterpolate};
