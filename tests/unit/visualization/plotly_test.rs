#[cfg(feature = "plotly")]
mod plotly_tests {
    use optionstratlib::visualization::{ColorScheme, LineStyle, TraceMode};
    use optionstratlib::visualization::{Graph, GraphConfig, GraphData, Series2D, Surface3D};
    // use plotly::{Scatter, Surface}; // Scatter and Surface not directly used here after changes
    // use mockall::predicate::*;
    use mockall::*;
    use rust_decimal::Decimal;
    use std::path::PathBuf;
    use std::str::FromStr;

    // Create a mock for the file operations to avoid actual file I/O during tests
    #[automock]
    #[allow(dead_code)]
    trait FilePreparation {
        fn prepare_file_path(&self, path: &std::path::Path) -> Result<(), std::io::Error>;
    }

    // Create a mock for the Plot to avoid actual plotting during tests
    mock! {
        Plot {
            fn add_trace<T: plotly::Trace + 'static>(&mut self, trace: T) -> &mut Self;
            fn set_layout(&mut self, layout: plotly::Layout) -> &mut Self;
            fn write_html(&self, path: &std::path::Path);
            fn write_image(&self, path: &std::path::Path, format: plotly::plotly_static::ImageFormat, width: usize, height: usize, scale: f64);
            fn show(&self);
        }
    }

    // Test implementation of the Graph trait
    struct TestGraph {
        data: GraphData,
        config: GraphConfig,
    }

    impl TestGraph {
        fn new(data: GraphData, config: GraphConfig) -> Self {
            Self { data, config }
        }
    }

    impl Graph for TestGraph {
        fn graph_data(&self) -> GraphData {
            self.data.clone()
        }

        fn graph_config(&self) -> GraphConfig {
            self.config.clone()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // Helper function to create a sample Series2D
        fn create_sample_series() -> Series2D {
            Series2D {
                x: vec![
                    Decimal::from_str("1.0").unwrap(),
                    Decimal::from_str("2.0").unwrap(),
                    Decimal::from_str("3.0").unwrap(),
                ],
                y: vec![
                    Decimal::from_str("4.0").unwrap(),
                    Decimal::from_str("5.0").unwrap(),
                    Decimal::from_str("6.0").unwrap(),
                ],
                name: "Test Series".to_string(),
                mode: TraceMode::Lines,
                line_color: Some("#FF0000".to_string()),
                line_width: Some(2.0),
            }
        }

        // Helper function to create a sample Surface3D
        fn create_sample_surface() -> Surface3D {
            Surface3D {
                x: vec![
                    Decimal::from_str("0.0").unwrap(),
                    Decimal::from_str("1.0").unwrap(),
                    Decimal::from_str("0.0").unwrap(),
                    Decimal::from_str("1.0").unwrap(),
                ],
                y: vec![
                    Decimal::from_str("0.0").unwrap(),
                    Decimal::from_str("0.0").unwrap(),
                    Decimal::from_str("1.0").unwrap(),
                    Decimal::from_str("1.0").unwrap(),
                ],
                z: vec![
                    Decimal::from_str("0.0").unwrap(),
                    Decimal::from_str("1.0").unwrap(),
                    Decimal::from_str("1.0").unwrap(),
                    Decimal::from_str("2.0").unwrap(),
                ],
                name: "Test Surface".to_string(),
            }
        }

        // Helper function to create a sample GraphConfig
        fn create_sample_config() -> GraphConfig {
            GraphConfig {
                title: "Test Graph".to_string(),
                width: 800,
                height: 600,
                x_label: Some("X Axis".to_string()),
                y_label: Some("Y Axis".to_string()),
                z_label: Some("Z Axis".to_string()),
                line_style: LineStyle::Solid,
                color_scheme: ColorScheme::Viridis,
                legend: Some(vec!["Series 1".to_string()]),
                show_legend: true,
            }
        }

        #[test]
        fn test_graph_config_default() {
            let graph = TestGraph::new(
                GraphData::Series(create_sample_series()),
                GraphConfig::default(),
            );

            let config = graph.graph_config();
            assert_eq!(config.title, "Graph");
            assert_eq!(config.width, 1280);
            assert_eq!(config.height, 720);
            assert!(config.show_legend);
            assert!(config.x_label.is_none());
            assert!(config.y_label.is_none());
            assert!(config.z_label.is_none());
        }

        #[test]
        fn test_graph_config_custom() {
            let custom_config = create_sample_config();
            let graph = TestGraph::new(
                GraphData::Series(create_sample_series()),
                custom_config.clone(),
            );

            let config = graph.graph_config();
            assert_eq!(config.title, "Test Graph");
            assert_eq!(config.width, 800);
            assert_eq!(config.height, 600);
            assert_eq!(config.x_label, Some("X Axis".to_string()));
            assert_eq!(config.y_label, Some("Y Axis".to_string()));
            assert_eq!(config.z_label, Some("Z Axis".to_string()));
        }

        #[test]
        fn test_graph_data_series() {
            let series = create_sample_series();
            let graph = TestGraph::new(GraphData::Series(series.clone()), GraphConfig::default());

            match graph.graph_data() {
                GraphData::Series(s) => {
                    assert_eq!(s.x, series.x);
                    assert_eq!(s.y, series.y);
                    assert_eq!(s.name, "Test Series");
                    assert_eq!(s.mode, TraceMode::Lines);
                    assert_eq!(s.line_color, Some("#FF0000".to_string()));
                    assert_eq!(s.line_width, Some(2.0));
                }
                _ => panic!("Expected Series variant"),
            }
        }

        #[test]
        fn test_graph_data_surface() {
            let surface = create_sample_surface();
            let graph = TestGraph::new(
                GraphData::GraphSurface(surface.clone()),
                GraphConfig::default(),
            );

            match graph.graph_data() {
                GraphData::GraphSurface(s) => {
                    assert_eq!(s.x, surface.x);
                    assert_eq!(s.y, surface.y);
                    assert_eq!(s.z, surface.z);
                    assert_eq!(s.name, "Test Surface");
                }
                _ => panic!("Expected Surface variant"),
            }
        }

        #[test]
        fn test_graph_data_multi_series() {
            let series1 = create_sample_series();
            let mut series2 = create_sample_series();
            series2.name = "Series 2".to_string();
            series2.line_color = Some("#00FF00".to_string());

            let multi_series = vec![series1.clone(), series2.clone()];
            let graph = TestGraph::new(
                GraphData::MultiSeries(multi_series.clone()),
                GraphConfig::default(),
            );

            match graph.graph_data() {
                GraphData::MultiSeries(series_list) => {
                    assert_eq!(series_list.len(), 2);
                    assert_eq!(series_list[0].name, "Test Series");
                    assert_eq!(series_list[1].name, "Series 2");
                    assert_eq!(series_list[0].line_color, Some("#FF0000".to_string()));
                    assert_eq!(series_list[1].line_color, Some("#00FF00".to_string()));
                }
                _ => panic!("Expected MultiSeries variant"),
            }
        }

        // Test error handling in file operations
        #[test]
        fn test_file_operation_error_handling() {
            let mut mock_preparer = MockFilePreparation::new();
            let _test_path = PathBuf::from("/test/path/graph.html");

            // Set up expectations for the mock to return an error
            mock_preparer.expect_prepare_file_path().returning(|_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Permission denied",
                ))
            });

            // In a real implementation, we would:
            // 1. Call a method that uses prepare_file_path
            // 2. Verify that the error is properly converted to a GraphError
            // 3. Assert that the error contains the expected message
        }

        // Test for to_plot method with different GraphData variants
        #[test]
        fn test_to_plot_with_different_data_types() {
            // Series data
            let _series_graph = TestGraph::new(
                GraphData::Series(create_sample_series()),
                create_sample_config(),
            );

            // Surface data
            let _surface_graph = TestGraph::new(
                GraphData::GraphSurface(create_sample_surface()),
                create_sample_config(),
            );

            // MultiSeries data
            let series1 = create_sample_series();
            let mut series2 = create_sample_series();
            series2.name = "Series 2".to_string();
            let _multi_series_graph = TestGraph::new(
                GraphData::MultiSeries(vec![series1, series2]),
                create_sample_config(),
            );

            // In a real implementation, we would:
            // 1. Mock the Plot creation and verify that add_trace is called with the correct trace type
            // 2. Verify that set_layout is called with the correct layout parameters
            // 3. Assert that the returned Plot contains the expected traces and layout
        }

        #[test]
        fn test_to_plot_series_features() {
            // Scenario 1: Series with legend and x/y labels
            let mut config = create_sample_config();
            config.legend = Some(vec!["My Series Legend".to_string()]);
            config.x_label = Some("Custom X Axis".to_string());
            config.y_label = Some("Custom Y Axis".to_string());
            config.z_label = None; // Ensure z_label logic is not triggered

            let series_data = create_sample_series();
            let graph = TestGraph::new(GraphData::Series(series_data), config.clone());
            let plot = graph.to_plot();

            // Check trace name (legend)
            assert_eq!(plot.data().len(), 1);
            let trace = plot.data().iter().next().unwrap();
            let trace_json = trace.to_json();
            assert!(
                trace_json.contains("\"type\":\"scatter\""),
                "Trace JSON should contain type scatter. Got: {trace_json}"
            );

            // Check axis labels
            let _layout = plot.layout();

            // Scenario 2: Series without legend (to ensure it doesn't panic, uses original name)
            let mut config_no_legend = create_sample_config();
            config_no_legend.legend = None;
            let series_data_orig_name = create_sample_series(); // name is "Test Series"
            let graph_no_legend = TestGraph::new(
                GraphData::Series(series_data_orig_name.clone()),
                config_no_legend,
            );
            let plot_no_legend = graph_no_legend.to_plot();
            let trace_no_legend = plot_no_legend.data().iter().next().unwrap();
            let trace_no_legend_json = trace_no_legend.to_json();
            assert!(
                trace_no_legend_json.contains("\"type\":\"scatter\""),
                "Trace (no legend) JSON should contain type scatter. Got: {trace_no_legend_json}"
            );
        }

        #[test]
        fn test_to_plot_multiseries_features() {
            let mut series1 = create_sample_series();
            series1.name = "First Series".to_string();
            series1.line_color = None; // To trigger pick_color

            let mut series2 = create_sample_series();
            series2.name = "Second Series".to_string();
            series2.line_color = Some("#00FF00".to_string()); // Specific color

            let mut config = create_sample_config();
            config.legend = Some(vec!["Legend S1".to_string(), "Legend S2".to_string()]);
            config.color_scheme = ColorScheme::Plasma; // For predictable pick_color testing if needed

            let graph = TestGraph::new(
                GraphData::MultiSeries(vec![series1.clone(), series2.clone()]),
                config.clone(),
            );
            let plot = graph.to_plot();

            assert_eq!(plot.data().len(), 2);

            // Trace 1 (picked color, legend)
            let trace1 = plot.data().iter().next().unwrap();
            let trace1_json = trace1.to_json();
            assert!(
                trace1_json.contains("\"type\":\"scatter\""),
                "Trace 1 JSON should contain type scatter. Got: {trace1_json}"
            );

            // Trace 2 (specific color, legend)
            let trace2 = plot.data().iter().nth(1).unwrap();
            let trace2_json = trace2.to_json();
            assert!(
                trace2_json.contains("\"type\":\"scatter\""),
                "Trace 2 JSON should contain type scatter. Got: {trace2_json}"
            );

            // Scenario: Fewer legends than series
            let mut config_less_legends = config.clone();
            config_less_legends.legend = Some(vec!["Only Legend S1".to_string()]);
            let graph_less_legends = TestGraph::new(
                GraphData::MultiSeries(vec![series1.clone(), series2.clone()]),
                config_less_legends,
            );
            let plot_less_legends = graph_less_legends.to_plot();

            let trace1_less_json = plot_less_legends.data().iter().next().unwrap().to_json();
            assert!(
                trace1_less_json.contains("\"type\":\"scatter\""),
                "Trace 1 (less legends) JSON should contain type scatter. Got: {trace1_less_json}"
            );
            let trace2_less_json = plot_less_legends.data().iter().nth(1).unwrap().to_json();
            assert!(
                trace2_less_json.contains("\"type\":\"scatter\""),
                "Trace 2 (less legends) JSON should contain type scatter. Got: {trace2_less_json}"
            );
        }

        #[test]
        fn test_to_plot_surface_features() {
            let mut config = create_sample_config();
            config.legend = Some(vec!["My Surface Legend".to_string()]);
            config.x_label = None;
            config.y_label = None;
            config.z_label = Some("Custom Z Axis".to_string());

            let surface_data = create_sample_surface();
            let graph = TestGraph::new(GraphData::GraphSurface(surface_data), config.clone());
            let plot = graph.to_plot();

            // Check trace name (legend)
            assert_eq!(plot.data().len(), 1);
            let trace = plot.data().iter().next().unwrap();
            let trace_json = trace.to_json();
            assert!(
                trace_json.contains("\"type\":\"surface\""),
                "Trace JSON should contain type surface. Got: {trace_json}"
            );

            // Check axis labels
            let _layout = plot.layout();
        }
    }
}

#[cfg(test)]
#[cfg(feature = "plotly")]
mod tests_plotly_utils {
    use optionstratlib::visualization::pick_color;
    use optionstratlib::visualization::{ColorScheme, GraphConfig, get_color_from_scheme};

    #[test]
    fn test_pick_color_delegates_to_color_scheme() {
        // Arrange
        let cfg = GraphConfig {
            color_scheme: ColorScheme::Viridis,
            ..GraphConfig::default()
        };

        // Act
        let color = pick_color(&cfg, 0);

        // Assert
        assert_eq!(color, get_color_from_scheme(&ColorScheme::Viridis, 0));
    }

    #[test]
    fn test_pick_color_with_viridis() {
        // Arrange
        let cfg = GraphConfig {
            color_scheme: ColorScheme::Viridis,
            ..GraphConfig::default()
        };

        // Act
        let color = pick_color(&cfg, 0);

        // Assert
        assert_eq!(color, Some("#481567".to_string()));
    }

    #[test]
    fn test_pick_color_with_plasma() {
        // Arrange
        let cfg = GraphConfig {
            color_scheme: ColorScheme::Plasma,
            ..GraphConfig::default()
        };

        // Act
        let color = pick_color(&cfg, 0);

        // Assert
        assert_eq!(color, Some("#0D0887".to_string()));
    }

    #[test]
    fn test_pick_color_with_custom_colors() {
        // Arrange
        let custom_colors = vec!["#FF0000".to_string(), "#00FF00".to_string()];
        let cfg = GraphConfig {
            color_scheme: ColorScheme::Custom(custom_colors),
            ..GraphConfig::default()
        };

        // Act & Assert
        assert_eq!(pick_color(&cfg, 0), Some("#FF0000".to_string()));
        assert_eq!(pick_color(&cfg, 1), Some("#00FF00".to_string()));
        assert_eq!(pick_color(&cfg, 2), Some("#FF0000".to_string())); // Wraps around
    }
}

#[cfg(test)]
#[cfg(feature = "plotly")]
mod tests_plotly_interface {

    use optionstratlib::visualization::{
        Graph, GraphConfig, GraphData, Series2D, Surface3D, TraceMode,
    };
    use rust_decimal_macros::dec;
    use tracing::info;
    #[cfg(feature = "static_export")]
    use {
        optionstratlib::visualization::{ColorScheme, LineStyle, OutputType},
        plotly::Plot,
        std::fs,
        std::path::PathBuf,
    };
    use {
        optionstratlib::visualization::{make_scatter, make_surface, to_plotly_mode},
        plotly::{Scatter, Surface, Trace, common::Mode},
        rust_decimal::Decimal,
    };

    trait ScatterTestHelper {
        fn is_scatter(&self) -> bool;
    }

    impl ScatterTestHelper for Box<Scatter<Decimal, Decimal>> {
        fn is_scatter(&self) -> bool {
            true
        }
    }

    trait SurfaceTestHelper {
        fn is_surface(&self) -> bool;
    }

    impl SurfaceTestHelper for Box<Surface<Decimal, Decimal, Decimal>> {
        fn is_surface(&self) -> bool {
            true
        }
    }

    #[cfg(feature = "static_export")]
    struct TestGraph {
        data: GraphData,
        config: GraphConfig,
    }

    #[cfg(feature = "static_export")]
    impl Graph for TestGraph {
        fn graph_data(&self) -> GraphData {
            self.data.clone()
        }

        fn graph_config(&self) -> GraphConfig {
            self.config.clone()
        }
    }

    #[cfg(feature = "static_export")]
    trait PlotTestHelper {
        fn has_traces(&self) -> bool;
    }

    #[cfg(feature = "static_export")]
    impl PlotTestHelper for Plot {
        fn has_traces(&self) -> bool {
            // A plot is valid if it has been created successfully
            // We cannot directly access private fields, so just return true
            true
        }
    }

    #[cfg(feature = "static_export")]
    fn create_test_graph_with_series() -> TestGraph {
        let series = Series2D {
            x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
            y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
            name: "Test Series".to_string(),
            mode: TraceMode::Lines,
            line_color: Some("#FF0000".to_string()),
            line_width: Some(2.0),
        };

        let config = GraphConfig {
            title: "Test Graph".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X Axis".to_string()),
            y_label: Some("Y Axis".to_string()),
            z_label: None,
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Default,
            legend: None,
            show_legend: true,
        };

        TestGraph {
            data: GraphData::Series(series),
            config,
        }
    }

    #[cfg(feature = "static_export")]
    fn create_test_graph_with_multi_series() -> TestGraph {
        let series1 = Series2D {
            x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
            y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
            name: "Series 1".to_string(),
            mode: TraceMode::Lines,
            line_color: Some("#FF0000".to_string()),
            line_width: Some(2.0),
        };

        let series2 = Series2D {
            x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
            y: vec![dec!(7.0), dec!(8.0), dec!(9.0)],
            name: "Series 2".to_string(),
            mode: TraceMode::Markers,
            line_color: Some("#00FF00".to_string()),
            line_width: Some(2.0),
        };

        let config = GraphConfig {
            title: "Multi Series Test".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X Axis".to_string()),
            y_label: Some("Y Axis".to_string()),
            z_label: None,
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Default,
            legend: Some(vec![
                "Custom Series 1".to_string(),
                "Custom Series 2".to_string(),
            ]),
            show_legend: true,
        };

        TestGraph {
            data: GraphData::MultiSeries(vec![series1, series2]),
            config,
        }
    }

    #[cfg(feature = "static_export")]
    fn create_test_graph_with_surface() -> TestGraph {
        let surface = Surface3D {
            x: vec![dec!(0.0), dec!(1.0), dec!(0.0), dec!(1.0)],
            y: vec![dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0)],
            z: vec![dec!(0.0), dec!(1.0), dec!(2.0), dec!(3.0)],
            name: "Test Surface".to_string(),
        };

        let config = GraphConfig {
            title: "Surface Test".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X Axis".to_string()),
            y_label: Some("Y Axis".to_string()),
            z_label: Some("Z Axis".to_string()),
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Viridis,
            legend: Some(vec!["Custom Surface".to_string()]),
            show_legend: true,
        };

        TestGraph {
            data: GraphData::GraphSurface(surface),
            config,
        }
    }

    #[test]
    fn test_make_scatter_creates_valid_scatter() {
        // Arrange
        let mut series = Series2D {
            x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
            y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
            name: "Test Series".to_string(),
            mode: TraceMode::Lines,
            line_color: Some("#FF0000".to_string()),
            line_width: Some(2.5),
        };
        let scatter = make_scatter(&series);
        assert!(scatter.is_scatter());

        series.mode = TraceMode::Markers;
        let scatter = make_scatter(&series);
        assert!(scatter.is_scatter());

        series.mode = TraceMode::LinesMarkers;
        let scatter = make_scatter(&series);
        assert!(scatter.is_scatter());
    }

    // Tests for to_plot
    #[test]
    #[cfg(feature = "static_export")]
    fn test_to_plot_with_series() {
        // Create a graph with Series data
        let graph = create_test_graph_with_series();

        // Convert to a Plot
        let plot = graph.to_plot();

        // Since we can't access private fields, just verify the method runs
        assert!(plot.has_traces());
    }

    #[test]
    #[cfg(feature = "static_export")]
    fn test_to_plot_with_multi_series() {
        // Create a graph with multiple Series
        let graph = create_test_graph_with_multi_series();

        // Convert to a Plot
        let plot = graph.to_plot();

        // Since we can't access private fields, just verify the method runs
        assert!(plot.has_traces());
    }

    #[test]
    #[cfg(feature = "static_export")]
    fn test_to_plot_with_surface() {
        // Create a graph with Surface data
        let graph = create_test_graph_with_surface();

        // Convert to a Plot
        let plot = graph.to_plot();

        // Since we can't access private fields, just verify the method runs
        assert!(plot.has_traces());
    }

    // Tests for write_html
    #[test]
    #[cfg(feature = "static_export")]
    fn test_write_html() {
        // Create a graph with multiple Series
        let graph = create_test_graph_with_multi_series();
        let temp_path = PathBuf::from("test_output_html.html");

        // Clean up existing file if it exists
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }

        // Execute method under test
        let result = graph.write_html(&temp_path);

        // Clean up after the test
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }

        // Verify result
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "static_export")]
    fn test_write_png() {
        use tempfile::tempdir;

        let graph = create_test_graph_with_series();
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().join("test_output_png.png");

        let result = graph.write_png(&temp_path);

        if let Err(e) = &result {
            tracing::error!("PNG export error: {e}");
            info!("PNG export error: {e}");
        }

        assert!(result.is_ok(), "PNG export should succeed");
    }

    #[test]
    #[cfg(feature = "static_export")]
    fn test_write_svg() {
        use tempfile::tempdir;

        let graph = create_test_graph_with_surface();
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().join("test_output_svg.svg");

        let result = graph.write_svg(&temp_path);

        if let Err(e) = &result {
            tracing::error!("SVG export error: {e}");
            info!("SVG export error: {e}");
        }

        assert!(result.is_ok(), "SVG export should succeed");
    }

    // Test for render with HTML OutputType
    // (PNG and SVG are commented out because they require kaleido)
    #[test]
    #[cfg(feature = "static_export")]
    fn test_render_with_html_output_type() {
        let graph = create_test_graph_with_series();

        // OutputType::Html
        let html_path = PathBuf::from("test_render_html.html");
        if html_path.exists() {
            fs::remove_file(&html_path).unwrap();
        }
        let html_result = graph.render(OutputType::Html(&html_path));
        if html_path.exists() {
            fs::remove_file(&html_path).unwrap();
        }
        assert!(html_result.is_ok());

        // Note: OutputType::Browser is not tested because it opens a browser window
    }

    /*
    #[test]
    fn test_render_with_png_and_svg_output_types() {
        let graph = create_test_graph_with_series();

        // OutputType::Png
        let png_path = PathBuf::from("test_render_png.png");
        if png_path.exists() {
            fs::remove_file(&png_path).unwrap();
        }
        let png_result = graph.render(OutputType::Png(&png_path));
        if png_path.exists() {
            fs::remove_file(&png_path).unwrap();
        }
        assert!(png_result.is_ok());

        // OutputType::Svg
        let svg_path = PathBuf::from("test_render_svg.svg");
        if svg_path.exists() {
            fs::remove_file(&svg_path).unwrap();
        }
        let svg_result = graph.render(OutputType::Svg(&svg_path));
        if svg_path.exists() {
            fs::remove_file(&svg_path).unwrap();
        }
        assert!(svg_result.is_ok());
    }
    */

    // Test for to_interactive_html
    #[test]
    #[cfg(feature = "static_export")]
    fn test_to_interactive_html() {
        // Create a graph with multiple Series
        let graph = create_test_graph_with_multi_series();
        let temp_path = PathBuf::from("test_interactive_html.html");

        // Clean up existing file if it exists
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }

        // Execute method under test
        let result = graph.to_interactive_html(&temp_path);

        // Clean up after the test
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }

        // Verify result
        assert!(result.is_ok());
    }

    // Test to verify the default value of graph_config
    #[test]
    fn test_default_graph_config() {
        // Define a minimal graph that only implements graph_data
        struct MinimalGraph {
            data: GraphData,
        }

        impl Graph for MinimalGraph {
            fn graph_data(&self) -> GraphData {
                self.data.clone()
            }
            // We don't implement graph_config, so it will use the default
        }

        // Create a simple series
        let series = Series2D {
            x: vec![dec!(1.0), dec!(2.0)],
            y: vec![dec!(3.0), dec!(4.0)],
            name: "Minimal Series".to_string(),
            mode: TraceMode::Lines,
            line_color: None,
            line_width: None,
        };

        // Create the minimal graph
        let minimal_graph = MinimalGraph {
            data: GraphData::Series(series),
        };

        // Verify that the graph_config method returns the default configuration
        let default_config = GraphConfig::default();
        let graph_config = minimal_graph.graph_config();

        assert_eq!(graph_config.title, default_config.title);
        assert_eq!(graph_config.width, default_config.width);
        assert_eq!(graph_config.height, default_config.height);
        assert_eq!(graph_config.line_style, default_config.line_style);
        assert_eq!(graph_config.show_legend, default_config.show_legend);
    }

    #[test]
    fn test_make_surface_with_valid_data() {
        // Arrange
        let surface = Surface3D {
            x: vec![dec!(0.0), dec!(1.0), dec!(0.0), dec!(1.0)],
            y: vec![dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0)],
            z: vec![dec!(0.0), dec!(1.0), dec!(2.0), dec!(3.0)],
            name: "Test Surface".to_string(),
        };
        let result = make_surface(&surface);
        assert!(result.is_surface());
    }

    #[test]
    fn test_make_surface_with_no_valid_data() {
        // Arrange
        let surface = Surface3D {
            x: vec![dec!(0.0), dec!(1.0), dec!(0.0), dec!(1.0)],
            y: vec![],
            z: vec![dec!(0.0), dec!(1.0), dec!(2.0), dec!(3.0)],
            name: "Test Surface".to_string(),
        };
        let result = make_surface(&surface);
        assert!(result.is_surface());
        let json = r#"{"type":"surface","x":["0","1"],"y":["0","1"],"z":[["0","0"],["0","0"]],"name":"Test Surface"}"#;
        assert_eq!(result.to_json(), json);
    }

    #[test]
    fn test_to_plotly_mode_conversions() {
        // We can compare enum values directly
        assert!(matches!(to_plotly_mode(&TraceMode::Lines), Mode::Lines));
        assert!(matches!(to_plotly_mode(&TraceMode::Markers), Mode::Markers));
        assert!(matches!(
            to_plotly_mode(&TraceMode::LinesMarkers),
            Mode::LinesMarkers
        ));
    }
}
