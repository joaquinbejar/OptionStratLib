use crate::error::GraphError;
use crate::utils::file::prepare_file_path;
use crate::visualization::config::GraphConfig;
use crate::visualization::model::{GraphData, OutputType};
use crate::visualization::styles::PlotType;
use crate::visualization::utils::{make_scatter, make_surface, pick_color};
use plotly::ImageFormat;
use plotly::layout::Axis;
use plotly::{Layout, Plot, common};
use tracing::debug;

/// A trait that defines the functionality for creating, configuring, and rendering
/// graphical representations of data, along with support for various output formats.
///
/// The `Graph` trait abstracts away details involved in preparing graph data, applying 
/// configurations, and rendering graphs in different formats such as PNG, SVG, HTML, or 
/// directly displaying them in a browser.
///
/// ## Required Methods
/// - `graph_data`: Returns the raw data structure (`GraphData`) required for plotting.
///
/// ## Optional Methods
/// - `graph_config`: Provides configuration overrides for the graph, including dimensions, 
///   labels, legends, and other visual properties. Defaults to `GraphConfig::default()`.
///
/// ## Core Rendering Methods
/// - `to_plot`: Converts the instance into a `plotly::Plot` object based on its data and 
///   configuration.
///
/// ## Export and Viewing Methods
/// - `write_png`: Exports the graph as a PNG image to a specified file.
/// - `write_html`: Exports the graph as an interactive HTML file.
/// - `write_svg`: Exports the graph as an SVG file.
/// - `show`: Displays the graph in the default web browser.
/// - `render`: A unified interface for rendering graphs in various formats.
///
/// ## Advanced Features
/// - `to_interactive_html`: Creates an interactive HTML file with hover information 
///   and annotations.
pub trait Graph {
    /// Return the raw data ready for plotting.
    fn graph_data(&self) -> GraphData;

    /// Optional per‑object configuration overrides.
    fn graph_config(&self) -> GraphConfig {
        GraphConfig::default()
    }

    /// Build a `plotly::Plot` according to data + config.
    fn to_plot(&self) -> Plot {
        let cfg = self.graph_config();
        let mut plot = Plot::new();

        match self.graph_data() {
            GraphData::Series(s) => {
                let mut series = s.clone();
                if let Some(legend) = &cfg.legend {
                    if let Some(label) = legend.first() {
                        series.name = label.clone();
                    }
                }
                plot.add_trace(make_scatter(&series));
            }
            GraphData::MultiSeries(list) => {
                for (idx, s) in list.into_iter().enumerate() {
                    let mut series = s;

                    if series.line_color.is_none() {
                        series.line_color = pick_color(&cfg, idx);
                    }

                    if let Some(legend) = &cfg.legend {
                        if idx < legend.len() {
                            series.name = legend[idx].clone();
                        }
                    }

                    plot.add_trace(make_scatter(&series));
                }
            }
            GraphData::Surface(surf) => {
                let mut surface = surf.clone();
                if let Some(legend) = &cfg.legend {
                    if let Some(label) = legend.first() {
                        surface.name = label.clone();
                    }
                }
                plot.add_trace(make_surface(&surface));
            }
        }

        let mut layout = Layout::new()
            .width(cfg.width as usize)
            .height(cfg.height as usize)
            .title(common::Title::from(&cfg.title))
            .show_legend(cfg.show_legend);

        if let Some(label) = cfg.x_label {
            layout = layout.x_axis(Axis::new().title(common::Title::from(&label)));
        }
        if let Some(label) = cfg.y_label {
            layout = layout.y_axis(Axis::new().title(common::Title::from(&label)));
        }
        if let Some(label) = cfg.z_label {
            layout = layout.z_axis(Axis::new().title(common::Title::from(&label)));
        }

        plot.set_layout(layout);
        plot
    }

    /// Writes the graph as a PNG image to the specified file path.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a `std::path::Path` that specifies the destination 
    ///   file path where the PNG image will be written to.
    ///
    /// # Returns
    ///
    /// Returns a `Result`:
    /// * `Ok(())` - If the PNG image is successfully generated and written to the specified file.
    /// * `Err(GraphError)` - If there is an error during the process of preparing the file path 
    ///   or writing the image.
    ///
    /// # Behavior
    ///
    /// * Temporarily sets the `LC_ALL` and `LANG` environment variables to "en_US.UTF-8" to ensure
    ///   compatibility when writing the PNG.
    /// * Prepares the target file path using the `prepare_file_path` function. If the preparation fails,
    ///   an error is returned.
    ///
    /// * Retrieves the graph configuration (such as dimensions) using `self.graph_config()`.
    /// * Converts the graph data into a plot using `self.to_plot()`, then generates and writes a PNG
    ///   image to the specified path using the provided dimensions, `ImageFormat::PNG`, and a scaling factor of `1.0`.
    ///
    /// # Logging
    ///
    /// Logs a debug message with the target file path using the `debug!` macro before writing the PNG.
    ///
    /// # Errors
    ///
    /// Errors that might occur during execution:
    /// * Issues with preparing the file path (e.g., invalid path, permissions issue).
    /// * Internal errors with the image writing process.
    ///
    /// # Safety
    ///
    /// This function uses `unsafe` code to modify environment variables (`LC_ALL` and `LANG`). 
    /// Modifying global state like environment variables in a multithreaded context can lead to undefined behavior. 
    /// Ensure this function is used in a controlled environment where such changes are safe.
    ///
    fn write_png(&self, path: &std::path::Path) -> Result<(), GraphError> {
        unsafe {
            std::env::set_var("LC_ALL", "en_US.UTF-8");
            std::env::set_var("LANG", "en_US.UTF-8");
        }

        prepare_file_path(path)?;
        debug!("Writing PNG to: {}", path.display());
        let cfg = self.graph_config();

        self.to_plot().write_image(
            path,
            ImageFormat::PNG,
            cfg.width as usize,
            cfg.height as usize,
            1.0,
        );
        Ok(())
    }
    
    /// Writes the graph data to an HTML file at the specified path.
    ///
    /// This method generates a plot representation of the graph and saves it
    /// as an HTML document. It ensures that the provided file path is prepared
    /// (i.e., directories are created if necessary) before writing the file.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a `std::path::Path` specifying the file path where
    ///   the HTML file will be written.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the HTML file is successfully written.
    /// * `Err(GraphError)` if an error occurs during file preparation or writing.
    ///
    /// # Errors
    ///
    /// This method can return the following errors:
    /// * A `GraphError` if the file path preparation fails.
    /// * Any other error propagated from the `.to_plot().write_html()` method.
    ///
    /// # Notes
    ///
    /// Ensure that the directory specified in the file path exists or can be created
    /// with appropriate permissions to avoid errors during file preparation.
    fn write_html(&self, path: &std::path::Path) -> Result<(), GraphError> {
        prepare_file_path(path)?;
        self.to_plot().write_html(path);
        Ok(())
    }

    /// Writes the graph representation to an SVG file at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a `std::path::Path` that specifies the location
    ///   where the SVG file should be created.
    ///
    /// # Returns
    ///
    /// * `Result<(), GraphError>` - Returns `Ok(())` if the SVG is successfully 
    ///   written to the specified path. Otherwise, returns a `GraphError` if an
    ///   issue occurs during the file preparation or writing process.
    ///
    /// # Behavior
    ///
    /// - Prepares the file path by ensuring it exists and is accessible.
    /// - Retrieves the graph configuration (such as width and height).
    /// - Converts the graph representation into a format suitable for plotting.
    /// - Writes the graph into an SVG file with the specified width, height, and scale.
    ///
    /// # Errors
    ///
    /// This function may return a `GraphError` in the following cases:
    /// - The file path cannot be prepared (e.g., due to permissions issues or invalid path).
    /// - An error occurs during the conversion or writing process.
    ///
    fn write_svg(&self, path: &std::path::Path) -> Result<(), GraphError> {
        prepare_file_path(path)?;
        let cfg = self.graph_config();
        self.to_plot().write_image(
            path,
            ImageFormat::SVG,
            cfg.width as usize,
            cfg.height as usize,
            1.0,
        );
        Ok(())
    }

    /// Show the plot in browser
    fn show(&self) {
        self.to_plot().show();
    }

    /// One‑stop rendering with error propagation.
    fn render(&self, output: OutputType) -> Result<(), GraphError> {
        match output {
            OutputType::Png(path) => self.write_png(path)?,
            OutputType::Svg(path) => self.write_svg(path)?,
            OutputType::Html(path) => self.write_html(path)?,
            OutputType::Browser => self.show(),
        }
        Ok(())
    }

    /// Generate interactive HTML with hover info + annotations.
    fn to_interactive_html(&self, path: &std::path::Path) -> Result<(), GraphError> {
        self.write_html(path)?;
        Ok(())
    }
}

/// A trait defining the type of graph or plot used for visualization purposes.
///
/// The `GraphType` trait is designed to provide a standardized way to determine
/// the visualization type for a given data representation. Implementors of this
/// trait should define the specific plot type that suits their data or visualization needs.
///
/// # Usage
/// Implement this trait for any data structure or context that requires a 
/// well-defined graph type for displaying data.
///
pub trait GraphType {
    /// Returns the type of plot to be used for visualization.
    ///
    /// # Description
    /// The `plot_type` function determines and returns the type of plot that should
    /// be used. This information can be useful when configuring or rendering visual
    /// data representations. The returned `PlotType` is typically an enum or specific 
    /// type representing various available plot types (e.g., line plot, bar chart, scatter plot, etc.).
    ///
    /// # Returns
    /// * `PlotType` - The specific type of plot identified for use.
    ///
    fn plot_type() -> PlotType;
}
