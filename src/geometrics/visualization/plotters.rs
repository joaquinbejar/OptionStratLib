/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/1/25
******************************************************************************/
use std::path::Path;
use crate::visualization::ColorScheme;

/// Plot configuration options for data visualization.
///
/// This structure provides comprehensive customization capabilities for plots,
/// including titles, axis labels, colors, dimensions, and visual styling.
/// It serves as the central configuration object for the plotting system.
///
/// Use with `PlotBuilder` to create customized visualizations that can be
/// rendered and saved to files through the `Plottable` trait implementation.
#[derive(Clone, Debug)]
pub struct PlotOptions {
    /// Optional plot title displayed at the top of the visualization
    pub title: Option<String>,
    /// Optional label for the X-axis
    pub x_label: Option<String>,
    /// Optional label for the Y-axis
    pub y_label: Option<String>,
    /// Optional label for the Z-axis (used in 3D plots)
    pub z_label: Option<String>,
    /// Optional collection of RGB colors to use for individual data series lines
    /// Each color in the vector corresponds to a different curve in the plot
    pub line_colors: Option<Vec<ColorScheme>>,
    /// Width of plotted lines in pixels
    pub line_width: u32,
    /// Background color of the entire plot area as an RGB value
    pub background_color: ColorScheme,
    /// Width of the plot in pixels
    pub width: u32,
    /// Height of the plot in pixels
    pub height: u32,
    /// Optional names for each curve/data series to be displayed in the legend
    pub curve_name: Option<Vec<String>>,
    /// Optional size of points in scatter plots, measured in pixels
    pub point_size: Option<u32>,
    /// Optional font size for axis labels and other text elements
    pub labels_size: Option<f64>,
}

#[allow(dead_code)]
impl PlotOptions {
    /// Default color palette for multiple curves
    pub(crate) fn default_colors() -> Vec<ColorScheme> {
        todo!()
    }
}

impl Default for PlotOptions {
    fn default() -> Self {
        PlotOptions {
            title: None,
            x_label: None,
            y_label: None,
            z_label: None,
            line_colors: None,
            line_width: 2,
            background_color: ColorScheme::White,
            width: 800,
            height: 600,
            curve_name: None,
            point_size: None,
            labels_size: None,
        }
    }
}

/// Trait for defining objects that can be visualized as plots.
///
/// The `Plottable` trait provides a standardized interface for types that can be
/// represented graphically. It enables visualization of data structures through
/// a fluent builder pattern, allowing for customizable plot creation.
///
/// Implementers of this trait can be visualized using the plotting system with
/// configurable options for appearance, labels, colors, and other visual attributes.
///
pub trait Plottable {
    /// The error type returned by plotting operations.
    ///
    /// This associated type allows implementers to define their specific
    /// error handling approach for plot generation and rendering.
    type Error;

    /// Creates a plot builder for configuring and generating visualizations.
    ///
    /// Returns a `PlotBuilder` instance that provides a fluent interface for
    /// customizing plot appearance and behavior before rendering.
    fn plot(&self) -> PlotBuilder<Self>
    where
        Self: Sized;
}
/// A builder for creating and configuring data visualizations.
///
/// `PlotBuilder` provides a fluent interface for customizing plots with various
/// styling and labeling options. It works with any type that implements the
/// `Plottable` trait, allowing for consistent visualization capabilities across
/// different data structures.
///
/// This builder is typically created via the `plot()` method on types that implement
/// the `Plottable` trait. After configuring the plot with the desired options,
/// it can be rendered and saved using the methods from `PlotBuilderExt`.
///
pub struct PlotBuilder<T: Plottable> {
    /// The data to be visualized in the plot.
    ///
    /// This field holds the instance of a type implementing the `Plottable` trait
    /// which contains the actual data points to be represented in the visualization.
    #[allow(dead_code)]
    pub(crate) data: T,

    /// Configuration settings that control the appearance and behavior of the plot.
    ///
    /// This includes visual styling like colors, dimensions, and line widths,
    /// as well as textual elements like titles and axis labels.
    /// See `PlotOptions` for the complete set of available configuration options.
    pub(crate) options: PlotOptions,
}

impl<T: Plottable> PlotBuilder<T> {
    /// Sets the title of the plot.
    ///
    /// This method configures the main title that appears at the top of the visualization.
    ///
    /// # Parameters
    /// * `title` - The text to display as the plot title
    ///
    /// # Returns
    /// The `PlotBuilder` instance with the updated title setting
    ///
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.options.title = Some(title.into());
        self
    }

    /// Sets the label for the x-axis.
    ///
    /// This method configures the descriptive text displayed along the horizontal axis.
    ///
    /// # Parameters
    /// * `label` - The text to display as the x-axis label
    ///
    /// # Returns
    /// The `PlotBuilder` instance with the updated x-axis label
    ///
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.options.x_label = Some(label.into());
        self
    }

    /// Sets the label for the y-axis.
    ///
    /// This method configures the descriptive text displayed along the vertical axis.
    ///
    /// # Parameters
    /// * `label` - The text to display as the y-axis label
    ///
    /// # Returns
    /// The `PlotBuilder` instance with the updated y-axis label
    ///
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.options.y_label = Some(label.into());
        self
    }

    /// Sets the label for the z-axis.
    ///
    /// This method configures the descriptive text displayed along the z-axis
    /// in three-dimensional plots.
    ///
    /// # Parameters
    /// * `label` - The text to display as the z-axis label
    ///
    /// # Returns
    /// The `PlotBuilder` instance with the updated z-axis label
    ///
    pub fn z_label(mut self, label: impl Into<String>) -> Self {
        self.options.z_label = Some(label.into());
        self
    }

    /// Sets the size of data points in scatter plots.
    ///
    /// This method configures the diameter of individual data points
    /// when rendering scatter plots.
    ///
    /// # Parameters
    /// * `size` - The size of points in pixels
    ///
    /// # Returns
    /// The `PlotBuilder` instance with the updated point size setting
    ///
    pub fn point_size(mut self, size: u32) -> Self {
        self.options.point_size = Some(size);
        self
    }

    /// Sets the font size for labels and text elements.
    ///
    /// This method configures the font size used for axis labels, titles,
    /// and other textual elements in the visualization.
    ///
    /// # Parameters
    /// * `size` - The font size as a floating point value
    ///
    /// # Returns
    /// The `PlotBuilder` instance with the updated label size setting
    ///
    pub fn label_size(mut self, size: f64) -> Self {
        self.options.labels_size = Some(size);
        self
    }

    /// Sets custom names for each data series/curve in the plot.
    ///
    /// This method configures the names displayed in the legend to identify
    /// different data series in the visualization.
    ///
    /// # Parameters
    /// * `label` - A vector of strings, each representing the name of a curve
    ///
    /// # Returns
    /// The `PlotBuilder` instance with the updated curve names
    ///
    pub fn curve_name(mut self, label: Vec<String>) -> Self {
        self.options.curve_name = Some(label);
        self
    }

    /// Sets the colors for data series lines.
    ///
    /// This method configures the colors used to render each data series or curve
    /// in the visualization.
    ///
    /// # Parameters
    /// * `colors` - A vector of RGB colors to use for the plot lines
    ///
    /// # Returns
    /// The `PlotBuilder` instance with the updated line colors
    ///
    pub fn line_colors(mut self, colors: Vec<ColorScheme>) -> Self {
        self.options.line_colors = Some(colors);
        self
    }

    /// Sets the width of plot lines.
    ///
    /// This method configures the thickness of lines used to render data series
    /// in the visualization.
    ///
    /// # Parameters
    /// * `width` - The width of lines in pixels
    ///
    /// # Returns
    /// The `PlotBuilder` instance with the updated line width setting
    ///
    pub fn line_width(mut self, width: u32) -> Self {
        self.options.line_width = width;
        self
    }

    /// Sets the overall dimensions of the plot.
    ///
    /// This method configures the width and height of the generated plot image.
    ///
    /// # Parameters
    /// * `width` - The width of the plot in pixels
    /// * `height` - The height of the plot in pixels
    ///
    /// # Returns
    /// The `PlotBuilder` instance with the updated dimensions
    ///
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.options.width = width;
        self.options.height = height;
        self
    }

    /// Saves the configured plot to a file.
    ///
    /// This method renders the plot with all configured options and writes
    /// the result to the specified file path.
    ///
    /// # Parameters
    /// * `path` - The file path where the plot should be saved
    ///
    /// # Returns
    /// A `Result` indicating success or containing an error if the save operation failed
    ///
    /// # Errors
    /// This method will return an error if the plot cannot be rendered or saved,
    /// with the specific error type determined by the `Plottable` implementation.
    ///
    pub fn save(self, path: impl AsRef<Path>) -> Result<(), T::Error>
    where
        Self: PlotBuilderExt<T>,
    {
        PlotBuilderExt::save(self, path)
    }
}

/// Extension methods for the plot building process.
///
/// This trait extends the `PlotBuilder` functionality to provide methods for
/// outputting and saving plots. It serves as the final step in the plot creation
/// pipeline after configuring visualization options.
///
/// `PlotBuilderExt` complements the builder pattern used in the plotting system by
/// providing output capabilities that work with any type implementing the `Plottable` trait.
/// This separation of concerns allows for a clean interface where plot configuration
/// and rendering/output are logically separated.
pub trait PlotBuilderExt<T: Plottable> {
    /// Saves the configured plot to a file at the specified path.
    ///
    /// This method renders the plot with all configured options and writes the
    /// resulting visualization to the given file path. The file format is determined
    /// by the path's extension (e.g., .png, .svg).
    ///
    /// # Arguments
    ///
    /// * `path` - The file path where the plot should be saved. Can be any type
    ///   that can be converted to a `Path`.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the plot was successfully saved
    /// * `Err(T::Error)` if an error occurred during rendering or saving
    ///
    fn save(self, path: impl AsRef<Path>) -> Result<(), T::Error>;
}

#[cfg(test)]
mod tests_plot_builder {
    use super::*;

    /// A mock Plottable implementation for testing
    struct MockPlottable;

    impl Plottable for MockPlottable {
        type Error = std::io::Error;

        fn plot(&self) -> PlotBuilder<Self> {
            PlotBuilder {
                data: MockPlottable,
                options: PlotOptions::default(),
            }
        }
    }

    /// Convenience function to create a test plot builder
    fn create_test_builder() -> PlotBuilder<MockPlottable> {
        MockPlottable.plot()
    }

    #[test]
    fn test_z_label_method() {
        // Test setting z_label with a string
        let builder = create_test_builder().z_label("Z Axis");

        assert_eq!(builder.options.z_label, Some("Z Axis".to_string()));
    }

    #[test]
    fn test_z_label_method_with_different_types() {
        // Test setting z_label with different string-like types
        let builder1 = create_test_builder().z_label(String::from("Z Axis"));
        let builder2 = create_test_builder().z_label("Z Axis");

        assert_eq!(builder1.options.z_label, Some("Z Axis".to_string()));
        assert_eq!(builder2.options.z_label, Some("Z Axis".to_string()));
    }

    #[test]
    fn test_point_size_method() {
        // Test setting point size
        let builder = create_test_builder().point_size(10);

        assert_eq!(builder.options.point_size, Some(10));
    }

    #[test]
    fn test_point_size_method_multiple_calls() {
        // Test that multiple calls override the previous value
        let builder = create_test_builder().point_size(5).point_size(15);

        assert_eq!(builder.options.point_size, Some(15));
    }

    #[test]
    fn test_label_size_method() {
        // Test setting label size
        let builder = create_test_builder().label_size(12.5);

        assert_eq!(builder.options.labels_size, Some(12.5));
    }

    #[test]
    fn test_label_size_method_multiple_calls() {
        // Test that multiple calls override the previous value
        let builder = create_test_builder().label_size(10.0).label_size(20.5);

        assert_eq!(builder.options.labels_size, Some(20.5));
    }

    #[test]
    fn test_chaining_multiple_methods() {
        // Test chaining multiple configuration methods
        let builder = create_test_builder()
            .z_label("Z Axis")
            .point_size(10)
            .label_size(12.5);

        assert_eq!(builder.options.z_label, Some("Z Axis".to_string()));
        assert_eq!(builder.options.point_size, Some(10));
        assert_eq!(builder.options.labels_size, Some(12.5));
    }
}
