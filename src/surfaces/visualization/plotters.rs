//! This module provides functionality for plotting surfaces using the `plotters` crate.
//! It supports plotting single surfaces as well as collections of surfaces.
//!
//! The core of this module revolves around the `Plottable` trait, which is implemented for both
//! `Surface` and `Vec<Surface>`. This trait provides a common interface for generating plots
//! with customizable options.
//!
//! The `PlotBuilder` struct is used to configure and build the plots. It offers various methods
//! to customize plot appearance, such as setting titles, labels, dimensions, and colors.
//!
//! The `save` method is used to save the generated plot to a file.
//!
//! The module also includes a set of utility functions for applying shading to points on a surface,
//! aiding in the visualization of 3D surfaces.  Error handling is managed using the `SurfaceError` type.
//!
//! # Example Usage
//!
//! ```
//! # use std::fs;
//! use std::path::PathBuf;
//! use rust_decimal_macros::dec;
//! use optionstratlib::geometrics::{GeometricObject, Plottable};
//! use optionstratlib::surfaces::{Point3D, Surface};
//!
//! # let p1 = Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0));
//! # let p2 = Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0));
//! # let p3 = Point3D::new(dec!(2.0), dec!(4.0), dec!(4.0));
//! # let points = vec![&p1, &p2, &p3];
//! # let surface = Surface::from_vector(points);
//!
//! // Create a surface
//! #[cfg(feature = "kaleido")]
//! {
//!     let dst = PathBuf::from("surface_plot.png");
//!     // Plot the surface and save to a file
//!     surface.plot()
//!         .title("My Surface Plot")
//!         .x_label("X Axis")
//!         .y_label("Y Axis")
//!         .z_label("Z Axis") // Example of z-axis label
//!         .save(dst.clone())
//!         .expect("Failed to save plot");
//!     if dst.exists() {
//!         fs::remove_file(dst)
//!            .unwrap_or_else(|_| panic!("Failed to remove surface_plot.png"));
//!     }
//! }
//! ```
//!
use crate::error::SurfaceError;
use crate::geometrics::{PlotBuilder, Plottable};
use crate::surfaces::Surface;
use crate::visualization::Graph;

/// Plottable implementation for single Surface
impl Plottable for Surface {
    type Error = SurfaceError;

    fn plot(&self) -> PlotBuilder<Self>
    where
        Self: Sized,
    {
        PlotBuilder {
            data: self.clone(),
            options: self.graph_config(),
        }
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    struct MockChart {
        pub x_desc: String,
        pub y_desc: String,
    }

    impl MockChart {
        pub fn new() -> Self {
            MockChart {
                x_desc: String::new(),
                y_desc: String::new(),
            }
        }

        pub fn configure_mesh(&mut self) -> &mut Self {
            self
        }

        pub fn x_label_formatter(&mut self, _formatter: &dyn Fn(f64) -> String) -> &mut Self {
            self
        }

        pub fn y_label_formatter(&mut self, _formatter: &dyn Fn(f64) -> String) -> &mut Self {
            self
        }

        pub fn x_desc(&mut self, desc: &str) -> &mut Self {
            self.x_desc = desc.to_string();
            self
        }

        pub fn y_desc(&mut self, desc: &str) -> &mut Self {
            self.y_desc = desc.to_string();
            self
        }
    }

    #[test]
    fn test_map_err_to_std_error() {
        let result: Result<(), SurfaceError> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Test error")).map_err(|e| {
                SurfaceError::StdError {
                    reason: e.to_string(),
                }
            });

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            SurfaceError::StdError { reason } => {
                assert_eq!(reason, "Test error");
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_configure_chart_mesh() {
        let mut chart = MockChart::new(); // Simular un gráfico
        chart
            .configure_mesh()
            .x_label_formatter(&|v| format!("{:.2}", v))
            .y_label_formatter(&|v| format!("{:.2}", v))
            .x_desc("X-axis")
            .y_desc("Y-axis");
        assert_eq!(chart.x_desc, "X-axis");
        assert_eq!(chart.y_desc, "Y-axis");
    }

    #[test]
    fn test_draw_series_error() {
        let result: Result<(), SurfaceError> =
            Err("Draw error".to_string()).map_err(|e| SurfaceError::StdError { reason: e });

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            SurfaceError::StdError { reason } => {
                assert_eq!(reason, "Draw error");
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
