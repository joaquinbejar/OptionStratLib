//! # Curve Visualization Module
//!
//! Provides a flexible plotting trait for mathematical curves and collections of curves.
//!
//! ## Features
//! - Generic plotting for single and multiple curves
//! - Customizable plot configuration
//! - Multiple output formats
//! - Error handling
//!
//! ## Usage Examples
//! ```rust
//! // Plot a single curve
//! use std::fs;
//! use std::path::{Path, PathBuf};
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::{Curve, Point2D};
//! use optionstratlib::geometrics::{GeometricObject, Plottable};
//!
//! let curve = Curve::from_vector(vec![
//!             Point2D::new(Decimal::ZERO, Decimal::ZERO), // p11
//!             Point2D::new(Decimal::ONE, Decimal::ONE),   // p12
//!             Point2D::new(Decimal::ZERO, Decimal::ONE),  // p21
//!             Point2D::new(Decimal::ONE, Decimal::TWO),   // p22
//!         ]);
//! #[cfg(feature = "static_export")]
//! {
//!     let filename = PathBuf::from("single_curve.png");
//!     curve.plot()
//!         .title("Single Curve")
//!         .save(filename.clone()).expect("panic message");
//!     if filename.exists() {
//!        fs::remove_file(&filename).unwrap_or_else(|_| panic!("Failed to remove {:?}", filename));
//!     }
//!     // Plot multiple curves
//!     let curve1 = curve.clone();
//!     let curve2 = curve.clone();
//!     let curve3 = curve.clone();
//!     let curves = vec![curve1, curve2, curve3];
//!     let filename = "multiple_curves.png";
//!     curves.plot()
//!         .title("Curve Comparison")
//!         .save(filename).expect("panic message");
//!    
//!     if Path::new(filename).exists() {
//!        fs::remove_file(&filename).unwrap_or_else(|_| panic!("Failed to remove {:?}", filename));
//!     }
//! }
//! ```

use crate::curves::Curve;
use crate::error::CurveError;
use crate::geometrics::{PlotBuilder, Plottable};
use crate::visualization::Graph;

/// Plottable implementation for single Curve
impl Plottable for Curve {
    type Error = CurveError;

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

/// Implementation of the `Plottable` trait for `Vec<Curve>`.
///
/// This implementation enables a vector of `Curve` instances to be plotted
/// using the `plot` method. The method creates a `PlotBuilder` instance, which
/// allows for flexible and configurable visualization of the curves.
///
/// # Overview
/// By implementing the `Plottable` trait, a vector of `Curve` objects gains the
/// ability to leverage plot-building functionality. The `plot` method clones the
/// data (to ensure immutability of the original input) and pairs it with
/// default plotting options (`PlotOptions`) for further configuration and
/// rendering.
///
/// The `PlotBuilder` struct, which is returned by this implementation, acts as a
/// pipeline for customizing and generating the final plot. Once the plot is fully
/// configured in terms of styling and layout, it can be saved to a file, rendered
/// in memory, or manipulated further depending on the builder's available methods.
///
/// # Method Details
/// - **`plot`**:
///   - Creates a `PlotBuilder` instance containing the data from the `Vec<Curve>`
///     and populates it with default plot options.
///   - Returns a configurable tool for building curve visualizations.
///
/// # Considerations
/// - This implementation assumes that it is appropriate to clone the data from
///   the vector of `Curve` instances. If the cloning behavior is expensive or not
///   necessary, further optimization may be required.
/// - `PlotOptions` default values provide a reasonable starting point, but
///   most real-world applications will override some of these values for more
///   customization.
///
/// # Example Behavior
/// A vector of `Curve` objects can be passed to the `plot` method to generate
/// a plot tailored to the desired styling and configuration. Methods available
/// on `PlotBuilder` can then be chained to adjust plot dimensions,
/// colors, titles, labels, and more.
///
/// # Returns
/// - A `PlotBuilder` instance configured with the cloned curve data (`self.clone()`)
///   and fully initialized with default `PlotOptions`.
///
/// # Default Settings
/// - The default `PlotOptions`, as used in this implementation, include:
///   - White background
///   - Line width of 2 pixels
///   - Default dimensions (800x600 pixels)
///   - No title or axis labels
///   - No default line colors
///
/// # Errors
/// - While creating a `PlotBuilder` instance does not directly raise errors, subsequent
///   operations (e.g., saving a plot or generating a view) may encounter runtime issues
///   related to file I/O, data validity, or plot rendering.
///
/// # See Also
/// - [`Plottable`]: The trait allowing
///   generalized plotting functionality.
/// - [`PlotBuilder`]: The plot generation
///   and configuration builder.
///
/// # Modules
/// Code related to this implementation exists within the
/// `crate::curves::visualization::plotters` module, and it works in conjunction with the
/// `Curve` struct, `PlotBuilder`, and `PlotOptions`. These modules provide the functionality
/// required to create, configure, and render curve plots.
impl Plottable for Vec<Curve> {
    type Error = CurveError;

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
mod tests {
    use super::*;
    use crate::curves::Point2D;
    use crate::geometrics::GeometricObject;
    use rust_decimal_macros::dec;
    #[cfg(feature = "plotly")]
    use {std::fs, std::path::Path, tracing::error};

    #[cfg(feature = "plotly")]
    fn cleanup_image(filename: &Path) {
        match fs::remove_file(filename) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File already gone - that's exactly what we wanted
            }
            Err(e) => {
                error!("Failed to remove {}: {}", filename.display(), e);
            }
        }
    }

    fn create_test_curves() -> (Curve, Curve, Curve) {
        // Create points first so they live long enough
        let p1_1 = Point2D::new(dec!(0.0), dec!(0.0));
        let p1_2 = Point2D::new(dec!(1.0), dec!(1.0));
        let p1_3 = Point2D::new(dec!(2.0), dec!(4.0));

        let p2_1 = Point2D::new(dec!(0.0), dec!(1.0));
        let p2_2 = Point2D::new(dec!(1.0), dec!(2.0));
        let p2_3 = Point2D::new(dec!(2.0), dec!(5.0));

        let p3_1 = Point2D::new(dec!(0.0), dec!(2.0));
        let p3_2 = Point2D::new(dec!(1.0), dec!(3.0));
        let p3_3 = Point2D::new(dec!(2.0), dec!(6.0));

        let points1 = vec![&p1_1, &p1_2, &p1_3];
        let points2 = vec![&p2_1, &p2_2, &p2_3];
        let points3 = vec![&p3_1, &p3_2, &p3_3];

        (
            Curve::from_vector(points1),
            Curve::from_vector(points2),
            Curve::from_vector(points3),
        )
    }

    #[test]
    fn test_single_curve_plot_bis() {
        let p1 = Point2D::new(dec!(0.0), dec!(0.0));
        let p2 = Point2D::new(dec!(1.0), dec!(1.0));
        let p3 = Point2D::new(dec!(2.0), dec!(4.0));
        let points = vec![&p1, &p2, &p3];
        let curve = Curve::from_vector(points);

        // Plot single curve
        curve
            .plot()
            .title("Test Curve")
            .x_label("X Axis")
            .y_label("Y Axis")
            .dimensions(800, 600);

        #[cfg(feature = "plotly")]
        {
            let file_path_html = "single_curve_test.html".as_ref();

            curve.write_html(file_path_html).unwrap();
            cleanup_image(file_path_html);

            #[cfg(feature = "static_export")]
            {
                let file_path_png = "single_curve_test.png".as_ref();
                // PNG rendering requires a headless browser, which may not be available in CI
                if curve.write_png(file_path_png).is_ok() {
                    cleanup_image(file_path_png);
                }
            }
        }
    }

    #[test]
    fn test_multiple_curves_plot() {
        let p1_1 = Point2D::new(dec!(0.0), dec!(0.0));
        let p1_2 = Point2D::new(dec!(1.0), dec!(1.0));
        let p1_3 = Point2D::new(dec!(2.0), dec!(4.0));

        let p2_1 = Point2D::new(dec!(0.0), dec!(1.0));
        let p2_2 = Point2D::new(dec!(1.0), dec!(2.0));
        let p2_3 = Point2D::new(dec!(2.0), dec!(5.0));

        let points1 = vec![&p1_1, &p1_2, &p1_3];
        let points2 = vec![&p2_1, &p2_2, &p2_3];

        let curve1 = Curve::from_vector(points1);
        let curve2 = Curve::from_vector(points2);

        let curve_vector = vec![curve1.clone(), curve2.clone()];

        // Plot multiple curves
        curve_vector
            .plot()
            .title("Multiple Curves")
            .x_label("X Axis")
            .y_label("Y Axis")
            .dimensions(800, 600);

        #[cfg(feature = "plotly")]
        {
            let file_path_html = "multiple_curves_test.html".as_ref();
            curve_vector.write_html(file_path_html).unwrap();
            cleanup_image(file_path_html);

            #[cfg(feature = "static_export")]
            {
                let file_path_png = "multiple_curves_test.png".as_ref();
                // PNG rendering requires a headless browser, which may not be available in CI
                if curve_vector.write_png(file_path_png).is_ok() {
                    cleanup_image(file_path_png);
                }
            }
        }
    }

    #[test]
    fn test_single_curve_plot() {
        let p1_1 = Point2D::new(dec!(0.0), dec!(0.0));
        let p1_2 = Point2D::new(dec!(1.0), dec!(1.0));
        let p1_3 = Point2D::new(dec!(2.0), dec!(4.0));

        let points = vec![&p1_1, &p1_2, &p1_3];
        let curve = Curve::from_vector(points);

        // Plot single curve
        curve
            .plot()
            .title("Test Curve")
            .x_label("X Axis")
            .y_label("Y Axis")
            .dimensions(800, 600);

        #[cfg(feature = "plotly")]
        {
            let file_path_html = "single_curve_test.html".as_ref();
            curve.write_html(file_path_html).unwrap();
            cleanup_image(file_path_html);

            #[cfg(feature = "static_export")]
            {
                let file_path_png = "single_curve_test.png".as_ref();
                // PNG rendering requires a headless browser, which may not be available in CI
                if curve.write_png(file_path_png).is_ok() {
                    cleanup_image(file_path_png);
                }
            }
        }
    }

    #[test]
    fn test_multiple_curves_plot_bis() {
        let (curve1, curve2, curve3) = create_test_curves();
        let curve_vector = vec![curve1.clone(), curve2.clone(), curve3.clone()];

        // Plot multiple curves
        curve_vector
            .plot()
            .title("Multiple Curves")
            .x_label("X Axis")
            .y_label("Y Axis")
            .dimensions(1000, 700);

        #[cfg(feature = "plotly")]
        {
            let file_path_html = "multiple_curves_test.html".as_ref();
            curve_vector.write_html(file_path_html).unwrap();
            cleanup_image(file_path_html);

            #[cfg(feature = "static_export")]
            {
                let file_path_png = "multiple_curves_test.png".as_ref();
                // PNG rendering requires a headless browser, which may not be available in CI
                if curve_vector.write_png(file_path_png).is_ok() {
                    cleanup_image(file_path_png);
                }
            }
        }
    }

    #[test]
    fn test_plot_with_custom_line_width() {
        let (curve1, curve2, _) = create_test_curves();
        let curve_vector = vec![curve1.clone(), curve2.clone()];

        // Custom line width
        curve_vector
            .plot()
            .title("Thick Line Curves")
            .dimensions(800, 600);

        #[cfg(feature = "plotly")]
        {
            let file_path_html = "thick_line_curves_test.html".as_ref();
            curve_vector.write_html(file_path_html).unwrap();
            cleanup_image(file_path_html);

            #[cfg(feature = "static_export")]
            {
                let file_path_png = "thick_line_curves_test.png".as_ref();
                // PNG rendering requires a headless browser, which may not be available in CI
                if curve_vector.write_png(file_path_png).is_ok() {
                    cleanup_image(file_path_png);
                }
            }
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
        let result: Result<(), CurveError> =
            Err(std::io::Error::other("Test error")).map_err(|e| CurveError::StdError {
                reason: e.to_string(),
            });

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurveError::StdError { reason } => {
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
            .x_label_formatter(&|v| format!("{v:.2}"))
            .y_label_formatter(&|v| format!("{v:.2}"))
            .x_desc("X-axis")
            .y_desc("Y-axis");
        assert_eq!(chart.x_desc, "X-axis");
        assert_eq!(chart.y_desc, "Y-axis");
    }

    #[test]
    fn test_draw_series_error() {
        let result: Result<(), CurveError> =
            Err("Draw error".to_string()).map_err(|e| CurveError::StdError { reason: e });

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurveError::StdError { reason } => {
                assert_eq!(reason, "Draw error");
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
