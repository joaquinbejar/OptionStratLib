//! # Curve Visualization Module
//!
//! Provides advanced plotting capabilities for mathematical curves with high flexibility and precision.
//!
//! ## Features
//! - Generic plotting for single and multiple curves
//! - High-precision visualization using Decimal types
//! - Customizable plot configuration
//! - Multiple styling options
//! - Error handling with detailed feedback
//!
//! ## Core Components
//! - `Plottable` trait: Enables plotting functionality for curves
//! - `PlotBuilder`: Configurable plot generation
//! - `PlotOptions`: Detailed plot styling options
//!
//! ## Usage Examples
//! ```rust
//! // Plot a single curve
//! use std::fs;
//! use std::path::{Path, PathBuf};
//! use rust_decimal::Decimal;
//! use rust_decimal_macros::dec;
//! use optionstratlib::curves::{Curve, Point2D};
//! use optionstratlib::geometrics::{GeometricObject, Plottable};
//!
//! let curve = Curve::from_vector(vec![
//!     Point2D::new(Decimal::ZERO, Decimal::ZERO),
//!     Point2D::new(Decimal::ONE, Decimal::ONE),
//!     Point2D::new(Decimal::TWO, dec!(4.0)),
//! ]);
//!
//! // Simple plot with default settings
//! #[cfg(feature = "kaleido")]
//! {
//!     let filename = "single_curve_doc.png";
//!     let filename = PathBuf::from("single_curve_doc.png");
//!     curve.plot()
//!         .title("My Curve")
//!         .save(filename.clone()).expect("panic message");
//!     if filename.exists() {
//!        fs::remove_file(&filename).unwrap_or_else(|_| panic!("Failed to remove {:?}", filename));
//!     }
//! }
//! 
//! 
//! // Customized multiple curve plot
//! let curves = vec![curve.clone(), curve.clone()];
//! #[cfg(feature = "kaleido")]
//! {
//!     let filename = PathBuf::from("multiple_curves_doc.png");
//!     curves.plot()
//!         .title("Curve Comparison")
//!         .dimensions(1000, 600)
//!         .save(filename.clone()).expect("panic message");
//!     if filename.exists() {
//!         fs::remove_file(&filename).unwrap_or_else(|_| panic!("Failed to remove {:?}", filename));
//!     }
//! }
//! ```
//!
//! ## Plot Customization
//! - Set titles and axis labels
//! - Customize line colors and width
//! - Adjust plot dimensions
//! - Support for multiple curves
//!
//! ## Error Handling
//! Uses `CurvesError` for robust error reporting during plot generation
//!
//! ## Dependencies
//! - `plotters`: High-performance plotting library
//! - `rust_decimal`: Precise decimal calculations
//! - `num_traits`: Type conversions
//!
//! ## Performance Considerations
//! - Uses f64 for plotting to ensure compatibility with plotting backends
//! - Minimal memory overhead
//! - Efficient point conversion
//!
//! ## Visualization Strategies
//! - Automatic range detection
//! - Smart color selection for multiple curves
//! - Adaptive plotting for different data ranges
//!
//! ## Extensibility
//! The `Plottable` trait allows easy extension to new curve types and plotting strategies.
//!
//! ## Limitations
//! - Requires at least two points for plotting
//! - Plots may look different with extreme data ranges
//!
//! ## Best Practices
//! - Always specify plot dimensions for consistent results
//! - Use high-precision Decimal types for input data
//! - Handle potential plotting errors gracefully
//!
//! ## Future Improvements
//! - Additional plot types (scatter, bar, etc.)
//! - More advanced styling options
//! - Enhanced error diagnostics
mod plotters;
