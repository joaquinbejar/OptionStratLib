use optionstratlib::visualization::{
    ColorScheme, Graph, GraphConfig, GraphData, LineStyle, Series2D, TraceMode,
};
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

fn create_test_series() -> GraphData {
    GraphData::Series(Series2D {
        x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
        y: vec![dec!(10.0), dec!(20.0), dec!(30.0)],
        name: "Test Series".to_string(),
        mode: TraceMode::Lines,
        line_color: Some("#FF0000".to_string()),
        line_width: Some(2.0),
    })
}

#[test]
fn test_graph_trait_default_config() {
    // Create a test graph with only data
    let data = create_test_series();
    let graph = TestGraph::new(data.clone());

    // Test that graph_data returns the expected data
    match graph.graph_data() {
        GraphData::Series(series) => {
            assert_eq!(series.x, vec![dec!(1.0), dec!(2.0), dec!(3.0)]);
            assert_eq!(series.y, vec![dec!(10.0), dec!(20.0), dec!(30.0)]);
            assert_eq!(series.name, "Test Series");
            assert_eq!(series.mode, TraceMode::Lines);
            assert_eq!(series.line_color, Some("#FF0000".to_string()));
            assert_eq!(series.line_width, Some(2.0));
        }
        _ => panic!("Expected Series graph data"),
    }

    // Test that graph_config returns the default config when none is specified
    let config = graph.graph_config();
    assert_eq!(config, GraphConfig::default());
}

#[test]
fn test_graph_trait_custom_config() {
    // Create a test graph with custom config
    let data = create_test_series();
    let custom_config = GraphConfig {
        title: "Custom Title".to_string(),
        width: 800,
        height: 600,
        x_label: Some("X Axis".to_string()),
        y_label: Some("Y Axis".to_string()),
        z_label: None,
        line_style: LineStyle::Dashed,
        color_scheme: ColorScheme::Viridis,
        legend: Some(vec!["Series 1".to_string()]),
        show_legend: true,
    };

    let graph = TestGraph::with_config(data, custom_config.clone());

    // Test that graph_config returns the custom config
    let config = graph.graph_config();
    assert_eq!(config.title, "Custom Title");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert_eq!(config.x_label, Some("X Axis".to_string()));
    assert_eq!(config.y_label, Some("Y Axis".to_string()));
    assert_eq!(config.z_label, None);
    assert_eq!(config.line_style, LineStyle::Dashed);
    assert_eq!(config.color_scheme, ColorScheme::Viridis);
    assert_eq!(config.legend, Some(vec!["Series 1".to_string()]));
    assert!(config.show_legend);
}

#[test]
fn test_graph_trait_multi_series() {
    // Create a test graph with multiple series
    let series1 = Series2D {
        x: vec![dec!(1.0), dec!(2.0)],
        y: vec![dec!(10.0), dec!(20.0)],
        name: "Series 1".to_string(),
        mode: TraceMode::Lines,
        line_color: Some("#FF0000".to_string()),
        line_width: Some(2.0),
    };

    let series2 = Series2D {
        x: vec![dec!(1.0), dec!(2.0)],
        y: vec![dec!(5.0), dec!(15.0)],
        name: "Series 2".to_string(),
        mode: TraceMode::Markers,
        line_color: Some("#00FF00".to_string()),
        line_width: Some(1.0),
    };

    let data = GraphData::MultiSeries(vec![series1.clone(), series2.clone()]);
    let graph = TestGraph::new(data);

    // Test that graph_data returns the expected multi-series data
    match graph.graph_data() {
        GraphData::MultiSeries(series) => {
            assert_eq!(series.len(), 2);
            assert_eq!(series[0], series1);
            assert_eq!(series[1], series2);
        }
        _ => panic!("Expected MultiSeries graph data"),
    }
}
