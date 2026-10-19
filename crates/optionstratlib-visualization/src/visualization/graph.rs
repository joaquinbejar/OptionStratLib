//! The `Graph` contract: one trait on every feature surface.
//!
//! An implementor supplies [`Graph::graph_data`] and may override
//! [`Graph::graph_config`]; that is the whole contract, with or without a
//! backend. `plotly` adds the provided rendering methods (`to_plot`,
//! `write_html`, `show`, `render`, `to_interactive_html`) and
//! `static_export` adds `write_png` and `write_svg`. Features only add
//! provided methods, so an implementation never changes with them
//! (ADR-0002 section 4).

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
use crate::visualization::native;

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

                    if let Some(label) = cfg.legend.as_ref().and_then(|legend| legend.get(idx)) {
                        series.name = label.clone();
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

    /// Writes the chart to `path` as a PNG of `graph_config().width` x
    /// `graph_config().height` pixels, creating the parent directory when it
    /// is missing.
    ///
    /// The image is drawn natively from [`Graph::graph_data`] and
    /// [`Graph::graph_config`]: no browser, WebDriver, network or installed
    /// font is involved, and concurrent exports, in threads or processes,
    /// are independent. Since 0.23 it no longer goes through
    /// [`Graph::to_plot`], so an override of `to_plot` changes the HTML
    /// outputs but not this image.
    ///
    /// # Errors
    ///
    /// * [`GraphError::Io`] when the file or its directory cannot be written
    ///   or the image cannot be encoded into it.
    /// * [`GraphError::InvalidDimensions`] for a width or height of zero or
    ///   above 16384 pixels; [`GraphError::TooSmall`] or
    ///   [`GraphError::NoRoom`] when the figure cannot hold the chart.
    /// * [`GraphError::LengthMismatch`], [`GraphError::SurfaceLengthMismatch`],
    ///   [`GraphError::DegenerateSurface`], [`GraphError::DuplicateSurfacePoint`],
    ///   [`GraphError::IncompleteSurface`] or [`GraphError::NotRepresentable`]
    ///   for data it cannot draw without inventing or dropping values.
    /// * [`GraphError::InvalidLineWidth`] or [`GraphError::InvalidColor`] for
    ///   a series style it cannot reproduce.
    /// * [`GraphError::MissingGlyph`] or [`GraphError::UnsupportedText`] for
    ///   a label the bundled font cannot draw.
    /// * [`GraphError::Render`] when drawing fails.
    #[cfg(feature = "static_export")]
    fn write_png(&self, path: &std::path::Path) -> Result<(), GraphError> {
        native::write_png(&self.graph_data(), &self.graph_config(), path)
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

    /// Writes the chart to `path` as an SVG of `graph_config().width` x
    /// `graph_config().height` user units, creating the parent directory
    /// when it is missing.
    ///
    /// Drawn natively like [`Graph::write_png`], with real vector shapes.
    /// Every label is drawn as glyph outlines of the bundled Open Sans, so
    /// the file looks the same in every viewer and needs no font. Each label
    /// keeps its text as an accessible name (`<g role="img" aria-label>`)
    /// and the document carries a `<title>` and `<desc>`; the text can no
    /// longer be selected, searched or edited as text.
    ///
    /// # Errors
    ///
    /// Everything [`Graph::write_png`] returns, and
    /// [`GraphError::NotXmlText`] for a label, title or series name that
    /// contains a character XML cannot carry.
    #[cfg(feature = "static_export")]
    fn write_svg(&self, path: &std::path::Path) -> Result<(), GraphError> {
        native::write_svg(&self.graph_data(), &self.graph_config(), path)
    }

    /// Show the plot in browser
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::Io`] when the page cannot be written to the
    /// system temp directory or the platform's opener (`open`, `xdg-open`,
    /// `explorer`) cannot be started, as on a headless server, and
    /// [`GraphError::Render`] on a platform with no default opener.
    #[cfg(feature = "plotly")]
    fn show(&self) -> Result<(), GraphError> {
        show_in_browser(&self.to_plot())
    }

    /// One‑stop rendering with error propagation.
    ///
    /// # Errors
    ///
    /// A PNG or SVG target returns whatever `Graph::write_png` or
    /// `Graph::write_svg` returns, unchanged; in a build without
    /// `static_export` it is `GraphError::Render`. HTML and browser targets
    /// return `GraphError::Render` when serialization fails and
    /// `GraphError::Io` when the destination cannot be written.
    #[cfg(feature = "plotly")]
    fn render(&self, output: OutputType) -> Result<(), GraphError> {
        match output {
            // The export's own error, typed, not folded into `Render`.
            #[cfg(feature = "static_export")]
            OutputType::Png(path) => self.write_png(path)?,
            #[cfg(feature = "static_export")]
            OutputType::Svg(path) => self.write_svg(path)?,
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
/// Writes `plot` as a standalone page in the system temp directory and hands
/// it to the platform's default opener.
///
/// `plotly`'s own `Plot::show` does the same, with the same page, but aborts
/// the process when the temp file cannot be written or the opener cannot be
/// started (a headless server has no `xdg-open`); here both come back as a
/// [`GraphError`].
#[cfg(feature = "plotly")]
#[inline(never)]
fn show_in_browser(plot: &Plot) -> Result<(), GraphError> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Process id, wall-clock nanoseconds and a per-process sequence number
    // keep two pages shown at once from overwriting each other; the
    // sequence only names files, so it is free to wrap.
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "plotly_optionstratlib_{}_{nanos}_{sequence}.html",
        std::process::id()
    ));
    std::fs::write(&path, plot.to_html())?;
    open_in_default_app(&path)
}

/// Opens `path` with the platform's default application, the way `plotly`
/// does: `open` on macOS, `xdg-open` on the other Unix systems and
/// `explorer` on Windows.
#[cfg(feature = "plotly")]
#[inline(never)]
fn open_in_default_app(path: &std::path::Path) -> Result<(), GraphError> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    Command::new("open").arg(path).output()?;
    #[cfg(all(unix, not(target_os = "macos")))]
    Command::new("xdg-open").arg(path).output()?;
    #[cfg(target_os = "windows")]
    Command::new("explorer").arg(path).spawn()?;
    #[cfg(not(any(unix, target_os = "windows")))]
    return Err(GraphError::Render(format!(
        "no default application to open {} on this platform",
        path.display()
    )));
    #[cfg(any(unix, target_os = "windows"))]
    Ok(())
}

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
