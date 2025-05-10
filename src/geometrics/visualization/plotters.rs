/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/1/25
******************************************************************************/
use crate::visualization::{ColorScheme, Graph, GraphConfig, GraphData, LineStyle};
#[cfg(feature = "kaleido")]
use {crate::error::GraphError, std::path::Path};

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
        Self: Sized + Graph;
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
pub struct PlotBuilder<T: Plottable + Graph> {
    /// The data to be visualized in the plot.
    ///
    /// This field holds the instance of a type implementing the `Plottable` trait
    /// which contains the actual data points to be represented in the visualization.
    pub(crate) data: T,

    /// Configuration settings that control the appearance and behavior of the plot.
    ///
    /// This includes visual styling like colors, dimensions, and line widths,
    /// as well as textual elements like titles and axis labels.
    /// See `PlotOptions` for the complete set of available configuration options.
    pub(crate) options: GraphConfig,
}

impl<T: Plottable + Graph> PlotBuilder<T> {
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
        self.options.title = title.into();
        if self.options.legend.is_none() {
            self.options.legend = Some(vec![self.options.title.clone()]);
        }
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
    pub fn line_style(mut self, line_style: LineStyle) -> Self {
        self.options.line_style = line_style;
        self
    }

    /// Sets the legend for the current instance.
    ///
    /// This method accepts a vector of items that can be converted into strings and sets the `legend`
    /// field of the current instance's options. It returns the modified instance for further chaining.
    ///
    /// # Arguments
    ///
    /// * `legend` - A `Vec` containing items that implement the `Into<String>` trait. Each element
    ///   will be converted into a `String` and assigned to the legend.
    ///
    /// # Returns
    ///
    /// Returns the modified instance with the legend updated.
    ///
    /// In this example, the legend is updated to include "Item 1", "Item 2", and "Item 3".
    ///
    pub fn legend(mut self, legend: Vec<impl Into<String>>) -> Self {
        let legend: Vec<String> = legend.into_iter().map(|l| l.into()).collect();
        self.options.legend = Some(legend);
        self
    }

    /// Adds a legend entry to the existing legend configuration or initializes a new legend
    /// with the provided entry if none exists.
    ///
    /// This method allows chaining, modifying the `legend` configuration within the `options`
    /// of the current object. If a `legend` already exists, the provided legend entry will
    /// be appended to it. If no `legend` exists, a new legend will be created containing the
    /// provided entry.
    ///
    /// # Arguments
    ///
    /// * `legend` - An item implementing `Into<String>` that represents the legend entry
    ///   to add.
    ///
    /// # Returns
    ///
    /// Returns `Self` (the modified object) to allow method chaining.
    ///
    /// In this example, two legend entries, `"Legend 1"` and `"Legend 2"`, are added to the chart's
    /// legend configuration.
    pub fn add_legend(mut self, legend: impl Into<String>) -> Self {
        if let Some(ref mut legends) = self.options.legend {
            legends.push(legend.into());
        } else {
            self.options.legend = Some(vec![legend.into()]);
        }
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

    /// Sets the color scheme for the current instance.
    ///
    /// This method allows you to specify a `ColorScheme` to customize the appearance or theme
    /// of the associated object. The method updates the `color_scheme` field in the `options`
    /// struct with the provided value and returns an updated instance of `self`.
    ///
    /// # Parameters
    /// - `color_scheme`: The desired `ColorScheme` to be applied. This defines the visual style
    ///   or theme to be used by the object.
    ///
    /// # Returns
    /// An updated instance of `Self` with the new color scheme applied.
    ///
    pub fn color_scheme(mut self, color_scheme: ColorScheme) -> Self {
        self.options.color_scheme = color_scheme;
        self
    }

    /// Sets the visibility of the legend in the chart or visualization.
    ///
    /// # Parameters
    /// - `show_legend`: A boolean specifying whether the legend should be displayed.
    ///   - `true`: The legend will be displayed.
    ///   - `false`: The legend will be hidden.
    ///
    /// # Returns
    /// - Returns an updated instance of `Self` with the `show_legend` option set
    ///   according to the provided parameter.
    ///
    pub fn show_legend(mut self, show_legend: bool) -> Self {
        self.options.show_legend = show_legend;
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
    #[cfg(feature = "kaleido")]
    pub fn save(self, path: impl AsRef<Path>) -> Result<(), GraphError> {
        let path = path.as_ref();
        self.write_png(path).map_err(|e| {
            GraphError::Render(format!("Failed to save plot to {} {}", path.display(), e))
        })
    }
}

impl<T: Plottable + Graph> Graph for PlotBuilder<T> {
    fn graph_data(&self) -> GraphData {
        self.data.graph_data()
    }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: self.options.title.clone(),
            width: 1600,
            height: 900,
            x_label: self.options.x_label.clone(),
            y_label: self.options.y_label.clone(),
            z_label: self.options.z_label.clone(),
            line_style: self.options.line_style,
            color_scheme: self.options.color_scheme.clone(),
            legend: self.options.legend.clone(),
            show_legend: self.options.show_legend,
        }
    }
}
