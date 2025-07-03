use crate::error::GraphError;
use crate::utils::file::prepare_file_path;
use crate::visualization::{GraphConfig, GraphData, make_scatter, make_surface, pick_color};
use plotly::layout::Axis;
use plotly::{Layout, Plot, common};

#[cfg(feature = "kaleido")]
use {crate::visualization::OutputType, plotly::ImageFormat, tracing::debug};

pub trait Graph {
    /// Return the raw data ready for plotting.
    fn graph_data(&self) -> GraphData;

    /// Optional per‑object configuration overrides.
    fn graph_config(&self) -> GraphConfig {
        GraphConfig::default()
    }

    /// Build a `plotly::Plot` according to data + config.
    #[cfg(feature = "plotly")]
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
    #[cfg(feature = "kaleido")]
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
    #[cfg(feature = "plotly")]
    fn write_html(&self, path: &std::path::Path) -> Result<(), GraphError> {
        prepare_file_path(path)?;

        // Create a plot with the graph data
        let plot = self.to_plot();

        // Get the plot configuration
        let cfg = self.graph_config();

        // Get the JSON representation of the plot
        let plot_json = plot.to_json();

        // Create a complete HTML document with embedded Plotly.js
        let html = format!(
            "\
<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"utf-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
    <title>{}</title>
    <script src=\"https://cdn.plot.ly/plotly-2.24.1.min.js\" charset=\"utf-8\"></script>
    <style>
        body {{ margin: 0; padding: 20px; font-family: Arial, sans-serif; }}
        #plotly-graph {{ width: 100%; height: 600px; }}
    </style>
</head>
<body>
    <div id=\"plotly-graph\"></div>
    <script>
        var plotJson = {};
        Plotly.newPlot('plotly-graph', plotJson);
    </script>
</body>
</html>",
            cfg.title, plot_json
        );

        // Write HTML content to file
        std::fs::write(path, html)
            .map_err(|e| GraphError::Render(format!("Failed to write HTML file: {e}")))?;

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
    #[cfg(feature = "kaleido")]
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
    #[cfg(feature = "kaleido")]
    fn show(&self) {
        self.to_plot().show();
    }

    /// One‑stop rendering with error propagation.
    #[cfg(feature = "kaleido")]
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
    #[cfg(feature = "kaleido")]
    fn to_interactive_html(&self, path: &std::path::Path) -> Result<(), GraphError> {
        self.write_html(path)?;
        Ok(())
    }
}
