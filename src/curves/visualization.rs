//! Compatibility module: curve plotting has moved to the top-level
//! `visualization` module.
//!
//! `Plottable` and `PlotBuilder` are visualization-owned, and the `Graph` /
//! `Plottable` adapters for [`crate::curves::Curve`] and `Vec<Curve>` live
//! next to them (ADR-0001 D2, M1-05). This module is kept only because
//! `curves::visualization` is a public path in 0.21; it exports nothing.
//!
//! ```rust,no_run
//! # fn main() -> Result<(), optionstratlib::error::Error> {
//! use std::fs;
//! use std::path::PathBuf;
//! use rust_decimal::Decimal;
//! use rust_decimal_macros::dec;
//! use optionstratlib::curves::{Curve, Point2D};
//! use optionstratlib::geometrics::GeometricObject;
//! use optionstratlib::visualization::Plottable;
//!
//! let curve = Curve::from_vector(vec![
//!     Point2D::new(Decimal::ZERO, Decimal::ZERO),
//!     Point2D::new(Decimal::ONE, Decimal::ONE),
//!     Point2D::new(Decimal::TWO, dec!(4.0)),
//! ]);
//!
//! #[cfg(feature = "static_export")]
//! {
//!     let filename = PathBuf::from("single_curve_doc.png");
//!     curve.plot()
//!         .title("My Curve")
//!         .save(filename.clone())?;
//!     if filename.exists() {
//!         fs::remove_file(&filename)?;
//!     }
//! }
//! # Ok(())
//! # }
//! ```
