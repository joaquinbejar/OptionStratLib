//! # Curve Construction Module
//!
//! Provides flexible methods for creating mathematical curves from various data sources.
//!
//! ## Construction Methods
//! - From Explicit Data Points
//! - Parametric Curve Generation
//!
//! ## Features
//! - Direct point-based curve creation
//! - Function-based curve generation
//! - Parallel curve point computation
//! - Customizable generation strategies
//!
//! ## Construction Types
//! - `FromData`: Create curves from predefined points
//! - `Parametric`: Generate curves using mathematical functions
//!
//! ## Example
//! ```rust
//! use rust_decimal::{Decimal, MathematicalOps};
//! use optionstratlib::curves::construction::CurveConstructionMethod;
//! use optionstratlib::curves::{Curve, Point2D};
//! use optionstratlib::geometrics::GeometricObject;
//!
//! // Create a curve from data points
//! let curve = Curve::from_vector(vec![
//!             Point2D::new(Decimal::ZERO, Decimal::ZERO), // p11
//!             Point2D::new(Decimal::ONE, Decimal::ONE),   // p12
//!             Point2D::new(Decimal::ZERO, Decimal::ONE),  // p21
//!             Point2D::new(Decimal::ONE, Decimal::TWO),   // p22
//!         ]);
//!
//! // Generate a parametric curve
//! let parametric_curve = Curve::construct(CurveConstructionMethod::Parametric {
//!     f: Box::new(|t| Ok(Point2D::new(t, t.sin()))),
//!     t_start: Decimal::ZERO,
//!     t_end: Decimal::TWO_PI,
//!     steps: 100
//! });
//! ```

mod from_data;
mod parametric;
mod types;

pub use types::CurveConstructionMethod;
