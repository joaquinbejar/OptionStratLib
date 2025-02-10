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
//! use std::path::Path;
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
//! let filename = "single_curve.png";
//! curve.plot()
//!     .title("Single Curve")
//!     .save(filename).expect("panic message");
//! if Path::new(filename).exists() {
//!    fs::remove_file(filename).unwrap_or_else(|_| panic!("Failed to remove {}", filename));
//! }
//! // Plot multiple curves
//! let curve1 = curve.clone();
//! let curve2 = curve.clone();
//! let curve3 = curve.clone();
//! let curves = vec![curve1, curve2, curve3];
//! let filename = "multiple_curves.png";
//! curves.plot()
//!     .title("Curve Comparison")
//!     .save(filename).expect("panic message");
//!
//! if Path::new(filename).exists() {
//!    fs::remove_file(filename).unwrap_or_else(|_| panic!("Failed to remove {}", filename));
//! }
//! ```

use crate::curves::Curve;
use crate::error::CurveError;
use crate::geometrics::{PlotBuilder, PlotBuilderExt, PlotOptions, Plottable};
#[cfg(not(target_arch = "wasm32"))]
use num_traits::ToPrimitive;
#[cfg(not(target_arch = "wasm32"))]
use plotters::prelude::*;
use std::path::Path;

/// Plottable implementation for single Curve
impl Plottable for Curve {
    type Error = CurveError;

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
            options: PlotOptions::default(),
        }
    }
}

/// Plotting implementation for single Curve
impl PlotBuilderExt<Curve> for PlotBuilder<Curve> {
    #[cfg(target_arch = "wasm32")]
    fn save(self, _path: impl AsRef<Path>) -> Result<(), CurveError> {
        // Do nothing in wasm
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save(self, path: impl AsRef<Path>) -> Result<(), CurveError> {
        // Convert points to f64
        let points: Vec<(f64, f64)> = self
            .data
            .points
            .iter()
            .map(|p| (p.x.to_f64().unwrap_or(0.0), p.y.to_f64().unwrap_or(0.0)))
            .collect();
        
        if points.is_empty() {
            return Err(CurveError::ConstructionError("No points to plot".to_string()));
        }

        // Determine plot range
        let x_min = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let x_max = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let y_min = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let y_max = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);

        // Create drawing area
        let root = BitMapBackend::new(path.as_ref(), (self.options.width, self.options.height))
            .into_drawing_area();

        root.fill(&self.options.background_color)
            .map_err(|e| CurveError::StdError {
                reason: e.to_string(),
            })?;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                self.options.title.unwrap_or_default(),
                ("Arial", 30).into_font(),
            )
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)
            .map_err(|e| CurveError::StdError {
                reason: e.to_string(),
            })?;

        // Configure axes
        chart
            .configure_mesh()
            .x_label_formatter(&|v| format!("{:.2}", v))
            .y_label_formatter(&|v| format!("{:.2}", v))
            .x_desc(self.options.x_label.as_deref().unwrap_or("X"))
            .y_desc(self.options.y_label.as_deref().unwrap_or("Y"))
            .draw()
            .map_err(|e| CurveError::StdError {
                reason: e.to_string(),
            })?;

        // Draw the curve
        let color = self
            .options
            .line_colors
            .unwrap_or_else(|| vec![PlotOptions::default_colors()[0]])
            .first()
            .cloned()
            .unwrap_or(RGBColor(0, 0, 255));

        chart
            .draw_series(LineSeries::new(
                points,
                color.stroke_width(self.options.line_width),
            ))
            .map_err(|e| CurveError::StdError {
                reason: e.to_string(),
            })?;

        root.present().map_err(|e| CurveError::StdError {
            reason: e.to_string(),
        })?;

        Ok(())
    }
}

/// Implementation of the `PlotBuilderExt` trait for `PlotBuilder<Vec<Curve>>`.
///
/// This implementation allows saving a plot to a file by utilizing the `plotters` library.
/// The `save` method takes a file path as input and generates a plot based on the data and
/// configuration options provided in the `PlotBuilder`.
///
/// # Functionality
/// - **Curve Points Preparation**: It iterates over the curve data (`data`) and transforms
///   the points into a collection of `(f64, f64)` tuples, which are compatible with the `plotters` library.
/// - **Plot Range Calculation**: Determines the plot's x and y axis ranges by collecting
///   minimum and maximum values across all the curve points.
/// - **Plot Rendering**: The method sets up a plot with custom title, axis labels, line colors,
///   line widths, and other visual properties defined in `PlotOptions`.
/// - **Curve Drawing**: Each curve is drawn using the `LineSeries` feature from `plotters`.
///   A unique color is assigned to each curve, repeated cyclically if the number of curves
///   exceeds the number of available colors in the palette.
/// - **Legend Display**: A legend is added to the plot using the series' labels.
/// - **Error Handling**: The method handles unexpected errors during chart creation, curve
///   rendering, or plot saving, by propagating them as `CurvesError` instances.
///
/// # Parameters
/// - **`self`**: The `PlotBuilder` instance containing the curve data (`data`) and configuration
///   options (`options`).
/// - **`path`**: A path to the file where the plot will be saved. This path can be provided as
///   any value that implements the `AsRef<Path>` trait.
///
/// # Return Value
/// - Returns `Ok(())` on success, indicating that the plot was saved successfully.
/// - Returns `Err(CurvesError)` on failure, encapsulating the failure reason as a string.
///
/// # Dependencies
/// - Uses the `plotters` library for rendering the plot.
/// - Leverages utility methods like `.fold()`, `.iter()`, and `.map()` to process curve data.
/// - Relies on `self.options` for plot customization (e.g., width, height, colors, etc.).
///
/// # Error Handling
/// Any errors encountered during the plot creation or file save process are encapsulated
/// as `CurvesError` with a `StdError` variant and a descriptive error message.
///
/// # Algorithm
/// 1. **Fetch Curve Points**: Convert the curves' `Point2D` instances to `(f64, f64)` tuples.
///    Use `to_f64` conversion for high precision.
/// 2. **Calculate Axis Ranges**: Find minimum (`x_min`, `y_min`) and maximum (`x_max`, `y_max`)
///    values for x and y axes across all curve points.
/// 3. **Set Up Plot**: Create the drawing area using `BitMapBackend` with the specified dimensions
///    and background color in `options`.
/// 4. **Configure Chart**: Use `ChartBuilder` to define margins, axis labels, and title.
/// 5. **Draw Axes**: Configure and draw the x and y axes with proper labels and formatting.
/// 6. **Draw Curves**: Iterate through the prepared curve points and draw each curve with a distinct color.
/// 7. **Add Legend**: Add a legend area showing the labels for each curve.
/// 8. **Save Plot**: Serialize and save the plot to the specified file path, returning any errors if encountered.
///
/// # Usage Considerations
/// - The `self.options.line_colors` must contain enough colors to accommodate all curves.
///   If fewer colors are specified, the colors will repeat cyclically.
/// - The `background_color` and `line_width` options affect the overall appearance.
/// - The success of the plot rendering depends on valid and well-formed curve data (`Vec<Curve>`).
///
/// # Examples of Dependencies
/// - **Associated Traits**: Must be used with the `PlotBuilder` struct and a compatible `Vec<Curve>` data type.
/// - **Color Palettes**: The `PlotOptions::default_colors` method provides a default color palette.
///
/// # Related Types
/// - **`PlotBuilder`**: Used to encapsulate curve data and configuration options.
/// - **`PlotOptions`**: Provides visual and layout customization for the plot.
/// - **`CurvesError`**: Represents errors that can occur while saving the plot.
///
/// # Remarks
/// - The method is tightly integrated with `plotters` and uses its core components
///   (`BitMapBackend`, `ChartBuilder`, `LineSeries`, etc.) for chart creation.
/// - The precision of `Point2D::x` and `Point2D::y` values is preserved by converting them from
///   `Decimal` to `f64` when plotting.
impl PlotBuilderExt<Vec<Curve>> for PlotBuilder<Vec<Curve>> {
    #[cfg(target_arch = "wasm32")]
    fn save(self, _path: impl AsRef<Path>) -> Result<(), CurveError> {
        // Do nothing in wasm
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save(self, path: impl AsRef<Path>) -> Result<(), CurveError> {
        // Prepare all curve points
        let all_curve_points: Vec<Vec<(f64, f64)>> = self
            .data
            .iter()
            .map(|curve| {
                curve
                    .points
                    .iter()
                    .map(|p| (p.x.to_f64().unwrap_or(0.0), p.y.to_f64().unwrap_or(0.0)))
                    .collect()
            })
            .collect();

        // Determine overall plot range
        let x_min = all_curve_points
            .iter()
            .flat_map(|points| points.iter().map(|p| p.0))
            .fold(f64::INFINITY, f64::min);
        let x_max = all_curve_points
            .iter()
            .flat_map(|points| points.iter().map(|p| p.0))
            .fold(f64::NEG_INFINITY, f64::max);
        let y_min = all_curve_points
            .iter()
            .flat_map(|points| points.iter().map(|p| p.1))
            .fold(f64::INFINITY, f64::min);
        let y_max = all_curve_points
            .iter()
            .flat_map(|points| points.iter().map(|p| p.1))
            .fold(f64::NEG_INFINITY, f64::max);

        // Create drawing area
        let root = BitMapBackend::new(path.as_ref(), (self.options.width, self.options.height))
            .into_drawing_area();

        root.fill(&self.options.background_color)
            .map_err(|e| CurveError::StdError {
                reason: e.to_string(),
            })?;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                self.options.title.unwrap_or_default(),
                ("Arial", 30).into_font(),
            )
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)
            .map_err(|e| CurveError::StdError {
                reason: e.to_string(),
            })?;

        // Configure axes
        chart
            .configure_mesh()
            .x_label_formatter(&|v| format!("{:.2}", v))
            .y_label_formatter(&|v| format!("{:.2}", v))
            .x_desc(self.options.x_label.as_deref().unwrap_or("X"))
            .y_desc(self.options.y_label.as_deref().unwrap_or("Y"))
            .draw()
            .map_err(|e| CurveError::StdError {
                reason: e.to_string(),
            })?;

        // Determine colors
        let colors: &Vec<RGBColor> = &self
            .options
            .line_colors
            .unwrap_or_else(PlotOptions::default_colors);

        // Draw curves
        for (i, points) in all_curve_points.into_iter().enumerate() {
            let default_name = &format!("Curve {}", i + 1);
            let label = match &self.options.curve_name {
                Some(names) => names.get(i).map(|s| s.as_str()).unwrap_or(default_name),
                None => default_name,
            };

            // Clone colors for this iteration
            let legend_color = colors[i % colors.len()];

            chart
                .draw_series(LineSeries::new(
                    points,
                    colors[i % colors.len()].stroke_width(self.options.line_width),
                ))
                .map_err(|e| CurveError::StdError {
                    reason: e.to_string(),
                })?
                .label(label)
                .legend(move |c| Circle::new(c, 3, legend_color.filled()));
        }

        // Add legend
        chart
            .configure_series_labels()
            .border_style(BLACK)
            .draw()
            .map_err(|e| CurveError::StdError {
                reason: e.to_string(),
            })?;

        root.present().map_err(|e| CurveError::StdError {
            reason: e.to_string(),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curves::Point2D;
    use crate::geometrics::GeometricObject;
    use plotters::prelude::RGBColor;
    use rust_decimal_macros::dec;
    use std::fs;

    fn cleanup_image(filename: &str) {
        if Path::new(filename).exists() {
            fs::remove_file(filename).unwrap_or_else(|_| panic!("Failed to remove {}", filename));
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
            .dimensions(800, 600) // Añade dimensiones explícitas
            .save("single_curve_test.png")
            .expect("Single curve plot failed");
        cleanup_image("single_curve_test.png")
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

        // Plot multiple curves
        vec![curve1, curve2]
            .plot()
            .title("Multiple Curves")
            .x_label("X Axis")
            .y_label("Y Axis")
            .dimensions(800, 600) // Añade dimensiones explícitas
            .save("multiple_curves_test.png")
            .expect("Multiple curves plot failed");

        cleanup_image("multiple_curves_test.png");
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
            .dimensions(800, 600)
            .save("single_curve_test.png")
            .expect("Single curve plot failed");
        cleanup_image("single_curve_test.png");
    }

    #[test]
    fn test_multiple_curves_plot_bis() {
        let (curve1, curve2, curve3) = create_test_curves();

        // Plot multiple curves
        vec![curve1, curve2, curve3]
            .plot()
            .title("Multiple Curves")
            .x_label("X Axis")
            .y_label("Y Axis")
            .dimensions(1000, 700)
            .save("multiple_curves_test.png")
            .expect("Multiple curves plot failed");

        cleanup_image("multiple_curves_test.png");
    }

    #[test]
    fn test_plot_with_custom_colors() {
        let (curve1, curve2, _) = create_test_curves();

        // Custom color plot
        vec![curve1, curve2]
            .plot()
            .title("Colored Curves")
            .line_colors(vec![
                RGBColor(255, 0, 0), // Red
                RGBColor(0, 0, 255), // Blue
            ])
            .dimensions(800, 600)
            .save("colored_curves_test.png")
            .expect("Colored curves plot failed");
        cleanup_image("colored_curves_test.png");
    }

    #[test]
    fn test_plot_with_custom_line_width() {
        let (curve1, curve2, _) = create_test_curves();

        // Custom line width
        vec![curve1, curve2]
            .plot()
            .title("Thick Line Curves")
            .line_width(4) // Thicker lines
            .dimensions(800, 600)
            .save("thick_line_curves_test.png")
            .expect("Thick line curves plot failed");
        cleanup_image("thick_line_curves_test.png");
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
    fn test_plot_builder_chaining() {
        let (curve1, curve2, _) = create_test_curves();

        // Test method chaining
        vec![curve1, curve2]
            .plot()
            .title("Chained Curves")
            .x_label("Custom X")
            .y_label("Custom Y")
            .line_width(3)
            .dimensions(1200, 800)
            .save("chained_curves_test.png")
            .expect("Chained curves plot failed");
        cleanup_image("chained_curves_test.png");
    }

    #[test]
    fn test_plot_with_extreme_points() {
        let extreme_points = vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1000.0), dec!(1000.0)),
            Point2D::new(dec!(-500.0), dec!(-500.0)),
        ];
        let curve = Curve::from_vector(extreme_points);

        // Plot curve with extreme points
        curve
            .plot()
            .title("Extreme Points Curve")
            .dimensions(800, 600)
            .save("extreme_points_curve_test.png")
            .expect("Extreme points curve plot failed");

        cleanup_image("extreme_points_curve_test.png");
    }

    #[test]
    fn test_plot_with_few_points() {
        let few_points = vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ];
        let curve = Curve::from_vector(few_points);

        // Plot curve with few points
        curve
            .plot()
            .title("Few Points Curve")
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
        type Error = CurveError;

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
        #[cfg(target_arch = "wasm32")]
        fn save(self, _path: impl AsRef<Path>) -> Result<(), CurveError> {
            Ok(())
        }

        #[cfg(not(target_arch = "wasm32"))]
        fn save(self, _path: impl AsRef<Path>) -> Result<(), CurveError> {
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
        let result = plot.curve_name(vec!["Test Curve".to_string()]);
        assert_eq!(
            result.options.curve_name,
            Some(vec!["Test Curve".to_string()])
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
    #[cfg(target_arch = "wasm32")]
    fn test_save_wasm() {
        let plot = Plot {
            options: PlotOptions::default(),
        }
        .plot();
        let result = plot.save("test_path.png");
        assert!(result.is_ok());
    }

    #[test]
    fn test_map_err_to_std_error() {
        let result: Result<(), CurveError> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Test error")).map_err(|e| {
                CurveError::StdError {
                    reason: e.to_string(),
                }
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
            .x_label_formatter(&|v| format!("{:.2}", v))
            .y_label_formatter(&|v| format!("{:.2}", v))
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

    #[test]
    fn test_curve_label() {
        let options = PlotOptions {
            curve_name: Some(vec!["Test Curve".to_string()]),
            ..Default::default()
        };
        let plot = Plot { options };
        let label = match &plot.options.curve_name {
            Some(names) => names.first().map(|s| s.as_str()).unwrap_or("Default"),
            None => "Default",
        };
        assert_eq!(label, "Test Curve");
    }
}
