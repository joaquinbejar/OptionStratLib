use crate::visualization::config::GraphConfig;
use crate::visualization::model::GraphData;

/// A trait that defines the functionality for creating, configuring, and rendering
/// graphical representations of data, along with support for various output formats.
///
/// The `Graph` trait abstracts away details involved in preparing graph data, applying
/// configurations, and rendering graphs in different formats such as PNG, SVG, PDF, or
/// directly displaying them in an interactive terminal.
///
/// ## Required Methods
/// - `graph_data`: Returns the raw data structure (`GraphData`) required for plotting.
///
/// ## Optional Methods
/// - `graph_config`: Provides configuration overrides for the graph, including dimensions,
///   labels, legends, and other visual properties. Defaults to `GraphConfig::default()`.
///
/// ## Core Render Methods
/// - `to_gnuplot`: Converts the instance into a sequence of gnuplot commands based on its data and
///   configuration.
///
/// ## Export and Viewing Methods
/// - `write_png`: Exports the graph as a PNG image to a specified file.
/// - `write_html`: (For compatibility) Creates a self-contained HTML file with the image embedded.
/// - `write_svg`: Exports the graph as an SVG file.
/// - `write_pdf`: Exports the graph as a PDF file.
/// - `show`: Displays the graph interactively.
/// - `render`: A unified interface for rendering graphs in various formats.
///
/// ## Advanced Features
/// - `to_interactive_html`: (For compatibility) Creates an interactive HTML file with hover information
///   and annotations.
pub trait Graph {
    /// Return the raw data ready for plotting.
    fn graph_data(&self) -> GraphData;

    /// Optional per‑object configuration overrides.
    fn graph_config(&self) -> GraphConfig {
        GraphConfig::default()
    }
    
}

