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

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use super::*;
    use crate::visualization::{ColorScheme, Series2D};
    use crate::visualization::config::GraphConfig;
    use crate::visualization::model::GraphData;
    use crate::visualization::LineStyle;

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
        assert_eq!(config.show_legend, true);
    }
}
