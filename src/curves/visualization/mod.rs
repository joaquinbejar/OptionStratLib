//! # Curve Visualization Module
//!
//! Advanced visualization tools for rendering and plotting mathematical curves.
//!
//! ## Visualization Capabilities
//! - High-quality curve plotting
//! - Multiple rendering backends
//! - Customizable plot styles
//! - Support for various curve types
//!
//! ## Supported Rendering
//! - 2D Line Plots
//! - Point Scatter Plots
//! - Interpolated Curve Rendering
//!
//! ## Features
//! - Precise decimal-based plotting
//! - Configurable color and style
//! - Export to multiple formats
//!
//! ## Example
//! ```rust
//! // Plotting a curve with custom styling
//! curve.plot()
//!     .with_color(Color::Blue)
//!     .with_line_width(2.0)
//!     .render("curve_plot.png");
//! ```
mod plotters;
