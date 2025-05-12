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
    use std::any::{Any, TypeId};
    use std::collections::BTreeSet;
    use rust_decimal_macros::dec;
    use crate::error::CurveError;
    use crate::surfaces::Point3D;
    use crate::visualization::{GraphConfig, GraphData, Series2D, TraceMode};
    use super::*;

    #[derive(Clone)]
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
    
    impl Graph for MockChart {
        fn graph_data(&self) -> GraphData {
            GraphData::Series(Series2D {
                x: vec![dec!(1.0), dec!(2.0)],
                y: vec![dec!(3.0), dec!(4.0)],
                name: "Test Series".to_string(),
                mode: TraceMode::Lines,
                line_color: None,
                line_width: None,
            })
        }

        fn graph_config(&self) -> GraphConfig {
            GraphConfig::default()
        }
    }
    
    impl Plottable for MockChart {
        type Error = CurveError;

        fn plot(&self) -> PlotBuilder<Self>
        where
            Self: Sized + Graph
        {
            PlotBuilder {
                data: self.clone(),
                options: self.graph_config(),
            }
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
        let mut chart = MockChart::new(); 
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
    fn test_plot() {
        let mut chart = MockChart::new(); 
        chart
            .configure_mesh()
            .x_label_formatter(&|v| format!("{:.2}", v))
            .y_label_formatter(&|v| format!("{:.2}", v))
            .x_desc("X-axis")
            .y_desc("Y-axis");
        assert_eq!(chart.x_desc, "X-axis");
        assert_eq!(chart.y_desc, "Y-axis");
        
        let plot = chart.plot();
        assert_eq!(plot.data.graph_data(), chart.graph_data());
        assert_eq!(plot.options, chart.graph_config());
        assert_eq!(plot.data.graph_config(), chart.graph_config());
        assert_eq!(plot.options.width, 1280);
        assert_eq!(plot.options.height, 720);
        assert_eq!(plot.options.title, "Graph");
        assert_eq!(plot.options.x_label, None);
        assert_eq!(plot.options.y_label, None);
        assert_eq!(plot.options.z_label, None);
        assert_eq!(plot.data.x_desc, "X-axis");
        assert_eq!(plot.data.y_desc, "Y-axis");
    }
    #[test]
    fn test_plot_surface() {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.5), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(0.5), dec!(1.0)),
            Point3D::new(dec!(0.5), dec!(0.5), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(0.5), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(0.5), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
        ]);
        let surface = Surface::new(points);
        let plot = surface.plot();
        // Check if the plot is type PlotBuilder<Surface>
        assert_eq!(plot.type_id(), TypeId::of::<PlotBuilder<Surface>>());
        assert_eq!(plot.data.graph_data(), surface.graph_data());
        assert_eq!(plot.options, surface.graph_config());
        assert_eq!(plot.data.graph_config(), surface.graph_config());
        assert_eq!(plot.options.width, 1280);
        assert_eq!(plot.options.height, 720);
        assert_eq!(plot.options.title, "Graph");
        assert_eq!(plot.options.x_label, None);
        assert_eq!(plot.options.y_label, None);
        assert_eq!(plot.options.z_label, None);

        
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
