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
//! // let surface = /* Your surface creation logic here */;
//!
//! // Plot the surface and save to a file
//! surface.plot()
//!     .title("My Surface Plot")
//!     .x_label("X Axis")
//!     .y_label("Y Axis")
//!     .z_label("Z Axis") // Example of z-axis label
//!     .save("surface_plot.png")
//!     .expect("Failed to save plot");
//!  fs::remove_file("surface_plot.png").unwrap_or_else(|_| panic!("Failed to remove surface_plot.png"));
//! ```
//!

use crate::error::SurfaceError;
use crate::geometrics::{PlotBuilder, PlotBuilderExt, PlotOptions, Plottable};
use crate::surfaces::Surface;

use crate::visualization::utils::apply_shade;

use plotters::prelude::*;
use std::path::Path;

/// Plottable implementation for single Surface
impl Plottable for Surface {
    type Error = SurfaceError;

    fn plot(&self) -> PlotBuilder<Self>
    where
        Self: Sized,
    {
        PlotBuilder {
            data: self.clone(),
            options: PlotOptions::default(),
        }
    }
}

/// Implementation of the `Plottable` trait for `Vec<Surface>`.
///
/// This implementation enables a vector of `Surface` instances to be plotted
/// using the `plot` method. The method creates a `PlotBuilder` instance, which
/// allows for flexible and configurable visualization of the curves.
///
/// # Overview
/// By implementing the `Plottable` trait, a vector of `Surface` objects gains the
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
///   - Creates a `PlotBuilder` instance containing the data from the `Vec<Surface>`
///     and populates it with default plot options.
///   - Returns a configurable tool for building curve visualizations.
///
/// # Considerations
/// - This implementation assumes that it is appropriate to clone the data from
///   the vector of `Surface` instances. If the cloning behavior is expensive or not
///   necessary, further optimization may be required.
/// - `PlotOptions` default values provide a reasonable starting point, but
///   most real-world applications will override some of these values for more
///   customization.
///
/// # Example Behavior
/// A vector of `Surface` objects can be passed to the `plot` method to generate
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
/// `Surface` struct, `PlotBuilder`, and `PlotOptions`. These modules provide the functionality
/// required to create, configure, and render curve plots.
impl Plottable for Vec<Surface> {
    type Error = SurfaceError;

    fn plot(&self) -> PlotBuilder<Self>
    where
        Self: Sized,
    {
        PlotBuilder {
            data: self.clone(),
            options: PlotOptions::default(),
        }
    }
}

/// Plotting implementation for single Surface
impl PlotBuilderExt<Surface> for PlotBuilder<Surface> {
    fn save(self, path: impl AsRef<Path>) -> Result<(), SurfaceError> {
        // Convert points to f64
        let points: Vec<(f64, f64, f64)> = self.data.get_f64_points();
        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No points to plot".to_string(),
            ));
        }

        let label30: u32 = (self.options.labels_size.unwrap_or(1.0) * 30.0) as u32;
        let label20: u32 = (self.options.labels_size.unwrap_or(1.0) * 20.0) as u32;
        let label10: u32 = (self.options.labels_size.unwrap_or(1.0) * 10.0) as u32;

        // Determine plot range
        let x_min = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let x_max = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let y_min = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let y_max = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let z_min = points.iter().map(|p| p.2).fold(f64::INFINITY, f64::min);
        let z_max = points.iter().map(|p| p.2).fold(f64::NEG_INFINITY, f64::max);

        // Create drawing area
        let root = BitMapBackend::new(path.as_ref(), (self.options.width, self.options.height))
            .into_drawing_area();

        root.fill(&self.options.background_color)
            .map_err(|e| SurfaceError::StdError {
                reason: e.to_string(),
            })?;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                self.options.title.unwrap_or_default(),
                ("Arial", label30).into_font(),
            )
            .margin(5)
            .x_label_area_size(label30)
            .y_label_area_size(label30)
            .build_cartesian_3d(x_min..x_max, y_min..y_max, z_min..z_max)
            .map_err(|e| SurfaceError::StdError {
                reason: e.to_string(),
            })?;

        chart.with_projection(|mut pb| {
            pb.pitch = 0.3;
            pb.yaw = 0.5;
            pb.scale = 0.8;
            pb.into_matrix()
        });

        // Configure axes
        chart
            .configure_axes()
            .label_style(("Arial", label10))
            .draw()
            .map_err(|e| SurfaceError::StdError {
                reason: e.to_string(),
            })?;

        let point_size = self.options.point_size.unwrap_or(1);

        chart
            .draw_series(points.iter().map(|(x, y, z)| {
                // Clonar self.options.line_colors para evitar moverlo
                let line_colors = self.options.line_colors.clone();

                // Draw the surface as points with shading
                let base_color = apply_shade(
                    line_colors
                        .unwrap_or_else(|| vec![PlotOptions::default_colors()[0]])
                        .first()
                        .cloned()
                        .unwrap_or(RGBColor(0, 0, 255)),
                    *z / z_max,
                );

                Circle::new((*x, *y, *z), point_size, base_color.filled())
            }))
            .map_err(|e| SurfaceError::StdError {
                reason: e.to_string(),
            })?;

        root.draw(&Text::new(
            self.options.x_label.as_deref().unwrap_or("X"),
            (
                self.options.width as i32 / 4,
                self.options.height as i32 * 15 / 16,
            ),
            ("Arial", label20).into_font(),
        ))
        .map_err(|e| SurfaceError::StdError {
            reason: e.to_string(),
        })?;

        root.draw(&Text::new(
            self.options.z_label.as_deref().unwrap_or("Z"),
            (
                self.options.width as i32 / 10,
                self.options.height as i32 / 2,
            ),
            ("Arial", label20).into_font(),
        ))
        .map_err(|e| SurfaceError::StdError {
            reason: e.to_string(),
        })?;

        root.draw(&Text::new(
            self.options.y_label.as_deref().unwrap_or("Y"),
            (
                self.options.width as i32 * 3 / 4,
                self.options.height as i32 * 14 / 16,
            ),
            ("Arial", label20).into_font().color(&BLACK),
        ))
        .map_err(|e| SurfaceError::StdError {
            reason: e.to_string(),
        })?;

        root.present().map_err(|e| SurfaceError::StdError {
            reason: e.to_string(),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometrics::GeometricObject;
    use crate::surfaces::Point3D;
    use plotters::prelude::RGBColor;
    use rust_decimal_macros::dec;
    use std::fs;

    fn cleanup_image(filename: &str) {
        if Path::new(filename).exists() {
            fs::remove_file(filename).unwrap_or_else(|_| panic!("Failed to remove {}", filename));
        }
    }

    fn create_test_curves() -> (Surface, Surface, Surface) {
        // Create points first so they live long enough
        let p1_1 = Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0));
        let p1_2 = Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0));
        let p1_3 = Point3D::new(dec!(2.0), dec!(4.0), dec!(4.0));
        let p2_1 = Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0));
        let p2_2 = Point3D::new(dec!(1.0), dec!(2.0), dec!(2.0));
        let p2_3 = Point3D::new(dec!(2.0), dec!(5.0), dec!(5.0));
        let p3_1 = Point3D::new(dec!(0.0), dec!(2.0), dec!(2.0));
        let p3_2 = Point3D::new(dec!(1.0), dec!(3.0), dec!(3.0));
        let p3_3 = Point3D::new(dec!(2.0), dec!(6.0), dec!(6.0));

        let points1 = vec![&p1_1, &p1_2, &p1_3];
        let points2 = vec![&p2_1, &p2_2, &p2_3];
        let points3 = vec![&p3_1, &p3_2, &p3_3];

        (
            Surface::from_vector(points1),
            Surface::from_vector(points2),
            Surface::from_vector(points3),
        )
    }

    #[test]
    fn test_single_curve_plot_bis() {
        let p1 = Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0));
        let p2 = Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0));
        let p3 = Point3D::new(dec!(2.0), dec!(4.0), dec!(4.0));
        let points = vec![&p1, &p2, &p3];
        let curve = Surface::from_vector(points);

        // Plot single curve
        curve
            .plot()
            .title("Test Surface")
            .x_label("X Axis")
            .y_label("Y Axis")
            .dimensions(800, 600) // Añade dimensiones explícitas
            .save("single_curve_test.png")
            .expect("Single curve plot failed");
        cleanup_image("single_curve_test.png")
    }

    #[test]
    fn test_single_curve_plot() {
        let p1_1 = Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0));
        let p1_2 = Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0));
        let p1_3 = Point3D::new(dec!(2.0), dec!(4.0), dec!(4.0));

        let points = vec![&p1_1, &p1_2, &p1_3];
        let curve = Surface::from_vector(points);

        // Plot single curve
        curve
            .plot()
            .title("Test Surface")
            .x_label("X Axis")
            .y_label("Y Axis")
            .dimensions(800, 600)
            .save("single_curve_test.png")
            .expect("Single curve plot failed");
        cleanup_image("single_curve_test.png");
    }

    #[test]
    fn test_plot_options_defaults() {
        let curve = create_test_curves().0;
        let plot_builder = curve.plot();

        // Check default options
        assert_eq!(plot_builder.options.width, 800);
        assert_eq!(plot_builder.options.height, 600);
        assert_eq!(plot_builder.options.line_width, 2);
        assert_eq!(
            plot_builder.options.background_color,
            RGBColor(255, 255, 255)
        );
    }

    #[test]
    fn test_plot_with_extreme_points() {
        let extreme_points = vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1000.0), dec!(1000.0), dec!(1000.0)),
            Point3D::new(dec!(-500.0), dec!(-500.0), dec!(-500.0)),
        ];
        let curve = Surface::from_vector(extreme_points);

        // Plot curve with extreme points
        curve
            .plot()
            .title("Extreme Points Surface")
            .dimensions(800, 600)
            .save("extreme_points_curve_test.png")
            .expect("Extreme points curve plot failed");

        cleanup_image("extreme_points_curve_test.png");
    }

    #[test]
    fn test_plot_with_few_points() {
        let few_points = vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
        ];
        let curve = Surface::from_vector(few_points);

        // Plot curve with few points
        curve
            .plot()
            .title("Few Points Surface")
            .dimensions(800, 600)
            .save("few_points_curve_test.png")
            .expect("Few points curve plot failed");

        cleanup_image("few_points_curve_test.png");
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[derive(Debug, Clone)]
    struct Plot {
        options: PlotOptions,
    }

    impl Plottable for Plot {
        type Error = SurfaceError;

        fn plot(&self) -> PlotBuilder<Self>
        where
            Self: Sized,
        {
            PlotBuilder {
                data: self.clone(),
                options: PlotOptions::default(),
            }
        }
    }

    impl PlotBuilderExt<Plot> for PlotBuilder<Plot> {
        fn save(self, _path: impl AsRef<Path>) -> Result<(), SurfaceError> {
            Ok(())
        }
    }

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
    fn test_curve_name() {
        let options = PlotOptions {
            curve_name: None,
            ..Default::default()
        };
        let plot = Plot { options }.plot();
        let result = plot.curve_name(vec!["Test Surface".to_string()]);
        assert_eq!(
            result.options.curve_name,
            Some(vec!["Test Surface".to_string()])
        );
    }

    #[test]
    fn test_save_standard() {
        let plot = Plot {
            options: PlotOptions::default(),
        }
        .plot();
        let result = plot.save("test_path.png");
        assert!(result.is_ok());
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

    #[test]
    fn test_curve_label() {
        let options = PlotOptions {
            curve_name: Some(vec!["Test Surface".to_string()]),
            ..Default::default()
        };
        let plot = Plot { options };
        let label = match &plot.options.curve_name {
            Some(names) => names.first().map(|s| s.as_str()).unwrap_or("Default"),
            None => "Default",
        };
        assert_eq!(label, "Test Surface");
    }
}
