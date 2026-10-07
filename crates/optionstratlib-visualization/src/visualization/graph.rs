//! The `Graph` contract: one trait on every feature surface.
//!
//! An implementor supplies [`Graph::graph_data`] and may override
//! [`Graph::graph_config`]; that is the whole contract, with or without a
//! backend. `plotly` adds the provided rendering methods (`to_plot`,
//! `write_html`, `show`, `render`, `to_interactive_html`) and
//! `static_export` adds `write_png` and `write_svg`. Features only add
//! provided methods, so an implementation never changes with them
//! (ADR-0002 section 4).

// Scoped allow: bulk migration of unchecked `[]` indexing to
// `.get().ok_or_else(..)` tracked as follow-ups to #341.
#![allow(clippy::indexing_slicing)]

use crate::visualization::{GraphConfig, GraphData};
#[cfg(feature = "plotly")]
use {
    crate::error::GraphError,
    crate::visualization::OutputType,
    crate::visualization::file::prepare_file_path,
    crate::visualization::{make_scatter, make_surface, pick_color},
    plotly::layout::Axis,
    plotly::{Layout, Plot, common},
};

#[cfg(feature = "static_export")]
use {plotly::plotly_static::ImageFormat, std::sync::Mutex, tracing::debug};

/// Serializes every static export in the process.
///
/// `Plot::write_image` builds a `StaticExporter` per call, which attaches to a
/// chromedriver already listening on the default port or spawns its own. The
/// exporter that spawned the driver SIGKILLs it on close, even while another
/// exporter that attached to it still has a browser session open; that
/// session's headless Chrome is then orphaned and never exits (#724). Holding
/// this lock across the whole export, retries included, makes every
/// spawn → session → close → stop cycle run alone.
#[cfg(feature = "static_export")]
static STATIC_EXPORT_LOCK: Mutex<()> = Mutex::new(());

/// Writes `plot` to `path` as a static image, retrying up to three times.
///
/// Holds [`STATIC_EXPORT_LOCK`] for the whole call so no two exports share a
/// chromedriver.
#[cfg(feature = "static_export")]
fn export_image(
    plot: &Plot,
    path: &std::path::Path,
    format: ImageFormat,
    width: u32,
    height: u32,
) -> Result<(), GraphError> {
    const MAX_ATTEMPTS: u32 = 3;

    prepare_file_path(path)?;
    let label = format.to_string().to_uppercase();
    debug!("Writing {label} to: {}", path.display());

    // The guarded value is `()`, so a panic in another export leaves nothing
    // inconsistent behind: recover the guard instead of failing every export
    // after it.
    let _guard = STATIC_EXPORT_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let mut last_error = String::new();
    for attempt in 1..=MAX_ATTEMPTS {
        debug!("{label} export attempt {attempt} of {MAX_ATTEMPTS}");
        match plot.write_image(path, format.clone(), width as usize, height as usize, 1.0) {
            Ok(()) => {
                debug!("Successfully wrote {label} to: {}", path.display());
                return Ok(());
            }
            Err(e) => {
                debug!("{label} export attempt {attempt} failed: {e}");
                last_error = e.to_string();
                if attempt < MAX_ATTEMPTS {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }

    Err(GraphError::Render(format!(
        "Failed to write {label} after {MAX_ATTEMPTS} attempts: {last_error} on path: {}",
        path.display()
    )))
}

/// Chart contract of every type the library can draw: strategies, option
/// chains' curves and surfaces, options, positions and simulations.
///
/// The required items are the same on every feature surface: implementors
/// supply `graph_data()` and may override `graph_config()`. The backend
/// features add provided renderers on top: `to_plot`, `write_html`, `show`,
/// `render` and `to_interactive_html` under `plotly`, and `write_png` and
/// `write_svg` under `static_export`. None of them needs to be implemented.
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
                if let Some(legend) = &cfg.legend
                    && let Some(label) = legend.first()
                {
                    series.name = label.clone();
                }
                plot.add_trace(make_scatter(&series));
            }
            GraphData::MultiSeries(list) => {
                for (idx, s) in list.into_iter().enumerate() {
                    let mut series = s;

                    if series.line_color.is_none() {
                        series.line_color = pick_color(&cfg, idx);
                    }

                    if let Some(legend) = &cfg.legend
                        && idx < legend.len()
                    {
                        series.name = legend[idx].clone();
                    }

                    plot.add_trace(make_scatter(&series));
                }
            }
            GraphData::GraphSurface(surf) => {
                let mut surface = surf.clone();
                if let Some(legend) = &cfg.legend
                    && let Some(label) = legend.first()
                {
                    surface.name = label.clone();
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
    /// # Concurrency
    ///
    /// Static exports run one at a time per process: concurrent calls wait on
    /// a shared lock so no two exports share a chromedriver (#724).
    ///
    #[cfg(feature = "static_export")]
    fn write_png(&self, path: &std::path::Path) -> Result<(), GraphError> {
        let cfg = self.graph_config();
        export_image(
            &self.to_plot(),
            path,
            ImageFormat::PNG,
            cfg.width,
            cfg.height,
        )
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
    /// # Concurrency
    ///
    /// Static exports run one at a time per process: concurrent calls wait on
    /// a shared lock so no two exports share a chromedriver (#724).
    ///
    #[cfg(feature = "static_export")]
    fn write_svg(&self, path: &std::path::Path) -> Result<(), GraphError> {
        let cfg = self.graph_config();
        export_image(
            &self.to_plot(),
            path,
            ImageFormat::SVG,
            cfg.width,
            cfg.height,
        )
    }

    /// Show the plot in browser
    ///
    /// # Errors
    ///
    /// Currently infallible (the underlying `plotly` `show` call does
    /// not return a `Result`); the `Result` signature is retained to
    /// allow future plot kernels that can surface
    /// `GraphError::Render` or `GraphError::Io` without
    /// a breaking change.
    #[cfg(feature = "plotly")]
    fn show(&self) -> Result<(), GraphError> {
        self.to_plot().show();
        Ok(())
    }

    /// One‑stop rendering with error propagation.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::Render` when the chosen `OutputType`
    /// backend (PNG/SVG via `static_export`, HTML, etc.) fails to
    /// serialize or render, or when a PNG or SVG is asked for in a build
    /// without `static_export`, and `GraphError::Io` when the
    /// destination path cannot be written.
    #[cfg(feature = "plotly")]
    fn render(&self, output: OutputType) -> Result<(), GraphError> {
        match output {
            #[cfg(feature = "static_export")]
            OutputType::Png(path) => {
                debug!("Rendering PNG to: {}", path.display());
                match self.write_png(path) {
                    Ok(_) => debug!("Successfully wrote PNG to: {}", path.display()),
                    Err(e) => return Err(GraphError::Render(format!("Failed to write PNG: {e}"))),
                }
            }
            #[cfg(feature = "static_export")]
            OutputType::Svg(path) => {
                debug!("Rendering SVG to: {}", path.display());
                match self.write_svg(path) {
                    Ok(_) => debug!("Successfully wrote SVG to: {}", path.display()),
                    Err(e) => return Err(GraphError::Render(format!("Failed to write SVG: {e}"))),
                }
            }
            #[cfg(not(feature = "static_export"))]
            OutputType::Png(_) | OutputType::Svg(_) => {
                return Err(static_export_disabled());
            }
            OutputType::Browser => self.show()?,
            OutputType::Html(path) => self.to_interactive_html(path)?,
        }
        Ok(())
    }

    /// Generate interactive HTML with hover info + annotations.
    ///
    /// # Errors
    ///
    /// Propagates any [`GraphError`] returned by
    /// `PlotlyChart::write_html`, typically
    /// `GraphError::Io` when the target file cannot be
    /// created or written.
    #[cfg(feature = "plotly")]
    fn to_interactive_html(&self, path: &std::path::Path) -> Result<(), GraphError> {
        self.write_html(path)
    }
}

/// The error `Graph::render` returns for a PNG or SVG target in a build
/// without the `static_export` feature, which is what writes images.
#[cfg(all(feature = "plotly", not(feature = "static_export")))]
#[cold]
#[inline(never)]
fn static_export_disabled() -> GraphError {
    GraphError::Render(
        "png and svg export need the `static_export` feature of optionstratlib-visualization"
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::{ColorScheme, LineStyle, Series2D};
    use rust_decimal_macros::dec;

    // A simple struct that implements the Graph trait for testing
    struct TestGraph {
        data: GraphData,
        config: Option<GraphConfig>,
    }

    impl TestGraph {
        fn new(data: GraphData) -> Self {
            TestGraph { data, config: None }
        }

        fn with_config(data: GraphData, config: GraphConfig) -> Self {
            TestGraph {
                data,
                config: Some(config),
            }
        }
    }

    impl Graph for TestGraph {
        fn graph_data(&self) -> GraphData {
            self.data.clone()
        }

        fn graph_config(&self) -> GraphConfig {
            match &self.config {
                Some(config) => config.clone(),
                None => GraphConfig::default(),
            }
        }
    }

    fn default_series() -> GraphData {
        GraphData::Series(Series2D {
            x: vec![dec!(1.0), dec!(2.0)],
            y: vec![dec!(3.0), dec!(4.0)],
            name: "Test Series".to_string(),
            mode: crate::visualization::TraceMode::Lines,
            line_color: None,
            line_width: None,
        })
    }

    #[test]
    fn test_graph_data() {
        // Create some test data
        let data = default_series();
        let graph = TestGraph::new(data.clone());

        // Verify that graph_data returns the expected data
        assert_eq!(graph.graph_data(), data);
    }

    #[test]
    fn test_default_graph_config() {
        let data = default_series();
        let graph = TestGraph::new(data);

        // Verify that the default config is returned when none is specified
        let default_config = GraphConfig::default();
        assert_eq!(graph.graph_config(), default_config);
    }

    #[test]
    fn test_custom_graph_config() {
        let data = default_series();

        // Create a custom config
        let custom_config = GraphConfig {
            title: "Custom Title".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X Axis".to_string()),
            y_label: Some("Y Axis".to_string()),
            z_label: None,
            line_style: LineStyle::Dashed,
            color_scheme: ColorScheme::Default,
            legend: Some(vec!["Series 1".to_string(), "Series 2".to_string()]),
            show_legend: true,
        };

        let graph = TestGraph::with_config(data, custom_config.clone());

        // Verify that the custom config is returned
        assert_eq!(graph.graph_config(), custom_config);
    }

    #[test]
    fn test_graph_config_fields() {
        let data = default_series();

        // Create a custom config with specific properties to test
        let custom_config = GraphConfig {
            title: "Test Chart".to_string(),
            width: 1024,
            height: 768,
            x_label: Some("Time".to_string()),
            y_label: Some("Value".to_string()),
            z_label: Some("Depth".to_string()),
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Viridis,
            legend: Some(vec!["Data A".to_string(), "Data B".to_string()]),
            show_legend: true,
        };

        let graph = TestGraph::with_config(data, custom_config);
        let config = graph.graph_config();

        // Test individual fields
        assert_eq!(config.title, "Test Chart");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.x_label, Some("Time".to_string()));
        assert_eq!(config.y_label, Some("Value".to_string()));
        assert_eq!(config.z_label, Some("Depth".to_string()));
        assert_eq!(config.line_style, LineStyle::Solid);
        assert_eq!(config.color_scheme, ColorScheme::Viridis);
        assert_eq!(
            config.legend,
            Some(vec!["Data A".to_string(), "Data B".to_string()])
        );
        assert!(config.show_legend);
    }

    #[cfg(all(feature = "plotly", not(feature = "static_export")))]
    #[test]
    fn test_render_png_and_svg_without_static_export_return_render_error() {
        let graph = TestGraph::new(default_series());
        let path = std::path::PathBuf::from("render_without_static_export.png");
        for output in [OutputType::Png(&path), OutputType::Svg(&path)] {
            match graph.render(output) {
                Err(GraphError::Render(message)) => assert!(message.contains("static_export")),
                other => panic!("expected GraphError::Render, got {other:?}"),
            }
        }
        assert!(!path.exists());
    }
}
